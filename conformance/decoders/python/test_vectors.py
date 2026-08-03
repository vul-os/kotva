"""Run the frozen Rust corpora through the independent Python decoder.

The corpora are **read out of the Rust test files at run time**, never copied here. That is the
whole point of the arrangement: a second copy of the hex would let the two corpora drift apart
silently, and the first thing to notice would be an interop failure at a third party. Reading them
means a Rust-side edit to a vector immediately re-runs against a decoder written from the spec.

    python3 -m unittest discover -s conformance/decoders/python -v

Standard library only — `unittest`, no pytest, no pip.
"""

from __future__ import annotations

import re
import unittest
from pathlib import Path
from typing import Dict, List

import kotva_decode as kd

REPO_ROOT = Path(__file__).resolve().parents[3]
DEPOT_VECTORS_RS = REPO_ROOT / "crates" / "kotva-depot" / "tests" / "vectors.rs"
COORD_VECTORS_RS = REPO_ROOT / "crates" / "kotva-coordinator" / "tests" / "vectors.rs"


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Corpus extraction
# ─────────────────────────────────────────────────────────────────────────────────────────────

_CONST_RE = re.compile(r"const\s+([A-Z0-9_]+)\s*:\s*&str\s*=\s*(.*?);", re.S)
_STR_LITERAL_RE = re.compile(r'"([^"\\]*)"')
_HEX_CALL_RE = re.compile(r'hex\(&?"([^"\\]*)"\)')


def _strip_comments(src: str) -> str:
    """Drop `//`-to-end-of-line comments.

    Done before anything else because the doc comments carry byte decompositions containing
    quoted strings (`; text(6) "bucket"`), which would otherwise be scraped as if they were
    vector hex. No hex literal contains `//`, so the naive strip is safe here.
    """
    return "\n".join(line.split("//")[0] for line in src.splitlines())


def _rust_hex_consts(path: Path) -> Dict[str, str]:
    """Extract `const NAME: &str = "…";` and `const NAME: &str = concat!("…", "…");`."""
    src = _strip_comments(path.read_text(encoding="utf-8"))
    out: Dict[str, str] = {}
    for name, rhs in _CONST_RE.findall(src):
        pieces = _STR_LITERAL_RE.findall(rhs)
        if not pieces:
            continue
        out[name] = "".join(pieces)
    return out


def _inline_hex_in_fn(path: Path, fn_name: str) -> List[str]:
    """Extract every `hex("…")` literal argument inside one `#[test] fn`.

    Some corruption controls are inline in the test body rather than named constants; they are
    part of the corpus and must be read from the Rust like everything else.
    """
    src = _strip_comments(path.read_text(encoding="utf-8"))
    start = src.index(f"fn {fn_name}(")
    end = src.index("\n}", start)
    return _HEX_CALL_RE.findall(src[start:end])


DEPOT = _rust_hex_consts(DEPOT_VECTORS_RS)
COORD = _rust_hex_consts(COORD_VECTORS_RS)
DEPOT_INLINE_CORRUPTIONS = _inline_hex_in_fn(DEPOT_VECTORS_RS, "corrupted_vectors_are_rejected")


def h(hex_str: str) -> bytes:
    return bytes.fromhex(hex_str)


