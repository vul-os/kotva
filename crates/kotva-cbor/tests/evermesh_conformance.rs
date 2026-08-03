//! Replay of evermesh's canonical-CBOR conformance corpus against `kotva-cbor`.
//!
//! `kotva-cbor` was seeded from `evermesh-kernel::codec`, whose module docs state
//! cross-implementation byte identity as **consensus-critical**. This test is what
//! keeps that inheritance honest: the corpus in
//! `tests/vectors/evermesh-canonical-cbor.txt` is extracted from evermesh's 189
//! conformance vectors, and a divergence between the two codecs shows up here as a
//! failure rather than as a signature that stops verifying in production.
//!
//! It deliberately does **not** depend on evermesh. The corpus is a frozen
//! snapshot with its source revision recorded in the file header; a live mirror
//! would silently track upstream and so could never catch the drift it exists to
//! catch.
//!
//! Every assertion below counts itself. A corpus file that went missing, got
//! truncated, or quietly stopped matching any line would otherwise let this test
//! report success having checked nothing — the failure mode kotva's own CI notes
//! call out by name.

use kotva_cbor::{decode_canonical, encode_canonical, CborError, Value};

const CORPUS: &str = include_str!("vectors/evermesh-canonical-cbor.txt");

/// Exact counts the corpus must contain. Hard-coded, so adding or removing a
/// vector fails this test until the number is updated in the same commit.
const EXPECT_ACCEPT: usize = 153;
const EXPECT_REJECT: usize = 30;

fn unhex(s: &str) -> Vec<u8> {
    assert!(s.len() % 2 == 0, "odd-length hex: {s}");
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex digit"))
        .collect()
}

struct Vector {
    accept: bool,
    label: String,
    bytes: Vec<u8>,
}

fn corpus() -> Vec<Vector> {
    let mut out = Vec::new();
    for (lineno, line) in CORPUS.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut f = line.split_whitespace();
        let verdict = f.next().unwrap_or_default();
        let label = f.next().unwrap_or_default().to_string();
        let hex = f.next().unwrap_or_default();
        assert!(
            f.next().is_none(),
            "line {}: expected exactly 3 fields",
            lineno + 1
        );
        assert!(
            !label.is_empty() && !hex.is_empty(),
            "line {}: malformed vector",
            lineno + 1
        );
        let accept = match verdict {
            "accept" => true,
            "reject" => false,
            other => panic!("line {}: unknown verdict {other:?}", lineno + 1),
        };
        out.push(Vector {
            accept,
            label,
            bytes: unhex(hex),
        });
    }
    out
}

#[test]
fn corpus_is_present_and_the_expected_size() {
    let vs = corpus();
    let accept = vs.iter().filter(|v| v.accept).count();
    let reject = vs.len() - accept;
    assert_eq!(
        (accept, reject),
        (EXPECT_ACCEPT, EXPECT_REJECT),
        "corpus size moved: {accept} accept / {reject} reject. If a vector was \
         added or removed on purpose, update EXPECT_ACCEPT/EXPECT_REJECT in the \
         same commit — a silently shrinking corpus reads exactly like a passing one."
    );
}

/// The malleability guarantee, on real signed-record bytes: every canonical byte
/// string decodes, and re-encodes to *itself*. If `kotva-cbor` had inherited even
/// one head-width or key-ordering difference from evermesh, some record here
/// would re-encode to different bytes and its content address would change.
#[test]
fn every_canonical_vector_re_encodes_to_itself() {
    let mut checked = 0usize;
    for v in corpus().iter().filter(|v| v.accept) {
        let value = decode_canonical(&v.bytes)
            .unwrap_or_else(|e| panic!("{}: canonical bytes were REJECTED: {e}", v.label));
        let again = encode_canonical(&value)
            .unwrap_or_else(|e| panic!("{}: decoded value would not re-encode: {e}", v.label));
        assert_eq!(
            hex(&again),
            hex(&v.bytes),
            "{}: re-encoding produced different bytes — a byte-level divergence \
             from evermesh's codec, i.e. a broken content address",
            v.label
        );
        checked += 1;
    }
    assert_eq!(
        checked, EXPECT_ACCEPT,
        "not every accept vector was checked"
    );
}

