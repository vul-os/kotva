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

**Draft.** Every section is written; none is a placeholder.

**§16 (wire format) is normative and frozen at v0** — it defines bytes, so a change to it is a MAJOR
version change rather than a correction. That freeze is what lets conformance vectors exist at all:
a second implementation cannot prove it matches *this document* rather than matching the reference
implementation until there is something fixed to match.

Everything else carries a drafting banner and is not yet normative. A section becomes normative when
the RFC 2119 keywords appear in capitals, and `make lint` enforces that a banner-carrying section
cannot quietly acquire them — a check that has caught real drift twice.

## Repository

```
00-overview.md        architecture, roles, the one operator class, public/sealed split
01–20                 the numbered sections (see §0.8 for the map)
21-grounding.md       what the evidence supports, and what it contradicts
build/                markdown -> single HTML -> PDF via headless Chrome. No LaTeX.
tools/lint.py         internal-consistency checks; belongs in CI
conformance/          planned case catalogue + vector discipline; zero vectors until §16 is normative
```

```sh
make lint          # internal consistency
make lint-strict   # warnings fail too; use before a release tag
make pdf           # first run: (cd build && npm install)
```

## Help wanted

Several things in this specification are blocked on **expertise this project does not have**, and
in some cases multiple research passes returned nothing verifiable. Those are recorded as
unevidenced rather than quietly asserted — see [docs/HELP-WANTED.md](docs/HELP-WANTED.md) for the
specific questions and what kind of person can answer each.

The short version:

| | |
|---|---|
| **A second implementation** | the single most valuable contribution. §16 is frozen and has 21 conformance vectors, and exactly one implementation — which proves nothing about whether the *document* is buildable from |
| **Data-protection law** | three passes, nothing verified. Needs a practitioner, not another literature review (§22) |
| **EU VAT** | decided on an engineering reading, explicitly pending advice (D1) |
| **Logistics, analytics, reputation** | §8, §13 and §10 are unevidenced and marked as such |
| **Adversarial review** | the two most valuable contributions so far both came from trying to break something |

If your finding is that something here is **wrong**, that is the most welcome kind. §21 exists
because the evidence contradicted the design.

## Implementation

[**Soko**](https://github.com/vul-os/soko) is the reference implementation — Rust, MIT, with a
crate per section of this document and tests concentrated on the failures that are silent rather
than loud: volumetric weight, currency mismatch in a route total, escrow scope intersection, place
of supply for an event held abroad, concurrent replicas not overselling.

It is **not the standard** and is not required to speak it: independent implementations must be
buildable from this specification alone, without reading any code. Where Soko and this document
disagree, **this document wins** — and if this document turns out to be unimplementable, that is a
defect here, not there.

## Licence

CC BY 4.0 — implement, quote and build on it freely, with attribution. Reference implementations
are licensed separately under MIT.