class CorpusExtraction(unittest.TestCase):
    """If extraction silently produced nothing, every test below would pass while proving nothing
    — the exact failure mode this repo has hit repeatedly. Pin the shape of what was read."""

    def test_every_expected_constant_was_found(self):
        self.assertEqual(
            sorted(DEPOT),
            [
                "FORMULA_REDIS",
                "IMAGE_EDGE_FN",
                "MEASUREMENT_UPTIME",
                "POLICY_MINIMAL",
                "SITE_MINIMAL",
            ],
        )
        self.assertEqual(
            sorted(COORD),
            [
                "DESCRIPTOR_KIND_COMPUTE",
                "DESCRIPTOR_SHORT_IK",
                "DESCRIPTOR_SMUGGLED_SCORE",
                "DESCRIPTOR_TERMINATING_STRUCTURAL",
                "DESCRIPTOR_V0",
                "DESCRIPTOR_WITH_TARIFF_V0",
                "IK_DESCRIPTOR",
                "IK_RECEIPT",
                "IK_TARIFF",
                "RECEIPT_V0",
                "SIG_DESCRIPTOR",
                "SIG_RECEIPT",
                "SIG_TARIFF",
                "TARIFF_V0",
            ],
        )

    def test_every_extracted_string_is_well_formed_hex(self):
        for source, table in (("depot", DEPOT), ("coordinator", COORD)):
            for name, value in table.items():
                with self.subTest(source=source, const=name):
                    self.assertNotEqual(value, "", "empty extraction")
                    self.assertEqual(len(value) % 2, 0, "odd-length hex")
                    self.assertRegex(value, r"\A[0-9a-f]+\Z")

    def test_the_inline_corruption_controls_were_found(self):
        # Three inline mutations plus a re-use of IMAGE_EDGE_FN as the false-positive control; the
        # truncation and empty-input cases are constructed, not literal.
        self.assertEqual(len(DEPOT_INLINE_CORRUPTIONS), 3, DEPOT_INLINE_CORRUPTIONS)

    def test_filler_lengths_match_18_2(self):
        for name in ("IK_DESCRIPTOR", "IK_TARIFF", "IK_RECEIPT"):
            self.assertEqual(len(h(COORD[name])), 32, f"{name}: ik-pub is 32 B under suite 0x01")
        for name in ("SIG_DESCRIPTOR", "SIG_TARIFF", "SIG_RECEIPT"):
            self.assertEqual(len(h(COORD[name])), 64, f"{name}: sig-val is 64 B under suite 0x01")


# ─────────────────────────────────────────────────────────────────────────────────────────────
# §18.8a — coordinator-layer objects
# ─────────────────────────────────────────────────────────────────────────────────────────────


class CoordinatorVectors(unittest.TestCase):
    def test_usage_receipt(self):
        r = kd.read_usage_receipt(h(COORD["RECEIPT_V0"]))
        self.assertEqual(r.suite, 0x01)
        self.assertEqual(r.identity, h(COORD["IK_RECEIPT"]))
        self.assertEqual(r.sig, h(COORD["SIG_RECEIPT"]))
        # The operation blob is opaque to §18.8a.2 but IS declared det_cbor, so a decoder that
        # validates it gets exactly what the Rust decomposition claims: map(1){ 1 : unsigned(7) }.
        self.assertEqual(r.operation, bytes.fromhex("a10107"))
        self.assertEqual(r.operation_decoded, {1: 7})

    def test_tariff(self):
        t = kd.read_tariff(h(COORD["TARIFF_V0"]))
        self.assertEqual(t.suite, 0x01)
        self.assertEqual(t.identity, h(COORD["IK_TARIFF"]))
        self.assertEqual(t.sig, h(COORD["SIG_TARIFF"]))
        self.assertEqual(t.valid_until, 1_700_000_000_000)
        self.assertEqual(t.schedule_decoded, {"gb": 3, "byte": 5})
        # Length-first ordering inside the opaque schedule: a decoder that sorted text keys
        # lexicographically would have rejected the vector outright, so reaching this assertion
        # already proves the rule; the explicit order check makes the intent visible.
        self.assertEqual(list(t.schedule_decoded), ["gb", "byte"])
        # `valid_until` bounds the window inclusively (§18.8a.1: "presented past `valid_until`").
        self.assertTrue(t.is_valid_at(1_700_000_000_000))
        self.assertFalse(t.is_valid_at(1_700_000_000_001))

    def test_descriptor_without_tariff(self):
        d = kd.read_coordinator_descriptor(h(COORD["DESCRIPTOR_V0"]))
        self.assertEqual(d.suite, 0x01)
        self.assertEqual(d.kind, "infra-service")
        self.assertEqual(d.identity, h(COORD["IK_DESCRIPTOR"]))
        self.assertEqual(d.visibility, kd.Visibility(cls="blind", level="structural"))
        self.assertEqual(d.sig, h(COORD["SIG_DESCRIPTOR"]))
        self.assertIsNone(d.tariff, "key 6 absent ⇒ this coordinator does not charge")
        self.assertFalse(d.charges)

    def test_descriptor_with_tariff(self):
        d = kd.read_coordinator_descriptor(h(COORD["DESCRIPTOR_WITH_TARIFF_V0"]))
        self.assertTrue(d.charges)
        # The nested tariff decodes to exactly the standalone vector's value — i.e. key 6 carries a
        # nested MAP, not a bytes-wrapped blob.
        self.assertEqual(d.tariff, kd.read_tariff(h(COORD["TARIFF_V0"])))
        # …and it is NOT the descriptor's own identity (§18.8a.1: a client MUST attribute the
        # tariff to `Tariff.identity` and surface the distinction, §26.10).
        self.assertNotEqual(d.tariff.identity, d.identity)
        self.assertEqual(d.tariff.identity, h(COORD["IK_TARIFF"]))
        self.assertEqual(d.identity, h(COORD["IK_DESCRIPTOR"]))

    def test_the_descriptor_policy_blob_is_the_depot_policy_vector(self):
        """The cross-corpus seam, checked from outside both crates.

        `profiles/cloud.md` §3.3 says a `DepotServicePolicy` **is** the `policy` blob of an
        `infra-service` descriptor. The two corpora live in different crates; this asserts the
        bytes are literally the same and that a spec-derived reader parses them as the schema
        §3.3 names.
        """
        d = kd.read_coordinator_descriptor(h(COORD["DESCRIPTOR_V0"]))
        self.assertEqual(d.policy, h(DEPOT["POLICY_MINIMAL"]))
        p = kd.read_depot_service_policy(d.policy)
        self.assertEqual(p.service, "bucket")
        self.assertEqual(p.backing, "operator")


