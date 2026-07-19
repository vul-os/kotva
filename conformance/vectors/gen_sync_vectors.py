#!/usr/bin/env python3
"""
gen_sync_vectors.py — generates conformance/vectors/sync_vectors.json

Throwaway, deterministic vector generator for the Sync substrate capability
(substrate/SYNC.md §10 conformance-vector stubs). Mirrors the approach and
conventions of gen_pub_vectors.py exactly (same rationale: no reference
implementation yet exists for this wire shape — SYNC.md itself is "the one
genuinely new normative specification" in the substrate, and dmtap-core does
not implement it).

Scope discipline (freeze-check, per the wave that added this file): this
script freezes ONLY the SYNC-* stubs whose byte-exact inputs/outputs are
FULLY determined by substrate/SYNC.md's text with no further design choice
required of the generator. Stubs whose wire shape is genuinely underspecified
(the COSE_Sign1 envelope framing for SYNC-OP-02, the canonical "observable
state" CBOR schema for SYNC-SNAP-01/02, the range-Merkle fingerprint "fold"
function for SYNC-RECON-01, and the apparent earlier/later-wins ambiguity in
SYNC-TREE-01's cycle-resolution phrasing vs. §4.8's body text) are left as
stubs in substrate/SYNC.md, each with a NOT-FROZEN marker explaining why —
see that document, not this script, for the authoritative list.

Dependencies: `pip install cryptography` (Ed25519 pubkey derivation only;
these vectors do not exercise signing — see NOT-FROZEN note on SYNC-OP-02).
Everything below is a FIXED constant: fixed 32-byte Ed25519 seeds, fixed
HLC wall-clock values. No randomness, no wall-clock reads.

Run: python3 conformance/vectors/gen_sync_vectors.py > conformance/vectors/sync_vectors.json
"""
import json
from cryptography.hazmat.primitives.asymmetric import ed25519
from cryptography.hazmat.primitives import serialization

# ── fixed test constants (no randomness, no timestamps read from the clock) ──────────────
SEED_SYNC_A = bytes([0xCC] * 32)   # author A — admitted in every scenario below
SEED_SYNC_B = bytes([0xDD] * 32)   # author B — a second admitted author (cross-author cases)
SEED_SYNC_X = bytes([0xEE] * 32)   # author X — NOT admitted (SYNC-AUTH-01 reject case)

HLC_WALL = 1_700_000_100_000  # ms epoch; fixed, distinct from gen_pub_vectors.py's TS_FIXED
                               # so the two vector families are visibly independent


def keypair(seed: bytes):
    sk = ed25519.Ed25519PrivateKey.from_private_bytes(seed)
    pk = sk.public_key().public_bytes(
        encoding=serialization.Encoding.Raw, format=serialization.PublicFormat.Raw
    )
    return sk, pk


_, PK_A = keypair(SEED_SYNC_A)
_, PK_B = keypair(SEED_SYNC_B)
_, PK_X = keypair(SEED_SYNC_X)


# ── minimal deterministic (RFC 8949 §4.2 canonical) CBOR encoder ─────────────────────────
# Same subset as gen_pub_vectors.py, plus negative integers (PN-counter deltas) and bool
# (SyncOp itself needs no bool field, but ext-value (§18.3.6) allows it; included for
# completeness / future reuse). Integer-keyed maps, ascending key order, definite lengths,
# shortest-form integers — §18.1.1.
def _enc_head(major: int, n: int) -> bytes:
    m = major << 5
    if n < 24:
        return bytes([m | n])
    if n < 2**8:
        return bytes([m | 24, n])
    if n < 2**16:
        return bytes([m | 25]) + n.to_bytes(2, "big")
    if n < 2**32:
        return bytes([m | 26]) + n.to_bytes(4, "big")
    return bytes([m | 27]) + n.to_bytes(8, "big")


def enc_uint(n: int) -> bytes:
    return _enc_head(0, n)


def enc_int(n: int) -> bytes:
    """Signed integer, CBOR major type 0 (>=0) or 1 (<0, encoded as -1-n)."""
    return enc_uint(n) if n >= 0 else _enc_head(1, -1 - n)


def enc_bstr(b: bytes) -> bytes:
    return _enc_head(2, len(b)) + b


