//! §18.8a conformance vectors — **hand-derived from the CDDL in `18-wire-format.md`**, byte by
//! byte, and deliberately *not* produced by this crate's encoder.
//!
//! # Why these exist and why they are hand-written
//!
//! The unit tests in `src/` are round-trips: `decode(encode(x)) == x`. A round-trip proves the
//! encoder and decoder agree **with each other**, and would pass unchanged if both were wrong in
//! the same way — a wrong major type, a wrong key number, a non-canonical map order. It cannot
//! detect a systematic disagreement with the spec, because the spec never enters the loop. That is
//! not hypothetical here: §18.8a's objects existed for several revisions with exactly one
//! implementation, so "conformant" and "agrees with that implementation" were the same sentence.
//!
//! Every byte below was written by reading the CDDL and RFC 8949, then checked against this
//! implementation. Each is annotated with its full decomposition so a second implementation — in
//! any language — can be checked against the same corpus without running this crate.
//!
//! Signatures in these vectors are **filler bytes of the correct §18.2 length**, not real
//! signatures: the vectors pin the *encoding*, and an Ed25519 signature cannot be hand-derived. The
//! signing preimage rule is pinned separately, by `src/sig.rs`'s cross-DS-tag and
//! classical-vs-composite representative tests and by every `verify()` test in `src/`.

use kotva_coordinator::{
    AssuranceLevel, CoordinatorDescriptor, CoordinatorError, CoordinatorKind, DetCbor, Tariff,
    UsageReceipt, Visibility, VisibilityClass,
};
use kotva_core::cbor::Cv;
use kotva_core::Suite;

/// Decode a hex string into bytes. Vectors are written as hex so the byte layout is visible in
/// review rather than hidden behind a builder.
fn hex(s: &str) -> Vec<u8> {
    assert!(s.len() % 2 == 0, "odd-length hex vector");
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("bad hex"))
        .collect()
}

// ── Filler crypto fields ───────────────────────────────────────────────────────────────────────
//
// §18.2 fixes `ik-pub` at 32 B and `sig-val` at 64 B under suite `0x01`. These are uniform-byte
// fillers of exactly those lengths; `filler_lengths_are_what_18_2_requires` below asserts both the
// length and the uniformity, so a transcription slip in the long runs below fails immediately and
// loudly rather than shifting every subsequent byte of a vector.

/// 32 B of `0xd1` — the descriptor's `ik-pub`.
const IK_DESCRIPTOR: &str = "d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1";
/// 32 B of `0x7a` — the tariff's `ik-pub`. **Deliberately different from `IK_DESCRIPTOR`**: §18.8a.1
/// says a `Tariff` "MAY, but need not, equal the enclosing `CoordinatorDescriptor.identity`", and a
/// corpus that always used the same key could not tell a conformant decoder from one that quietly
/// assumed they matched.
const IK_TARIFF: &str = "7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a";
/// 32 B of `0x3c` — the usage receipt's `ik-pub`.
const IK_RECEIPT: &str = "3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c";

/// 64 B of `0xd5` — the descriptor's `sig-val`.
const SIG_DESCRIPTOR: &str = concat!(
    "d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5",
    "d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5",
);
/// 64 B of `0x75` — the tariff's `sig-val`.
const SIG_TARIFF: &str = concat!(
    "7575757575757575757575757575757575757575757575757575757575757575",
    "7575757575757575757575757575757575757575757575757575757575757575",
);
/// 64 B of `0x35` — the usage receipt's `sig-val`.
const SIG_RECEIPT: &str = concat!(
    "3535353535353535353535353535353535353535353535353535353535353535",
    "3535353535353535353535353535353535353535353535353535353535353535",
);

// ── The vectors ────────────────────────────────────────────────────────────────────────────────

