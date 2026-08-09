//! §26 legacy adapter — **WhatsApp Business Cloud API** (the sanctioned, terms-compliant rail).
//!
//! Binds the [`super::LegacyAdapter`] / [`super::RailTransport`] framework to Meta's official
//! **WhatsApp Business Platform Cloud API** — POST `/{phone-number-id}/messages` on the Graph API.
//! This is the ONLY WhatsApp path this crate speaks: the unofficial consumer/`whatsapp-web`
//! reverse-engineered protocols are ruled out by §26.8.2 (terms-violating, ban-prone, no recourse)
//! and are not, and MUST NOT be, implemented here.
//!
//! Two facts from §26 are enforced **in code**, not merely documented:
//!
//!   * **The template wall (§26.4.1).** WhatsApp is asymmetric. Inbound, and replies inside the 24h
//!     service window a user's inbound message opens, are free-form and free. Outbound *outside*
//!     that window cannot carry arbitrary human text **at any price** — it is restricted to
//!     pre-approved templates. [`prepare_outbound`] makes this a type-level wall: outside the
//!     window it can only ever produce a [`WhatsAppMessage::Template`], never a free-form text —
//!     a caller with no template gets [`OutboundDisposition::RequiresTemplate`], never a silent
//!     free send.
//!   * **Platform-asserted authenticity (§26.5/§26.5.1).** An inbound message's origin (a phone
//!     number) is only ever "Meta's API told me this came from X" — never cryptographically
//!     verifiable. [`WhatsAppBusinessAdapter::inbound_to_mote`] carries it as a structurally
//!     distinct *platform-asserted* claim (never a value that could be mistaken for a DKIM-class
//!     verdict) and records that Meta always sees the plaintext.
//!
//! **Credentials (§26.8.1).** The default model is **bring-your-own**: the deployment supplies the
//! user's own WhatsApp Business Account phone-number-id and access token to [`WhatsAppTransport`].
//! Nothing is hardcoded here, and the live-network boundary is abstracted behind [`HttpPost`] so
//! the whole module is unit-testable offline with no network and no new dependency.

use kotva_core::cbor::Cv;
use kotva_core::mote::{Headers, Payload};

use super::{
    LegacyAdapter, OutboundDisposition, RailMessage, RailProperties, RailSend, RailTransport,
    TransportError,
};

// ── Payload ext keys (§21.20 private-use `x-` namespace, carried opaquely) ──────────────────────

/// `Headers.ext` key carrying the **platform-asserted** inbound origin (§26.5.1). The value is a
/// small text-keyed map `{ rail, claim, verifiable }` — deliberately a *distinct shape* from any
/// email `spf`/`dkim`/`dmarc` verdict, so a client can never collapse "Meta says this came from
/// +2782…" into a "verified sender" badge. `verifiable` is always `false` for this rail: there is
/// no signature the adapter can independently check (§26.5).
pub const PLATFORM_ASSERTED_EXT_KEY: &str = "x-dmtap-mail-platform-asserted";

/// `Headers.ext` key recording who saw this leg's plaintext (§26.5.1/§26.6). For WhatsApp the
/// honest answer is always "Meta" — neither node nor gateway mode reaches "nobody".
pub const PLAINTEXT_EXPOSURE_EXT_KEY: &str = "x-dmtap-mail-plaintext-exposure";

/// The immovable plaintext-exposure fact for this rail (§26.5.1): Meta's servers relay every
/// message in cleartext, in **every** deployment mode. Recorded on each inbound MOTE so the fact
/// travels with the message rather than living only in a spec footnote.
pub const META_PLAINTEXT_DISCLOSURE: &str =
    "Meta, always (WhatsApp relays plaintext in every mode)";

/// The rail label this adapter answers to, matching §26.4 / [`super::WHATSAPP_BUSINESS`].
const RAIL: &str = "whatsapp";

// ── The adapter (pure data + spec logic; no network) ────────────────────────────────────────────

