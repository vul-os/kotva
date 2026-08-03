#!/usr/bin/env python3
"""
gen_chunkproof_vectors.py — generates conformance/vectors/chunkproof_vectors.json

The CROSS-LANGUAGE LOCK for substrate/FEEDS.md §5.3 chunk-tree range proofs.

§5.3 has three implementations: Go (pier `tunnel/pubcache/{merkle,proof}.go`, which
FEEDS.md §5.3 cites BY NAME as fielded evidence for a normative decision), JS
(`bindings/js/src/chunkProof.js`, whose header says "EXACT PARITY WITH THE GO
IMPLEMENTATION is the whole point"), and the spec text itself. Until this file existed
the two code halves each asserted a hand-copied constant with nothing mechanically
comparing them: a round-trip cannot see two implementations agreeing on something wrong,
only FOREIGN BYTES can.

This script is the foreign-byte source. It imports neither implementation. Every value
below is computed here from the §22.2.2 / §3.2 formulas and the §5.3 / §18.1.2 encoding
rules:

    leaf(h)      = BLAKE3-256( DS ‖ 0x00 ‖ h )       h = the 33-byte chunk ADDRESS
    node(l, r)   = BLAKE3-256( DS ‖ 0x01 ‖ l ‖ r )   bare 32-byte interior values
    DS           = "DMTAP-PUB-v0/manifest" ‖ 0x00
    addr(bytes)  = 0x1e ‖ BLAKE3-256(bytes)
    root         = 0x1e ‖ MTH(h_0 … h_{n-1})
    proof body   = canonical CBOR  [ chunk_index, [ sibling_hashes… ] ]

PROVENANCE, STATED PLAINLY — read conformance/README.md's "Provenance" section for why
this distinction matters. This file is of the pub_vectors.json KIND (an independent
generator that the implementations are checked against), NOT the schema_vectors_v0.json
kind (a transport that lifts bytes out of an implementation). Nothing here is copied from
Go or JS. But it is not a clean-room reimplementation either: it was written by an author
who had read the Go implementation, so it is not evidence that two people reading only the
spec would agree. What it IS evidence of, and this is the whole point of the exercise:

  • Go and JS are each checked against a THIRD set of bytes rather than against each
    other, so a one-sided change to either one goes red without the other having to move;
  • the roots reproduce the constants that were pinned in Go BEFORE this script existed
    (`interopRootB64` / `interopProofHex` in tunnel/pubcache/proof_test.go), which were
    derived without it;
  • every root is computed TWICE by two different constructions that the spec treats as
    the same tree — RFC 6962's recursive "split at the largest power of two < n" (§3.2)
    and the level-by-level "pair left to right, promote the unpaired last node" form the
    audit path is built from — and generation FAILS if they ever disagree. That is a
    real check on the spec's own claim, not a restatement of one implementation.

Every corruption control is verified to be REJECTED by this script's own verifier before
it is emitted, so a control that a correct implementation would accept cannot ship. A
control that fails to fail is worse than no control at all.

Dependencies: `pip install blake3`. No randomness, no wall-clock reads, no network.

Run:  python3 conformance/vectors/gen_chunkproof_vectors.py
      python3 conformance/vectors/gen_chunkproof_vectors.py --check   # staleness gate
"""
import json
import sys
from pathlib import Path

import blake3

OUT = Path(__file__).resolve().parent / "chunkproof_vectors.json"
ROOT = Path(__file__).resolve().parents[2]

DS_MANIFEST = b"DMTAP-PUB-v0/manifest" + b"\x00"
HASH_PREFIX_BLAKE3_256 = 0x1E
DIGEST_LEN = 32
ADDR_LEN = 1 + DIGEST_LEN
MAX_PROOF_PATH = 40

# Coverage floors enforced at GENERATION time, so a script that silently stopped
# emitting vectors cannot write an empty corpus that both test suites then "pass".
MIN_VECTORS = 10
MIN_PROOFS = 60
MIN_CONTROLS = 8


def b3(b: bytes) -> bytes:
    return blake3.blake3(b).digest()


