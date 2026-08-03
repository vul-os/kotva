"""An **independent** KOTVA wire decoder, written from the specification text.

Why this file exists
--------------------

§18 opens with: "Independent implementations MUST be able to encode and decode every object
below, byte-for-byte identically, **from this text alone**." Until now KOTVA had exactly one
implementation of §18.8a and of the DEPOT schemas, both in Rust, both written in the same
session — so "conformant" and "agrees with that implementation" were the same sentence, exactly
as `profiles/cloud.md` §8 and `conformance/README.md` say out loud. A cross-check between two
Rust crates narrows nothing that a shared assumption would not survive.

Everything below was written by reading:

  * `18-wire-format.md` §18.1 (deterministic CBOR), §18.1.5 (hash prefix), §18.1.7 (prelude),
    §18.2 (length governance by suite), §18.8a.1/§18.8a.2 (`CoordinatorDescriptor`, `Visibility`,
    `Tariff`, `UsageReceipt`);
  * `profiles/cloud.md` §3 (elementals), §3.1/§3.2 (resource + attribute vocabularies),
    §3.3 (`DepotServicePolicy`, `Capacity`), §3.6 (`DepotFormula`, `Part`), §3.7 (`DepotSite`),
    §4.1 (`DepotImage`), §5.2 (ability registry), §7 (`DepotMeasurement`);
  * RFC 8949 §3 and §4.2.1 for the CBOR encoding and the deterministic-encoding rules §18.1.1
    cites.

It deliberately does **not** import, mirror, or transcribe the Rust. Where the spec was
ambiguous the ambiguity is recorded in a comment beginning `SPEC-AMBIGUITY:` rather than resolved
by looking at what the Rust happened to do.

Standard library only, no third-party CBOR: a decoder built on someone else's CBOR library would
inherit that library's idea of "canonical", which is the very thing under test here.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Sequence, Tuple

__all__ = [
    "CborError",
    "SchemaError",
    "decode_canonical",
    "encode_canonical",
    "Visibility",
    "Tariff",
    "UsageReceipt",
    "CoordinatorDescriptor",
    "Capacity",
    "DepotServicePolicy",
    "Part",
    "DepotFormula",
    "DepotSite",
    "DepotImage",
    "DepotMeasurement",
    "read_visibility",
    "read_tariff",
    "read_usage_receipt",
    "read_coordinator_descriptor",
    "read_depot_service_policy",
    "read_depot_formula",
    "read_depot_site",
    "read_depot_image",
    "read_depot_measurement",
]


class CborError(ValueError):
    """A violation of §18.1.1 / RFC 8949 §4.2.1 — the encoding layer."""


class SchemaError(ValueError):
    """A violation of an object's own rule — the schema layer (§18.8a, profiles/cloud.md)."""


# ─────────────────────────────────────────────────────────────────────────────────────────────
# 1. Canonical CBOR (§18.1.1, RFC 8949 §4.2.1)
# ─────────────────────────────────────────────────────────────────────────────────────────────
#
# §18.1.1 admits a deliberately small subset, and this decoder implements exactly it:
#
#   1. shortest-possible argument encoding; NO indefinite-length items;
#   2. map keys sorted by their **encoded** bytes, compared bytewise, ascending;
#   3. no duplicate map keys;
#   4. no floats anywhere; bools only where a rule admits them;
#   5. no `undefined`, no tags, no NaN/Infinity.
#
# Plus the paragraph after the list: `null` is the canonical *absent optional* only inside a
# signing preimage — "A decoder MUST reject a wire map that carries an optional key whose value
# is `null`." Since no wire position admits `null` at all, this decoder rejects `0xf6` outright,
# which is the strictly stronger reading and cannot admit anything the narrow reading forbids.
#
# Negative integers (major type 1) are not produced by any rule in §18 — every integer field is
# declared against `u8`/`u16`/`u32`/`u64`/`ts`, and §18.1.7's integer-domain paragraph requires a
# negative value to be refused "at the decode boundary, before the value ever reaches a
# comparison, an ordering check, or arithmetic". Refusing major type 1 in the decoder is that
# boundary.

_MT_UINT = 0
_MT_NINT = 1
_MT_BYTES = 2
_MT_TEXT = 3
_MT_ARRAY = 4
_MT_MAP = 5
_MT_TAG = 6
_MT_SIMPLE = 7

_MAX_DEPTH = 64  # a bound on nesting, so hostile input cannot exhaust the interpreter stack


def _head(buf: bytes, pos: int) -> Tuple[int, int, int]:
    """Read one CBOR head. Returns (major type, argument, position after the head).

    Enforces §18.1.1 rule 1 (shortest possible encoding of the argument) and the ban on
    indefinite-length items.
    """
    if pos >= len(buf):
        raise CborError(f"truncated: expected an initial byte at offset {pos}")
    ib = buf[pos]
    mt = ib >> 5
    ai = ib & 0x1F
    pos += 1

    if ai < 24:
        return mt, ai, pos
    if ai == 31:
        raise CborError(
            f"indefinite-length item at offset {pos - 1} "
            f"(§18.1.1 rule 1: no indefinite-length items are permitted)"
        )
    if ai in (28, 29, 30):
        raise CborError(f"reserved additional-information {ai} at offset {pos - 1}")

    nbytes = 1 << (ai - 24)  # 24->1, 25->2, 26->4, 27->8
    if pos + nbytes > len(buf):
        raise CborError(f"truncated: {nbytes}-byte argument at offset {pos} runs past the end")
    arg = int.from_bytes(buf[pos : pos + nbytes], "big")
    pos += nbytes

    # Shortest-form check. Major type 7 uses the argument as a simple/float selector rather than
    # as a number, so the numeric rule does not apply there; every such value is rejected below
    # regardless, so no shortest-form exemption can smuggle anything in.
    if mt != _MT_SIMPLE:
        smallest = 0 if ai == 24 else (1 << (8 * (nbytes >> 1)))
        if ai == 24 and arg < 24:
            raise CborError(
                f"non-shortest argument at offset {pos - nbytes}: {arg} fits the 5-bit immediate "
                f"(§18.1.1 rule 1)"
            )
        if ai > 24 and arg < smallest:
            raise CborError(
                f"non-shortest {nbytes}-byte argument at offset {pos - nbytes}: {arg} fits a "
                f"shorter head (§18.1.1 rule 1)"
            )
    return mt, arg, pos


