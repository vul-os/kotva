# 15. Conformance

## 15.1. Vectors are normative

The conformance vectors in `conformance/wrap_vectors.json` are **normative and
take precedence over the prose of this document** wherever the two disagree.

This inversion is deliberate. Prose cannot tell an implementer that they broke
a tie the wrong way, sorted a map incorrectly, or omitted a domain-separation
tag — all three produce code that passes every test the author thought to
write, interoperates with itself perfectly, and fails silently against a second
implementation. Vectors catch exactly this class of error, and they are the
difference between protocols that acquire independent implementations and
protocols that acquire admirers.

A claim of WRAP v0 conformance means: all vectors pass, with no skips. An
implementation that skips vectors MUST say which and why.

## 15.2. What the substrate already fixes, and must not be re-vectored here

Because WRAP objects are substrate Sync ops, Feed entries, and content objects
(§7), the byte-level behaviour they share with every other substrate product is
fixed by the **substrate's** conformance vectors, not WRAP's: deterministic-CBOR
encoding, the `0x1e ‖ BLAKE3-256` content address, `COSE_Sign1` + DS-tag signing,
the HLC and its `(wall, counter, author)` tie-break, OR-Set / LWW merge
convergence, and the state root ([`SYNC.md`](https://github.com/vul-os/dmtap/blob/main/substrate/SYNC.md)
§10; [`FEEDS.md`](https://github.com/vul-os/dmtap/blob/main/substrate/FEEDS.md) §4.3).
A WRAP implementation adopts a conformant substrate engine and inherits these; it
**MUST NOT** ship a parallel WRAP vector that re-asserts them, because two vector
corpora for one behaviour drift, and the substrate's is authoritative (adoption
rule 4).

## 15.3. Required WRAP coverage

WRAP's own vectors (`conformance/wrap_vectors.json`) fix only what is WRAP's — the
work spine on top of the substrate:

| Group | What it fixes |
|---|---|
| `map` | Each object → substrate primitive and address (§7.2): `Offer`/`Bid`/`Progress` → OR-Set at the right `target`; `Assignment` → LWW register; `WorkOrder` → immutable object; `Attestation` → feed entry |
| `authorship` | The §5.5 admission table: `Assignment` by a non-issuer is *inadmissible* (never enters the LWW register); `Bid` by any admitted principal accepted |
| `withdraw` | Bid withdrawal is the OR-Set observed-remove (§7.2), not a `withdrawn` flag; a withdraw cancels only its own add |
| `fold` | Lifecycle state computed from a work order's op set, including unreachable-state discard (§6.3) |
| `expiry` | Computed expiry with no message exchanged (§6.2) |
| `retain` | Attestations survive compaction of a terminal work order (§7.3) |
| `forward` | Unknown WRAP kind ignored; unknown field preserved through re-encode; unknown profile stored (§4.4) |
| `proof` | Handoff commitment verifies; wrong code fails (§10) |

## 15.4. The cases most likely to be got wrong

Implementers SHOULD run these first.

1. **`Assignment` admission, not merge (§5.5).** A non-issuer assignment must be
   refused at admission, never allowed into the LWW register to "lose" on HLC —
   admission is what makes the single-writer register safe.
2. **Bid withdrawal as observed-remove (§7.2).** A withdraw must cancel exactly the
   add-tags of the bid it retracts and nothing else; getting this wrong either
   resurrects withdrawn bids or clobbers a concurrent bidder.
3. **Preserving unknown fields through a re-encode (§4.4).** Dropping them
   invalidates the signature for everyone downstream, and the implementation that
   dropped them will not be the one that reports the failure.
4. **Pending rather than rejecting unknown referents (§5.6).** Rejecting makes
   out-of-order delivery lossy, and out-of-order delivery is normal.
5. **Computed expiry (§6.2).** Implementations that wait for an expiry message will
   hold work orders forever when a peer is unreachable.

## 15.5. Interoperability

Two implementations are interoperable at v0 if, riding the same substrate engine
and given the same op set delivered in any order, they compute the same lifecycle
state (§6.3) — and, because both derive from one substrate replica, the same
substrate **state root** ([`SYNC.md`](https://github.com/vul-os/dmtap/blob/main/substrate/SYNC.md) §6.1).
State-root equality is the RECOMMENDED interoperability test because it fails
loudly on history divergence a current-state comparison would hide.

> **Vector regeneration (v0.2 debt).** The `conformance/wrap_vectors.json` corpus
> predates this rebase and still encodes the retired bespoke signing envelope and
> string-HLC. It MUST be regenerated against the substrate byte formats before a
> conformance claim is made — dropping the `encode`/`id`/`sign`/`hlc`/`tiebreak`
> groups (now the substrate's) and adding `map`/`withdraw`/`retain`. Until then the
> prose of this document governs.
