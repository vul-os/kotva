# Changelog

All notable changes to the DMTAP specification are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added

- **Conformance: 22 cases for the new normative requirements** (172 → **194**), closing the gap the
  hardening and one-binary-with-roles commits left open — a MUST with no case is unenforceable, and
  §10.3 makes the suite the operational definition of compatibility. New families: `MIXPROF`
  (§4.4.10a Bootstrap-profile anti-drift constraints), `FLEET` (§4.4.2 derived fleet view), `GUARD`
  (§4.4.8 persistent guard sample + ASN/attested-operator diversity), `LOC` (§4.2 per-epoch
  `peer_id`, §4.2.1 resolution order), `FLOOR` (§9.7a zero-relationship delivery floor, §9.4.1
  memory-hard-PoW floor), `FAILCLASS` (§10.7.0 failure classes) and `GWROLE` (§7.11.4/§9.11
  authorize-never-classify, §7.1b privilege separation). Partition: 46 vectored + 6 self-contained
  + 137 construction-todo + 5 manual-attestation.
- **§21.10 `0x070F` `ERR_POLICY_BELOW_FLOOR`** — referenced by §9.7a since the hardening pass but
  never allocated. The one code in the anti-abuse block whose fault is the recipient's *own* policy
  (`N_floor = 0`, or a VDF-only cold-contact requirement) rather than an inbound object. Registry:
  140 → 141 codes.

### Changed

- **Bucket ladder floor 8 KiB → 16 KiB; inline attachment cap 64 KiB → 48 KiB** (§4.4.1, §16.3,
  §16.4, §2.5, §5.5.1). The 8 KiB floor was arithmetically unsound: it was sized against *one*
  ML-DSA-65 signature and *one* public key, but a MOTE carries **two** of each (`Envelope.sender_sig`
  + `Payload.sig`, `sender_key` + `Payload.from`) plus the X-Wing encapsulated key, so the minimum
  conformant suite-`0x02` MOTE is **11 967 B** with an empty body — 3 775 B *over* the rung it was
  supposed to fit in. §4.4.1 now states the byte arithmetic explicitly, from the §18.2 lengths, so
  it cannot drift again. Two rungs are kept (a third would take the per-message size leak a pinned
  guard observes from 1 bit to log₂3 ≈ 1.58); anchor-suite (`0x04`) announcements are ordinary
  **top-rung** MOTEs at ≈ 26 kB and are *not* excluded from the inline path. The inline attachment
  cap follows: 64 KiB top rung − 11 967 B envelope ⇒ 48 KiB of content.
- **VDF demoted SHOULD → MAY** (§9.4.1, §16.5, `DMTAP-FLOOR-03`). Memory-hard PoW remains the
  interoperable MUST floor and VDF-only remains non-conformant, both unchanged. Three disclosures
  added: sequentiality is a **conjecture** defined only *relatively*, against a `p(t)`-processor
  bound (the foundational definition permits `Eval` up to poly log(t) parallelism); a VDF bounds
  **aggregate parallelism only**, leaving a **10–100×** per-gate latency advantage; and it is
  **not post-quantum** — a quantum adversary computes the group order and collapses the delay.
  The asymmetry that makes this tolerable is stated rather than hidden: a broken VDF is a *future
  spam-cost* problem, repairable locally, not a retroactive confidentiality loss like a broken KEM.
- **X-Wing's standing described accurately** (§1.3, §11.1, §11.3, §15, §16.7, README). It is
  `draft-connolly-cfrg-xwing-kem-10` on the **Independent Submission** stream, **not CFRG-adopted**,
  and **FIPS 203 standardizes no combiner**, warning that a combined KEM containing ML-KEM "might
  not meet IND-CCA2 security" and deferring to SP 800-227. Still pinned — on analysis and a fixed
  HPKE code point, not on standing. `draft-yun-privacypass-arc` likewise relabelled an individual
  draft rather than WG work (§9.3, §11.1).
- **Hybrid signatures: composite message representative, and the exact assurance level** (§1.3,
  §18.1.6, §10.7.1). AND-composition stands, but the components do **not** independently sign the
  object preimage: following the IETF LAMPS composite PQ/T construction both sign
  `M' = DS-tag ‖ 0x00 ‖ suite ‖ body`, which is what makes a component non-separable from the
  composite. Assurance stated as **EUF-CMA, not SUF-CMA** — no composite variant achieves strong
  unforgeability against a quantum adversary — with the note that DMTAP derives no identifier from
  a signature (`Envelope.id` is the content address of `ciphertext`), so it never needed it.
  Suite `0x01` signing is unchanged; the frozen vectors are all `0x01` and are byte-identical.
- **§16.7 gains the `0x04` row and §18.2 the `0x04` lengths** (`sig-val` 7 920 B, `ik-pub` 64 B) —
  the anchor suite was normative in §1.1/§1.2.0 but absent from both length registries, which is
  where the ladder arithmetic reads its numbers from.
- **§4.4.2a's growth argument labelled a design bet**, not a result: volunteer take-up of the mix
  role at scale is unmeasured, and §4.4.10a/§11.3 are what make being wrong about it survivable.

### Fixed

- The **class-group immaturity argument is removed** from §9.4.1/§16.5. It did not survive
  scrutiny — 2018/2019 silence on class-group performance is not evidence about 2026 — and the
  trusted-setup objection is weaker than its strong form (the literature offers a sufficiently
  large random `N`, at a disclosed cost, and class groups). What keeps a VDF out of the floor is
  the absence of a standard, an interoperable parameter set and a pinned proof encoding.
- `0x0311` (`ERR_MIX_DIRECTORY_STALE`) is **FAIL-QUEUED** per §10.7.0/§10.7.2, not
  `FAIL_CLOSED_BLOCK` — the registry still carried the pre-reclassification disposition, which is
  the exact "liveness failure handed a denial-of-service surface" error §10.7.0 exists to forbid.
- `0x030D` (`ERR_MIX_PATH_UNBUILDABLE`) now names the diversity-unmet case, not only the
  empty-layer one, and is scoped to the in-force profile's bar.
- Catalog rows that outlived their clauses: `DMTAP-PRIV-01` still declared the `{2,8,32,64}` KiB
  bucket ladder (cut to `{8,64}`), `DMTAP-PRIV-02` and the §21.12 condition matrix still spoke of a
  mix "directory authority" (deleted — the fleet view is derived).
- `conformance/README.md` stated 157 cases / 104 construction-todo, two waves behind.

## [0.1.0] — 2026-07-21

First versioned cut of the DMTAP specification — sovereign, end-to-end-encrypted, metadata-private mail/chat/files/identity over a peer-to-peer mesh. 22 numbered sections plus conformance vectors. Spec text is CC BY 4.0.
