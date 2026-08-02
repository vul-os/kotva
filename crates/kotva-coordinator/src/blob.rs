//! [`DetCbor`] — the opaque deterministic-CBOR payload carried by `CoordinatorDescriptor.policy`
//! (§18.8a.1 key 5), `Tariff.schedule` (key 3) and `UsageReceipt.operation` (§18.8a.2 key 3).
//!
//! §18 does not interpret any of the three. `policy` "exists so §2.1's 'no reputation/price/stake
//! field' rule has exactly one escape hatch (self-declared, never a ranking input) rather than a
//! slow accretion of new top-level descriptor keys"; `schedule` carries numbers that are "operator
//! policy and out of scope for this specification"; `operation` is "kind-specific". This type
//! carries and signs over the bytes without reading them.
//!
//! It is a distinct type rather than a bare `Vec<u8>` for one reason: **it is where an implementer
//! hand-builds a text-keyed map, and text-keyed maps are where canonical ordering goes wrong.**

use kotva_core::cbor::{self, Cv, CborError};

use crate::canonical_key_cmp;

/// Opaque deterministic-CBOR bytes (§18.1.1 / RFC 8949 §4.2).
///
/// An **empty** payload means "nothing declared" and is valid — it is not itself required to be a
/// decodable CBOR item, which is why [`DetCbor::decode`] is fallible rather than an invariant.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DetCbor(pub Vec<u8>);

impl DetCbor {
    /// An empty payload — "nothing declared".
    pub fn empty() -> Self {
        DetCbor(Vec::new())
    }

    /// Wrap raw bytes that are already canonical (e.g. `kotva_depot::DepotServicePolicy::det_cbor`).
    ///
    /// Not validated here: a profile crate's encoder is the authority on its own blob, and
    /// re-decoding it would only prove `kotva_core::cbor` agrees with itself. Use
    /// [`DetCbor::decode`] where a *received* blob must be proven canonical before use.
    pub fn from_bytes(b: impl Into<Vec<u8>>) -> Self {
        DetCbor(b.into())
    }

    /// Encode a value tree as canonical deterministic CBOR.
    pub fn from_cv(cv: &Cv) -> Self {
        DetCbor(cbor::encode(cv))
    }

    /// Build a **text-keyed** blob with the keys in canonical CBOR order (§18.1.1 rule 2).
    ///
    /// Canonical CBOR sorts map entries by their **encoded key bytes**, and a text string's head
    /// encodes its length monotonically — so the order is **length first, then bytewise**, not
    /// lexicographic. `"gb"` precedes `"byte"`. A naive `sort()` produces the other order, and the
    /// resulting bytes are not merely unusual: the strict decoder **rejects** them
    /// (`CborError::MapKeyOrder`), so the mistake surfaces as an interop failure at a third party
    /// rather than as a local test failure. The `schedule` blob — a price map keyed by unit names —
    /// is exactly the shape that trips it.
    ///
    /// Duplicate keys are not deduplicated here; the encoder emits what it is given and the strict
    /// decoder rejects duplicates, so a caller passing the same key twice gets a hard failure at the
    /// first decode rather than a silently-dropped entry.
    pub fn from_text_map(pairs: impl IntoIterator<Item = (String, Cv)>) -> Self {
        let mut v: Vec<(String, Cv)> = pairs.into_iter().collect();
        v.sort_by(|a, b| canonical_key_cmp(&a.0, &b.0));
        DetCbor(cbor::encode(&Cv::TextMap(v)))
    }

    /// Decode this payload as canonical deterministic CBOR, failing closed on any non-canonical or
    /// malformed byte (§18.1.1) — never guessing at a lenient re-encoding.
    pub fn decode(&self) -> Result<Cv, CborError> {
        cbor::decode(&self.0)
    }

    /// The raw bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Whether this is the empty "nothing declared" payload.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_map_keys_sort_length_first_not_lexicographically() {
        // "gb" (2 bytes) sorts BEFORE "byte" (4 bytes) because the encoded key head carries the
        // length. Lexicographically "byte" < "gb", so a naive sort produces the opposite order.
        let blob = DetCbor::from_text_map([
            ("byte".to_string(), Cv::U64(5)),
            ("gb".to_string(), Cv::U64(3)),
        ]);
        // a2 | 62 "gb" 03 | 64 "byte" 05
        assert_eq!(
            blob.as_bytes(),
            &[0xa2, 0x62, b'g', b'b', 0x03, 0x64, b'b', b'y', b't', b'e', 0x05]
        );
        // The order survives the round-trip, so a decoded blob compares equal to a rebuilt one.
        assert_eq!(
            blob.decode().unwrap(),
            Cv::TextMap(vec![
                ("gb".into(), Cv::U64(3)),
                ("byte".into(), Cv::U64(5)),
            ])
        );
    }

    #[test]
    fn the_lexicographic_ordering_is_actually_rejected_by_the_strict_decoder() {
        // The false-positive control for the test above: prove the wrong order is not merely
        // "different bytes we happen not to emit" but bytes the decoder refuses. Without this, a
        // `canonical_key_cmp` that silently degraded to lexicographic would look like a cosmetic
        // difference rather than an interop break.
        let wrong = cbor::encode(&Cv::TextMap(vec![
            ("byte".into(), Cv::U64(5)),
            ("gb".into(), Cv::U64(3)),
        ]));
        // `cbor::encode` re-sorts canonically regardless of insertion order, so it cannot be used
        // to build the broken form — the bytes have to be laid out by hand.
        assert_eq!(wrong, DetCbor::from_text_map([
            ("byte".to_string(), Cv::U64(5)),
            ("gb".to_string(), Cv::U64(3)),
        ]).0);
        let hand_built_wrong_order: Vec<u8> = vec![
            0xa2, 0x64, b'b', b'y', b't', b'e', 0x05, 0x62, b'g', b'b', 0x03,
        ];
        assert_eq!(
            cbor::decode(&hand_built_wrong_order),
            Err(CborError::MapKeyOrder)
        );
    }

    #[test]
    fn empty_is_valid_but_is_not_a_decodable_item() {
        let e = DetCbor::empty();
        assert!(e.is_empty());
        assert!(e.decode().is_err(), "empty is 'nothing declared', not a CBOR item");
        assert_eq!(e, DetCbor::default());
    }

    #[test]
    fn from_cv_round_trips() {
        let cv = Cv::Map(vec![(1, Cv::U64(7)), (2, Cv::Text("za-jhb".into()))]);
        let b = DetCbor::from_cv(&cv);
        assert_eq!(b.decode().unwrap(), cv);
        assert_eq!(DetCbor::from_bytes(b.as_bytes().to_vec()), b);
    }
}