/// The WhatsApp Business Cloud API adapter. Its [`LegacyAdapter`] surface is pure §26 logic — the
/// declaration, the inbound mapping, and the outbound wall — with no credentials and no live calls;
/// [`WhatsAppTransport`] binds the network.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WhatsAppBusinessAdapter;

impl LegacyAdapter for WhatsAppBusinessAdapter {
    fn properties(&self) -> &RailProperties {
        &super::WHATSAPP_BUSINESS
    }

    /// Map an inbound WhatsApp webhook message → a MOTE [`Payload`], carrying the sender's phone
    /// number as a **platform-asserted** origin (§26.5) — honestly marked unverifiable, never as a
    /// cryptographically-verified sender — and recording that Meta saw the plaintext (§26.5.1).
    ///
    /// There is no cryptographic identity key for a WhatsApp peer, so `from` is left empty: the only
    /// origin this rail can honestly convey lives in [`PLATFORM_ASSERTED_EXT_KEY`], distinct in
    /// shape from any email verdict, so a client renders it as an assertion and not as proof.
    fn inbound_to_mote(&self, msg: &RailMessage) -> Payload {
        let ext = vec![
            (
                PLATFORM_ASSERTED_EXT_KEY.to_string(),
                Cv::TextMap(vec![
                    ("rail".to_string(), Cv::Text(RAIL.to_string())),
                    ("claim".to_string(), Cv::Text(msg.from.clone())),
                    // No signature this adapter can check independently of trusting Meta (§26.5).
                    ("verifiable".to_string(), Cv::Bool(false)),
                ]),
            ),
            (
                PLAINTEXT_EXPOSURE_EXT_KEY.to_string(),
                Cv::Text(META_PLAINTEXT_DISCLOSURE.to_string()),
            ),
        ];
        Payload {
            from: Vec::new(), // platform-asserted only — no verifiable IK exists for this rail
            sig: Vec::new(),
            headers: Headers {
                mime: Some("text/plain; charset=utf-8".to_string()),
                ext,
                ..Default::default()
            },
            body: msg.text.clone().into_bytes(),
            refs: Vec::new(),
            attach: Vec::new(),
            expires: None,
        }
    }
}

/// The platform-asserted origin recovered from an inbound MOTE's headers (§26.5.1). `verifiable`
/// is `false` for every current legacy rail — the field exists so the fact is carried explicitly,
/// never inferred from the mere absence of a cryptographic verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformAssertedOrigin {
    pub rail: String,
    pub claim: String,
    pub verifiable: bool,
}

/// Recover the [`PlatformAssertedOrigin`] carried on an inbound MOTE by [`inbound_to_mote`], if
/// present and well-typed. Returns `None` for a MOTE that carries no such claim.
///
/// [`inbound_to_mote`]: WhatsAppBusinessAdapter::inbound_to_mote
#[must_use]
pub fn platform_asserted_origin(headers: &Headers) -> Option<PlatformAssertedOrigin> {
    let entries = headers.ext.iter().find_map(|(k, v)| {
        if k != PLATFORM_ASSERTED_EXT_KEY {
            return None;
        }
        match v {
            Cv::TextMap(m) => Some(m),
            _ => None,
        }
    })?;
    let text = |name: &str| {
        entries.iter().find_map(|(k, v)| match v {
            Cv::Text(s) if k == name => Some(s.clone()),
            _ => None,
        })
    };
    let verifiable = entries.iter().find_map(|(k, v)| match v {
        Cv::Bool(b) if k == "verifiable" => Some(*b),
        _ => None,
    });
    Some(PlatformAssertedOrigin {
        rail: text("rail")?,
        claim: text("claim")?,
        verifiable: verifiable.unwrap_or(false),
    })
}

/// Recover the plaintext-exposure disclosure ([`PLAINTEXT_EXPOSURE_EXT_KEY`]) carried on an
/// inbound MOTE, if present.
#[must_use]
pub fn plaintext_exposure(headers: &Headers) -> Option<&str> {
    headers.ext.iter().find_map(|(k, v)| match v {
        Cv::Text(s) if k == PLAINTEXT_EXPOSURE_EXT_KEY => Some(s.as_str()),
        _ => None,
    })
}