def _item(buf: bytes, pos: int, depth: int) -> Tuple[Any, int]:
    """Decode one item. Returns (value, position after the item)."""
    if depth > _MAX_DEPTH:
        raise CborError(f"nesting deeper than {_MAX_DEPTH}")
    start = pos
    mt, arg, pos = _head(buf, pos)

    if mt == _MT_UINT:
        return arg, pos

    if mt == _MT_NINT:
        raise CborError(
            f"negative integer at offset {start}: every §18 integer field is declared against an "
            f"unsigned domain (§18.1.7 integer-domain governance)"
        )

    if mt in (_MT_BYTES, _MT_TEXT):
        if pos + arg > len(buf):
            raise CborError(f"truncated string at offset {start}: {arg} bytes run past the end")
        raw = buf[pos : pos + arg]
        pos += arg
        if mt == _MT_BYTES:
            return raw, pos
        try:
            return raw.decode("utf-8"), pos
        except UnicodeDecodeError as exc:
            raise CborError(f"text string at offset {start} is not valid UTF-8: {exc}") from exc

    if mt == _MT_ARRAY:
        out: List[Any] = []
        for _ in range(arg):
            v, pos = _item(buf, pos, depth + 1)
            out.append(v)
        return out, pos

    if mt == _MT_MAP:
        out_map: Dict[Any, Any] = {}
        prev_key_bytes: Optional[bytes] = None
        for _ in range(arg):
            k_start = pos
            k, pos = _item(buf, pos, depth + 1)
            k_bytes = buf[k_start:pos]
            if not isinstance(k, (int, str)) or isinstance(k, bool):
                raise CborError(
                    f"map key at offset {k_start} is neither an unsigned integer (§18.1.2) nor a "
                    f"text string (§18.3.6 Headers.ext)"
                )
            # §18.1.1 rule 2 — sorted by their deterministic CBOR encoding, compared bytewise,
            # ascending. NOT lexicographic on the abstract key: a text key's head carries its
            # length, so 0x62 "gb" precedes 0x64 "byte" even though "byte" < "gb" as a string.
            if prev_key_bytes is not None:
                if k_bytes == prev_key_bytes:
                    raise CborError(
                        f"duplicate map key {k!r} at offset {k_start} (§18.1.1 rule 3)"
                    )
                if k_bytes < prev_key_bytes:
                    raise CborError(
                        f"map key {k!r} at offset {k_start} is out of canonical order: encoded "
                        f"{k_bytes.hex()} sorts before the preceding {prev_key_bytes.hex()} "
                        f"(§18.1.1 rule 2, RFC 8949 §4.2.1)"
                    )
            if k in out_map:
                # Unreachable while shortest-form encoding holds (equal keys encode equally, and
                # the ordering check above catches that) — kept because a decoder must never
                # depend on another check having fired first.
                raise CborError(f"duplicate map key {k!r} at offset {k_start} (§18.1.1 rule 3)")
            prev_key_bytes = k_bytes
            v, pos = _item(buf, pos, depth + 1)
            out_map[k] = v
        return out_map, pos

    if mt == _MT_TAG:
        raise CborError(
            f"CBOR tag {arg} at offset {start}: no tag is defined by §18 and §18.1.1 rule 5 "
            f"requires rejection"
        )

    # major type 7
    if arg == 20:
        return False, pos
    if arg == 21:
        return True, pos
    if arg == 22:
        raise CborError(
            f"`null` at offset {start}: on the wire an absent optional field is omitted from the "
            f"map, never present with a null value (§18.1.1)"
        )
    if arg == 23:
        raise CborError(f"`undefined` at offset {start} (§18.1.1 rule 5)")
    if arg in (25, 26, 27):
        raise CborError(
            f"floating-point value at offset {start}: floats do not appear anywhere in KOTVA wire "
            f"objects (§18.1.1 rule 4)"
        )
    raise CborError(f"simple value {arg} at offset {start} is not admitted by §18.1.1")


def decode_canonical(data: bytes) -> Any:
    """Decode one canonical-CBOR item from `data`, rejecting anything left over.

    Trailing bytes are a rejection, not a truncation: §18.1.1's determinism claim is about the
    whole byte string, and a decoder that ignores a tail lets an attacker append to a signed
    object's encoding without changing what a lax verifier sees.
    """
    if not isinstance(data, (bytes, bytearray)):
        raise CborError("input must be bytes")
    data = bytes(data)
    if not data:
        raise CborError("empty input is not a CBOR item")
    value, pos = _item(data, 0, 0)
    if pos != len(data):
        raise CborError(
            f"{len(data) - pos} trailing byte(s) after a complete item "
            f"(0x{data[pos:].hex()}) — §18.1.1"
        )
    return value