def enc_tstr(s: str) -> bytes:
    b = s.encode("utf-8")
    return _enc_head(3, len(b)) + b


def enc_bool(b: bool) -> bytes:
    return bytes([0xF5 if b else 0xF4])


def enc_array(items) -> bytes:
    out = _enc_head(4, len(items))
    for it in items:
        out += it
    return out


def enc_map(pairs) -> bytes:
    """pairs: list of (int_key, encoded_value_bytes); sorted ascending by key (canonical)."""
    pairs = sorted(pairs, key=lambda kv: kv[0])
    out = _enc_head(5, len(pairs))
    for k, v in pairs:
        out += enc_uint(k) + v
    return out


# ── Hlc (§3) — {1: wall u64, 2: counter u32, 3: author ik-pub} ───────────────────────────
def encode_hlc(wall: int, counter: int, author: bytes) -> bytes:
    return enc_map([(1, enc_uint(wall)), (2, enc_uint(counter)), (3, enc_bstr(author))])


def hlc_tuple(wall, counter, author_hex):
    """(wall, counter, author) — the §3 total-order comparison key (lexicographic)."""
    return (wall, counter, author_hex)


# ── OpRef (§4.1) — {1: target tstr, ?2: Hlc} ─────────────────────────────────────────────
def encode_opref(target: str, hlc_bytes: bytes = None) -> bytes:
    fields = [(1, enc_tstr(target))]
    if hlc_bytes is not None:
        fields.append((2, hlc_bytes))
    return enc_map(fields)


# ── AddTag (§4.1) — {1: author ik-pub, 2: Hlc} ───────────────────────────────────────────
def encode_addtag(author: bytes, hlc_bytes: bytes) -> bytes:
    return enc_map([(1, enc_bstr(author)), (2, hlc_bytes)])


# ── SyncOp envelope (§4.1) ────────────────────────────────────────────────────────────────
def encode_sync_op(kind, ns, target, field=None, value=None, hlc_bytes=None,
                    observed=None, ref=None):
    fields = [(1, enc_uint(kind)), (2, enc_tstr(ns)), (3, enc_tstr(target))]
    if field is not None:
        fields.append((4, enc_tstr(field)))
    if value is not None:
        fields.append((5, value))
    fields.append((6, hlc_bytes))
    if observed is not None:
        fields.append((7, enc_array(observed)))
    if ref is not None:
        fields.append((8, ref))
    return enc_map(fields)


vectors = []


def add(name, operation, input_, expected, note):
    vectors.append({"name": name, "operation": operation, "input": input_, "expected": expected, "note": note})


# ══════════════════════════════════════════════════════════════════════════════════════
# SYNC-OP-01 — Op canonical encoding (§4.1)
# ══════════════════════════════════════════════════════════════════════════════════════
hlc_op01 = encode_hlc(HLC_WALL, 0, PK_A)
op01 = encode_sync_op(kind=3, ns="", target="a", field="x", value=enc_tstr("v"), hlc_bytes=hlc_op01)
add(
    "sync_op_lww_canonical",
    "sync_op_encode",
    {
        "kind": 3, "ns": "", "target": "a", "field": "x", "value_tstr": "v",
        "hlc": {"wall": HLC_WALL, "counter": 0, "author_hex": PK_A.hex()},
    },
    {"cbor_hex": op01.hex()},
    "§4.1 SyncOp{kind:3 (lww-set), ns:\"\", target:\"a\", field:\"x\", value:\"v\", hlc}, "
    "keys 1,2,3,4,5,6 ascending (7 observed / 8 ref absent, both OPTIONAL for this kind, "
    "§4.2 table). value is the ext-value tstr \"v\" (§18.3.6). Deterministic CBOR (§18.1.1): "
    "shortest-form integers, ascending integer keys, definite lengths. Re-decoding MUST "
    "round-trip to the same fields and re-encoding MUST reproduce cbor_hex byte-for-byte; "
    "the generic canonical-CBOR reject rules (non-preferred ints, unsorted/duplicate keys, "
    "indefinite lengths, floats, undefined) already covered by DMTAP-CBOR-05..12 apply "
    "identically to SyncOp — no SYNC-specific reject case is needed for those.",
)

