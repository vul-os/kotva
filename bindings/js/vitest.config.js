/**
 * vitest.config.js — test runner for kotva-client.
 *
 * `environment: 'node'`: unlike the product SDK this package was carved from,
 * nothing here touches the DOM. The protocol modules use only WebCrypto,
 * btoa/atob, TextEncoder and fetch — all global in Node >= 20 — so the jsdom
 * environment the product suite needs is dead weight here.
 *
 * `testTimeout`: chunkProof.test.js walks every chunk index of every tree shape
 * up to n = 64 and builds a 4096-leaf tree, deliberately exhaustive rather than
 * sampled. Those two cases run for tens of seconds of real BLAKE3 work on a
 * laptop and blow vitest's 5 s default, which is a limit on the CLOCK and not on
 * the assertions. The exhaustive walk is the point of the test, so the budget is
 * raised rather than the coverage cut.
 */

import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    environment: 'node',
    globals: true,
    include: ['test/**/*.test.js'],
    testTimeout: 120_000,
  },
})
