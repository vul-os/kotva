//! SMTP message submission (RFC 6409 on port 587, over RFC 5321) — the outbound edge for legacy
//! clients (spec §8.2). On a completed `DATA`, submission converts to a MOTE (native peer) or is
//! handed to the legacy gateway (spec §8.2 / §7); [`SmtpSession::take_submissions`] yields the
//! accepted messages and [`build_mote_draft`] shows the MOTE conversion.
//!
//! Advertised extensions: STARTTLS (RFC 3207), AUTH PLAIN/LOGIN (RFC 4954), PIPELINING (RFC 2920),
//! 8BITMIME (RFC 6152), SIZE (RFC 1870), SMTPUTF8 (RFC 6531), DSN (RFC 3461), ENHANCEDSTATUSCODES.

use kotva_core::cbor::Cv;
use kotva_core::mote::{Headers, Kind, MoteDraft};
use kotva_core::TimestampMs;

use crate::auth::{self, Authenticator, SaslMechanism};
use crate::mime::ParsedMessage;

/// The maximum message size advertised via the SIZE extension (RFC 1870). 50 MiB.
pub const MAX_SIZE: usize = 50 * 1024 * 1024;

/// What the sender asked to have returned in a DSN (RFC 3461 `RET`): the full original message,
/// or its headers only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ret {
    Full,
    Headers,
}

impl Ret {
    fn parse(v: &str) -> Option<Ret> {
        match v.to_ascii_uppercase().as_str() {
            "FULL" => Some(Ret::Full),
            "HDRS" => Some(Ret::Headers),
            _ => None,
        }
    }
}

/// An accepted submission (envelope + RFC 5322 bytes), including the DSN parameters (RFC 3461) the
/// client attached so the node's delivery path can emit delivery-status notifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Submission {
    pub mail_from: String,
    pub rcpt_to: Vec<String>,
    pub data: Vec<u8>,
    /// The identity key (`IK`) that authenticated this session — the key a downstream signer signs
    /// the outbound MOTE with, and against which `mail_from`'s sender authorisation was checked
    /// (§7.11.2 step 2). Empty only when the session ran with `require_auth = false`.
    pub authed_identity: Vec<u8>,
    /// `ENVID=` — echoed back as `Original-Envelope-Id` in any DSN (RFC 3461 §4.4).
    pub envid: Option<String>,
    /// `RET=` — whether a DSN should carry the full message or just headers (RFC 3461 §4.3).
    pub ret: Option<Ret>,
    /// Per-recipient `NOTIFY=` values (aligned with `rcpt_to`), e.g. `SUCCESS,FAILURE` / `NEVER`.
    pub dsn_notify: Vec<Option<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Greeting,
    Command,
    Data,
}

/// A stateful SMTP submission session.
pub struct SmtpSession<A: Authenticator> {
    auth: A,
    tls: bool,
    require_auth: bool,
    /// The effective SIZE limit (RFC 1870), advertised in EHLO and enforced in MAIL/DATA. Defaults
    /// to [`MAX_SIZE`]; a node may lower it per its storage plan.
    max_size: usize,
    phase: Phase,
    authed: Option<Vec<u8>>,
    /// The address (SASL authcid) the current session authenticated as — the address the submitter
    /// is authorised to send from (§7.11.2 step 2). `None` until AUTH succeeds.
    authed_user: Option<String>,
    mail_from: Option<String>,
    rcpt_to: Vec<String>,
    dsn_notify: Vec<Option<String>>,
    envid: Option<String>,
    ret: Option<Ret>,
    data_buf: Vec<u8>,
    /// Set when the accumulated DATA exceeds [`MAX_SIZE`]; the final `.` then returns 552 (RFC 1870
    /// §6.3) instead of accepting a message we advertised we would refuse.
    data_oversized: bool,
    pending_auth: Option<SmtpPendingAuth>,
    submissions: Vec<Submission>,
    /// Set only when the QUIT **command** is processed (Command phase). The transport consults
    /// [`Self::wants_close`] instead of sniffing the raw line, so a DATA body line that happens to
    /// be `QUIT` is buffered as content (RFC 5321 §4.1.1.10) rather than tearing down the
    /// connection mid-message and silently dropping the mail.
    should_close: bool,
}

enum SmtpPendingAuth {
    Plain,
    LoginUser,
    LoginPass(String),
}

impl<A: Authenticator> SmtpSession<A> {
    pub fn new(auth: A, tls: bool) -> Self {
        SmtpSession {
            auth,
            tls,
            require_auth: true,
            max_size: MAX_SIZE,
            phase: Phase::Greeting,
            authed: None,
            authed_user: None,
            mail_from: None,
            rcpt_to: Vec::new(),
            dsn_notify: Vec::new(),
            envid: None,
            ret: None,
            data_buf: Vec::new(),
            data_oversized: false,
            pending_auth: None,
            submissions: Vec::new(),
            should_close: false,
        }
    }

    /// Whether the session has processed a QUIT **command** and the transport should close the
    /// connection. Distinguishes a real QUIT from a `QUIT` line inside a DATA body (which is buffered
    /// as message content, never a close).
    pub fn wants_close(&self) -> bool {
        self.should_close
    }

    /// Override the advertised/enforced SIZE limit (RFC 1870).
    pub fn set_max_size(&mut self, max: usize) -> &mut Self {
        self.max_size = max;
        self
    }