# ══════════════════════════════════════════════════════════════════════════════════════
# SYNC-AUTH-01 — Unauthorized author (§8, §9)
# ══════════════════════════════════════════════════════════════════════════════════════
hlc_auth01 = encode_hlc(HLC_WALL, 1, PK_X)
op_auth01 = encode_sync_op(kind=1, ns="", target="doc1", value=enc_tstr("e1"), hlc_bytes=hlc_auth01)
add(
    "sync_author_unauthorized",
    "sync_author_admission",
    {
        "op_cbor_hex": op_auth01.hex(),
        "op_hlc_author_hex": PK_X.hex(),
        "admitted_authors_hex": [PK_A.hex(), PK_B.hex()],
    },
    {
        "outcome": "reject",
        "error_code": "0x0A01",
        "error_name": "ERR_SYNC_AUTHOR_UNAUTHORIZED",
        "action": "FAIL_CLOSED_BLOCK",
    },
    "§8, §9: a set-add op whose hlc.author (PK_X) is not a member of the namespace's "
    "admitted-author set ({PK_A, PK_B} — a closed multi-owner member-set, §8 row 2, or "
    "the analogous single-owner DeviceCert set, §8 row 1) MUST be rejected regardless of "
    "whether its (hypothetical) signature verifies — admission is checked in addition to, "
    "not instead of, signature validity (§4.1, §8 'the authorization check is the same'). "
    "This vector tests the admission predicate only; it does not assert anything about "
    "COSE_Sign1 envelope bytes (see SYNC-OP-02, NOT-FROZEN).",
)

# ══════════════════════════════════════════════════════════════════════════════════════
# SYNC-LWW-01 / SYNC-LWW-02 — LWW register winner (§4.4)
# ══════════════════════════════════════════════════════════════════════════════════════
hlc_lww1a = encode_hlc(HLC_WALL, 0, PK_A)
hlc_lww1b = encode_hlc(HLC_WALL, 1, PK_A)  # same wall, higher counter => strictly greater HLC
op_lww1a = encode_sync_op(kind=3, ns="", target="doc1", field="title", value=enc_tstr("m"), hlc_bytes=hlc_lww1a)
op_lww1b = encode_sync_op(kind=3, ns="", target="doc1", field="title", value=enc_tstr("n"), hlc_bytes=hlc_lww1b)
add(
    "sync_lww_hlc_winner",
    "sync_lww_merge",
    {
        "ops_cbor_hex": [op_lww1a.hex(), op_lww1b.hex()],
        "hlcs": [
            {"wall": HLC_WALL, "counter": 0, "author_hex": PK_A.hex()},
            {"wall": HLC_WALL, "counter": 1, "author_hex": PK_A.hex()},
        ],
        "values": ["m", "n"],
    },
    {"winner_value": "n", "winner_hlc_hex": hlc_lww1b.hex(), "apply_order_independent": True},
    "§4.4: two lww-set on (doc1,title), HLC h1=(wall,0,A) < h2=(wall,1,A) (strictly "
    "greater counter, same wall/author). Winner = greater HLC = h2's value \"n\", on "
    "either apply order (merge is a join, order-independent).",
)

hlc_lww2 = encode_hlc(HLC_WALL, 5, PK_A)  # IDENTICAL hlc for both writes (the tie case)
val_m = enc_tstr("m")
val_n = enc_tstr("n")
op_lww2a = encode_sync_op(kind=3, ns="", target="doc1", field="title", value=val_m, hlc_bytes=hlc_lww2)
op_lww2b = encode_sync_op(kind=3, ns="", target="doc1", field="title", value=val_n, hlc_bytes=hlc_lww2)
assert val_n.hex() > val_m.hex()  # 0x616e > 0x616d — sanity-check the tiebreak direction
add(
    "sync_lww_exact_tie",
    "sync_lww_merge",
    {
        "ops_cbor_hex": [op_lww2a.hex(), op_lww2b.hex()],
        "hlc": {"wall": HLC_WALL, "counter": 5, "author_hex": PK_A.hex()},
        "values": ["m", "n"],
        "value_cbor_hex": [val_m.hex(), val_n.hex()],
    },
    {
        "winner_value": "n",
        "winner_value_cbor_hex": val_n.hex(),
        "rule": "identical HLC on both writes; winner = larger det_cbor(value) byte string",
    },
    "§4.4: two lww-set on (doc1,title) with the IDENTICAL hlc (same author+tick — a "
    "forged duplicate or same-tick re-derivation). Tiebreak descends to encoded-value "
    "bytes (§2.2 rule 2): det_cbor(\"n\") = 0x616e > det_cbor(\"m\") = 0x616d "
    "lexicographically, so \"n\" wins — identical on every replica regardless of local "
    "application order.",
)