# ─────────────────────────────────────────────────────────────────────────────────────────────
# profiles/cloud.md — the DEPOT schemas
# ─────────────────────────────────────────────────────────────────────────────────────────────


class DepotVectors(unittest.TestCase):
    def test_service_policy(self):
        p = kd.read_depot_service_policy(h(DEPOT["POLICY_MINIMAL"]))
        self.assertEqual(p.service, "bucket")
        self.assertEqual(p.backing, "operator")
        self.assertIsNone(p.capacity, "absent capacity means UNDECLARED, never unlimited (§3.3)")
        self.assertEqual(p.attributes, ())
        self.assertEqual(p.abilities, ())

    def test_image(self):
        i = kd.read_depot_image(h(DEPOT["IMAGE_EDGE_FN"]))
        self.assertEqual(i.target, "edge-fn")
        self.assertEqual(i.format, "wasm")
        self.assertEqual(i.digest, bytes([0x1E]) + bytes([0xD1]) * 32)
        self.assertEqual(i.size_bytes, 4096)
        self.assertIsNone(i.arch)
        self.assertEqual(i.boot, ())
        self.assertIsNone(i.parent)

    def test_measurement(self):
        m = kd.read_depot_measurement(h(DEPOT["MEASUREMENT_UPTIME"]))
        self.assertEqual(m.service, "box")
        self.assertEqual(m.metric, "uptime")
        self.assertEqual(m.value, 995)
        self.assertNotIsInstance(m.value, bool, "uptime is uint per-mille, not a bool (§7)")
        self.assertEqual(m.method, "probe")
        self.assertEqual(m.observed_at, 1)
        self.assertIsNone(m.evidence)

    def test_site(self):
        s = kd.read_depot_site(h(DEPOT["SITE_MINIMAL"]))
        self.assertEqual(s.root, bytes([0x1E]) + bytes([0xD1]) * 32)
        self.assertIsNone(s.fallback)
        self.assertEqual(s.redirects, ())
        self.assertIsNone(s.cache_max_age_s)
        self.assertIsNone(s.cache_immutable)

    def test_formula(self):
        f = kd.read_depot_formula(h(DEPOT["FORMULA_REDIS"]))
        self.assertEqual(f.kind, "redis")
        self.assertEqual(len(f.parts), 1)
        self.assertEqual(f.parts[0].service, "box")
        self.assertEqual(f.parts[0].provider, bytes([0xAA]))
        self.assertIsNone(f.parts[0].descriptor)
        self.assertIsNone(f.recipe)
        self.assertIsNone(f.consensus)
        self.assertFalse(
            f.scales_horizontally,
            "absent `consensus` means single-writer; such a formula MUST NOT advertise "
            "horizontal scaling (§3.6)",
        )


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Re-encoding — the direction that catches structure loss
# ─────────────────────────────────────────────────────────────────────────────────────────────


