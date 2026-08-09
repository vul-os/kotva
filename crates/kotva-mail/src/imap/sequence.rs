//! IMAP sequence sets (RFC 9051 §9, `sequence-set`): `1:3,5,*,2:*`.
//!
//! A set is a comma-separated list of numbers and `lo:hi` ranges, where `*` denotes the largest
//! value in use (the last sequence number, or the highest UID). Ranges are inclusive and
//! order-independent (`5:3` == `3:5`, RFC 9051 §9).

/// One endpoint of a range: a fixed number or `*` (the maximum in use).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Point {
    Num(u32),
    Star,
}

impl Point {
    fn resolve(self, max: u32) -> u32 {
        match self {
            Point::Num(n) => n,
            Point::Star => max,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Range {
    lo: Point,
    hi: Point,
}

/// A parsed IMAP sequence set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceSet {
    ranges: Vec<Range>,
    /// The SEARCHRES saved-result reference `$` (RFC 5182). When set, the concrete numbers come
    /// from the session's last-saved SEARCH/SORT result, not from `ranges`; the session
    /// materializes it via [`SequenceSet::from_uids`] before resolving targets.
    saved: bool,
}

impl SequenceSet {
    /// Parse a `sequence-set`. Returns `None` on any malformed token (fail closed). The bare `$`
    /// token (RFC 5182 SEARCHRES) parses to the saved-result reference.
    pub fn parse(s: &str) -> Option<SequenceSet> {
        if s.is_empty() {
            return None;
        }
        if s == "$" {
            return Some(SequenceSet {
                ranges: Vec::new(),
                saved: true,
            });
        }
        // Cap the number of comma-separated ranges. A real client sends at most a handful, but a
        // `1:*,1:*,…` set of hundreds of thousands still fits in one MAX_LINE-sized command and would
        // otherwise drive resolve_targets into a ranges × window scan (a CPU-exhaustion complement to
        // the memory blowup the marker-Vec now bounds). Fail closed on an absurd count.
        const MAX_RANGES: usize = 10_000;
        if s.split(',').count() > MAX_RANGES {
            return None;
        }
        let mut ranges = Vec::new();
        for item in s.split(',') {
            let range = match item.split_once(':') {
                Some((a, b)) => Range {
                    lo: parse_point(a)?,
                    hi: parse_point(b)?,
                },
                None => {
                    let p = parse_point(item)?;
                    Range { lo: p, hi: p }
                }
            };
            ranges.push(range);
        }
        Some(SequenceSet {
            ranges,
            saved: false,
        })
    }

    /// Whether this set is the SEARCHRES saved-result reference `$` (RFC 5182).
    pub fn is_saved(&self) -> bool {
        self.saved
    }

    /// Build a concrete sequence set from an explicit list of numbers (used to materialize a `$`
    /// reference from the session's saved UID/seq list).
    pub fn from_uids(nums: &[u32]) -> SequenceSet {
        SequenceSet {
            ranges: nums
                .iter()
                .map(|&n| Range {
                    lo: Point::Num(n),
                    hi: Point::Num(n),
                })
                .collect(),
            saved: false,
        }
    }

    /// Does `val` fall in the set, given `max` (the largest value in use, for `*`)?
    pub fn contains(&self, val: u32, max: u32) -> bool {
        self.ranges.iter().any(|r| {
            let a = r.lo.resolve(max);
            let b = r.hi.resolve(max);
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            val >= lo && val <= hi
        })
    }

    /// Resolve to concrete sequence numbers in `1..=count` (ascending, de-duplicated).
    pub fn resolve_seqs(&self, count: u32) -> Vec<u32> {
        if count == 0 {
            return Vec::new();
        }
        (1..=count).filter(|&n| self.contains(n, count)).collect()
    }

    /// The normalized `(lo, hi)` ranges (`lo <= hi`) with `*` resolved to `max`. This lets a caller
    /// walk only the matched span — e.g. a targeted `UID FETCH 5` yields `[(5, 5)]` and can be
    /// answered with an `O(log n)` binary search instead of scanning the whole mailbox.
    pub fn ranges_resolved(&self, max: u32) -> Vec<(u32, u32)> {
        self.ranges
            .iter()
            .map(|r| {
                let a = r.lo.resolve(max);
                let b = r.hi.resolve(max);
                if a <= b {
                    (a, b)
                } else {
                    (b, a)
                }
            })
            .collect()
    }

    /// Build an `O(log R)`-membership view (see [`Membership`]) by sorting and coalescing the
    /// resolved ranges. Use this instead of repeated [`Self::contains`] when probing MANY values
    /// against a set whose range count may be large — notably a SEARCHRES `$` set materialized via
    /// [`Self::from_uids`], which has one range per saved UID and bypasses the `MAX_RANGES` cap. A
    /// per-probe `contains` is `O(R)`, so probing `V` values is `O(V·R)` ≈ `O(n²)`; this makes it
    /// `O(R log R)` to build + `O(V log R)` to probe — the same asymmetric-DoS class the
    /// `resolve_targets` binary-search path already eliminated for FETCH/STORE/COPY/EXPUNGE.
    pub fn membership(&self, max: u32) -> Membership {
        let mut r = self.ranges_resolved(max);
        r.sort_unstable_by_key(|&(lo, _)| lo);
        let mut coalesced: Vec<(u32, u32)> = Vec::with_capacity(r.len());
        for (lo, hi) in r {
            match coalesced.last_mut() {
                // Merge overlapping OR adjacent ranges so the search sees disjoint, ordered spans.
                Some(last) if lo <= last.1.saturating_add(1) => last.1 = last.1.max(hi),
                _ => coalesced.push((lo, hi)),
            }
        }
        Membership { ranges: coalesced }
    }
}

/// A sorted, coalesced view of a [`SequenceSet`]'s concrete ranges supporting `O(log R)` membership
/// tests. Built once via [`SequenceSet::membership`] and probed many times — the bounded-cost
/// alternative to calling [`SequenceSet::contains`] (an `O(R)` linear scan) in a hot loop.
pub struct Membership {
    ranges: Vec<(u32, u32)>,
}

impl Membership {
    /// Does `uid` fall in the set? `O(log R)` binary search over the disjoint, ordered ranges.
    pub fn contains(&self, uid: u32) -> bool {
        self.ranges
            .binary_search_by(|&(lo, hi)| {
                if uid < lo {
                    std::cmp::Ordering::Greater
                } else if uid > hi {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .is_ok()
    }
}

fn parse_point(s: &str) -> Option<Point> {
    let s = s.trim();
    if s == "*" {
        Some(Point::Star)
    } else {
        s.parse::<u32>().ok().filter(|&n| n > 0).map(Point::Num)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_resolves() {
        let set = SequenceSet::parse("1:3,5,*").unwrap();
        assert_eq!(set.resolve_seqs(6), vec![1, 2, 3, 5, 6]);
    }

    #[test]
    fn range_is_order_independent() {
        let set = SequenceSet::parse("5:3").unwrap();
        assert_eq!(set.resolve_seqs(10), vec![3, 4, 5]);
    }

    #[test]
    fn membership_matches_contains() {
        // The O(log R) membership view must be identical to the O(R) contains scan it replaces on
        // the QRESYNC/VANISHED hot paths — including reversed ranges, `*`, and a `$`-style from_uids
        // set (one point-range per UID, the uncapped case membership was introduced to bound).
        let set = SequenceSet::parse("5,1:3,10:8,2:*").unwrap();
        let m = set.membership(100);
        for uid in 0..=110u32 {
            assert_eq!(
                m.contains(uid),
                set.contains(uid, 100),
                "divergence at uid {uid}"
            );
        }
        // from_uids (the `$` materialization) with duplicate/unordered points coalesces correctly.
        let f = SequenceSet::from_uids(&[100, 3, 3, 4, 5, 50]);
        let fm = f.membership(200);
        for uid in [3u32, 4, 5, 50, 100] {
            assert!(fm.contains(uid), "missing {uid}");
        }
        for uid in [0u32, 2, 6, 49, 51, 99, 101] {
            assert!(!fm.contains(uid), "false positive {uid}");
        }
    }

    #[test]
    fn star_range() {
        let set = SequenceSet::parse("2:*").unwrap();
        assert_eq!(set.resolve_seqs(4), vec![2, 3, 4]);
        assert!(set.contains(100, 100));
    }

    #[test]
    fn rejects_malformed() {
        assert!(SequenceSet::parse("").is_none());
        assert!(SequenceSet::parse("abc").is_none());
        assert!(SequenceSet::parse("0").is_none());
        assert!(SequenceSet::parse("1:").is_none());
        assert!(SequenceSet::parse("1:2:3").is_none());
        assert!(SequenceSet::parse(",").is_none());
    }

    #[test]
    fn ranges_resolved_normalizes_and_resolves_star() {
        let set = SequenceSet::parse("5:3,10,*").unwrap();
        assert_eq!(set.ranges_resolved(100), vec![(3, 5), (10, 10), (100, 100)]);
    }
}
