//! §26 legacy adapter — Slack.
//!
//! Binds the [`super::LegacyAdapter`] / [`super::RailTransport`] framework (§26.3, §26.4) to Slack's
//! **official, terms-compliant Web API** (`chat.postMessage`) and its Events API / Socket Mode
//! inbound shape. Nothing here holds a live credential or dials the network: the live-network
//! boundary is the [`HttpPost`] trait, which tests mock, so the whole module builds and runs in CI
//! with no dependency beyond the `serde`/`serde_json` already in this crate.
//!
//! **Where Slack sits in the §26.4 table.** Inbound-triggered in *both* directions — a Slack app can
//! only reach a workspace that installed it, and cannot DM a user cold (§26.4.2); there is no tier,
//! template, or "premium outreach" that unlocks initiation, and this adapter offers none. Inbound
//! arrives over an outbound-persistent connection (Socket Mode), so it works behind CGNAT. Free at
//! the platform layer — but "free" is never "private": Slack is always a plaintext party on every
//! message (§26.5.1, §26.10). Authenticity is **platform-asserted** and cryptographically
//! unverifiable (§26.5): the most this adapter can honestly say of an inbound message is "Slack's
//! API told me this came from user id `U…`", never a DKIM-class *verified* sender.
//!
//! **Credential posture (§26.8-analogue).** Outbound uses `chat.postMessage` with a **bot token**
//! (`xoxb-…`), the sanctioned Web API surface — never a user token (`xoxp-…`) driving a human
//! account, which would be the Slack equivalent of the unofficial-library / automation path §26.8.2
//! rules out. The bearer credential is attached by the concrete [`HttpPost`] implementation a real
//! deployment supplies (Slack requires `Authorization: Bearer <token>` on the HTTPS request); this
//! module formats the request and interprets the response, and holds no credential itself.

use kotva_core::cbor::Cv;
use kotva_core::mote::{Headers, Payload};
use serde::{Deserialize, Serialize};

use super::{
    LegacyAdapter, RailMessage, RailProperties, RailSend, RailTransport, TransportError, SLACK,
};

/// Slack Web API endpoint for sending a message (the sanctioned, ToS-compliant outbound surface).
pub const CHAT_POST_MESSAGE_URL: &str = "https://slack.com/api/chat.postMessage";

/// The `Headers.ext` key (§18.3.6, §21.20 private-use `x-` namespace) under which an inbound Slack
/// message carries its **platform-asserted** origin (§26.5). Held as a text-map `{ rail, claim }`,
/// the structurally-distinct shape §26.5.1 requires — deliberately *not* the email-verdict shape
/// (`spf`/`dkim`/`dmarc`), so a consumer can never mistake "Slack's backend says this is user `U…`"
/// for a cryptographically *verified* sender. The claim is unverifiable by construction: there is no
/// signature this adapter can check independent of trusting Slack, so the mapped [`Payload`] also
/// carries an empty `from` (no identity key) and an empty `sig` (nothing to verify).
pub const PLATFORM_ASSERTED_EXT_KEY: &str = super::PLATFORM_ASSERTED_EXT_KEY;

/// The Slack rail label, as in the §26.4 table and the `platform_asserted` claim's `rail` field.
const RAIL: &str = "slack";

// ── The adapter (pure data + inbound mapping; no network) ───────────────────────────────────────

/// The Slack legacy adapter (§26). The §26.4 declaration is pure data; sends bind a [`SlackTransport`].
#[derive(Debug, Clone, Copy, Default)]
pub struct SlackAdapter;

impl LegacyAdapter for SlackAdapter {
    fn properties(&self) -> &RailProperties {
        &SLACK
    }

