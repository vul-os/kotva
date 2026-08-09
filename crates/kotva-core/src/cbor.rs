//! Canonical deterministic CBOR — spec §18.1.1 / §18.1.2.
//!
//! DMTAP wire objects are **integer-keyed** CBOR maps (COSE/CWT style, §18.1.2) encoded with
//! RFC 8949 Core Deterministic Encoding (§18.1.1). This module is the single canonical codec:
//! serde/`ciborium`-derived encodings are **text-keyed** (struct field names) and MUST NOT be
//! used on the wire — a second implementer following §18 would produce different bytes, so the
//! conformance vectors would validate the code only against itself. Everything the reference
//! serializes for the wire, signs, or content-addresses flows through [`encode`]/[`decode`].
//!
//! ## Encoding rules enforced here (§18.1.1)
//! 1. Shortest-form integers / lengths / counts (RFC 8949 §4.2.1); no indefinite-length items.
//! 2. Map keys sorted by their **encoded bytes**, ascending (for the small unsigned keys used
//!    everywhere this equals numeric key order).
//! 3. No duplicate keys (rejected on decode).
//! 4. No floating-point values anywhere.
//! 5. No NaN/Infinity, no tags, no `undefined`; and no `null` on the wire (an absent optional is
//!    simply omitted from the map, never present as `null`).

/// A canonical CBOR value restricted to the DMTAP wire subset (§18.1.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cv {
    /// Unsigned integer (major type 0). DMTAP uses only unsigned integers on the wire.
    U64(u64),
    /// Byte string (major type 2).
    Bytes(Vec<u8>),
    /// UTF-8 text string (major type 3).
    Text(String),
    /// Boolean (major type 7, `0xf4`/`0xf5`) — admitted only where a rule allows `bool`.
    Bool(bool),
    /// Definite-length array (major type 4).
    Array(Vec<Cv>),
    /// Integer-keyed map (major type 5) — the DMTAP object encoding (§18.1.2).
    Map(Vec<(u64, Cv)>),
    /// Text-keyed map (major type 5) — the **only** place text keys are admitted:
    /// `Headers.ext` (§18.3.6). Values are still restricted to this `Cv` subset.
    TextMap(Vec<(String, Cv)>),
}

/// Errors from decoding / validating canonical CBOR (fail closed, §18.1.1).
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum CborError {
    #[error("malformed CBOR")]
    Malformed,
    #[error("non-shortest-form integer/length encoding (§18.1.1 rule 1)")]
    NonShortestForm,
    #[error("indefinite-length item is forbidden (§18.1.1 rule 1)")]
    IndefiniteLength,
    #[error("map keys are not in strictly ascending encoded-byte order (§18.1.1 rule 2)")]
    MapKeyOrder,
    #[error("trailing bytes after the top-level CBOR item (§18.1.1)")]
    TrailingData,
    #[error("floating-point value is forbidden on the DMTAP wire (§18.1.1 rule 4)")]
    FloatPresent,
    #[error("CBOR null is forbidden on the wire — absent optionals are omitted (§18.1.1)")]
    NullPresent,
    #[error("CBOR tag / undefined is forbidden on the DMTAP wire (§18.1.1 rule 5)")]
    TagOrUndefined,
    #[error("duplicate map key {0} (§18.1.1 rule 3)")]
    DuplicateKey(u64),
    #[error("duplicate text map key")]
    DuplicateTextKey,
    #[error("map mixes integer and text keys")]
    MixedMapKeys,
    #[error("negative or out-of-range integer")]
    IntRange,
    #[error("unexpected CBOR type for this field")]
    TypeMismatch,
    #[error("unknown key {0} in a signed object (fail closed, §18.1.2)")]
    UnknownKey(u64),
    #[error("missing required key {0}")]
    MissingKey(u64),
    #[error("Manifest carries forbidden key 5 (ERR_MANIFEST_KEY_PRESENT, §18.3.8)")]
    ManifestKeyPresent,
    #[error(
        "Manifest chunk list is empty — a manifest MUST carry ≥ 1 chunk (§18.3.8, fail closed)"
    )]
    ManifestEmptyChunks,
    #[error("unsupported / unknown algorithm suite byte {0:#04x} (fail closed)")]
    UnknownSuite(u8),
    #[error("unknown enum discriminator {0}")]
    UnknownDiscriminant(u64),
}

