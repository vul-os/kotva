#!/usr/bin/env python3
"""
gen_sync_vectors.py — generates conformance/vectors/sync_vectors.json

Throwaway, deterministic vector generator for the Sync substrate capability
(substrate/SYNC.md §10 conformance-vector stubs). Mirrors the approach and
conventions of gen_pub_vectors.py exactly (same rationale: no reference
implementation yet exists for this wire shape — SYNC.md itself is "the one
genuinely new normative specification" in the substrate, and dmtap-core does
not implement it).

Scope discipline: this script freezes SYNC-* stubs only where substrate/SYNC.md's
text fully determines the byte-exact inputs/outputs, with no design choice left
to the generator. All 20 stubs are now byte-exact: the five that were previously
NOT-FROZEN (SYNC-OP-02 COSE_Sign1 envelope framing, SYNC-SNAP-01/02 canonical
observable-state schema, SYNC-RECON-01 fingerprint fold, and the SYNC-TREE-01
earlier/later-wins contradiction) were resolved *in the specification first* —
§4.1, §6.1.1, §5.3 and §4.8 respectively now carry the normative frozen text
plus its rationale — and only then vectored here. The spec, not this script,
remains the authoritative source of every decision below.

Corrections C-01..C-04 (substrate/SYNC.md §14) are applied here: the §4.6
PN-counter merge is now the per-author UNION of op-id-keyed deltas (the old
per-author max is non-associative and loses writes across partial states);
SYNC-PN-01's third op is a TRUE replay (identical HLC ⇒ identical op-id);
SYNC-RGA-02's atom order including tombstones is ["x(tombstoned)", "Z"] per
§4.7's insert-after rule; and SYNC-SNAP-02's `covers` is a §5.1 ik-pub-keyed
VersionVector, not an integer-keyed map.

Dependencies: `pip install blake3 cryptography` (BLAKE3-256 for content
addresses / state roots / reconciliation fingerprints; Ed25519 for author keys
and the SYNC-OP-02 COSE_Sign1 signature). Everything below is a FIXED constant:
fixed 32-byte Ed25519 seeds, fixed HLC wall-clock values. No randomness, no
wall-clock reads; Ed25519 (RFC 8032) is itself deterministic, so the signature
bytes are reproducible.

Run: python3 conformance/vectors/gen_sync_vectors.py > conformance/vectors/sync_vectors.json
"""
import json
import blake3
from cryptography.hazmat.primitives.asymmetric import ed25519
from cryptography.hazmat.primitives import serialization

# ── DMTAP-SYNC DS-tags (substrate/SYNC.md §4.1/§5.3/§6.1; §21.24c registry) ───────────────
# Each is the ASCII identifier terminated by a single 0x00 (the §18.1.6 DS-tag convention).
DS_OP = b"DMTAP-SYNC-v0/op\x00"                       # COSE_Sign1 external_aad (§4.1)
DS_OP_ID = b"DMTAP-SYNC-v0/op-id\x00"                 # op content-address hash preimage (§4.1)
DS_SNAPSHOT_STATE = b"DMTAP-SYNC-v0/snapshot-state\x00"  # observable-state root hash (§6.1.1)
DS_RECON_FP = b"DMTAP-SYNC-v0/recon-fp\x00"           # range-Merkle fingerprint fold (§5.3)


def b3(data: bytes) -> bytes:
    return blake3.blake3(data).digest()


def content_addr(ds_tag: bytes, body: bytes) -> bytes:
    """0x1e || BLAKE3-256(DS-tag || body) — a §18.1.5 v0 `hash` (33 bytes) over a DS-tagged preimage."""
    return b"\x1e" + b3(ds_tag + body)

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


SK_A, PK_A = keypair(SEED_SYNC_A)
SK_B, PK_B = keypair(SEED_SYNC_B)
SK_X, PK_X = keypair(SEED_SYNC_X)


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