# ══════════════════════════════════════════════════════════════════════════════════════
# SYNC-ORSET-01 / SYNC-ORSET-02 — OR-Set add-wins + causal-integrity reject (§4.3)
# ══════════════════════════════════════════════════════════════════════════════════════
hlc_add0 = encode_hlc(HLC_WALL, 0, PK_A)     # the add later tombstoned
hlc_remove = encode_hlc(HLC_WALL, 1, PK_B)   # remove citing only add0
hlc_add1 = encode_hlc(HLC_WALL, 2, PK_A)     # a concurrent, UNCITED add — survives

op_add0 = encode_sync_op(kind=1, ns="", target="tags", value=enc_tstr("e1"), hlc_bytes=hlc_add0)
op_remove = encode_sync_op(
    kind=2, ns="", target="tags", value=enc_tstr("e1"), hlc_bytes=hlc_remove,
    observed=[encode_addtag(PK_A, hlc_add0)],
)
op_add1 = encode_sync_op(kind=1, ns="", target="tags", value=enc_tstr("e1"), hlc_bytes=hlc_add1)

add(
    "sync_orset_add_wins",
    "sync_orset_merge",
    {
        "element": "e1",
        "ops_cbor_hex": [op_add0.hex(), op_remove.hex(), op_add1.hex()],
        "add_tags": [
            {"author_hex": PK_A.hex(), "hlc": {"wall": HLC_WALL, "counter": 0, "author_hex": PK_A.hex()}},
            {"author_hex": PK_A.hex(), "hlc": {"wall": HLC_WALL, "counter": 2, "author_hex": PK_A.hex()}},
        ],
        "tombstoned_add_tags": [
            {"author_hex": PK_A.hex(), "hlc": {"wall": HLC_WALL, "counter": 0, "author_hex": PK_A.hex()}},
        ],
    },
    {"present": True, "surviving_add_tag_hlc_hex": hlc_add1.hex()},
    "§4.3: element \"e1\" has two add-tags — add0 (wall,0,A), later tombstoned by a "
    "remove citing exactly add0 in `observed`, and a concurrent add1 (wall,2,A) the "
    "remove never observed. Presence = at least one add-tag not covered by a tombstone: "
    "add1 survives, so the element is present (add-wins) even though a remove for the "
    "same element exists.",
)

hlc_remove_early = encode_hlc(HLC_WALL, 1, PK_B)
hlc_add_future = encode_hlc(HLC_WALL, 10, PK_A)  # cited add-tag postdates the remove
op_remove_bad = encode_sync_op(
    kind=2, ns="", target="tags", value=enc_tstr("e2"), hlc_bytes=hlc_remove_early,
    observed=[encode_addtag(PK_A, hlc_add_future)],
)
add(
    "sync_orset_future_add_remove_rejected",
    "sync_orset_remove_validity",
    {
        "op_cbor_hex": op_remove_bad.hex(),
        "remove_hlc": {"wall": HLC_WALL, "counter": 1, "author_hex": PK_B.hex()},
        "cited_add_tag_hlc": {"wall": HLC_WALL, "counter": 10, "author_hex": PK_A.hex()},
    },
    {
        "outcome": "reject",
        "error_code": "0x0A03",
        "error_name": "ERR_SYNC_OP_INVALID",
        "action": "FAIL_CLOSED_BLOCK",
    },
    "§4.3 causal integrity: a set-remove citing an add-tag whose HLC (wall,10,A) is "
    "GREATER than the remove's own HLC (wall,1,B) — \"you cannot have observed an add "
    "from the future\" — MUST be rejected. This validity check is state-free (pure HLC "
    "comparison), so it never depends on local delivery order.",
)

