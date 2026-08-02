//! DEPOT v0 conformance vectors — **hand-derived from the CDDL in `profiles/cloud.md`**, byte by
//! byte, and deliberately *not* produced by this crate's encoder.
//!
//! # Why these exist and why they are hand-written
//!
//! The unit tests in `src/` are round-trips: `decode(encode(x)) == x`. A round-trip proves the
//! encoder and decoder agree **with each other**, and would pass unchanged if both were wrong in the
//! same way — a wrong major type, a wrong key number, a non-canonical map order. It cannot detect a
//! systematic disagreement with the spec, because the spec never enters the loop.
//!
//! Every byte below was written by reading the CDDL and RFC 8949, then checked against the
//! implementation. That makes them an **independent second opinion** rather than a snapshot: if the
//! encoder's key numbering drifts, or a `tstr` becomes a `bytes`, or canonical map ordering breaks,
//! these fail and the round-trip tests do not.
//!
//! Each vector is annotated with its full byte decomposition so a second implementation — in any
//! language — can be checked against the same corpus without running this crate.

use kotva_depot::{
    Backing, DepotFormula, DepotImage, DepotMeasurement, DepotServicePolicy, DepotSite, ImageFormat,
    ImageTarget, Metric, Method, Service,
};
use kotva_depot::measurement::MeasurementValue;
use kotva_core::ContentId;

/// Decode a hex string into bytes. Vectors are written as hex so the byte layout is visible in
/// review rather than hidden behind a builder.
fn hex(s: &str) -> Vec<u8> {
    assert!(s.len() % 2 == 0, "odd-length hex vector");
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("bad hex"))
        .collect()
}

/// `DepotServicePolicy { service: "bucket", backing: "operator" }` — the minimal policy (§3.3).
///
/// ```text
/// a2                          map(2)
///   01                        unsigned(1)        ; service
///   66 62 75 63 6b 65 74      text(6) "bucket"
///   02                        unsigned(2)        ; backing
///   68 6f 70 65 72 61 74 6f 72 text(8) "operator"
/// ```
const POLICY_MINIMAL: &str = "a201666275636b657402686f70657261746f72";

/// `DepotImage { target: "edge-fn", format: "wasm", digest: h'010203', bytes: 4096 }` (§4.1).
///
/// ```text
/// a4                          map(4)
///   01 67 65 64 67 65 2d 66 6e  1 : text(7) "edge-fn"
///   02 64 77 61 73 6d           2 : text(4) "wasm"
///   03 43 01 02 03              3 : bytes(3) 010203
///   04 19 10 00                 4 : unsigned(4096)  ; uint16 form, 0x1000
/// ```
const IMAGE_EDGE_FN: &str = "a40167656467652d666e02647761736d034301020304191000";

/// `DepotMeasurement { service: "box", metric: "uptime", value: 995, method: "probe",
/// observed_at: 1 }` (§7). `value` is per-mille, so 995 encodes as uint16 `0x03e3`.
///
/// ```text
/// a5                          map(5)
///   01 63 62 6f 78              1 : text(3) "box"
///   02 66 75 70 74 69 6d 65     2 : text(6) "uptime"
///   03 19 03 e3                 3 : unsigned(995)
///   04 65 70 72 6f 62 65        4 : text(5) "probe"
///   05 01                       5 : unsigned(1)
/// ```
const MEASUREMENT_UPTIME: &str = "a50163626f780266757074696d65031903e3046570726f62650501";

/// `DepotSite { root: h'010203' }` — the minimal site (§3.7).
///
/// ```text
/// a1 01 43 01 02 03           map(1) { 1 : bytes(3) 010203 }
/// ```
const SITE_MINIMAL: &str = "a10143010203";

/// `DepotFormula { kind: "redis", parts: [{ service: "box", provider: h'aa' }] }` (§3.6).
///
/// ```text
/// a2                          map(2)
///   01 65 72 65 64 69 73        1 : text(5) "redis"
///   02 81                       2 : array(1)
///      a2                          map(2)
///        01 63 62 6f 78              1 : text(3) "box"
///        02 41 aa                    2 : bytes(1) aa
/// ```
const FORMULA_REDIS: &str = "a2016572656469730281a20163626f780241aa";

#[test]
fn policy_vector_decodes_to_the_specified_value() {
    let p = DepotServicePolicy::from_det_cbor(&hex(POLICY_MINIMAL)).expect("vector must decode");
    assert_eq!(p.service, Service::Bucket);
    assert_eq!(p.backing, Backing::Operator);
    assert!(p.capacity.is_none());
    assert!(p.attributes.is_empty());
    assert!(p.abilities.is_empty());
}