def encode_canonical(value: Any) -> bytes:
    """Encode `value` back to canonical CBOR (RFC 8949 §4.2.1).

    Present so a test can demand the **original bytes back**. Decoding alone only proves the
    decoder is permissive enough to accept a vector; re-encoding is what pins the structure —
    a nested map flattened to a byte string, a lost key, or a widened integer all survive a
    decode and die here.
    """
    if isinstance(value, bool):
        return b"\xf5" if value else b"\xf4"
    if isinstance(value, int):
        if value < 0 or value > 0xFFFF_FFFF_FFFF_FFFF:
            raise CborError(f"integer {value} outside the u64 domain")
        return _head_bytes(_MT_UINT, value)
    if isinstance(value, (bytes, bytearray)):
        return _head_bytes(_MT_BYTES, len(value)) + bytes(value)
    if isinstance(value, str):
        raw = value.encode("utf-8")
        return _head_bytes(_MT_TEXT, len(raw)) + raw
    if isinstance(value, list):
        return _head_bytes(_MT_ARRAY, len(value)) + b"".join(encode_canonical(v) for v in value)
    if isinstance(value, dict):
        items = [(encode_canonical(k), encode_canonical(v)) for k, v in value.items()]
        items.sort(key=lambda kv: kv[0])  # bytewise on the ENCODED key (§18.1.1 rule 2)
        return _head_bytes(_MT_MAP, len(items)) + b"".join(k + v for k, v in items)
    raise CborError(f"{type(value).__name__} has no §18.1.1 encoding")


def _head_bytes(mt: int, arg: int) -> bytes:
    if arg < 24:
        return bytes([(mt << 5) | arg])
    if arg <= 0xFF:
        return bytes([(mt << 5) | 24, arg])
    if arg <= 0xFFFF:
        return bytes([(mt << 5) | 25]) + arg.to_bytes(2, "big")
    if arg <= 0xFFFF_FFFF:
        return bytes([(mt << 5) | 26]) + arg.to_bytes(4, "big")
    return bytes([(mt << 5) | 27]) + arg.to_bytes(8, "big")


# ─────────────────────────────────────────────────────────────────────────────────────────────
# 2. Field helpers — the schema layer
# ─────────────────────────────────────────────────────────────────────────────────────────────


def _as_map(value: Any, what: str) -> Dict[Any, Any]:
    if not isinstance(value, dict):
        raise SchemaError(f"{what}: expected a CBOR map, got {_kind_of(value)}")
    return value


def _maybe_decode(data: Any) -> Any:
    """A **top-level** reader entry point: raw wire bytes, or an item already decoded.

    Deliberately NOT used for a nested field. `CoordinatorDescriptor.tariff` (key 6) is
    `? 6 => Tariff`, a nested map — while `policy` and `schedule` are `bytes`. If a nested reader
    ran its field value through this helper, a `bytes`-wrapped tariff whose content happened to be
    valid CBOR would decode, and the one type confusion §18.8a.1's CDDL distinguishes would pass
    unnoticed. Nested reads go through `_as_map`, which refuses anything that is not a map.
    """
    return decode_canonical(data) if isinstance(data, (bytes, bytearray)) else data


def _kind_of(value: Any) -> str:
    if isinstance(value, bool):
        return "bool"
    if isinstance(value, int):
        return "uint"
    if isinstance(value, bytes):
        return f"bytes({len(value)})"
    if isinstance(value, str):
        return f"text({len(value)})"
    if isinstance(value, list):
        return f"array({len(value)})"
    if isinstance(value, dict):
        return f"map({len(value)})"
    return type(value).__name__


def _check_keys(m: Dict[Any, Any], what: str, required: Sequence[int], optional: Sequence[int]) -> None:
    """§18.1.2 fail-closed: a decoder processing a signed object MUST reject any key it does not
    recognise, so the signing preimage is unambiguous. Every object here is either signed
    (§18.8a) or is a `bytes`-embedded schema whose enclosing descriptor is (`profiles/cloud.md`
    §3.7 "Encoding"), so the strict rule is applied uniformly.
    """
    known = set(required) | set(optional)
    unknown = sorted(k for k in m if k not in known)
    if unknown:
        raise SchemaError(f"{what}: unrecognised key(s) {unknown} — fail closed (§18.1.2)")
    missing = [k for k in required if k not in m]
    if missing:
        raise SchemaError(f"{what}: required key(s) {missing} absent")
    for k in m:
        if not isinstance(k, int) or isinstance(k, bool):
            raise SchemaError(f"{what}: key {k!r} is not an unsigned integer (§18.1.2)")


def _u(m: Dict[Any, Any], key: int, what: str, name: str, *, bits: int = 64) -> int:
    v = m[key]
    if isinstance(v, bool) or not isinstance(v, int):
        raise SchemaError(f"{what}.{name} (key {key}): expected uint, got {_kind_of(v)}")
    if v >> bits:
        raise SchemaError(f"{what}.{name} (key {key}): {v} exceeds the u{bits} domain (§18.1.7)")
    return v


def _t(m: Dict[Any, Any], key: int, what: str, name: str) -> str:
    v = m[key]
    if not isinstance(v, str):
        raise SchemaError(f"{what}.{name} (key {key}): expected tstr, got {_kind_of(v)}")
    return v


def _bs(m: Dict[Any, Any], key: int, what: str, name: str) -> bytes:
    v = m[key]
    if not isinstance(v, bytes):
        raise SchemaError(f"{what}.{name} (key {key}): expected bytes, got {_kind_of(v)}")
    return v


def _closed(value: str, registry: Sequence[str], what: str, name: str, clause: str) -> str:
    if value not in registry:
        raise SchemaError(
            f"{what}.{name}: {value!r} is not in the closed registry {sorted(registry)} ({clause})"
        )
    return value


