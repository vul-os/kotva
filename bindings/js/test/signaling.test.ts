/**
 * signaling.test.ts — SignalingClient smoke test.
 *
 * No suite existed in any of the three pre-existing consumers (the signaling
 * + fabric layers shipped without unit tests in office, mail, or OS). New
 * coverage added here:
 *   • connect() opens a WebSocket to the signaling URL, appending the auth
 *     token when present
 *   • on open, the client auto-sends a "join" frame envelope on the "signal"
 *     channel
 *   • inbound frames addressed to this peer are dispatched as a 'signal'
 *     CustomEvent on the client
 *   • inbound frames on the wrong channel / wrong peer are dropped silently
 *   • close() sends a "leave" frame and stops reconnecting
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { SignalingClient, type SignalEventDetail, type OfflineEventDetail, type ErrorEventDetail } from '../src/signaling.js'

type FakeListener = (payload: { data?: string }) => void

/** The shape of a "join"/"leave" frame this client sends over the (fake) WebSocket. */
interface SentFrame {
  channel: string
  payload: { type: string, session?: string, depositPubKey?: string }
}
function parseSentFrame(raw: string): SentFrame {
  return JSON.parse(raw) as SentFrame
}

class FakeWebSocket {
  static OPEN = 1
  static CLOSED = 3
  static CONNECTING = 0
  static instances: FakeWebSocket[] = []
  static lastUrl: string | null = null
  static lastProtocols: string[] | undefined = undefined

  url: string
  protocols: string[] | undefined
  readyState: number
  sent: string[]
  _listeners: Record<string, FakeListener[]>

  constructor(url: string, protocols?: string[]) {
    FakeWebSocket.lastUrl = url
    FakeWebSocket.lastProtocols = protocols
    FakeWebSocket.instances.push(this)
    this.url = url
    this.protocols = protocols
    this.readyState = FakeWebSocket.CONNECTING
    this.sent = []
    this._listeners = {}
  }
  addEventListener(evt: string, fn: FakeListener) {
    if (!this._listeners[evt]) this._listeners[evt] = []
    this._listeners[evt].push(fn)
  }
  send(data: string) {
    this.sent.push(data)
  }
  close() {
    this.readyState = FakeWebSocket.CLOSED
    this._fire('close', {})
  }
  _fire(evt: string, payload: { data?: string }) {
    for (const fn of (this._listeners[evt] || [])) fn(payload)
  }
  _open() {
    this.readyState = FakeWebSocket.OPEN
    this._fire('open', {})
  }
  _message(frame: unknown) {
    this._fire('message', { data: JSON.stringify(frame) })
  }
}
FakeWebSocket.instances = []
FakeWebSocket.lastUrl = null

beforeEach(() => {
  FakeWebSocket.instances = []
  FakeWebSocket.lastUrl = null
  FakeWebSocket.lastProtocols = undefined
  vi.stubGlobal('WebSocket', FakeWebSocket)
})

afterEach(() => {
  vi.restoreAllMocks()
})

