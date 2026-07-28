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
to the generator. All stubs are now byte-exact: the five that were previously
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
DS_SNAPSHOT = b"DMTAP-SYNC-v0/snapshot\x00"           # Snapshot SIGNATURE preimage (§6.1 key 8)


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


def cose_sign1(op_bytes: bytes, sk, pk: bytes) -> bytes:
    """The §4.1 wire object for one op: the RFC 9052 COSE_Sign1 four-element ARRAY
    [protected, unprotected, payload, signature], deterministic-CBOR encoded.

    Per §5.2's op-framing rule this value is embedded in an `ops` array as a CBOR
    ITEM — it is NOT bstr-wrapped (correction C-06)."""
    protected = enc_bstr(enc_map([(1, enc_int(COSE_ALG_EDDSA)), (4, enc_bstr(pk))]))
    payload = enc_bstr(op_bytes)
    sig_structure = enc_array([enc_tstr("Signature1"), protected, enc_bstr(DS_OP), payload])
    return enc_array([protected, enc_map([]), payload, enc_bstr(sk.sign(sig_structure))])

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
        "snapshot_observable_state_role": "the COMMITMENT hashed to produce `root` (§6.1.1) — NOT the "
                                          "transferred body and NOT importable; what a joining replica "
                                          "actually adopts is the §6.1.2 SnapshotBody, a compacted op "
                                          "set that FOLDS to these bytes (see SYNC-SNAP-03).",
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
    "§6.1/§6.1.1 (frozen): a joining replica ingests the snapshot's BODY (§6.1.2 — a compacted op "
    "set that folds to the observable state below; corrected by C-09, which is vectored separately "
    "as SYNC-SNAP-03), sets its "
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
# SYNC-SNAP-03 — the snapshot BODY is a compacted op set, not ObservableState (§6.1.2, C-09)
# ══════════════════════════════════════════════════════════════════════════════════════
# A deliberately minimal namespace so the failure is arithmetic rather than atmospheric:
# ONE live LWW cell. The body is the ONE op that produced it; the state is what that op
# folds to. The post-`covers` op is chosen to have a LOWER HLC than the body's op, which is
# possible precisely because `covers` bounds each author's OWN stream while the §3 HLC is a
# total order ACROSS authors — B is absent from `covers`, so every op of B's is "after"
# `covers`, including ones the total order places before A's winning write.
hlc_body_a = encode_hlc(HLC_WALL, 4, PK_A)
op_body_a = encode_sync_op(kind=3, ns="", target="doc1", field="title",
                           value=enc_tstr("n"), hlc_bytes=hlc_body_a)
cose_body_a = cose_sign1(op_body_a, SK_A, PK_A)

snap3_state = observable_state([], [enc_array([enc_tstr("doc1"), enc_tstr("title"), enc_tstr("n")])],
                               [], [], [], [])
snap3_root = state_root(snap3_state)
snap3_covers = enc_bstr_map([(PK_A, hlc_body_a)])
snap3_body = enc_array([cose_body_a])   # §6.1.2 SnapshotBody: ops item-embedded per §5.2 (C-06)

# The post-`covers` op: author B, counter 3 < the body op's counter 4, same wall.
hlc_post_b = encode_hlc(HLC_WALL, 3, PK_B)
op_post_b = encode_sync_op(kind=3, ns="", target="doc1", field="title",
                           value=enc_tstr("q"), hlc_bytes=hlc_post_b)
cose_post_b = cose_sign1(op_post_b, SK_B, PK_B)

# Correct outcome: fold the body, then apply the post-op. (W,4,A) > (W,3,B), so "n" HOLDS.
snap3_state_after = snap3_state          # unchanged — the incumbent wins
snap3_root_after = snap3_root
# The WRONG outcome an ObservableState-adopter reaches: it has the value "n" but not its HLC,
# so it has nothing to compare (W,3,B) against and applies the write.
snap3_state_projection = observable_state(
    [], [enc_array([enc_tstr("doc1"), enc_tstr("title"), enc_tstr("q")])], [], [], [], [])
snap3_root_projection = state_root(snap3_state_projection)

add(
    "sync_snapshot_body_is_op_set",
    "sync_snapshot_body_fold",
    {
        "snapshot_covers_cbor_hex": snap3_covers.hex(),
        "snapshot_root_hex": snap3_root.hex(),
        "snapshot_body_cbor_hex": snap3_body.hex(),
        "snapshot_body_ops": [{"kind": 3, "target": "doc1", "field": "title", "value": "n",
                               "hlc": {"wall": HLC_WALL, "counter": 4, "author_hex": PK_A.hex()}}],
        "post_covers_op_cbor_hex": cose_post_b.hex(),
        "post_covers_op": {"kind": 3, "target": "doc1", "field": "title", "value": "q",
                           "hlc": {"wall": HLC_WALL, "counter": 3, "author_hex": PK_B.hex()}},
        "observable_state_cbor_hex": snap3_state.hex(),
    },
    {
        "body_folds_to_root": True,
        "folded_state_cbor_hex": snap3_state.hex(),
        "folded_root_hex": snap3_root.hex(),
        "state_after_post_op_cbor_hex": snap3_state_after.hex(),
        "root_after_post_op_hex": snap3_root_after.hex(),
        "winning_value_after_post_op": "n",
        "projection_adopt_state_cbor_hex": snap3_state_projection.hex(),
        "projection_adopt_root_hex": snap3_root_projection.hex(),
        "projection_adopt_is_nonconformant": True,
        "roots_differ": True,
        "body_mismatch_error_code": "0x0A09",
        "body_mismatch_error_name": "ERR_SYNC_SNAPSHOT_ROOT_MISMATCH",
    },
    "§6.1.2 (C-09): what a fast-joining replica transfers and adopts is a SnapshotBody — the "
    "minimal set of canonical, individually-signed ops whose fold equals the observable state — "
    "NOT det_cbor(ObservableState). The body here is one COSE_Sign1 op, item-embedded per §5.2's "
    "framing rule, and it is verified by FOLD-THEN-RECOMPUTE: ingest each op through the ordinary "
    "§4 path, derive ObservableState per §6.1.1, hash with the snapshot-state DS-tag, and require "
    "equality with Snapshot.root (0x0A09 otherwise, body discarded whole). That is strictly "
    "stronger than hashing the transfer bytes, because it proves the ops PRODUCE the committed "
    "state. The second half of this vector is why the distinction is normative rather than "
    "stylistic: §6.1.1 drops the winning cell's HLC, so a replica that adopted the PROJECTION has "
    "the value \"n\" but nothing to compare a later op against. The post-`covers` op here is "
    "(W,3,B) — genuinely after `covers` (B is absent from it, so every op of B's is), yet BELOW "
    "(W,4,A) in the §3 total order, which orders across authors while `covers` bounds each author's "
    "own stream. A conformant replica folds the body, sees (W,3,B) < (W,4,A), and KEEPS \"n\"; a "
    "projection-adopter has no incumbent HLC, applies the write, and lands on \"q\" with a "
    "different root — silent, permanent divergence with no error raised on either side. Compare "
    "SYNC-SNAP-02, whose post-`covers` op is (W,20,B) and genuinely does win: that vector's "
    "expectation is unchanged and remains correct; it simply does not exercise the ordering case "
    "that distinguishes the two adoption models.",
)