    /// The 220 service-ready banner.
    pub fn greeting(&mut self) -> String {
        self.phase = Phase::Command;
        "220 mail.dmtap.local Envoir DMTAP Submission ready\r\n".into()
    }

    /// Accepted submissions collected so far (drains the buffer).
    pub fn take_submissions(&mut self) -> Vec<Submission> {
        std::mem::take(&mut self.submissions)
    }

    pub fn is_authenticated(&self) -> bool {
        self.authed.is_some()
    }

    /// Feed one line (without CRLF). Returns the reply (possibly multi-line).
    ///
    /// Compatibility wrapper over [`Self::feed_line_bytes`] — prefer that for a real socket: a
    /// `&str` can only exist post-UTF-8-validation, so an 8-bit-CTE ISO-8859-x/GB18030 DATA line
    /// has already been rejected or lossy-mangled by the time it is a `&str`.
    pub fn feed_line(&mut self, line: &str) -> String {
        self.feed_line_bytes(line.as_bytes())
    }

    /// Feed one **raw** line (without CRLF). This is the lossless entry point: while we advertise
    /// 8BITMIME, DATA lines must be buffered byte-exact — decoding them through UTF-8 first turns
    /// every legacy 8-bit body (ISO-8859-x, GB18030, Shift_JIS…) into U+FFFD soup before any
    /// parser ever sees it. Command lines (the non-DATA phase) are ASCII per RFC 5321, with
    /// SMTPUTF8 (RFC 6531) arguments as valid UTF-8 — both decode losslessly below; genuinely
    /// undecodable command bytes can only come from a broken client and at worst mis-spell its
    /// own 500 reply.
    pub fn feed_line_bytes(&mut self, line: &[u8]) -> String {
        if self.phase == Phase::Data {
            return self.feed_data_line(line);
        }
        let line: &str = &String::from_utf8_lossy(line);
        if let Some(p) = self.pending_auth.take() {
            return self.continue_auth(p, line);
        }
        let (verb, rest) = match line.split_once(' ') {
            Some((v, r)) => (v.to_ascii_uppercase(), r.trim()),
            None => (line.trim().to_ascii_uppercase(), ""),
        };
        match verb.as_str() {
            "EHLO" => self.ehlo(rest, true),
            "HELO" => self.ehlo(rest, false),
            "STARTTLS" => {
                self.tls = true;
                "220 2.0.0 Ready to start TLS\r\n".into()
            }
            "AUTH" => self.cmd_auth(rest),
            "MAIL" => self.cmd_mail(rest),
            "RCPT" => self.cmd_rcpt(rest),
            "DATA" => self.cmd_data(),
            "RSET" => {
                self.reset_txn();
                "250 2.0.0 OK\r\n".into()
            }
            "NOOP" => "250 2.0.0 OK\r\n".into(),
            "VRFY" => "252 2.1.5 Cannot VRFY, but will accept and attempt delivery\r\n".into(),
            "QUIT" => {
                self.should_close = true;
                "221 2.0.0 Bye\r\n".into()
            }
            "HELP" => "214 2.0.0 Envoir DMTAP submission\r\n".into(),
            _ => "500 5.5.1 Command unrecognized\r\n".into(),
        }
    }

    fn ehlo(&mut self, domain: &str, esmtp: bool) -> String {
        self.reset_txn();
        if !esmtp {
            return format!("250 mail.dmtap.local greets {domain}\r\n");
        }
        let mut lines = vec![format!("250-mail.dmtap.local greets {domain}")];
        lines.push(format!("250-SIZE {}", self.max_size));
        lines.push("250-8BITMIME".into());
        lines.push("250-SMTPUTF8".into());
        lines.push("250-PIPELINING".into());
        lines.push("250-DSN".into());
        lines.push("250-ENHANCEDSTATUSCODES".into());
        if self.tls {
            lines.push("250-AUTH PLAIN LOGIN".into());
        } else {
            lines.push("250-STARTTLS".into());
        }
        lines.push("250 HELP".into());
        lines.join("\r\n") + "\r\n"
    }

    fn cmd_auth(&mut self, rest: &str) -> String {
        if self.authed.is_some() {
            return "503 5.5.1 Already authenticated\r\n".into();
        }
        if !self.tls {
            return "538 5.7.11 Encryption required for requested authentication mechanism\r\n".into();
        }
        let mut it = rest.split_whitespace();
        let mech = it.next().unwrap_or("");
        let initial = it.next();
        match SaslMechanism::parse(mech) {
            Some(SaslMechanism::Plain) => match initial {
                Some(ir) => self.finish_plain(ir),
                None => {
                    self.pending_auth = Some(SmtpPendingAuth::Plain);
                    "334 \r\n".into()
                }
            },
            Some(SaslMechanism::Login) => {
                self.pending_auth = Some(SmtpPendingAuth::LoginUser);
                format!("334 {}\r\n", crate::util::base64_encode(b"Username:"))
            }
            None => "504 5.5.4 Unrecognized authentication type\r\n".into(),
        }
    }

