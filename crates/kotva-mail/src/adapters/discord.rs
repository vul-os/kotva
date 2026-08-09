//! §26 legacy adapter — **Discord**, bound to the official, ToS-compliant **Bot API**.
//!
//! This binds the [`super::LegacyAdapter`] / [`super::RailTransport`] framework (§26.3, §26.4) to
//! Discord's sanctioned integration:
//!
//!   * **outbound** is `POST /channels/{id}/messages` (the Bot REST API, §26.4 `SanctionedApi`);
//!   * **inbound** arrives over the **Gateway WebSocket** (`MESSAGE_CREATE`), the
//!     `OutboundPersistent` transport class (§26.4) — one held-open connection, works behind CGNAT.
//!
//! Two facts the spec makes load-bearing are enforced here, not merely documented:
//!
//!   * **A Discord bot cannot initiate a conversation cold** — a shared guild is required (§26.4.2).
//!     The rail is `InboundTriggered` in *both* directions, so [`super::LegacyAdapter::outbound_disposition`]
//!     returns [`super::OutboundDisposition::BlockedNoWindow`] for a cold send. There is no tier,
//!     template, or price that unlocks freely-initiating behaviour (§26.4.2), and this adapter MUST
//!     NOT imply one.
//!   * **The origin is platform-asserted, never cryptographically verifiable** (§26.5). All the
//!     adapter can honestly convey on inbound is "Discord's API told me this arrived from snowflake
//!     `N`". [`DiscordAdapter::inbound_to_mote`] therefore carries the snowflake in the MOTE's
//!     `Headers.ext` under a **structurally distinct** platform-asserted marker (§26.5.1), with
//!     `verifiable = false`, and leaves `Payload.from` empty — the snowflake is never smuggled into
//!     the place a cryptographically-verified sender IK would live.
//!
//! **Bot token only.** A conformant Discord adapter authenticates with a **bot** token
//! (`Authorization: Bot …`), never a user token driving a self-bot: user-token automation is
//! ToS-banned, exactly the class §26.8.2 rules out for WhatsApp's unofficial libraries. The
//! [`DiscordTransport`] models this — its authorization is always `Bot …`.
//!
//! The live-network boundary is the small [`HttpPost`] trait; nothing here opens a socket or holds a
//! real credential, so it builds and tests offline (the tests mock [`HttpPost`]).

use serde::{Deserialize, Serialize};

use kotva_core::cbor::Cv;
use kotva_core::mote::{Headers, Payload};

use super::{LegacyAdapter, RailMessage, RailProperties, RailSend, RailTransport, TransportError};

/// The default Discord REST API base (versioned, §26.4 sanctioned API). Overridable for tests.
pub const DISCORD_API_BASE: &str = "https://discord.com/api/v10";

/// `Headers.ext` key (§21.20 private-use `x-` namespace) under which an inbound MOTE carries the
/// **platform-asserted** rail origin (§26.5). It is deliberately a *distinct key* — not a value
/// overloaded onto any email-verdict field — so a client can tell a Discord-bridged claim from a
/// DKIM-verified one **by which key is present** (§26.5.1), never by parsing a string. The value is
/// a small `TextMap` of the informal `{ rail, claim, verifiable }` shape §26.5.1 describes.
pub const PLATFORM_ASSERTED_EXT_KEY: &str = "x-dmtap-mail-platform-asserted";

// ── The §26 adapter (pure data + inbound mapping; no network) ───────────────────────────────────

/// The Discord adapter's §26 declaration and inbound→MOTE mapping. Stateless.
#[derive(Debug, Clone, Copy, Default)]
pub struct DiscordAdapter;

impl LegacyAdapter for DiscordAdapter {
    fn properties(&self) -> &RailProperties {
        &super::DISCORD
    }

    /// Map an inbound Discord message → a DMTAP MOTE [`Payload`], carrying the Discord user snowflake
    /// as a **platform-asserted** origin (§26.5), honestly marked unverifiable.
    ///
    /// `Payload.from` is left **empty**: there is no cryptographically-verifiable sender identity key
    /// for a legacy rail, and putting the snowflake there would let it be mistaken for one (§26.5).
    /// The snowflake instead rides in `Headers.ext` under [`PLATFORM_ASSERTED_EXT_KEY`].
    fn inbound_to_mote(&self, msg: &RailMessage) -> Payload {
        let headers = Headers {
            mime: Some("text/plain; charset=utf-8".to_string()),
            ext: vec![(
                PLATFORM_ASSERTED_EXT_KEY.to_string(),
                platform_asserted_cv("discord", &msg.from),
            )],
            ..Default::default()
        };
        Payload {
            from: Vec::new(), // no verifiable IK — the origin is platform-asserted only (§26.5)
            sig: Vec::new(),
            headers,
            body: msg.text.clone().into_bytes(),
            refs: Vec::new(),
            attach: Vec::new(),
            expires: None,
        }
    }
}