/// `UsageReceipt { suite: 0x01, identity: 32×d1…, operation: det_cbor({1: 7}), sig: 64×35 }`
/// (§18.8a.2).
///
/// ```text
/// a4                        map(4)
///   01 01                   1 : unsigned(1)              ; suite = 0x01
///   02 58 20 <32 B>         2 : bytes(32)                ; ik-pub — §18.2 row for suite 0x01
///   03 43 a1 01 07          3 : bytes(3)                 ; operation, opaque det_cbor:
///                                 a1 01 07  =  map(1){ 1 : unsigned(7) }
///   04 58 40 <64 B>         4 : bytes(64)                ; sig-val — §18.2 row for suite 0x01
/// ```
///
/// Head derivations (RFC 8949 §3, shortest form, §18.1.1 rule 1): `bytes(32)` needs a one-byte
/// length argument, so the head is `0x58 0x20`, not `0x40|32` (which is not expressible);
/// `bytes(3)` fits the 5-bit immediate, so `0x40|3 = 0x43`; `bytes(64)` is `0x58 0x40`.
const RECEIPT_V0: &str = concat!(
    "a4",
    "0101",
    "025820",
    "3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c",
    "0343a10107",
    "045840",
    "3535353535353535353535353535353535353535353535353535353535353535",
    "3535353535353535353535353535353535353535353535353535353535353535",
);

/// `Tariff { suite: 0x01, identity: 32×7a, schedule: det_cbor({"gb": 3, "byte": 5}),
/// valid_until: 1700000000000, sig: 64×75 }` (§18.8a.1).
///
/// ```text
/// a5                        map(5)
///   01 01                   1 : unsigned(1)
///   02 58 20 <32 B>         2 : bytes(32)               ; ik-pub (self-certifying signer)
///   03 4b …                 3 : bytes(11)               ; schedule, opaque det_cbor:
///        a2                     map(2)
///          62 67 62  03           text(2) "gb"   : unsigned(3)
///          64 62 79 74 65  05     text(4) "byte" : unsigned(5)
///   04 1b 00 00 01 8b cf e5 68 00
///                           4 : unsigned(1700000000000) ; valid_until, ms since epoch
///   05 58 40 <64 B>         5 : bytes(64)
/// ```
///
/// **The schedule blob is the canonical-ordering trap, on purpose.** §18.1.1 rule 2 sorts map
/// entries by their **encoded key bytes**, and a text key's head carries its length — so `0x62 "gb"`
/// precedes `0x64 "byte"` even though `"byte" < "gb"` lexicographically. A naive `sort()` emits the
/// other order, which the strict decoder **rejects** (`CborError::MapKeyOrder`), so the mistake
/// surfaces as an interop failure at a third party rather than locally.
///
/// `valid_until` derivation: 1 700 000 000 000 = `0x0000018BCFE56800`, which exceeds `0xFFFFFFFF`,
/// so the shortest admissible head is the 8-byte `0x1b` form.
const TARIFF_V0: &str = concat!(
    "a5",
    "0101",
    "025820",
    "7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a",
    "034b",
    "a262676203646279746505",
    "041b0000018bcfe56800",
    "055840",
    "7575757575757575757575757575757575757575757575757575757575757575",
    "7575757575757575757575757575757575757575757575757575757575757575",
);

/// `CoordinatorDescriptor { suite: 0x01, kind: "infra-service", identity: 32×d1,
/// visibility: {"blind","structural"}, policy: <kotva-depot's own frozen minimal policy>,
/// sig: 64×d5 }` — **no tariff**, i.e. a coordinator that does not charge (§18.8a.1).
///
/// ```text
/// a6                        map(6)     ; keys 1,2,3,4,5,7 — key 6 absent ⇒ free (CONTRACT §6)
///   01 01                   1 : unsigned(1)
///   02 6d 69 6e 66 72 61 2d 73 65 72 76 69 63 65
///                           2 : text(13) "infra-service"
///   03 58 20 <32 B>         3 : bytes(32)
///   04 a2 …                 4 : Visibility
///        a2                     map(2)
///          01 65 62 6c 69 6e 64          1 : text(5)  "blind"
///          02 6a 73 74 72 75 63 74 75 72 61 6c
///                                        2 : text(10) "structural"
///   05 53 …                 5 : bytes(19)  ; opaque policy — byte-identical to
///                                          ; kotva-depot's own POLICY_MINIMAL vector:
///        a2 01 66 "bucket" 02 68 "operator"
///   07 58 40 <64 B>         7 : bytes(64)
/// ```
///
/// The `policy` bytes are lifted verbatim from `kotva-depot/tests/vectors.rs::POLICY_MINIMAL`. That
/// is the point: `profiles/cloud.md` §3.3 documents `DepotServicePolicy` as *the* policy blob for
/// `kind = "infra-service"`, and until now that claim was prose in two crates that had never met.
const DESCRIPTOR_V0: &str = concat!(
    "a6",
    "0101",
    "026d696e6672612d73657276696365",
    "035820",
    "d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1",
    "04",
    "a20165626c696e64026a7374727563747572616c",
    "0553",
    "a201666275636b657402686f70657261746f72",
    "075840",
    "d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5",
    "d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5",
);