def _det_cbor_blob(raw: bytes, what: str, name: str) -> Any:
    """An opaque field the spec types as **det_cbor bytes**.

    Opaque means "this document does not interpret it" (§18.8a.1 `policy`), not "unvalidated
    bytes": the declared type is deterministic CBOR, so a blob that is not canonical CBOR is
    malformed regardless of what it would have meant. Returning the decoded value lets a caller
    inspect it without re-parsing.
    """
    try:
        return decode_canonical(raw)
    except CborError as exc:
        raise SchemaError(f"{what}.{name}: not canonical det_cbor — {exc}") from exc


# §18.2, "Length & type governance by suite". `suite = 0x03` and `0x05` share `0x02`'s layout
# exactly; `0x04` differs in the two signature rows only.
_SUITE_LENGTHS: Dict[int, Tuple[int, int]] = {
    0x01: (32, 64),  # Ed25519
    0x02: (32 + 1952, 64 + 3309),  # Ed25519 ‖ ML-DSA-65
    0x03: (32 + 1952, 64 + 3309),  # identical layout to 0x02 (AEAD differs)
    0x04: (64, 7920),  # Ed25519 ‖ SLH-DSA-128s
    0x05: (32 + 1952, 64 + 3309),  # identical layout to 0x02 (hash differs)
}


def _suite(m: Dict[Any, Any], what: str) -> int:
    s = _u(m, 1, what, "suite", bits=8)
    if s == 0:
        raise SchemaError(f"{what}.suite: 0x00 is outside the `suite = 0x01..0xff` domain (§18.1.7)")
    if s not in _SUITE_LENGTHS:
        raise SchemaError(
            f"{what}.suite: 0x{s:02x} is not a suite this decoder implements — fail closed, "
            f"ERR_UNKNOWN_SUITE (§18.1.4, §18.2)"
        )
    return s


def _suite_governed(raw: bytes, suite: int, kind: str, what: str, name: str) -> bytes:
    ik_len, sig_len = _SUITE_LENGTHS[suite]
    expected = ik_len if kind == "ik-pub" else sig_len
    if len(raw) != expected:
        raise SchemaError(
            f"{what}.{name}: {kind} is {len(raw)} B; suite 0x{suite:02x} fixes it at {expected} B "
            f"(§18.2)"
        )
    return raw


# §18.1.5 — a v0 `hash` is exactly 33 bytes: a one-byte multicodec algorithm prefix followed by a
# 32-byte digest. §18.1.7's prelude widens the CDDL to `bytes .size (33..129)` for future suites,
# so 33 is the floor under every suite defined today.
_HASH_PREFIXES = {0x1E: "BLAKE3-256", 0x12: "SHA2-256", 0x16: "SHA3-256"}


def _hash_field(raw: bytes, what: str, name: str, deviations: List[str]) -> bytes:
    """Read a `hash`, recording — never silently tolerating — a §18.1.5 violation.

    This is NOT a schema rejection, and the choice is deliberate. The frozen DEPOT corpora carry
    3-byte values in `hash`-typed positions, which §18.1.5 and the §18.1.7 prelude both forbid.
    Rejecting would make this decoder disagree with the corpus for a reason that has nothing to do
    with the encoding under test, and silently accepting would hide a real spec/corpus
    disagreement. Recording it makes the disagreement a value a test can assert on, so it cannot
    drift in either direction unnoticed.
    """
    if not isinstance(raw, bytes):
        raise SchemaError(f"{what}.{name}: expected bytes, got {_kind_of(raw)}")
    if len(raw) < 33:
        deviations.append(
            f"{what}.{name}: hash is {len(raw)} B; §18.1.5 fixes a v0 hash at 33 B "
            f"(1-byte alg prefix ‖ 32-byte digest) and §18.1.7's prelude at .size (33..129)"
        )
    elif len(raw) > 129:
        deviations.append(f"{what}.{name}: hash is {len(raw)} B; §18.1.7's prelude caps it at 129 B")
    elif raw[0] not in _HASH_PREFIXES:
        deviations.append(
            f"{what}.{name}: multihash prefix 0x{raw[0]:02x} is not one of "
            f"{{0x1e, 0x12, 0x16}} (§18.1.5)"
        )
    return raw


# ─────────────────────────────────────────────────────────────────────────────────────────────
# 3. §18.8a — coordinator-layer objects
# ─────────────────────────────────────────────────────────────────────────────────────────────

# §18.8a.1's `kind` row: "the **eleven** of that table … which is authoritative and which no other
# document may re-enumerate differently". `"compute"` is deliberately absent — it folded into
# `"infra-service"` and a decoder MUST reject it like any other unknown string.
COORDINATOR_KINDS: Tuple[str, ...] = (
    "gateway",
    "relay",
    "media-relay",
    "reachability-adapter",
    "infra-service",
    "indexer",
    "labeler",
    "matcher",
    "arbiter",
    "oracle",
    "custodial-escrow",
)

VISIBILITY_CLASSES: Tuple[str, ...] = ("blind", "blind-routing", "terminating")
ASSURANCE_LEVELS: Tuple[str, ...] = ("structural", "attested", "declared")


@dataclass(frozen=True)
class Visibility:
    """§18.8a.1 `Visibility` — exactly one declared class at one assurance level."""

    cls: str
    level: str