    fn continue_auth(&mut self, p: SmtpPendingAuth, line: &str) -> String {
        match p {
            SmtpPendingAuth::Plain => self.finish_plain(line.trim()),
            SmtpPendingAuth::LoginUser => {
                let user = auth::decode_login_field(line.trim()).unwrap_or_default();
                self.pending_auth = Some(SmtpPendingAuth::LoginPass(user));
                format!("334 {}\r\n", crate::util::base64_encode(b"Password:"))
            }
            SmtpPendingAuth::LoginPass(user) => {
                let pass = auth::decode_login_field(line.trim()).unwrap_or_default();
                self.finish_credentials(&user, &pass)
            }
        }
    }

    fn finish_plain(&mut self, ir: &str) -> String {
        match auth::decode_plain(ir) {
            Some(cred) => self.finish_credentials(&cred.authcid, &cred.password),
            None => "501 5.5.2 Cannot decode AUTH PLAIN\r\n".into(),
        }
    }

    fn finish_credentials(&mut self, user: &str, pass: &str) -> String {
        match self.auth.verify(user, pass) {
            Some(id) => {
                self.authed = Some(id);
                self.authed_user = Some(user.to_string()); // the address this credential may send as
                "235 2.7.0 Authentication successful\r\n".into()
            }
            None => "535 5.7.8 Authentication credentials invalid\r\n".into(),
        }
    }

    fn cmd_mail(&mut self, rest: &str) -> String {
        if self.require_auth && self.authed.is_none() {
            return "530 5.7.0 Authentication required\r\n".into();
        }
        // MAIL FROM:<addr> [ SIZE=n BODY=8BITMIME SMTPUTF8 RET=… ENVID=… ]
        let addr = match parse_path_param(rest, "FROM") {
            Some(a) => a,
            None => return "501 5.5.4 Syntax: MAIL FROM:<address>\r\n".into(),
        };
        // §7.11.2 step 2 / RFC 6409: bind the envelope sender to the authenticated identity. An
        // authenticated submitter may only send AS the address it authenticated as (the address the
        // app-password is bound to) — otherwise any app-password holder could submit
        // MAIL FROM:<victim@any-served-domain> and have the gateway sign + relay fully DMARC-aligned
        // mail as anyone. Fail closed rather than sign-and-send an unauthorised sender
        // (ERR_GATEWAY_SENDER_ADDRESS_UNAUTHORIZED, 0x060A, §21). A null return-path (<>) is exempt —
        // it is the bounce/DSN sender and cannot spoof a victim. (A deployment that authorises an
        // identity for *additional* addresses does so in its Authenticator / via a send-as capability
        // token, §18.8a.3; the reference binds one address per credential.)
        if self.require_auth && !addr.is_empty() {
            let authorised = self.authed_user.as_deref().is_some_and(|u| addr.eq_ignore_ascii_case(u));
            if !authorised {
                return "550 5.7.1 Sender address not authorised for the authenticated identity\r\n"
                    .into();
            }
        }
        // Honor a declared SIZE against our advertised limit (RFC 1870).
        if let Some(sz) = param_value(rest, "SIZE").and_then(|v| v.parse::<usize>().ok()) {
            if sz > self.max_size {
                return "552 5.3.4 Message size exceeds fixed limit\r\n".into();
            }
        }
        // Capture the DSN envelope parameters (RFC 3461): RET= and ENVID=.
        self.ret = param_value(rest, "RET").and_then(|v| Ret::parse(&v));
        self.envid = param_value(rest, "ENVID");
        self.mail_from = Some(addr);
        self.rcpt_to.clear();
        self.dsn_notify.clear();
        "250 2.1.0 Sender OK\r\n".into()
    }

    fn cmd_rcpt(&mut self, rest: &str) -> String {
        if self.mail_from.is_none() {
            return "503 5.5.1 Need MAIL before RCPT\r\n".into();
        }
        let addr = match parse_path_param(rest, "TO") {
            Some(a) => a,
            None => return "501 5.5.4 Syntax: RCPT TO:<address>\r\n".into(),
        };
        self.rcpt_to.push(addr);
        self.dsn_notify.push(param_value(rest, "NOTIFY"));
        "250 2.1.5 Recipient OK\r\n".into()
    }

    fn cmd_data(&mut self) -> String {
        if self.mail_from.is_none() || self.rcpt_to.is_empty() {
            return "503 5.5.1 Need MAIL and RCPT before DATA\r\n".into();
        }
        self.phase = Phase::Data;
        self.data_buf.clear();
        self.data_oversized = false;
        "354 End data with <CR><LF>.<CR><LF>\r\n".into()
    }

    fn feed_data_line(&mut self, line: &[u8]) -> String {
        if line == b"." {
            // End of data.
            self.phase = Phase::Command;
            // Enforce the advertised SIZE limit (RFC 1870): refuse an over-limit message rather
            // than silently accepting it.
            if self.data_oversized {
                self.data_buf.clear();
                self.reset_txn();
                return "552 5.3.4 Message exceeds fixed maximum message size\r\n".into();
            }
            let data = std::mem::take(&mut self.data_buf);
            let sub = Submission {
                mail_from: self.mail_from.take().unwrap_or_default(),
                rcpt_to: std::mem::take(&mut self.rcpt_to),
                data,
                envid: self.envid.take(),
                ret: self.ret.take(),
                dsn_notify: std::mem::take(&mut self.dsn_notify),
                authed_identity: self.authed.clone().unwrap_or_default(),
            };
            self.submissions.push(sub);
            return "250 2.0.0 OK: queued as MOTE\r\n".into();
        }
        // Dot-unstuffing (RFC 5321 §4.5.2) — on the raw bytes, so 8-bit content is untouched.
        let content = line.strip_prefix(b".").unwrap_or(line);
        // Track the total size but stop buffering once over the limit (bounded memory even for a
        // hostile stream); the terminating `.` then returns 552.
        if self.data_buf.len() <= self.max_size {
            self.data_buf.extend_from_slice(content);
            self.data_buf.extend_from_slice(b"\r\n");
        }
        if self.data_buf.len() > self.max_size {
            self.data_oversized = true;
        }
        String::new()
    }