// ── The byte layer lives in `kotva-cbor` ───────────────────────────────────────────────────────
//
// Everything below is an *adapter*, not an implementation. The encoder and strict decoder that
// used to live here (≈310 lines of hand-written recursive descent) are now `kotva_cbor`, which is
// byte-for-byte the same codec and carries the proof: a frozen 183-vector corpus extracted from
// evermesh's conformance suite, the only implementation in this family that already treated
// cross-implementation byte identity as consensus-critical.
//
// WHY DELEGATE RATHER THAN KEEP A COPY. Four codecs in this family claimed these same rules with
// no cross-check between any two (`kotva_core::cbor`, `kotva_sync::detcbor`,
// `evermesh_kernel::codec`, `magnetite_seams::cbor`). A signature is a statement about bytes, so
// two honest implementations that disagree by one head byte silently break verification. Sharing
// the compiled algebra — not merely matching bytes — is what `substrate/SOVEREIGNTY.md` §3.5 now
// requires of an adopter, and kotva-core cannot require of others what it does not do itself.
//
// WHAT DID **NOT** MOVE, deliberately: [`Cv`], [`CborError`], [`Fields`] and the `as_*` helpers.
// `Cv` is the *narrower* DMTAP wire subset — integer-keyed maps with a separate [`Cv::TextMap`]
// for the one text-keyed site (§18.3.6), and **no** `null`, **no** negative integers, no floats,
// no tags. `kotva_cbor::Value` is deliberately wider (it accepts exactly what evermesh accepts, so
// a migration cannot change a verdict). The narrowing therefore happens in [`from_value`] below,
// and every rejection it makes maps onto kotva-core's own long-standing error variants. This
// crate's public API is unchanged, which matters because external adopters pin it by tag.

/// Lower a [`Cv`] into the shared codec's wider value type. Total and allocation-only: every `Cv`
/// is representable as a [`kotva_cbor::Value`], which is the direction that must never fail.
fn to_value(v: &Cv) -> kotva_cbor::Value {
    use kotva_cbor::Value as V;
    match v {
        Cv::U64(n) => V::Uint(*n),
        Cv::Bytes(b) => V::Bytes(b.clone()),
        Cv::Text(s) => V::Text(s.clone()),
        Cv::Bool(b) => V::Bool(*b),
        Cv::Array(a) => V::Array(a.iter().map(to_value).collect()),
        Cv::Map(m) => V::Map(
            m.iter()
                .map(|(k, val)| (V::Uint(*k), to_value(val)))
                .collect(),
        ),
        Cv::TextMap(m) => V::Map(
            m.iter()
                .map(|(k, val)| (V::Text(k.clone()), to_value(val)))
                .collect(),
        ),
    }
}