/// The same descriptor **with** a tariff at key 6 — i.e. a coordinator that charges (§18.8a.1).
///
/// ```text
/// a7                        map(7)     ; keys 1..7, key 6 now present
///   … keys 1-5 exactly as DESCRIPTOR_V0 …
///   06 <TARIFF_V0 verbatim> 6 : Tariff  ; a nested MAP, not a byte string
///   07 58 40 <64 B>         7 : bytes(64)
/// ```
///
/// Two things this pins that no other vector does: the tariff nests as a **map** (a `bytes`-wrapped
/// tariff would round-trip happily inside a single implementation), and its `identity` (`0x7a…`)
/// differs from the enclosing descriptor's (`0xd1…`), which §18.8a.1 explicitly permits and requires
/// a client to surface.
const DESCRIPTOR_WITH_TARIFF_V0: &str = concat!(
    "a7",
    "0101",
    "026d696e6672612d73657276696365",
    "035820",
    "d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1",
    "04",
    "a20165626c696e64026a7374727563747572616c",
    "0553",
    "a201666275636b657402686f70657261746f72",
    "06",
    // ── TARIFF_V0, verbatim ──
    "a5",
    "0101",
    "025820",
    "7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a",
    "034b",
    "a262676203646279746505",
    "041b0000018bcfe56800",
    "055840",
    "7575757575757575757575757575757575757575757575757575757575757575",
    "7575757575757575757575757575757575757575757575757575757575757575",
    // ── end TARIFF_V0 ──
    "075840",
    "d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5",
    "d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5",
);

// ── Corruption controls, each one byte-level surgery on DESCRIPTOR_V0 ──────────────────────────

/// `DESCRIPTOR_V0` with `kind` = `"compute"` — `02 67 63 6f 6d 70 75 74 65` in place of
/// `02 6d "infra-service"`. Still a well-formed map(6); only the kind string changed.
///
/// `compute` is the kind CONTRACT §5 **folded into `infra-service`**, and which §18.8a.1's stale
/// parenthetical still lists. A decoder that accepts it keeps a retired concept alive on the wire.
const DESCRIPTOR_KIND_COMPUTE: &str = concat!(
    "a6",
    "0101",
    "0267636f6d70757465",
    "035820",
    "d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1",
    "04",
    "a20165626c696e64026a7374727563747572616c",
    "0553",
    "a201666275636b657402686f70657261746f72",
    "075840",
    "d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5",
    "d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5",
);

/// `DESCRIPTOR_V0` with `visibility` = `{"terminating", "structural"}` —
/// `01 6b "terminating"` in place of `01 65 "blind"`.
///
/// The one class/level pair §18.8a.1 forbids: a role that sees the data cannot also claim the
/// protocol structurally prevents it from seeing the data.
const DESCRIPTOR_TERMINATING_STRUCTURAL: &str = concat!(
    "a6",
    "0101",
    "026d696e6672612d73657276696365",
    "035820",
    "d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1",
    "04",
    "a2016b7465726d696e6174696e67026a7374727563747572616c",
    "0553",
    "a201666275636b657402686f70657261746f72",
    "075840",
    "d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5",
    "d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5",
);

/// `DESCRIPTOR_V0` with a **31-byte** `ik-pub` — `03 58 1f <31 B>`. Well-formed CBOR, wrong length
/// for suite `0x01` (§18.2), which is what a truncated or component-stripped key looks like on the
/// wire.
const DESCRIPTOR_SHORT_IK: &str = concat!(
    "a6",
    "0101",
    "026d696e6672612d73657276696365",
    "03581f",
    "d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1",
    "04",
    "a20165626c696e64026a7374727563747572616c",
    "0553",
    "a201666275636b657402686f70657261746f72",
    "075840",
    "d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5",
    "d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5",
);

