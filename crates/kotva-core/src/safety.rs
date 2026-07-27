//! Safety numbers — out-of-band key verification (spec §3.4.1).
//!
//! A **safety number** is a deterministic fingerprint of a *pair of identities*, compared
//! out-of-band (in person, over a trusted channel, or by QR) to confirm that the identity you
//! pinned is the one your correspondent meant (§3.4.1). It is **verification, not an address**: it
//! is never used to route or reach anyone and never appears in `Identity.names`.
//!
//! ## Computed over the whole Identity, not a single key (normative, §3.4.1)
//! The fingerprint MUST be taken over each party's **Identity content address**
//! (`Identity_id = 0x1e ‖ BLAKE3-256(det_cbor(Identity))`, §18.9.4 — [`Identity::content_id`]),
//! which commits to **every** suite key in `iks` at once, **never** over a single `ik`.
//! Fingerprinting one suite key would let an attacker who injected a rogue additional suite key
//! (e.g. a forged PQ `0x02` entry) sit behind a pin the user verified only against the classical
//! key; and because adding a suite key is a new Identity version with a different content address,
//! taking the fingerprint over the content address makes the mandated **re-verification on keyset
//! change** fall out automatically. To make that failure unrepresentable, this module's API takes a
//! [`ContentId`] per party — a bare `ik` (a raw suite public key) will not type-check.
//!
//! ## Construction (reference)
//! The spec fixes the *properties* — a deterministic function over the full Identity, order-
//! independent so both parties compute the same value — and leaves the exact *rendering* optional
//! (§3.4.1 "Word rendering (optional)"). This module pins one concrete, documented construction so
//! the value is reproducible and machine-checkable:
//!
//! ```text
//! lo, hi      = sort_bytewise(id_a, id_b)                       ; order independence (content addrs)
//! fingerprint = BLAKE3( "dmtap-safety-number-v0" || lo || hi )  ; 32 bytes, domain-separated
//! ```
//!
//! `fingerprint` is rendered two ways, both deterministic:
//! - [`safety_number`] — a Signal-style **decimal** string: 6 groups of 5 digits (30 digits),
//!   each group = 5 big-endian fingerprint bytes reduced mod 100000, space-separated.
//! - [`safety_number_hex`] — the raw 32-byte fingerprint as lowercase hex, for exact comparison.
//!
//! Rendering the same bits as a §3.4.1 word sequence reuses the [`crate::keyname`] wordlist
//! mechanism and is intentionally left to that module; only the numeric forms are pinned here.

use crate::id::ContentId;

/// Domain-separation label for the safety-number fingerprint, so it can never collide with any
/// other BLAKE3 use in the protocol (content addresses, key-names, …).
const SAFETY_DS: &[u8] = b"dmtap-safety-number-v0";

/// The raw 32-byte safety fingerprint of the pair of **Identity content addresses** `(id_a, id_b)`
/// (spec §3.4.1). Each argument MUST be a party's [`Identity::content_id`](crate::identity::Identity::content_id),
/// which binds the *whole* multi-suite Identity, not any single `ik`.
///
/// **Order-independent:** the two content addresses are sorted bytewise before hashing, so both
/// correspondents derive the identical value regardless of who is "a" and who is "b". Domain-separated.
pub fn fingerprint(id_a: &ContentId, id_b: &ContentId) -> [u8; 32] {
    let (a, b) = (id_a.as_bytes(), id_b.as_bytes());
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    let mut h = blake3::Hasher::new();
    h.update(SAFETY_DS);
    h.update(lo);
    h.update(hi);
    *h.finalize().as_bytes()
}

/// The Signal-style decimal safety number for the identity pair `(id_a, id_b)`: 6 groups of 5
/// digits, space-separated (spec §3.4.1). Deterministic and order-independent (see [`fingerprint`]).
/// Arguments are the parties' whole-Identity content addresses (never a bare `ik`).
pub fn safety_number(id_a: &ContentId, id_b: &ContentId) -> String {
    let fp = fingerprint(id_a, id_b);
    let mut groups = Vec::with_capacity(6);
    for g in 0..6 {
        // 5 big-endian bytes → a 40-bit integer → reduced into a 5-digit group.
        let mut acc: u64 = 0;
        for b in &fp[g * 5..g * 5 + 5] {
            acc = (acc << 8) | *b as u64;
        }
        groups.push(format!("{:05}", acc % 100_000));
    }
    groups.join(" ")
}

/// The raw safety fingerprint as lowercase hex (spec §3.4.1) — the exact-comparison form. Arguments
/// are the parties' whole-Identity content addresses (never a bare `ik`).
pub fn safety_number_hex(id_a: &ContentId, id_b: &ContentId) -> String {
    fingerprint(id_a, id_b)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Two distinct Identity content addresses (0x1e ‖ BLAKE3-256(det_cbor)) as fixtures. The value
    // is defined over the whole-Identity address, so any content-address bytes exercise the render.
    fn addr(tag: &[u8]) -> ContentId {
        ContentId::of(tag)
    }

    #[test]
    fn is_order_independent() {
        let a = addr(b"identity-a");
        let b = addr(b"identity-b");
        assert_eq!(fingerprint(&a, &b), fingerprint(&b, &a));
        assert_eq!(safety_number(&a, &b), safety_number(&b, &a));
    }

    #[test]
    fn distinct_pairs_differ() {
        let a = addr(b"identity-a");
        let b = addr(b"identity-b");
        let c = addr(b"identity-c");
        assert_ne!(fingerprint(&a, &b), fingerprint(&a, &c));
    }

    #[test]
    fn adding_a_suite_key_changes_the_number() {
        // The whole point of §3.4.1: because the fingerprint is over the Identity content address
        // (which commits to every suite key), a keyset change yields a different safety number. Two
        // different Identity versions → different content addresses → different number.
        let v1 = addr(b"identity-classical-only");
        let v2 = addr(b"identity-classical-plus-rogue-pq"); // same party, a suite key added
        assert_ne!(safety_number(&v1, &v2), safety_number(&v1, &v1));
        assert_ne!(fingerprint(&v1, &v2), fingerprint(&v1, &v1));
    }

    #[test]
    fn number_shape_is_six_groups_of_five() {
        let n = safety_number(&addr(b"x"), &addr(b"y"));
        let groups: Vec<&str> = n.split(' ').collect();
        assert_eq!(groups.len(), 6);
        assert!(groups.iter().all(|g| g.len() == 5 && g.chars().all(|c| c.is_ascii_digit())));
    }

    #[test]
    fn hex_is_64_chars() {
        assert_eq!(safety_number_hex(&addr(b"a"), &addr(b"b")).len(), 64);
    }

    #[test]
    fn is_deterministic() {
        let a = addr(b"party-a");
        let b = addr(b"party-b");
        assert_eq!(safety_number_hex(&a, &b), safety_number_hex(&a, &b));
    }
}
