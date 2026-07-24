#!/usr/bin/env python3
# gen_wrap_vectors.py — regenerate WRAP v0.2.0 conformance vectors against the
# DMTAP substrate byte formats.
#
#   pip install blake3 pynacl            (cbor2 is NOT used here; the encoder is
#                                         hand-rolled so the bytes are portable and
#                                         so the independent verifier can use cbor2
#                                         as a genuinely separate second encoder.)
#
# Byte formats implemented, with their governing spec text:
#   * deterministic CBOR (RFC 8949 §4.2)         — dmtap/18-wire-format.md §18.1.1
#   * 0x1e ‖ BLAKE3-256 content address           — dmtap/18-wire-format.md §18.1.5,
#                                                    wrap/03-wire-format.md §4.3
#   * WRAP object field registry (keys 1-15)      — wrap/02-objects.md, §3.2
#   * object -> substrate-primitive mapping       — wrap/06-merge.md §7.2
#   * handoff commitment BLAKE3-256(code‖order)   — wrap/10-fulfilment.md §10.2
#
# What is DELIBERATELY NOT here (it is the SUBSTRATE's, per 14-conformance.md §15.2
# and MUST NOT be re-vectored by WRAP): the COSE_Sign1 envelope bytes, the DS-tag
# signature value, the HLC tie-break, and OR-Set/LWW convergence at the byte level.
# WRAP's DS-tag "WRAP-v0/object" and its authorship/admission/fold rules are WRAP's,
# and are what these vectors fix.

import json
from nacl.signing import SigningKey
from blake3 import blake3

# --------------------------------------------------------------------------- #
# Hand-rolled deterministic CBOR (RFC 8949 §4.2 core deterministic encoding).
# Only the value shapes WRAP objects actually use are implemented; floats,
# negatives-in-practice, tags, and indefinite lengths are intentionally absent.
# --------------------------------------------------------------------------- #

def _head(major, n):
    mt = major << 5
    if n < 24:
        return bytes([mt | n])
    if n < 0x100:
        return bytes([mt | 24, n])
    if n < 0x10000:
        return bytes([mt | 25]) + n.to_bytes(2, "big")
    if n < 0x100000000:
        return bytes([mt | 26]) + n.to_bytes(4, "big")
    return bytes([mt | 27]) + n.to_bytes(8, "big")


class Val:
    """A typed CBOR value carrying both its wire encoding and its {t,v} JSON form."""
    def __init__(self, t, v):
        self.t = t
        self.v = v

    # ---- wire encoding (deterministic CBOR) ----
    def encode(self):
        t = self.t
        if t == "uint":
            assert self.v >= 0
            return _head(0, self.v)
        if t == "int":
            if self.v >= 0:
                return _head(0, self.v)
            return _head(1, -1 - self.v)
        if t == "tstr":
            b = self.v.encode("utf-8")
            return _head(3, len(b)) + b
        if t == "bstr":
            b = bytes.fromhex(self.v)
            return _head(2, len(b)) + b
        if t == "bool":
            return bytes([0xf5 if self.v else 0xf4])
        if t == "array":
            out = _head(4, len(self.v))
            for e in self.v:
                out += e.encode()
            return out
        if t == "map":
            # v is dict[int, Val]; sort by ENCODED KEY BYTES ascending (§18.1.1 rule 2)
            pairs = [(_head(0, k), k, val) for k, val in self.v.items()]
            pairs.sort(key=lambda p: p[0])
            out = _head(5, len(pairs))
            for kbytes, _k, val in pairs:
                out += kbytes + val.encode()
            return out
        if t == "refmap":
            # v is dict[str, Val]; text keys, sorted by encoded key bytes ascending
            def kb(s):
                b = s.encode("utf-8")
                return _head(3, len(b)) + b
            pairs = [(kb(k), k, val) for k, val in self.v.items()]
            pairs.sort(key=lambda p: p[0])
            out = _head(5, len(pairs))
            for kbytes, _k, val in pairs:
                out += kbytes + val.encode()
            return out
        raise ValueError(f"unknown type {t}")

    # ---- {t,v} JSON form for the vector file ----
    def json(self):
        t = self.t
        if t in ("uint", "int", "tstr", "bstr", "bool"):
            return {"t": t, "v": self.v}
        if t == "array":
            return {"t": "array", "v": [e.json() for e in self.v]}
        if t == "map":
            return {"t": "map", "v": {str(k): val.json() for k, val in self.v.items()}}
        if t == "refmap":
            return {"t": "refmap", "v": {k: val.v for k, val in self.v.items()}}
        raise ValueError(t)