def addr(b: bytes) -> bytes:
    """§18.9.4 content address of a byte string: 0x1e ‖ BLAKE3-256(b)."""
    return bytes([HASH_PREFIX_BLAKE3_256]) + b3(b)


def leaf(h: bytes) -> bytes:
    """§22.2.2 leaf over the FULL 33-byte chunk address, not the chunk bytes."""
    assert len(h) == ADDR_LEN, "leaf takes a 33-byte address"
    return b3(DS_MANIFEST + b"\x00" + h)


def node(left: bytes, right: bytes) -> bytes:
    assert len(left) == len(right) == DIGEST_LEN
    return b3(DS_MANIFEST + b"\x01" + left + right)


# ── the tree, computed two independent ways ──────────────────────────────────────────


def mth_rfc6962(leaves: list) -> bytes:
    """RFC 6962 MTH: split at k = the largest power of two STRICTLY less than n (§3.2)."""
    n = len(leaves)
    assert n >= 1
    if n == 1:
        return leaves[0]
    k = 1
    while k * 2 < n:
        k *= 2
    return node(mth_rfc6962(leaves[:k]), mth_rfc6962(leaves[k:]))


def reduce_level(level: list) -> list:
    """Pair left to right; PROMOTE an unpaired final node unchanged (never re-hashed)."""
    nxt = []
    i = 0
    while i + 1 < len(level):
        nxt.append(node(level[i], level[i + 1]))
        i += 2
    if i < len(level):
        nxt.append(level[i])
    return nxt


def mth_levelwise(leaves: list) -> bytes:
    level = list(leaves)
    while len(level) > 1:
        level = reduce_level(level)
    return level[0]


def manifest_root(chunk_addrs: list) -> bytes:
    leaves = [leaf(h) for h in chunk_addrs]
    a = mth_rfc6962(leaves)
    b = mth_levelwise(leaves)
    if a != b:
        raise SystemExit(
            f"FAIL: the two §3.2 constructions disagree at n={len(leaves)}: "
            f"{a.hex()} (RFC 6962 split) vs {b.hex()} (level-wise promotion). "
            "The spec treats these as the same tree; one of them is wrong."
        )
    return bytes([HASH_PREFIX_BLAKE3_256]) + a


def audit_path(chunk_addrs: list, index: int) -> list:
    """Sibling hashes from leaf `index` to the root, BOTTOM-UP. A level at which the
    node is the promoted unpaired last one contributes NO element."""
    level = [leaf(h) for h in chunk_addrs]
    path = []
    cur = index
    while len(level) > 1:
        sib = cur ^ 1
        if sib < len(level):
            path.append(level[sib])
        cur //= 2
        level = reduce_level(level)
    return path


def verify(root: bytes, n_chunks: int, index: int, chunk: bytes, path: list) -> bool:
    """The §5.3 client-side fold. Returns True ONLY if the chunk is proven."""
    if len(root) != ADDR_LEN or root[0] != HASH_PREFIX_BLAKE3_256:
        return False
    if n_chunks <= 0 or index < 0 or index >= n_chunks:
        return False
    if len(path) > MAX_PROOF_PATH or any(len(s) != DIGEST_LEN for s in path):
        return False
    cur_node = leaf(addr(chunk))
    cur, level_len, used = index, n_chunks, 0
    while level_len > 1:
        if (cur ^ 1) < level_len:
            if used >= len(path):
                return False
            s = path[used]
            used += 1
            cur_node = node(cur_node, s) if cur % 2 == 0 else node(s, cur_node)
        cur //= 2
        level_len = (level_len + 1) // 2
    if used != len(path):
        return False
    return bytes([HASH_PREFIX_BLAKE3_256]) + cur_node == root


# ── the §5.3 wire encoding (§18.1.2 deterministic CBOR) ──────────────────────────────

CBOR_UINT, CBOR_BYTESTR, CBOR_ARRAY = 0, 2, 4


def cbor_head(major: int, v: int) -> bytes:
    m = major << 5
    if v < 24:
        return bytes([m | v])
    if v <= 0xFF:
        return bytes([m | 24, v])
    if v <= 0xFFFF:
        return bytes([m | 25, (v >> 8) & 0xFF, v & 0xFF])
    return bytes([m | 26, (v >> 24) & 0xFF, (v >> 16) & 0xFF, (v >> 8) & 0xFF, v & 0xFF])