def enc_bstr_map(pairs) -> bytes:
    """pairs: list of (bytes_key, encoded_value_bytes) for a bstr-KEYED map — the §5.1
    `VersionVector = { * ik-pub => Hlc }` shape. RFC 8949 §4.2.1 canonical ordering is by the
    ENCODED key bytes; all ik-pub keys are the same length, so that is ascending raw-key order."""
    enc = [(enc_bstr(k), v) for k, v in pairs]
    enc.sort(key=lambda kv: kv[0])
    out = _enc_head(5, len(enc))
    for k, v in enc:
        out += k + v
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
# SYNC-OP-02 — COSE_Sign1 envelope framing + signature bind (§4.1, frozen)
# ══════════════════════════════════════════════════════════════════════════════════════
# protected = bstr(det_cbor({1: alg, 4: kid})); alg = -8 (EdDSA) for suite 0x01, kid = hlc.author
COSE_ALG_EDDSA = -8
protected_map01 = enc_map([(1, enc_int(COSE_ALG_EDDSA)), (4, enc_bstr(PK_A))])
protected01 = enc_bstr(protected_map01)      # the bstr-wrapped protected header, as it appears on the wire
unprotected01 = enc_map([])                  # 0xa0 — nothing outside the signature
payload01 = enc_bstr(op01)                   # det_cbor(SyncOp) carried inline, never detached/nil

# Signable preimage: RFC 9052 §4.4 Sig_structure ["Signature1", protected, external_aad, payload],
# external_aad = the DS-tag "DMTAP-SYNC-v0/op" || 0x00 (bound into the signature, never transmitted).
sig_structure01 = enc_array([enc_tstr("Signature1"), protected01, enc_bstr(DS_OP), payload01])
signature01 = SK_A.sign(sig_structure01)
cose_sign1_01 = enc_array([protected01, unprotected01, payload01, enc_bstr(signature01)])
op_id_01 = content_addr(DS_OP_ID, op01)