/// `DESCRIPTOR_V0` with an extra key 8 (`08 18 61` = unsigned(97)) — a "reputation: 97" field, the
/// exact smuggling §18.8a.1's unknown-key rule exists to stop. map(6) → map(7).
const DESCRIPTOR_SMUGGLED_SCORE: &str = concat!(
    "a7",
    "0101",
    "026d696e6672612d73657276696365",
    "035820",
    "d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1",
    "04",
    "a20165626c696e64026a7374727563747572616c",
    "0553",
    "a201666275636b657402686f70657261746f72",
    "075840",
    "d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5",
    "d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5d5",
    "081861",
);

// ── Tests ──────────────────────────────────────────────────────────────────────────────────────

#[test]
fn filler_lengths_are_what_18_2_requires() {
    // A self-check on the long uniform runs above: a dropped or doubled nibble would otherwise
    // shift every following byte and turn a precise vector into an arbitrary one.
    for (name, s, byte) in [
        ("IK_DESCRIPTOR", IK_DESCRIPTOR, 0xd1u8),
        ("IK_TARIFF", IK_TARIFF, 0x7a),
        ("IK_RECEIPT", IK_RECEIPT, 0x3c),
    ] {
        let b = hex(s);
        assert_eq!(b.len(), 32, "{name}: ik-pub is 32 B under suite 0x01 (§18.2)");
        assert!(b.iter().all(|x| *x == byte), "{name}: not uniform");
    }
    for (name, s, byte) in [
        ("SIG_DESCRIPTOR", SIG_DESCRIPTOR, 0xd5u8),
        ("SIG_TARIFF", SIG_TARIFF, 0x75),
        ("SIG_RECEIPT", SIG_RECEIPT, 0x35),
    ] {
        let b = hex(s);
        assert_eq!(b.len(), 64, "{name}: sig-val is 64 B under suite 0x01 (§18.2)");
        assert!(b.iter().all(|x| *x == byte), "{name}: not uniform");
    }
}

#[test]
fn receipt_vector_decodes_to_the_specified_value() {
    let r = UsageReceipt::from_det_cbor(&hex(RECEIPT_V0)).expect("vector must decode");
    assert_eq!(r.suite, Suite::Classical);
    assert_eq!(r.identity, hex(IK_RECEIPT));
    assert_eq!(r.sig, hex(SIG_RECEIPT));
    // The opaque operation blob is carried, not interpreted — but it IS canonical det_cbor, and a
    // consumer that decodes it gets exactly what the decomposition claims.
    assert_eq!(r.operation, DetCbor::from_bytes(vec![0xa1, 0x01, 0x07]));
    assert_eq!(r.operation.decode().unwrap(), Cv::Map(vec![(1, Cv::U64(7))]));
}

#[test]
fn tariff_vector_decodes_to_the_specified_value() {
    let t = Tariff::from_det_cbor(&hex(TARIFF_V0)).expect("vector must decode");
    assert_eq!(t.suite, Suite::Classical);
    assert_eq!(t.identity, hex(IK_TARIFF));
    assert_eq!(t.valid_until, Some(1_700_000_000_000));
    assert_eq!(t.sig, hex(SIG_TARIFF));
    // Length-first key ordering inside the opaque schedule: "gb" (2 B) before "byte" (4 B).
    assert_eq!(
        t.schedule.decode().unwrap(),
        Cv::TextMap(vec![("gb".into(), Cv::U64(3)), ("byte".into(), Cv::U64(5))])
    );
    // The expiry is a signed fact and the boundary is inclusive.
    t.check_valid_at(1_700_000_000_000).unwrap();
    assert!(matches!(
        t.check_valid_at(1_700_000_000_001),
        Err(CoordinatorError::TariffExpired { .. })
    ));
}