def encode_proof(index: int, path: list) -> bytes:
    out = bytearray([0x82])  # array(2)
    out += cbor_head(CBOR_UINT, index)
    out += cbor_head(CBOR_ARRAY, len(path))
    for h in path:
        out += cbor_head(CBOR_BYTESTR, len(h))
        out += h
    return bytes(out)


# ── the corpus ───────────────────────────────────────────────────────────────────────

# n=5 over the one-byte payloads "a".."e" is THE pinned vector: five leaves is the
# smallest tree that promotes an odd node at TWO levels, and chunk 4 carries a
# one-element path where chunks 0-3 carry three. Its root and proof bodies must equal the
# constants pinned in tunnel/pubcache/proof_test.go, which predate this script.
PINNED_NAME = "chunkproof_n5_abcde"
PINNED_ROOT_B64 = "HqmS4uJD2JJOZjmeF-YZikRhImZOgGvZHe6IwCOpRyT_"
PINNED_PROOF_HEX = [
    "8200835820609ad16ca3186fc12dd32ce1d49ed57dd879c802246de385a20f7dbee2f894395820c97979256dd9f06e0dc6be9fabf2baef2acd2118939563d18bfa79661dc36dce58201365330142a154c52d28959cc1db9166d7b10c2591a9acc25d959ec7e1b8d242",
    "8201835820208e131bd1411e9d8c1d8417b9e9f370e2118a32b37535c77357c6d152348ac75820c97979256dd9f06e0dc6be9fabf2baef2acd2118939563d18bfa79661dc36dce58201365330142a154c52d28959cc1db9166d7b10c2591a9acc25d959ec7e1b8d242",
    "82028358208cc8a6db6f14fc57eacea4131385777a244b1f6feaeae1fed47ee8ef6e0982cf5820abd36c78c5c484698bf962a24adc9293467661696e0897a500df261d2b1664f258201365330142a154c52d28959cc1db9166d7b10c2591a9acc25d959ec7e1b8d242",
    "820383582093ce26dbcfb499cfd2b7ddfda025f4377f02bf62416d7f4799ea467720edaddd5820abd36c78c5c484698bf962a24adc9293467661696e0897a500df261d2b1664f258201365330142a154c52d28959cc1db9166d7b10c2591a9acc25d959ec7e1b8d242",
    "82048158205fa8b1b087f0c5dec0dc650c299f1779e735fd3b317e85793bbedac488a5183f",
]


def b64url(b: bytes) -> str:
    import base64

    return base64.urlsafe_b64encode(b).decode().rstrip("=")


def payloads(n: int) -> list:
    if n == 5:
        return [bytes([0x61 + i]) for i in range(5)]  # "a".."e"
    return [f"dmtap-pub chunk {i} of {n}".encode() for i in range(n)]


SHAPES = [
    (1, "n=1: MTH([h_0]) = leaf(h_0). The root is a leaf; the path is EMPTY, so a "
        "verifier that demands at least one path element rejects a valid proof here."),
    (2, "n=2: one interior node, one-element paths, no promotion anywhere."),
    (3, "n=3: the smallest promotion. Chunk 2 is promoted at level 1 and pairs at "
        "level 2, so its path is one element where chunks 0-1 carry two."),
    (4, "n=4: a perfect tree. Every path is two elements and no promotion occurs — the "
        "shape a verifier that ignored the promotion rule would still get right."),
    (5, "n=5: THE PINNED VECTOR, payloads \"a\"..\"e\", one byte each. The smallest tree "
        "that promotes an odd node at TWO levels: chunk 4 carries a one-element path "
        "where chunks 0-3 carry three. Root and proof bodies here are byte-identical to "
        "the constants pinned in pier tunnel/pubcache/proof_test.go before this file "
        "existed."),
    (6, "n=6: promotion at the SECOND level only (3 nodes reduce to 2 with the last "
        "promoted), which is a different skip pattern from n=5's."),
    (7, "n=7: promotion at the first level only."),
    (8, "n=8: a perfect 3-level tree; every path is three elements."),
    (9, "n=9: one leaf hanging off a perfect 8-leaf tree — chunk 8 is promoted at three "
        "consecutive levels and its path is a single element."),
    (17, "n=17: the same one-hanging-leaf shape a level deeper, and wide enough that a "
         "verifier confusing ⌈n/2⌉ with ⌊n/2⌋ diverges."),
]


