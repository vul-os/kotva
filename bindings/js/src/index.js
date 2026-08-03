/**
 * src/index.js — kotva-client root barrel.
 *
 * Every module is also reachable as its own subpath (see `exports` in
 * package.json) so a consumer that only wants the chunk-proof verifier does not
 * pay for the signaling stack:
 *
 *   import { verifyChunkProof } from 'kotva-client/chunkProof'   // tree-shake
 *   import { verifyChunkProof } from 'kotva-client'              // barrel
 *
 * There are no name collisions across these modules; a re-export conflict here
 * means a module gained a duplicate export name and should be reviewed.
 */

export * from './errors.js'
export * from './secureTransport.js'
export * from './chunkProof.js'
export * from './relayBox.js'
export * from './prekeys.js'
export * from './signaling.js'
export * from './rendezvous.js'
export * from './rendezvousSignaling.js'
