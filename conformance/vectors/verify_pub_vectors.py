#!/usr/bin/env python3
"""Independent re-derivation of every conformance/vectors/pub_vectors.json entry.

Deliberately does NOT import gen_pub_vectors.py — this is a from-scratch second
implementation of the same formulas, to catch bugs the generator itself might share
with a naive check. Exits nonzero on any mismatch.
"""
import json
import os
import sys
import blake3
from cryptography.hazmat.primitives.asymmetric import ed25519
from cryptography.hazmat.primitives import serialization

PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "pub_vectors.json")
d = json.load(open(PATH))
byname = {v["name"]: v for v in d["vectors"]}
fails = []


def check(name, cond, msg=""):
    status = "OK" if cond else "FAIL"
    print(f"[{status}] {name} {msg}")
    if not cond:
        fails.append(name)


def b3(b):
    return blake3.blake3(b).digest()


def ca(b):
    return b"\x1e" + b3(b)


# CBOR minimal decoder for the flat integer-keyed maps used here (major types 0/2/4/5 only)
def _enc_head(major, n):
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


def cbor_read_head(buf, i):
    b0 = buf[i]
    major = b0 >> 5
    ai = b0 & 0x1F
    i += 1
    if ai < 24:
        return major, ai, i
    if ai == 24:
        return major, buf[i], i + 1
    if ai == 25:
        return major, int.from_bytes(buf[i:i+2], "big"), i + 2
    if ai == 26:
        return major, int.from_bytes(buf[i:i+4], "big"), i + 4
    if ai == 27:
        return major, int.from_bytes(buf[i:i+8], "big"), i + 8
    raise ValueError("unsupported ai")


def cbor_skip_value(buf, i):
    major, n, j = cbor_read_head(buf, i)
    if major in (0, 1):
        return j
    if major == 2:
        return j + n
    if major == 4:
        for _ in range(n):
            j = cbor_skip_value(buf, j)
        return j
    if major == 5:
        for _ in range(n):
            j = cbor_skip_value(buf, j)
            j = cbor_skip_value(buf, j)
        return j
    raise ValueError(f"major {major}")


def cbor_map_spans(buf):
    """key -> (kv_start, kv_end) byte span covering that key's key+value bytes."""
    major, n, i = cbor_read_head(buf, 0)
    assert major == 5
    spans = {}
    for _ in range(n):
        kv_start = i
        kmaj, k, j = cbor_read_head(buf, i)
        assert kmaj == 0
        j = cbor_skip_value(buf, j)
        spans[k] = (kv_start, j)
        i = j
    return spans


def cbor_strip_key(buf, drop_key):
    """Re-encode buf's top-level map with `drop_key` removed (rebuilds the header's field
    count; all other bytes are reused verbatim from their original spans)."""
    spans = cbor_map_spans(buf)
    kept = sorted(k for k in spans if k != drop_key)
    out = _enc_head(5, len(kept))
    for k in kept:
        s, e = spans[k]
        out += buf[s:e]
    return out


def cbor_decode_map(buf):
    major, n, i = cbor_read_head(buf, 0)
    assert major == 5, f"expected map, got major {major}"
    out = {}
    for _ in range(n):
        kmaj, k, i = cbor_read_head(buf, i)
        assert kmaj == 0
        vmaj, vlen_or_val, i2 = cbor_read_head(buf, i)
        if vmaj == 2:  # bstr
            out[k] = ("bstr", buf[i2:i2+vlen_or_val])
            i = i2 + vlen_or_val
        elif vmaj == 0:  # uint
            out[k] = ("uint", vlen_or_val)
            i = i2
        elif vmaj == 4:  # array of bstr (only usage here)
            arr = []
            j = i2
            for _ in range(vlen_or_val):
                amaj, alen, j2 = cbor_read_head(buf, j)
                assert amaj == 2
                arr.append(buf[j2:j2+alen])
                j = j2 + alen
            out[k] = ("array", arr)
            i = j
        elif vmaj == 5:  # nested map (meta) — only empty map used here
            assert vlen_or_val == 0
            out[k] = ("map", {})
            i = i2
        else:
            raise ValueError(f"unhandled major {vmaj}")
    return out, i