# Tamper case: flip the low bit of the FINAL payload byte (the last byte of det_cbor(SyncOp)) and
# re-frame with the SAME signature — the signature must now fail.
op01_tampered = op01[:-1] + bytes([op01[-1] ^ 0x01])
cose_sign1_01_tampered = enc_array(
    [protected01, unprotected01, enc_bstr(op01_tampered), enc_bstr(signature01)]
)
# Substituted-kid case: same signature + payload, but the protected header names PK_B as signer.
protected01_badkid = enc_bstr(enc_map([(1, enc_int(COSE_ALG_EDDSA)), (4, enc_bstr(PK_B))]))
cose_sign1_01_badkid = enc_array(
    [protected01_badkid, unprotected01, payload01, enc_bstr(signature01)]
)
add(
    "sync_op_cose_sign1_bind",
    "sync_op_cose_sign1_verify",
    {
        "sync_op_cbor_hex": op01.hex(),
        "signer_seed_hex": SEED_SYNC_A.hex(),
        "signer_pubkey_hex": PK_A.hex(),
        "alg": COSE_ALG_EDDSA,
        "external_aad_hex": DS_OP.hex(),
        "cose_sign1_hex": cose_sign1_01.hex(),
        "tampered_payload_cose_sign1_hex": cose_sign1_01_tampered.hex(),
        "substituted_kid_cose_sign1_hex": cose_sign1_01_badkid.hex(),
    },
    {
        "protected_hex": protected01.hex(),
        "unprotected_hex": unprotected01.hex(),
        "payload_hex": payload01.hex(),
        "sig_structure_hex": sig_structure01.hex(),
        "signature_hex": signature01.hex(),
        "op_id_hex": op_id_01.hex(),
        "verifies": True,
        "tampered_payload": {
            "verifies": False,
            "error_code": "0x0A02",
            "error_name": "ERR_SYNC_OP_SIG_INVALID",
            "action": "FAIL_CLOSED_BLOCK",
        },
        "substituted_kid": {
            "verifies": False,
            "error_code": "0x0A02",
            "error_name": "ERR_SYNC_OP_SIG_INVALID",
            "action": "FAIL_CLOSED_BLOCK",
        },
    },
    "§4.1 (frozen): the wire object is the RFC 9052 `COSE_Sign1` four-element array "
    "[protected, unprotected, payload, signature], itself deterministic CBOR. protected = "
    "bstr(det_cbor({1: alg = -8 EdDSA (suite 0x01), 4: kid = hlc.author})) — kid is inside the "
    "INTEGRITY-COVERED header, so substituting a signer key is a verification failure, never a "
    "silent mis-attribution (see substituted_kid_cose_sign1_hex, which reuses a valid signature "
    "under a different kid and MUST fail). unprotected = the empty map 0xa0. payload = "
    "bstr(det_cbor(SyncOp)), always inline. signature = Ed25519(sk_author, det_cbor(Sig_structure)) "
    "over [\"Signature1\", protected, external_aad, payload] with external_aad = the DS-tag "
    "\"DMTAP-SYNC-v0/op\" || 0x00 — the RFC-9052-idiomatic realization of §18.1.6's "
    "preimage = DS-tag || body, bound into the signature but never transmitted, so a COSE_Sign1 "
    "minted for any other DMTAP object can never verify as a SyncOp and no peer-flippable "
    "discriminator flag exists. A flipped payload byte is 0x0A02. The op content address "
    "op_id = 0x1e || BLAKE3-256(\"DMTAP-SYNC-v0/op-id\" || 0x00 || det_cbor(SyncOp)) is computed "
    "over the SyncOp, NOT the envelope, so per-op-signed and SyncFrame-carried forms of one op "
    "share a single dedup/fingerprint identity. Ed25519 is deterministic (RFC 8032), so "
    "signature_hex is a reproducible known answer.",
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
    "This vector tests the admission predicate ALONE, deliberately independent of the envelope: "
    "the COSE_Sign1 framing is exercised separately by SYNC-OP-02, and admission must reject here "
    "even if that framing were byte-perfect.",
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
# A TRUE replay is the byte-identical op, hence the IDENTICAL HLC — an op-id is the content address
# of the whole SyncOp (§4.1), so bumping the counter to 1 would make this a DISTINCT op whose delta
# §4.6 correctly accumulates (P[A]=10, total=8), contradicting this vector's own expectation.
# (Correction C-02, SYNC.md §14.)
hlc_pn_a2 = hlc_pn_a1  # replay of the SAME +5(a) op: identical bytes ⇒ identical op-id
op_pn_a1 = encode_sync_op(kind=5, ns="", target="stock1", field="qty", value=enc_int(5), hlc_bytes=hlc_pn_a1)
op_pn_b1 = encode_sync_op(kind=5, ns="", target="stock1", field="qty", value=enc_int(-2), hlc_bytes=hlc_pn_b1)
op_pn_a2 = encode_sync_op(kind=5, ns="", target="stock1", field="qty", value=enc_int(5), hlc_bytes=hlc_pn_a2)
add(
    "sync_pn_counter_convergence",
    "sync_pn_merge",
    {
        "ops_cbor_hex": [op_pn_a1.hex(), op_pn_b1.hex(), op_pn_a2.hex()],
        "op_ids_hex": [content_addr(DS_OP_ID, o).hex() for o in (op_pn_a1, op_pn_b1, op_pn_a2)],
        "deltas": [
            {"author_hex": PK_A.hex(), "delta": 5},
            {"author_hex": PK_B.hex(), "delta": -2},
            {"author_hex": PK_A.hex(), "delta": 5,
             "note": "TRUE replay of author A's own op: byte-identical SyncOp, hence the identical "
                     "op-id as ops_cbor_hex[0] — not merely 'another +5 from A'"},
        ],
        # The associativity sub-case (§4.6, correction C-01): two replicas holding DIFFERENT
        # SUBSETS of one author's deltas. Declarative — the ops above are the byte-exact artifact;
        # this names the partial states an implementation MUST merge losslessly.
        "partial_merge_subcase": {
            "replica_1_op_indices": [0],
            "replica_2_op_indices": [1, 2],
            "note": "union of op-id-keyed deltas: replica_1 ⊔ replica_2 = the full state below, in "
                    "any grouping or order. A per-author MAX join would collapse differing subsets "
                    "of one author's deltas to the larger subtotal and lose the rest.",
        },
    },
    {
        "P": {PK_A.hex(): 5, PK_B.hex(): 0},
        "N": {PK_A.hex(): 0, PK_B.hex(): 2},
        "total": 3,
        "replay_is_noop": True,
        "distinct_op_ids": 2,
        "merge_is_associative": True,
    },
    "§4.6 (corrected — see SYNC.md §14 C-01/C-02): author A contributes +5 (P[A]=5), author B "
    "contributes -2 (N[B]=2). The merge is the per-author UNION of the author's op-id-keyed "
    "deltas — commutative, ASSOCIATIVE and idempotent — so the third op, which is byte-identical "
    "to the first and therefore carries the identical op-id, re-inserts a key already present and "
    "is a no-op: only 2 distinct op-ids exist here. Total = ΣP - ΣN = 5 - 2 = 3. NOTE the earlier "
    "form of this vector gave the third op hlc.counter=1, making it a DISTINCT op (different "
    "det_cbor ⇒ different op-id) whose delta §4.6 correctly accumulates to P[A]=10 / total=8 — it "
    "was never a replay, and the fix is the identical HLC used here. The merge is deliberately NOT "
    "per-author max of P/N: max is sound only when both replicas hold an author's COMPLETE op "
    "prefix, and silently loses deltas when they hold different subsets.",
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
        "atom_order_incl_tombstones": ["x(tombstoned)", "Z"],
        "atom_order_incl_tombstones_is": "a human-readable LABEL list, not normative bytes: it "
                                          "names the atoms in sequence order, tombstones included",
        "visible_sequence": ["Z"],
    },
    "§4.7: `seq-remove(x)` tombstones atom \"x\" (tombstones are retained until GC). A "
    "concurrent `seq-insert` (\"Z\") whose left-origin `ref` names \"x\" still resolves "
    "against the retained tombstone — it is buffered/rejected only if the origin is "
    "genuinely absent (`ERR_SYNC_SEQ_ORIGIN_MISSING`, 0x0A07), never merely because the "
    "origin was removed. \"Z\" sorts immediately AFTER \"x\"'s (tombstoned) position — the "
    "§4.7 insert-after rule — so the atom order INCLUDING tombstones is "
    "[\"x(tombstoned)\", \"Z\"] and the visible (non-tombstoned) sequence is just [\"Z\"]. "
    "(Corrected — SYNC.md §14 C-03: this array previously read [\"Z\", \"x(tombstoned)\"], "
    "the opposite of both §4.7 and this note. The note was right.)",
)