/// Every vector evermesh classifies as `cbor` or `non-canonical` must be refused
/// here too, for a reason drawn from the canonical rules rather than by accident.
#[test]
fn every_non_canonical_vector_is_refused() {
    let mut checked = 0usize;
    for v in corpus().iter().filter(|v| !v.accept) {
        let err = match decode_canonical(&v.bytes) {
            Ok(_) => panic!(
                "{}: non-canonical bytes were ACCEPTED — the decoder normalizes \
                 where it must refuse",
                v.label
            ),
            Err(e) => e,
        };
        assert!(
            !matches!(err, CborError::Json(_)),
            "{}: refused for a JSON reason, which the CBOR decoder cannot produce",
            v.label
        );
        checked += 1;
    }
    assert_eq!(
        checked, EXPECT_REJECT,
        "not every reject vector was checked"
    );
}

/// A decoded value's in-memory form must already be in canonical order, so a
/// consumer comparing a decoded value against a hand-built one is comparing like
/// with like.
#[test]
fn decoded_values_are_already_in_canonical_order() {
    let mut checked = 0usize;
    for v in corpus().iter().filter(|v| v.accept) {
        let value = decode_canonical(&v.bytes).unwrap();
        assert_eq!(
            value.clone().into_canonical(),
            value,
            "{}: decode produced a map whose entries were not in canonical order",
            v.label
        );
        checked += 1;
    }
    assert_eq!(checked, EXPECT_ACCEPT);
}

/// The corpus is real protocol data, not toy scalars: assert it actually exercises
/// nesting, both map key majors, and multi-byte heads. A corpus of 183 `0x00`s
/// would pass every test above.
#[test]
fn corpus_exercises_the_shapes_it_claims_to() {
    let mut int_keys = 0usize;
    let mut text_keys = 0usize;
    let mut deep = 0usize;
    let mut wide_heads = 0usize;
    let mut byte_strings = 0usize;

    fn walk(
        v: &Value,
        depth: u32,
        int_keys: &mut usize,
        text_keys: &mut usize,
        deep: &mut usize,
        wide_heads: &mut usize,
        byte_strings: &mut usize,
    ) {
        if depth >= 3 {
            *deep += 1;
        }
        match v {
            Value::Uint(n) if *n > 0xff => *wide_heads += 1,
            Value::Bytes(b) => {
                *byte_strings += 1;
                if b.len() > 23 {
                    *wide_heads += 1;
                }
            }
            Value::Array(items) => {
                for i in items {
                    walk(
                        i,
                        depth + 1,
                        int_keys,
                        text_keys,
                        deep,
                        wide_heads,
                        byte_strings,
                    );
                }
            }
            Value::Map(entries) => {
                for (k, val) in entries {
                    match k {
                        Value::Uint(_) => *int_keys += 1,
                        Value::Text(_) => *text_keys += 1,
                        _ => {}
                    }
                    walk(
                        val,
                        depth + 1,
                        int_keys,
                        text_keys,
                        deep,
                        wide_heads,
                        byte_strings,
                    );
                }
            }
            _ => {}
        }
    }

    for v in corpus().iter().filter(|v| v.accept) {
        let value = decode_canonical(&v.bytes).unwrap();
        walk(
            &value,
            0,
            &mut int_keys,
            &mut text_keys,
            &mut deep,
            &mut wide_heads,
            &mut byte_strings,
        );
    }
    assert!(int_keys > 500, "integer map keys seen: {int_keys}");
    assert!(text_keys > 100, "text map keys seen: {text_keys}");
    assert!(deep > 100, "values at depth >= 3 seen: {deep}");
    assert!(wide_heads > 300, "multi-byte heads seen: {wide_heads}");
    assert!(byte_strings > 300, "byte strings seen: {byte_strings}");
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