    fn reset_txn(&mut self) {
        self.mail_from = None;
        self.rcpt_to.clear();
        self.dsn_notify.clear();
        self.envid = None;
        self.ret = None;
        self.data_buf.clear();
    }
}

/// Extract the `<addr>` after `MAIL FROM:` / `RCPT TO:`.
fn parse_path_param(rest: &str, keyword: &str) -> Option<String> {
    let up = rest.to_ascii_uppercase();
    let kw = format!("{keyword}:");
    let idx = up.find(&kw)?;
    let after = &rest[idx + kw.len()..];
    let after = after.trim_start();
    // Angle-addr "<addr>" only when '<' precedes '>'. A malformed path where '>' comes first
    // (e.g. `MAIL FROM:><`, `RCPT TO:> x <`) must not reach `after[lt + 1..gt]` — that inverted
    // slice range would PANIC and kill the per-connection session (a remote DoS). `get()` yields
    // None on the inverted range, and `filter(gt > lt)` keeps the bare-address fallback for it.
    if let Some(addr) = after
        .find('<')
        .zip(after.find('>'))
        .filter(|&(lt, gt)| gt > lt)
        .and_then(|(lt, gt)| after.get(lt + 1..gt))
    {
        Some(addr.to_string())
    } else {
        // Bare address up to first space.
        after.split_whitespace().next().map(str::to_string)
    }
}

fn param_value(rest: &str, key: &str) -> Option<String> {
    for tok in rest.split_whitespace() {
        if let Some((k, v)) = tok.split_once('=') {
            if k.eq_ignore_ascii_case(key) {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// Convert a submitted RFC 5322 message into a native MOTE draft (spec §8.2 outbound path).
/// The Subject/Content-Type map into MOTE [`Headers`]; the message body becomes the MOTE body.
///
/// **Byte-exactness (§7.2 step 4 / §7.2b).** The original header block — every header, not just
/// Subject/Content-Type (Reply-To, In-Reply-To, References, X-*, …) — is carried byte-exact in
/// `Headers.ext` under [`crate::mime::RAW_HEADERS_EXT_KEY`], so [`crate::mime::render_rfc5322`]
/// can replay it verbatim on the way back out instead of re-synthesizing a reduced header set.
/// `subject`/`mime` are still populated too, for native-side display/search that only wants the
/// decoded MOTE-native fields.
///
/// **Header hygiene (§7.2c).** Before capturing the raw block, every trust-boundary
/// `Authentication-Results`/`ARC-*` header is stripped — those are meaningful only relative to
/// the boundary that added them, and carrying an attacker-supplied one byte-exact into the
/// wrapped (and eventually signed) bytes would let a forged verdict ride along under the
/// gateway's own vouching signature.
pub fn build_mote_draft(data: &[u8], ts: TimestampMs) -> MoteDraft {
    let parsed = ParsedMessage::parse(data);
    let mut draft = MoteDraft::new(Kind::Mail, ts, parsed.body.clone());
    let (raw_headers, _) = crate::mime::header_and_body(data);
    let raw_headers = crate::mime::strip_trust_boundary_headers(&raw_headers);
    let mut ext = Vec::new();
    if !raw_headers.is_empty() {
        ext.push((crate::mime::RAW_HEADERS_EXT_KEY.to_string(), Cv::Bytes(raw_headers)));
    }
    draft.headers = Headers {
        thread: None,
        // MOTE headers are native UTF-8 (spec §2.4): lift the subject through RFC 2047 decode so a
        // `=?UTF-8?B?…?=` (or ISO-8859-1/Q) legacy subject becomes real text, not wire gibberish.
        subject: parsed.header("Subject").map(crate::mime::decode_encoded_words),
        mime: parsed.header("Content-Type").map(str::to_string),
        cc: Vec::new(),
        ext,
        sensitive: None,
    };
    draft
}

// --- DSN (Delivery Status Notification) generation (RFC 3461 / RFC 3464) --------------------

/// The per-recipient delivery action reported in a DSN (RFC 3464 §2.3.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsnAction {
    Failed,
    Delayed,
    Delivered,
    Relayed,
    Expanded,
}

impl DsnAction {
    fn wire(self) -> &'static str {
        match self {
            DsnAction::Failed => "failed",
            DsnAction::Delayed => "delayed",
            DsnAction::Delivered => "delivered",
            DsnAction::Relayed => "relayed",
            DsnAction::Expanded => "expanded",
        }
    }
    fn human(self) -> &'static str {
        match self {
            DsnAction::Failed => "Delivery to the following recipient failed permanently:",
            DsnAction::Delayed => "Delivery to the following recipient has been delayed:",
            DsnAction::Delivered => "Your message was successfully delivered to:",
            DsnAction::Relayed => "Your message was relayed to:",
            DsnAction::Expanded => "Your message was expanded to:",
        }
    }
}

/// One recipient's status in a DSN report (RFC 3464 §2.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DsnRecipient {
    pub address: String,
    pub action: DsnAction,
    /// The RFC 3463 enhanced status code, e.g. `5.1.1` (no such user).
    pub status: String,
    /// An optional SMTP diagnostic (`Diagnostic-Code: smtp; 550 …`).
    pub diagnostic: Option<String>,
}