def read_visibility(value: Any) -> Visibility:
    m = _as_map(_maybe_decode(value), "Visibility")
    _check_keys(m, "Visibility", required=(1, 2), optional=())
    cls = _closed(_t(m, 1, "Visibility", "class"), VISIBILITY_CLASSES, "Visibility", "class", "§18.8a.1")
    level = _closed(_t(m, 2, "Visibility", "level"), ASSURANCE_LEVELS, "Visibility", "level", "§18.8a.1")
    # "A `terminating` class MUST NOT declare `level = "structural"`: there is no `"structural"`
    # assurance level for a plaintext-terminating role." Both `declared` and `attested` remain
    # declarable for `terminating` — §18.8a.1 records that the older "MUST declare declared"
    # wording was over-broad, so this decoder forbids exactly the one pair.
    if cls == "terminating" and level == "structural":
        raise SchemaError(
            "Visibility: class 'terminating' MUST NOT declare level 'structural' — a role that "
            "sees the data cannot claim the protocol prevents it from seeing the data (§18.8a.1)"
        )
    return Visibility(cls=cls, level=level)


@dataclass(frozen=True)
class Tariff:
    """§18.8a.1 `Tariff` — self-certifying: it carries its own signer."""

    suite: int
    identity: bytes
    schedule: bytes
    schedule_decoded: Any
    valid_until: Optional[int]
    sig: bytes

    def is_valid_at(self, now_ms: int) -> bool:
        """`valid_until` absent ⇒ no expiry; present ⇒ the window is inclusive of its endpoint.

        SPEC-AMBIGUITY (resolved from the spec, not the Rust): §18.8a.1 says a tariff "presented
        past `valid_until` MUST be treated as expired" without saying whether `now == valid_until`
        is past. "Past X" in ordinary reading excludes X, so the endpoint is inclusive. The Rust
        agrees, but this reading was taken from the sentence.
        """
        return self.valid_until is None or now_ms <= self.valid_until


def read_tariff(value: Any) -> Tariff:
    m = _as_map(_maybe_decode(value), "Tariff")
    _check_keys(m, "Tariff", required=(1, 2, 3, 5), optional=(4,))
    suite = _suite(m, "Tariff")
    identity = _suite_governed(_bs(m, 2, "Tariff", "identity"), suite, "ik-pub", "Tariff", "identity")
    schedule = _bs(m, 3, "Tariff", "schedule")
    decoded = _det_cbor_blob(schedule, "Tariff", "schedule")
    valid_until = _u(m, 4, "Tariff", "valid_until") if 4 in m else None
    sig = _suite_governed(_bs(m, 5, "Tariff", "sig"), suite, "sig-val", "Tariff", "sig")
    return Tariff(
        suite=suite,
        identity=identity,
        schedule=schedule,
        schedule_decoded=decoded,
        valid_until=valid_until,
        sig=sig,
    )


@dataclass(frozen=True)
class UsageReceipt:
    """§18.8a.2 `UsageReceipt` — independently self-certifying."""

    suite: int
    identity: bytes
    operation: bytes
    operation_decoded: Any
    sig: bytes


def read_usage_receipt(value: Any) -> UsageReceipt:
    m = _as_map(_maybe_decode(value), "UsageReceipt")
    _check_keys(m, "UsageReceipt", required=(1, 2, 3, 4), optional=())
    suite = _suite(m, "UsageReceipt")
    identity = _suite_governed(
        _bs(m, 2, "UsageReceipt", "identity"), suite, "ik-pub", "UsageReceipt", "identity"
    )
    operation = _bs(m, 3, "UsageReceipt", "operation")
    decoded = _det_cbor_blob(operation, "UsageReceipt", "operation")
    sig = _suite_governed(_bs(m, 4, "UsageReceipt", "sig"), suite, "sig-val", "UsageReceipt", "sig")
    return UsageReceipt(
        suite=suite, identity=identity, operation=operation, operation_decoded=decoded, sig=sig
    )


@dataclass(frozen=True)
class CoordinatorDescriptor:
    """§18.8a.1 `CoordinatorDescriptor` — discovery-only and self-asserted."""

    suite: int
    kind: str
    identity: bytes
    visibility: Visibility
    policy: bytes
    policy_decoded: Any
    tariff: Optional[Tariff]
    sig: bytes

    @property
    def charges(self) -> bool:
        """§18.8a.1: `tariff` is present **iff** this coordinator charges; absent ⇒ free."""
        return self.tariff is not None


def read_coordinator_descriptor(value: Any) -> CoordinatorDescriptor:
    m = _as_map(_maybe_decode(value), "CoordinatorDescriptor")
    what = "CoordinatorDescriptor"
    # Key 6 is the only optional. The unknown-key rejection here is load-bearing rather than
    # hygienic: §18.8a.1 says the descriptor "has no field for a global score, a price rank, or a
    # stake amount … a decoder MUST reject an unknown key exactly so a future field cannot smuggle
    # one back in".
    _check_keys(m, what, required=(1, 2, 3, 4, 5, 7), optional=(6,))
    suite = _suite(m, what)
    kind = _closed(_t(m, 2, what, "kind"), COORDINATOR_KINDS, what, "kind", "§18.8a.1 / CONTRACT §5")
    identity = _suite_governed(_bs(m, 3, what, "identity"), suite, "ik-pub", what, "identity")
    visibility = read_visibility(_as_map(m[4], f"{what}.visibility"))
    policy = _bs(m, 5, what, "policy")
    policy_decoded = _det_cbor_blob(policy, what, "policy")
    # The tariff nests as a **map**, not a bytes-wrapped blob: §18.8a.1's CDDL writes
    # `? 6 => Tariff`, where `policy` and `schedule` are explicitly `bytes`. A `bytes`-wrapped
    # tariff would round-trip happily inside one implementation and interop with nobody.
    tariff = read_tariff(_as_map(m[6], f"{what}.tariff")) if 6 in m else None
    sig = _suite_governed(_bs(m, 7, what, "sig"), suite, "sig-val", what, "sig")
    return CoordinatorDescriptor(
        suite=suite,
        kind=kind,
        identity=identity,
        visibility=visibility,
        policy=policy,
        policy_decoded=policy_decoded,
        tariff=tariff,
        sig=sig,
    )