/// Narrow a decoded [`kotva_cbor::Value`] to the DMTAP wire subset, failing closed on anything
/// `Cv` cannot represent. This is where "canonical CBOR" becomes "canonical *DMTAP* CBOR".
fn from_value(v: kotva_cbor::Value) -> Result<Cv, CborError> {
    use kotva_cbor::Value as V;
    match v {
        V::Uint(n) => Ok(Cv::U64(n)),
        // Major type 1. Canonical CBOR, but never on the DMTAP wire (§18.1.1 rule 4).
        V::Nint(_) => Err(CborError::IntRange),
        V::Bytes(b) => Ok(Cv::Bytes(b)),
        V::Text(s) => Ok(Cv::Text(s)),
        V::Bool(b) => Ok(Cv::Bool(b)),
        // An absent optional is omitted from the map, never present as `null` (§18.1.1).
        V::Null => Err(CborError::NullPresent),
        V::Array(items) => items
            .into_iter()
            .map(from_value)
            .collect::<Result<Vec<_>, _>>()
            .map(Cv::Array),
        V::Map(entries) => {
            // An empty map is variant-neutral and matches what `encode` emits for either variant.
            if entries.is_empty() {
                return Ok(Cv::Map(Vec::new()));
            }
            // Key ordering and duplicate rejection already happened in the shared decoder. What is
            // left is DMTAP's own restriction: every key is an unsigned integer (§18.1.2), or every
            // key is a text string (`Headers.ext`, §18.3.6) — never a mixture, and never any other
            // major type.
            match &entries[0].0 {
                V::Uint(_) => {
                    let mut out = Vec::with_capacity(entries.len());
                    for (k, val) in entries {
                        match k {
                            V::Uint(k) => out.push((k, from_value(val)?)),
                            _ => return Err(CborError::MixedMapKeys),
                        }
                    }
                    Ok(Cv::Map(out))
                }
                V::Text(_) => {
                    let mut out = Vec::with_capacity(entries.len());
                    for (k, val) in entries {
                        match k {
                            V::Text(k) => out.push((k, from_value(val)?)),
                            _ => return Err(CborError::MixedMapKeys),
                        }
                    }
                    Ok(Cv::TextMap(out))
                }
                // A key that is neither a small unsigned integer nor a text string is not a DMTAP
                // wire map at all. `kotva_cbor` permits any key type; DMTAP does not.
                _ => Err(CborError::MixedMapKeys),
            }
        }
    }
}

/// Translate the shared codec's refusal into kotva-core's own taxonomy.
///
/// `kotva_cbor::CborError` is deliberately finer-grained than this enum (it separates `Truncated`
/// from `LengthExceedsInput` from `DepthExceeded`, all of which are [`CborError::Malformed`] here),
/// so the mapping is many-to-one and every variant kotva-core already published is preserved.
fn map_err(e: kotva_cbor::CborError) -> CborError {
    use kotva_cbor::CborError as E;
    match e {
        E::NonShortestForm => CborError::NonShortestForm,
        E::IndefiniteLength => CborError::IndefiniteLength,
        E::TrailingBytes => CborError::TrailingData,
        E::Float => CborError::FloatPresent,
        // `undefined`, one-byte simple values and tags are one refusal to DMTAP (§18.1.1 rule 5).
        E::Undefined | E::SimpleValue | E::Tag => CborError::TagOrUndefined,
        E::MapKeyOrder => CborError::MapKeyOrder,
        // Same encoded key bytes ⇒ same key. Report the key itself where it is an integer, matching
        // the pre-delegation behaviour exactly.
        E::DuplicateMapKey(k) => match *k {
            kotva_cbor::Value::Uint(k) => CborError::DuplicateKey(k),
            _ => CborError::DuplicateTextKey,
        },
        // Truncation, over-long lengths, bad UTF-8, over-deep nesting and the reserved
        // additional-info values are all just "these are not well-formed canonical bytes".
        E::Truncated
        | E::ReservedAdditionalInfo
        | E::InvalidUtf8
        | E::DepthExceeded
        | E::LengthExceedsInput
        | E::LengthTooLarge => CborError::Malformed,
        // Only the optional `json` feature produces this, and kotva-core does not enable it.
        E::Json(_) => CborError::Malformed,
        // `kotva_cbor::CborError` is `#[non_exhaustive]`: a new refusal added upstream must fail
        // closed here rather than fail to compile or, worse, be treated as acceptance.
        _ => CborError::Malformed,
    }
}

// ── Encoding ───────────────────────────────────────────────────────────────────────────────

/// Encode a [`Cv`] as deterministic CBOR (§18.1.1). Infallible: `Cv` cannot hold a forbidden value.
///
/// Map keys are emitted sorted by their **encoded bytes**, ascending (rule 2), at every depth, so
/// insertion order does not affect the bytes. For the small unsigned keys used throughout DMTAP
/// that is identical to numeric key order.
pub fn encode(v: &Cv) -> Vec<u8> {
    // `_unchecked` is the infallible encoder, which is what this function's published signature
    // requires. It differs from the checked one only for a `Cv::Map`/`Cv::TextMap` that holds the
    // same key twice — a value with no canonical encoding, which this function has always emitted
    // as-is (and which [`decode`] then refuses). Silently dropping an entry would be worse, and
    // returning a `Result` would break every external adopter pinned to this tag.
    kotva_cbor::encode_canonical_unchecked(&to_value(v))
}