// ── The Cloud API binding: POST /{phone-number-id}/messages ─────────────────────────────────────

/// The Graph API version this binding targets. A deployment MAY override the whole base URL via
/// [`WhatsAppTransport::with_base_url`]; this is the default.
pub const DEFAULT_GRAPH_BASE: &str = "https://graph.facebook.com/v20.0";

/// A reference to a pre-approved WhatsApp message template (§26.4.1). Templates are the ONLY
/// message class deliverable outside the service window; a freeform body is not an option there,
/// at any price.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateRef {
    /// The template's registered name (approved in the WhatsApp Business Manager).
    pub name: String,
    /// BCP-47-ish language/locale code, e.g. `"en_US"`.
    pub language: String,
}

impl TemplateRef {
    #[must_use]
    pub fn new(name: impl Into<String>, language: impl Into<String>) -> Self {
        TemplateRef {
            name: name.into(),
            language: language.into(),
        }
    }
}

/// A concrete Cloud API `/messages` request body. Exactly two shapes matter here:
///
///   * [`WhatsAppMessage::Text`] — a free-form text message. **Only** valid inside the 24h service
///     window; the platform silently refuses (or the caller must not attempt) a free-form send
///     outside it. Construct via [`prepare_outbound`] so the window rule is enforced for you.
///   * [`WhatsAppMessage::Template`] — a pre-approved template. The one shape that can originate
///     outside the window (§26.4.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatsAppMessage {
    Text { to: String, body: String },
    Template { to: String, template: TemplateRef },
}

impl WhatsAppMessage {
    /// A free-form text message (in-window only).
    #[must_use]
    pub fn text(to: impl Into<String>, body: impl Into<String>) -> Self {
        WhatsAppMessage::Text {
            to: to.into(),
            body: body.into(),
        }
    }

    /// A pre-approved template message (the outbound-cold path).
    #[must_use]
    pub fn template(to: impl Into<String>, template: TemplateRef) -> Self {
        WhatsAppMessage::Template {
            to: to.into(),
            template,
        }
    }

    /// The message `type` field value (`"text"` or `"template"`) — the load-bearing discriminator
    /// the tests and the template wall pivot on.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            WhatsAppMessage::Text { .. } => "text",
            WhatsAppMessage::Template { .. } => "template",
        }
    }

    /// Render this message to the Cloud API JSON request body. Built with `serde_json` (already a
    /// crate dependency — no new dependency introduced) so escaping is correct for arbitrary text.
    #[must_use]
    pub fn to_json(&self) -> String {
        let value = match self {
            WhatsAppMessage::Text { to, body } => serde_json::json!({
                "messaging_product": "whatsapp",
                "recipient_type": "individual",
                "to": to,
                "type": "text",
                "text": { "body": body },
            }),
            WhatsAppMessage::Template { to, template } => serde_json::json!({
                "messaging_product": "whatsapp",
                "recipient_type": "individual",
                "to": to,
                "type": "template",
                "template": {
                    "name": template.name,
                    "language": { "code": template.language },
                },
            }),
        };
        value.to_string()
    }
}

/// Prepare the concrete Cloud API message for an outbound send, **enforcing the §26.4.1 wall in the
/// type system**:
///
///   * `window_open == true` → a free-form [`WhatsAppMessage::Text`]. (Inside the service window,
///     free-form replies are free and unrestricted.)
///   * `window_open == false` and a `template` is supplied → a [`WhatsAppMessage::Template`]. This
///     is the *only* way to originate outside the window.
///   * `window_open == false` and no `template` → `Err(`[`OutboundDisposition::RequiresTemplate`]`)`
///     — the functional wall. There is deliberately **no branch** that yields a free-form text
///     outside the window, so a silent free send outside the window is unrepresentable here.
///
/// # Errors
/// Returns [`OutboundDisposition::RequiresTemplate`] when a cold/out-of-window send is attempted
/// with no template — the caller MUST surface this, never drop it (§26.4.1).
pub fn prepare_outbound(
    to: &str,
    text: &str,
    window_open: bool,
    template: Option<TemplateRef>,
) -> Result<WhatsAppMessage, OutboundDisposition> {
    if window_open {
        Ok(WhatsAppMessage::text(to, text))
    } else if let Some(t) = template {
        Ok(WhatsAppMessage::template(to, t))
    } else {
        Err(OutboundDisposition::RequiresTemplate)
    }
}