# ══════════════════════════════════════════════════════════════════════════════════════
# SYNC-TREE-01 — concurrent-move cycle: earlier-HLC move applied, later skipped (§4.8, frozen)
# ══════════════════════════════════════════════════════════════════════════════════════
TREE_ROOT = ""  # §6.1.1: the reserved tree-root node id is the empty string
# Baseline: A and B are both top-level nodes (children of the root sentinel).
hlc_t_a0 = encode_hlc(HLC_WALL, 0, PK_A)
hlc_t_b0 = encode_hlc(HLC_WALL, 0, PK_B)   # (W,0,A) < (W,0,B): equal wall+counter, author breaks the tie
op_tree_a0 = encode_sync_op(kind=8, ns="", target="A", field="a", hlc_bytes=hlc_t_a0, ref=encode_opref(TREE_ROOT))
op_tree_b0 = encode_sync_op(kind=8, ns="", target="B", field="b", hlc_bytes=hlc_t_b0, ref=encode_opref(TREE_ROOT))
# The colliding concurrent pair: move(A -> under B) at h1, move(B -> under A) at h2, h1 < h2.
hlc_t_h1 = encode_hlc(HLC_WALL, 1, PK_A)
hlc_t_h2 = encode_hlc(HLC_WALL, 2, PK_B)
op_tree_h1 = encode_sync_op(kind=8, ns="", target="A", field="1", hlc_bytes=hlc_t_h1, ref=encode_opref("B"))
op_tree_h2 = encode_sync_op(kind=8, ns="", target="B", field="1", hlc_bytes=hlc_t_h2, ref=encode_opref("A"))
add(
    "sync_tree_concurrent_move_cycle",
    "sync_tree_move_replay",
    {
        "baseline_ops_cbor_hex": [op_tree_a0.hex(), op_tree_b0.hex()],
        "baseline_edges": [
            {"node": "A", "parent": TREE_ROOT, "ord": "a"},
            {"node": "B", "parent": TREE_ROOT, "ord": "b"},
        ],
        "colliding_ops_cbor_hex": [op_tree_h1.hex(), op_tree_h2.hex()],
        "colliding_moves": [
            {"label": "h1", "move": "A -> under B", "hlc": {"wall": HLC_WALL, "counter": 1, "author_hex": PK_A.hex()}},
            {"label": "h2", "move": "B -> under A", "hlc": {"wall": HLC_WALL, "counter": 2, "author_hex": PK_B.hex()}},
        ],
    },
    {
        "applied": ["h1"],
        "skipped": ["h2"],
        "skipped_is_error": False,
        "final_edges": [
            {"node": "A", "parent": "B", "ord": "1"},
            {"node": "B", "parent": TREE_ROOT, "ord": "b"},
        ],
        "acyclic": True,
        "apply_order_independent": True,
    },
    "§4.8 (frozen): moves are replayed in ASCENDING HLC order (oldest first), and a move "
    "(node -> new_parent) would create a cycle iff new_parent == node or new_parent is a "
    "descendant of node in the tree formed by all strictly-earlier-HLC moves already applied. "
    "Replay order here is (W,0,A) (W,0,B) h1 h2. When h1 is evaluated, B is not a descendant of "
    "A, so A becomes a child of B. When h2 is then evaluated, A IS already a descendant of B, so "
    "moving B under A would close the cycle B->A->B and h2 is SKIPPED (a recorded no-op, never an "
    "error); B keeps its pre-swap parent, the root. The observable result is therefore the "
    "EARLIER move applied and the LATER move skipped — the correction to the original stub text, "
    "which asserted the reverse. This is Kleppmann's cycle-safe replicated-tree result and is "
    "deliberately NOT last-writer-wins for the colliding pair: LWW (§4.4) governs only repeated "
    "moves of the SAME node; the ordered replay, not the clock, decides the interaction between "
    "moves of DIFFERENT nodes, so every replica reaches this identical acyclic tree regardless of "
    "arrival order (a replica receiving h2 before h1 re-evaluates the affected subtree in HLC "
    "order and reaches the same result).",
)