/// A platform-asserted origin (§26.5.1): the rail that asserted it, the claimed handle/id, and the
/// always-`false` `verifiable` flag that keeps it from being mistaken for a cryptographic verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformAsserted {
    pub rail: String,
    pub claim: String,
    pub verifiable: bool,
}

/// Build the `Headers.ext` value carrying a platform-asserted origin (§26.5.1).
fn platform_asserted_cv(rail: &str, claim: &str) -> Cv {
    Cv::TextMap(vec![
        ("rail".to_string(), Cv::Text(rail.to_string())),
        ("claim".to_string(), Cv::Text(claim.to_string())),
        // §26.5 / §26.5.1: a platform rail is never cryptographically verifiable — say so, honestly.
        ("verifiable".to_string(), Cv::Bool(false)),
    ])
}

/// Read the platform-asserted origin (§26.5.1) back out of an inbound MOTE, if present and well
/// formed. Returns `None` for a MOTE that carries no such marker.
#[must_use]
pub fn platform_asserted_origin(payload: &Payload) -> Option<PlatformAsserted> {
    let entries = payload.headers.ext.iter().find_map(|(k, v)| {
        if k == PLATFORM_ASSERTED_EXT_KEY {
            match v {
                Cv::TextMap(m) => Some(m),
                _ => None,
            }
        } else {
            None
        }
    })?;
    let text = |key: &str| {
        entries.iter().find_map(|(k, v)| match v {
            Cv::Text(s) if k == key => Some(s.clone()),
            _ => None,
        })
    };
    let verifiable = entries.iter().find_map(|(k, v)| match v {
        Cv::Bool(b) if k == "verifiable" => Some(*b),
        _ => None,
    });
    Some(PlatformAsserted {
        rail: text("rail")?,
        claim: text("claim")?,
        verifiable: verifiable.unwrap_or(false),
    })
}

// ── The Discord Bot API binding (official, ToS-compliant) ───────────────────────────────────────

/// The live-network boundary for the Discord REST API: POST a JSON `body` to `url` under the given
/// `Authorization` header. A real deployment supplies a TLS HTTP client; the tests mock it, so this
/// module needs no HTTP dependency and opens no socket. Errors map onto [`TransportError`].
pub trait HttpPost {
    fn post_json(
        &self,
        url: &str,
        authorization: &str,
        body: &str,
    ) -> Result<String, TransportError>;
}

/// Body of `POST /channels/{id}/messages` — the minimal create-message request carrying freeform
/// text (Discord's `content` field). Other optional fields (`embeds`, `message_reference`, …) are
/// deliberately omitted: this rail carries plain human text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateMessageRequest {
    pub content: String,
}

/// A Discord user object (subset) — `id` is the snowflake the platform asserts (§26.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    #[serde(default)]
    pub bot: bool,
}

/// The response to a successful create-message call (subset of Discord's message object).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub channel_id: String,
    pub content: String,
    pub author: DiscordUser,
}

/// A Gateway (WebSocket) event frame (§26.4 `OutboundPersistent`): the op-coded envelope Discord
/// pushes over the held-open connection. `MESSAGE_CREATE` (`op = 0`, `t = "MESSAGE_CREATE"`) carries
/// a message object in `d`; that inbound shape is [`GatewayMessage`].
#[derive(Debug, Clone, Deserialize)]
pub struct GatewayEvent {
    pub op: u8,
    #[serde(rename = "t", default)]
    pub event_type: Option<String>,
    #[serde(rename = "s", default)]
    pub sequence: Option<u64>,
    #[serde(rename = "d", default)]
    pub data: Option<serde_json::Value>,
}

/// The `MESSAGE_CREATE` payload (subset): the inbound message the Gateway delivers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GatewayMessage {
    pub id: String,
    pub channel_id: String,
    pub content: String,
    pub author: DiscordUser,
    #[serde(default)]
    pub guild_id: Option<String>,
}

impl GatewayMessage {
    /// Project an inbound Gateway message onto the rail-agnostic [`RailMessage`] the framework
    /// consumes. The platform-asserted origin is the author snowflake (§26.5); Discord has no
    /// WhatsApp-style reply window, so `opens_window` is always `false`.
    #[must_use]
    pub fn to_rail_message(&self) -> RailMessage {
        RailMessage {
            from: self.author.id.clone(),
            text: self.content.clone(),
            opens_window: false,
        }
    }
}