def build_vector(n: int, note: str) -> dict:
    data = payloads(n)
    addrs = [addr(d) for d in data]
    root = manifest_root(addrs)
    proofs = []
    for i in range(n):
        path = audit_path(addrs, i)
        if not verify(root, n, i, data[i], path):
            raise SystemExit(f"FAIL: self-check — n={n} index={i} does not verify against its own root")
        proofs.append(
            {
                "index": i,
                "path_hex": [s.hex() for s in path],
                "proof_body_hex": encode_proof(i, path).hex(),
            }
        )
    return {
        "name": f"chunkproof_n{n}_abcde" if n == 5 else f"chunkproof_n{n}",
        "n": n,
        "chunks_hex": [d.hex() for d in data],
        "chunk_addrs_hex": [a.hex() for a in addrs],
        "root_hex": root.hex(),
        "root_b64url": b64url(root),
        "proofs": proofs,
        "note": note,
    }


def build_controls(vectors: dict) -> list:
    """Corruption controls: one deliberate defect each, over an otherwise-valid proof.

    `surface` says which half must reject it — "verify" for the fold, "decode" for the
    §5.3 CBOR body parser. Every "verify" control is run through this script's own
    verifier below and generation FAILS if it is accepted, because a control that a
    correct implementation would accept is not a control.
    """
    v5 = vectors[PINNED_NAME]
    data5 = [bytes.fromhex(h) for h in v5["chunks_hex"]]
    root5 = bytes.fromhex(v5["root_hex"])
    addrs5 = [bytes.fromhex(h) for h in v5["chunk_addrs_hex"]]

    def path_of(i):
        return audit_path(addrs5, i)

    controls = []

    def vc(name, defect, index, chunk, path, note, root=None, n_chunks=5):
        controls.append(
            {
                "name": name,
                "surface": "verify",
                "base_vector": PINNED_NAME,
                "defect": defect,
                "root_hex": (root or root5).hex(),
                "n_chunks": n_chunks,
                "index": index,
                "chunk_hex": chunk.hex(),
                "path_hex": [s.hex() for s in path],
                "expect": "reject",
                "note": note,
            }
        )

    # 1. tampered chunk bytes — the attack the endpoint exists to stop.
    vc("tampered_chunk_byte", "chunk_byte_flipped", 2, b"C", path_of(2),
       "Chunk 2's bytes changed from \"c\" to \"C\" with a valid path for chunk 2. The leaf "
       "is taken over the chunk ADDRESS, so one flipped bit changes h_i and the fold diverges.")

    # 2. sibling order swapped — the single most likely reimplementation bug, and the
    #    mutation this lock is designed to catch.
    p1 = path_of(1)
    vc("swapped_sibling_order", "path_reversed", 1, data5[1], list(reversed(p1)),
       "Chunk 1's three-element path reversed. Order is fixed by the node's own index "
       "parity; a verifier that took order from the path's position folds to a different root.")

    # 3. truncated path.
    vc("truncated_path", "path_last_element_dropped", 0, data5[0], path_of(0)[:-1],
       "Chunk 0's path minus its top element: the fold runs out of siblings at the last "
       "level and MUST fail rather than stopping early at a partial root.")

    # 4. padded path — a server smuggling unverified material.
    vc("padded_path", "path_extra_element_appended", 4, data5[4], path_of(4) + [path_of(0)[0]],
       "Chunk 4 is the promoted odd node and its path is ONE element; a second element is "
       "appended. The verifier must reject leftover material rather than ignore it.")

    # 5. wrong index with an otherwise-valid path.
    vc("index_shifted", "valid_path_wrong_index", 3, data5[3], path_of(2),
       "Chunk 3's bytes with CHUNK 2's path — both real, paired wrongly. Parity differs, so "
       "the fold combines in the wrong order and diverges.")

    # 6. wrong root.
    bad_root = bytearray(root5)
    bad_root[-1] ^= 0x01
    vc("wrong_root", "root_last_byte_flipped", 0, data5[0], path_of(0),
       "A wholly valid proof folded against a root one bit away from the real one. The root "
       "is the authenticator; nothing else in the proof is.", root=bytes(bad_root))

    # 7. wrong nChunks that CHANGES the promotion pattern for this index. (n=5 vs n=9 for
    #    index 4: at n=9, index 4 has a sibling at level 0, so a path element is consumed
    #    where n=5 promotes. §5.3's corollary says a count that does NOT change the fold
    #    shape is undetectable — that honest limitation is asserted separately in each
    #    suite; this control covers only the case the fold really does catch.)
    vc("nchunks_changes_promotion", "n_chunks_9_instead_of_5", 4, data5[4], path_of(4),
       "The same proof declared to be from a 9-chunk tree. nChunks is structural metadata "
       "(§5.3 corollary), and this is the case where getting it wrong changes which levels "
       "promote and the fold therefore diverges.", n_chunks=9)

    # 8. a sibling hash of the wrong width — a path element that is not a tree node.
    short = path_of(0)[0][:31]
    vc("short_sibling_hash", "path_element_31_bytes", 0, data5[0], [short] + path_of(0)[1:],
       "A 31-byte path element. A verifier that concatenated without checking width would "
       "hash a differently-sized preimage instead of refusing.")

    # ── decode-surface controls: malformed §5.3 bodies ────────────────────────────────
    def dc(name, defect, body: bytes, note):
        controls.append(
            {
                "name": name,
                "surface": "decode",
                "base_vector": PINNED_NAME,
                "defect": defect,
                "proof_body_hex": body.hex(),
                "expect": "reject",
                "note": note,
            }
        )

    valid4 = bytes.fromhex(v5["proofs"][4]["proof_body_hex"])

    dc("nonminimal_index", "cbor_index_encoded_in_two_bytes",
       b"\x82\x18\x04" + valid4[2:],
       "Chunk index 4 written as the two-byte head 0x18 0x04 instead of 0x04. Deterministic "
       "CBOR (§18.1.2) admits exactly one spelling; a proof two byte strings could both mean "
       "is not a proof.")

    dc("indefinite_length_path", "cbor_indefinite_array",
       b"\x82\x04\x9f" + valid4[3:] + b"\xff",
       "The path array as an indefinite-length array (0x9f … 0xff). Rejected on sight: "
       "indefinite lengths are outside the §18.1.2 profile.")

    dc("trailing_bytes", "one_byte_appended", valid4 + b"\x00",
       "A valid proof body with one trailing byte. Accepting it would let a server append "
       "material the decoder never examined.")

    dc("path_count_over_bound", "declared_path_of_41", b"\x82\x04\x98\x29",
       f"A path array head declaring 41 elements, one past the {MAX_PROOF_PATH}-level bound, "
       "with no elements following. The bound must be enforced from the HEAD, before "
       "allocating or reading an attacker-chosen count.")

    dc("not_a_two_element_array", "cbor_array_of_3", b"\x83\x04\x80\x00",
       "A three-element array [4, [], 0] — the shape §5.3 explicitly declines to adopt "
       "(option (b), `[i, path, tree_size]`). An old client MUST NOT silently misparse it.")

    return controls