# ─────────────────────────────────────────────────────────────────────────────────────────────
# 4. profiles/cloud.md — the DEPOT schemas
# ─────────────────────────────────────────────────────────────────────────────────────────────

# §3, "The registry below is four rows and is meant to stay four."
SERVICES: Tuple[str, ...] = ("bucket", "volume", "box", "edge-fn")
# §3.3, `backing` — CLOSED (§1.2's three modes).
BACKINGS: Tuple[str, ...] = ("operator", "customer", "mixed")
# §3.3 `Capacity.class` — the latency tier.
CAPACITY_CLASSES: Tuple[str, ...] = ("cold", "warm", "commit-path")
# §4.1 — both CLOSED. `bucket` is absent from the targets: a bucket holds images, it is not
# instantiated from one.
IMAGE_TARGETS: Tuple[str, ...] = ("box", "edge-fn", "volume")
IMAGE_FORMATS: Tuple[str, ...] = ("raw", "qcow2", "oci", "wasm", "qir", "qasm", "fs-dump")
# §7 — `metric`, `method` and evidence `kind` are closed value sets.
METRICS: Tuple[str, ...] = (
    "uptime",
    "conformance",
    "visibility-audit",
    "latency-ms",
    "capacity-conformance",
    "export-conformance",
    "ability-conformance",
)
METHODS: Tuple[str, ...] = ("probe", "conformance-vector", "audit", "self-report")
EVIDENCE_KINDS: Tuple[str, ...] = ("recipe", "vector-id", "transcript")
# §5.2 — CLOSED, extended by registry addition, never by coinage.
COMMON_ABILITIES: Tuple[str, ...] = (
    "provision",
    "inspect",
    "list",
    "reconfigure",
    "observe",
    "export",
    "destroy",
)
PER_SERVICE_ABILITIES: Dict[str, Tuple[str, ...]] = {
    "box": ("start", "stop", "restart", "snapshot", "console"),
    "volume": ("attach", "detach", "resize", "snapshot"),
    "bucket": ("read", "write", "delete", "serve"),
    "edge-fn": ("deploy", "invoke", "rollback"),
}


def _text_map(value: Any, what: str, name: str, *, values: str) -> List[Tuple[str, Any]]:
    """A `{ * tstr => X }` field. Returned as a list so the canonical order stays visible."""
    m = _as_map(value, f"{what}.{name}")
    out: List[Tuple[str, Any]] = []
    for k, v in m.items():
        if not isinstance(k, str):
            raise SchemaError(f"{what}.{name}: key {k!r} is not a tstr")
        if values == "tstr" and not isinstance(v, str):
            raise SchemaError(f"{what}.{name}[{k!r}]: expected tstr, got {_kind_of(v)}")
        if values == "uint" and (isinstance(v, bool) or not isinstance(v, int)):
            raise SchemaError(f"{what}.{name}[{k!r}]: expected uint, got {_kind_of(v)}")
        out.append((k, v))
    return out


@dataclass(frozen=True)
class Capacity:
    """§3.3 `Capacity` — declared ceilings. **Absent means undeclared, never unlimited.**"""

    total_bytes: Optional[int] = None
    max_object_bytes: Optional[int] = None
    egress_bps: Optional[int] = None
    max_concurrent: Optional[int] = None
    cls: Optional[str] = None
    uptime_target: Optional[int] = None
    resources: Tuple[Tuple[str, int], ...] = ()


def read_capacity(value: Any) -> Capacity:
    m = _as_map(_maybe_decode(value), "Capacity")
    _check_keys(m, "Capacity", required=(), optional=(1, 2, 3, 4, 5, 6, 7))
    uptime = _u(m, 6, "Capacity", "uptime_target") if 6 in m else None
    if uptime is not None and uptime > 1000:
        raise SchemaError(
            f"Capacity.uptime_target: {uptime} is outside the per-mille domain 0…1000 (§3.3)"
        )
    return Capacity(
        total_bytes=_u(m, 1, "Capacity", "total_bytes") if 1 in m else None,
        max_object_bytes=_u(m, 2, "Capacity", "max_object_bytes") if 2 in m else None,
        egress_bps=_u(m, 3, "Capacity", "egress_bps") if 3 in m else None,
        max_concurrent=_u(m, 4, "Capacity", "max_concurrent") if 4 in m else None,
        cls=_closed(_t(m, 5, "Capacity", "class"), CAPACITY_CLASSES, "Capacity", "class", "§3.3")
        if 5 in m
        else None,
        uptime_target=uptime,
        resources=tuple(_text_map(m[7], "Capacity", "resources", values="uint")) if 7 in m else (),
    )


@dataclass(frozen=True)
class DepotServicePolicy:
    """§3.3 `DepotServicePolicy` — the det_cbor `policy` blob for `kind = "infra-service"`."""

    service: str
    backing: str
    capacity: Optional[Capacity] = None
    attributes: Tuple[Tuple[str, str], ...] = ()
    abilities: Tuple[str, ...] = ()
    deviations: Tuple[str, ...] = ()


