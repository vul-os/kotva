import { describe, it, expect } from 'vitest'
import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import {
  hashBytes,
  manifestRoot,
  chunkProof,
  encodeChunkProof,
  decodeChunkProof,
  verifyChunkProof,
  isChunkProofValid,
  encodeAddr,
} from '../src/chunkProof.js'

// chunkProofVectors.test.js — the BROWSER half of the cross-language lock for
// substrate/FEEDS.md § 5.3.
//
// The problem this closes. § 5.3 has three implementations: this module, the Go
// node (pier `tunnel/pubcache/{merkle,proof}.go`, which FEEDS.md § 5.3 cites BY
// NAME as fielded evidence for a normative decision), and the spec text. Until
// this file existed the two code halves each asserted a HAND-COPIED constant —
// INTEROP_ROOT_B64 / INTEROP_PROOF_HEX in chunkProof.test.js, interopRootB64 /
// interopProofHex in proof_test.go — with nothing mechanically comparing them.
// Copies drift, and worse: a round-trip cannot see two implementations agreeing
// on something WRONG. Only foreign bytes can.
//
// ../../../conformance/vectors/chunkproof_vectors.json is those foreign bytes,
// produced by conformance/vectors/gen_chunkproof_vectors.py — an independent
// generator that imports neither implementation and computes every value from
// the § 22.2.2 leaf/node formulas and the § 18.1.2 CBOR rules. Its `generated_by`
// field is honest that it is not clean-room; what it buys is that Go and JS are
// each checked against a THIRD set of bytes rather than against each other, so a
// one-sided change to either goes red on its own.
//
// HOW THE TWO REPOS ARE BOUND, without either needing the other checked out. The
// canonical corpus is the one read here; pier holds a byte-identical copy at
// tunnel/pubcache/testdata/chunkproof_vectors.json. Both suites assert the SAME
// sha256 of their own copy — CHUNKPROOF_VECTORS_SHA256 below,
// chunkProofVectorsSHA256 in pier's tunnel/pubcache/vectors_test.go. If the
// copies diverge, or if either is edited to accommodate a broken implementation,
// one side's digest assertion fails. A pin needs no sibling checkout and no
// network, so unlike a cross-repo path it cannot degrade into a gate that skips
// itself and reports success.

const CHUNKPROOF_VECTORS_SHA256 =
  '2ab20686f293b3a142bd574640c141fa9163aec1eb7666ab6166d0142fa2ad22'

// Coverage floors. A harness that iterates nothing must not read as a pass, so
// the counts are asserted three ways: against these floors, against the corpus's
// own declared `counts`, and against what the loops actually executed.
const MIN_VECTORS = 10
const MIN_PROOFS = 60
const MIN_CONTROLS = 8

const CORPUS_PATH = fileURLToPath(
  new URL('../../../conformance/vectors/chunkproof_vectors.json', import.meta.url),
)

// ── the corpus schema, as produced by gen_chunkproof_vectors.py ──────────────
// This is untrusted-until-hash-pinned external JSON, not a TS-authored wire
// format, so the shape below only needs to cover the fields this suite reads.

interface ChunkProofVectorProof {
  index: number
  path_hex: string[]
  proof_body_hex: string
}

interface ChunkProofVector {
  name: string
  n: number
  chunks_hex: string[]
  chunk_addrs_hex: string[]
  proofs: ChunkProofVectorProof[]
  root_hex: string
  root_b64url: string
}

interface ChunkProofCorruptionControl {
  name: string
  expect: string
  surface: 'verify' | 'decode'
  defect: string
  base_vector: string
  index: number
  n_chunks: number
  root_hex: string
  chunk_hex: string
  path_hex: string[]
  proof_body_hex: string
}

interface ChunkProofCorpus {
  format: string
  vectors: ChunkProofVector[]
  corruption_controls: ChunkProofCorruptionControl[]
  counts: { vectors: number, proofs: number, corruption_controls: number }
}

