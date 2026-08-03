//! [`DepotImage`] — **one schema, three targets** (§4.1).
//!
//! An image is immutable content-addressed bytes in a `bucket` plus a manifest. It is deliberately
//! **not** a fifth elemental: an OCI registry is a content-addressed blob store with a manifest
//! convention on top, and "manifest convention over content-addressed bytes" is a schema, not a
//! service (§3.7).
//!
//! The one thing a bucket does not supply — a **mutable tag** (`myapp:latest` → digest) — is a
//! signed §22 `PubAnnounce` superseding the previous one: atomic, attributable, and better than a
//! registry tag because the superseded digest stays addressable, so rollback is a pointer rather
//! than a rebuild.
//!
//! This schema covers what every cloud keeps in three separate systems — VM images, function
//! artefacts, and volume snapshots — because all three are an immutable artefact plus a manifest
//! plus a signed tag.

use kotva_core::cbor::{self, as_bytes, as_text, as_u64, Cv, Fields};
use kotva_core::ContentId;

use crate::service::Service;
use crate::{canonical_key_cmp, DepotError};

/// What a [`DepotImage`] instantiates (§4.1, CLOSED).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageTarget {
    /// A machine image — `qcow2` / `raw`.
    Box,
    /// A function artefact — `oci` / `wasm`, or a quantum circuit (§3.1).
    EdgeFn,
    /// A **snapshot** — `fs-dump` / `raw`. Note a snapshot is not automatically an export (below).
    Volume,
}

impl ImageTarget {
    /// The wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            ImageTarget::Box => "box",
            ImageTarget::EdgeFn => "edge-fn",
            ImageTarget::Volume => "volume",
        }
    }

    /// Parse; CLOSED, fails closed. Note `bucket` is deliberately absent — a bucket holds images,
    /// it is not instantiated from one.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "box" => ImageTarget::Box,
            "edge-fn" => ImageTarget::EdgeFn,
            "volume" => ImageTarget::Volume,
            _ => return None,
        })
    }

    /// The elemental this target instantiates.
    pub fn service(self) -> Service {
        match self {
            ImageTarget::Box => Service::Box,
            ImageTarget::EdgeFn => Service::EdgeFn,
            ImageTarget::Volume => Service::Volume,
        }
    }
}

/// Artefact format (§4.1, CLOSED registry — extended by registry addition).
///
/// An unrecognised format MUST be refused, never guessed at: booting an artefact you have
/// misidentified is the failure this closed set exists to prevent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ImageFormat {
    /// Raw block image.
    Raw,
    /// QEMU copy-on-write v2.
    Qcow2,
    /// An OCI image.
    Oci,
    /// A WASI/WebAssembly module.
    Wasm,
    /// Quantum Intermediate Representation — a QPU is an `edge-fn`, not a box (§3.1).
    Qir,
    /// OpenQASM circuit text.
    Qasm,
    /// A filesystem dump.
    FsDump,
}

impl ImageFormat {
    /// The wire string.
    pub fn as_str(self) -> &'static str {
        match self {
            ImageFormat::Raw => "raw",
            ImageFormat::Qcow2 => "qcow2",
            ImageFormat::Oci => "oci",
            ImageFormat::Wasm => "wasm",
            ImageFormat::Qir => "qir",
            ImageFormat::Qasm => "qasm",
            ImageFormat::FsDump => "fs-dump",
        }
    }

    /// Parse; CLOSED, fails closed.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        Some(match s {
            "raw" => ImageFormat::Raw,
            "qcow2" => ImageFormat::Qcow2,
            "oci" => ImageFormat::Oci,
            "wasm" => ImageFormat::Wasm,
            "qir" => ImageFormat::Qir,
            "qasm" => ImageFormat::Qasm,
            "fs-dump" => ImageFormat::FsDump,
            _ => return None,
        })
    }
}

/// A content-addressed artefact plus its manifest (§4.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepotImage {
    /// What it instantiates.
    pub target: ImageTarget,
    /// Artefact format.
    pub format: ImageFormat,
    /// Content address of the artefact, held in a `bucket`.
    pub digest: ContentId,
    /// Artefact size in bytes.
    pub bytes: u64,
    /// Compatibility predicate, matching `attributes.arch` (§3.2).
    pub arch: Option<String>,
    /// Engine hints — a cloud-init dataset reference, an OCI entrypoint, …
    pub boot: Vec<(String, String)>,
    /// Content address of the image this derives from.
    pub parent: Option<ContentId>,
}

impl DepotImage {
    /// A minimal image manifest.
    pub fn new(target: ImageTarget, format: ImageFormat, digest: ContentId, bytes: u64) -> Self {
        DepotImage {
            target,
            format,
            digest,
            bytes,
            arch: None,
            boot: Vec::new(),
            parent: None,
        }
    }

    /// Encode to deterministic CBOR (§18.1.2).
    pub fn det_cbor(&self) -> Vec<u8> {
        let mut m: Vec<(u64, Cv)> = vec![
            (1, Cv::Text(self.target.as_str().to_string())),
            (2, Cv::Text(self.format.as_str().to_string())),
            (3, Cv::Bytes(self.digest.0.clone())),
            (4, Cv::U64(self.bytes)),
        ];
        if let Some(a) = &self.arch {
            m.push((5, Cv::Text(a.clone())));
        }
        if !self.boot.is_empty() {
            let mut b: Vec<(String, Cv)> = self
                .boot
                .iter()
                .map(|(k, v)| (k.clone(), Cv::Text(v.clone())))
                .collect();
            b.sort_by(|x, y| canonical_key_cmp(&x.0, &y.0));
            m.push((6, Cv::TextMap(b)));
        }
        if let Some(p) = &self.parent {
            m.push((7, Cv::Bytes(p.0.clone())));
        }
        cbor::encode(&Cv::Map(m))
    }