# constructors
def U(n): return Val("uint", n)
def I(n): return Val("int", n)
def T(s): return Val("tstr", s)
def B(h): return Val("bstr", h if isinstance(h, str) else h.hex())
def Bool(b): return Val("bool", b)
def A(xs): return Val("array", xs)
def M(d): return Val("map", d)
def RefM(d): return Val("refmap", d)


DS_TAG = b"WRAP-v0/object"


def content_id(canonical_bytes: bytes) -> bytes:
    # 0x1e ‖ BLAKE3-256(canonical_bytes)   (§18.1.5, §4.3)
    return b"\x1e" + blake3(canonical_bytes).digest()


def wrap_preimage(canonical_bytes: bytes) -> bytes:
    # §5.2: preimage = "WRAP-v0/object" ‖ 0x00 ‖ canonical_bytes
    # (recorded for reference; the COSE_Sign1 signature over it is the SUBSTRATE's
    #  vector, §15.2, and is NOT emitted as a checked value.)
    return DS_TAG + b"\x00" + canonical_bytes


# --------------------------------------------------------------------------- #
# Keys: fixed Ed25519 seeds = one byte repeated 32× (eyeball-identifiable).
# --------------------------------------------------------------------------- #
KEY_SEEDS = {
    "issuer":     0x11,
    "performer":  0x22,
    "imposter":   0x33,
    "pool":       0x44,
    "attestor":   0x55,
    "performer2": 0x66,
}
KEYS = {}
for name, byte in KEY_SEEDS.items():
    seed = bytes([byte]) * 32
    sk = SigningKey(seed)
    KEYS[name] = {"seed": seed.hex(), "pub": sk.verify_key.encode().hex()}

def pub(name): return KEYS[name]["pub"]


def hlc(wall, counter, author_name):
    # Hlc = {1: u64 wall, 2: u32 counter, 3: ik-pub author}  (SYNC §3, §4.1)
    return M({1: U(wall), 2: U(counter), 3: B(pub(author_name))})


# --------------------------------------------------------------------------- #
# Object builder.  canonical_bytes = det_cbor(object map with keys 1,2,4,5,6+),
# with key 3 (id) and the signature OMITTED (§4.3, §5.2).
# --------------------------------------------------------------------------- #
OBJECTS = {}

def build(name, kind_hex, author_name, ts_hlc, fields, extra_common=None):
    kind = int(kind_hex, 16)
    m = {1: U(0), 2: U(kind), 4: B(pub(author_name)), 5: ts_hlc}
    if extra_common:
        m.update(extra_common)
    for k, v in fields.items():
        m[k] = v
    obj_map = M(m)
    canonical = obj_map.encode()
    cid = content_id(canonical)
    kind_names = {1: "WorkOrder", 2: "Offer", 3: "Bid",
                  4: "Assignment", 5: "Progress", 6: "Attestation"}
    entry = {
        "kind": kind_names.get(kind, f"0x{kind:02x}"),
        "kind_hex": kind_hex,
        "author": author_name,
        "author_pub_hex": pub(author_name),
        "ts": {"wall": ts_hlc.v[1].v, "counter": ts_hlc.v[2].v, "author_hex": pub(author_name)},
        "fields": {str(k): v.json() for k, v in fields.items()},
        "canonical_bytes_hex": canonical.hex(),
        "id_hex": cid.hex(),
        "wrap_preimage_hex": wrap_preimage(canonical).hex(),
    }
    OBJECTS[name] = entry
    entry["_id_bytes"] = cid  # internal, stripped before serialization
    entry["_canonical"] = canonical
    return entry