#[test]
fn descriptor_vector_decodes_to_the_specified_value() {
    let d = CoordinatorDescriptor::from_det_cbor(&hex(DESCRIPTOR_V0)).expect("vector must decode");
    assert_eq!(d.suite, Suite::Classical);
    assert_eq!(d.kind, CoordinatorKind::InfraService);
    assert_eq!(d.identity, hex(IK_DESCRIPTOR));
    assert_eq!(d.visibility, Visibility::BLIND_STRUCTURAL);
    assert_eq!(d.visibility.class, VisibilityClass::Blind);
    assert_eq!(d.visibility.level, AssuranceLevel::Structural);
    assert_eq!(d.sig, hex(SIG_DESCRIPTOR));
    assert!(d.tariff.is_none(), "key 6 absent ⇒ this coordinator does not charge");
    assert!(!d.charges());
}

#[test]
fn descriptor_with_tariff_vector_decodes_to_the_specified_value() {
    let d = CoordinatorDescriptor::from_det_cbor(&hex(DESCRIPTOR_WITH_TARIFF_V0))
        .expect("vector must decode");
    assert!(d.charges());
    let t = d.tariff.as_ref().expect("key 6 present");
    // The nested tariff decodes to exactly the standalone TARIFF_V0 value.
    assert_eq!(*t, Tariff::from_det_cbor(&hex(TARIFF_V0)).unwrap());
    // …and it is NOT the descriptor's own identity. §18.8a.1: a client MUST attribute the tariff to
    // `Tariff.identity` and MUST surface the distinction when comparing prices (§26.10).
    assert!(!t.is_self_issued_by(&d.identity));
    assert_eq!(t.identity, hex(IK_TARIFF));
    assert_eq!(d.identity, hex(IK_DESCRIPTOR));
}

/// The DEPOT seam: `profiles/cloud.md` §3.3 says a `DepotServicePolicy` **is** the `policy` blob of
/// an `infra-service` `CoordinatorDescriptor`. Until this test that was prose in two crates that had
/// never met.
#[test]
fn a_depot_service_policy_round_trips_through_a_real_coordinator_descriptor() {
    use kotva_depot::{Backing, DepotServicePolicy, Service};

    // (a) From the frozen vector: the descriptor's `policy` bytes are byte-identical to
    //     kotva-depot's own POLICY_MINIMAL vector, and decode as that policy.
    let d = CoordinatorDescriptor::from_det_cbor(&hex(DESCRIPTOR_V0)).unwrap();
    let p = DepotServicePolicy::from_det_cbor(d.policy.as_bytes())
        .expect("the descriptor's policy blob must be a DepotServicePolicy");
    assert_eq!(p.service, Service::Bucket);
    assert_eq!(p.backing, Backing::Operator);
    assert_eq!(p.det_cbor(), d.policy.as_bytes(), "re-encode must reproduce the carried bytes");

    // (b) The other direction, through a REAL signature: build a richer policy, sign a descriptor
    //     around it, encode, decode+verify, and pull the policy back out intact.
    let ik = kotva_core::identity::IdentityKey::from_seed(&[0x5a; 32]);
    let rich = DepotServicePolicy {
        attributes: vec![
            ("region".into(), "za-jhb".into()),
            ("arch".into(), "aarch64".into()),
        ],
        ..DepotServicePolicy::new(Service::Box, Backing::Customer)
    }
    .normalized();
    rich.validate().expect("the sample policy must be conformant DEPOT");

    let signed = CoordinatorDescriptor::sign(
        &kotva_coordinator::Signer::Classical(&ik),
        CoordinatorKind::InfraService,
        // A `box` is terminating; DEPOT's own honesty check agrees this is the honest declaration.
        Visibility::TERMINATING,
        DetCbor::from_bytes(rich.det_cbor()),
        None,
    )
    .unwrap();
    kotva_depot::check_visibility(
        Service::Box,
        kotva_depot::Visibility::TERMINATING,
        false,
        false,
    )
    .expect("DEPOT must agree the declared visibility is not an overclaim");

    let back = CoordinatorDescriptor::from_det_cbor_verified(&signed.det_cbor()).unwrap();
    let recovered = DepotServicePolicy::from_det_cbor(back.policy.as_bytes()).unwrap();
    assert_eq!(recovered, rich);
    assert_eq!(recovered.attribute("region"), Some("za-jhb"));
}