ALL_WELLFORMED = {
    "DepotServicePolicy": ("depot", "POLICY_MINIMAL"),
    "DepotImage": ("depot", "IMAGE_EDGE_FN"),
    "DepotMeasurement": ("depot", "MEASUREMENT_UPTIME"),
    "DepotSite": ("depot", "SITE_MINIMAL"),
    "DepotFormula": ("depot", "FORMULA_REDIS"),
    "UsageReceipt": ("coord", "RECEIPT_V0"),
    "Tariff": ("coord", "TARIFF_V0"),
    "CoordinatorDescriptor": ("coord", "DESCRIPTOR_V0"),
    "CoordinatorDescriptor+Tariff": ("coord", "DESCRIPTOR_WITH_TARIFF_V0"),
}


def _vector(spec) -> bytes:
    source, name = spec
    return h((DEPOT if source == "depot" else COORD)[name])


class ReEncoding(unittest.TestCase):
    def test_every_vector_re_encodes_to_its_own_bytes(self):
        self.assertEqual(len(ALL_WELLFORMED), 9, "every well-formed vector, not a subset")
        for name, spec in ALL_WELLFORMED.items():
            with self.subTest(vector=name):
                raw = _vector(spec)
                self.assertEqual(kd.encode_canonical(kd.decode_canonical(raw)), raw)


# ─────────────────────────────────────────────────────────────────────────────────────────────
# The corruption controls, each paired with a false-positive control
# ─────────────────────────────────────────────────────────────────────────────────────────────