/// A delivery-status report to be rendered into an RFC 3464 `multipart/report` message.
#[derive(Debug, Clone)]
pub struct DsnReport {
    /// Who the DSN is addressed to — the original `MAIL FROM` (the return path).
    pub notify_to: String,
    /// This node's reporting MTA name (`Reporting-MTA: dns; <name>`).
    pub reporting_mta: String,
    /// Echoed `ENVID` (`Original-Envelope-Id`), if the submission supplied one.
    pub original_envelope_id: Option<String>,
    pub recipients: Vec<DsnRecipient>,
    /// The submitted message (for the returned `message/rfc822` or `text/rfc822-headers` part).
    pub original_message: Vec<u8>,
    /// Whether to attach the full message or just its headers (from `RET=`).
    pub ret: Ret,
    pub arrival: TimestampMs,
}

impl DsnReport {
    /// A permanent-failure DSN for every recipient of `sub` that asked to be notified on failure
    /// (RFC 3461: `NOTIFY=NEVER`/`SUCCESS`-only recipients are skipped). Returns `None` when no
    /// recipient wants a failure DSN. `status`/`diagnostic` describe the failure.
    pub fn failure_for(
        sub: &Submission,
        reporting_mta: impl Into<String>,
        status: &str,
        diagnostic: Option<&str>,
        now: TimestampMs,
    ) -> Option<DsnReport> {
        let recipients: Vec<DsnRecipient> = sub
            .rcpt_to
            .iter()
            .zip(sub.dsn_notify.iter().chain(std::iter::repeat(&None)))
            .filter(|(_, notify)| wants_failure_dsn(notify.as_deref()))
            .map(|(addr, _)| DsnRecipient {
                address: addr.clone(),
                action: DsnAction::Failed,
                status: status.to_string(),
                diagnostic: diagnostic.map(str::to_string),
            })
            .collect();
        if recipients.is_empty() {
            return None;
        }
        Some(DsnReport {
            notify_to: sub.mail_from.clone(),
            reporting_mta: reporting_mta.into(),
            original_envelope_id: sub.envid.clone(),
            recipients,
            original_message: sub.data.clone(),
            ret: sub.ret.unwrap_or(Ret::Headers),
            arrival: now,
        })
    }

    /// Render the report as an RFC 3464 `multipart/report; report-type=delivery-status` message,
    /// ready to be filed into the sender's INBOX by the node.
    pub fn render(&self) -> Vec<u8> {
        // Deterministic boundary from the content (stable across renders → reproducible tests).
        let boundary = format!("=_dsn_{}", crate::util::hex(&stable_tag(&self.original_message)));
        let date = crate::mime::format_rfc5322_date(self.arrival);
        let overall = if self.recipients.iter().all(|r| r.action == DsnAction::Delivered) {
            "Delivered"
        } else if self.recipients.iter().any(|r| r.action == DsnAction::Failed) {
            "Failure"
        } else {
            "Delayed"
        };

        let mut m = String::new();
        m.push_str(&format!("From: Mail Delivery System <postmaster@{}>\r\n", self.reporting_mta));
        m.push_str(&format!("To: <{}>\r\n", self.notify_to));
        m.push_str(&format!("Subject: Delivery Status Notification ({overall})\r\n"));
        m.push_str(&format!("Date: {date}\r\n"));
        m.push_str("MIME-Version: 1.0\r\n");
        m.push_str("Auto-Submitted: auto-replied\r\n");
        m.push_str(&format!(
            "Content-Type: multipart/report; report-type=delivery-status;\r\n\tboundary=\"{boundary}\"\r\n\r\n"
        ));

        // Part 1 — human-readable notification (RFC 3464 §2.1).
        m.push_str(&format!("--{boundary}\r\n"));
        m.push_str("Content-Type: text/plain; charset=utf-8\r\n\r\n");
        m.push_str("This is an automatically generated Delivery Status Notification.\r\n\r\n");
        for r in &self.recipients {
            m.push_str(r.action.human());
            m.push_str(&format!("\r\n  {} (status {})\r\n", r.address, r.status));
            if let Some(d) = &r.diagnostic {
                m.push_str(&format!("  {d}\r\n"));
            }
        }
        m.push_str("\r\n");

        // Part 2 — machine-readable delivery-status (RFC 3464 §2.1 / §2.3).
        m.push_str(&format!("--{boundary}\r\n"));
        m.push_str("Content-Type: message/delivery-status\r\n\r\n");
        m.push_str(&format!("Reporting-MTA: dns; {}\r\n", self.reporting_mta));
        if let Some(envid) = &self.original_envelope_id {
            m.push_str(&format!("Original-Envelope-Id: {envid}\r\n"));
        }
        m.push_str(&format!("Arrival-Date: {date}\r\n"));
        for r in &self.recipients {
            m.push_str("\r\n");
            m.push_str(&format!("Original-Recipient: rfc822;{}\r\n", r.address));
            m.push_str(&format!("Final-Recipient: rfc822;{}\r\n", r.address));
            m.push_str(&format!("Action: {}\r\n", r.action.wire()));
            m.push_str(&format!("Status: {}\r\n", r.status));
            if let Some(d) = &r.diagnostic {
                m.push_str(&format!("Diagnostic-Code: smtp; {d}\r\n"));
            }
        }
        m.push_str("\r\n");

        // Part 3 — the returned content (RFC 3461 RET): full message or just headers.
        m.push_str(&format!("--{boundary}\r\n"));
        let mut bytes = m.into_bytes();
        match self.ret {
            Ret::Full => {
                bytes.extend_from_slice(b"Content-Type: message/rfc822\r\n\r\n");
                bytes.extend_from_slice(&self.original_message);
            }
            Ret::Headers => {
                bytes.extend_from_slice(b"Content-Type: text/rfc822-headers\r\n\r\n");
                let (headers, _) = crate::mime::header_and_body(&self.original_message);
                bytes.extend_from_slice(&headers);
            }
        }
        if !bytes.ends_with(b"\r\n") {
            bytes.extend_from_slice(b"\r\n");
        }
        bytes.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
        bytes
    }
}

