# 11. Jurisdiction

> **Drafting status.** This section is scoped but not yet normative. It states what it will
> specify, which existing standards it profiles, and the decisions still open. Nothing here is
> implementable yet; text becomes normative when the RFC 2119 keywords appear.

## 11.1 Scope

Making the legally responsible party explicit, and getting tax anchors right by construction.

## 11.2 The four anchors

The most common commerce-tax error is conflating party location with where a supply happens. These
are four distinct fields, derived from four different places:

| Anchor | Derived from | Governs |
|---|---|---|
| seller establishment | seller identity | licensing, seller-side registration |
| buyer residence | buyer disclosure at order | consumer-protection rights (generally non-waivable) |
| **place of supply** | **the Fulfilment axis (§4)** | VAT/GST, especially services and events |
| delivery destination | the shipping leg (§8) | customs, duty, product-safety regimes |

The forcing example: an event held in one country, sold by a seller in another, to a buyer in a
third. Admission to events is generally taxed where the event physically takes place, so only the
Fulfilment object knows the answer.

## 11.3 What this section will specify

- **Responsibility follows the money**: every order names seller of record, facilitator (the
  gateway, if it settled), importer of record, in-region responsible person, and escrow/rail.
- Geo-availability on offers, and fail-closed construction when a required in-region role is absent.
- Tax treatment categories (not rates — see §5.5).

## 11.4 Regimes this section must accommodate

South Africa (electronic-transaction disclosure and cooling-off, consumer protection, POPIA, VAT,
payment-side KYC); the EU (GDPR, platform trader traceability, consumer rights, in-region
responsible person for product safety, VAT one-stop schemes, platform reporting); other African
markets (national data-protection acts, local VAT registration, regional trade frameworks); New
Zealand (privacy, fair trading, consumer guarantees, GST on low-value imports); the Americas (US
economic nexus and marketplace-facilitator rules, seller-traceability legislation, Canadian and
Brazilian privacy law).

The protocol guarantees the **facts** a regulator asks for are present, signed and attributable.
It does not make any deployment compliant, and must not be read as legal advice.

## 11.5 The erasure conflict, resolved structurally

Published objects are irrevocable; erasure rights cannot be satisfied against them. Therefore **no
personal data enters the public quadrant** (§0.5.1). Orders and everything identifying a person are
sealed and deletable at the edges. Reviews are the single bounded exception (§10.4).