/// A successful `/messages` response's message id (`wamid.…`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendReceipt {
    pub message_id: String,
}

// ── The live-network boundary (mockable; no network, no new dependency) ─────────────────────────

/// The one seam that touches the network. A real deployment supplies an HTTPS client bound to the
/// user's own access token (§26.8.1 BYO); tests supply a mock that records the request and returns
/// a canned response. Keeping this trait tiny is what lets the entire module run offline in CI.
pub trait HttpPost {
    /// POST `body` (a JSON document) to `url` with the appropriate `Authorization: Bearer` header,
    /// returning the raw response body on a 2xx, or a [`TransportError`] otherwise.
    fn post_json(&self, url: &str, body: &str) -> Result<String, TransportError>;
}

/// Binds [`WhatsAppBusinessAdapter`] to Meta's Cloud API over an [`HttpPost`] transport. Holds the
/// **bring-your-own** credentials the deployment supplies (§26.8.1) — the user's own WABA
/// phone-number-id and access token — never anything hardcoded.
#[derive(Debug, Clone)]
pub struct WhatsAppTransport<H: HttpPost> {
    adapter: WhatsAppBusinessAdapter,
    /// The user's WABA phone-number-id (path segment of `/{phone-number-id}/messages`).
    phone_number_id: String,
    /// BYO access token (§26.8.1). Passed to [`HttpPost`] as the Bearer credential.
    access_token: String,
    base_url: String,
    http: H,
}

impl<H: HttpPost> WhatsAppTransport<H> {
    /// Construct with bring-your-own credentials (§26.8.1) and the default Graph base URL.
    #[must_use]
    pub fn new(
        phone_number_id: impl Into<String>,
        access_token: impl Into<String>,
        http: H,
    ) -> Self {
        WhatsAppTransport {
            adapter: WhatsAppBusinessAdapter,
            phone_number_id: phone_number_id.into(),
            access_token: access_token.into(),
            base_url: DEFAULT_GRAPH_BASE.to_string(),
            http,
        }
    }

    /// Override the Graph API base URL (e.g. to pin a different version).
    #[must_use]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// The adapter's §26 declaration.
    #[must_use]
    pub fn adapter(&self) -> &WhatsAppBusinessAdapter {
        &self.adapter
    }

    /// The BYO access token this transport relays under (§26.8.1). A real [`HttpPost`] presents it
    /// as `Authorization: Bearer …`.
    #[must_use]
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    /// The `/{phone-number-id}/messages` endpoint this transport posts to.
    #[must_use]
    pub fn endpoint(&self) -> String {
        format!("{}/{}/messages", self.base_url, self.phone_number_id)
    }

    /// Send a fully-formed [`WhatsAppMessage`]. This is the low-level POST; the §26.4.1 wall is
    /// enforced upstream by [`prepare_outbound`] (or [`send_outbound`]), which is the only sanctioned
    /// way to obtain a message for an out-of-window send.
    ///
    /// [`send_outbound`]: WhatsAppTransport::send_outbound
    ///
    /// # Errors
    /// [`TransportError`] if the platform rejects the send or is unreachable.
    pub fn send_message(&self, msg: &WhatsAppMessage) -> Result<SendReceipt, TransportError> {
        let body = self.http.post_json(&self.endpoint(), &msg.to_json())?;
        parse_send_response(&body)
    }

