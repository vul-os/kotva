# 13. Errors

## 13.1. Codes

| Code | Name | Meaning |
|---|---|---|
| `0x0101` | `ERR_FORBIDDEN_KEY` | Object contains key 0 (§4.5) |
| `0x0102` | `ERR_NOT_CANONICAL` | Encoding is not deterministic CBOR (§4.1) |
| `0x0103` | `ERR_BAD_ID` | Recomputed `id` does not match key 3 (§4.3) |
| `0x0104` | `ERR_BAD_SIG` | Ed25519 verification failed (§5.3) |
| `0x0105` | `ERR_TOO_LARGE` | Object exceeds 65 536 bytes (§4.6) |
| `0x0106` | `ERR_UNSUPPORTED_VERSION` | `v` not supported (§4.2) |
| `0x0201` | `ERR_NOT_AUTHORIZED` | Author violates the kind's authorship rule (§5.5) |
| `0x0202` | `ERR_NOT_ISSUER` | `Assignment` not authored by the work order's issuer (§3.6) |
| `0x0203` | `ERR_EXPIRED` | Work order's `expires` has passed (§3.3) |
| `0x0204` | `ERR_CLOSED` | Offer's `closes` has passed (§3.4) |
| `0x0301` | `ERR_UNKNOWN_REFERENT` | Referenced object not held — object is pending, not rejected (§5.6) |
| `0x0302` | `ERR_UNREACHABLE_STATE` | `Progress` state not reachable from current state (§6.3) |
| `0x0401` | `ERR_STALE_REQUEST` | Transport timestamp outside the freshness window (§11.1.2) |
| `0x0402` | `ERR_REPLAY` | Transport nonce already seen (§11.1.2) |
| `0x0403` | `ERR_UNPAIRED` | Caller's key is not enrolled (§11.1.3) |

Codes `0x0900` and above are reserved for profile-specific errors.

## 13.2. Reporting

Errors are diagnostic, not protocol state. An implementation MUST NOT create a
WRAP object to report an error, and MUST NOT let a rejected object affect
merge outcomes.

Over the HTTP binding, a rejection is reported as `400` with a CBOR body
`{1: code, 2: message}`. A batch containing some invalid objects MUST accept
the valid ones and report the rejected subset by `id`; rejecting an entire
batch because one object failed lets a single malformed object block a peer's
whole queue.

## 13.3. Silence

Three conditions are explicitly **not** errors and MUST NOT be reported:

- an **unknown kind** (§4.4) — ignored silently;
- an **unknown field** (§4.4) — preserved and ignored;
- an **unknown profile** (§12.1) — stored, relayed, merged, not rendered.

Each of these is a forward-compatibility path. An implementation that errors on
them converts every future extension into a breaking change, which is the most
common way an extensible format stops being extensible.