// ── Decoding ───────────────────────────────────────────────────────────────────────────────

/// Parse and validate **canonical** CBOR into a [`Cv`], **failing closed** on any deviation from
/// RFC 8949 Core Deterministic Encoding as profiled by §18.1.1. This is a *strict* decoder (not a
/// lenient library normalize-and-accept), so it enforces the input side of §18.1.1 that a canonical
/// decoder MUST re-check:
///
/// 1. **Shortest-form** integers, string/array/map lengths (rule 1) — a longer-than-minimal head
///    (`0x18 0x0a` for 10, etc.) is rejected ([`CborError::NonShortestForm`]).
/// 2. **Definite-length only** — no indefinite-length items or the `break` code
///    ([`CborError::IndefiniteLength`]).
/// 3. **Strictly ascending map keys**, compared by their *encoded bytes* (rule 2), with **no
///    duplicates** (rule 3) ([`CborError::MapKeyOrder`] / [`CborError::DuplicateKey`]).
/// 4. **No floats / NaN / Infinity** (rule 4), **no tags / `undefined` / simple values**, **no
///    `null` on the wire** (rule 5), **no negative integers**, and **no trailing bytes** after the
///    top-level item.
///
/// Rules 1–3 and the trailing-byte check are enforced by the shared `kotva-cbor` decoder; the
/// DMTAP-specific narrowing in rule 4 (no `null`, no negative integers, integer-or-text map keys but
/// never both) is applied on the way into `Cv`.
///
/// ## What the delegation changed, measured rather than asserted
///
/// A differential harness drove this function and the hand-written decoder it replaced over 800,183
/// inputs — 200,000 generated `Cv` values round-tripped, 400,000 random byte strings, 200,000
/// near-canonical mutations, and all 183 evermesh conformance vectors:
///
/// * **The accept/reject verdict is identical on every input**, and
/// * **every input both accept decodes to the identical `Cv`.**
///
/// Those are the two properties a signature and a content address depend on, and they hold exactly.
///
/// What *did* change is **which** [`CborError`] variant a refusal reports: 83,670 of the invalid
/// inputs are refused by both decoders for a differently-named reason. Nothing is accepted that was
/// refused, or refused that was accepted — the variant is diagnostic only. Two causes, neither worth
/// re-introducing a second decoder to preserve:
///
/// 1. **Defect precedence.** This decoder validates canonical form completely before narrowing to
///    `Cv`, so for input that is invalid in more than one way it reports the canonical-form defect
///    where the old one reported whichever it met first in byte order (`0x28 …` — a negative integer
///    followed by trailing bytes — was [`CborError::IntRange`], and is now
///    [`CborError::TrailingData`]).
/// 2. **The old decoder mislabelled two cases**, and the shared codec is more accurate. Major type 7
///    with additional info 0–19 is an *unassigned simple value*; the old `simple()` arm fell through
///    to [`CborError::IndefiniteLength`], which it is not. A bare truncated float or tag head was
///    [`CborError::Malformed`] rather than naming the forbidden major type.
///
/// Because the decoder accepts *only* the canonical encoding of a value, `encode(decode(b)) == b`
/// for every accepted `b` — the malleability / signature-reproducibility guarantee §18.1.1 exists
/// to provide. Higher layers additionally reject unknown keys in *signed* objects (§18.1.2).
pub fn decode(bytes: &[u8]) -> Result<Cv, CborError> {
    from_value(kotva_cbor::decode_canonical(bytes).map_err(map_err)?)
}

// ── Field extraction helpers ─────────────────────────────────────────────────────────────────

/// A consuming reader over an integer-keyed map, used by every object's decoder. Take the keys
/// you know, then call [`Fields::deny_unknown`] on a **signed** object so any leftover key fails
/// closed (§18.1.2).
pub struct Fields {
    map: Vec<(u64, Cv)>,
}

