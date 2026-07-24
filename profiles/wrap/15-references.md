# 16. References

## 16.1. Normative

The **DMTAP substrate** is WRAP's primary normative dependency: WRAP adopts its
Identity, Feeds & Blobs, Sync, and Roles capabilities and the RFCs each profiles,
rather than referencing those RFCs directly (§1.2). The individual RFCs below are
listed for the reader's benefit; the binding statement of how WRAP uses them is the
substrate document cited.

| Reference | Used for |
|---|---|
| RFC 2119 / RFC 8174 (BCP 14) | Requirements language |
| **DMTAP substrate — [`README`](https://github.com/vul-os/dmtap/blob/main/substrate/README.md)** | Adoption rules; the four adopted capabilities (of the substrate's six) and their shared primitives (§1.2, §4, §5, §7) |
| **DMTAP substrate — [`IDENTITY`](https://github.com/vul-os/dmtap/blob/main/substrate/IDENTITY.md)** | Principal = `IK`, `DeviceCert`, key-name, rotation/recovery (§2) |
| **DMTAP substrate — [`SYNC`](https://github.com/vul-os/dmtap/blob/main/substrate/SYNC.md)** | HLC, CRDT op algebra, reconciliation wire, snapshots (§7, §11) |
| **DMTAP substrate — [`FEEDS`](https://github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md)** | Content addressing, author feeds for attestations (§4.3, §7, §9) |
| **DMTAP substrate — [`ROLES`](https://github.com/vul-os/dmtap/blob/main/substrate/ROLES.md)** | Announce/resolve, relay, mailbox for pools (§8, §11) |
| RFC 8032 | Ed25519 — via the substrate's Identity/signing primitive (§5) |
| RFC 8949 | Deterministic CBOR §4.2 — via the substrate's encoding primitive (§4) |
| BLAKE3 | `0x1e`-prefixed content addresses — via the substrate (§4.3) |
| RFC 9052 | `COSE_Sign1` — via the substrate's signing primitive (§5.3) |
| RFC 6962 | Merkle-log structure — via the substrate's Feeds anti-rollback (§9) |

## 16.2. Informative

**OpenCourier** — *An Open Protocol for Building a Decentralised Ecosystem of
Community-owned Delivery Platforms*, CHI 2026 Extended Abstracts.
`arxiv.org/abs/2511.02455`

The closest prior art, and the source of the delivery vocabulary in §12.2.
OpenCourier proposes a three-layer architecture — registry, instance-app,
instance-requester — for worker-owned delivery platforms, and its framing of
worker ownership as the goal rather than decentralisation for its own sake
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
heavier than WRAP needs and assumes centralised capture.

**OASIS UBL** — Universal Business Language, including Despatch Advice and
Waybill. Informative for `Compensation` (§3.11). XML and enterprise-oriented;
adopted for vocabulary only.

**W3C ActivityStreams 2.0** — the `Offer` / `Accept` / `Reject` verb vocabulary
parallels §3 closely. Not adopted: ActivityStreams has no merge semantics and
no offline story, both of which are the substance of WRAP.

**FlowStock** — inventory and sales across branches, offline-first.
`github.com/vul-os/flowstock`

The substrate's proof-of-life for the Sync capability WRAP rides: its stateless
symmetric sync rounds and file-exchange transport are the concrete shape WRAP's
§11 inherits through the substrate. Its treatment of quantities as an append-only
movement log summed at read time — rather than a stored counter — is the same
add-only pattern WRAP's OR-Set objects use (§7).

*(DMTAP is no longer listed here as informative prior art — it is WRAP's normative
foundation; see §16.1.)*

## 16.3. Acknowledgements

The problem framing owes a debt to the platform cooperativism literature,
particularly the argument that the defect of gig platforms is ownership rather
than technology. WRAP takes that seriously enough to try to make the technology
stop being an obstacle to it.