# ── 1/2: pub_manifest_{single,three}_chunk ────────────────────────────────────────────────
DS_MANIFEST = b"DMTAP-PUB-v0/manifest\x00"

for name in ("pub_manifest_single_chunk", "pub_manifest_three_chunks"):
    v = byname[name]
    chunks = [bytes.fromhex(x) for x in v["input"]["plaintext_chunks_hex"]]
    hs = [ca(c) for c in chunks]
    check(f"{name}/chunk_hashes", [h.hex() for h in hs] == v["expected"]["chunk_hashes_hex"])

    def leaf(h):
        return b3(DS_MANIFEST + b"\x00" + h)

    def node(l, r):
        return b3(DS_MANIFEST + b"\x01" + l + r)

    def mth(hs):
        n = len(hs)
        if n == 1:
            return leaf(hs[0])
        # different k-search style than the generator (linear scan) to be a genuinely
        # independent implementation of the same RFC 6962 split rule
        k = 1
        while (k << 1) < n:
            k <<= 1
        return node(mth(hs[:k]), mth(hs[k:]))

    root = b"\x1e" + mth(hs)
    check(f"{name}/id", root.hex() == v["expected"]["id_hex"])

# ── 3: pub_manifest_type_incompatibility ──────────────────────────────────────────────────
v = byname["pub_manifest_type_incompatibility"]
hs = [bytes.fromhex(x) for x in v["input"]["chunk_hashes_hex"]]


def sleaf(h):
    return b3(b"\x00" + h)


def snode(l, r):
    return b3(b"\x01" + l + r)


def smth(hs):
    n = len(hs)
    if n == 1:
        return sleaf(hs[0])
    k = 1
    while (k << 1) < n:
        k <<= 1
    return snode(smth(hs[:k]), smth(hs[k:]))


sealed_root = b"\x1e" + smth(hs)
check("pub_manifest_type_incompatibility/sealed_style_root", sealed_root.hex() == v["expected"]["sealed_style_root_hex"])
check("pub_manifest_type_incompatibility/roots_differ", sealed_root.hex() != v["expected"]["public_root_hex"])

# ── 4: pub_manifest_key5_forbidden — decode and confirm key 5 present + fields consistent ──
v = byname["pub_manifest_key5_forbidden"]
cbor = bytes.fromhex(v["input"]["cbor_hex"])
decoded, consumed = cbor_decode_map(cbor)
check("pub_manifest_key5_forbidden/full_consumed", consumed == len(cbor))
check("pub_manifest_key5_forbidden/key5_present", 5 in decoded, str(sorted(decoded.keys())))
check("pub_manifest_key5_forbidden/keys_ascending", list(decoded.keys()) == sorted(decoded.keys()))
# cross-check id (key1) matches the single-chunk manifest's id
single = byname["pub_manifest_single_chunk"]
check("pub_manifest_key5_forbidden/id_matches_single_chunk", decoded[1][1].hex() == single["expected"]["id_hex"])

# ── 5/6: pub_announce_signing_preimage, pub_announce_id ───────────────────────────────────
v_pre = byname["pub_announce_signing_preimage"]
seed = bytes.fromhex(v_pre["input"]["seed_hex"])
domain = bytes.fromhex(v_pre["input"]["domain_hex"])
msg = bytes.fromhex(v_pre["input"]["msg_hex"])
check("pub_announce_signing_preimage/domain", domain == b"DMTAP-PUB-v0/announce\x00")
sk = ed25519.Ed25519PrivateKey.from_private_bytes(seed)
pk = sk.public_key().public_bytes(encoding=serialization.Encoding.Raw, format=serialization.PublicFormat.Raw)
sig = sk.sign(domain + msg)
check("pub_announce_signing_preimage/pubkey", pk.hex() == v_pre["expected"]["pubkey_hex"])
check("pub_announce_signing_preimage/sig", sig.hex() == v_pre["expected"]["sig_hex"])