describe('SignalingClient', () => {
  it('carries the auth token as a WebSocket subprotocol, not in the URL (default)', () => {
    const c = new SignalingClient({
      signalingUrl: 'ws://localhost:8080/api/peering/stream',
      sessionId: 'doc-1',
      peerId: 'alice',
      authToken: 'jwt-token',
    })
    c.connect()
    // URL must be clean — the JWT must NOT leak into the query string.
    expect(FakeWebSocket.lastUrl).toBe('ws://localhost:8080/api/peering/stream')
    expect(FakeWebSocket.lastUrl).not.toContain('token')
    // The JWT rides on the Sec-WebSocket-Protocol header instead.
    expect(FakeWebSocket.lastProtocols).toEqual(['vula.token.jwt-token'])
    c.close()
  })

  it('legacy tokenTransport:"query" appends ?token= for backends that need it', () => {
    const c = new SignalingClient({
      signalingUrl: 'ws://localhost:8080/api/peering/stream',
      sessionId: 'doc-1',
      peerId: 'alice',
      authToken: 'jwt-token',
      tokenTransport: 'query',
    })
    c.connect()
    expect(FakeWebSocket.lastUrl).toBe('ws://localhost:8080/api/peering/stream?token=jwt-token')
    expect(FakeWebSocket.lastProtocols).toBeUndefined()
    c.close()
  })

  it('opens with no token transport when unauthenticated', () => {
    const c = new SignalingClient({
      signalingUrl: 'ws://x/y',
      sessionId: 'doc-1',
      peerId: 'alice',
    })
    c.connect()
    expect(FakeWebSocket.lastUrl).toBe('ws://x/y')
    expect(FakeWebSocket.lastProtocols).toBeUndefined()
    c.close()
  })

  it('auto-sends a "join" envelope on the "signal" channel after open', () => {
    const c = new SignalingClient({
      signalingUrl: 'ws://x/y',
      sessionId: 'doc-1',
      peerId: 'alice',
    })
    c.connect()
    const ws = FakeWebSocket.instances[0]!
    ws._open()
    expect(ws.sent.length).toBe(1)
    const frame = parseSentFrame(ws.sent[0]!)
    expect(frame.channel).toBe('signal')
    expect(frame.payload.type).toBe('join')
    expect(frame.payload.session).toBe('doc-1')
    // No deposit key configured → field is omitted.
    expect(frame.payload.depositPubKey).toBeUndefined()
  })

  it('publishes the deposit signing public key in the "join" frame when available', () => {
    const c = new SignalingClient({
      signalingUrl: 'ws://x/y',
      sessionId: 'doc-1',
      peerId: 'alice',
      getDepositPubKey: () => 'BASE64_PUBKEY',
    })
    c.connect()
    const ws = FakeWebSocket.instances[0]!
    ws._open()
    const frame = parseSentFrame(ws.sent[0]!)
    expect(frame.payload.type).toBe('join')
    expect(frame.payload.depositPubKey).toBe('BASE64_PUBKEY')
  })

  it('omits depositPubKey when the callback returns null (key not yet generated)', () => {
    const c = new SignalingClient({
      signalingUrl: 'ws://x/y',
      sessionId: 'doc-1',
      peerId: 'alice',
      getDepositPubKey: () => null,
    })
    c.connect()
    const ws = FakeWebSocket.instances[0]!
    ws._open()
    const frame = parseSentFrame(ws.sent[0]!)
    expect(frame.payload.depositPubKey).toBeUndefined()
  })

  it('dispatches inbound frames addressed to this peer as a "signal" event', () => {
    const c = new SignalingClient({
      signalingUrl: 'ws://x/y',
      sessionId: 'doc-1',
      peerId: 'alice',
    })
    c.connect()
    const ws = FakeWebSocket.instances[0]!
    ws._open()

    const received: SignalEventDetail[] = []
    c.addEventListener('signal', (ev) => received.push((ev as CustomEvent<SignalEventDetail>).detail))

    ws._message({
      channel: 'signal',
      from: 'bob',
      payload: { type: 'offer', session: 'doc-1', to: 'alice', sdp: 'v=0...' },
    })
    expect(received).toHaveLength(1)
    expect(received[0]!.from).toBe('bob')
    expect(received[0]!.payload.type).toBe('offer')
  })

  it('drops frames on the wrong channel and frames addressed to a different peer', () => {
    const c = new SignalingClient({
      signalingUrl: 'ws://x/y',
      sessionId: 'doc-1',
      peerId: 'alice',
    })
    c.connect()
    const ws = FakeWebSocket.instances[0]!
    ws._open()

    const received: SignalEventDetail[] = []
    c.addEventListener('signal', (ev) => received.push((ev as CustomEvent<SignalEventDetail>).detail))

    // Wrong channel → dropped.
    ws._message({ channel: 'presence', from: 'bob', payload: { type: 'offer' } })
    // Wrong target peer → dropped.
    ws._message({
      channel: 'signal',
      from: 'bob',
      payload: { type: 'offer', session: 'doc-1', to: 'carol' },
    })
    // Wrong session → dropped.
    ws._message({
      channel: 'signal',
      from: 'bob',
      payload: { type: 'offer', session: 'other-doc', to: 'alice' },
    })

    expect(received).toHaveLength(0)
  })

  it('close() sends a "leave" frame and prevents reconnect', () => {
    const c = new SignalingClient({
      signalingUrl: 'ws://x/y',
      sessionId: 'doc-1',
      peerId: 'alice',
    })
    c.connect()
    const ws = FakeWebSocket.instances[0]!
    ws._open()
    // Clear the join frame.
    ws.sent.length = 0

    c.close()
    expect(ws.sent.length).toBe(1)
    const frame = parseSentFrame(ws.sent[0]!)
    expect(frame.channel).toBe('signal')
    expect(frame.payload.type).toBe('leave')
  })

  it('routes a signFrame rejection on the join to an "error" event instead of silently dropping the join', async () => {
    // Before the fix, `_buildJoinPayload().then(...)` had no rejection handler:
    // a signing failure (locked/revoked key, WebCrypto error, ...) vanished as
    // an unhandled promise rejection, no join frame was ever sent, and this
    // peer never became visible to the rest of the session — with nothing
    // telling the caller why.
    const signError = new Error('signing key locked')
    const c = new SignalingClient({
      signalingUrl: 'ws://x/y',
      sessionId: 'doc-1',
      peerId: 'alice',
      signFrame: () => Promise.reject(signError),
    })
    const errorCb = vi.fn()
    c.addEventListener('error', errorCb)
    c.connect()
    const ws = FakeWebSocket.instances[0]!
    ws._open()

    await vi.waitFor(() => expect(errorCb).toHaveBeenCalledTimes(1))

    // No join frame reached the wire — the failure is real, not papered over.
    expect(ws.sent).toHaveLength(0)
    const { detail } = errorCb.mock.calls[0]![0] as CustomEvent<ErrorEventDetail>
    expect(detail.context).toBe('join-sign-failed')
    expect(detail.error).toBe(signError)
  })

  it('routes a _processSignal rejection to an "error" event instead of an unhandled rejection', async () => {
    // _processSignal is defensive (every risky step has its own try/catch) so
    // it does not reject in normal operation. This proves the WEBSOCKET
    // message listener's own wiring: a non-async listener that invokes the
    // async work and routes a rejection to 'error', rather than an async
    // listener whose throw becomes an unhandled rejection the WebSocket has
    // no way to surface.
    const c = new SignalingClient({
      signalingUrl: 'ws://x/y',
      sessionId: 'doc-1',
      peerId: 'alice',
    })
    const processError = new Error('boom')
    c._processSignal = (): Promise<void> => Promise.reject(processError)

    const errorCb = vi.fn()
    c.addEventListener('error', errorCb)
    c.connect()
    const ws = FakeWebSocket.instances[0]!
    ws._open()
    ws.sent.length = 0 // clear the join frame; irrelevant here

    ws._message({ channel: 'signal', from: 'bob', payload: { type: 'offer', session: 'doc-1', to: 'alice' } })

    await vi.waitFor(() => expect(errorCb).toHaveBeenCalledTimes(1))
    const { detail } = errorCb.mock.calls[0]![0] as CustomEvent<ErrorEventDetail>
    expect(detail.context).toBe('process-signal-failed')
    expect(detail.error).toBe(processError)
  })
})