# ══════════════════════════════════════════════════════════════════════════════════════
# SYNC-DEATH-01 / SYNC-DEATH-02 — remove-wins domination + tie fail-safe (§4.5)
# ══════════════════════════════════════════════════════════════════════════════════════
hlc_death1 = encode_hlc(HLC_WALL, 1, PK_A)
hlc_add_h2 = encode_hlc(HLC_WALL, 5, PK_B)  # h2 > h1, but a bare set-add never writes the death dimension
op_death1 = encode_sync_op(kind=4, ns="", target="rec1", field="redact", hlc_bytes=hlc_death1)
op_addh2 = encode_sync_op(kind=1, ns="", target="rec1", value=enc_tstr("rec1-payload"), hlc_bytes=hlc_add_h2)
add(
    "sync_death_domination",
    "sync_death_domination",
    {
        "death_op_cbor_hex": op_death1.hex(),
        "death_hlc": {"wall": HLC_WALL, "counter": 1, "author_hex": PK_A.hex()},
        "death_class": "redact",
        "concurrent_add_op_cbor_hex": op_addh2.hex(),
        "concurrent_add_hlc": {"wall": HLC_WALL, "counter": 5, "author_hex": PK_B.hex()},
    },
    {"present": False, "rule": "death dominates regardless of a numerically greater concurrent set-add HLC"},
    "§4.5 D3 invariant: `death(redact)` at h1=(wall,1,A); a concurrent bare `set-add` at "
    "h2=(wall,5,B), h2 > h1. An object is present iff NOT deleted AND the OR-Set says "
    "present; a bare set-add never writes the death dimension, so it can never outrank "
    "a death certificate even with a strictly greater HLC. Object is absent. Only an "
    "explicit `death(\"live\")` write with HLC > h1 would revive it.",
)

hlc_tie = encode_hlc(HLC_WALL, 7, PK_A)
op_death_tie = encode_sync_op(kind=4, ns="", target="rec2", field="redact", hlc_bytes=hlc_tie)
op_live_tie = encode_sync_op(kind=4, ns="", target="rec2", field="live", hlc_bytes=hlc_tie)
add(
    "sync_death_tie_failsafe",
    "sync_death_tie",
    {
        "death_op_cbor_hex": op_death_tie.hex(),
        "live_op_cbor_hex": op_live_tie.hex(),
        "hlc": {"wall": HLC_WALL, "counter": 7, "author_hex": PK_A.hex()},
    },
    {"winner": "Deleted", "class": "redact", "rule": "exact-HLC tie: Deleted > Live in the state order"},
    "§4.5: `death(redact)` and `death(\"live\")` written at the IDENTICAL HLC (wall,7,A) "
    "— only possible same-author-same-tick or a forged duplicate. Winner = greater HLC; "
    "at an exact tie, greater DeathState wins, and Deleted > Live by definition (fail-safe "
    "toward deletion) — \"Deleted\" wins.",
)

# ══════════════════════════════════════════════════════════════════════════════════════
# SYNC-PN-01 / SYNC-PN-02 — PN-counter merge + foreign-entry reject (§4.6)
# ══════════════════════════════════════════════════════════════════════════════════════
hlc_pn_a1 = encode_hlc(HLC_WALL, 0, PK_A)
hlc_pn_b1 = encode_hlc(HLC_WALL, 0, PK_B)
hlc_pn_a2 = encode_hlc(HLC_WALL, 1, PK_A)  # replay of the SAME +5(a) contribution
op_pn_a1 = encode_sync_op(kind=5, ns="", target="stock1", field="qty", value=enc_int(5), hlc_bytes=hlc_pn_a1)
op_pn_b1 = encode_sync_op(kind=5, ns="", target="stock1", field="qty", value=enc_int(-2), hlc_bytes=hlc_pn_b1)
op_pn_a2 = encode_sync_op(kind=5, ns="", target="stock1", field="qty", value=enc_int(5), hlc_bytes=hlc_pn_a2)
add(
    "sync_pn_counter_convergence",
    "sync_pn_merge",
    {
        "ops_cbor_hex": [op_pn_a1.hex(), op_pn_b1.hex(), op_pn_a2.hex()],
        "deltas": [
            {"author_hex": PK_A.hex(), "delta": 5},
            {"author_hex": PK_B.hex(), "delta": -2},
            {"author_hex": PK_A.hex(), "delta": 5, "note": "replay of author A's own contribution"},
        ],
    },
    {
        "P": {PK_A.hex(): 5, PK_B.hex(): 0},
        "N": {PK_A.hex(): 0, PK_B.hex(): 2},
        "total": 3,
        "replay_is_noop": True,
    },
    "§4.6: author A contributes +5 (P[A]=5), author B contributes -2 (N[B]=2). Merge is "
    "per-author MAX of P and of N, so a replayed +5(A) does not double-count "
    "(max(5,5)=5). Total = ΣP - ΣN = 5 - 2 = 3.",
)

