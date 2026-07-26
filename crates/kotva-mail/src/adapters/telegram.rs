//! §26 legacy adapter — **Telegram**, bound to the official **Bot API** only.
//!
//! Binds the [`super::LegacyAdapter`] / [`super::RailTransport`] framework to Telegram's official,
//! terms-compliant **Bot API** (`https://api.telegram.org/bot<token>/<method>`). This module speaks
//! **only** that sanctioned surface — never an unofficial/self-bot ("MTProto user client") protocol,
//! which would be ToS-violating and ban-prone exactly as §26.8.2 rules out for WhatsApp. No
//! credentials and no live network calls live here: the live-network boundary is [`HttpPost`], which
//! a real deployment supplies (an actual HTTPS client) and tests mock — so this file builds and runs
//! in CI with **no new crate dependency** (the request/response (de)serialization reuses the crate's
//! existing `serde`/`serde_json`, already pulled in for JMAP).
//!
//! ## What the §26.4 table says about Telegram, encoded honestly here
//!
//! * **Initiation — inbound-triggered, both directions (§26.4.2).** A Telegram bot **cannot** DM a
//!   user who has not started a chat with it. There is no tier, template, or price that unlocks
//!   cold outreach — a platform-imposed ceiling on §26.3 field 1, not an adapter limitation. This
//!   adapter therefore relies on the default [`super::LegacyAdapter::outbound_disposition`], which
//!   returns [`OutboundDisposition::BlockedNoWindow`] for a cold send: the wall is **surfaced**,
//!   never silently dropped, and this module offers no "premium outreach" that does not exist.
//! * **Inbound transport — outbound-persistent (§26.3).** The adapter holds an outbound connection
//!   open and receives over it (`getUpdates` long-polling), so it works behind CGNAT with no public
//!   endpoint. [`GetUpdates`] models that long-poll shape.
//! * **Price — free (§26.10).** Telegram charges nothing to send or receive over the Bot API.
//!   "Free" is **not** "private": the exposure field still names Telegram as an always-present
//!   plaintext party (§26.5.1, §26.10 / ADAPT-11) — see [`super::TELEGRAM`].
//! * **Authenticity — platform-asserted, unverifiable (§26.5).** The only thing this adapter can
//!   honestly convey about an inbound message is *"Telegram's API told me this came from user id N"*.
//!   There is no signature it can check independent of trusting Telegram's backend — unlike DKIM.
//!   [`TelegramAdapter::inbound_to_mote`] carries the origin **as such**, in an explicitly-labelled
//!   header ([`RAIL_ORIGIN_EXT_KEY`]), and **never** as the MOTE's cryptographic sender
//!   (`Payload.from`, a DMTAP identity key).
#![allow(dead_code)]

use kotva_core::cbor::Cv;
use kotva_core::mote::{Headers, Payload};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{LegacyAdapter, RailMessage, RailProperties, RailSend, RailTransport, TransportError};

/// `Headers.ext` (§18.3.6, §21.20 private-use `x-` namespace: FCFS, receiver-opaque, no
/// cross-implementation registration) key carrying the **platform-asserted** rail origin of an
/// inbound message. The value is a human-legible, honestly-hedged string of the form
/// `telegram:<id> (platform-asserted, unverifiable)` — chosen so that even a consumer that only ever
/// prints the raw header value cannot mistake it for a cryptographically-verified sender (§26.5).
///
/// This is deliberately **not** `Payload.from`: `from` is a DMTAP identity key (a cryptographic
/// sender, sealed-sender §2.4), and a Telegram user id is neither an identity key nor cryptographic
/// evidence of anything but what Telegram's own backend asserts. Overloading `from` with it would
/// manufacture exactly the false "verified sender" parity §26.5.1 forbids.
pub const RAIL_ORIGIN_EXT_KEY: &str = "x-dmtap-rail-origin";

/// Build the honest, self-hedging platform-asserted origin string for a Telegram user id.
#[must_use]
pub fn platform_asserted_origin(telegram_id: &str) -> String {
    format!("telegram:{telegram_id} (platform-asserted, unverifiable)")
}