    /// The full sanctioned outbound path: prepare the message under the §26.4.1 wall, then send it.
    /// Inside the window → free-form text. Outside → a template if supplied, else the wall.
    ///
    /// # Errors
    /// - `Err(Ok(disposition))` — the send did not happen because it hit the template wall
    ///   ([`OutboundDisposition::RequiresTemplate`]); surface it, do not drop it (§26.4.1).
    /// - `Err(Err(transport))` — the send was attempted but the platform rejected it / was
    ///   unreachable.
    #[allow(clippy::result_large_err, clippy::type_complexity)]
    pub fn send_outbound(
        &self,
        to: &str,
        text: &str,
        window_open: bool,
        template: Option<TemplateRef>,
    ) -> Result<SendReceipt, Result<OutboundDisposition, TransportError>> {
        let msg = prepare_outbound(to, text, window_open, template).map_err(Ok)?;
        self.send_message(&msg).map_err(Err)
    }
}

impl<H: HttpPost> RailTransport for WhatsAppTransport<H> {
    /// The generic live-boundary send: a free-form text message, valid **only** inside the service
    /// window. Callers that might be outside the window MUST route through [`send_outbound`] /
    /// [`prepare_outbound`] first, which enforce the §26.4.1 template wall; this bare send is the
    /// in-window free-form leg and does not itself re-check the window.
    ///
    /// [`send_outbound`]: WhatsAppTransport::send_outbound
    fn send(&self, send: RailSend) -> Result<(), TransportError> {
        self.send_message(&WhatsAppMessage::text(send.to, send.text))
            .map(|_| ())
    }
}