def read_depot_service_policy(data: Any) -> DepotServicePolicy:
    value = _maybe_decode(data)
    what = "DepotServicePolicy"
    m = _as_map(value, what)
    _check_keys(m, what, required=(1, 2), optional=(3, 4, 5))
    service = _closed(_t(m, 1, what, "service"), SERVICES, what, "service", "§3")
    backing = _closed(_t(m, 2, what, "backing"), BACKINGS, what, "backing", "§3.3 / §1.2")
    abilities: Tuple[str, ...] = ()
    if 5 in m:
        raw = m[5]
        if not isinstance(raw, list):
            raise SchemaError(f"{what}.abilities: expected an array, got {_kind_of(raw)}")
        allowed = set(COMMON_ABILITIES) | set(PER_SERVICE_ABILITIES[service])
        for a in raw:
            if not isinstance(a, str):
                raise SchemaError(f"{what}.abilities: element {a!r} is not a tstr")
            if a not in allowed:
                # §5.2: "A coordinator receiving an ability outside this registry MUST refuse and
                # MUST NOT map it onto a similar-sounding one."
                raise SchemaError(
                    f"{what}.abilities: {a!r} is not in the CLOSED §5.2 registry for "
                    f"service {service!r}"
                )
        abilities = tuple(raw)
    return DepotServicePolicy(
        service=service,
        backing=backing,
        capacity=read_capacity(m[3]) if 3 in m else None,
        attributes=tuple(_text_map(m[4], what, "attributes", values="tstr")) if 4 in m else (),
        abilities=abilities,
    )


@dataclass(frozen=True)
class Part:
    """§3.6 `Part` — one primitive coordinator a formula composes."""

    service: str
    provider: bytes
    descriptor: Optional[bytes] = None


@dataclass(frozen=True)
class DepotFormula:
    """§3.6 `DepotFormula` — a recipe that composes elementals, never a new mechanism."""

    kind: str
    parts: Tuple[Part, ...]
    recipe: Optional[bytes] = None
    consensus: Optional[str] = None
    deviations: Tuple[str, ...] = ()

    @property
    def scales_horizontally(self) -> bool:
        """§3.6: "an absent `consensus` means single-writer, and such a formula MUST NOT advertise
        horizontal scaling"."""
        return self.consensus is not None


def read_depot_formula(data: Any) -> DepotFormula:
    value = _maybe_decode(data)
    what = "DepotFormula"
    m = _as_map(value, what)
    _check_keys(m, what, required=(1, 2), optional=(3, 4))
    kind = _t(m, 1, what, "kind")
    raw_parts = m[2]
    if not isinstance(raw_parts, list):
        raise SchemaError(f"{what}.parts: expected an array, got {_kind_of(raw_parts)}")
    if not raw_parts:
        raise SchemaError(f"{what}.parts: `[+ Part]` requires at least one part (§3.6)")
    deviations: List[str] = []
    parts: List[Part] = []
    for i, rp in enumerate(raw_parts):
        pm = _as_map(rp, f"{what}.parts[{i}]")
        _check_keys(pm, f"{what}.parts[{i}]", required=(1, 2), optional=(3,))
        service = _closed(
            _t(pm, 1, f"{what}.parts[{i}]", "service"), SERVICES, what, f"parts[{i}].service", "§3"
        )
        provider = _bs(pm, 2, f"{what}.parts[{i}]", "provider")
        # SPEC-AMBIGUITY (recorded, not resolved by reading the Rust): `Part.provider` is typed
        # `ik-pub`, whose length §18.2 fixes "by the suite of the key that made the signature" —
        # but `DepotFormula` carries no `suite` field and is not in §18.1.4's index of objects and
        # their versioning hooks, so there is no suite to select a row with. The length is
        # therefore unenforceable here as specified; a mismatch against every defined suite is
        # recorded rather than rejected.
        if len(provider) not in {32, 64, 1984}:
            deviations.append(
                f"{what}.parts[{i}].provider: ik-pub is {len(provider)} B, which is no suite's "
                f"length (§18.2: 32 / 64 / 1984) — and DepotFormula carries no `suite` hook "
                f"(§18.1.4) to select a row with, so the constraint is unenforceable as specified"
            )
        descriptor = None
        if 3 in pm:
            descriptor = _hash_field(
                pm[3], what, f"parts[{i}].descriptor", deviations
            )
        parts.append(Part(service=service, provider=provider, descriptor=descriptor))
    recipe = _bs(m, 3, what, "recipe") if 3 in m else None
    if recipe is not None:
        _det_cbor_blob(recipe, what, "recipe")
    return DepotFormula(
        kind=kind,
        parts=tuple(parts),
        recipe=recipe,
        consensus=_t(m, 4, what, "consensus") if 4 in m else None,
        deviations=tuple(deviations),
    )


@dataclass(frozen=True)
class Redirect:
    """§3.7 `DepotSite` redirect entry — `{ from, to, status }`."""

    src: str
    dst: str
    status: int


@dataclass(frozen=True)
class DepotSite:
    """§3.7 `DepotSite` — static-site / SPA hosting as PUB objects in a public-serving bucket."""

    root: bytes
    fallback: Optional[str] = None
    redirects: Tuple[Redirect, ...] = ()
    cache_max_age_s: Optional[int] = None
    cache_immutable: Optional[bool] = None
    deviations: Tuple[str, ...] = ()