# ns for a work order (§7.2). The spec writes ns = "wrap:" ‖ WorkOrder.id where id
# is 33 raw bytes; how those bytes sit inside a tstr namespace is not pinned by the
# spec, so we record the two parts structurally rather than assert a concatenation
# encoding we cannot verify against the substrate.  (See README "gaps".)
def ns_of(order_id_hex):
    return {"prefix": "wrap:", "order_id_hex": order_id_hex,
            "note": "ns = \"wrap:\" || WorkOrder.id (§7.2); raw-byte vs hex embedding in the tstr is unpinned"}


# --------------------------------------------------------------------------- #
# The object set.  Float-free by construction: Place.lat/lon are integer
# microdegrees (round(1e6 * decimal-degrees)) per MATCH §3.4, so §18.1.1's
# floats-forbidden rule holds with no exception. No open float question remains.
# --------------------------------------------------------------------------- #
WO_EXPIRES = 1784563200          # unix seconds
NOW_BEFORE = 1784500000          # < expires
NOW_AFTER  = 1784563201          # >= expires

wo1 = build("wo1", "0x01", "issuer", hlc(1784500000000, 0, "issuer"), {
    6: T("delivery/v0"),
    7: T("Deliver 2 bags of cement to 12 River Rd"),
    12: A([T("vehicle:bicycle")]),
    13: U(WO_EXPIRES),
    14: RefM({"order": T("BB-4417")}),
})
WO1_ID = wo1["id_hex"]

offer1 = build("offer1", "0x02", "issuer", hlc(1784500001000, 0, "issuer"), {
    6: B(WO1_ID),
    7: B(pub("pool")),
    8: U(1),               # open bid
})
OFFER1_ID = offer1["id_hex"]

bid1 = build("bid1", "0x03", "performer", hlc(1784500002000, 0, "performer"), {
    6: B(WO1_ID),
    7: B(OFFER1_ID),
    9: U(1784500600),      # eta
})
BID1_ID = bid1["id_hex"]

bid2 = build("bid2", "0x03", "performer2", hlc(1784500002000, 0, "performer2"), {
    6: B(WO1_ID),
    7: B(OFFER1_ID),
    9: U(1784500900),
})
BID2_ID = bid2["id_hex"]

asg1 = build("asg1", "0x04", "issuer", hlc(1784500003000, 0, "issuer"), {
    6: B(WO1_ID),
    7: B(pub("performer")),
})
ASG1_ID = asg1["id_hex"]

# same register, authored by a NON-issuer: numerically LATER HLC, but inadmissible.
asg_bad = build("asg_bad", "0x04", "imposter", hlc(1784500003500, 0, "imposter"), {
    6: B(WO1_ID),
    7: B(pub("imposter")),
})
ASG_BAD_ID = asg_bad["id_hex"]

prog_started = build("prog_started", "0x05", "performer", hlc(1784500004000, 0, "performer"), {
    6: B(WO1_ID),
    7: T("started"),
})
prog_completed = build("prog_completed", "0x05", "performer", hlc(1784500005000, 0, "performer"), {
    6: B(WO1_ID),
    7: T("completed"),
})
# completed reported with NO assignment reachable => unreachable-state discard (§6.3)
prog_unreach = build("prog_unreach", "0x05", "performer", hlc(1784500004500, 0, "performer"), {
    6: B(WO1_ID),
    7: T("completed"),
})
PROG_COMPLETED_ID = prog_completed["id_hex"]
PROG_UNREACH_ID = prog_unreach["id_hex"]

att1 = build("att1", "0x06", "attestor", hlc(1784500006000, 0, "attestor"), {
    6: B(WO1_ID),
    7: B(pub("performer")),
    8: U(0),               # completed
    9: U(5),               # rating
})
ATT1_ID = att1["id_hex"]