# decode the announce body (msg) and confirm structure: keys 1,2,3,4,5,7,8 ascending, v=0, suite=1
decoded_body, consumed = cbor_decode_map(msg)
check("pub_announce_signing_preimage/body_keys", list(decoded_body.keys()) == [1, 2, 3, 4, 5, 7, 8])
check("pub_announce_signing_preimage/body_v0", decoded_body[1] == ("uint", 0))
check("pub_announce_signing_preimage/body_suite1", decoded_body[2] == ("uint", 1))
check("pub_announce_signing_preimage/body_pub_is_pk", decoded_body[3][1] == pk)

v_id = byname["pub_announce_id"]
full = bytes.fromhex(v_id["input"]["bytes_hex"])
check("pub_announce_id/id", ca(full).hex() == v_id["expected"]["id_hex"])
decoded_full, consumed = cbor_decode_map(full)
check("pub_announce_id/full_consumed", consumed == len(full))
check("pub_announce_id/full_keys", list(decoded_full.keys()) == [1, 2, 3, 4, 5, 7, 8, 9])
check("pub_announce_id/sig_matches_preimage_vector", decoded_full[9][1].hex() == v_pre["expected"]["sig_hex"])
# and that stripping key 9 (rebuilding the map header for one fewer field, not just
# truncating bytes — the header's field-count nibble differs) reproduces the exact
# signing-preimage msg bytes
check("pub_announce_id/strip_sig_equals_preimage_msg", cbor_strip_key(full, 9) == msg)

# ── 7: supersede same-author / cross-author ───────────────────────────────────────────────
v_same = byname["pub_announce_supersede_same_author_valid"]
check("supersede_same_author/pub_match", v_same["input"]["predecessor_pub_hex"] == v_same["input"]["successor_pub_hex"])
succ_cbor = bytes.fromhex(v_same["input"]["successor_cbor_hex"])
decoded_succ, _ = cbor_decode_map(succ_cbor)
check("supersede_same_author/successor_pub_field", decoded_succ[3][1].hex() == v_same["input"]["successor_pub_hex"])
check("supersede_same_author/successor_supersedes_field", decoded_succ[6][1].hex() == v_same["input"]["successor_supersedes_hex"])
check("supersede_same_author/supersedes_points_at_predecessor", decoded_succ[6][1].hex() == v_same["input"]["predecessor_announce_id_hex"])
# verify successor sig independently
succ_sig = decoded_succ[9][1]
succ_signer = decoded_succ[8][1]
succ_body = cbor_strip_key(succ_cbor, 9)
try:
    ed25519.Ed25519PublicKey.from_public_bytes(succ_signer).verify(succ_sig, domain + succ_body)
    ok = True
except Exception:
    ok = False
check("supersede_same_author/successor_sig_verifies", ok)

v_cross = byname["pub_announce_supersede_cross_author_invalid"]
check("supersede_cross_author/pub_differs", v_cross["input"]["predecessor_pub_hex"] != v_cross["input"]["successor_pub_hex"])

# ── 8: pub_feed_entry_chain ────────────────────────────────────────────────────────────────
v = byname["pub_feed_entry_chain"]
entries = [bytes.fromhex(x) for x in v["input"]["entries_cbor_hex"]]
computed_ids = [ca(e).hex() for e in entries]
check("pub_feed_entry_chain/ids", computed_ids == v["expected"]["entry_ids_hex"])
d0, _ = cbor_decode_map(entries[0])
d1, _ = cbor_decode_map(entries[1])
d2, _ = cbor_decode_map(entries[2])
check("pub_feed_entry_chain/genesis_no_prev", 3 not in d0 and d0[1] == ("uint", 0))
check("pub_feed_entry_chain/entry1_prev_is_entry0", 3 in d1 and d1[3][1].hex() == computed_ids[0])
check("pub_feed_entry_chain/entry2_prev_is_entry1", 3 in d2 and d2[3][1].hex() == computed_ids[1])