impl Fields {
    /// Wrap a decoded map (expects [`Cv::Map`]).
    pub fn from_cv(cv: Cv) -> Result<Self, CborError> {
        match cv {
            Cv::Map(map) => Ok(Fields { map }),
            _ => Err(CborError::TypeMismatch),
        }
    }

    /// Whether key `k` is present (without removing it).
    pub fn has(&self, k: u64) -> bool {
        self.map.iter().any(|(kk, _)| *kk == k)
    }

    /// Remove and return the value at key `k`, if present.
    pub fn take(&mut self, k: u64) -> Option<Cv> {
        self.map
            .iter()
            .position(|(kk, _)| *kk == k)
            .map(|pos| self.map.remove(pos).1)
    }

    /// Remove and return the value at required key `k`, or [`CborError::MissingKey`].
    pub fn req(&mut self, k: u64) -> Result<Cv, CborError> {
        self.take(k).ok_or(CborError::MissingKey(k))
    }

    /// Consume the reader, yielding every remaining `(key, value)` pair (for maps whose keys are
    /// data, e.g. `Identity.iks`, rather than a fixed schema).
    pub fn into_pairs(self) -> Vec<(u64, Cv)> {
        self.map
    }

    /// After taking every recognized key, reject any that remain (signed-object rule, §18.1.2).
    pub fn deny_unknown(&self) -> Result<(), CborError> {
        match self.map.first() {
            Some((k, _)) => Err(CborError::UnknownKey(*k)),
            None => Ok(()),
        }
    }
}

// Coercions from `Cv` to concrete types (fail closed on the wrong CBOR type).

pub fn as_u64(cv: Cv) -> Result<u64, CborError> {
    match cv {
        Cv::U64(n) => Ok(n),
        _ => Err(CborError::TypeMismatch),
    }
}

pub fn as_u8(cv: Cv) -> Result<u8, CborError> {
    let n = as_u64(cv)?;
    u8::try_from(n).map_err(|_| CborError::IntRange)
}

pub fn as_u32(cv: Cv) -> Result<u32, CborError> {
    let n = as_u64(cv)?;
    u32::try_from(n).map_err(|_| CborError::IntRange)
}

pub fn as_bytes(cv: Cv) -> Result<Vec<u8>, CborError> {
    match cv {
        Cv::Bytes(b) => Ok(b),
        _ => Err(CborError::TypeMismatch),
    }
}

pub fn as_text(cv: Cv) -> Result<String, CborError> {
    match cv {
        Cv::Text(s) => Ok(s),
        _ => Err(CborError::TypeMismatch),
    }
}

pub fn as_bool(cv: Cv) -> Result<bool, CborError> {
    match cv {
        Cv::Bool(b) => Ok(b),
        _ => Err(CborError::TypeMismatch),
    }
}