# ══════════════════════════════════════════════════════════════════════════════════════
# SYNC-VAL-01 — the `ext-value` boundary for a SyncOp value (§4.1/§4.1.1, C-08)
# ══════════════════════════════════════════════════════════════════════════════════════
# ext-value = bool / int / bytes / tstr / [* ext-value] / { * tstr => ext-value }   (§18.3.6)
def enc_tstr_map(pairs) -> bytes:
    """A text-KEYED map — the §18.3.6 ext-value map arm. RFC 8949 §4.2.1 canonical ordering is
    by the ENCODED key bytes, which for tstr keys is shortest-first then lexicographic."""
    enc = sorted(((enc_tstr(k), v) for k, v in pairs), key=lambda kv: kv[0])
    out = _enc_head(5, len(enc))
    for k, v in enc:
        out += k + v
    return out


val_nested = enc_tstr_map([
    ("id", enc_tstr("shape-a")),
    ("x", enc_uint(10)),
    ("locked", enc_bool(False)),
    ("pts", enc_array([enc_uint(1), enc_int(-2)])),
    ("meta", enc_tstr_map([("z", enc_bstr(b"\x01\x02"))])),   # recursion, depth 2
])
val_hetero = enc_array([enc_tstr("fade"), enc_uint(3), enc_bool(True)])
val_opaque = enc_tstr('{"id":"shape-a","x":10}')

VAL_ACCEPT = [
    ("tstr", enc_tstr("v"), "text — the baseline value of SYNC-OP-01"),
    ("bstr", enc_bstr(b"\xde\xad"), "byte string"),
    ("uint", enc_uint(10), "unsigned integer"),
    ("nint", enc_int(-2), "negative integer"),
    ("bool", enc_bool(True), "boolean"),
    ("tstr_map_nested", val_nested,
     "TEXT-KEYED MAP, recursively nested (depth 2) — the §18.3.6 arm §4.1 previously omitted"),
    ("array_heterogeneous", val_hetero,
     "array of MIXED ext-values — §18.3.6 is [* ext-value]; homogeneity was never required"),
    ("tstr_opaque_json", val_opaque,
     "an opaque JSON payload carried as tstr — still conformant, and still the right choice "
     "for foreign or byte-exact-round-trip content (§4.1.1)"),
    ("map_empty", enc_map([]),
     "the EMPTY map, 0xa0 (§14 C-14). `{ * tstr => ext-value }` admits zero entries, so this IS "
     "an ext-value — and 0xa0 is the encoding of an empty map of ANY key type, so the key-type "
     "ambiguity a decoder faces here is REAL but VACUOUS: there are no keys to disagree about and "
     "nothing that could be smuggled through them. Refusing it (the reflex of a validator that "
     "rejects maps whose key type it cannot confirm) would make a legal ext-value un-decodable. "
     "MUST be accepted as the empty text-keyed map"),
    ("array_empty", enc_array([]),
     "the EMPTY array, 0x80 — accepted on the same reasoning as map_empty and with no ambiguity "
     "at all; listed so the two empties are frozen as a pair"),
]

VAL_REJECT = [
    ("int_keyed_map", enc_map([(1, enc_tstr("v"))]),
     "INTEGER-keyed map — ext-value has no such arm; this is the shape a product reaches for "
     "second, and it validates cleanly to `false` rather than crashing"),
    ("float64", bytes.fromhex("fb3ff0000000000000"),
     "float (1.0) — excluded: no canonical form across encoders"),
    ("null", bytes.fromhex("f6"),
     "null — excluded, which is why §4.1.1 requires a DISCRIMINATED empty value rather than an "
     "ad-hoc sentinel"),
    ("undefined", bytes.fromhex("f7"), "undefined — excluded"),
    ("tag", bytes.fromhex("c07818323032362d30372d32305430303a30303a30305a"),
     "tag 0 (standard date-time string) — tags are excluded; the tagged item is otherwise a tstr, "
     "so the rejection is on the TAG, not on what it wraps"),
    ("nested_int_keyed_map", enc_tstr_map([("meta", enc_map([(1, enc_uint(0))]))]),
     "a valid text-keyed map whose VALUE is an integer-keyed map — validation is RECURSIVE, so "
     "the violation is caught at depth 2, not waved through by a shallow check"),
]

