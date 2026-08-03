//! [`DepotSite`] — static-site / SPA serving behaviour (§3.7).
//!
//! A static site is already what PUB is: signed, content-addressed, self-verifying public objects
//! servable over plain HTTPS. It composes as **PUB objects in a public-serving `bucket`, named via
//! REACH**, and adds no registry row and no coordinator kind. A deploy is publishing a new
//! content-addressed root plus a signed announcement superseding the previous one — which makes the
//! switch **atomic** and **rollback** a pointer back to a root that is still addressable.
//!
//! The one thing it needs is this schema, purely so the site stays **portable between providers**
//! (DEPOT-4): without it each operator invents its own hosting config and the site stops being
//! swappable.

use kotva_core::cbor::{self, as_bool, as_text, as_u64, Cv, Fields};
use kotva_core::ContentId;

use crate::{check_hash_shape, DepotError};

/// One redirect rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect {
    /// Source path.
    pub from: String,
    /// Destination path.
    pub to: String,
    /// HTTP status to emit.
    pub status: u64,
}

/// Cache directives for served objects.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CachePolicy {
    /// `max-age` in seconds.
    pub max_age_s: Option<u64>,
    /// Whether objects may be treated as immutable.
    pub immutable: Option<bool>,
}

/// Serving behaviour for a static site (§3.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepotSite {
    /// Content address of the site root manifest (§22).
    pub root: ContentId,
    /// SPA fallback path, e.g. `/index.html`.
    pub fallback: Option<String>,
    /// Redirects, applied **in array order**.
    pub redirects: Vec<Redirect>,
    /// Cache directives.
    pub cache: Option<CachePolicy>,
}

impl DepotSite {
    /// A site with no redirects, no fallback, and no cache policy.
    pub fn new(root: ContentId) -> Self {
        DepotSite {
            root,
            fallback: None,
            redirects: Vec::new(),
            cache: None,
        }
    }

    /// Resolve a request path against this site's rules (§3.7).
    ///
    /// A provider MUST apply `redirects` **in array order**, and — for a path that resolves to no
    /// object — serve `fallback` when present or return **404 when absent, never a guess**.
    /// `object_exists` reports whether the path resolves in the site root.
    pub fn resolve(&self, path: &str, object_exists: impl Fn(&str) -> bool) -> Resolution {
        for r in &self.redirects {
            if r.from == path {
                return Resolution::Redirect {
                    to: r.to.clone(),
                    status: r.status,
                };
            }
        }
        if object_exists(path) {
            return Resolution::Serve(path.to_string());
        }
        match &self.fallback {
            Some(f) => Resolution::Serve(f.clone()),
            None => Resolution::NotFound,
        }
    }

    /// Encode to deterministic CBOR (§18.1.2).
    pub fn det_cbor(&self) -> Vec<u8> {
        let mut m: Vec<(u64, Cv)> = vec![(1, Cv::Bytes(self.root.0.clone()))];
        if let Some(f) = &self.fallback {
            m.push((2, Cv::Text(f.clone())));
        }
        if !self.redirects.is_empty() {
            m.push((
                3,
                Cv::Array(
                    self.redirects
                        .iter()
                        .map(|r| {
                            Cv::Map(vec![
                                (1, Cv::Text(r.from.clone())),
                                (2, Cv::Text(r.to.clone())),
                                (3, Cv::U64(r.status)),
                            ])
                        })
                        .collect(),
                ),
            ));
        }
        if let Some(c) = &self.cache {
            let mut cm: Vec<(u64, Cv)> = Vec::new();
            if let Some(a) = c.max_age_s {
                cm.push((1, Cv::U64(a)));
            }
            if let Some(i) = c.immutable {
                cm.push((2, Cv::Bool(i)));
            }
            m.push((4, Cv::Map(cm)));
        }
        cbor::encode(&Cv::Map(m))
    }

    /// Decode from deterministic CBOR.
    pub fn from_det_cbor(bytes: &[u8]) -> Result<Self, DepotError> {
        let mut f = Fields::from_cv(cbor::decode(bytes)?)?;
        let root_bytes = cbor::as_bytes(f.req(1)?)?;
        check_hash_shape("DepotSite.root", &root_bytes)?;
        let root = ContentId(root_bytes);
        let fallback = f.take(2).map(as_text).transpose()?;

        let mut redirects = Vec::new();
        if let Some(cv) = f.take(3) {
            for item in cbor::as_array(cv)? {
                let mut rf = Fields::from_cv(item)?;
                let from = as_text(rf.req(1)?)?;
                let to = as_text(rf.req(2)?)?;
                let status = as_u64(rf.req(3)?)?;
                rf.deny_unknown()?;
                redirects.push(Redirect { from, to, status });
            }
        }

        let mut cache = None;
        if let Some(cv) = f.take(4) {
            let mut cf = Fields::from_cv(cv)?;
            let max_age_s = cf.take(1).map(as_u64).transpose()?;
            let immutable = cf.take(2).map(as_bool).transpose()?;
            cf.deny_unknown()?;
            cache = Some(CachePolicy {
                max_age_s,
                immutable,
            });
        }

        f.deny_unknown()?;
        Ok(DepotSite {
            root,
            fallback,
            redirects,
            cache,
        })
    }
}

/// What a provider should do with a request path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// Serve this path from the site root.
    Serve(String),
    /// Emit a redirect.
    Redirect {
        /// Destination.
        to: String,
        /// HTTP status.
        status: u64,
    },
    /// No object and no fallback — **404, never a guess** (§3.7).
    NotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site() -> DepotSite {
        DepotSite {
            root: ContentId::of(b"root"),
            fallback: Some("/index.html".into()),
            redirects: vec![
                Redirect {
                    from: "/old".into(),
                    to: "/new".into(),
                    status: 301,
                },
                Redirect {
                    from: "/a".into(),
                    to: "/b".into(),
                    status: 302,
                },
            ],
            cache: Some(CachePolicy {
                max_age_s: Some(3600),
                immutable: Some(true),
            }),
        }
    }

    #[test]
    fn round_trips() {
        let s = site();
        let back = DepotSite::from_det_cbor(&s.det_cbor()).unwrap();
        assert_eq!(s, back);
        assert_eq!(back.det_cbor(), s.det_cbor());
    }

    #[test]
    fn minimal_site_round_trips() {
        let s = DepotSite::new(ContentId::of(b"r"));
        assert_eq!(DepotSite::from_det_cbor(&s.det_cbor()).unwrap(), s);
    }

    #[test]
    fn redirects_apply_in_array_order() {
        let mut s = site();
        s.redirects.insert(
            0,
            Redirect {
                from: "/old".into(),
                to: "/first-wins".into(),
                status: 308,
            },
        );
        assert_eq!(
            s.resolve("/old", |_| true),
            Resolution::Redirect {
                to: "/first-wins".into(),
                status: 308
            }
        );
    }

    #[test]
    fn existing_object_is_served_directly() {
        assert_eq!(
            site().resolve("/style.css", |p| p == "/style.css"),
            Resolution::Serve("/style.css".into())
        );
    }

    #[test]
    fn missing_path_uses_fallback_when_present() {
        assert_eq!(
            site().resolve("/deep/route", |_| false),
            Resolution::Serve("/index.html".into())
        );
    }

    #[test]
    fn missing_path_without_fallback_is_404_never_a_guess() {
        let s = DepotSite::new(ContentId::of(b"r"));
        assert_eq!(s.resolve("/anything", |_| false), Resolution::NotFound);
    }
}