#[test]
fn image_vector_decodes_to_the_specified_value() {
    let i = DepotImage::from_det_cbor(&hex(IMAGE_EDGE_FN)).expect("vector must decode");
    assert_eq!(i.target, ImageTarget::EdgeFn);
    assert_eq!(i.format, ImageFormat::Wasm);
    assert_eq!(i.digest, ContentId(vec![1, 2, 3]));
    assert_eq!(i.bytes, 4096);
    assert_eq!(i.arch, None);
    assert!(i.boot.is_empty());
    assert_eq!(i.parent, None);
}

#[test]
fn measurement_vector_decodes_to_the_specified_value() {
    let m = DepotMeasurement::from_det_cbor(&hex(MEASUREMENT_UPTIME)).expect("vector must decode");
    assert_eq!(m.service, Service::Box);
    assert_eq!(m.metric, Metric::Uptime);
    assert_eq!(m.value, MeasurementValue::Uint(995));
    assert_eq!(m.method, Method::Probe);
    assert_eq!(m.observed_at, 1);
    assert_eq!(m.evidence, None);
    assert!(m.is_wellformed());
}

#[test]
fn site_vector_decodes_to_the_specified_value() {
    let s = DepotSite::from_det_cbor(&hex(SITE_MINIMAL)).expect("vector must decode");
    assert_eq!(s.root, ContentId(vec![1, 2, 3]));
    assert_eq!(s.fallback, None);
    assert!(s.redirects.is_empty());
    assert_eq!(s.cache, None);
}

#[test]
fn formula_vector_decodes_to_the_specified_value() {
    let f = DepotFormula::from_det_cbor(&hex(FORMULA_REDIS)).expect("vector must decode");
    assert_eq!(f.kind, "redis");
    assert_eq!(f.parts.len(), 1);
    assert_eq!(f.parts[0].service, Service::Box);
    assert_eq!(f.parts[0].provider, vec![0xaa]);
    assert_eq!(f.parts[0].descriptor, None);
    assert_eq!(f.recipe, None);
    assert_eq!(f.consensus, None);
}

/// The direction that actually catches an encoder that drifted from the spec.
///
/// Decoding a vector only proves the decoder is permissive enough to accept it. Re-encoding the
/// decoded value and demanding the **original bytes back** is what pins the encoder: a changed key
/// number, a `tstr` that became a `bytes`, a lost canonical ordering, or a different integer width
/// all fail here while every round-trip test in `src/` continues to pass.
#[test]
fn re_encoding_each_vector_reproduces_the_specified_bytes() {
    let cases: Vec<(&str, &str, Vec<u8>)> = vec![
        (
            "DepotServicePolicy",
            POLICY_MINIMAL,
            DepotServicePolicy::from_det_cbor(&hex(POLICY_MINIMAL))
                .unwrap()
                .det_cbor(),
        ),
        (
            "DepotImage",
            IMAGE_EDGE_FN,
            DepotImage::from_det_cbor(&hex(IMAGE_EDGE_FN)).unwrap().det_cbor(),
        ),
        (
            "DepotMeasurement",
            MEASUREMENT_UPTIME,
            DepotMeasurement::from_det_cbor(&hex(MEASUREMENT_UPTIME))
                .unwrap()
                .det_cbor(),
        ),
        (
            "DepotSite",
            SITE_MINIMAL,
            DepotSite::from_det_cbor(&hex(SITE_MINIMAL)).unwrap().det_cbor(),
        ),
        (
            "DepotFormula",
            FORMULA_REDIS,
            DepotFormula::from_det_cbor(&hex(FORMULA_REDIS)).unwrap().det_cbor(),
        ),
    ];
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
/// above would pass while proving nothing — the failure mode this repo has hit repeatedly. These
/// mutations MUST be rejected.
#[test]
fn corrupted_vectors_are_rejected() {
    // Truncation.
    assert!(DepotServicePolicy::from_det_cbor(&hex(&POLICY_MINIMAL[..10])).is_err());
    // Empty input is not a valid object.
    assert!(DepotServicePolicy::from_det_cbor(&[]).is_err());
    // A required key removed: map(2) -> map(1) header with the backing entry still present is
    // malformed; dropping to a genuine map(1) loses `backing`, which is REQUIRED (§3.3).
    assert!(DepotServicePolicy::from_det_cbor(&hex("a101666275636b6574")).is_err());
    // An unknown closed-registry value in an otherwise well-formed object.
    //   a2 01 62 "vm" 02 68 "operator"
    assert!(DepotServicePolicy::from_det_cbor(&hex("a20162766d02686f70657261746f72")).is_err());
    // A vector whose value type is wrong: image size (key 4) as text(1) "x" rather than a uint.
    //   a4 01 67 "edge-fn" 02 64 "wasm" 03 43 010203 04 61 78
    assert!(DepotImage::from_det_cbor(&hex("a40167656467652d666e02647761736d0343010203046178")).is_err());
    // And the same object with the size restored is accepted — a false-positive control, so the
    // assertion above is known to fail on the mutation rather than on some unrelated defect.
    assert!(DepotImage::from_det_cbor(&hex(IMAGE_EDGE_FN)).is_ok());
}