/// The Telegram adapter: pure data + logic (the §26.3/§26.4 declaration and the inbound→MOTE map).
/// It holds no credentials and makes no network call — the wire is [`TelegramTransport`].
#[derive(Debug, Clone, Copy, Default)]
pub struct TelegramAdapter;

impl LegacyAdapter for TelegramAdapter {
    /// The canonical §26.4 Telegram row (see [`super::TELEGRAM`]): inbound-triggered both
    /// directions, outbound-persistent, free, platform-asserted, sanctioned bot API.
    fn properties(&self) -> &RailProperties {
        &super::TELEGRAM
    }

    /// Map an inbound Telegram message → a DMTAP MOTE [`Payload`], carrying the **platform-asserted**
    /// origin **as such** (§26.5). The Telegram user id lands in the explicitly-labelled
    /// [`RAIL_ORIGIN_EXT_KEY`] header — never in `Payload.from` (a cryptographic identity key), which
    /// is left empty because this inbound message claims **no** DMTAP identity.
    fn inbound_to_mote(&self, msg: &RailMessage) -> Payload {
        let headers = Headers {
            // A MOTE body is native text (§8.2); mark it plainly so a legacy render is well-formed.
            mime: Some("text/plain; charset=utf-8".to_string()),
            // The one honest thing we can say about who sent this: the platform *asserts* it.
            ext: vec![(RAIL_ORIGIN_EXT_KEY.to_string(), Cv::Text(platform_asserted_origin(&msg.from)))],
            ..Default::default()
        };
        Payload {
            // NOT the Telegram id: `from` is a cryptographic sender (§2.4, §26.5). No identity key is
            // being asserted for a legacy inbound, so this is empty and the origin rides the header.
            from: Vec::new(),
            sig: Vec::new(),
            headers,
            body: msg.text.clone().into_bytes(),
            refs: Vec::new(),
            attach: Vec::new(),
            expires: None,
        }
    }

    // `outbound_disposition` is the framework default: for Telegram (inbound-triggered, cannot
    // initiate cold — §26.4.2) a cold send with no open window returns `BlockedNoWindow`. We do NOT
    // override it, precisely so the §26.4.2 wall is enforced and never papered over.
}

// ── The official Bot API binding ───────────────────────────────────────────────────────────────

/// Default Telegram Bot API host. A method call is `<base>/bot<token>/<method>`.
pub const BOT_API_BASE: &str = "https://api.telegram.org";

/// The live-network seam. A real deployment supplies an HTTPS `POST` (the Bot API is JSON over
/// HTTPS); tests supply a mock. Keeping this a trait is what lets the whole adapter build and be
/// tested with **no** async runtime and **no** new HTTP crate (§26.9's "ships and versions
/// independently, no vendor churn touches the node core" applied at the dependency level too).
pub trait HttpPost {
    /// POST `body` (a JSON document) to `url` and return the response body as text. Transport-level
    /// failure is a [`TransportError`]; an HTTP/Bot-API *rejection* is surfaced by the caller from
    /// the returned body (`{ "ok": false, "description": ... }`).
    fn post_json(&self, url: &str, body: &str) -> Result<String, TransportError>;
}

/// A `sendMessage` request (the two load-bearing fields, §Bot API `sendMessage`). `chat_id` is the
/// destination chat/user; `text` is the message. Serialized with `chat_id` as a JSON **number** when
/// it is a bare integer id (the common case) and a **string** otherwise (an `@channelusername`),
/// which is exactly what the Bot API accepts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendMessage {
    pub chat_id: String,
    pub text: String,
}

impl SendMessage {
    /// The JSON request body the Bot API `sendMessage` method expects.
    #[must_use]
    pub fn to_json(&self) -> String {
        let chat_id = match self.chat_id.parse::<i64>() {
            Ok(n) => Value::from(n),
            Err(_) => Value::from(self.chat_id.clone()),
        };
        serde_json::json!({ "chat_id": chat_id, "text": self.text }).to_string()
    }
}

/// A `getUpdates` request — the **long-poll** shape (§26.3 outbound-persistent). `offset`
/// acknowledges updates already seen (Telegram then drops them); `timeout` is the long-poll hold
/// time in seconds (0 = short poll). This is how the adapter receives with no inbound reachability,
/// working behind CGNAT.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct GetUpdates {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
}