# ══════════════════════════════════════════════════════════════════════════════════════
# SYNC-SNAP-01 / SYNC-SNAP-02 — canonical observable state + snapshot root (§6.1.1, frozen)
# ══════════════════════════════════════════════════════════════════════════════════════
def section(entries) -> bytes:
    """A §6.1.1 section: entries sorted ASCENDING by their deterministic-CBOR bytes."""
    return enc_array(sorted(entries))


def observable_state(orset, lww, pn, death, rga, tree) -> bytes:
    """The fixed SIX-element positional array of §6.1.1 — kind-ascending, never omitted."""
    return enc_array([section(orset), section(lww), section(pn), section(death),
                      section(rga), section(tree)])


def state_root(state_cbor: bytes) -> bytes:
    """root = 0x1e || BLAKE3-256("DMTAP-SYNC-v0/snapshot-state" || 0x00 || det_cbor(ObservableState))."""
    return content_addr(DS_SNAPSHOT_STATE, state_cbor)


# The state below is exactly what the earlier vectors in this file converge to, so the snapshot
# vectors are not a fresh invention: OR-Set "e1" present on "tags" (SYNC-ORSET-01), the LWW winner
# "n" on (doc1,title) (SYNC-LWW-01), the PN total 3 on (stock1,qty) (SYNC-PN-01), rec1 deleted with
# class "redact" (SYNC-DEATH-01), the RGA sequence [atom0, Y, X] on line1 (SYNC-RGA-01), and the
# acyclic tree A-under-B / B-under-root (SYNC-TREE-01).
sect_orset = [enc_array([enc_tstr("tags"), enc_tstr("e1")])]
sect_lww = [enc_array([enc_tstr("doc1"), enc_tstr("title"), enc_tstr("n")])]
sect_pn = [enc_array([enc_tstr("stock1"), enc_tstr("qty"), enc_int(3)])]
sect_death = [enc_array([enc_tstr("rec1"), enc_tstr("redact")])]
sect_rga = [enc_array([enc_tstr("line1"),
                       enc_array([enc_tstr("atom0"), enc_tstr("Y"), enc_tstr("X")])])]
sect_tree = [
    enc_array([enc_tstr("A"), enc_tstr("B"), enc_tstr("1")]),
    enc_array([enc_tstr("B"), enc_tstr(TREE_ROOT), enc_tstr("b")]),
]
state_v1 = observable_state(sect_orset, sect_lww, sect_pn, sect_death, sect_rga, sect_tree)
root_v1 = state_root(state_v1)

