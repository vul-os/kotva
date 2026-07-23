<div align="center">

# KOTVA

### The sovereign substrate — the anchor everything is addressed to.

*kotva* — Slavic for **anchor**. The **key is the anchor**: your identity is a keypair you
hold, and every name, address, alias, or operator is a *swappable pointer* to it. Lose a
name, a domain, or a provider — the anchor holds.

</div>

---

KOTVA is a **narrow-waist substrate** for a decentralized world: a small, shared core —
identity, signed objects, transport, publish/subscribe, sync, and infrastructure roles —
over which many independent protocols compose as thin **profiles**. Mail, commerce, work,
social, media, and real-time calling are profiles of the *same* waist, not separate stacks.

## The one rule

> **Decentralize the substrate and the exit. Every unavoidable coordinator — gateway,
> relay, indexer, labeler, arbiter, oracle — is _accountable, swappable, and
> self-hostable_, and _never load-bearing_. Coordinators add reach; they never gate
> function.**

This is DMTAP's legacy-mail-gateway model — one accountable operator class, swappable via a
DNS change, with a self-host backstop — generalized from mail to the whole system. See
**[DIRECTION.md](DIRECTION.md)** for the full principles and
**[coordinator/CONTRACT.md](coordinator/CONTRACT.md)** for the rules every coordinator must honor.

## Layout

| Path | What it is |
|------|------------|
| **`substrate/`** | The narrow waist: Identity, MOTE (the signed object), Transport, PUB (public objects), SYNC (multi-author CRDT), Roles & Wake. |
| **`§00`–`§27`** (root) | **DMTAP** — the mail + messaging + gateway profile, the first and reference profile of the waist. *(To be relocated under `profiles/dmtap-mail/` in a later restructure.)* |
| **`profiles/tract/`** | **TRACT** — the commerce profile (offer, cart, order, settlement, dispute). |
| **`profiles/wrap/`** | **WRAP** — the work profile (jobs, dispatch, delivery, milestone escrow). |
| **`coordinator/`** | The coordinator contract — accountable / swappable / self-hostable / TEE-attestable, plus the content-visibility property. |
| **`bindings/`** | What KOTVA **adopts** rather than reinvents — identity, reputation, payments, storage, dispute, media transport. See [bindings/README.md](bindings/README.md). |
| **`conformance/`** | Conformance suite and test vectors. |
| **`docs/research/`** | The research and rationale behind the design. |

## Design in one breath

Identity is a key (recovery is [adopted](bindings/README.md), not invented). Everything is a
**MOTE** — a signed, encrypted, content-addressed object. The **async world** (mail, chat,
social, commerce, files) composes from MOTEs + PUB + SYNC; the **real-time world** (calls,
live video) rides a parallel media plane that reuses the *same* identity and keys. Money is
an existing stablecoin, **never a new token**; trust is *staked existing value*, never a mint.
Where a job genuinely needs a coordinator — matching, search, moderation, legal
accountability — that coordinator is a **hireable, fireable, self-hostable role**, the same
shape as DMTAP's mail gateway, generalized.

## Profiles compose primitives

Every service is the same handful of primitives rearranged — which is why "different
products" collapse into one substrate:

```
OFFER · MATCH/RESERVE · REPUTATION · ESCROW · ORACLE · DISPUTE · PAY
```

Uber, delivery, bookings, auctions, freelance, and classifieds are all this set with a
different **assignment rule** on MATCH (nearest / highest-bid / best-fit) — one engine, not
six. See [DIRECTION.md](DIRECTION.md).

## Status

`v0.1.0` — early and evolving. The substrate, the DMTAP mail profile, and the TRACT/WRAP
profiles are specified; the **coordinator contract** and **bindings index** are the active
consolidation work. Far-future cryptographic research (mixnet, VDF, PQ envelope tuning) is
being quarantined to `research/` as **non-normative** rather than perfected on the critical
path. See [ROADMAP.md](ROADMAP.md).

## License & governance

Spec license — see [LICENSE.md](LICENSE.md) and [GOVERNANCE.md](GOVERNANCE.md). Copyright
held by VulOS (github.com/vul-os). **No protocol token exists and none will be added.**