add(
    "sync_ext_value_boundary",
    "sync_ext_value_validate",
    {
        "accept": [{"case": n, "cbor_hex": b.hex(), "note": d} for n, b, d in VAL_ACCEPT],
        "reject": [{"case": n, "cbor_hex": b.hex(), "note": d} for n, b, d in VAL_REJECT],
        "carrier_op_cbor_hex": encode_sync_op(
            kind=3, ns="", target="slide:s1", field="obj:shape-a",
            value=val_nested, hlc_bytes=encode_hlc(HLC_WALL, 4, PK_A)).hex(),
    },
    {
        "accept_all": True,
        "reject_all": True,
        "reject_error_code": "0x0A03",
        "reject_error_name": "ERR_SYNC_OP_INVALID",
        "validation_is_recursive": True,
        "carrier_op_accepted": True,
        "empty_map_is_ext_value": True,
        "empty_map_cbor_hex": "a0",
        "empty_map_key_type_is_undeterminable": True,
        "empty_map_note": "0xa0 encodes an empty map of ANY key type; it is accepted as the empty "
                          "{ * tstr => ext-value }. The ambiguity is vacuous — no entries, hence "
                          "nothing to smuggle — and refusing it would make a legal ext-value "
                          "un-decodable (§4.1, §14 C-14).",
        "nonempty_int_keyed_map_still_rejected": True,
        "max_nesting_depth": 64,
        "max_nesting_depth_is_a_MUST": True,
        "max_nesting_depth_note": "§4.1 (§14 C-14): the ceiling is REQUIRED and fixed at 64 "
                                  "container levels (outermost value = depth 0), the same number "
                                  "DMTAP's deterministic-CBOR decoder applies to every other "
                                  "object, so one encoder cannot mint a value a second decoder "
                                  "refuses. It MUST be checked BEFORE recursing — an over-deep "
                                  "item is a 0x0A03 refusal, never a stack exhaustion — and it "
                                  "applies to ALL sync decoding (ops, PullResponse, SnapshotBody, "
                                  "fingerprint requests), not only to `value`. No byte-exact "
                                  "over-deep case is frozen here on purpose: an off-by-one at the "
                                  "boundary would freeze an accident rather than the rule.",
        "value_profile_subtoken": "sync-1/ext-value-2",
        "value_profile_subtoken_is_a_gate": False,
        "value_profile_subtoken_note": "§4.1.2 (§14 C-13): a node accepting the full ext-value MAY "
                                       "advertise `sync-1/ext-value-2` — in the sync-1 capability "
                                       "token and/or as `profiles` (key 4) in the GET /sync/vector "
                                       "response. It is OBSERVATIONAL: absence means 'unknown', "
                                       "never 'profile 1'; a node MUST NOT refuse, downgrade, or "
                                       "narrow its own validation on it. Its one conformant use is "
                                       "a PRODUCER deciding whether to mint nested values.",
    },
    "§4.1/§4.1.1 (C-08): a SyncOp `value` is EXACTLY §18.3.6's `ext-value` — "
    "bool / int / bytes / tstr / [* ext-value] / { * tstr => ext-value } — recursive, with "
    "heterogeneous arrays and TEXT-KEYED maps both admitted. §4.1 previously described a strictly "
    "narrower type under the same name, dropping the map arm and adding a homogeneity constraint "
    "§18.3.6 does not impose, which left nested application data blocked twice over: no tag for a "
    "string-keyed map (an ENCODER failure), and the integer-keyed map CBOR does offer correctly "
    "not an ext-value (a VALIDATION `false`). Both refusals are frozen here, on opposite sides of "
    "the boundary, so the two are never conflated again. Determinism is unaffected: §2.2 "
    "canonicalizes text keys by ENCODED key bytes exactly as it does integer keys, applied "
    "recursively, and §18.3.6's `Headers.ext` already carries this same recursive type INSIDE a "
    "signature preimage on every MOTE. Validation MUST be recursive (see the depth-2 reject case) "
    "and MUST fail closed at the first violating node — 0x0A03, never a canonicalization guess. "
    "The carrier op shows the intended shape end-to-end: one LWW register per (slide, object), "
    "nesting used for REPRESENTATION while §4.1.1's merge boundary stays at the whole value. "
    "C-14 adds the one case the original boundary did not cover: the EMPTY map 0xa0, which is "
    "key-type-AMBIGUOUS (0xa0 is the encoding of an empty map whatever its keys would have been) "
    "and is nonetheless a legal ext-value, so it MUST be accepted — the ambiguity carries no "
    "entries and therefore no smuggling channel, while refusing it would make a legal value "
    "un-decodable. The depth ceiling is also fixed here at 64 and made a MUST rather than left to "
    "each implementation's 'ordinary' ceiling, and C-13's `sync-1/ext-value-2` sub-token is the "
    "(advisory, never gating) handle by which a peer's profile can be asked about at all.",
)

# ══════════════════════════════════════════════════════════════════════════════════════
# SYNC-FJ-01 / SYNC-FJ-02 — §5.2.1 fast-join `pull` response (frozen; correction C-05)
# ══════════════════════════════════════════════════════════════════════════════════════
SNAP_TS = 1_700_000_200_000  # fixed; strictly after every HLC wall folded into `covers`


def encode_snapshot(ns: str, covers: bytes, root: bytes, ts: int, sk, signer: bytes) -> bytes:
    """§6.1 Snapshot. Keys 1..7 are signed; key 8 is the signature over
    DS-tag("DMTAP-SYNC-v0/snapshot" || 0x00) || det_cbor(Snapshot \\ {8}) — the §18.1.6 rule."""
    body_pairs = [
        (1, enc_uint(0)),            # v
        (2, enc_uint(1)),            # suite 0x01 (classical)
        (3, enc_tstr(ns)),           # ns
        (4, covers),                 # covers — §5.1 VersionVector, ik-pub bstr keys
        (5, enc_bstr(root)),         # root — §6.1.1 observable-state content address
        (6, enc_uint(ts)),           # ts
        (7, enc_bstr(signer)),       # signer
    ]
    body = enc_map(body_pairs)
    sig = sk.sign(DS_SNAPSHOT + body)
    return enc_map(body_pairs + [(8, enc_bstr(sig))]), body, sig