impl GetUpdates {
    /// The JSON request body for the Bot API `getUpdates` method.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// The Bot API's uniform response envelope: every method returns `{ ok, result?, description? }`.
#[derive(Debug, Clone, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct ApiResponse<T> {
    pub ok: bool,
    #[serde(default)]
    pub result: Option<T>,
    /// Human-readable failure reason when `ok == false`.
    #[serde(default)]
    pub description: Option<String>,
}

/// One incoming update from `getUpdates`. Only the fields this adapter needs are modelled; the Bot
/// API's many other update kinds are simply absent (`message == None`) and skipped.
#[derive(Debug, Clone, Deserialize)]
pub struct Update {
    pub update_id: i64,
    #[serde(default)]
    pub message: Option<Message>,
}

/// A Telegram message (the subset this adapter reads).
#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    #[serde(default)]
    pub text: Option<String>,
    /// The sender — **platform-asserted**, unverifiable (§26.5).
    #[serde(default)]
    pub from: Option<User>,
    pub chat: Chat,
}

/// A Telegram user (the id is the platform-asserted origin, §26.5).
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: i64,
}

/// A Telegram chat (its id is the `chat_id` a reply is addressed to).
#[derive(Debug, Clone, Deserialize)]
pub struct Chat {
    pub id: i64,
}

impl Message {
    /// Project an inbound Bot API message onto the rail-agnostic [`RailMessage`]. The `from` is the
    /// platform-asserted sender id (§26.5); any inbound message opens/refreshes the reply window
    /// (this is the user starting or continuing the conversation the bot may then answer within).
    #[must_use]
    pub fn to_rail_message(&self) -> RailMessage {
        RailMessage {
            from: self.from.as_ref().map(|u| u.id.to_string()).unwrap_or_default(),
            text: self.text.clone().unwrap_or_default(),
            opens_window: true,
        }
    }
}

/// The Telegram transport: binds the official Bot API over a pluggable [`HttpPost`]. Generic over
/// the HTTP seam so production supplies a real client and tests a mock — no network, no new crate.
#[derive(Debug, Clone)]
pub struct TelegramTransport<H: HttpPost> {
    http: H,
    token: String,
    base: String,
}

impl<H: HttpPost> TelegramTransport<H> {
    /// Bind against the public Bot API host with the given bot token.
    pub fn new(http: H, token: impl Into<String>) -> Self {
        Self { http, token: token.into(), base: BOT_API_BASE.to_string() }
    }

    /// Bind against a custom base URL (a self-hosted Bot API server, or a test double).
    pub fn with_base(http: H, token: impl Into<String>, base: impl Into<String>) -> Self {
        Self { http, token: token.into(), base: base.into() }
    }

    /// The full method URL: `<base>/bot<token>/<method>`.
    #[must_use]
    pub fn method_url(&self, method: &str) -> String {
        format!("{}/bot{}/{}", self.base, self.token, method)
    }

    /// Long-poll for updates (§26.3 outbound-persistent). Returns the decoded updates on `ok`, or a
    /// [`TransportError::Rejected`] carrying the Bot API's own `description` on `ok == false`.
    pub fn get_updates(&self, req: &GetUpdates) -> Result<Vec<Update>, TransportError> {
        let url = self.method_url("getUpdates");
        let resp = self.http.post_json(&url, &req.to_json())?;
        let parsed: ApiResponse<Vec<Update>> = serde_json::from_str(&resp)
            .map_err(|e| TransportError::Rejected(format!("malformed Bot API response: {e}")))?;
        if parsed.ok {
            Ok(parsed.result.unwrap_or_default())
        } else {
            Err(TransportError::Rejected(
                parsed.description.unwrap_or_else(|| "getUpdates returned ok=false".to_string()),
            ))
        }
    }
}