/// The Discord [`RailTransport`] — binds the platform specifics of an outbound send to the
/// [`HttpPost`] boundary. Authenticates with a **bot** token only (`Authorization: Bot …`), never a
/// user token (self-bots are ToS-banned, §26.8.2's structural rule applied to Discord).
#[derive(Debug, Clone)]
pub struct DiscordTransport<H: HttpPost> {
    http: H,
    bot_token: String,
    api_base: String,
}

impl<H: HttpPost> DiscordTransport<H> {
    /// Bind a transport to an HTTP client and a **bot** token, against the default API base.
    pub fn new(http: H, bot_token: impl Into<String>) -> Self {
        Self {
            http,
            bot_token: bot_token.into(),
            api_base: DISCORD_API_BASE.to_string(),
        }
    }

    /// Override the API base (for tests / self-hosted proxies). Not for production credentials.
    #[must_use]
    pub fn with_api_base(mut self, api_base: impl Into<String>) -> Self {
        self.api_base = api_base.into();
        self
    }

    /// The `POST /channels/{id}/messages` URL for a channel snowflake.
    fn create_message_url(&self, channel_id: &str) -> String {
        format!("{}/channels/{}/messages", self.api_base, channel_id)
    }

    /// The `Authorization` header value — always the **bot** scheme (§26.8.2 analogue).
    fn authorization(&self) -> String {
        format!("Bot {}", self.bot_token)
    }
}

impl<H: HttpPost> RailTransport for DiscordTransport<H> {
    /// Send an outbound message by creating a Discord message in the target channel. `send.to` is the
    /// channel snowflake; `send.text` becomes the message `content`.
    ///
    /// A conformant caller reaches this only after [`LegacyAdapter::outbound_disposition`] returned
    /// [`super::OutboundDisposition::Deliverable`] — Discord cannot initiate cold (§26.4.2), so a
    /// cold send never gets this far.
    fn send(&self, send: RailSend) -> Result<(), TransportError> {
        let url = self.create_message_url(&send.to);
        let req = CreateMessageRequest { content: send.text };
        let body = serde_json::to_string(&req)
            .map_err(|e| TransportError::Rejected(format!("serialize create-message: {e}")))?;
        self.http.post_json(&url, &self.authorization(), &body)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::{
        InboundTransportClass, InitiationClass, OutboundDisposition, PriceShape, RailAuthenticity,
        Sanctioning,
    };
    use std::cell::RefCell;

    /// §26.4 (the Discord row) + §26.4.2: inbound-triggered in **both** directions, outbound-
    /// persistent, free, platform-asserted, sanctioned — and cannot initiate cold.
    #[test]
    fn properties_match_spec_26_4() {
        let a = DiscordAdapter;
        let p = a.properties();
        // Identity: the adapter declares exactly the crate's canonical Discord row.
        assert_eq!(p, &super::super::DISCORD);
        assert_eq!(p.rail, "discord");
        // §26.4.2: no outbound-cold path in either direction.
        assert_eq!(p.inbound.initiation, InitiationClass::InboundTriggered);
        assert_eq!(p.outbound.initiation, InitiationClass::InboundTriggered);
        assert!(
            !p.can_initiate_outbound_cold(),
            "Discord cannot initiate cold (§26.4.2)"
        );
        // §26.4 row: outbound-persistent (Gateway WebSocket), free, sanctioned bot API.
        assert_eq!(
            p.inbound_transport,
            InboundTransportClass::OutboundPersistent
        );
        assert_eq!(p.inbound.price, PriceShape::Free);
        assert_eq!(p.outbound.price, PriceShape::Free);
        assert_eq!(p.sanctioning, Sanctioning::SanctionedApi);
        // §26.5: platform-asserted, never cryptographically verifiable.
        assert_eq!(p.authenticity, RailAuthenticity::PlatformAsserted);
    }

    /// §26.5: an inbound message maps to a MOTE carrying the Discord snowflake as a platform-asserted
    /// origin, honestly marked unverifiable — and never in `Payload.from`.
    #[test]
    fn inbound_to_mote_carries_platform_asserted_origin() {
        let a = DiscordAdapter;
        let msg = RailMessage {
            from: "123456789012345678".to_string(), // a Discord user snowflake
            text: "hello from discord".to_string(),
            opens_window: false,
        };
        let mote = a.inbound_to_mote(&msg);

        // The body is the message text; the origin is NOT smuggled into the verified-sender slot.
        assert_eq!(mote.body, b"hello from discord");
        assert!(
            mote.from.is_empty(),
            "the snowflake must not masquerade as a verified sender IK (§26.5)"
        );

        // The platform-asserted marker is present, structurally distinct, and honest.
        let origin = platform_asserted_origin(&mote).expect("platform-asserted origin present");
        assert_eq!(origin.rail, "discord");
        assert_eq!(origin.claim, "123456789012345678");
        assert!(
            !origin.verifiable,
            "a platform-asserted claim is never verifiable (§26.5.1)"
        );

        // The marker survives a real wire CBOR round-trip (it rides in the signed Headers.ext).
        let wire = mote.det_cbor();
        let decoded =
            Payload::from_det_cbor(&wire).expect("MOTE round-trips through canonical CBOR");
        let origin2 = platform_asserted_origin(&decoded).expect("origin survives the wire");
        assert_eq!(origin, origin2);
    }

    /// A Gateway `MESSAGE_CREATE` frame projects onto a `RailMessage` whose origin is the author
    /// snowflake, then onto a platform-asserted MOTE — the full inbound path, offline.
    #[test]
    fn gateway_message_projects_to_platform_asserted_mote() {
        let frame = r#"{
            "op": 0,
            "t": "MESSAGE_CREATE",
            "s": 42,
            "d": {
                "id": "999",
                "channel_id": "555",
                "content": "hi there",
                "author": { "id": "123456789012345678", "username": "alice", "bot": false }
            }
        }"#;
        let event: GatewayEvent = serde_json::from_str(frame).unwrap();
        assert_eq!(event.op, 0);
        assert_eq!(event.event_type.as_deref(), Some("MESSAGE_CREATE"));
        let gm: GatewayMessage = serde_json::from_value(event.data.unwrap()).unwrap();
        let rail = gm.to_rail_message();
        assert_eq!(rail.from, "123456789012345678");
        assert_eq!(rail.text, "hi there");
        assert!(!rail.opens_window, "Discord has no reply window");

        let origin = platform_asserted_origin(&DiscordAdapter.inbound_to_mote(&rail)).unwrap();
        assert_eq!(origin.claim, "123456789012345678");
    }