describe('SignalingClient reconnect budget', () => {
  beforeEach(() => {
    FakeWebSocket.instances = []
    vi.stubGlobal('WebSocket', FakeWebSocket)
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.restoreAllMocks()
  })

  it('emits "offline" after maxAttempts consecutive failures', async () => {
    const c = new SignalingClient({
      signalingUrl: 'ws://x/y',
      sessionId: 'doc-1',
      peerId: 'alice',
      maxAttempts: 3,
    })
    const offlineCb = vi.fn()
    c.addEventListener('offline', offlineCb)
    c.connect()

    // Trigger 3 close events (each schedules a reconnect, then we close again).
    for (let i = 0; i < 3; i++) {
      const ws = FakeWebSocket.instances[FakeWebSocket.instances.length - 1]!
      ws._fire('close', {})
      await vi.runAllTimersAsync()
    }

    expect(offlineCb).toHaveBeenCalledTimes(1)
    const { detail } = offlineCb.mock.calls[0]![0] as CustomEvent<OfflineEventDetail>
    expect(detail.attempts).toBeGreaterThanOrEqual(3)
    c.close()
  })

  it('resets the attempt counter on a successful open', async () => {
    const c = new SignalingClient({
      signalingUrl: 'ws://x/y',
      sessionId: 'doc-1',
      peerId: 'alice',
      maxAttempts: 2,
    })
    const offlineCb = vi.fn()
    c.addEventListener('offline', offlineCb)
    c.connect()

    // One failure.
    const ws0 = FakeWebSocket.instances[0]!
    ws0._fire('close', {})
    await vi.runAllTimersAsync()

    // Successful open on the new connection (counter resets).
    const ws1 = FakeWebSocket.instances[FakeWebSocket.instances.length - 1]!
    ws1._open()

    // One more failure after reset — budget should not be exhausted yet.
    ws1._fire('close', {})
    await vi.runAllTimersAsync()

    expect(offlineCb).not.toHaveBeenCalled()
    c.close()
  })

  it('emits "offline" only once even after many failures', async () => {
    const c = new SignalingClient({
      signalingUrl: 'ws://x/y',
      sessionId: 'doc-1',
      peerId: 'alice',
      maxAttempts: 2,
    })
    const offlineCb = vi.fn()
    c.addEventListener('offline', offlineCb)
    c.connect()

    for (let i = 0; i < 8; i++) {
      const ws = FakeWebSocket.instances[FakeWebSocket.instances.length - 1]!
      ws._fire('close', {})
      await vi.runAllTimersAsync()
    }

    expect(offlineCb).toHaveBeenCalledTimes(1)
    c.close()
  })
})