add(
    "sync_pn_counter_foreign_reject",
    "sync_counter_foreign_check",
    {
        "op_hlc_author_hex": PK_A.hex(),
        "target_entry_author_hex": PK_B.hex(),
    },
    {
        "outcome": "reject",
        "error_code": "0x0A06",
        "error_name": "ERR_SYNC_COUNTER_FOREIGN",
        "action": "FAIL_CLOSED_BLOCK",
    },
    "§4.6: a `counter` op authored by A (hlc.author = PK_A) MUST NOT mutate P[B]/N[B] — "
    "an author may only advance its own P[author]/N[author] entry. A op/entry-author "
    "mismatch is rejected; this predicate tests the mismatch check itself (the wire "
    "representation of *how* an implementation would even construct a mismatched op is "
    "an implementation-internal concern the spec text does not spell out byte-for-byte, "
    "hence declarative fields rather than a single self-contained signed op here).",
)

# ══════════════════════════════════════════════════════════════════════════════════════
# SYNC-RGA-01 / SYNC-RGA-02 — RGA sibling order + insert-after-tombstone (§4.7)
# ══════════════════════════════════════════════════════════════════════════════════════
hlc_origin = encode_hlc(HLC_WALL, 0, PK_A)  # the shared left-origin atom ("atom0")
ref_to_origin = encode_opref("line1", hlc_origin)
hlc_ins1 = encode_hlc(HLC_WALL, 3, PK_A)   # h1
hlc_ins2 = encode_hlc(HLC_WALL, 4, PK_A)   # h2 > h1, SAME left-origin as h1 (concurrent siblings)
op_rga_origin = encode_sync_op(kind=6, ns="", target="line1", value=enc_tstr("atom0"), hlc_bytes=hlc_origin)
op_rga_ins1 = encode_sync_op(kind=6, ns="", target="line1", value=enc_tstr("X"), hlc_bytes=hlc_ins1, ref=ref_to_origin)
op_rga_ins2 = encode_sync_op(kind=6, ns="", target="line1", value=enc_tstr("Y"), hlc_bytes=hlc_ins2, ref=ref_to_origin)
add(
    "sync_rga_concurrent_sibling_order",
    "sync_rga_sibling_order",
    {
        "origin_op_cbor_hex": op_rga_origin.hex(),
        "sibling_ops_cbor_hex": [op_rga_ins1.hex(), op_rga_ins2.hex()],
        "sibling_hlcs": [
            {"wall": HLC_WALL, "counter": 3, "author_hex": PK_A.hex()},
            {"wall": HLC_WALL, "counter": 4, "author_hex": PK_A.hex()},
        ],
        "sibling_values": ["X", "Y"],
    },
    {
        "order_by_element_id_desc": [hlc_ins2.hex(), hlc_ins1.hex()],
        "order_values": ["Y", "X"],
        "rule": "atoms sharing a left-origin are ordered by descending element-id HLC (newer-first)",
    },
    "§4.7 RGA order rule: two seq-insert atoms (\"X\" at h1=(wall,3,A), \"Y\" at "
    "h2=(wall,4,A), h2>h1) sharing the SAME left-origin (atom0). Same-origin siblings "
    "order by descending element-id HLC — the newer insertion (\"Y\", h2) sorts BEFORE "
    "the older (\"X\", h1) among siblings of that origin. Identical on every replica "
    "because element ids are HLC-total-ordered.",
)