/// The two crates model visibility separately — DEPOT's is `Ord`-ordered so formula inheritance can
/// compute the least-blind of a composition (`profiles/cloud.md` §3.6), and it carries **no wire
/// strings** because visibility rides the descriptor, not the policy blob; this one is the §18.8a.1
/// wire field. Nothing but this test stops the two vocabularies drifting apart, and a drift would be
/// silent: each crate's own round-trips would keep passing while a DEPOT coordinator's declared
/// visibility stopped meaning the same thing as the one on its descriptor.
///
/// The `match`es are exhaustive on purpose — a fourth DEPOT class or level fails to **compile**
/// here, which is the earliest and loudest place for that to surface.
#[test]
fn every_depot_visibility_is_declarable_on_the_wire_and_means_the_same_thing() {
    use kotva_depot::{Assurance, Visibility as DepotVisibility, VisibilityClass as DepotClass};

    fn to_wire(v: DepotVisibility) -> Visibility {
        Visibility::new(
            match v.class {
                DepotClass::Blind => VisibilityClass::Blind,
                DepotClass::BlindRouting => VisibilityClass::BlindRouting,
                DepotClass::Terminating => VisibilityClass::Terminating,
            },
            match v.assurance {
                Assurance::Structural => AssuranceLevel::Structural,
                Assurance::Attested => AssuranceLevel::Attested,
                Assurance::Declared => AssuranceLevel::Declared,
            },
        )
    }

    // Every visibility DEPOT ships as a named constant must be publishable on a descriptor. This is
    // the concrete stake in the §18.8a.1 divergence recorded on `Visibility::check`: under the
    // literal "a terminating class MUST declare declared" reading, `TERMINATING_ATTESTED` — a box or
    // edge-fn in a client-verified TEE, `profiles/cloud.md` §5 — could not be published at all.
    for depot_v in [
        DepotVisibility::BLIND_STRUCTURAL,
        DepotVisibility::BLIND_DECLARED,
        DepotVisibility::BLIND_ROUTING,
        DepotVisibility::TERMINATING,
        DepotVisibility::TERMINATING_ATTESTED,
    ] {
        let wire = to_wire(depot_v);
        wire.check()
            .unwrap_or_else(|e| panic!("{depot_v:?} is a DEPOT constant but undeclarable: {e}"));
        // The honesty predicate must give the same verdict in both crates, or a service could be
        // advertised as private through one and not the other.
        assert_eq!(
            wire.may_be_called_private(),
            depot_v.may_be_called_private(),
            "{depot_v:?}: the two crates disagree on whether this may be called private"
        );
    }

    // And the pair both crates agree is dishonest is still refused here.
    assert!(to_wire(DepotVisibility::new(
        DepotClass::Terminating,
        Assurance::Structural
    ))
    .check()
    .is_err());
}

/// The direction that actually catches an encoder that drifted from the spec.
///
/// Decoding a vector only proves the decoder is permissive enough to accept it. Re-encoding the
/// decoded value and demanding the **original bytes back** is what pins the encoder: a changed key
/// number, a `tstr` that became a `bytes`, a lost canonical ordering, a nested object flattened into
/// a byte string, or a different integer width all fail here while every round-trip test in `src/`
/// continues to pass.
#[test]
fn re_encoding_each_vector_reproduces_the_specified_bytes() {
    let cases: Vec<(&str, &str, Vec<u8>)> = vec![
        (
            "UsageReceipt",
            RECEIPT_V0,
            UsageReceipt::from_det_cbor(&hex(RECEIPT_V0)).unwrap().det_cbor(),
        ),
        (
            "Tariff",
            TARIFF_V0,
            Tariff::from_det_cbor(&hex(TARIFF_V0)).unwrap().det_cbor(),
        ),
        (
            "CoordinatorDescriptor",
            DESCRIPTOR_V0,
            CoordinatorDescriptor::from_det_cbor(&hex(DESCRIPTOR_V0)).unwrap().det_cbor(),
        ),
        (
            "CoordinatorDescriptor+Tariff",
            DESCRIPTOR_WITH_TARIFF_V0,
            CoordinatorDescriptor::from_det_cbor(&hex(DESCRIPTOR_WITH_TARIFF_V0))
                .unwrap()
                .det_cbor(),
        ),
    ];
    assert_eq!(cases.len(), 4, "every vector must be re-encoded, not a subset");
    for (name, expected_hex, actual) in cases {
        assert_eq!(
            actual,
            hex(expected_hex),
            "{name}: re-encoding drifted from the hand-derived vector"
        );
    }
}

