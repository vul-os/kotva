# 16. References

## 16.1. Normative

| Reference | Used for |
|---|---|
| RFC 2119 / RFC 8174 (BCP 14) | Requirements language |
| RFC 8032 | Ed25519 signatures (§5) |
| RFC 8949 | CBOR, including deterministic encoding §4.2.1 (§4) |
| BLAKE3 | Object identifiers, commitments, state roots (§4.3, §7.4, §10.2) |
| RFC 9052 | COSE_Sign1, OPTIONAL alternative envelope (§5.3) |
| RFC 6962 | Hash-chained log structure for attestation feeds (§9.5) |

## 16.2. Informative

**OpenCourier** — *An Open Protocol for Building a Decentralized Ecosystem of
Community-owned Delivery Platforms*, CHI 2026 Extended Abstracts.
`arxiv.org/abs/2511.02455`

The closest prior art, and the source of the delivery vocabulary in §12.2.
OpenCourier proposes a three-layer architecture — registry, instance-app,
instance-requester — for worker-owned delivery platforms, and its framing of
worker ownership as the goal rather than decentralization for its own sake
directly informed §9.2 and §8.1.

It differs from WRAP in two ways that matter. It is a position paper with
embedded protocol sketches rather than a specification: there is no formal wire
format, no public reference repository, and no defined trust or
reputation-portability mechanism. And its architecture is always-online
federated REST instances, which cannot make progress during a partition — the
constraint that drives most of WRAP's design (§1.4). The two are complementary
rather than competing; a bridge would be a mapping, not a translation.

**OpenTripModel** — lightweight logistics trip vocabulary. `opentripmodel.org`
Informative for `Place` and `Window` shaping (§3.9, §3.10).

**GS1 EPCIS** — supply-chain event capture (what / when / where / why).
Informative for the append-only event model of `Progress` (§3.7). EPCIS is far
heavier than WRAP needs and assumes centralized capture.

**OASIS UBL** — Universal Business Language, including Despatch Advice and
Waybill. Informative for `Compensation` (§3.11). XML and enterprise-oriented;
adopted for vocabulary only.

**W3C ActivityStreams 2.0** — the `Offer` / `Accept` / `Reject` verb vocabulary
parallels §3 closely. Not adopted: ActivityStreams has no merge semantics and
no offline story, both of which are the substance of WRAP.

**DMTAP** — Decentralized Message Transfer & Access Protocol, and its substrate
capabilities (Identity, Feeds & Blobs, Sync, Roles, Wake). `draft-dmtap-01`

Source of the 8-word key name convention (§2.3), the CRDT sync algebra
underlying binding B (§11.2), and the append-only feed structure recommended
for attestations (§9.5). WRAP depends on none of it: DMTAP is a binding WRAP
happens to fit, not a substrate WRAP requires. The `0x1e` multihash prefix,
domain-separated hashing, and the reserved-forbidden-key technique of §4.5 are
adopted from its public-objects profile.

**FlowStock** — inventory and sales across branches, offline-first.
`github.com/vul-os/flowstock`

The HLC stamp format (§7.2), stateless symmetric sync rounds, TOFU pairing
(§11.1.3), and the file-exchange transport (§11.1.4) are taken from its working
implementation rather than designed here. Its treatment of quantities as an
append-only movement log summed at read time — rather than a stored counter —
is the pattern that makes concurrent offline work converge correctly, and it is
the direct ancestor of §7.1.

## 16.3. Acknowledgements

The problem framing owes a debt to the platform cooperativism literature,
particularly the argument that the defect of gig platforms is ownership rather
than technology. WRAP takes that seriously enough to try to make the technology
stop being an obstacle to it.