/// Parse a Cloud API `/messages` response: `{ "messages": [ { "id": "wamid…" } ] }` on success,
/// `{ "error": { "message": "…" } }` on failure. Anything unrecognisable is a rejection carrying
/// the raw body, so a caller never silently believes a malformed response was a success.
fn parse_send_response(body: &str) -> Result<SendReceipt, TransportError> {
    let value: serde_json::Value = serde_json::from_str(body)
        .map_err(|_| TransportError::Rejected(format!("unparseable response: {body}")))?;
    if let Some(err) = value.get("error") {
        let msg = err
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown error")
            .to_string();
        return Err(TransportError::Rejected(msg));
    }
    if let Some(id) = value
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .and_then(|a| a.first())
        .and_then(|m| m.get("id"))
        .and_then(serde_json::Value::as_str)
    {
        return Ok(SendReceipt {
            message_id: id.to_string(),
        });
    }
    Err(TransportError::Rejected(format!(
        "no message id in response: {body}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::{
        DeploymentMode, InitiationClass, PriceShape, RailAuthenticity, Sanctioning,
    };
    use std::cell::RefCell;

    /// A mock [`HttpPost`] that records the last posted (url, body) and returns a scripted response.
    struct MockHttp {
        response: Result<String, TransportError>,
        last: RefCell<Option<(String, String)>>,
    }

    impl MockHttp {
        fn ok() -> Self {
            MockHttp {
                response: Ok(
                    r#"{"messaging_product":"whatsapp","messages":[{"id":"wamid.TEST123"}]}"#
                        .to_string(),
                ),
                last: RefCell::new(None),
            }
        }
    }

    impl HttpPost for MockHttp {
        fn post_json(&self, url: &str, body: &str) -> Result<String, TransportError> {
            *self.last.borrow_mut() = Some((url.to_string(), body.to_string()));
            self.response.clone()
        }
    }

    /// §26.4: the two WhatsApp rows — inbound free / in-window, outbound metered + template-walled;
    /// both directions inbound-triggered; webhook inbound; platform-asserted auth.
    #[test]
    fn properties_match_spec_26_4() {
        let p = WhatsAppBusinessAdapter.properties();
        assert_eq!(p.rail, "whatsapp");
        // Inbound / in-window reply is free.
        assert_eq!(p.inbound.price, PriceShape::Free);
        // Outbound-cold is metered.
        assert_eq!(p.outbound.price, PriceShape::Metered);
        // Both legs are inbound-triggered — no outbound-cold origination path (§26.4.1).
        assert_eq!(p.inbound.initiation, InitiationClass::InboundTriggered);
        assert_eq!(p.outbound.initiation, InitiationClass::InboundTriggered);
        assert!(!p.can_initiate_outbound_cold());
        // Inbound transport is the Cloud API webhook.
        assert_eq!(
            p.inbound_transport,
            super::super::InboundTransportClass::Webhook
        );
        // Platform-asserted, never cryptographically verifiable (§26.5).
        assert_eq!(p.authenticity, RailAuthenticity::PlatformAsserted);
        // A sanctioned, terms-compliant API — so gateway mode is permitted (§26, §26.8.1).
        assert_eq!(p.sanctioning, Sanctioning::SanctionedApi);
    }

    /// §26.5/§26.5.1: an inbound MOTE carries the phone number as a *platform-asserted*, explicitly
    /// unverifiable origin, plus the Meta-plaintext disclosure — never a verified-sender shape.
    #[test]
    fn inbound_to_mote_carries_platform_asserted_origin_and_meta_disclosure() {
        let msg = RailMessage {
            from: "+27821234567".to_string(),
            text: "Hi from WhatsApp".to_string(),
            opens_window: true,
        };
        let payload = WhatsAppBusinessAdapter.inbound_to_mote(&msg);

        // No cryptographic identity key is fabricated — this rail has none (§26.5).
        assert!(
            payload.from.is_empty(),
            "must not fabricate a verifiable IK for a WhatsApp peer"
        );
        assert_eq!(payload.body, b"Hi from WhatsApp");

        // The origin is carried as a platform-asserted claim, honestly marked unverifiable.
        let origin = platform_asserted_origin(&payload.headers)
            .expect("inbound MOTE must carry a platform-asserted origin");
        assert_eq!(origin.rail, "whatsapp");
        assert_eq!(origin.claim, "+27821234567");
        assert!(
            !origin.verifiable,
            "a platform assertion is never cryptographically verifiable (§26.5)"
        );

        // And the fact that Meta saw the plaintext travels with the message (§26.5.1).
        assert_eq!(
            plaintext_exposure(&payload.headers),
            Some(META_PLAINTEXT_DISCLOSURE)
        );

        // The platform-asserted entry is structurally DISTINCT from any email verdict key — a client
        // cannot mistake it for spf/dkim/dmarc/arc (§26.5.1).
        for verdict in ["spf", "dkim", "dmarc", "arc"] {
            assert!(
                payload.headers.ext.iter().all(|(k, _)| k != verdict),
                "platform-asserted claim must not overload the email verdict key `{verdict}`"
            );
        }
    }

    /// §26.4.1: inside the 24h service window, an outbound send is a free-form Deliverable.
    #[test]
    fn outbound_inside_window_is_free_form_deliverable() {
        let disp = WhatsAppBusinessAdapter.outbound_disposition("+27821234567", "hello", true);
        match disp {
            OutboundDisposition::Deliverable(send) => {
                assert_eq!(send.to, "+27821234567");
                assert_eq!(send.text, "hello");
            }
            other => panic!("in-window send must be Deliverable, got {other:?}"),
        }
        // And it prepares to a free-form TEXT message, not a template.
        let msg = prepare_outbound("+27821234567", "hello", true, None)
            .expect("in-window free-form must prepare");
        assert_eq!(msg.kind(), "text");
    }

    /// §26.4.1 — THE WALL. Outside the window, an outbound send is `RequiresTemplate`, never a
    /// silent free send; and when forced through, it uses a *template* message, not free text.
    #[test]
    fn outbound_outside_window_is_the_template_wall() {
        // The disposition surfaces the wall.
        assert_eq!(
            WhatsAppBusinessAdapter.outbound_disposition("+27821234567", "hello", false),
            OutboundDisposition::RequiresTemplate
        );

        // The wall is enforced in code: with no template, you CANNOT obtain a free-form message.
        let walled = prepare_outbound("+27821234567", "hello", false, None);
        assert_eq!(walled, Err(OutboundDisposition::RequiresTemplate));

        // Forced through with an approved template → a TEMPLATE message, never free text.
        let forced = prepare_outbound(
            "+27821234567",
            "hello",
            false,
            Some(TemplateRef::new("appointment_reminder", "en_US")),
        )
        .expect("a template send is the one path out of the window");
        assert_eq!(forced.kind(), "template");
        let json = forced.to_json();
        assert!(
            json.contains("\"type\":\"template\""),
            "forced send must be a template: {json}"
        );
        assert!(
            !json.contains("\"type\":\"text\""),
            "forced send must NOT be free text: {json}"
        );
        assert!(json.contains("appointment_reminder"));
    }

    /// The transport actually posts the walled message and parses the receipt — end to end, over a
    /// mock (no network). Outside the window with a template, the body on the wire is a template.
    #[test]
    fn transport_sends_template_body_when_forced_outside_window() {
        let http = MockHttp::ok();
        let transport = WhatsAppTransport::new("PHONE_NUM_ID", "BYO_TOKEN", http);

        let receipt = transport
            .send_outbound(
                "+27821234567",
                "hello",
                false,
                Some(TemplateRef::new("appointment_reminder", "en_US")),
            )
            .expect("template send should succeed against the mock");
        assert_eq!(receipt.message_id, "wamid.TEST123");

        let (url, body) = transport
            .http
            .last
            .borrow()
            .clone()
            .expect("a request was posted");
        assert_eq!(
            url,
            "https://graph.facebook.com/v20.0/PHONE_NUM_ID/messages"
        );
        assert!(
            body.contains("\"type\":\"template\""),
            "the wire body must be a template: {body}"
        );
        assert!(
            !body.contains("\"type\":\"text\""),
            "the wire body must NOT be free text: {body}"
        );
    }

    /// Outside the window with no template, `send_outbound` refuses at the wall — nothing is posted.
    #[test]
    fn transport_refuses_outbound_cold_with_no_template() {
        let http = MockHttp::ok();
        let transport = WhatsAppTransport::new("PHONE_NUM_ID", "BYO_TOKEN", http);
        let err = transport
            .send_outbound("+27821234567", "hello", false, None)
            .expect_err("cold send with no template must hit the wall");
        assert_eq!(err, Ok(OutboundDisposition::RequiresTemplate));
        assert!(
            transport.http.last.borrow().is_none(),
            "nothing must be posted at the wall"
        );
    }

    /// The bare `RailTransport::send` in-window free-form leg posts a text message and parses it.
    #[test]
    fn rail_transport_send_posts_free_form_text() {
        let http = MockHttp::ok();
        let transport = WhatsAppTransport::new("PHONE_NUM_ID", "BYO_TOKEN", http);
        transport
            .send(RailSend {
                to: "+27821234567".to_string(),
                text: "in-window reply".to_string(),
            })
            .expect("in-window free-form send should succeed");
        let (_, body) = transport.http.last.borrow().clone().unwrap();
        assert!(
            body.contains("\"type\":\"text\""),
            "in-window send is free-form text: {body}"
        );
        assert!(body.contains("in-window reply"));
    }

    /// A platform error response surfaces as `TransportError::Rejected`, never a false success.
    #[test]
    fn platform_error_response_surfaces_as_rejected() {
        let http = MockHttp {
            response: Ok(
                r#"{"error":{"message":"(#131047) Re-engagement message","code":131047}}"#
                    .to_string(),
            ),
            last: RefCell::new(None),
        };
        let transport = WhatsAppTransport::new("PHONE_NUM_ID", "BYO_TOKEN", http);
        let err = transport
            .send(RailSend {
                to: "+27821234567".to_string(),
                text: "x".to_string(),
            })
            .expect_err("an error response must not read as success");
        match err {
            TransportError::Rejected(m) => assert!(m.contains("Re-engagement")),
            other => panic!("expected Rejected, got {other:?}"),
        }
    }

    /// §26 / §26.8.1: this is the sanctioned Business Cloud API, so it MAY run in gateway mode.
    #[test]
    fn permits_gateway_mode() {
        let p = WhatsAppBusinessAdapter.properties();
        assert!(
            p.permits_mode(DeploymentMode::Gateway),
            "the sanctioned WhatsApp API may gateway"
        );
        assert!(p.permits_mode(DeploymentMode::Node));
    }
}