/// A control on the corpus itself.
///
/// If [`hex`] silently produced empty input, or the decoders accepted anything at all, every test
/// above would pass while proving nothing — the failure mode this repo has hit repeatedly. Each
/// mutation below MUST be rejected, and each is paired with the pristine vector being accepted, so
/// a rejection is known to come from the mutation and not from some unrelated defect.
#[test]
fn corrupted_vectors_are_rejected() {
    // ── false-positive controls: the pristine vectors decode ────────────────────────────────
    assert!(CoordinatorDescriptor::from_det_cbor(&hex(DESCRIPTOR_V0)).is_ok());
    assert!(CoordinatorDescriptor::from_det_cbor(&hex(DESCRIPTOR_WITH_TARIFF_V0)).is_ok());
    assert!(Tariff::from_det_cbor(&hex(TARIFF_V0)).is_ok());
    assert!(UsageReceipt::from_det_cbor(&hex(RECEIPT_V0)).is_ok());

    // ── structural corruption ───────────────────────────────────────────────────────────────
    // Truncation.
    assert!(CoordinatorDescriptor::from_det_cbor(&hex(&DESCRIPTOR_V0[..20])).is_err());
    assert!(Tariff::from_det_cbor(&hex(&TARIFF_V0[..20])).is_err());
    // Empty input is not an object.
    assert!(CoordinatorDescriptor::from_det_cbor(&[]).is_err());
    assert!(UsageReceipt::from_det_cbor(&[]).is_err());
    // Trailing bytes after a complete item (§18.1.1): re-encoding would silently drop the tail.
    assert!(UsageReceipt::from_det_cbor(&hex(&format!("{RECEIPT_V0}00"))).is_err());

    // ── closed-registry corruption ──────────────────────────────────────────────────────────
    // A folded kind that must not survive on the wire.
    assert!(matches!(
        CoordinatorDescriptor::from_det_cbor(&hex(DESCRIPTOR_KIND_COMPUTE)),
        Err(CoordinatorError::UnknownRegistryValue { registry: "coordinator-kind", .. })
    ));
    // The one undeclarable class/level pair.
    assert!(matches!(
        CoordinatorDescriptor::from_det_cbor(&hex(DESCRIPTOR_TERMINATING_STRUCTURAL)),
        Err(CoordinatorError::UndeclarableVisibility { .. })
    ));

    // ── §18.2 length governance ─────────────────────────────────────────────────────────────
    assert!(matches!(
        CoordinatorDescriptor::from_det_cbor(&hex(DESCRIPTOR_SHORT_IK)),
        Err(CoordinatorError::BadFieldLength { field: "identity", expected: 32, actual: 31, .. })
    ));

    // ── the unknown-key rule §2.1's "no score field" rests on ────────────────────────────────
    assert!(matches!(
        CoordinatorDescriptor::from_det_cbor(&hex(DESCRIPTOR_SMUGGLED_SCORE)),
        Err(CoordinatorError::Cbor(kotva_core::cbor::CborError::UnknownKey(8)))
    ));

    // ── type confusion: the tariff at key 6 delivered as a byte string rather than a map ─────
    // (`06 41 aa` = bytes(1)). An implementation that wrapped nested objects in `bytes` would
    // round-trip perfectly against itself and interop with nobody.
    let tariff_as_bytes = DESCRIPTOR_WITH_TARIFF_V0.replace(
        concat!(
            "06",
            "a5",
            "0101",
            "025820",
            "7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a7a",
            "034b",
            "a262676203646279746505",
            "041b0000018bcfe56800",
            "055840",
            "7575757575757575757575757575757575757575757575757575757575757575",
            "7575757575757575757575757575757575757575757575757575757575757575",
        ),
        "0641aa",
    );
    assert_ne!(
        tariff_as_bytes, DESCRIPTOR_WITH_TARIFF_V0,
        "the substitution must actually have matched"
    );
    assert!(CoordinatorDescriptor::from_det_cbor(&hex(&tariff_as_bytes)).is_err());
}
