# 10. Fulfilment

## 10.1. The problem

"Delivered", signed only by the performer who was paid to deliver it, is worth
nothing in a dispute. The issuer was not present at the handoff and the
beneficiary — the customer receiving the parcel, the tenant whose tap was
fixed — usually holds no key and never will.

WRAP needs a proof that a party who was *not there* can check afterwards,
without requiring the beneficiary to run software.

## 10.2. Handoff codes

The RECOMMENDED mechanism is a **handoff code**: a secret known to the issuer
and the beneficiary, presented at the moment of handoff and included in the
completing attestation.

The commitment below is published *before* the handoff, on the wire, in an
object the assigned performer holds — so the code itself MUST carry at least
128 bits of entropy. A 4-6 digit code is not sufficient: its whole space is
exhaustively searchable offline against the published commitment in
microseconds, which is precisely what the performer is not supposed to be
able to do. Choosing a longer digit string does not fix this; choosing a
high-entropy code does.

1. On assignment, the issuer generates a code — a randomly generated token
   of at least 128 bits, rendered as e.g. a QR code or a short link — and
   delivers it to the beneficiary out of band (printed on the receipt, sent
   by message, scanned from a QR code).
2. The issuer includes `commit = BLAKE3-256(code ‖ order_id)` in the
   `Assignment` (§3.6, `commit`), so the commitment is published *before* the
   handoff and cannot be constructed afterwards to match whatever was typed.
3. At handoff, the beneficiary shows or reads the code to the performer.
4. The performer includes the code in the `proof` map of their completing
   `Attestation` (§3.8).
5. Any party recomputes the hash and checks it against the commitment.

The performer cannot recover the code by brute force, because 128 bits of
entropy makes offline exhaustive search infeasible; they learn it only from
the beneficiary at handoff. The issuer cannot deny a completion that
happened, because they published the commitment beforehand and the code
verifies against it. A short, spoken digit code protects only against issuer
denial — it does **not** protect against performer fabrication, because the
performer already holds the published commitment and can search a
four-to-six-digit space offline; implementations MUST NOT describe a
low-entropy code as resisting fabrication.

This requires no app and no key from the beneficiary, but it does need a
channel that can carry more than a few spoken digits — a QR code, a link, or
a printed string. A phone-only handoff with no such channel MAY fall back to
a short spoken code, but implementations MUST disclose that the fallback is
a courtesy to the beneficiary only and does not defeat a performer willing to
brute-force the published commitment offline.

## 10.3. Proof map

| Key | Name | Type | Notes |
|---|---|---|---|
| 1 | `kind` | uint | `0` code, `1` signature, `2` photo, `3` geo, `4` none |
| 2 | `code` | tstr | The handoff code, for `kind = 0` |
| 3 | `sig` | bstr | Beneficiary signature, for `kind = 1` |
| 4 | `ref` | tstr | Content address or URL of a photo, for `kind = 2` |
| 5 | `at` | map | `Place` (§3.9), for `kind = 3` |
| 6 | `note` | tstr | Why proof is absent, for `kind = 4` |

## 10.4. Weaker proofs

**Beneficiary signature (`kind = 1`)** is the strongest available and the least
practical: it requires the beneficiary to hold a key. Use it where the
beneficiary is itself a business — a restaurant receiving a supplier delivery,
a landlord accepting completed work.

**Photo (`kind = 2`)** proves something was photographed, not that it was
delivered to the right person. It is evidence, not proof, and implementations
MUST NOT present it as verification.

**Geolocation (`kind = 3`)** proves a device reported coordinates. Device
location is trivially falsified on both major mobile platforms.
Implementations MUST NOT treat geolocation as proof of anything, and SHOULD NOT
present it to users as verification. It is useful for operations — routing,
ETAs, dispute context — and useless for adjudication.

**None (`kind = 4`)** is legitimate and MUST be supported. Much real work has
no verifiable handoff: a bin is emptied, a garden is mowed, nobody is home. The
honest encoding of "we have no proof" is `kind = 4` with a note, not a weaker
proof dressed up as a strong one.

## 10.5. Partial and multi-visit work

Trades work frequently completes across several visits, and a first visit may
be a quotation rather than work at all. Implementations MUST NOT assume one
assignment means one visit.

Each visit is a `Progress` object (§3.7). Only the final one carries the
terminal state, and only then is a completing `Attestation` appropriate. A work
order requiring a return visit stays `in_progress` between them; this is the
normal case for trades and needs no special handling.

Where a first visit produces a revised price, the performer records it as a
`Progress` object carrying the new terms in `body`, and the issuer accepts by
issuing a fresh `Assignment` with updated `terms`. Neither party can silently
change agreed compensation, because both objects are signed and both remain in
the record.