# ── 9: pub_feed_head_signing_preimage ─────────────────────────────────────────────────────
v = byname["pub_feed_head_signing_preimage"]
domain_f = bytes.fromhex(v["input"]["domain_hex"])
check("pub_feed_head_signing_preimage/domain", domain_f == b"DMTAP-PUB-v0/feed\x00")
seed_f = bytes.fromhex(v["input"]["seed_hex"])
msg_f = bytes.fromhex(v["input"]["msg_hex"])
sk_f = ed25519.Ed25519PrivateKey.from_private_bytes(seed_f)
pk_f = sk_f.public_key().public_bytes(encoding=serialization.Encoding.Raw, format=serialization.PublicFormat.Raw)
sig_f = sk_f.sign(domain_f + msg_f)
check("pub_feed_head_signing_preimage/pubkey", pk_f.hex() == v["expected"]["pubkey_hex"])
check("pub_feed_head_signing_preimage/sig", sig_f.hex() == v["expected"]["sig_hex"])
dh, _ = cbor_decode_map(msg_f)
check("pub_feed_head_signing_preimage/tip_is_entry1", dh[5][1].hex() == computed_ids[1])
check("pub_feed_head_signing_preimage/seq_is_1", dh[4] == ("uint", 1))

# ── 10/11/12: anti-rollback vectors — structural cross-checks ────────────────────────────
v = byname["pub_feed_rollback_strict_less_than"]
check("rollback_strict/seq_less_than", v["input"]["presented_seq"] < v["input"]["last_accepted_seq"])

v = byname["pub_feed_equal_seq_identical_tip_idempotent"]
check("idempotent/seq_equal", v["input"]["presented_seq"] == v["input"]["last_accepted_seq"])
check("idempotent/tip_identical", v["input"]["presented_tip_hex"] == v["input"]["last_accepted_tip_hex"])
check("idempotent/tip_is_entry1", v["input"]["presented_tip_hex"] == computed_ids[1])

v = byname["pub_feed_equal_seq_different_tip_fork"]
check("fork/seq_equal", v["input"]["presented_seq"] == v["input"]["last_accepted_seq"])
check("fork/tip_differs", v["input"]["presented_tip_hex"] != v["input"]["last_accepted_tip_hex"])
alt_cbor = bytes.fromhex(v["input"]["presented_tip_cbor_hex"])
check("fork/alt_tip_id_matches", ca(alt_cbor).hex() == v["input"]["presented_tip_hex"])
d_alt, _ = cbor_decode_map(alt_cbor)
check("fork/alt_same_prev_as_entry1", d_alt[3][1].hex() == d1[3][1].hex())
check("fork/alt_same_seq_as_entry1", d_alt[1] == d1[1])
check("fork/alt_different_announce_than_entry1", d_alt[2][1] != d1[2][1])

# ── 13/14: genesis malformed vectors ──────────────────────────────────────────────────────
v = byname["pub_feed_genesis_carries_prev_malformed"]
cbor = bytes.fromhex(v["input"]["cbor_hex"])
dec, _ = cbor_decode_map(cbor)
check("genesis_carries_prev/seq0", dec[1] == ("uint", 0))
check("genesis_carries_prev/has_prev", 3 in dec)

v = byname["pub_feed_nongenesis_missing_prev_malformed"]
cbor = bytes.fromhex(v["input"]["cbor_hex"])
dec, _ = cbor_decode_map(cbor)
check("nongenesis_missing_prev/seq1", dec[1] == ("uint", 1))
check("nongenesis_missing_prev/no_prev", 3 not in dec)

print()
if fails:
    print(f"FAILURES: {fails}")
    sys.exit(1)
print(f"All checks passed. {len(byname)} vectors independently re-derived/cross-checked.")