hlc_x = encode_hlc(HLC_WALL, 2, PK_A)         # atom "x"
hlc_remove_x = encode_hlc(HLC_WALL, 3, PK_B)  # seq-remove(x)
hlc_y = encode_hlc(HLC_WALL, 4, PK_A)         # concurrent seq-insert with ref=x
op_rga_x = encode_sync_op(kind=6, ns="", target="line1", value=enc_tstr("x"), hlc_bytes=hlc_x)
op_rga_remove_x = encode_sync_op(kind=7, ns="", target="line1", hlc_bytes=hlc_remove_x, ref=encode_opref("line1", hlc_x))
op_rga_y = encode_sync_op(kind=6, ns="", target="line1", value=enc_tstr("Z"), hlc_bytes=hlc_y, ref=encode_opref("line1", hlc_x))
add(
    "sync_rga_insert_after_tombstone",
    "sync_rga_tombstone_origin",
    {
        "insert_x_cbor_hex": op_rga_x.hex(),
        "remove_x_cbor_hex": op_rga_remove_x.hex(),
        "insert_y_cbor_hex": op_rga_y.hex(),
        "y_ref_origin_hlc": {"wall": HLC_WALL, "counter": 2, "author_hex": PK_A.hex()},
    },
    {
        "resolves": True,
        "reject": False,
        "atom_order_incl_tombstones": ["Z", "x(tombstoned)"],
        "visible_sequence": ["Z"],
    },
    "§4.7: `seq-remove(x)` tombstones atom \"x\" (tombstones are retained until GC). A "
    "concurrent `seq-insert` (\"Z\") whose left-origin `ref` names \"x\" still resolves "
    "against the retained tombstone — it is buffered/rejected only if the origin is "
    "genuinely absent (`ERR_SYNC_SEQ_ORIGIN_MISSING`, 0x0A07), never merely because the "
    "origin was removed. \"Z\" sorts immediately after \"x\"'s (tombstoned) position; the "
    "visible (non-tombstoned) sequence is just [\"Z\"].",
)

# ══════════════════════════════════════════════════════════════════════════════════════
# SYNC-NS-01 / SYNC-NS-02 — sparse scoping + cross-namespace ref reject (§7)
# ══════════════════════════════════════════════════════════════════════════════════════
hlc_nsx = encode_hlc(HLC_WALL, 0, PK_A)
hlc_nsy = encode_hlc(HLC_WALL, 0, PK_B)
op_ns_x = encode_sync_op(kind=1, ns="x", target="item1", value=enc_tstr("v"), hlc_bytes=hlc_nsx)
op_ns_y = encode_sync_op(kind=1, ns="y", target="item2", value=enc_tstr("v"), hlc_bytes=hlc_nsy)
add(
    "sync_ns_sparse_scoping",
    "sync_ns_sparse_filter",
    {
        "responder_ops_cbor_hex": [op_ns_x.hex(), op_ns_y.hex()],
        "responder_ops_ns": ["x", "y"],
        "caller_subscribed_ns": ["x"],
    },
    {"shipped_ops_cbor_hex": [op_ns_x.hex()], "shipped_ns": ["x"]},
    "§7: responder holds ops in namespaces {x,y}; caller subscribes only to {x}. "
    "`pull`/`fingerprint`/`ops` are scoped to the intersection — only the ns=\"x\" op is "
    "shipped; the ns=\"y\" op is never sent to a caller that never subscribed to \"y\".",
)

hlc_leak = encode_hlc(HLC_WALL, 1, PK_A)
ref_cross_ns = encode_opref("other-target")  # a target that in fact lives in ns "y", not this op's ns "x"
op_ns_leak = encode_sync_op(kind=6, ns="x", target="line1", value=enc_tstr("atom"), hlc_bytes=hlc_leak, ref=ref_cross_ns)
add(
    "sync_ns_cross_namespace_ref_rejected",
    "sync_ns_leak_check",
    {
        "op_cbor_hex": op_ns_leak.hex(),
        "op_ns": "x",
        "ref_target": "other-target",
        "ref_target_actual_ns": "y",
    },
    {
        "outcome": "reject",
        "error_code": "0x0A0A",
        "error_name": "ERR_SYNC_NS_LEAK",
        "action": "FAIL_CLOSED_BLOCK",
    },
    "§7 causal soundness: an RGA `ref` (or tree `parent`) MUST name a `target` in the "
    "SAME `ns` as the op itself. This op is in ns=\"x\" but its `ref` names a target that "
    "lives in ns=\"y\" — a cross-namespace reference, rejected so a sparse subscriber to "
    "\"x\" alone is never forced to fetch \"y\" to converge its own namespace.",
)