snapshot_fj, snapshot_fj_body, snapshot_fj_sig = encode_snapshot(
    ns="", covers=covers_v1, root=root_v1, ts=SNAP_TS, sk=SK_A, signer=PK_A
)

# The responder truncated its op-log below this §6.2 floor; the snapshot at `covers_v1` replaces
# the discarded prefix. The floor is the cut itself — the caller's audit handle on what was dropped.
floor_hlc = encode_hlc(HLC_WALL, 5, PK_A)

fastjoin_map = enc_map([(1, snapshot_fj), (2, floor_hlc)])
pull_resp_fastjoin = enc_map([(2, fastjoin_map)])

# The ordinary {1: ops} answer, for contrast. §5.2 op framing (correction C-06): the member is a REAL
# COSE_Sign1 (§4.1) embedded as a CBOR ITEM, never bstr-wrapped. op_post is authored by B, so it is
# signed under SK_B with kid = PK_B. The non-conformant bstr-wrapped framing is emitted alongside it
# purely so an implementer can see which byte string is the WRONG one.
cose_post = cose_sign1(op_post, SK_B, PK_B)
pull_resp_ops = enc_map([(1, enc_array([cose_post]))])
pull_resp_ops_bstr_wrapped_NONCONFORMANT = enc_map([(1, enc_array([enc_bstr(cose_post)]))])

# ── The §6.1.2 SnapshotBody for this snapshot (correction C-11) ──────────────────────────────────
# Key 3 carries the BODY, and after C-09 the body is a compacted OP SET — not det_cbor(
# ObservableState). This vector previously froze the state document here, which is the one value a
# caller must NOT adopt; §10's row claimed C-09 "does not change this vector's bytes — key 3 is
# absent", true of pull_response_cbor_hex and false of the inline variant the vector also carries.
#
# The body is the §6.2 RETENTION SET for state_v1 — one op per live element, and nothing else.
# Op → the §6.1.1 section entry it produces:
#
#   (W,0,A) counter      stock1/qty += 5        ┐
#   (W,0,B) counter      stock1/qty += −2       ┘→ pn    [stock1,qty,3]   BOTH retained: no counter
#                                                        delta is ever superseded (C-13)
#   (W,1,A) lww-set      doc1/title = "n"        → lww   [doc1,title,n]
#   (W,2,A) set-add      "e1" on "tags"          → orset [tags,e1]
#   (W,3,A) seq-insert   "atom0" on line1, ⊥     ┐
#   (W,6,B) seq-insert   "Y"  ref=atom0          ├→ rga   [line1,[atom0,Y,X]]  same-origin siblings
#   (W,5,B) seq-insert   "X"  ref=atom0          ┘       order by DESCENDING element id: Y then X
#   (W,4,A) death        rec1, class "redact"    → death [rec1,redact]
#   (W,4,B) tree-move    B under "" , ord "b"    ┐
#   (W,7,B) tree-move    A under B  , ord "1"    ┘→ tree  [A,B,1] [B,"",b]   replayed oldest-first:
#                                                        B lands at the root, then A under B (no
#                                                        cycle, so no §4.8 skip)
#
# HLCs are strictly monotonic per author (A: 0,1,2,3,4 — B: 0,4,5,6,7), as §3's tick rule requires,
# and every one lies within `covers` = {A@(W,4), B@(W,7)}. These are freshly minted rather than
# reused from this file's earlier vectors: those vectors are independent scenarios that reuse
# counters across each other, so composing their literal ops into one history would freeze a journal
# in which an author minted two ops at the same HLC. The observable state they fold to is identical,
# which is what `root` commits to.
fjb_pn_a = encode_sync_op(kind=5, ns="", target="stock1", field="qty", value=enc_int(5),
                          hlc_bytes=encode_hlc(HLC_WALL, 0, PK_A))
fjb_pn_b = encode_sync_op(kind=5, ns="", target="stock1", field="qty", value=enc_int(-2),
                          hlc_bytes=encode_hlc(HLC_WALL, 0, PK_B))
fjb_lww = encode_sync_op(kind=3, ns="", target="doc1", field="title", value=enc_tstr("n"),
                         hlc_bytes=encode_hlc(HLC_WALL, 1, PK_A))
fjb_add = encode_sync_op(kind=1, ns="", target="tags", value=enc_tstr("e1"),
                         hlc_bytes=encode_hlc(HLC_WALL, 2, PK_A))
fjb_atom0_hlc = encode_hlc(HLC_WALL, 3, PK_A)
fjb_rga_atom0 = encode_sync_op(kind=6, ns="", target="line1", value=enc_tstr("atom0"),
                               hlc_bytes=fjb_atom0_hlc)                      # ⊥ = list head
fjb_rga_x = encode_sync_op(kind=6, ns="", target="line1", value=enc_tstr("X"),
                           hlc_bytes=encode_hlc(HLC_WALL, 5, PK_B),
                           ref=encode_opref("line1", fjb_atom0_hlc))
fjb_rga_y = encode_sync_op(kind=6, ns="", target="line1", value=enc_tstr("Y"),
                           hlc_bytes=encode_hlc(HLC_WALL, 6, PK_B),
                           ref=encode_opref("line1", fjb_atom0_hlc))
fjb_death = encode_sync_op(kind=4, ns="", target="rec1", field="redact",
                           hlc_bytes=encode_hlc(HLC_WALL, 4, PK_A))
fjb_tree_b = encode_sync_op(kind=8, ns="", target="B", field="b",
                            hlc_bytes=encode_hlc(HLC_WALL, 4, PK_B), ref=encode_opref(TREE_ROOT))
fjb_tree_a = encode_sync_op(kind=8, ns="", target="A", field="1",
                            hlc_bytes=encode_hlc(HLC_WALL, 7, PK_B), ref=encode_opref("B"))