class CoordinatorCorruptionControls(unittest.TestCase):
    def test_pristine_vectors_decode(self):
        """The false-positive control: every rejection below is known to come from the mutation
        and not from some unrelated defect in this decoder."""
        kd.read_coordinator_descriptor(h(COORD["DESCRIPTOR_V0"]))
        kd.read_coordinator_descriptor(h(COORD["DESCRIPTOR_WITH_TARIFF_V0"]))
        kd.read_tariff(h(COORD["TARIFF_V0"]))
        kd.read_usage_receipt(h(COORD["RECEIPT_V0"]))

    def test_truncation_is_rejected(self):
        with self.assertRaises(kd.CborError):
            kd.read_coordinator_descriptor(h(COORD["DESCRIPTOR_V0"][:20]))
        with self.assertRaises(kd.CborError):
            kd.read_tariff(h(COORD["TARIFF_V0"][:20]))

    def test_empty_input_is_rejected(self):
        with self.assertRaises(kd.CborError):
            kd.read_coordinator_descriptor(b"")
        with self.assertRaises(kd.CborError):
            kd.read_usage_receipt(b"")

    def test_trailing_bytes_are_rejected(self):
        with self.assertRaises(kd.CborError) as ctx:
            kd.read_usage_receipt(h(COORD["RECEIPT_V0"] + "00"))
        self.assertIn("trailing", str(ctx.exception))

    def test_the_folded_compute_kind_is_rejected(self):
        with self.assertRaises(kd.SchemaError) as ctx:
            kd.read_coordinator_descriptor(h(COORD["DESCRIPTOR_KIND_COMPUTE"]))
        self.assertIn("compute", str(ctx.exception))

    def test_terminating_structural_is_undeclarable(self):
        with self.assertRaises(kd.SchemaError) as ctx:
            kd.read_coordinator_descriptor(h(COORD["DESCRIPTOR_TERMINATING_STRUCTURAL"]))
        self.assertIn("terminating", str(ctx.exception))

    def test_a_short_ik_pub_violates_18_2_length_governance(self):
        with self.assertRaises(kd.SchemaError) as ctx:
            kd.read_coordinator_descriptor(h(COORD["DESCRIPTOR_SHORT_IK"]))
        self.assertIn("31 B", str(ctx.exception))
        self.assertIn("§18.2", str(ctx.exception))

    def test_a_smuggled_score_field_is_rejected(self):
        """§18.8a.1: the descriptor has no field for a global score, a price rank, or a stake
        amount, and the unknown-key rule is what stops one being added back in."""
        with self.assertRaises(kd.SchemaError) as ctx:
            kd.read_coordinator_descriptor(h(COORD["DESCRIPTOR_SMUGGLED_SCORE"]))
        self.assertIn("[8]", str(ctx.exception))

    def test_a_bytes_wrapped_tariff_is_rejected(self):
        """Type confusion at key 6: an implementation that wrapped nested objects in `bytes` would
        round-trip perfectly against itself and interop with nobody."""
        mutated = COORD["DESCRIPTOR_WITH_TARIFF_V0"].replace("06" + COORD["TARIFF_V0"], "0641aa")
        self.assertNotEqual(
            mutated, COORD["DESCRIPTOR_WITH_TARIFF_V0"], "the substitution must actually match"
        )
        with self.assertRaises(kd.SchemaError):
            kd.read_coordinator_descriptor(h(mutated))


class DepotCorruptionControls(unittest.TestCase):
    def test_pristine_vectors_decode(self):
        kd.read_depot_service_policy(h(DEPOT["POLICY_MINIMAL"]))
        kd.read_depot_image(h(DEPOT["IMAGE_EDGE_FN"]))
        kd.read_depot_measurement(h(DEPOT["MEASUREMENT_UPTIME"]))
        kd.read_depot_site(h(DEPOT["SITE_MINIMAL"]))
        kd.read_depot_formula(h(DEPOT["FORMULA_REDIS"]))

    def test_truncation_and_empty_input_are_rejected(self):
        with self.assertRaises(kd.CborError):
            kd.read_depot_service_policy(h(DEPOT["POLICY_MINIMAL"][:10]))
        with self.assertRaises(kd.CborError):
            kd.read_depot_service_policy(b"")

    def test_a_required_key_removed_is_rejected(self):
        """`a1 01 66 "bucket"` — a genuine map(1) that loses `backing`, which is REQUIRED (§3.3)."""
        missing_backing = DEPOT_INLINE_CORRUPTIONS[0]
        self.assertEqual(missing_backing, "a101666275636b6574", "corpus drifted")
        with self.assertRaises(kd.SchemaError) as ctx:
            kd.read_depot_service_policy(h(missing_backing))
        self.assertIn("[2]", str(ctx.exception))

    def test_an_unknown_closed_registry_value_is_rejected(self):
        """`service = "vm"` in an otherwise well-formed object."""
        unknown_service = DEPOT_INLINE_CORRUPTIONS[1]
        with self.assertRaises(kd.SchemaError) as ctx:
            kd.read_depot_service_policy(h(unknown_service))
        self.assertIn("'vm'", str(ctx.exception))

    def test_a_wrong_value_type_is_rejected(self):
        """Image size (key 4) as `text(1) "x"` rather than a uint."""
        bad_size = DEPOT_INLINE_CORRUPTIONS[2]
        with self.assertRaises(kd.SchemaError) as ctx:
            kd.read_depot_image(h(bad_size))
        self.assertIn("expected uint", str(ctx.exception))
        # …and the same object with the size restored is accepted.
        kd.read_depot_image(h(DEPOT["IMAGE_EDGE_FN"]))