# ══════════════════════════════════════════════════════════════════════════════════════
# SYNC-GC-01 — stability-cut safety (§6.2)
# ══════════════════════════════════════════════════════════════════════════════════════
add(
    "sync_gc_stability_cut",
    "sync_gc_stability_cut",
    {
        "live_replica_watermarks": [
            {"replica": "R1", "max_applied_hlc": {"wall": HLC_WALL, "counter": 10, "author_hex": PK_A.hex()}},
            {"replica": "R2", "max_applied_hlc": {"wall": HLC_WALL, "counter": 15, "author_hex": PK_A.hex()}},
        ],
        "stale_replica_watermark": {
            "replica": "R3-stale", "max_applied_hlc": {"wall": HLC_WALL, "counter": 3, "author_hex": PK_B.hex()},
            "seen_within_liveness_window": False,
        },
    },
    {
        "stability_cut_counter": 10,
        "stale_replica_excluded": True,
        "note_no_watermark_case": "a live replica with NO known watermark yields NO cut at all (fail-closed, never GC on incomplete knowledge)",
    },
    "§6.2: stability cut = min, across every LIVE subscribed replica, of that replica's "
    "max-applied HLC. R1=counter 10, R2=counter 15 are live -> cut = 10 (the min of the "
    "two). R3-stale (counter 3, not seen within the liveness window) is EXCLUDED from the "
    "min, so it cannot stall compaction at counter 3. Separately (not a distinct byte "
    "input here, stated as a rule): a live replica with no known watermark at all yields "
    "no cut whatsoever — GC never proceeds on incomplete knowledge.",
)

# ══════════════════════════════════════════════════════════════════════════════════════
out = {
    "format": "dmtap-conformance-vectors/1",
    "suite": "Sync substrate capability (substrate/SYNC.md) — suite 0x01 (classical): Ed25519 / BLAKE3-256 "
    "primitives shared with the core; these vectors exercise only the deterministic CBOR + CRDT-algebra "
    "layer (no signing math — see NOT-FROZEN note on SYNC-OP-02 in substrate/SYNC.md §10)",
    "generated_by": "conformance/vectors/gen_sync_vectors.py (this repo) — NOT the dmtap-core reference "
    "crate, which does not implement the Sync substrate capability (it is substrate/SYNC.md's own "
    "'one genuinely new normative specification', ungrounded in any single existing numbered section). "
    "Every value here is a direct, mechanical application of substrate/SYNC.md's §3/§4/§6/§7 rules to "
    "fixed inputs — no randomness, no wall-clock reads.",
    "methodology": "Fixed 32-byte Ed25519 seeds (0xCC/0xDD/0xEE) via the `cryptography` package for author "
    "identity bytes only (no signatures are computed — this file freezes the CBOR + CRDT-merge layer, "
    "not the COSE_Sign1 signing envelope, which substrate/SYNC.md §10 marks NOT-FROZEN pending a frozen "
    "COSE profile). CBOR here is the same §18-canonical, integer-keyed deterministic encoding (RFC 8949 "
    "§4.2) used by conformance/vectors/vectors.json and conformance/vectors/pub_vectors.json.",
    "scope_note": "This file freezes 15 of substrate/SYNC.md §10's 20 stubs (SYNC-OP-01, SYNC-AUTH-01, "
    "SYNC-LWW-01/02, SYNC-ORSET-01/02, SYNC-DEATH-01/02, SYNC-PN-01/02, SYNC-RGA-01/02, SYNC-NS-01/02, "
    "SYNC-GC-01). The remaining 5 (SYNC-OP-02, SYNC-TREE-01, SYNC-SNAP-01/02, SYNC-RECON-01) are left as "
    "construction-recipe stubs in substrate/SYNC.md, each marked NOT-FROZEN with the specific "
    "underspecified design choice blocking a byte-exact vector — see that document, not this script.",
    "vectors": vectors,
}
print(json.dumps(out, indent=2))