const toHex = (b: Uint8Array) => Array.from(b).map((x) => x.toString(16).padStart(2, '0')).join('')
const fromHex = (h: string) => Uint8Array.from(h.match(/../g) || [], (x) => parseInt(x, 16))

function loadCorpus(): ChunkProofCorpus {
  // An absent corpus is a FAILURE, not a skip: readFileSync throwing here fails
  // the file, which is the point.
  const raw = readFileSync(CORPUS_PATH)
  const digest = createHash('sha256').update(raw).digest('hex')
  if (digest !== CHUNKPROOF_VECTORS_SHA256) {
    throw new Error(
      `corpus sha256 = ${digest}, pinned ${CHUNKPROOF_VECTORS_SHA256}. This copy and pier's ` +
        'have diverged, or this one was edited. Fix the IMPLEMENTATION, or regenerate the ' +
        'corpus and update the pin in BOTH bindings/js/test/chunkProofVectors.test.ts and ' +
        'pier tunnel/pubcache/vectors_test.go.',
    )
  }
  const c = JSON.parse(raw.toString('utf8')) as ChunkProofCorpus
  if (c.format !== 'kotva-conformance-vectors/1') {
    throw new Error(`corpus format ${c.format}, want kotva-conformance-vectors/1`)
  }
  if (c.vectors.length !== c.counts.vectors) {
    throw new Error(`corpus carries ${c.vectors.length} vectors but declares ${c.counts.vectors}`)
  }
  if (c.corruption_controls.length !== c.counts.corruption_controls) {
    throw new Error(
      `corpus carries ${c.corruption_controls.length} controls but declares ${c.counts.corruption_controls}`,
    )
  }
  return c
}

const corpus = loadCorpus()

