# `kotva-client` — the JS binding

The browser/JS implementation of the KOTVA substrate protocols. Its Rust siblings live in
[`crates/`](../../crates); the WASM-ABI binding for Go lives in [`bindings/go`](../go). Like that
one, this directory is **its own package** — own `package.json`, own `LICENSE-MIT`, own tests — so
it versions and ships independently of the spec.

Licence: **MIT**, matching the crates (`crates/kotva-core/Cargo.toml`). The specification text is
CC BY 4.0; this is code, so it follows the code.

---

## Why this exists

A product had grown a second implementation of a kotva spec section. `chunkProof.js` opened with:

```
// chunkProof.js — the BROWSER half of the DMTAP-PUB § 5.3 chunk-tree range proof.
// EXACT PARITY WITH THE GO IMPLEMENTATION is the whole point
```

That parity requirement is a substrate concern, not a product one — the spec it implements
([`substrate/FEEDS.md § 5.3`](../../substrate/FEEDS.md)) lives here, and `substrate/FEEDS.md`
itself cites the Go half as fielded evidence for a normative decision. Two implementations of one
wire format, in two repos, with no shared owner, is precisely the drift this package removes.

Products consume this package. **This package never imports a product.**

---

## What is in it

| Module | Spec it implements | Parity partner |
|---|---|---|
| `chunkProof` | [`substrate/FEEDS.md § 5.3`](../../substrate/FEEDS.md) range proofs over the [§ 22.2.2](../../22-public-objects.md) DS-tagged Merkle tree; [§ 18.1.5](../../18-wire-format.md) `0x1e ‖ BLAKE3-256` addressing; [§ 18.1.2](../../18-wire-format.md) deterministic CBOR | Pier `tunnel/pubcache/proof.go` |
| `relayBox` | Relay-fallback confidentiality envelope — `version(1) ‖ nonce(24) ‖ XChaCha20-Poly1305`, AAD binds sender/recipient/session | — |
| `prekeys` | X3DH forward-secret content keys; HKDF-SHA256 with info `vula-x3dh-content-v2` | `vulos/backend/services/peering/prekeys.go` |
| `signaling` | Authenticated signaling: ECDSA-P256 over a canonical preimage, TOFU peer pinning, DTLS-fingerprint pinning via SDP coverage | host `ws.go` frame envelope |
| `rendezvous` | Key-addressed announce / resolve / signal / mailbox; Ed25519 over a domain-separated, length-prefixed canonical message | `tunnel/rendezvous/service.go` |
| `rendezvousSignaling` | The full WebRTC signaling lifecycle over the rendezvous surface — no host box required | — |
| `secureTransport` | Fail-closed credential-transport policy: a token rides only same-origin, TLS, or loopback | — |
| `errors` | Structured error types thrown across the above | — |

**Deliberately not here:** the *orchestrator* that wires these together (Pier's `FabricClient`).
It hardcodes host HTTP paths, carries a billing meter, and names its data channel after a product.
It is an application of the protocol, not the protocol, so it stays in the product.

## Install

```
npm install kotva-client
```

## Use

```js
import { verifyChunkProof, hashBytes, manifestRoot, chunkProof } from 'kotva-client/chunkProof'

const addrs = chunkBytes.map(hashBytes)          // § 18.1.5 addresses
const root  = manifestRoot(addrs)                // § 22.2.2 DS-tagged root
const path  = chunkProof(addrs, i)               // O(log n) audit path

// Throws ChunkProofError unless the fold reaches the root you already trust
// from the signed PubAnnounce. `nChunks` comes from the manifest header
// (⌈size ÷ chunk_sz⌉), never from the proof response — see FEEDS.md § 5.3.
verifyChunkProof({ root, nChunks, index: i, chunk: chunkBytes[i], path })
```

Every module is also on the root barrel (`import { … } from 'kotva-client'`); the subpaths exist so
a consumer that only wants the proof verifier does not pull in the signaling stack.

## Test

```
npm install && npm test
```

Runs under `environment: 'node'` — nothing here touches the DOM. `chunkProof.test.js` is
exhaustive rather than sampled (every chunk index of every tree shape up to n = 64, plus a
4096-leaf tree), which is why `vitest.config.js` raises `testTimeout`: the default 5 s is a budget
on the clock, not on the assertions, and it was cutting the walk short.

> **Not yet wired into CI.** `.github/workflows/ci.yml` does not run this suite. Until it does,
> `npm test` here is a local gate only — a green result is a claim about the machine that ran it.