snap_body_members = [
    ((HLC_WALL, 0, PK_A), fjb_pn_a, SK_A, PK_A, {"kind": 5, "target": "stock1", "field": "qty", "value": 5}),
    ((HLC_WALL, 0, PK_B), fjb_pn_b, SK_B, PK_B, {"kind": 5, "target": "stock1", "field": "qty", "value": -2}),
    ((HLC_WALL, 1, PK_A), fjb_lww, SK_A, PK_A, {"kind": 3, "target": "doc1", "field": "title", "value": "n"}),
    ((HLC_WALL, 2, PK_A), fjb_add, SK_A, PK_A, {"kind": 1, "target": "tags", "value": "e1"}),
    ((HLC_WALL, 3, PK_A), fjb_rga_atom0, SK_A, PK_A, {"kind": 6, "target": "line1", "value": "atom0", "ref": None}),
    ((HLC_WALL, 4, PK_A), fjb_death, SK_A, PK_A, {"kind": 4, "target": "rec1", "field": "redact"}),
    ((HLC_WALL, 4, PK_B), fjb_tree_b, SK_B, PK_B, {"kind": 8, "target": "B", "parent": TREE_ROOT, "ord": "b"}),
    ((HLC_WALL, 5, PK_B), fjb_rga_x, SK_B, PK_B, {"kind": 6, "target": "line1", "value": "X", "ref": "atom0"}),
    ((HLC_WALL, 6, PK_B), fjb_rga_y, SK_B, PK_B, {"kind": 6, "target": "line1", "value": "Y", "ref": "atom0"}),
    ((HLC_WALL, 7, PK_B), fjb_tree_a, SK_B, PK_B, {"kind": 8, "target": "A", "parent": "B", "ord": "1"}),
]
# Member order is (hlc, op-id) ascending. The root commits to the FOLD, not to these bytes, so the
# order is NOT a correctness requirement (§5.2: two replicas MAY serialize different valid bodies
# that fold to the same root); it is fixed so two replicas compacting the same state produce the
# same bytes and the body dedupes in a content-addressed store instead of forking per replica.
_snap_body_sorted = sorted(
    ((h, content_addr(DS_OP_ID, o), cose_sign1(o, sk, pk), o, d)
     for h, o, sk, pk, d in snap_body_members),
    key=lambda t: (t[0], t[1]),
)
snapshot_body_fj = enc_array([c for _, _, c, _, _ in _snap_body_sorted])
snapshot_body_fj_ops = [
    dict(d, hlc={"wall": h[0], "counter": h[1], "author_hex": h[2].hex()},
         syncop_cbor_hex=o.hex(), op_id_hex=i.hex())
    for h, i, _c, o, d in _snap_body_sorted
]

# The OPTIONAL bounded inline copy (key 3): the same SnapshotBody bytes the caller would otherwise
# fetch from GET /sync/state/<root>. It is a CACHE HINT — verified by FOLDING it and recomputing
# Snapshot.root exactly as a fetched body is (§6.1.2), and discarded whole on mismatch.
fastjoin_map_inline = enc_map([(1, snapshot_fj), (2, floor_hlc), (3, enc_bstr(snapshot_body_fj))])
pull_resp_fastjoin_inline = enc_map([(2, fastjoin_map_inline)])