empty_state = observable_state([], [], [], [], [], [])
add(
    "sync_snapshot_root_determinism",
    "sync_snapshot_state_root",
    {
        "observable_state": {
            "orset": [["tags", "e1"]],
            "lww": [["doc1", "title", "n"]],
            "pn": [["stock1", "qty", 3]],
            "death": [["rec1", "redact"]],
            "rga": [["line1", ["atom0", "Y", "X"]]],
            "tree": [["A", "B", "1"], ["B", TREE_ROOT, "b"]],
        },
        "ds_tag_hex": DS_SNAPSHOT_STATE.hex(),
        "empty_state_sections": 6,
    },
    {
        "observable_state_cbor_hex": state_v1.hex(),
        "root_hex": root_v1.hex(),
        "empty_state_cbor_hex": empty_state.hex(),
        "empty_state_root_hex": state_root(empty_state).hex(),
        "same_covers_same_root": True,
        "mismatch_error_code": "0x0A09",
        "mismatch_error_name": "ERR_SYNC_SNAPSHOT_ROOT_MISMATCH",
        "mismatch_action": "HALT_ALERT",
    },
    "§6.1.1 (frozen): ObservableState is a fixed SIX-element positional array — "
    "[orset, lww, pn, death, rga, tree] in kind-ascending order, positional rather than keyed so "
    "no map-key scheme can be a source of divergence, and empty sections are the empty array [] "
    "present in position (see empty_state_cbor_hex = 0x86 followed by six 0x80). Every section is "
    "sorted ASCENDING by the deterministic-CBOR bytes of each entry; the sole exception is the "
    "RGA inner atom list, which is in SEQUENCE order (the §4.7 pre-order walk) and is NOT "
    "re-sorted, because for a sequence the order IS the observable value. Only OBSERVABLE state "
    "appears — add-tags, tombstones, per-author P/N maps, RGA element ids, Live death cells and "
    "superseded LWW cells are all internal — so two replicas at the same `covers` vector produce "
    "byte-identical bytes regardless of apply order or internal bookkeeping. "
    "root = 0x1e || BLAKE3-256(\"DMTAP-SYNC-v0/snapshot-state\" || 0x00 || det_cbor(ObservableState)), "
    "a §18.1.5 v0 hash; the DS-tag is distinct from the snapshot SIGNATURE tag "
    "(\"DMTAP-SYNC-v0/snapshot\") so a state-root preimage and a signature preimage can never be "
    "confused. A root mismatch at equal `covers` is 0x0A09 (HALT_ALERT) — evidence of divergence. "
    "The state serialized here is exactly the convergence result of this file's earlier vectors "
    "(ORSET-01, LWW-01, PN-01, DEATH-01, RGA-01, TREE-01).",
)

# SYNC-SNAP-02 — fast join (snapshot + post-`covers` ops) == full replay, byte-for-byte.
hlc_post = encode_hlc(HLC_WALL, 20, PK_B)  # strictly after everything folded into `covers`
op_post = encode_sync_op(kind=3, ns="", target="doc1", field="title", value=enc_tstr("p"), hlc_bytes=hlc_post)
sect_lww_v2 = [enc_array([enc_tstr("doc1"), enc_tstr("title"), enc_tstr("p")])]
state_v2 = observable_state(sect_orset, sect_lww_v2, sect_pn, sect_death, sect_rga, sect_tree)
root_v2 = state_root(state_v2)
# §5.1: a VersionVector is `{ * ik-pub => Hlc }` — the keys are the authors' 32-byte ik-pub BYTE
# STRINGS, canonically ordered by encoded key bytes. (Correction C-04, SYNC.md §14: this was
# previously mis-encoded as an integer-keyed map {1: Hlc, 2: Hlc}, which no expectation exercised.)
covers_v1 = enc_bstr_map([(PK_A, encode_hlc(HLC_WALL, 4, PK_A)), (PK_B, encode_hlc(HLC_WALL, 7, PK_B))])
add(
    "sync_snapshot_fast_join_equals_replay",
    "sync_snapshot_fast_join",
    {
        "snapshot_covers_note": "per-author max HLC folded into the snapshot: A@(W,4), B@(W,7). "
                                "§5.1 VersionVector = { * ik-pub => Hlc }: ik-pub BYTE-STRING keys, "
                                "canonically ordered by encoded key bytes (B's key sorts before A's).",
        "snapshot_covers_cbor_hex": covers_v1.hex(),
        "snapshot_observable_state_cbor_hex": state_v1.hex(),
        "snapshot_root_hex": root_v1.hex(),
        "post_covers_ops_cbor_hex": [op_post.hex()],
        "post_covers_ops": [{"kind": 3, "target": "doc1", "field": "title", "value": "p",
                             "hlc": {"wall": HLC_WALL, "counter": 20, "author_hex": PK_B.hex()}}],
    },
    {
        "fast_join_state_cbor_hex": state_v2.hex(),
        "full_replay_state_cbor_hex": state_v2.hex(),
        "states_byte_identical": True,
        "root_hex": root_v2.hex(),
        "roots_equal": True,
    },
    "§6.1/§6.1.1 (frozen): a joining replica adopts the snapshot's observable state, sets its "
    "local vector to `covers`, and applies only the ops AFTER `covers` — here one lww-set writing "
    "(doc1,title) = \"p\" at (W,20,B), which supersedes the snapshot's winning value \"n\" because "
    "its HLC is greater. The resulting ObservableState bytes are IDENTICAL to those of a replica "
    "that replayed the entire history from genesis, hence the roots are identical: only the "
    "observable projection is serialized, so the two replicas' differing internal bookkeeping "
    "(one has the pre-snapshot op log, the other never saw it) cannot show through. This is the "
    "strong-eventual-consistency equality the fast-join guarantee rests on, and it is what makes "
    "a snapshot VERIFIABLE rather than merely trusted — a replica that later backfills the "
    "pre-`covers` ops MUST recompute this same root.",
)