describe('shared § 5.3 corpus — the cross-language lock', () => {
  it('is the corpus this suite pinned, byte for byte', () => {
    // Asserted explicitly as well as inside loadCorpus, so the digest is a
    // visible assertion in the report rather than a side effect of an import.
    const digest = createHash('sha256').update(readFileSync(CORPUS_PATH)).digest('hex')
    expect(digest).toBe(CHUNKPROOF_VECTORS_SHA256)
    expect(corpus.vectors.length).toBeGreaterThanOrEqual(MIN_VECTORS)
  })

  // The positive half. Both directions matter and they are not the same check:
  // reproducing the corpus bytes catches an encoder divergence, and verifying the
  // CORPUS'S OWN bytes catches a verifier that only accepts what its own encoder
  // produced.
  it('reproduces every root and every proof body, and verifies the corpus bytes', () => {
    let rootsChecked = 0
    let proofsChecked = 0
    let addrsChecked = 0

    for (const v of corpus.vectors) {
      expect(v.chunks_hex.length, `${v.name}: chunk count`).toBe(v.n)
      expect(v.chunk_addrs_hex.length, `${v.name}: address count`).toBe(v.n)
      expect(v.proofs.length, `${v.name}: proof count`).toBe(v.n)

      const data = v.chunks_hex.map(fromHex)
      const chunks = data.map((d) => hashBytes(d))
      for (let i = 0; i < v.n; i++) {
        expect(toHex(chunks[i]!), `${v.name} chunk ${i}: address`).toBe(v.chunk_addrs_hex[i])
        addrsChecked++
      }

      const root = manifestRoot(chunks)
      expect(toHex(root), `${v.name}: root — the tree, the DS tag or the leaf rule differs`).toBe(
        v.root_hex,
      )
      expect(encodeAddr(root), `${v.name}: root base64url`).toBe(v.root_b64url)
      rootsChecked++

      for (const p of v.proofs) {
        const path = chunkProof(chunks, p.index)
        expect(path.length, `${v.name} chunk ${p.index}: path length — promotion rule differs`).toBe(
          p.path_hex.length,
        )
        expect(path.map(toHex), `${v.name} chunk ${p.index}: path`).toEqual(p.path_hex)
        expect(
          toHex(encodeChunkProof(p.index, path)),
          `${v.name} chunk ${p.index}: § 5.3 proof body`,
        ).toBe(p.proof_body_hex)

        // The corpus's OWN bytes, decoded and folded — not this module's.
        const decoded = decodeChunkProof(fromHex(p.proof_body_hex))
        expect(decoded.index, `${v.name} chunk ${p.index}: decoded index`).toBe(p.index)
        expect(() =>
          { verifyChunkProof({
            root: fromHex(v.root_hex),
            nChunks: v.n,
            index: p.index,
            chunk: data[p.index]!,
            path: decoded.path,
          }); },
        ).not.toThrow()
        proofsChecked++
      }
    }

    expect(rootsChecked, 'COVERAGE: an empty corpus must not read as a pass').toBeGreaterThan(0)
    expect(proofsChecked, 'COVERAGE: an empty corpus must not read as a pass').toBeGreaterThan(0)
    expect(addrsChecked, 'COVERAGE: an empty corpus must not read as a pass').toBeGreaterThan(0)
    expect(rootsChecked, 'COVERAGE: roots vs declared').toBe(corpus.counts.vectors)
    expect(proofsChecked, 'COVERAGE: proofs vs declared').toBe(corpus.counts.proofs)
    expect(rootsChecked).toBeGreaterThanOrEqual(MIN_VECTORS)
    expect(proofsChecked).toBeGreaterThanOrEqual(MIN_PROOFS)
  })

  // The negative half, and the half that says the verifier is a verifier. Every
  // control carries ONE deliberate defect over an otherwise-valid proof — a
  // flipped chunk byte, a reversed path, a truncated or padded path, a mispaired
  // index, a one-bit-wrong root, an nChunks that moves where promotion happens, a
  // short sibling, or a malformed § 5.3 body — so a rejection can only be about
  // that defect. The generator refuses to emit a control its own verifier accepts.
  it('rejects every corruption control', () => {
    let verifyControls = 0
    let decodeControls = 0

    for (const c of corpus.corruption_controls) {
      expect(c.expect, `${c.name}: only "reject" is meaningful here`).toBe('reject')
      if (c.surface === 'verify') {
        const args = {
          root: fromHex(c.root_hex),
          nChunks: c.n_chunks,
          index: c.index,
          chunk: fromHex(c.chunk_hex),
          path: c.path_hex.map(fromHex),
        }
        expect(
          isChunkProofValid(args),
          `CONTROL ${c.name} (${c.defect}) VERIFIED — a chunk that must be discarded was accepted`,
        ).toBe(false)
        expect(() => { verifyChunkProof(args); }).toThrow()
        verifyControls++
      } else if (c.surface === 'decode') {
        expect(
          () => decodeChunkProof(fromHex(c.proof_body_hex)),
          `CONTROL ${c.name} (${c.defect}) DECODED — a malformed § 5.3 body was accepted`,
        ).toThrow()
        decodeControls++
      } else {
        // Unreachable for a corpus honoring the 'verify' | 'decode' schema
        // above (hence the never-typed cast) — kept as defense in depth
        // against a corpus that no longer matches its pinned hash/schema.
        throw new Error(`${c.name}: unknown surface ${c.surface as string}`)
      }
    }

    expect(verifyControls, 'COVERAGE: verify-surface controls').toBeGreaterThan(0)
    expect(decodeControls, 'COVERAGE: decode-surface controls').toBeGreaterThan(0)
    expect(verifyControls + decodeControls, 'COVERAGE: controls vs declared').toBe(
      corpus.counts.corruption_controls,
    )
    expect(verifyControls + decodeControls).toBeGreaterThanOrEqual(MIN_CONTROLS)
  })

  // A FALSE-POSITIVE CONTROL for the negative half. If verifyChunkProof threw on
  // everything — a verifier that rejects all input passes every corruption
  // control — the test above would still be green. This asserts the same corpus
  // entries with the defect ABSENT are ACCEPTED, so "rejects the controls" means
  // something.
  it('accepts the undefected form of the controls it rejects', () => {
    const byName: Record<string, ChunkProofVector> = Object.fromEntries(
      corpus.vectors.map((v): [string, ChunkProofVector] => [v.name, v]),
    )
    let repaired = 0

    for (const c of corpus.corruption_controls) {
      if (c.surface !== 'verify') continue
      const v = byName[c.base_vector]
      expect(v, `${c.name}: base vector ${c.base_vector} missing`).toBeTruthy()
      const p = v!.proofs[c.index]
      if (!p) continue
      expect(
        isChunkProofValid({
          root: fromHex(v!.root_hex),
          nChunks: v!.n,
          index: c.index,
          chunk: fromHex(v!.chunks_hex[c.index]!),
          path: p.path_hex.map(fromHex),
        }),
        `${c.name}: the undefected proof for chunk ${c.index} must VERIFY, or the control ` +
          'proves nothing',
      ).toBe(true)
      repaired++
    }

    expect(repaired, 'COVERAGE: no control was repaired, so nothing was controlled').toBeGreaterThan(
      0,
    )
    // Every decode control is likewise built over a valid body; assert one such
    // body still decodes, so "throws" is not the decoder's only behaviour.
    const pinned = byName['chunkproof_n5_abcde']!
    expect(decodeChunkProof(fromHex(pinned.proofs[4]!.proof_body_hex)).index).toBe(4)
  })

  // The corpus must SUPERSEDE the hand-copied constants, not sit beside them:
  // its n=5 "a".."e" vector is asserted equal to the constants chunkProof.test.js
  // still pins. Without this, someone could regenerate the corpus with a
  // different tree, watch both cross-language suites go green together, and never
  // learn that the fielded constants moved.
  it('contains the hand-copied interop vector byte for byte', () => {
    const INTEROP_ROOT_B64 = 'HqmS4uJD2JJOZjmeF-YZikRhImZOgGvZHe6IwCOpRyT_'
    const INTEROP_PROOF_HEX = [
      '8200835820609ad16ca3186fc12dd32ce1d49ed57dd879c802246de385a20f7dbee2f894395820c97979256dd9f06e0dc6be9fabf2baef2acd2118939563d18bfa79661dc36dce58201365330142a154c52d28959cc1db9166d7b10c2591a9acc25d959ec7e1b8d242',
      '8201835820208e131bd1411e9d8c1d8417b9e9f370e2118a32b37535c77357c6d152348ac75820c97979256dd9f06e0dc6be9fabf2baef2acd2118939563d18bfa79661dc36dce58201365330142a154c52d28959cc1db9166d7b10c2591a9acc25d959ec7e1b8d242',
      '82028358208cc8a6db6f14fc57eacea4131385777a244b1f6feaeae1fed47ee8ef6e0982cf5820abd36c78c5c484698bf962a24adc9293467661696e0897a500df261d2b1664f258201365330142a154c52d28959cc1db9166d7b10c2591a9acc25d959ec7e1b8d242',
      '820383582093ce26dbcfb499cfd2b7ddfda025f4377f02bf62416d7f4799ea467720edaddd5820abd36c78c5c484698bf962a24adc9293467661696e0897a500df261d2b1664f258201365330142a154c52d28959cc1db9166d7b10c2591a9acc25d959ec7e1b8d242',
      '82048158205fa8b1b087f0c5dec0dc650c299f1779e735fd3b317e85793bbedac488a5183f',
    ]

    const v = corpus.vectors.find((x) => x.name === 'chunkproof_n5_abcde')
    expect(v, 'the fielded n=5 vector must be IN the corpus').toBeTruthy()
    expect(v!.root_b64url).toBe(INTEROP_ROOT_B64)
    expect(v!.chunks_hex).toEqual(['61', '62', '63', '64', '65'])
    expect(v!.proofs.map((p) => p.proof_body_hex)).toEqual(INTEROP_PROOF_HEX)
  })
})