impl<H: HttpPost> RailTransport for TelegramTransport<H> {
    /// Format and issue the Bot API `sendMessage` call for an outbound rail send. A non-`ok`
    /// response is surfaced as [`TransportError::Rejected`] with the platform's own `description`
    /// (a rate limit, a bot the user never started, …) — never silently swallowed.
    fn send(&self, send: RailSend) -> Result<(), TransportError> {
        let req = SendMessage { chat_id: send.to, text: send.text };
        let url = self.method_url("sendMessage");
        let resp = self.http.post_json(&url, &req.to_json())?;
        let parsed: ApiResponse<Message> = serde_json::from_str(&resp)
            .map_err(|e| TransportError::Rejected(format!("malformed Bot API response: {e}")))?;
        if parsed.ok {
            Ok(())
        } else {
            Err(TransportError::Rejected(
                parsed.description.unwrap_or_else(|| "sendMessage returned ok=false".to_string()),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::OutboundDisposition;
    use std::cell::RefCell;

    /// A mock HTTP seam that records every call and returns a scripted response — no network.
    struct MockHttp {
        calls: RefCell<Vec<(String, String)>>,
        response: String,
    }

    impl MockHttp {
        fn new(response: &str) -> Self {
            Self { calls: RefCell::new(Vec::new()), response: response.to_string() }
        }
        fn last_call(&self) -> (String, String) {
            self.calls.borrow().last().cloned().expect("a call was made")
        }
    }

    impl HttpPost for MockHttp {
        fn post_json(&self, url: &str, body: &str) -> Result<String, TransportError> {
            self.calls.borrow_mut().push((url.to_string(), body.to_string()));
            Ok(self.response.clone())
        }
    }

    /// §26.4 / §26.4.2: the Telegram row this adapter declares matches the spec on every
    /// load-bearing fact — inbound-triggered both directions (cannot initiate cold), outbound
    /// transport is persistent, price is free, authenticity is platform-asserted.
    #[test]
    fn properties_match_spec_26_4() {
        use crate::adapters::{
            InboundTransportClass, InitiationClass, PriceShape, RailAuthenticity,
        };
        let p = TelegramAdapter.properties();
        assert_eq!(p.rail, "telegram");
        // §26.4.2 — no outbound-cold path in EITHER direction.
        assert_eq!(p.inbound.initiation, InitiationClass::InboundTriggered);
        assert_eq!(p.outbound.initiation, InitiationClass::InboundTriggered);
        assert!(!p.can_initiate_outbound_cold(), "Telegram must not initiate cold (§26.4.2)");
        // §26.3 — outbound-persistent (works behind CGNAT via long-poll).
        assert_eq!(p.inbound_transport, InboundTransportClass::OutboundPersistent);
        // §26.10 — free, both directions.
        assert_eq!(p.inbound.price, PriceShape::Free);
        assert_eq!(p.outbound.price, PriceShape::Free);
        // §26.5 — platform-asserted, never cryptographically verifiable.
        assert_eq!(p.authenticity, RailAuthenticity::PlatformAsserted);
    }

    /// §26.5: `inbound_to_mote` carries the Telegram origin as PLATFORM-ASSERTED — in the labelled
    /// ext header, hedged as unverifiable — and never as the cryptographic sender (`from` is empty).
    #[test]
    fn inbound_to_mote_carries_platform_asserted_origin() {
        let msg = RailMessage {
            from: "123456789".to_string(),
            text: "hello from telegram".to_string(),
            opens_window: true,
        };
        let payload = TelegramAdapter.inbound_to_mote(&msg);

        // The body is the message text.
        assert_eq!(payload.body, b"hello from telegram");
        // The Telegram id is NOT the cryptographic sender.
        assert!(payload.from.is_empty(), "a platform-asserted id must never masquerade as `from`");

        // The origin rides the labelled ext header, hedged honestly (§26.5).
        let origin = payload
            .headers
            .ext
            .iter()
            .find(|(k, _)| k == RAIL_ORIGIN_EXT_KEY)
            .map(|(_, v)| v.clone())
            .expect("the platform-asserted origin header must be present");
        match origin {
            Cv::Text(s) => {
                assert!(s.contains("telegram:123456789"), "must name the platform + id: {s}");
                assert!(
                    s.contains("platform-asserted") && s.contains("unverifiable"),
                    "must be hedged as platform-asserted + unverifiable (§26.5): {s}"
                );
            }
            other => panic!("origin header must be text, got {other:?}"),
        }
    }

    /// §26.4.2: an outbound-cold send on Telegram (no prior inbound, no open window) is a functional
    /// wall — surfaced as `BlockedNoWindow`, never silently deliverable. This uses the framework's
    /// default `outbound_disposition`, which this adapter deliberately does not override.
    #[test]
    fn outbound_cold_is_blocked_no_window() {
        let d = TelegramAdapter.outbound_disposition("123456789", "cold outreach", false);
        assert_eq!(d, OutboundDisposition::BlockedNoWindow);
        // And it is NOT a mere pricing tier: there is no template path either (that is WhatsApp).
        assert!(!matches!(d, OutboundDisposition::RequiresTemplate));
    }

    /// A mock-transport send formats the correct Bot API `sendMessage` request: the right URL
    /// (`<base>/bot<token>/sendMessage`) and a JSON body with the numeric `chat_id` and `text`.
    #[test]
    fn send_formats_correct_bot_api_request() {
        let http = MockHttp::new(r#"{"ok":true,"result":{"chat":{"id":42},"text":"hi"}}"#);
        let transport = TelegramTransport::with_base(http, "123:ABC", "https://api.telegram.org");

        transport
            .send(RailSend { to: "42".to_string(), text: "hi there".to_string() })
            .expect("a well-formed ok=true response is a successful send");

        // Re-borrow the mock via the transport to inspect the recorded call.
        let (url, body) = transport_last_call(&transport);
        assert_eq!(url, "https://api.telegram.org/bot123:ABC/sendMessage");

        let sent: Value = serde_json::from_str(&body).expect("the request body is JSON");
        assert_eq!(sent["chat_id"], serde_json::json!(42), "numeric chat_id id");
        assert_eq!(sent["text"], serde_json::json!("hi there"));
    }

    /// Helper: read the mock's last recorded call back out of the transport it was moved into.
    fn transport_last_call(t: &TelegramTransport<MockHttp>) -> (String, String) {
        t.http.last_call()
    }

    /// A `sendMessage` string chat_id (an `@channelusername`) serializes as a JSON string, not a
    /// number — matching what the Bot API accepts.
    #[test]
    fn send_message_string_chat_id_serializes_as_string() {
        let json = SendMessage { chat_id: "@somechannel".to_string(), text: "x".to_string() }.to_json();
        let v: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["chat_id"], serde_json::json!("@somechannel"));
    }

    /// The `getUpdates` long-poll response decodes into updates, and each message projects onto a
    /// `RailMessage` whose `from` is the platform-asserted sender id.
    #[test]
    fn get_updates_decodes_and_projects() {
        let resp = r#"{"ok":true,"result":[
            {"update_id":1,"message":{"text":"hi","from":{"id":777},"chat":{"id":777}}},
            {"update_id":2}
        ]}"#;
        let http = MockHttp::new(resp);
        let transport = TelegramTransport::new(http, "tok");
        let updates = transport
            .get_updates(&GetUpdates { offset: Some(0), timeout: Some(30) })
            .expect("ok=true decodes");
        assert_eq!(updates.len(), 2);
        let m = updates[0].message.as_ref().unwrap().to_rail_message();
        assert_eq!(m.from, "777");
        assert_eq!(m.text, "hi");
        assert!(m.opens_window, "an inbound message opens the reply window");
        assert!(updates[1].message.is_none(), "a non-message update is skipped");
    }

    /// A Bot API `ok:false` response surfaces as a `Rejected` transport error carrying the platform's
    /// own description — never silently swallowed.
    #[test]
    fn send_surfaces_bot_api_rejection() {
        let http = MockHttp::new(r#"{"ok":false,"description":"Forbidden: bot can't initiate conversation with a user"}"#);
        let transport = TelegramTransport::new(http, "tok");
        let err = transport
            .send(RailSend { to: "42".to_string(), text: "hi".to_string() })
            .expect_err("ok=false is an error");
        match err {
            TransportError::Rejected(d) => assert!(d.contains("can't initiate")),
            other => panic!("expected Rejected, got {other:?}"),
        }
    }
}