pub fn as_array(cv: Cv) -> Result<Vec<Cv>, CborError> {
    match cv {
        Cv::Array(a) => Ok(a),
        _ => Err(CborError::TypeMismatch),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortest_form_integer_heads() {
        assert_eq!(encode(&Cv::U64(0)), vec![0x00]);
        assert_eq!(encode(&Cv::U64(23)), vec![0x17]);
        assert_eq!(encode(&Cv::U64(24)), vec![0x18, 0x18]);
        assert_eq!(encode(&Cv::U64(255)), vec![0x18, 0xff]);
        assert_eq!(encode(&Cv::U64(256)), vec![0x19, 0x01, 0x00]);
        assert_eq!(encode(&Cv::U64(1_700_000_000_000)), {
            let mut e = vec![0x1b];
            e.extend_from_slice(&1_700_000_000_000u64.to_be_bytes());
            e
        });
    }

    #[test]
    fn map_keys_emitted_ascending_regardless_of_insertion_order() {
        let m = Cv::Map(vec![
            (10, Cv::U64(1)),
            (2, Cv::U64(2)),
            (1, Cv::U64(3)),
            (24, Cv::U64(4)),
        ]);
        let bytes = encode(&m);
        // map(4) then keys 1,2,10,24 (24 is two-byte-encoded, sorts after single-byte 10).
        assert_eq!(bytes[0], 0xa4);
        assert_eq!(
            &bytes[1..],
            &[0x01, 0x03, 0x02, 0x02, 0x0a, 0x01, 0x18, 0x18, 0x04]
        );
    }

    #[test]
    fn round_trip_through_decode() {
        let v = Cv::Map(vec![
            (1, Cv::U64(0)),
            (2, Cv::Bytes(vec![0xde, 0xad])),
            (3, Cv::Text("hi".into())),
            (4, Cv::Array(vec![Cv::U64(7), Cv::Bool(true)])),
        ]);
        let bytes = encode(&v);
        assert_eq!(decode(&bytes).unwrap(), v);
    }

    #[test]
    fn rejects_float() {
        // A CBOR half-float 0xf9 0x00 0x00 (0.0).
        assert_eq!(decode(&[0xf9, 0x00, 0x00]), Err(CborError::FloatPresent));
    }

    #[test]
    fn rejects_null_on_the_wire() {
        // map{1: null}
        assert_eq!(decode(&[0xa1, 0x01, 0xf6]), Err(CborError::NullPresent));
    }

    #[test]
    fn rejects_duplicate_key() {
        // map claiming 2 entries, both key 1.
        assert_eq!(
            decode(&[0xa2, 0x01, 0x00, 0x01, 0x01]),
            Err(CborError::DuplicateKey(1))
        );
    }

    #[test]
    fn rejects_tag() {
        // tag(0) "text" — tag major type 6.
        assert_eq!(decode(&[0xc0, 0x61, 0x41]), Err(CborError::TagOrUndefined));
    }

    #[test]
    fn deny_unknown_flags_leftover_key() {
        let mut f = Fields::from_cv(Cv::Map(vec![(1, Cv::U64(0)), (99, Cv::U64(0))])).unwrap();
        let _ = f.take(1);
        assert_eq!(f.deny_unknown(), Err(CborError::UnknownKey(99)));
    }

    // ── Strict-canonical decode: each §18.1.1 rejection the harness proved was missing ──────

    #[test]
    fn rejects_non_shortest_integer() {
        // uint 10 encoded in a two-byte head (0x18 0x0a); preferred form is the single byte 0x0a.
        assert_eq!(decode(&[0x18, 0x0a]), Err(CborError::NonShortestForm));
        // uint 23 in a two-byte head (DMTAP-CBOR-05: 0x18 0x17).
        assert_eq!(decode(&[0x18, 0x17]), Err(CborError::NonShortestForm));
        // uint 200 encoded in a two-byte (0x19) head when one-byte (0x18 0xc8) suffices.
        assert_eq!(decode(&[0x19, 0x00, 0xc8]), Err(CborError::NonShortestForm));
        // uint 0 encoded 8-wide.
        assert_eq!(
            decode(&[0x1b, 0, 0, 0, 0, 0, 0, 0, 0]),
            Err(CborError::NonShortestForm)
        );
        // A non-shortest *length* head on a byte string is equally rejected.
        assert_eq!(decode(&[0x58, 0x01, 0xaa]), Err(CborError::NonShortestForm));
    }

    #[test]
    fn accepts_genuine_shortest_forms() {
        // These are the *canonical* two-byte forms (value truly needs the wider head) — accepted.
        assert_eq!(decode(&[0x18, 0x18]).unwrap(), Cv::U64(24));
        assert_eq!(decode(&[0x19, 0x01, 0x00]).unwrap(), Cv::U64(256));
    }

    #[test]
    fn rejects_indefinite_length_items() {
        // Indefinite-length array (DMTAP-CBOR-06: 0x9f … 0xff).
        assert_eq!(decode(&[0x9f, 0xff]), Err(CborError::IndefiniteLength));
        // Indefinite-length byte string, text string, and map.
        assert_eq!(decode(&[0x5f, 0xff]), Err(CborError::IndefiniteLength));
        assert_eq!(decode(&[0x7f, 0xff]), Err(CborError::IndefiniteLength));
        assert_eq!(decode(&[0xbf, 0xff]), Err(CborError::IndefiniteLength));
    }

    #[test]
    fn rejects_descending_map_keys() {
        // map {2:0, 1:0} — keys 2 then 1 are descending (DMTAP-CBOR-07: 0xa2 02 00 01 00).
        assert_eq!(
            decode(&[0xa2, 0x02, 0x00, 0x01, 0x00]),
            Err(CborError::MapKeyOrder)
        );
    }

    #[test]
    fn accepts_ascending_map_keys() {
        let cv = decode(&[0xa2, 0x01, 0x00, 0x02, 0x00]).unwrap();
        assert_eq!(cv, Cv::Map(vec![(1, Cv::U64(0)), (2, Cv::U64(0))]));
    }

    #[test]
    fn rejects_descending_text_map_keys() {
        // map {"b":0, "a":0} — text keys 0x62.. then 0x61.. are descending.
        assert_eq!(
            decode(&[0xa2, 0x61, 0x62, 0x00, 0x61, 0x61, 0x00]),
            Err(CborError::MapKeyOrder)
        );
    }

    #[test]
    fn rejects_negative_integer() {
        // -1 is major type 1; DMTAP wire maps carry only unsigned integers.
        assert_eq!(decode(&[0x20]), Err(CborError::IntRange));
    }

    #[test]
    fn rejects_undefined_and_simple() {
        assert_eq!(decode(&[0xf7]), Err(CborError::TagOrUndefined)); // undefined (DMTAP-CBOR-10)
        assert_eq!(decode(&[0xf8, 0xff]), Err(CborError::TagOrUndefined)); // simple(255)
    }

    #[test]
    fn rejects_trailing_bytes() {
        // A valid single item (0x00) followed by a stray byte MUST be rejected — exactly one
        // top-level item is permitted, else re-encoding would silently drop the tail.
        assert_eq!(decode(&[0x00, 0x00]), Err(CborError::TrailingData));
    }

    #[test]
    fn strict_decode_is_reencode_idempotent_on_canonical_bytes() {
        // Every accepted encoding round-trips byte-for-byte (the malleability guarantee).
        let v = Cv::Map(vec![
            (1, Cv::U64(24)),
            (2, Cv::Bytes(vec![0xde, 0xad, 0xbe, 0xef])),
            (3, Cv::Text("hi".into())),
            (
                7,
                Cv::Array(vec![Cv::U64(256), Cv::Bool(true), Cv::Bool(false)]),
            ),
            (24, Cv::U64(1_700_000_000_000)),
        ]);
        let bytes = encode(&v);
        assert_eq!(decode(&bytes).unwrap(), v);
        assert_eq!(encode(&decode(&bytes).unwrap()), bytes);
    }

    #[test]
    fn rejects_over_deep_nesting_without_stack_overflow() {
        // ~50 KB of single-element-array heads (0x81 = array(1)) would recurse per level and
        // overflow the native stack on an unbounded decoder. It MUST fail closed instead.
        let deep = vec![0x81u8; 50_000];
        assert_eq!(decode(&deep), Err(CborError::Malformed));
        // Right at the boundary: MAX_DEPTH + 1 nested arrays around a scalar exceed the bound and
        // are rejected — no panic, a clean error. The bound is the *shared* codec's constant, read
        // from it rather than restated here, so kotva-core and kotva-cbor cannot drift apart on the
        // one number where an encoder minting a value a second decoder refuses would be a silent
        // interoperability break.
        let mut too_deep = vec![0x81u8; (kotva_cbor::MAX_DEPTH as usize) + 2];
        too_deep.push(0x00); // innermost scalar
        assert_eq!(decode(&too_deep), Err(CborError::Malformed));
    }

    #[test]
    fn accepts_nesting_up_to_the_bound() {
        // A structure nested right up to the limit still decodes (real objects are far shallower).
        // depth 0 is the outermost value; children sit at depth 1.., so MAX_DEPTH nested
        // arrays around a scalar is the deepest accepted shape.
        let n = kotva_cbor::MAX_DEPTH as usize;
        let mut buf = vec![0x81u8; n];
        buf.push(0x00);
        let decoded = decode(&buf).expect("nesting at the bound must decode");
        // And it re-encodes byte-for-byte (idempotence holds for the deep-but-legal case).
        assert_eq!(encode(&decoded), buf);
    }
}
