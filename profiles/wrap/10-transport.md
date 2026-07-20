# 11. Transport bindings

## 11.1. Binding A — HTTP (REQUIRED)

Every conformant implementation MUST support the HTTP binding. It is the
baseline that makes WRAP implementable without adopting anything else.

### 11.1.1. Endpoints

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/.well-known/wrap/identity` | Principal's public key and supported versions |
| `GET` | `/wrap/v0/vector` | Version vector — what this node holds |
| `POST` | `/wrap/v0/objects` | Push signed objects |
| `POST` | `/wrap/v0/pull` | Request objects the caller lacks |
| `GET` | `/wrap/v0/order/{id}` | All objects for one work order |

Requests and responses carry `application/wrap+cbor`: a definite-length CBOR
array of signed-object envelopes (§5.3).

A sync round is **stateless and symmetric** — push what the peer lacks, then
pull what we lack. Nodes hold no per-peer session state, so any node may relay
any other node's objects and a round may be abandoned and retried at any point
without reconciliation. Implementations SHOULD cap a batch at 2000 objects.

### 11.1.2. Request authentication

The transport is authenticated independently of object signatures, because a
transport signature answers a different question: not "who wrote this object"
but "who is talking to me right now".

The caller signs a canonical envelope:

```
sigbase = "WRAP-v0/http" ‖ 0x00 ‖ method ‖ 0x00 ‖ path ‖ 0x00 ‖
          BLAKE3-256(body) ‖ 0x00 ‖ timestamp ‖ 0x00 ‖ nonce
```

sent as headers `Wrap-Key`, `Wrap-Ts`, `Wrap-Nonce`, `Wrap-Sig`.

A receiver MUST reject a request whose `timestamp` is more than **300 seconds**
from local time, and MUST maintain a nonce cache with a TTL of at least twice
that window, rejecting repeats. Both checks are REQUIRED: freshness alone
permits replay inside the window.

Note that transport authentication is not required for correctness of the
*data* — every object is independently signed and verified (§5), so a
compromised transport cannot forge state. It exists to control who may consume
resources and to make relaying accountable.

### 11.1.3. Pairing

A node that has never seen a peer has no key to verify against. WRAP uses
trust-on-first-use bootstrapped by a **pairing secret**: an unenrolled caller
proves knowledge of a shared secret, which authorizes recording the public key
it presents. Thereafter the key alone authenticates and the secret is
irrelevant to that peer.

Implementations MUST default to rejecting unenrolled peers, and MUST require
explicit operator action to accept a pairing. Revocation is deletion of the
peer record; full revocation is deletion plus rotation of the pairing secret.

### 11.1.4. Offline transports

Because objects are self-authenticating (§5.1), any medium that moves bytes is
a valid transport. Implementations SHOULD support **file exchange**: each node
appends only its *own* objects to a file named for its Principal, and imports
every other file it finds.

Since no two nodes ever write the same file, a shared folder — a synced drive,
a NAS, or a USB stick carried between sites — becomes a transport with no
possibility of a write conflict. This is not a curiosity: it is the transport
that works during a power cut, and for many deployments it is the only one that
works reliably.

## 11.2. Binding B — DMTAP substrate (OPTIONAL)

WRAP objects MAY be carried as operations in the DMTAP substrate sync algebra,
and attestation feeds (§9.5) MAY be published as DMTAP-PUB feed objects over
plain HTTPS at `/.well-known/dmtap-pub/*`.

This binding is OPTIONAL and nothing in the core depends on it. Two properties
make it nearly free:

- WRAP identities are Ed25519 public keys, and the DMTAP identity key is also
  Ed25519, so a DMTAP participant is already a valid WRAP Principal (§2.1).
- The tie-break rule of §7.3 is byte comparison of the author public key, which
  is the substrate's rule. An implementation that follows §7.3 can move state
  between the two without reordering it.

Had WRAP tied on a node identifier instead, this binding would have been
lossy — the two engines would converge to different orders while each believed
it was correct. This is why §7.3 is normative rather than advisory.

Deployments MUST choose one merge engine per deployment and MUST NOT mix them.
Two engines with different total orders cannot share a replica set, and a
gradual migration between them is specifically the thing that does not work.

## 11.3. Other bindings

Nothing prevents a further binding — a message queue, a mesh, a Bluetooth link
between a courier's phone and a shop tablet. A binding is conformant if it
delivers intact signed objects and preserves the rules of §5.4. Bindings do not
need to preserve order, deliver exactly once, or guarantee delivery; the merge
algebra (§7) tolerates reordering, duplication and loss by construction.