# ══════════════════════════════════════════════════════════════════════════════════════
# SYNC-RECON-01 — range-Merkle fingerprint fold + drill-down (§5.3, frozen)
# ══════════════════════════════════════════════════════════════════════════════════════
def recon_fp(op_ids) -> bytes:
    """fp = 0x1e || BLAKE3-256("DMTAP-SYNC-v0/recon-fp" || 0x00 || det_cbor([* op-id])),
    over the range's op ids sorted ASCENDING by the HLC of their ops (§3 total order)."""
    return content_addr(DS_RECON_FP, enc_array([enc_bstr(i) for i in op_ids]))


hlc_r1 = encode_hlc(HLC_WALL, 1, PK_A)
hlc_r2 = encode_hlc(HLC_WALL, 2, PK_A)
hlc_r3 = encode_hlc(HLC_WALL, 3, PK_A)  # the op replica R2 is missing
op_r1 = encode_sync_op(kind=1, ns="", target="tags", value=enc_tstr("r1"), hlc_bytes=hlc_r1)
op_r2 = encode_sync_op(kind=1, ns="", target="tags", value=enc_tstr("r2"), hlc_bytes=hlc_r2)
op_r3 = encode_sync_op(kind=1, ns="", target="tags", value=enc_tstr("r3"), hlc_bytes=hlc_r3)
id_r1, id_r2, id_r3 = (content_addr(DS_OP_ID, o) for o in (op_r1, op_r2, op_r3))