# forward-compat objects
# unknown field: a profile-range key (40, §4.5 keys >=32) an old core does not know
wo_unknown_field = build("wo_unknown_field", "0x01", "issuer", hlc(1784500000000, 0, "issuer"), {
    6: T("delivery/v0"),
    7: T("Deliver 2 bags of cement to 12 River Rd"),
    13: U(WO_EXPIRES),
    40: B("deadbeef"),     # unknown key -> MUST be preserved through re-encode (§4.4)
})
# unknown profile: a WorkOrder whose profile string is unrecognized
wo_unknown_profile = build("wo_unknown_profile", "0x01", "issuer", hlc(1784500000000, 0, "issuer"), {
    6: T("space-elevator/v9"),
    7: T("Assemble tether segment 12"),
    13: U(WO_EXPIRES),
})
# unknown kind: a profile-range object kind (0x42, §3.1 0x40-0x7f) -> ignored silently
unknown_kind = build("unknown_kind", "0x42", "issuer", hlc(1784500007000, 0, "issuer"), {
    6: B(WO1_ID),
    7: T("some profile-specific object"),
})


# --------------------------------------------------------------------------- #
# Handoff commitment (§10.2): commit = BLAKE3-256(code ‖ order_id), no prefix,
# no DS-tag (it is a commitment, not a content address). order_id = WorkOrder.id
# (the full 33-byte 0x1e-prefixed id). The code is ASCII digits.
# --------------------------------------------------------------------------- #
HANDOFF_CODE = "4417"
order_id_bytes = bytes.fromhex(WO1_ID)
commit = blake3(HANDOFF_CODE.encode("ascii") + order_id_bytes).digest()
COMMIT_HEX = commit.hex()
WRONG_CODE = "0000"
wrong_commit = blake3(WRONG_CODE.encode("ascii") + order_id_bytes).digest().hex()


# --------------------------------------------------------------------------- #
# Vectors.
# --------------------------------------------------------------------------- #
vectors = []

# ---- group: map  (object -> substrate primitive + address, §7.2 / §15.3) ----
vectors += [
    {"id": "map-workorder", "group": "map", "object": "wo1",
     "description": "WorkOrder maps to an immutable substrate content object; it is the namespace anchor, address = 0x1e‖BLAKE3-256(canonical_bytes).",
     "expect": {"primitive": "immutable-content-object", "ns": ns_of(WO1_ID),
                "target": None, "field": None, "address_hex": WO1_ID}},
    {"id": "map-offer", "group": "map", "object": "offer1",
     "description": "Offer maps to an OR-Set add at target 'offers' within the work order's namespace.",
     "expect": {"primitive": "or-set-add", "ns": ns_of(WO1_ID),
                "target": "offers", "field": None, "address_hex": OFFER1_ID}},
    {"id": "map-bid", "group": "map", "object": "bid1",
     "description": "Bid maps to an OR-Set add-wins observed-remove element at target 'bids'.",
     "expect": {"primitive": "or-set-add", "ns": ns_of(WO1_ID),
                "target": "bids", "field": None, "address_hex": BID1_ID}},
    {"id": "map-assignment", "group": "map", "object": "asg1",
     "description": "Assignment maps to an LWW register at target 'assignment', field '' (empty).",
     "expect": {"primitive": "lww-register", "ns": ns_of(WO1_ID),
                "target": "assignment", "field": "", "address_hex": ASG1_ID}},
    {"id": "map-progress", "group": "map", "object": "prog_started",
     "description": "Progress maps to an OR-Set add / append at target 'progress'.",
     "expect": {"primitive": "or-set-add", "ns": ns_of(WO1_ID),
                "target": "progress", "field": None, "address_hex": prog_started["id_hex"]}},
    {"id": "map-attestation", "group": "map", "object": "att1",
     "description": "Attestation maps to an author-feed entry (FEEDS §4) on the subject's feed; address = 0x1e‖BLAKE3-256(canonical_bytes).",
     "expect": {"primitive": "author-feed-entry", "ns": None,
                "target": "feed", "field": "subject", "subject_hex": pub("performer"),
                "address_hex": ATT1_ID}},
]