    /// Decode from deterministic CBOR, failing closed on an unknown target or format.
    pub fn from_det_cbor(bytes: &[u8]) -> Result<Self, DepotError> {
        let mut f = Fields::from_cv(cbor::decode(bytes)?)?;

        let t_s = as_text(f.req(1)?)?;
        let target = ImageTarget::from_str(&t_s).ok_or(DepotError::UnknownRegistryValue {
            registry: "image-target",
            value: t_s,
        })?;

        let fmt_s = as_text(f.req(2)?)?;
        let format = ImageFormat::from_str(&fmt_s).ok_or(DepotError::UnknownRegistryValue {
            registry: "image-format",
            value: fmt_s,
        })?;

        let digest = ContentId(as_bytes(f.req(3)?)?);
        let size = as_u64(f.req(4)?)?;
        let arch = f.take(5).map(as_text).transpose()?;

        let mut boot = Vec::new();
        if let Some(Cv::TextMap(pairs)) = f.take(6) {
            for (k, v) in pairs {
                boot.push((k, as_text(v)?));
            }
        }

        let parent = f.take(7).map(as_bytes).transpose()?.map(ContentId);

        f.deny_unknown()?;
        Ok(DepotImage {
            target,
            format,
            digest,
            bytes: size,
            arch,
            boot,
            parent,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cid(seed: &[u8]) -> ContentId {
        ContentId::of(seed)
    }

    #[test]
    fn round_trips_all_three_targets() {
        for (target, format) in [
            (ImageTarget::Box, ImageFormat::Qcow2),
            (ImageTarget::EdgeFn, ImageFormat::Oci),
            (ImageTarget::Volume, ImageFormat::FsDump),
        ] {
            let img = DepotImage {
                arch: Some("x86_64".into()),
                boot: vec![
                    ("cloud-init".into(), "ds=nocloud".into()),
                    ("entrypoint".into(), "/usr/bin/app".into()),
                ],
                parent: Some(cid(b"parent")),
                ..DepotImage::new(target, format, cid(b"artefact"), 4096)
            };
            let back = DepotImage::from_det_cbor(&img.det_cbor()).unwrap();
            assert_eq!(img, back);
            assert_eq!(back.det_cbor(), img.det_cbor());
        }
    }

    #[test]
    fn minimal_image_round_trips() {
        let img = DepotImage::new(ImageTarget::EdgeFn, ImageFormat::Wasm, cid(b"m"), 1);
        assert_eq!(DepotImage::from_det_cbor(&img.det_cbor()).unwrap(), img);
    }

    #[test]
    fn unknown_format_fails_closed() {
        let bad = cbor::encode(&Cv::Map(vec![
            (1, Cv::Text("box".into())),
            (2, Cv::Text("vmdk-proprietary".into())),
            (3, Cv::Bytes(cid(b"x").0)),
            (4, Cv::U64(1)),
        ]));
        assert!(matches!(
            DepotImage::from_det_cbor(&bad),
            Err(DepotError::UnknownRegistryValue {
                registry: "image-format",
                ..
            })
        ));
    }

    #[test]
    fn unknown_target_fails_closed_including_bucket() {
        // `bucket` is deliberately not a target: a bucket holds images, it is not made from one.
        for bad_target in ["bucket", "vm", ""] {
            let bad = cbor::encode(&Cv::Map(vec![
                (1, Cv::Text(bad_target.into())),
                (2, Cv::Text("raw".into())),
                (3, Cv::Bytes(cid(b"x").0)),
                (4, Cv::U64(1)),
            ]));
            assert!(
                matches!(
                    DepotImage::from_det_cbor(&bad),
                    Err(DepotError::UnknownRegistryValue {
                        registry: "image-target",
                        ..
                    })
                ),
                "{bad_target:?} must not parse as a target"
            );
        }
    }

    #[test]
    fn target_maps_to_its_elemental() {
        assert_eq!(ImageTarget::Box.service(), Service::Box);
        assert_eq!(ImageTarget::EdgeFn.service(), Service::EdgeFn);
        assert_eq!(ImageTarget::Volume.service(), Service::Volume);
    }

    #[test]
    fn format_registry_round_trips_and_is_closed() {
        // There is deliberately no `is_interchange()` predicate: every format in the v0 registry is
        // an open standard, so such a method would return `true` unconditionally — a check that
        // examines nothing while reading as a DEPOT-4 guard. Whether an export satisfies DEPOT-4 is
        // a conformance-vector question (`export-conformance`, §7), not a runtime branch. If a
        // vendor format is ever added to the registry, that is when the predicate earns its place.
        for f in [
            ImageFormat::Raw,
            ImageFormat::Qcow2,
            ImageFormat::Oci,
            ImageFormat::Wasm,
            ImageFormat::Qir,
            ImageFormat::Qasm,
            ImageFormat::FsDump,
        ] {
            assert_eq!(ImageFormat::from_str(f.as_str()), Some(f), "{f:?}");
        }
    }
}