/// Whether a recipient's `NOTIFY=` value asks for a failure DSN. Absent = default (notify on
/// failure); `NEVER` suppresses; otherwise honor an explicit `FAILURE` (RFC 3461 §4.1).
fn wants_failure_dsn(notify: Option<&str>) -> bool {
    match notify {
        None => true,
        Some(v) => {
            let up = v.to_ascii_uppercase();
            if up.split(',').any(|t| t.trim() == "NEVER") {
                false
            } else {
                up.split(',').any(|t| t.trim() == "FAILURE")
            }
        }
    }
}

/// A short deterministic tag over some bytes, for a stable MIME boundary (BLAKE3-256, first 8 B).
fn stable_tag(b: &[u8]) -> [u8; 8] {
    let cid = kotva_core::ContentId::of(b);
    let digest = cid.digest();
    let mut out = [0u8; 8];
    let n = digest.len().min(8);
    out[..n].copy_from_slice(&digest[..n]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::StaticAuthenticator;
    use crate::util::base64_encode;

    fn authed_session() -> SmtpSession<StaticAuthenticator> {
        let mut a = StaticAuthenticator::new();
        a.issue("alice@dmtap.local", "pw", vec![9, 9], "test");
        let mut s = SmtpSession::new(a, true);
        let _ = s.greeting();
        s
    }

    #[test]
    fn malformed_path_param_does_not_panic() {
        // Regression: a MAIL FROM / RCPT TO path where '>' precedes '<' must not slice an inverted
        // range and panic the per-connection session (a remote DoS). Each must return without
        // crashing; the extracted value is best-effort.
        for rest in ["FROM:><", "FROM:> x <", "TO:>", "TO:> <a@b>", "FROM: >bad<"] {
            let kw = if rest.starts_with("TO") { "TO" } else { "FROM" };
            let _ = parse_path_param(rest, kw); // must not panic
        }
        // A well-formed path still extracts the address.
        assert_eq!(parse_path_param("FROM:<a@b.example>", "FROM").as_deref(), Some("a@b.example"));
        assert_eq!(parse_path_param("TO:<x@y.example> SIZE=10", "TO").as_deref(), Some("x@y.example"));
    }

    #[test]
    fn ehlo_advertises_extensions() {
        let mut s = authed_session();
        let reply = s.feed_line("EHLO client.example");
        assert!(reply.contains("250-SIZE"));
        assert!(reply.contains("8BITMIME"));
        assert!(reply.contains("SMTPUTF8"));
        assert!(reply.contains("AUTH PLAIN LOGIN"));
    }

    #[test]
    fn mail_from_must_match_authenticated_identity() {
        // §7.11.2 step 2 / RFC 6409 (security regression): an authenticated submitter may only send
        // AS the address it authenticated as. A spoofed MAIL FROM:<victim@...> is refused fail-closed
        // (550), never signed and relayed; the own address is accepted case-insensitively; the null
        // return-path (<>) is exempt (bounces cannot spoof a victim).
        let mut s = authed_session();
        s.feed_line("EHLO c");
        assert!(s
            .feed_line(&format!("AUTH PLAIN {}", base64_encode(b"\0alice@dmtap.local\0pw")))
            .starts_with("235"));
        let spoof = s.feed_line("MAIL FROM:<victim@paypal.com>");
        assert!(spoof.starts_with("550"), "spoofed sender must be refused: {spoof}");
        assert!(spoof.to_ascii_lowercase().contains("not authorised"), "{spoof}");
        // The authenticated address (case-insensitive) is accepted, and a spoof did not leave state.
        assert!(s.feed_line("MAIL FROM:<Alice@DMTAP.local>").starts_with("250"));

        let mut s2 = authed_session();
        s2.feed_line("EHLO c");
        s2.feed_line(&format!("AUTH PLAIN {}", base64_encode(b"\0alice@dmtap.local\0pw")));
        assert!(s2.feed_line("MAIL FROM:<>").starts_with("250"), "null return-path must be allowed");
    }

    #[test]
    fn full_submission_flow() {
        let mut s = authed_session();
        s.feed_line("EHLO c");
        let cred = base64_encode(b"\0alice@dmtap.local\0pw");
        assert!(s.feed_line(&format!("AUTH PLAIN {cred}")).starts_with("235"));
        assert!(s.feed_line("MAIL FROM:<alice@dmtap.local> SIZE=100").starts_with("250"));
        assert!(s.feed_line("RCPT TO:<bob@example.net>").starts_with("250"));
        assert!(s.feed_line("DATA").starts_with("354"));
        s.feed_line("Subject: Hi");
        s.feed_line("");
        s.feed_line("Hello Bob");
        assert!(s.feed_line(".").starts_with("250"));
        let subs = s.take_submissions();
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].rcpt_to, vec!["bob@example.net"]);
        assert!(subs[0].data.windows(9).any(|w| w == b"Hello Bob"));
    }

    #[test]
    fn quit_line_in_data_body_does_not_close_connection() {
        // Correctness/availability regression: the transport used to compute
        // `quit = line.eq_ignore_ascii_case(b"QUIT")` for EVERY line and drop the connection when
        // true — but during DATA, feed_line_bytes buffers the line as content. A body line equal to
        // `QUIT` therefore tore the connection down mid-message: the terminating `.` was never
        // reached and the mail was silently lost. Close is now the session's decision (wants_close),
        // set ONLY by the QUIT command in Command phase.
        let mut s = authed_session();
        s.feed_line("EHLO c");
        let cred = base64_encode(b"\0alice@dmtap.local\0pw");
        assert!(s.feed_line(&format!("AUTH PLAIN {cred}")).starts_with("235"));
        assert!(s.feed_line("MAIL FROM:<alice@dmtap.local>").starts_with("250"));
        assert!(s.feed_line("RCPT TO:<bob@example.net>").starts_with("250"));
        assert!(s.feed_line("DATA").starts_with("354"));
        assert!(!s.wants_close(), "not closing before any QUIT");
        s.feed_line("Subject: Hi");
        s.feed_line("");
        s.feed_line("QUIT"); // a bare QUIT line INSIDE the body — must be buffered, not a close
        assert!(!s.wants_close(), "a DATA-body QUIT line must NOT request connection close");
        s.feed_line("still here");
        assert!(s.feed_line(".").starts_with("250"), "message completes normally");
        let subs = s.take_submissions();
        assert_eq!(subs.len(), 1, "the message is delivered, not lost");
        // The buffered body carries the literal QUIT line byte-exact.
        assert!(subs[0].data.windows(4).any(|w| w == b"QUIT"), "QUIT line kept as content");
        assert!(subs[0].data.windows(10).any(|w| w == b"still here"));

        // A real QUIT *command* (Command phase) does request close.
        let mut s2 = authed_session();
        s2.feed_line("EHLO c");
        assert!(s2.feed_line("QUIT").starts_with("221"));
        assert!(s2.wants_close(), "the QUIT command requests close");
    }

    #[test]
    fn requires_auth_before_mail() {
        let mut s = authed_session();
        s.feed_line("EHLO c");
        assert!(s.feed_line("MAIL FROM:<x@y>").starts_with("530"));
    }

    #[test]
    fn builds_mote_draft() {
        let draft = build_mote_draft(b"Subject: Test\r\n\r\nbody", 42);
        assert_eq!(draft.headers.subject.as_deref(), Some("Test"));
        assert_eq!(draft.body, b"body");
    }

    #[test]
    fn builds_mote_draft_decodes_rfc2047_subject() {
        // A legacy sender's encoded subject must land in the MOTE as real text, not `=?UTF-8?B?…`.
        let draft = build_mote_draft(
            b"Subject: =?UTF-8?B?0J/RgNC40LLQtdGC?= =?ISO-8859-1?Q?Gr=FC=DFe?=\r\n\r\nbody",
            42,
        );
        assert_eq!(draft.headers.subject.as_deref(), Some("ПриветGrüße"));
    }

    #[test]
    fn eight_bit_data_survives_byte_exact() {
        // We advertise 8BITMIME: a Latin-1 (or any 8-bit) DATA payload must reach the Submission
        // byte-for-byte — no UTF-8 validation, no U+FFFD replacement — and parse with the body
        // untouched.
        let mut s = authed_session();
        s.feed_line("EHLO c");
        let cred = base64_encode(b"\0alice@dmtap.local\0pw");
        s.feed_line(&format!("AUTH PLAIN {cred}"));
        s.feed_line("MAIL FROM:<alice@dmtap.local> BODY=8BITMIME");
        s.feed_line("RCPT TO:<bob@example.net>");
        s.feed_line("DATA");
        s.feed_line_bytes(b"Content-Type: text/plain; charset=iso-8859-1");
        s.feed_line_bytes(b"Content-Transfer-Encoding: 8bit");
        s.feed_line_bytes(b"");
        s.feed_line_bytes(b"Gr\xfc\xdfe aus M\xfcnchen");
        // Dot-unstuffing must also be byte-safe for 8-bit lines.
        s.feed_line_bytes(b".\xe9 dot-stuffed latin-1");
        assert!(s.feed_line_bytes(b".").starts_with("250"));
        let sub = s.take_submissions().pop().unwrap();
        let expected: &[u8] = b"Content-Type: text/plain; charset=iso-8859-1\r\n\
                                Content-Transfer-Encoding: 8bit\r\n\r\n\
                                Gr\xfc\xdfe aus M\xfcnchen\r\n\xe9 dot-stuffed latin-1\r\n";
        assert_eq!(sub.data, expected, "8-bit DATA must survive byte-exact");
        // And the parser must keep those body bytes untouched too.
        let parsed = ParsedMessage::parse(&sub.data);
        assert_eq!(parsed.body, b"Gr\xfc\xdfe aus M\xfcnchen\r\n\xe9 dot-stuffed latin-1\r\n");
    }

    /// Drive a full submission that declares DSN parameters, then generate the failure DSN.
    fn submitted_with_dsn(ret: &str, notify: &str) -> Submission {
        let mut s = authed_session();
        s.feed_line("EHLO c");
        let cred = base64_encode(b"\0alice@dmtap.local\0pw");
        s.feed_line(&format!("AUTH PLAIN {cred}"));
        s.feed_line(&format!("MAIL FROM:<alice@dmtap.local> RET={ret} ENVID=abc123"));
        s.feed_line(&format!("RCPT TO:<bob@example.net> NOTIFY={notify}"));
        s.feed_line("DATA");
        s.feed_line("Subject: Hi");
        s.feed_line("From: alice@dmtap.local");
        s.feed_line("");
        s.feed_line("Hello Bob");
        s.feed_line(".");
        s.take_submissions().pop().unwrap()
    }

    #[test]
    fn captures_dsn_params() {
        let sub = submitted_with_dsn("FULL", "FAILURE");
        assert_eq!(sub.ret, Some(Ret::Full));
        assert_eq!(sub.envid.as_deref(), Some("abc123"));
        assert_eq!(sub.dsn_notify, vec![Some("FAILURE".to_string())]);
    }

    #[test]
    fn generates_failure_dsn_report() {
        let sub = submitted_with_dsn("HDRS", "FAILURE");
        let report =
            DsnReport::failure_for(&sub, "mail.dmtap.local", "5.1.1", Some("550 no such user"), 0)
                .expect("a failure DSN should be produced");
        let bytes = report.render();
        let text = String::from_utf8_lossy(&bytes);
        // Structure: RFC 3464 multipart/report with a machine-readable delivery-status part.
        assert!(text.contains("Content-Type: multipart/report; report-type=delivery-status"), "{text}");
        assert!(text.contains("Content-Type: message/delivery-status"), "{text}");
        assert!(text.contains("Reporting-MTA: dns; mail.dmtap.local"), "{text}");
        assert!(text.contains("Original-Envelope-Id: abc123"), "{text}");
        assert!(text.contains("Final-Recipient: rfc822;bob@example.net"), "{text}");
        assert!(text.contains("Action: failed"), "{text}");
        assert!(text.contains("Status: 5.1.1"), "{text}");
        assert!(text.contains("Diagnostic-Code: smtp; 550 no such user"), "{text}");
        // RET=HDRS returns only the original headers, as text/rfc822-headers.
        assert!(text.contains("Content-Type: text/rfc822-headers"), "{text}");
        assert!(text.contains("Subject: Hi"), "{text}");
        assert!(!text.contains("Hello Bob"), "RET=HDRS must not echo the body: {text}");
    }

    #[test]
    fn auth_when_already_authenticated_is_rejected() {
        let mut s = authed_session();
        s.feed_line("EHLO c");
        let cred = base64_encode(b"\0alice@dmtap.local\0pw");
        assert!(s.feed_line(&format!("AUTH PLAIN {cred}")).starts_with("235"));
        // A second AUTH must be refused (RFC 4954 §4).
        assert!(s.feed_line(&format!("AUTH PLAIN {cred}")).starts_with("503"), "second AUTH must 503");
    }

    #[test]
    fn oversize_data_is_refused() {
        let mut s = authed_session();
        s.set_max_size(64);
        s.feed_line("EHLO c");
        let cred = base64_encode(b"\0alice@dmtap.local\0pw");
        s.feed_line(&format!("AUTH PLAIN {cred}"));
        assert!(s.feed_line("EHLO c").contains("250-SIZE 64"), "advertises the lowered SIZE");
        s.feed_line("MAIL FROM:<alice@dmtap.local>");
        s.feed_line("RCPT TO:<bob@example.net>");
        s.feed_line("DATA");
        for _ in 0..20 {
            s.feed_line("this line pushes the message over the 64-byte limit");
        }
        assert!(s.feed_line(".").starts_with("552"), "over-limit DATA must 552");
        assert!(s.take_submissions().is_empty(), "no submission accepted when oversized");
    }

    #[test]
    fn declared_size_over_limit_rejected_at_mail() {
        let mut s = authed_session();
        s.set_max_size(1000);
        s.feed_line("EHLO c");
        let cred = base64_encode(b"\0alice@dmtap.local\0pw");
        s.feed_line(&format!("AUTH PLAIN {cred}"));
        assert!(s.feed_line("MAIL FROM:<alice@dmtap.local> SIZE=99999").starts_with("552"));
    }

    #[test]
    fn notify_never_suppresses_dsn() {
        let sub = submitted_with_dsn("FULL", "NEVER");
        assert!(DsnReport::failure_for(&sub, "mta", "5.0.0", None, 0).is_none());
    }
}