def read_depot_site(data: Any) -> DepotSite:
    value = _maybe_decode(data)
    what = "DepotSite"
    m = _as_map(value, what)
    _check_keys(m, what, required=(1,), optional=(2, 3, 4))
    deviations: List[str] = []
    root = _hash_field(m[1], what, "root", deviations)
    redirects: List[Redirect] = []
    if 3 in m:
        raw = m[3]
        if not isinstance(raw, list):
            raise SchemaError(f"{what}.redirects: expected an array, got {_kind_of(raw)}")
        for i, r in enumerate(raw):
            rm = _as_map(r, f"{what}.redirects[{i}]")
            _check_keys(rm, f"{what}.redirects[{i}]", required=(1, 2, 3), optional=())
            redirects.append(
                Redirect(
                    src=_t(rm, 1, f"{what}.redirects[{i}]", "from"),
                    dst=_t(rm, 2, f"{what}.redirects[{i}]", "to"),
                    status=_u(rm, 3, f"{what}.redirects[{i}]", "status"),
                )
            )
    max_age = None
    immutable = None
    if 4 in m:
        cm = _as_map(m[4], f"{what}.cache")
        _check_keys(cm, f"{what}.cache", required=(), optional=(1, 2))
        max_age = _u(cm, 1, f"{what}.cache", "max_age_s") if 1 in cm else None
        if 2 in cm:
            v = cm[2]
            if not isinstance(v, bool):
                raise SchemaError(f"{what}.cache.immutable: expected bool, got {_kind_of(v)}")
            immutable = v
    return DepotSite(
        root=root,
        fallback=_t(m, 2, what, "fallback") if 2 in m else None,
        redirects=tuple(redirects),
        cache_max_age_s=max_age,
        cache_immutable=immutable,
        deviations=tuple(deviations),
    )


@dataclass(frozen=True)
class DepotImage:
    """§4.1 `DepotImage` — one schema, three targets."""

    target: str
    format: str
    digest: bytes
    size_bytes: int
    arch: Optional[str] = None
    boot: Tuple[Tuple[str, str], ...] = ()
    parent: Optional[bytes] = None
    deviations: Tuple[str, ...] = ()


def read_depot_image(data: Any) -> DepotImage:
    value = _maybe_decode(data)
    what = "DepotImage"
    m = _as_map(value, what)
    _check_keys(m, what, required=(1, 2, 3, 4), optional=(5, 6, 7))
    deviations: List[str] = []
    target = _closed(_t(m, 1, what, "target"), IMAGE_TARGETS, what, "target", "§4.1 CLOSED")
    # "An unrecognised `format` MUST be refused, never guessed at."
    fmt = _closed(_t(m, 2, what, "format"), IMAGE_FORMATS, what, "format", "§4.1 CLOSED registry")
    digest = _hash_field(m[3], what, "digest", deviations)
    size = _u(m, 4, what, "bytes")
    parent = _hash_field(m[7], what, "parent", deviations) if 7 in m else None
    return DepotImage(
        target=target,
        format=fmt,
        digest=digest,
        size_bytes=size,
        arch=_t(m, 5, what, "arch") if 5 in m else None,
        boot=tuple(_text_map(m[6], what, "boot", values="tstr")) if 6 in m else (),
        parent=parent,
        deviations=tuple(deviations),
    )


@dataclass(frozen=True)
class Evidence:
    """§7 `DepotMeasurement.evidence` — `{ kind: CLOSED, ref }`."""

    kind: str
    ref: str


@dataclass(frozen=True)
class DepotMeasurement:
    """§7 `DepotMeasurement` — the claim body of an ATTEST claim, not a PUB payload."""

    service: str
    metric: str
    value: Any
    method: str
    observed_at: int
    evidence: Optional[Evidence] = None
    deviations: Tuple[str, ...] = ()


# §7: "`value` is typed **by `metric`**".
_BOOL_METRICS = frozenset(
    {
        "conformance",
        "visibility-audit",
        "capacity-conformance",
        "export-conformance",
        "ability-conformance",
    }
)


def read_depot_measurement(data: Any) -> DepotMeasurement:
    value = _maybe_decode(data)
    what = "DepotMeasurement"
    m = _as_map(value, what)
    _check_keys(m, what, required=(1, 2, 3, 4, 5), optional=(6,))
    service = _closed(_t(m, 1, what, "service"), SERVICES, what, "service", "§3")
    metric = _closed(_t(m, 2, what, "metric"), METRICS, what, "metric", "§7 CLOSED")
    raw = m[3]
    if metric in _BOOL_METRICS:
        if not isinstance(raw, bool):
            raise SchemaError(
                f"{what}.value: metric {metric!r} is bool-typed, got {_kind_of(raw)} (§7)"
            )
    else:
        if isinstance(raw, bool) or not isinstance(raw, int):
            raise SchemaError(
                f"{what}.value: metric {metric!r} is uint-typed, got {_kind_of(raw)} (§7)"
            )
        if metric == "uptime" and raw > 1000:
            raise SchemaError(f"{what}.value: uptime is per-mille 0…1000, got {raw} (§7)")
    method = _closed(_t(m, 4, what, "method"), METHODS, what, "method", "§7 CLOSED")
    observed_at = _u(m, 5, what, "observed_at")
    evidence = None
    if 6 in m:
        em = _as_map(m[6], f"{what}.evidence")
        _check_keys(em, f"{what}.evidence", required=(1, 2), optional=())
        evidence = Evidence(
            kind=_closed(
                _t(em, 1, f"{what}.evidence", "kind"),
                EVIDENCE_KINDS,
                what,
                "evidence.kind",
                "§7 CLOSED",
            ),
            ref=_t(em, 2, f"{what}.evidence", "ref"),
        )
    return DepotMeasurement(
        service=service,
        metric=metric,
        value=raw,
        method=method,
        observed_at=observed_at,
        evidence=evidence,
    )