add(
    "sync_fastjoin_response",
    "sync_fastjoin_pull_response",
    {
        "scenario": "The responder truncated its op-log below floor (W,5,A) per §6.2, retaining the "
                    "signed snapshot at covers {A@(W,4), B@(W,7)}. A caller whose vector lacks HLCs "
                    "inside `covers` pulls.",
        "snapshot_ns": "",
        "snapshot_covers_cbor_hex": covers_v1.hex(),
        "snapshot_root_hex": root_v1.hex(),
        "snapshot_ts": SNAP_TS,
        "snapshot_signer_seed_hex": SEED_SYNC_A.hex(),
        "snapshot_signer_pubkey_hex": PK_A.hex(),
        "snapshot_sig_ds_tag_hex": DS_SNAPSHOT.hex(),
        "floor_hlc_cbor_hex": floor_hlc.hex(),
        "observable_state_cbor_hex": state_v1.hex(),
        "observable_state_role": "the COMMITMENT hashed to produce `root` (§6.1.1) — NOT the body, "
                                 "NOT adoptable, and NOT what key 3 carries (§6.1.2, C-09/C-11).",
        "snapshot_body_cbor_hex": snapshot_body_fj.hex(),
        "snapshot_body_ops": snapshot_body_fj_ops,
        "snapshot_body_op_count": len(snapshot_body_fj_ops),
        "snapshot_body_role": "the §6.1.2 SnapshotBody — the compacted set of individually-signed "
                              "ops whose fold reproduces observable_state_cbor_hex, hence "
                              "snapshot_root_hex. This is what GET /sync/state/<root> serves and "
                              "what FastJoin key 3 inlines.",
    },
    {
        "snapshot_sig_preimage_hex": (DS_SNAPSHOT + snapshot_fj_body).hex(),
        "snapshot_sig_hex": snapshot_fj_sig.hex(),
        "snapshot_cbor_hex": snapshot_fj.hex(),
        "fastjoin_cbor_hex": fastjoin_map.hex(),
        "pull_response_cbor_hex": pull_resp_fastjoin.hex(),
        "pull_response_with_inline_state_cbor_hex": pull_resp_fastjoin_inline.hex(),
        "pull_response_keys": [2],
        "ops_key_present": False,
        "state_fetch_endpoint": "GET /sync/state/<root>",
        "state_fetch_address_hex": root_v1.hex(),
        "inline_body_is_cache_hint_verified_by_fold_then_recompute": True,
        "inline_body_cbor_hex": snapshot_body_fj.hex(),
        "inline_body_is_an_op_set_not_a_state_document": True,
        "inline_body_folds_to_root_hex": root_v1.hex(),
        "inline_state_document_would_be_nonconformant_cbor_hex": enc_map(
            [(2, enc_map([(1, snapshot_fj), (2, floor_hlc), (3, enc_bstr(state_v1))]))]).hex(),
        "inline_state_document_note": "the pre-C-09 shape this vector used to freeze, published so "
                                      "it can be recognized as WRONG rather than copied. A caller "
                                      "decodes key 3 as a SnapshotBody — an array of COSE_Sign1 — "
                                      "so det_cbor(ObservableState) fails at op decode (0x0A03); "
                                      "either way it never folds to `root`, so the body is "
                                      "discarded and the caller falls back to GET "
                                      "/sync/state/<root>. A caller that instead ADOPTED it would "
                                      "be non-conformant per §6.1.2.",
    },
    "§5.2.1 (frozen): the `pull` response is a deterministic-CBOR integer-keyed map whose two keys "
    "are MUTUALLY EXCLUSIVE — {1: ops} is the ordinary answer, {2: FastJoin} is the answer to a "
    "caller below the responder's §6.2 truncation floor. FastJoin = {1: Snapshot, 2: floor Hlc, "
    "?3: inline det_cbor(ObservableState)}. The ENCODING SPLIT is the point: the §6.1 signed "
    "descriptor ships INLINE (it is bounded — sized by the author count, not the data — and it "
    "carries the signature, `covers` and the `root` commitment), while the UNBOUNDED BODY ships "
    "BY REFERENCE at Snapshot.root, fetched from GET /sync/state/<root>. By-reference "
    "keeps a sync round's response size bounded and reuses the content-addressing the protocol "
    "already has: the body is immutable and self-verifying, so any relay may cache and pin it, any "
    "holder may serve it, and every peer fast-joining to the same `covers` dedupes to the same "
    "bytes. The cost is one extra round trip on a path taken only when a peer has fallen behind a "
    "truncation cut. Key 3 is an OPTIONAL bounded inline copy (RECOMMENDED ceiling 64 KiB) that "
    "collapses the small-namespace case to one round trip; it MUST be verified exactly as a "
    "fetched body is and discarded on mismatch, so there is ONE verification path, not two. The "
    "snapshot signature is the §18.1.6 preimage DS-tag || det_cbor(Snapshot \\ {8}) with the tag "
    "\"DMTAP-SYNC-v0/snapshot\", distinct from the state-root tag "
    "\"DMTAP-SYNC-v0/snapshot-state\". "
    "REGENERATED FOR C-09 (correction C-11): key 3 now carries a §6.1.2 SnapshotBody — the "
    "compacted set of ten individually-signed ops whose FOLD reproduces the observable state and "
    "hence `root` — where it previously carried det_cbor(ObservableState), the one value §6.1.2 "
    "says a caller must NOT adopt. C-09 recorded that this vector's bytes were unaffected because "
    "key 3 is absent, which is true of `pull_response_cbor_hex` and FALSE of "
    "`pull_response_with_inline_state_cbor_hex`, which the vector also carries and froze: a frozen "
    "vector was pinning an unadoptable value. Verification of key 3 is therefore "
    "FOLD-THEN-RECOMPUTE (§6.1.2), not a hash of the transferred bytes — the body is not required "
    "to be byte-stable across producers, only to fold to `root`. The body is the §6.2 RETENTION "
    "SET for this state: one uncancelled set-add, the winning lww-set, BOTH counter deltas (no "
    "counter op is ever superseded — C-13), the winning death certificate, all three live RGA "
    "inserts, and the winning tree-move per node. `snapshot_body_ops` lists each op with its "
    "op-id; member order is (hlc, op-id) ascending, which is a determinism convenience for "
    "content-addressed caching and NOT a conformance requirement.",
)