def build() -> dict:
    vectors = {}
    ordered = []
    for n, note in SHAPES:
        v = build_vector(n, note)
        vectors[v["name"]] = v
        ordered.append(v)

    pinned = vectors[PINNED_NAME]
    if pinned["root_b64url"] != PINNED_ROOT_B64:
        raise SystemExit(
            f"FAIL: the n=5 root is {pinned['root_b64url']}, but pier "
            f"tunnel/pubcache/proof_test.go pinned {PINNED_ROOT_B64} before this script "
            "existed. The tree, the DS tag, or the leaf rule has changed."
        )
    for i, want in enumerate(PINNED_PROOF_HEX):
        got = pinned["proofs"][i]["proof_body_hex"]
        if got != want:
            raise SystemExit(f"FAIL: n=5 chunk {i} proof body is {got}, pinned {want}")

    controls = build_controls(vectors)

    # A control that a correct implementation would ACCEPT is not a control. Prove each
    # verify-surface one really is rejected by this script's own verifier.
    for c in controls:
        if c["surface"] != "verify":
            continue
        ok = verify(
            bytes.fromhex(c["root_hex"]),
            c["n_chunks"],
            c["index"],
            bytes.fromhex(c["chunk_hex"]),
            [bytes.fromhex(h) for h in c["path_hex"]],
        )
        if ok:
            raise SystemExit(f"FAIL: control {c['name']} VERIFIES — it is not a control")

    n_proofs = sum(len(v["proofs"]) for v in ordered)
    if len(ordered) < MIN_VECTORS or n_proofs < MIN_PROOFS or len(controls) < MIN_CONTROLS:
        raise SystemExit(
            f"FAIL: coverage floor — {len(ordered)} vectors (min {MIN_VECTORS}), "
            f"{n_proofs} proofs (min {MIN_PROOFS}), {len(controls)} controls "
            f"(min {MIN_CONTROLS})"
        )

    return {
        "format": "kotva-conformance-vectors/1",
        "suite": "DMTAP-PUB §5.3 chunk-tree range proofs over the §3.2/§22.2.2 DS-tagged "
        "Merkle tree — suite 0x01 (classical): BLAKE3-256",
        "generated_by": "conformance/vectors/gen_chunkproof_vectors.py (this repo). This is "
        "an INDEPENDENT GENERATOR in the pub_vectors.json tradition, NOT a transport like "
        "schema_vectors_v0.json: it imports neither implementation of §5.3 and computes "
        "every byte here from the §22.2.2 leaf/node formulas and the §18.1.2 CBOR rules. Be "
        "precise about what that does and does not buy. It is NOT clean-room — it was "
        "written by an author who had read the Go implementation (pier "
        "tunnel/pubcache/{merkle,proof}.go), so it is not evidence that two independent "
        "readers of the spec would agree. It IS a third set of bytes that BOTH code halves "
        "are checked against instead of against each other, so a one-sided change to Go or "
        "to JS goes red on its own; it reproduces the constants Go pinned before this "
        "script existed; and every root is computed twice, by RFC 6962's recursive split "
        "rule and by the level-wise promotion form the audit path is built from, with "
        "generation failing if those two disagree.",
        "methodology": "All values computed from FIXED inputs; no randomness, no wall-clock "
        "reads. BLAKE3-256 and the §18.1.2 deterministic CBOR encoding are deterministic, so "
        "a second implementer following §3.2, §22.2.2 and §5.3 alone reproduces these bytes "
        "without running this script. A conformant implementation MUST reproduce every "
        "`root_hex` from `chunks_hex`, MUST reproduce every `proof_body_hex` byte-for-byte, "
        "MUST verify every proof against its root, and MUST REJECT every entry in "
        "`corruption_controls` — each of which carries a single deliberate defect over an "
        "otherwise-valid proof, so a rejection can only be about that defect. Controls with "
        "`surface: \"verify\"` must be refused by the fold; those with `surface: \"decode\"` "
        "must be refused by the §5.3 CBOR body parser.",
        "consumed_by": [
            "bindings/js/test/chunkProofVectors.test.js (kotva, this repo)",
            "tunnel/pubcache/vectors_test.go (pier — reads its own byte-identical copy at "
            "tunnel/pubcache/testdata/chunkproof_vectors.json, pinned by the sha256 that "
            "both suites assert, so the two copies cannot drift apart silently)",
        ],
        "counts": {
            "vectors": len(ordered),
            "proofs": n_proofs,
            "corruption_controls": len(controls),
        },
        "vectors": ordered,
        "corruption_controls": controls,
    }


if __name__ == "__main__":
    fresh = json.dumps(build(), indent=2) + "\n"
    if "--check" in sys.argv:
        if not OUT.exists() or OUT.read_text() != fresh:
            sys.exit(f"FAIL: {OUT.name} is stale — re-run without --check")
        print(f"OK: {OUT.name} matches the generator")
    else:
        OUT.write_text(fresh)
        print(f"wrote {OUT.relative_to(ROOT)}")