# ---- group: authorship  (§5.5 admission table) ----
vectors += [
    {"id": "authorship-assignment-nonissuer-inadmissible", "group": "authorship", "object": "asg_bad",
     "description": "An Assignment whose author is NOT the WorkOrder's author is INADMISSIBLE: it never enters the LWW register, even though its HLC (1784500003500) is numerically greater than the issuer's admissible assignment (1784500003000). Admission, not tie-break.",
     "context": {"workorder": "wo1", "workorder_author_hex": pub("issuer"),
                 "object_author_hex": pub("imposter")},
     "expect": {"admissible": False, "enters_register": False,
                "error_code": "0x0202", "error_name": "ERR_NOT_ISSUER"}},
    {"id": "authorship-assignment-issuer-admitted", "group": "authorship", "object": "asg1",
     "description": "An Assignment authored by the WorkOrder's author is admitted into the LWW register.",
     "context": {"workorder": "wo1", "workorder_author_hex": pub("issuer"),
                 "object_author_hex": pub("issuer")},
     "expect": {"admissible": True, "enters_register": True}},
    {"id": "authorship-bid-admitted", "group": "authorship", "object": "bid1",
     "description": "A Bid by any admitted principal is accepted into the 'bids' OR-Set (no issuer constraint).",
     "context": {"workorder": "wo1", "object_author_hex": pub("performer")},
     "expect": {"admissible": True, "enters_set": True}},
]

# ---- group: withdraw  (Bid OR-Set observed-remove, §7.2 / SYNC §4.3) ----
# add-tags are (author, hlc); a withdraw cites ONLY the add-tag of the bid it retracts.
bid1_addtag = {"element_id_hex": BID1_ID, "author_hex": pub("performer"),
               "hlc": {"wall": 1784500002000, "counter": 0, "author_hex": pub("performer")}}
bid2_addtag = {"element_id_hex": BID2_ID, "author_hex": pub("performer2"),
               "hlc": {"wall": 1784500002000, "counter": 0, "author_hex": pub("performer2")}}
vectors += [
    {"id": "withdraw-cancels-only-own-add", "group": "withdraw",
     "description": "performer withdraws bid1 by an OR-Set observed-remove citing bid1's own add-tag only. bid2 (a concurrent bid by performer2, identical wall, different author) is NOT cited and survives. Withdrawal is NOT a 'withdrawn' flag; it cancels only its own add-tag.",
     "input": {
        "adds": [bid1_addtag, bid2_addtag],
        "remove": {"target": "bids", "element_id_hex": BID1_ID,
                   "observed_add_tags": [{"author_hex": pub("performer"),
                                          "hlc": bid1_addtag["hlc"]}],
                   "remove_hlc": {"wall": 1784500002500, "counter": 0, "author_hex": pub("performer")}}
     },
     "expect": {"present_element_ids_hex": [BID2_ID], "cancelled_element_ids_hex": [BID1_ID]}},
    {"id": "withdraw-unseen-add-survives", "group": "withdraw",
     "description": "A remove may legitimately arrive before an add it never saw; an add-tag not cited by any remove is present (add-wins). Here only bid1 is added and no remove cites it, so bid1 is present.",
     "input": {"adds": [bid1_addtag], "remove": None},
     "expect": {"present_element_ids_hex": [BID1_ID], "cancelled_element_ids_hex": []}},
]

# ---- group: fold  (lifecycle state from op set, §6.3) ----
def fold_vec(vid, desc, objs, now, state, discarded=None):
    v = {"id": vid, "group": "fold", "description": desc,
         "input": {"object_set": objs, "now_unix_s": now},
         "expect": {"state": state}}
    if discarded is not None:
        v["expect"]["discarded_object_ids_hex"] = discarded
    return v

vectors += [
    fold_vec("fold-issued", "Only a WorkOrder, before expiry, no Offer: state 'issued'.",
             ["wo1"], NOW_BEFORE, "issued"),
    fold_vec("fold-offered", "WorkOrder + Offer, before expiry, no Assignment: state 'offered'.",
             ["wo1", "offer1"], NOW_BEFORE, "offered"),
    fold_vec("fold-assigned", "WorkOrder + Offer + valid Assignment, no Progress: state 'assigned'.",
             ["wo1", "offer1", "asg1"], NOW_BEFORE, "assigned"),
    fold_vec("fold-started", "... + a 'started' Progress: state 'started'.",
             ["wo1", "offer1", "asg1", "prog_started"], NOW_BEFORE, "started"),
    fold_vec("fold-completed", "... + a terminal 'completed' Progress (highest ts): state 'completed'.",
             ["wo1", "offer1", "asg1", "prog_started", "prog_completed"], NOW_BEFORE, "completed"),
    fold_vec("fold-unreachable-discard",
             "A 'completed' Progress on a work order with NO Assignment is unreachable from the implied state and MUST be discarded (§6.3); the fold ignores it and the state is 'offered' (Offer present, no Assignment).",
             ["wo1", "offer1", "prog_unreach"], NOW_BEFORE, "offered",
             discarded=[PROG_UNREACH_ID]),
]