# Caller B is AT/ABOVE the floor: its vector dominates everything in `covers`, so the surviving
# journal suffix IS a complete answer and the ordinary {1: ops} response is correct.
covers_caught_up = enc_bstr_map([
    (PK_A, encode_hlc(HLC_WALL, 9, PK_A)),
    (PK_B, encode_hlc(HLC_WALL, 9, PK_B)),
])
# Caller A is BELOW the floor: it lacks B@(W,7), which `covers` folded in.
covers_behind = enc_bstr_map([
    (PK_A, encode_hlc(HLC_WALL, 4, PK_A)),
    (PK_B, encode_hlc(HLC_WALL, 2, PK_B)),
])
add(
    "sync_fastjoin_below_floor_suffix_forbidden",
    "sync_fastjoin_floor_predicate",
    {
        "responder_floor_hlc_cbor_hex": floor_hlc.hex(),
        "responder_floor_hlc": {"wall": HLC_WALL, "counter": 5, "author_hex": PK_A.hex()},
        "responder_snapshot_covers_cbor_hex": covers_v1.hex(),
        "responder_snapshot_covers_note": "A@(W,4), B@(W,7) — note covers[A] is BELOW the floor HLC "
                                          "(W,5,A): author A produced no op in that window. This is "
                                          "well-formed (§5.2.2).",
        "caller_behind_vector_cbor_hex": covers_behind.hex(),
        "caller_behind_note": "lacks B@(W,7), an HLC folded into the responder's snapshot `covers`",
        "caller_caught_up_vector_cbor_hex": covers_caught_up.hex(),
        "caller_caught_up_note": "dominates every HLC in `covers`; the surviving suffix is complete",
        "surviving_suffix_ops_cbor_hex": [cose_post.hex()],
        "surviving_suffix_ops_framing": "each member is a §4.1 COSE_Sign1 four-element array embedded "
                                        "as a CBOR ITEM (§5.2 op framing, correction C-06)",
        "surviving_suffix_inner_syncop_cbor_hex": [op_post.hex()],
    },
    {
        "predicate": "behind_floor := exists (author, hlc) in Snapshot.covers such that "
                     "caller_vector.lacks(hlc)",
        "caller_behind_is_below_floor": True,
        "caller_behind_response_cbor_hex": pull_resp_fastjoin.hex(),
        "caller_behind_ops_response_forbidden": True,
        "caller_behind_ops_response_would_be_cbor_hex": pull_resp_ops.hex(),
        "caller_caught_up_is_below_floor": False,
        "caller_caught_up_response_cbor_hex": pull_resp_ops.hex(),
        "caller_caught_up_fastjoin_forbidden": True,
        "ops_member_framing": "item-embedded COSE_Sign1",
        "ops_member_bstr_wrapped_conformant": False,
        "ops_member_bstr_wrapped_NONCONFORMANT_cbor_hex":
            pull_resp_ops_bstr_wrapped_NONCONFORMANT.hex(),
        "ops_member_bstr_wrapped_error_code": "0x0A03",
        "floor_vs_covers_is_orderable": False,
        "floor_vs_covers_naive_predicate_rejected": "covers.lacks(floor)",
        "floor_vs_covers_naive_predicate_value_here": True,
        "floor_vs_covers_naive_predicate_note": "the naive predicate evaluates TRUE on this "
            "well-formed fast-join and would reject a conformant responder: `floor` is a single Hlc, "
            "`covers` is a per-author VersionVector, and there is no ordering between them (§5.2.2). "
            "No floor-vs-covers check exists in this specification.",
        "covers_carries_mark_for_floor_author": True,
        "covers_mark_for_floor_author_is_MUST": False,
        "caller_trusts_all_truncated_ops_folded_into_covers": True,
        "repeated_fastjoin_same_root_and_covers_error_code": "0x0A09",
        "repeated_fastjoin_same_root_and_covers_action": "fail the round; do NOT re-adopt",
        "state_body_unfetchable_error_code": "0x0A0C",
        "state_body_unfetchable_error_name": "ERR_SYNC_SNAPSHOT_STATE_UNAVAILABLE",
        "state_body_unfetchable_action": "FAIL_CLOSED_BLOCK",
        "state_body_unfetchable_caller_vector_unchanged": True,
        "suffix_fallback_after_failed_fastjoin_forbidden": True,
        "adopting_covers_may_regress_caller_vector": True,
        "adopting_covers_regression_is_an_error": False,
    },
    "§5.2.1 (frozen), the MUST this vector exists for: a responder whose §6.2 truncation floor is "
    "above the caller's vector MUST NOT answer with {1: ops}. The response shown as "
    "`caller_behind_ops_response_would_be_cbor_hex` is well-formed and would be applied without "
    "error — that is exactly the danger. It is INDISTINGUISHABLE to the caller from a complete "
    "answer, so the caller would advance its vector, believe itself converged, and have silently "
    "lost every truncated op: a lost write presented as a successful sync. An error is not the "
    "ordinary answer either, since it would strand a peer that has a working recovery path; an "
    "error (0x0A0C) is reserved for the responder that cannot honour the path at all. The "
    "predicate is domination of the snapshot's `covers`, NOT a comparison against the floor Hlc "
    "alone: if the caller lacks any HLC the snapshot folded in, some op it needs may have been "
    "truncated. Conversely a caught-up caller MUST NOT be forced to fast-join — the suffix is a "
    "complete answer for it, and fast-joining it would discard verified local history for a "
    "trusted checkpoint. Caller-side fail-closed: a state body no holder can serve is 0x0A0C "
    "with the caller's vector UNCHANGED — never a fallback to the truncated suffix, which would "
    "reintroduce the silent loss by the back door; and a repeated `fast-join` at the same "
    "root/`covers` is 0x0A09 (the responder is looping, §5.2.1 step 5's progress MUST). "
    "TWO CORRECTIONS ARE FROZEN HERE. (C-06, §5.2 op framing) the members of an `ops` array are "
    "§4.1 COSE_Sign1 four-element ARRAYS embedded as CBOR ITEMS, never bstr-wrapped — nothing is "
    "hashed or verified over the outer COSE_Sign1 encoding (the signature preimage is built from "
    "the protected/payload bstrs, the op-id from det_cbor(SyncOp) = the payload contents), so a "
    "wrapper buys no integrity property and only adds a framing two implementations can disagree "
    "on. `ops_member_bstr_wrapped_NONCONFORMANT_cbor_hex` is the WRONG encoding, published so it "
    "can be recognized; it is 0x0A03. This vector's ops are real signed COSE_Sign1 envelopes, not "
    "bare SyncOp stand-ins — the earlier stand-in is why the ambiguity survived. "
    "(C-07, §5.2.2 floor vs covers) `floor` is a single Hlc, `covers` is a per-author "
    "VersionVector; there is NO ordering between them and this specification states no "
    "floor-vs-covers rule. The tempting `covers.lacks(floor)` predicate evaluates TRUE on THIS "
    "well-formed data — floor (W,5,A) sits above covers[A]=(W,4) because A produced no op in "
    "that window — and an implementation using it rejects conformant responders. The caller "
    "VERIFIES: signature/admission/ns, covers well-formedness, the body against root, and "
    "progress. It MAY check that covers carries a mark for floor.author (a logging-grade signal; "
    "NOT a MUST, since an author whose only op is AT the floor is retained, not truncated). It "
    "TRUSTS that every truncated op was folded into covers — a statement about ops it cannot "
    "see, discharged at the responder by §6.2 and backed by §6.1's trust policy. Adopting covers "
    "may move the caller's vector BACKWARDS for an author; that is intended, not an error, since "
    "step 5's re-pull re-ships the retained suffix.",
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
    "the RFC 9052 COSE_Sign1 op envelope (§4.1), the canonical observable-state root (§6.1.1), the "
    "range-Merkle fingerprint fold (§5.3) and the §5.2.1 fast-join `pull` response",
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
    "scope_note": "This file freezes ALL 24 of substrate/SYNC.md §10's conformance vectors. The five that "
    "were previously NOT-FROZEN — SYNC-OP-02 (COSE_Sign1 envelope framing), SYNC-TREE-01 (which side of a "
    "concurrent-move cycle loses), SYNC-SNAP-01/02 (canonical observable-state schema) and SYNC-RECON-01 "
    "(range-Merkle fingerprint fold) — were each resolved by adding normative frozen text to the "
    "specification FIRST (§4.1, §4.8, §6.1.1, §5.3 respectively, each with its rationale) and only then "
    "vectored here. SYNC-FJ-01/SYNC-FJ-02 are NEW (correction C-05) and follow the same discipline: "
    "§5.2.1 decided the response shape, the below-floor MUST NOT and the inline-vs-by-reference "
    "encoding split FIRST, and only then was it vectored. No decision in this file originates in this "
    "file: substrate/SYNC.md is authoritative.",
    "corrections_note": "Regenerated after substrate/SYNC.md §14's corrections C-01..C-04, all of which "
    "were surfaced by an independent Rust implementation of SYNC.md (envoir `dmtap-sync`), not by review: "
    "C-01 changed §4.6's PN-counter merge from per-author max of P/N to the per-author UNION of "
    "op-id-keyed deltas (the max join is non-associative and LOSES writes when replicas hold different "
    "subsets of one author's deltas — a NORMATIVE merge-semantics correction); C-02 fixed SYNC-PN-01, "
    "whose 'replay' op carried a different HLC and was therefore a distinct op; C-03 fixed SYNC-RGA-02's "
    "atom_order_incl_tombstones, which contradicted §4.7 and the vector's own note; C-04 fixed "
    "SYNC-SNAP-02's snapshot_covers_cbor_hex, which encoded an integer-keyed map where §5.1 specifies "
    "ik-pub bstr keys. C-08/C-09/C-10 came from a different direction — the FIRST real product "
    "adoption of the engine (diwan's Sheets grid) rather than an independent reimplementation — and "
    "are the three gaps only a consumer could surface. C-08: §4.1 named §18.3.6's `ext-value` but "
    "described a strictly narrower type, dropping the recursive text-keyed map arm and adding a "
    "homogeneity constraint that is not there, so nested application data was blocked twice over "
    "(no tag to encode a string-keyed map; the integer-keyed map correctly not an ext-value) for no "
    "determinism reason — the recursive type is canonical under §2.2 and already rides inside a "
    "signature preimage via Headers.ext. NEW vector SYNC-VAL-01 freezes the boundary from both "
    "sides. C-09: the snapshot BODY is a compacted op set, not det_cbor(ObservableState) — the "
    "§6.1.1 projection deliberately drops the merge metadata the next merge needs (the LWW winner's "
    "HLC above all), so adopting it as base state silently diverges; NEW vector SYNC-SNAP-03 "
    "demonstrates the exact ordering case, with a post-`covers` op that is genuinely after `covers` "
    "yet BELOW the incumbent in the §3 total order. C-10 is selection guidance (§4.10, death "
    "certificate vs LWW-empty) and adds no vector — SYNC-DEATH-01 already pins the domination it "
    "warns about. Vector count goes 22 -> 24. C-05 (also found by envoir, while implementing §6.2 "
    "op-log truncation) closed a "
    "HOLE rather than a contradiction: §5.2's `pull` response had no way to say 'you are below my "
    "truncation floor, fast-join from this snapshot', leaving a truncating responder to either return "
    "the surviving suffix — silently losing every truncated op while APPEARING to succeed — or error "
    "and strand the peer. §5.2.1 now specifies the fast-join response, makes the bare suffix a MUST "
    "NOT, and adds GET /sync/state/<root> plus error 0x0A0C. Vector count goes 20 -> 22 (SYNC-FJ-01, "
    "SYNC-FJ-02 added; the other 20 are byte-identical to the previous generation). C-06 and C-07 (both "
    "found by the same implementation, which DECLINED to guess) tighten SYNC-FJ-02 without changing the "
    "count, which stays 22: C-06 pins op framing inside an `ops` array to item-embedded COSE_Sign1, "
    "never bstr-wrapped (§5.2) — this vector's ops are now REAL COSE_Sign1 envelopes rather than bare "
    "SyncOp stand-ins, since a stand-in that lacks the specified shape is precisely how the ambiguity "
    "survived a frozen vector; C-07 removes §5.2.1's unverifiable 'floor MUST NOT exceed covers' clause "
    "(a category error: a single Hlc vs a per-author VersionVector) in favour of §5.2.2's explicit split "
    "between what a caller verifies and what it trusts, and freezes the rejected `covers.lacks(floor)` "
    "predicate here alongside the data on which it wrongly fires. "
    "C-11..C-14 keep the count at 24. C-11 REGENERATES SYNC-FJ-01's inline FastJoin body: C-09 said "
    "making the body an op set left this vector's bytes alone because key 3 is absent, which is true "
    "of pull_response_cbor_hex and FALSE of pull_response_with_inline_state_cbor_hex — the vector was "
    "freezing, as key 3, the one value §6.1.2 says a caller must not adopt. Key 3 now carries a real "
    "SnapshotBody (ten signed ops = §6.2's retention set for this state, folding to the UNCHANGED "
    "root); the old state-document framing is kept as a labelled non-conformant artifact. C-12 and "
    "C-13(a) widen §6.2's retention set (a winning `Live` death cell, a retained tombstoned origin's "
    "own seq-remove, and EVERY counter op — no counter delta is ever superseded) and are vectored "
    "only indirectly, by SYNC-SNAP-03's fold-then-recompute, which is the check that catches a body "
    "missing any of them. C-13(b) adds §4.1.2's `sync-1/ext-value-2` sub-token — the mechanism C-08's "
    "mixed-deployment warning lacked — recorded here as ADVISORY: absence means unknown, never "
    "profile 1, and a node MUST NOT refuse or downgrade on it. C-14 extends SYNC-VAL-01 with the "
    "empty map 0xa0 (key-type-ambiguous but vacuously so, and legal, so it MUST be accepted) and the "
    "empty array 0x80, and freezes the nesting-depth ceiling at 64 as a MUST covering ALL sync "
    "decoding rather than `value` alone.",
    "vectors": vectors,
}
print(json.dumps(out, indent=2))