# ─────────────────────────────────────────────────────────────────────────────────────────────
# The canonical-CBOR rules themselves (§18.1.1), exercised against the real corpus bytes
# ─────────────────────────────────────────────────────────────────────────────────────────────


class CanonicalEncodingRules(unittest.TestCase):
    def test_map_keys_sort_by_encoded_bytes_not_lexicographically(self):
        """§18.1.1 rule 2 — the trap the `Tariff.schedule` blob exists to catch.

        A text key's head carries its length, so `0x62 "gb"` precedes `0x64 "byte"` even though
        `"byte" < "gb"` as a string. A naive `sort()` emits the other order.
        """
        schedule = kd.read_tariff(h(COORD["TARIFF_V0"])).schedule
        self.assertEqual(schedule, kd.encode_canonical({"gb": 3, "byte": 5}))
        naive_lexicographic = (
            b"\xa2"
            + kd.encode_canonical("byte")
            + kd.encode_canonical(5)
            + kd.encode_canonical("gb")
            + kd.encode_canonical(3)
        )
        self.assertNotEqual(naive_lexicographic, schedule)
        with self.assertRaises(kd.CborError) as ctx:
            kd.decode_canonical(naive_lexicographic)
        self.assertIn("canonical order", str(ctx.exception))

    def test_integer_keys_out_of_order_are_rejected(self):
        """The same rule on the integer-keyed objects §18.1.2 defines: swapping two entries of a
        real vector keeps every field intact and must still be refused."""
        swapped = b"\xa4" + b"".join(
            [
                kd.encode_canonical(2) + kd.encode_canonical(b"\xd1" * 32),
                kd.encode_canonical(1) + kd.encode_canonical(1),
                kd.encode_canonical(3) + kd.encode_canonical(bytes.fromhex("a10107")),
                kd.encode_canonical(4) + kd.encode_canonical(b"\x35" * 64),
            ]
        )
        with self.assertRaises(kd.CborError) as ctx:
            kd.read_usage_receipt(swapped)
        self.assertIn("canonical order", str(ctx.exception))

    def test_duplicate_keys_are_rejected(self):
        dup = b"\xa2" + (kd.encode_canonical(1) + kd.encode_canonical(1)) * 2
        with self.assertRaises(kd.CborError) as ctx:
            kd.decode_canonical(dup)
        self.assertIn("duplicate", str(ctx.exception))

    def test_indefinite_length_items_are_rejected(self):
        # An indefinite-length byte string carrying the receipt's operation blob.
        with self.assertRaises(kd.CborError) as ctx:
            kd.decode_canonical(b"\x5f\x43\xa1\x01\x07\xff")
        self.assertIn("indefinite", str(ctx.exception))

    def test_non_shortest_integer_encodings_are_rejected(self):
        """`DepotImage.bytes` = 4096 is `19 10 00`; the same value in the 4- and 8-byte forms is
        well-formed CBOR and non-canonical."""
        canonical = h(DEPOT["IMAGE_EDGE_FN"])
        self.assertIn(bytes.fromhex("04191000"), canonical)
        for wide in ("041a00001000", "041b0000000000001000"):
            with self.subTest(form=wide):
                mutated = canonical.replace(bytes.fromhex("04191000"), bytes.fromhex(wide))
                with self.assertRaises(kd.CborError) as ctx:
                    kd.read_depot_image(mutated)
                self.assertIn("non-shortest", str(ctx.exception))
        # A one-byte argument that fits the 5-bit immediate: `18 01` for suite = 1.
        with self.assertRaises(kd.CborError) as ctx:
            kd.read_usage_receipt(h(COORD["RECEIPT_V0"].replace("0101", "011801", 1)))
        self.assertIn("non-shortest", str(ctx.exception))

    def test_null_for_an_absent_optional_is_rejected(self):
        """§18.1.1: on the wire an absent optional field is omitted from the map, never present
        with a `null` value. Here: `DESCRIPTOR_V0` with `6 => null` spliced in before key 7."""
        src = COORD["DESCRIPTOR_V0"]
        mutated = "a7" + src[2:].replace("075840", "06f6075840", 1)
        with self.assertRaises(kd.CborError) as ctx:
            kd.read_coordinator_descriptor(h(mutated))
        self.assertIn("null", str(ctx.exception))

    def test_floats_tags_and_negative_integers_are_rejected(self):
        for name, blob in (
            ("float64", b"\xa1\x01\xfb\x40\x09\x21\xfb\x54\x44\x2d\x18"),
            ("tag", b"\xa1\x01\xc0\x61\x78"),
            ("negative int", b"\xa1\x01\x20"),
            ("undefined", b"\xa1\x01\xf7"),
        ):
            with self.subTest(item=name):
                with self.assertRaises(kd.CborError):
                    kd.decode_canonical(blob)