# ---- group: expiry  (computed, no message, §6.2) ----
vectors += [
    {"id": "expiry-computed-no-message", "group": "expiry",
     "description": "A work order whose 'expires' has passed with no valid Assignment is 'expired' everywhere with no message exchanged. Computed purely from wo1.expires and now.",
     "input": {"object_set": ["wo1", "offer1"], "expires_unix_s": WO_EXPIRES, "now_unix_s": NOW_AFTER},
     "expect": {"state": "expired", "message_required": False}},
    {"id": "expiry-assignment-beats-expiry", "group": "expiry",
     "description": "expiry is checked AFTER assignment (§6.3 step 2 before step 3): a work order past 'expires' but carrying a valid Assignment is 'assigned', not 'expired'.",
     "input": {"object_set": ["wo1", "offer1", "asg1"], "expires_unix_s": WO_EXPIRES, "now_unix_s": NOW_AFTER},
     "expect": {"state": "assigned", "message_required": False}},
]

# ---- group: retain  (attestations survive compaction, §7.3) ----
vectors += [
    {"id": "retain-attestation-survives-compaction", "group": "retain",
     "description": "Compacting a terminal work order MAY discard superseded Offer/Bid/Progress ops, but MUST retain the WorkOrder, the winning Assignment register value, and EVERY Attestation feed entry. An Attestation is never pruned on the basis of age.",
     "input": {"terminal_object_set": ["wo1", "offer1", "bid1", "asg1", "prog_completed", "att1"],
               "terminal_state": "completed"},
     "expect": {"must_retain_ids_hex": [WO1_ID, ASG1_ID, ATT1_ID],
                "may_discard_ids_hex": [OFFER1_ID, BID1_ID, PROG_COMPLETED_ID]}},
]

# ---- group: forward  (unknown kind / field / profile, §4.4) ----
vectors += [
    {"id": "forward-unknown-kind-ignored", "group": "forward", "object": "unknown_kind",
     "description": "An object of an unknown kind (0x42, profile range) MUST be ignored silently — not acknowledged, not rejected, not errored (§4.4, §13.3).",
     "expect": {"action": "ignore-silently", "is_error": False}},
    {"id": "forward-unknown-field-preserved", "group": "forward", "object": "wo_unknown_field",
     "description": "An object carrying an unknown key (40) MUST have that key preserved through a re-encode: dropping it would invalidate the signature downstream. canonical_bytes_hex includes key 40 in its sorted position (after key 13); a conformant re-encode is byte-identical to canonical_bytes_hex.",
     "expect": {"action": "preserve-and-ignore", "is_error": False,
                "reencode_must_equal_hex": wo_unknown_field["canonical_bytes_hex"]}},
    {"id": "forward-unknown-profile-stored", "group": "forward", "object": "wo_unknown_profile",
     "description": "An object with an unrecognized profile ('space-elevator/v9') MUST be stored, relayed, and merged — just not rendered (§12.1, §13.3). Not an error.",
     "expect": {"action": "store-relay-merge", "is_error": False}},
]