    /// §26.4.2: a cold send (no prior inbound, no window) on Discord is a **functional wall** —
    /// surfaced as `BlockedNoWindow`, never silently deliverable and never a price the caller can pay.
    #[test]
    fn cold_send_is_blocked_no_window() {
        let a = DiscordAdapter;
        assert_eq!(
            a.outbound_disposition("555", "hi", false),
            OutboundDisposition::BlockedNoWindow,
        );
    }

    /// A capturing mock [`HttpPost`] — records the last request instead of touching the network.
    #[derive(Default)]
    struct MockHttp {
        last: RefCell<Option<(String, String, String)>>, // (url, authorization, body)
        reply: String,
    }
    impl HttpPost for MockHttp {
        fn post_json(
            &self,
            url: &str,
            authorization: &str,
            body: &str,
        ) -> Result<String, TransportError> {
            *self.last.borrow_mut() =
                Some((url.to_string(), authorization.to_string(), body.to_string()));
            Ok(self.reply.clone())
        }
    }

    /// The mock-transport send formats the correct create-message request: the versioned
    /// `POST /channels/{id}/messages` URL, a **bot** authorization, and a `{"content":…}` JSON body.
    #[test]
    fn mock_transport_send_formats_create_message_request() {
        let http = MockHttp {
            reply: r#"{"id":"1","channel_id":"555","content":"hi","author":{"id":"7","username":"bot","bot":true}}"#.to_string(),
            ..Default::default()
        };
        let tx = DiscordTransport::new(http, "SECRET-BOT-TOKEN");
        tx.send(RailSend {
            to: "555".to_string(),
            text: "hi".to_string(),
        })
        .unwrap();

        let http = &tx.http;
        let (url, auth, body) = http.last.borrow().clone().expect("a request was made");
        assert_eq!(url, "https://discord.com/api/v10/channels/555/messages");
        // §26.8.2 analogue: BOT scheme only, never a user token / self-bot.
        assert_eq!(auth, "Bot SECRET-BOT-TOKEN");
        assert!(
            auth.starts_with("Bot "),
            "must use the bot authorization scheme"
        );
        // Exact create-message body.
        assert_eq!(body, r#"{"content":"hi"}"#);
    }

    /// A transport error from the HTTP boundary propagates as a [`TransportError`], not a panic.
    #[test]
    fn transport_surfaces_http_errors() {
        struct FailingHttp;
        impl HttpPost for FailingHttp {
            fn post_json(&self, _u: &str, _a: &str, _b: &str) -> Result<String, TransportError> {
                Err(TransportError::Unreachable)
            }
        }
        let tx = DiscordTransport::new(FailingHttp, "tok");
        assert_eq!(
            tx.send(RailSend {
                to: "1".to_string(),
                text: "x".to_string()
            }),
            Err(TransportError::Unreachable),
        );
    }
}