    /// Map an inbound Slack message → a MOTE [`Payload`] carrying the Slack user id as a
    /// **platform-asserted** origin (§26.5), honestly marked unverifiable:
    ///
    /// * the claim rides `Headers.ext` under [`PLATFORM_ASSERTED_EXT_KEY`] as `{ rail, claim }` —
    ///   the structurally-distinct entry §26.5.1 mandates, never the email-verdict shape;
    /// * `from` is left **empty** — a Slack user id is not a cryptographic identity key, so putting
    ///   it in `from` (where a real IK lives) would misrepresent it as a verified sender;
    /// * `sig` is left **empty** — there is nothing here a recipient could verify.
    fn inbound_to_mote(&self, msg: &RailMessage) -> Payload {
        let claim = Cv::TextMap(vec![
            ("rail".to_string(), Cv::Text(RAIL.to_string())),
            ("claim".to_string(), Cv::Text(msg.from.clone())),
            // §26.5.1 canonical shape: a platform rail is never verifiable — say so.
            ("verifiable".to_string(), Cv::Bool(false)),
        ]);
        Payload {
            // No verified identity: a Slack user id is platform-asserted, not a cryptographic key.
            from: Vec::new(),
            // Unsigned: nothing on this rail is cryptographically verifiable (§26.5).
            sig: Vec::new(),
            headers: Headers {
                mime: Some("text/plain; charset=utf-8".to_string()),
                ext: vec![(PLATFORM_ASSERTED_EXT_KEY.to_string(), claim)],
                ..Default::default()
            },
            body: msg.text.clone().into_bytes(),
            refs: Vec::new(),
            attach: Vec::new(),
            expires: None,
        }
    }
    // `outbound_disposition` uses the trait default: Slack cannot initiate cold (§26.4.2), so a send
    // with no open reply window resolves to `BlockedNoWindow` — a surfaced functional wall, never a
    // silently-dropped or "pay-to-unlock" send. No premium-outreach path exists and none is added.
}

/// Extract the platform-asserted `(rail, claim)` origin an inbound [`Payload`] carries under
/// [`PLATFORM_ASSERTED_EXT_KEY`] (§26.5.1), if present and well-typed. Returns `None` for a payload
/// that carries no such claim — the caller MUST NOT treat its absence as a *verified* sender either.
#[must_use]
pub fn platform_asserted_origin(headers: &Headers) -> Option<(String, String)> {
    let entry = headers.ext.iter().find(|(k, _)| k == PLATFORM_ASSERTED_EXT_KEY)?;
    let Cv::TextMap(fields) = &entry.1 else { return None };
    let get = |name: &str| {
        fields.iter().find_map(|(k, v)| match (k.as_str() == name, v) {
            (true, Cv::Text(s)) => Some(s.clone()),
            _ => None,
        })
    };
    Some((get("rail")?, get("claim")?))
}

// ── Slack Web API binding (official, ToS-compliant) ──────────────────────────────────────────────

/// The live HTTP boundary, deliberately tiny so tests mock it with **no** network and **no** new
/// dependency. A real deployment supplies an implementation that performs the HTTPS POST and attaches
/// the Slack `Authorization: Bearer <bot-token>` header (§26.8-analogue: bot token, never a user
/// token). Errors are surfaced as [`TransportError`].
pub trait HttpPost {
    /// POST `body` as `application/json` to `url`; return the response body text.
    fn post_json(&self, url: &str, body: &str) -> Result<String, TransportError>;
}

/// A `chat.postMessage` request (the two fields this adapter sends: target channel and text). Slack
/// accepts a channel id (`C…`), a DM channel (`D…`), or a user id it will open a DM with.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatPostMessageRequest {
    pub channel: String,
    pub text: String,
}

/// A `chat.postMessage` response. Slack signals success with `ok: true`; on failure `ok` is `false`
/// and `error` names the reason (e.g. `not_in_channel`, `channel_not_found`, `rate_limited`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ChatPostMessageResponse {
    pub ok: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub ts: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
}

/// The inner `event` of an Events API / Socket Mode `message` event. `user` is the **platform-
/// asserted** origin (§26.5); `bot_id` is set when the message came from a bot/app (including this
/// adapter's own echoes), which [`SlackMessageEvent::to_rail_message`] filters out.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SlackMessageEvent {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default)]
    pub bot_id: Option<String>,
    /// The message subtype (`bot_message`, `channel_join`, …). A plain user message has none.
    #[serde(default)]
    pub subtype: Option<String>,
    #[serde(default)]
    pub ts: Option<String>,
}

/// The Events API / Socket Mode envelope wrapping an `event_callback`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EventCallback {
    #[serde(rename = "type")]
    pub kind: String,
    pub event: SlackMessageEvent,
}

impl SlackMessageEvent {
    /// Project a Slack `message` event onto the rail-agnostic [`RailMessage`], or `None` for events
    /// this adapter must not treat as inbound human text: non-`message` events, any subtype (joins,
    /// bot messages, edits), and anything carrying a `bot_id` — critically the app's own outbound
    /// echoes, which would otherwise loop. An inbound user message opens a reply window (Slack lets
    /// the app respond in that conversation), so `opens_window` is `true`.
    #[must_use]
    pub fn to_rail_message(&self) -> Option<RailMessage> {
        if self.kind != "message" || self.subtype.is_some() || self.bot_id.is_some() {
            return None;
        }
        let from = self.user.clone()?;
        Some(RailMessage {
            from,
            text: self.text.clone().unwrap_or_default(),
            opens_window: true,
        })
    }
}