# Whole range [lo, hi) = [(W,0,A), (W,10,A)) — A holds {r1,r2,r3}, B holds {r1,r2}.
fp_A_full, fp_B_full = recon_fp([id_r1, id_r2, id_r3]), recon_fp([id_r1, id_r2])
# Split into a fan-out of 2 at (W,2,A): sub-range 1 = [(W,0,A),(W,2,A)) = {r1} on BOTH sides.
fp_A_sub1, fp_B_sub1 = recon_fp([id_r1]), recon_fp([id_r1])
# Sub-range 2 = [(W,2,A),(W,10,A)): {r2,r3} vs {r2} — the mismatch, which ships exactly r3.
fp_A_sub2, fp_B_sub2 = recon_fp([id_r2, id_r3]), recon_fp([id_r2])
fp_empty = recon_fp([])
add(
    "sync_recon_range_merkle_diff",
    "sync_recon_fingerprint",
    {
        "ops_cbor_hex": {"r1": op_r1.hex(), "r2": op_r2.hex(), "r3": op_r3.hex()},
        "op_ids_hex": {"r1": id_r1.hex(), "r2": id_r2.hex(), "r3": id_r3.hex()},
        "replica_A_holds": ["r1", "r2", "r3"],
        "replica_B_holds": ["r1", "r2"],
        "range": {"lo": {"wall": HLC_WALL, "counter": 0, "author_hex": PK_A.hex()},
                  "hi": {"wall": HLC_WALL, "counter": 10, "author_hex": PK_A.hex()}},
        "split_at": {"wall": HLC_WALL, "counter": 2, "author_hex": PK_A.hex()},
        "ds_tag_hex": DS_RECON_FP.hex(),
    },
    {
        "full_range": {
            "A": {"fp_hex": fp_A_full.hex(), "count": 3},
            "B": {"fp_hex": fp_B_full.hex(), "count": 2},
            "match": False,
        },
        "subrange_1": {
            "A": {"fp_hex": fp_A_sub1.hex(), "count": 1},
            "B": {"fp_hex": fp_B_sub1.hex(), "count": 1},
            "match": True,
            "ops_exchanged": [],
        },
        "subrange_2": {
            "A": {"fp_hex": fp_A_sub2.hex(), "count": 2},
            "B": {"fp_hex": fp_B_sub2.hex(), "count": 1},
            "match": False,
            "ops_shipped_to_B": [id_r3.hex()],
        },
        "ops_shipped_total": 1,
        "empty_range_fp_hex": fp_empty.hex(),
        "empty_range_count": 0,
    },
    "§5.3 (frozen): fp = 0x1e || BLAKE3-256(\"DMTAP-SYNC-v0/recon-fp\" || 0x00 || "
    "det_cbor([* op-id])) over the range's op ids sorted ascending by their ops' HLC — one "
    "DS-tagged BLAKE3 hash FOLDING the ordered ids into a single digest (matching the §5.6 `recon` "
    "reference fp = ContentId::of(det_cbor([* id]))), shipped with count = |R|. It is deliberately "
    "NOT a homomorphic/incremental combiner (XOR- or addition-of-hashes): a homomorphic fold buys "
    "O(1) range updates but admits cancellation (an even number of identical insertions vanishes) "
    "and adds integer arithmetic to the wire, whereas a changed range is simply re-hashed and "
    "BLAKE3 over the length-prefixed deterministic-CBOR array is collision-resistant and "
    "unambiguous across a range boundary. `count` guards the degenerate empty-vs-empty and "
    "duplicate cases a digest alone cannot distinguish (note empty_range_fp_hex is a well-defined "
    "hash of det_cbor([]) = 0x80, not a special case). Round: the full range mismatches, so it is "
    "split (fan-out of 2 at (W,2,A) — the SPLIT POINT is an input here, since §5.3 fixes only "
    "\"split by op count into a small fixed fan-out\", not a particular boundary); sub-range 1 has "
    "equal (fp,count) on both sides and exchanges NO ops; sub-range 2 mismatches and surfaces "
    "exactly the one differing op r3. Range-Merkle is a discovery optimization only — r3 is still "
    "applied through the same §4 verify+merge path.",
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
    "primitives shared with the core. These vectors exercise the deterministic CBOR + CRDT-algebra layer, "
    "the RFC 9052 COSE_Sign1 op envelope (§4.1), the canonical observable-state root (§6.1.1) and the "
    "range-Merkle fingerprint fold (§5.3)",
    "generated_by": "conformance/vectors/gen_sync_vectors.py (this repo) — NOT the dmtap-core reference "
    "crate, which does not implement the Sync substrate capability (it is substrate/SYNC.md's own "
    "'one genuinely new normative specification', ungrounded in any single existing numbered section). "
    "Every value here is a direct, mechanical application of substrate/SYNC.md's §3/§4/§6/§7 rules to "
    "fixed inputs — no randomness, no wall-clock reads.",
    "methodology": "Fixed 32-byte Ed25519 seeds (0xCC/0xDD/0xEE) via the `cryptography` package; BLAKE3-256 "
    "via the `blake3` package. Ed25519 (RFC 8032) is deterministic, so the one signature vector "
    "(SYNC-OP-02) is a reproducible known answer with no RNG involved. CBOR here is the same "
    "§18-canonical, integer-keyed deterministic encoding (RFC 8949 §4.2) used by "
    "conformance/vectors/vectors.json and conformance/vectors/pub_vectors.json; all content addresses use "
    "the §18.1.5 v0 form 0x1e || BLAKE3-256(DS-tag || 0x00 || body) with the §21.24c DMTAP-SYNC-v0 DS-tags.",
    "scope_note": "This file freezes ALL 20 of substrate/SYNC.md §10's conformance stubs. The five that "
    "were previously NOT-FROZEN — SYNC-OP-02 (COSE_Sign1 envelope framing), SYNC-TREE-01 (which side of a "
    "concurrent-move cycle loses), SYNC-SNAP-01/02 (canonical observable-state schema) and SYNC-RECON-01 "
    "(range-Merkle fingerprint fold) — were each resolved by adding normative frozen text to the "
    "specification FIRST (§4.1, §4.8, §6.1.1, §5.3 respectively, each with its rationale) and only then "
    "vectored here. No decision in this file originates in this file: substrate/SYNC.md is authoritative.",
    "corrections_note": "Regenerated after substrate/SYNC.md §14's corrections C-01..C-04, all of which "
    "were surfaced by an independent Rust implementation of SYNC.md (envoir `dmtap-sync`), not by review: "
    "C-01 changed §4.6's PN-counter merge from per-author max of P/N to the per-author UNION of "
    "op-id-keyed deltas (the max join is non-associative and LOSES writes when replicas hold different "
    "subsets of one author's deltas — a NORMATIVE merge-semantics correction); C-02 fixed SYNC-PN-01, "
    "whose 'replay' op carried a different HLC and was therefore a distinct op; C-03 fixed SYNC-RGA-02's "
    "atom_order_incl_tombstones, which contradicted §4.7 and the vector's own note; C-04 fixed "
    "SYNC-SNAP-02's snapshot_covers_cbor_hex, which encoded an integer-keyed map where §5.1 specifies "
    "ik-pub bstr keys. Vector count is unchanged at 20.",
    "vectors": vectors,
}
print(json.dumps(out, indent=2))