# ─────────────────────────────────────────────────────────────────────────────────────────────
# Where the spec and the corpus disagree
# ─────────────────────────────────────────────────────────────────────────────────────────────


class SpecCorpusDisagreements(unittest.TestCase):
    """Recorded, not silently tolerated and not papered over.

    A second implementation's most valuable output is the list of places the specification and the
    frozen corpus do not actually agree. Asserting the disagreements pins them: fixing either side
    fails this test, which is the point — the discrepancy cannot quietly change size.
    """

    def test_depot_hash_fields_now_honour_18_1_5(self):
        """RESOLVED — kept as a record, and as a regression guard.

        This decoder originally refused `DepotImage.digest` and `DepotSite.root`: both are typed
        `hash`, which §18.1.5 fixes at exactly 33 bytes (`0x1e` multicodec prefix followed by a
        32-byte BLAKE3-256 digest), and the frozen corpus carried a 3-byte placeholder. NOTHING IN
        RUST COULD SEE IT: the encoder, the decoder and the hand-written vectors all agreed on the
        placeholder, so every round-trip AND every foreign-byte test between the two Rust crates
        passed. It took a reader that had only ever seen §18.1.5.

        Both sides are now fixed — the corpus carries real 33-byte addresses and kotva-depot
        enforces the shape at every `hash`-typed decode site. This assertion is what stops it
        regressing, and the docstring is what stops the history being lost.
        """
        image = kd.read_depot_image(h(DEPOT["IMAGE_EDGE_FN"]))
        site = kd.read_depot_site(h(DEPOT["SITE_MINIMAL"]))
        for name, v in (("DepotImage.digest", image.digest), ("DepotSite.root", site.root)):
            self.assertEqual(len(v), 33, f"{name}: §18.1.5 fixes a v0 hash at 33 bytes")
            self.assertEqual(v[0], 0x1e, f"{name}: multicodec prefix for BLAKE3-256")
        self.assertEqual(list(image.deviations), [], "resolved, not merely moved")
        self.assertEqual(list(site.deviations), [], "resolved, not merely moved")

    def test_formula_provider_is_1_byte_and_no_suite_hook_can_govern_it(self):
        formula = kd.read_depot_formula(h(DEPOT["FORMULA_REDIS"]))
        self.assertEqual(len(formula.parts[0].provider), 1)
        self.assertEqual(len(formula.deviations), 1, formula.deviations)
        self.assertIn("no `suite` hook", formula.deviations[0])

    def test_the_coordinator_corpus_has_no_such_deviation(self):
        """The control that keeps the two findings above meaningful: §18.8a's objects carry a
        `suite` field, so their lengths ARE governed, and the corpus honours it exactly."""
        d = kd.read_coordinator_descriptor(h(COORD["DESCRIPTOR_WITH_TARIFF_V0"]))
        self.assertEqual(len(d.identity), 32)
        self.assertEqual(len(d.sig), 64)
        self.assertEqual(len(d.tariff.identity), 32)
        self.assertEqual(len(d.tariff.sig), 64)


if __name__ == "__main__":
    unittest.main()