// ── The transport (binds the Web API to the framework's send) ────────────────────────────────────

/// Sends a MOTE onto Slack via `chat.postMessage`, over any [`HttpPost`] (real HTTPS client in
/// production, a mock in tests). Holds no credential: the token is the [`HttpPost`]'s concern.
#[derive(Debug, Clone)]
pub struct SlackTransport<H: HttpPost> {
    http: H,
}

impl<H: HttpPost> SlackTransport<H> {
    #[must_use]
    pub fn new(http: H) -> Self {
        Self { http }
    }
}

impl<H: HttpPost> RailTransport for SlackTransport<H> {
    fn send(&self, send: RailSend) -> Result<(), TransportError> {
        let req = ChatPostMessageRequest { channel: send.to, text: send.text };
        let body = serde_json::to_string(&req)
            .map_err(|e| TransportError::Rejected(format!("encode chat.postMessage: {e}")))?;
        let raw = self.http.post_json(CHAT_POST_MESSAGE_URL, &body)?;
        let resp: ChatPostMessageResponse = serde_json::from_str(&raw)
            .map_err(|e| TransportError::Rejected(format!("malformed Slack response: {e}")))?;
        if resp.ok {
            Ok(())
        } else {
            Err(TransportError::Rejected(
                resp.error.unwrap_or_else(|| "slack: unknown error".to_string()),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::{
        InitiationClass, InboundTransportClass, OutboundDisposition, PriceShape, RailAuthenticity,
    };
    use std::cell::RefCell;

    /// §26.4 / §26.4.2: Slack's declared properties — inbound-triggered in **both** directions,
    /// outbound-persistent inbound, free, platform-asserted — match the spec table exactly.
    #[test]
    fn properties_match_spec_26_4() {
        let a = SlackAdapter;
        let p = a.properties();
        // Same object the framework table pins to §26.4.
        assert_eq!(p, &SLACK);
        // Inbound-triggered both directions: no cold initiation on either leg (§26.4.2).
        assert_eq!(p.inbound.initiation, InitiationClass::InboundTriggered);
        assert_eq!(p.outbound.initiation, InitiationClass::InboundTriggered);
        assert!(!p.can_initiate_outbound_cold());
        // Outbound-persistent inbound (Socket Mode) — works behind CGNAT.
        assert_eq!(p.inbound_transport, InboundTransportClass::OutboundPersistent);
        // Free, both directions.
        assert_eq!(p.inbound.price, PriceShape::Free);
        assert_eq!(p.outbound.price, PriceShape::Free);
        // Platform-asserted, never cryptographic (§26.5).
        assert_eq!(p.authenticity, RailAuthenticity::PlatformAsserted);
    }

    /// §26.5/§26.5.1: an inbound Slack message carries its user id as a platform-asserted origin,
    /// marked unverifiable — never as a verified sender.
    #[test]
    fn inbound_to_mote_carries_platform_asserted_origin() {
        let a = SlackAdapter;
        let msg = RailMessage {
            from: "U0A1B2C3D".to_string(),
            text: "ship it".to_string(),
            opens_window: true,
        };
        let payload = a.inbound_to_mote(&msg);

        // The body is the message text; the subject is absent (Slack has no subject line).
        assert_eq!(payload.body, b"ship it");
        assert_eq!(payload.headers.subject, None);

        // Honestly unverifiable: no identity key, no signature — the two ways this could otherwise
        // masquerade as a verified sender.
        assert!(payload.from.is_empty(), "a Slack user id must not sit in `from` as a verified key");
        assert!(payload.sig.is_empty(), "an unverifiable rail message must carry no signature");

        // The origin rides the structurally-distinct platform-asserted ext entry (§26.5.1).
        let origin = platform_asserted_origin(&payload.headers)
            .expect("inbound payload must carry a platform-asserted origin");
        assert_eq!(origin, ("slack".to_string(), "U0A1B2C3D".to_string()));

        // …and it is NOT the email-verdict shape: no spf/dkim/dmarc key is present.
        for verdict in ["spf", "dkim", "dmarc", "arc"] {
            assert!(
                !payload.headers.ext.iter().any(|(k, _)| k == verdict),
                "platform-asserted claim must not overload the email-verdict shape (§26.5.1)"
            );
        }
    }

    /// §26.4.2: a cold outbound send with no open window is a **functional wall**
    /// (`BlockedNoWindow`) — surfaced, never silently deliverable and never a pay-to-unlock tier.
    #[test]
    fn cold_send_with_no_window_is_blocked() {
        let a = SlackAdapter;
        assert_eq!(
            a.outbound_disposition("U0A1B2C3D", "hello?", false),
            OutboundDisposition::BlockedNoWindow
        );
        // Inside an open reply window (the user messaged first) a reply IS deliverable — the wall is
        // the *cold* case only.
        assert!(matches!(
            a.outbound_disposition("C0FEEDBAC", "on it", true),
            OutboundDisposition::Deliverable(_)
        ));
    }

    /// A mock [`HttpPost`] recording the last request, so tests assert the wire shape with no
    /// network and no new dependency.
    struct MockHttp {
        response: String,
        last: RefCell<Option<(String, String)>>,
    }
    impl MockHttp {
        fn ok() -> Self {
            Self { response: r#"{"ok":true,"channel":"C0FEEDBAC","ts":"1700000000.000100"}"#.to_string(), last: RefCell::new(None) }
        }
        fn err(reason: &str) -> Self {
            Self { response: format!(r#"{{"ok":false,"error":"{reason}"}}"#), last: RefCell::new(None) }
        }
    }
    impl HttpPost for MockHttp {
        fn post_json(&self, url: &str, body: &str) -> Result<String, TransportError> {
            *self.last.borrow_mut() = Some((url.to_string(), body.to_string()));
            Ok(self.response.clone())
        }
    }

    /// A send formats the correct `chat.postMessage` request: right URL, JSON `{channel, text}`.
    #[test]
    fn mock_transport_formats_chat_post_message() {
        let http = MockHttp::ok();
        let tx = SlackTransport::new(http);
        tx.send(RailSend { to: "C0FEEDBAC".to_string(), text: "deploy done".to_string() })
            .expect("ok:true response must succeed");

        let (url, body) = tx.http.last.borrow().clone().expect("the transport must have POSTed");
        assert_eq!(url, CHAT_POST_MESSAGE_URL);
        // Parse the body back rather than string-match, so field order is not load-bearing.
        let req: ChatPostMessageRequest = serde_json::from_str(&body).expect("body is a valid request");
        assert_eq!(req, ChatPostMessageRequest { channel: "C0FEEDBAC".to_string(), text: "deploy done".to_string() });
    }

    /// A Slack `ok:false` response surfaces as a `Rejected` error carrying the platform's reason —
    /// never silently swallowed.
    #[test]
    fn mock_transport_surfaces_slack_error() {
        let tx = SlackTransport::new(MockHttp::err("not_in_channel"));
        let err = tx
            .send(RailSend { to: "C0FEEDBAC".to_string(), text: "hi".to_string() })
            .expect_err("ok:false must be an error");
        assert_eq!(err, TransportError::Rejected("not_in_channel".to_string()));
    }

    /// Inbound event parsing: a plain user `message` projects to a `RailMessage` that opens a reply
    /// window; the app's own bot echoes and non-message subtypes are dropped (no reply loop).
    #[test]
    fn inbound_event_projects_and_filters() {
        let raw = r#"{"type":"event_callback","event":{"type":"message","user":"U0A1B2C3D","text":"hi there","channel":"D0DM","ts":"1700000000.000100"}}"#;
        let cb: EventCallback = serde_json::from_str(raw).expect("valid event_callback");
        let rm = cb.event.to_rail_message().expect("a user message projects");
        assert_eq!(rm.from, "U0A1B2C3D");
        assert_eq!(rm.text, "hi there");
        assert!(rm.opens_window, "an inbound user message opens the reply window");

        // A bot echo (our own outbound) must not loop back in.
        let echo = SlackMessageEvent {
            kind: "message".to_string(),
            user: None,
            text: Some("deploy done".to_string()),
            channel: Some("C0FEEDBAC".to_string()),
            bot_id: Some("B0BOT".to_string()),
            subtype: Some("bot_message".to_string()),
            ts: None,
        };
        assert!(echo.to_rail_message().is_none(), "bot echoes must be filtered");
    }
}
