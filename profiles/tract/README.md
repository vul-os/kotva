<div align="center">

# TRACT

### Trade, Routing, Attestation, Custody & Trust

**An open protocol for decentralized commerce.** Goods, services, rentals and subscriptions between
self-sovereign identities — with no marketplace operator, no registrar, and no token.

*A keypair is a store. A cart is the buyer's. Delivery is computed, not brokered.*

</div>

---

## The one idea

A commerce platform welds together four things that need not be joined: **who you are** (an account
it issues), **what you sell** (rows in its database), **who can find you** (its ranking), and **how
you get paid** (its payment relationship). Lose the account, lose all four.

TRACT separates them:

| | Platform | TRACT |
|---|---|---|
| Identity | an account it issues | a keypair you hold |
| Catalogue | rows in its database | a signed feed you publish |
| Discovery | its ranking | a derived index anyone may rebuild |
| Settlement | its payment relationship | a seam, chosen per order by the parties |

## One shape for every trade

Every offer declares four axes. Four small closed sets, combined, express a haircut booking, a
scaffolding hire, a made-to-measure suit, a metered API, and a tin of beans — with no
category-specific code path for any of them.

| Axis | Variants |
|---|---|
| **Item** | product · variant-of-group · service · right/licence · capacity |
| **Availability** | count · time slots (RFC 5545) · capacity per interval · unlimited · made-to-order |
| **Fulfilment** | ship · collect · digital grant · perform-at-place · perform-remote · access grant · return-required |
| **Consideration** | fixed · tiered · recurring · metered · deposit+balance · quote/RFQ |

## Built on the DMTAP substrate

TRACT is not a new stack. It adopts [DMTAP](https://github.com/vul-os/dmtap)'s five substrate
capabilities — Identity, Feeds & Blobs, Sync, Infrastructure Roles, Wake — under that directory's
à-la-carte rule, and adds only the commerce spine. **No new cryptography, no new hash construction,
no new signature framing.**

## Read the evidence first

[§21 Grounding](21-grounding.md) records a July 2026 adversarially-verified literature pass,
**including the findings that contradict this specification**:

- **OpenBazaar** — the closest deployed relative of this design — shut down in 2021 having moved
  ~US$86k over 14 months, with ~80 users online at a time and one vendor faking 60% of measured
  sales value. Discovery re-centralized first; catalogues vanished when nodes went offline; opt-in
  escrow was declined by exactly the actors it existed to constrain.
- **Beckn/ONDC**, the largest live decentralized-commerce network, avoids all of that by adopting a
  central approval-gating registry with DNS/TLS identity — the opposite of this design, at the
  precise points this design is weakest.
- **There is no deployed permissionless global product identity.** The space between GS1's licensed
  monopoly namespace and a nominal merchant string is currently evidence-free.

A specification that omitted this would be easier to believe and worse to build on.

## Status

**Draft, pre-normative.** §0 is written; §1–§20 are scoped stubs that state what they will specify,
which standards they profile, and what is still open. A section becomes normative when the RFC 2119
keywords appear in capitals — `make lint` enforces that a stub cannot quietly claim authority.

## Repository

```
00-overview.md        architecture, roles, the one operator class, public/sealed split
01–20                 the numbered sections (see §0.8 for the map)
21-grounding.md       what the evidence supports, and what it contradicts
build/                markdown -> single HTML -> PDF via headless Chrome. No LaTeX.
tools/lint.py         internal-consistency checks; belongs in CI
conformance/          test vectors (planned)
```

```sh
make lint          # internal consistency
make lint-strict   # warnings fail too; use before a release tag
make pdf           # first run: (cd build && npm install)
```

## Implementation

[**Soko**](https://github.com/vul-os/soko) is the reference implementation. It is not the standard
and is not required to speak it: independent implementations must be buildable from this
specification alone. Where Soko and this document disagree, **this document wins**.

## Licence

CC BY 4.0 — implement, quote and build on it freely, with attribution. Reference implementations
are licensed separately under MIT.