# ---- group: proof  (handoff commitment, §10) ----
vectors += [
    {"id": "proof-handoff-commit-verifies", "group": "proof",
     "description": "Handoff commitment (§10.2): commit = BLAKE3-256(code ‖ order_id), no 0x1e prefix and no DS-tag (it is a commitment, not a content address). order_id is the WorkOrder's full 33-byte id. The completing Attestation's proof.code recomputes to the published commit -> verified.",
     "input": {"code": HANDOFF_CODE, "code_ascii_hex": HANDOFF_CODE.encode("ascii").hex(),
               "order_id_hex": WO1_ID, "presented_code": HANDOFF_CODE},
     "expect": {"commit_hex": COMMIT_HEX, "verified": True}},
    {"id": "proof-handoff-wrong-code-fails", "group": "proof",
     "description": "A presented code that differs from the one behind the commitment recomputes to a different digest and MUST fail verification.",
     "input": {"code": HANDOFF_CODE, "order_id_hex": WO1_ID, "presented_code": WRONG_CODE},
     "expect": {"commit_hex": COMMIT_HEX, "presented_commit_hex": wrong_commit, "verified": False}},
]


# --------------------------------------------------------------------------- #
# Serialize.
# --------------------------------------------------------------------------- #
def clean_objects():
    out = {}
    for name, e in OBJECTS.items():
        out[name] = {k: v for k, v in e.items() if not k.startswith("_")}
    return out

doc = {
    "wrap_version": 0,
    "conformance_vectors_version": "0.2.0",
    "description": ("WRAP v0.2.0 conformance vectors (14-conformance.md §15.3), regenerated "
                    "against the DMTAP substrate byte formats. These fix ONLY what is WRAP's: "
                    "the object field registry + content address, the object->substrate-primitive "
                    "mapping, the authorship/admission table, Bid observed-remove withdrawal, "
                    "lifecycle fold, computed expiry, attestation retention, forward-compat, and the "
                    "handoff commitment. The substrate's own byte behaviour (deterministic-CBOR "
                    "encoding proper, COSE_Sign1 + DS-tag SIGNATURE VALUES, HLC tie-break, OR-Set/LWW "
                    "convergence) is the SUBSTRATE's vectors and is deliberately NOT re-vectored here "
                    "(§15.2). Every byte value is lowercase hex, no 0x. See conformance/README.md."),
    "value_typing_convention": {
        "uint": "CBOR major type 0. v is a JSON number >= 0.",
        "int": "CBOR major type 0 or 1 by sign. v is a JSON number, may be negative.",
        "tstr": "CBOR major type 3 (UTF-8 text). v is a JSON string.",
        "bstr": "CBOR major type 2 (byte string). v is lowercase hex, no 0x, '' for empty.",
        "bool": "CBOR major type 7 (0xf4/0xf5). v is a JSON boolean.",
        "array": "CBOR major type 4, definite length. v is a JSON array of typed values.",
        "map": "CBOR major type 5, definite length, UNSIGNED-INTEGER keys. v is a JSON object; keys are decimal strings of the uint key, values typed.",
        "refmap": "The one WRAP map with TEXT keys: WorkOrder.refs (key 14, §3.3). v is a plain JSON object string:string.",
    },
    "hlc_encoding_note": ("ts is the substrate Hlc CBOR map {1: u64 wall, 2: u32 counter, 3: ik-pub author} "
                          "(SYNC §3). The retired string form '{ms}-{counter}-{author}' is gone; objects[].ts "
                          "records wall/counter/author_hex, and the encoded map is inside canonical_bytes_hex."),
    "signing_note": ("A signed WRAP object is a substrate COSE_Sign1 (RFC 9052) over "
                     "preimage = \"WRAP-v0/object\" || 0x00 || canonical_bytes (§5.2, §18.1.6). "
                     "objects[].wrap_preimage_hex records that preimage. The COSE_Sign1 envelope bytes "
                     "and the signature value are the SUBSTRATE's conformance vectors (§15.2) and are "
                     "NOT emitted here; the retired bespoke [canonical_bytes, signature] array envelope "
                     "is removed."),
    "keys": KEYS,
    "objects": clean_objects(),
    "vectors": vectors,
}

if __name__ == "__main__":
    import sys
    out_path = sys.argv[1] if len(sys.argv) > 1 else "wrap_vectors.json"
    with open(out_path, "w") as f:
        json.dump(doc, f, indent=2, ensure_ascii=True, sort_keys=True)
        f.write("\n")
    print(f"wrote {out_path}: {len(vectors)} vectors, {len(OBJECTS)} objects, {len(KEYS)} keys")
