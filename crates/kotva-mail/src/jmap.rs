//! JMAP (RFC 8620 Core + RFC 8621 Mail) — the native modern sync surface (spec §8.1). JMAP maps
//! directly onto the MOTE store: Mailboxes are folders, Emails are messages, keywords are flags.
//!
//! This module provides the real request/response envelope types, the Session resource, and
//! handlers for the standard methods over Mailbox / Email / Thread / EmailSubmission
//! (`/get`, `/query`, `/set`, `/changes`), plus blob upload/download and the push (StateChange /
//! EventSource) types. Method arguments are handled as `serde_json::Value` so the wire shape is
//! exactly RFC 8620/8621; the handlers below are the reference projection.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::mime::ParsedMessage;
use crate::store::{Flag, JmapObj, MailStore};

/// A JMAP method invocation `[name, arguments, callId]` (RFC 8620 §3.2).
pub type Invocation = (String, Value, String);

/// A JMAP request object (RFC 8620 §3.3).
#[derive(Debug, Clone, Deserialize)]
pub struct Request {
    pub using: Vec<String>,
    #[serde(rename = "methodCalls")]
    pub method_calls: Vec<Invocation>,
}

/// A JMAP response object (RFC 8620 §3.4).
#[derive(Debug, Clone, Serialize)]
pub struct Response {
    #[serde(rename = "methodResponses")]
    pub method_responses: Vec<Invocation>,
    #[serde(rename = "sessionState")]
    pub session_state: String,
}

/// The capability URIs this server implements.
pub const CAP_CORE: &str = "urn:ietf:params:jmap:core";
pub const CAP_MAIL: &str = "urn:ietf:params:jmap:mail";
pub const CAP_SUBMISSION: &str = "urn:ietf:params:jmap:submission";

/// Build the JMAP Session resource (RFC 8620 §2) for an account at `base_url`.
pub fn session_resource(account_id: &str, base_url: &str, state: &str) -> Value {
    json!({
        "capabilities": {
            CAP_CORE: {
                "maxSizeUpload": 50_000_000u64,
                "maxConcurrentUpload": 4,
                "maxSizeRequest": 10_000_000u64,
                "maxConcurrentRequests": 4,
                "maxCallsInRequest": MAX_CALLS_IN_REQUEST,
                "maxObjectsInGet": MAX_OBJECTS_IN_GET,
                "maxObjectsInSet": MAX_OBJECTS_IN_SET,
                "collationAlgorithms": ["i;ascii-casemap", "i;unicode-casemap"]
            },
            CAP_MAIL: {
                "maxMailboxesPerEmail": null,
                "maxMailboxDepth": null,
                "maxSizeMailboxName": 200,
                "maxSizeAttachmentsPerEmail": 50_000_000u64,
                "emailQuerySortOptions": ["receivedAt", "size", "subject"],
                "mayCreateTopLevelMailbox": true
            },
            CAP_SUBMISSION: { "maxDelayedSend": 0, "submissionExtensions": {} }
        },
        "accounts": {
            account_id: {
                "name": account_id,
                "isPersonal": true,
                "isReadOnly": false,
                "accountCapabilities": { CAP_MAIL: {}, CAP_SUBMISSION: {} }
            }
        },
        "primaryAccounts": { CAP_MAIL: account_id, CAP_SUBMISSION: account_id },
        "username": account_id,
        "apiUrl": format!("{base_url}/jmap/api/"),
        "downloadUrl": format!("{base_url}/jmap/download/{{accountId}}/{{blobId}}/{{name}}"),
        "uploadUrl": format!("{base_url}/jmap/upload/{{accountId}}/"),
        "eventSourceUrl": format!("{base_url}/jmap/eventsource/?types={{types}}&closeafter={{closeafter}}&ping={{ping}}"),
        "state": state
    })
}

/// A JMAP StateChange push object (RFC 8620 §7.1) — emitted over EventSource / WebSocket.
#[derive(Debug, Clone, Serialize)]
pub struct StateChange {
    #[serde(rename = "@type")]
    pub kind: &'static str,
    pub changed: Value,
}

impl StateChange {
    pub fn new(account_id: &str, state: &str) -> StateChange {
        StateChange {
            kind: "StateChange",
            changed: json!({ account_id: { "Email": state, "Mailbox": state } }),
        }
    }
}

/// RFC 8620 §3.6: the advertised — and now enforced — cap on method calls per request. Shared with
/// `session_resource`'s `maxCallsInRequest` so the advertised and enforced values can never drift.
pub(crate) const MAX_CALLS_IN_REQUEST: usize = 16;

/// RFC 8620 §5.1: the advertised — and now enforced — cap on objects returned by a `/get`. Shared
/// with `session_resource`'s `maxObjectsInGet` so advertised and enforced cannot drift. A `/get`
/// (Email, Thread, …) with more requested ids than this — or a null-ids "fetch all" that would
/// exceed it — is rejected with `requestTooLarge` BEFORE the per-id parse loop, so an unbounded or
/// duplicate-packed id list cannot drive unbounded MIME parses / response building.
pub(crate) const MAX_OBJECTS_IN_GET: usize = 500;

/// RFC 8620 §5.3: the advertised — and now enforced — cap on the number of objects a `/set` may
/// create + update + destroy in one call. Shared with `session_resource`'s `maxObjectsInSet`.
pub(crate) const MAX_OBJECTS_IN_SET: usize = 500;

/// Enforce `maxObjectsInSet` (RFC 8620 §5.3) BEFORE a `/set` mutates anything: the total of
/// `create` + `update` + `destroy` must not exceed the cap, or the whole method is rejected with
/// `requestTooLarge` — so an unbounded create/update/destroy map cannot drive unbounded object
/// construction, store mutation, and response building.
fn enforce_set_limit(args: &Value) -> Result<(), Value> {
    let objs = |k: &str| {
        args.get(k)
            .and_then(|v| v.as_object())
            .map_or(0, |m| m.len())
    };
    let n = objs("create")
        + objs("update")
        + args
            .get("destroy")
            .and_then(|v| v.as_array())
            .map_or(0, |a| a.len());
    if n > MAX_OBJECTS_IN_SET {
        return Err(
            json!({ "type": "requestTooLarge", "description": "more than maxObjectsInSet objects in one /set" }),
        );
    }
    Ok(())
}

/// Defense-in-depth cap on cumulative response size. Even within `MAX_CALLS_IN_REQUEST`, a chain of
/// back-referencing `Core/echo` calls can double the response each step (`eval_pointer` returns a full
/// clone for an empty JSON-pointer path, and echo returns it verbatim) — up to `2^N` amplification
/// from a tiny request body. Bounding the running total stops the amplification building gigabytes.
const MAX_RESPONSE_BYTES: usize = 50_000_000;

/// Process a JMAP request against a store, dispatching each method call (RFC 8620 §3.3), with
/// back-reference (`#`) resolution (§3.7) so a client can chain e.g. `Email/query` → `Email/get`.
pub fn process<S: MailStore>(store: &mut S, account_id: &str, req: &Request) -> Response {
    // RFC 8620 §3.6: reject a request that exceeds the advertised call limit (a request-level error,
    // not a per-call methodResponse). Unbounded dispatch enables a Core/echo back-reference
    // amplification DoS, so this is load-bearing, not cosmetic.
    if req.method_calls.len() > MAX_CALLS_IN_REQUEST {
        return Response {
            method_responses: vec![(
                "error".to_string(),
                serde_json::json!({ "type": "requestTooLarge", "description": "too many method calls in one request" }),
                "0".to_string(),
            )],
            session_state: state_string(store),
        };
    }
    let mut responses: Vec<Invocation> = Vec::new();
    let mut total_bytes = 0usize;
    for (name, args, call_id) in &req.method_calls {
        // Resolve any `#name`/ResultReference arguments against the responses so far (§3.7).
        let resolved = match resolve_references(args, &responses) {
            Ok(a) => a,
            Err(e) => {
                responses.push(("error".to_string(), e, call_id.clone()));
                continue;
            }
        };
        let inv = match dispatch(store, account_id, name, &resolved) {
            Ok((rname, rargs)) => (rname, rargs, call_id.clone()),
            Err(err) => ("error".to_string(), err, call_id.clone()),
        };
        // Bound cumulative response size against back-reference amplification.
        total_bytes = total_bytes.saturating_add(inv.1.to_string().len());
        responses.push(inv);
        if total_bytes > MAX_RESPONSE_BYTES {
            responses.push((
                "error".to_string(),
                serde_json::json!({ "type": "requestTooLarge", "description": "cumulative response too large" }),
                call_id.clone(),
            ));
            break;
        }
    }
    Response {
        method_responses: responses,
        session_state: state_string(store),
    }
}

fn dispatch<S: MailStore>(
    store: &mut S,
    account: &str,
    name: &str,
    args: &Value,
) -> Result<(String, Value), Value> {
    match name {
        // Core/echo (RFC 8620 §4): the request arguments are returned verbatim.
        "Core/echo" => Ok(("Core/echo".into(), args.clone())),
        "Mailbox/get" => mailbox_get(store, account, args).map(|v| ("Mailbox/get".into(), v)),
        "Mailbox/query" => Ok(("Mailbox/query".into(), mailbox_query(store, account))),
        "Mailbox/changes" => Ok((
            "Mailbox/changes".into(),
            changes(store, account, JmapObj::Mailbox, args),
        )),
        "Mailbox/set" => mailbox_set(store, account, args).map(|v| ("Mailbox/set".into(), v)),
        "Email/get" => email_get(store, account, args).map(|v| ("Email/get".into(), v)),
        "Email/query" => Ok(("Email/query".into(), email_query(store, account, args))),
        "Email/changes" => Ok((
            "Email/changes".into(),
            changes(store, account, JmapObj::Email, args),
        )),
        "Email/queryChanges" => Ok((
            "Email/queryChanges".into(),
            email_query_changes(store, account, args),
        )),
        "Email/set" => email_set(store, account, args).map(|v| ("Email/set".into(), v)),
        "Thread/get" => thread_get(store, account, args).map(|v| ("Thread/get".into(), v)),
        "Thread/changes" => Ok((
            "Thread/changes".into(),
            changes(store, account, JmapObj::Thread, args),
        )),
        "SearchSnippet/get" => {
            search_snippet_get(store, account, args).map(|v| ("SearchSnippet/get".into(), v))
        }
        "Identity/get" => Ok(("Identity/get".into(), identity_get(account, args))),
        "Identity/changes" => Ok(("Identity/changes".into(), identity_changes(account, args))),
        "Identity/set" => Ok(("Identity/set".into(), identity_set(account, args))),
        "EmailSubmission/set" => Ok(("EmailSubmission/set".into(), submission_set(account, args))),
        "EmailSubmission/get" => Ok(("EmailSubmission/get".into(), submission_get(account, args))),
        "PushSubscription/get" => Ok(("PushSubscription/get".into(), push_subscription_get(args))),
        "PushSubscription/set" => Ok(("PushSubscription/set".into(), push_subscription_set(args))),
        _ => Err(json!({ "type": "unknownMethod", "description": name })),
    }
}

/// The opaque JMAP state token (an encoding of every mailbox's modseq — see
/// [`MailStore::jmap_state`]); every `state`/`queryState`/`newState` field uses it so a client can
/// feed it straight back into a `/changes` call.
fn state_string<S: MailStore>(store: &S) -> String {
    store.jmap_state()
}

// --- back-references (RFC 8620 §3.7) -------------------------------------------------------

/// Resolve `#`/ResultReference argument values against prior method responses. A top-level
/// argument whose value is `{ "resultOf": callId, "name": method, "path": jsonPointer }` is
/// replaced by evaluating the pointer over that earlier response's arguments.
fn resolve_references(args: &Value, prior: &[Invocation]) -> Result<Value, Value> {
    let obj = match args.as_object() {
        Some(o) => o,
        None => return Ok(args.clone()),
    };
    let mut out = serde_json::Map::new();
    // Bound the total bytes materialized by back-reference resolution WITHIN this single call.
    // eval_pointer with an empty path returns a full clone of a prior response (§3.7), and a call
    // may carry any number of `#`-keys — so a call with thousands of distinct `#`-keys each cloning
    // the same large prior response allocates N×R bytes BEFORE process()'s MAX_RESPONSE_BYTES check
    // (which runs only after resolve+dispatch, and only gates SUBSEQUENT calls). Accumulate resolved
    // size here and fail closed once it exceeds the same ceiling, so one call cannot balloon transient
    // memory and OOM the connection thread. Distinct from the cross-call chain-doubling that cap gates.
    let mut resolved_bytes = 0usize;
    for (k, v) in obj {
        // The wire spelling of a back-reference argument is `#<realArgName>`.
        if let Some(real) = k.strip_prefix('#') {
            let rr = v
                .as_object()
                .ok_or_else(|| ref_error("ResultReference not an object"))?;
            let result_of = rr.get("resultOf").and_then(Value::as_str);
            let mname = rr.get("name").and_then(Value::as_str);
            let path = rr.get("path").and_then(Value::as_str);
            let (result_of, mname, path) = match (result_of, mname, path) {
                (Some(a), Some(b), Some(c)) => (a, b, c),
                _ => return Err(ref_error("ResultReference missing resultOf/name/path")),
            };
            let source = prior
                .iter()
                .find(|(n, _, cid)| n == mname && cid == result_of)
                .ok_or_else(|| ref_error("ResultReference target not found"))?;
            let resolved = eval_pointer(&source.1, path)
                .ok_or_else(|| ref_error("bad ResultReference path"))?;
            resolved_bytes = resolved_bytes.saturating_add(resolved.to_string().len());
            if resolved_bytes > MAX_RESPONSE_BYTES {
                return Err(ref_error(
                    "resolved back-references exceed the response-size limit",
                ));
            }
            out.insert(real.to_string(), resolved);
        } else {
            out.insert(k.clone(), v.clone());
        }
    }
    Ok(Value::Object(out))
}

fn ref_error(desc: &str) -> Value {
    json!({ "type": "invalidResultReference", "description": desc })
}

/// Evaluate a JMAP JSON pointer (RFC 8620 §3.7 / RFC 6901) with the `*` array-map extension:
/// `/list/*/id` maps over `list` collecting each element's `id`.
fn eval_pointer(value: &Value, path: &str) -> Option<Value> {
    let path = path.strip_prefix('/').unwrap_or(path);
    if path.is_empty() {
        return Some(value.clone());
    }
    let (token, rest) = match path.split_once('/') {
        Some((t, r)) => (t, r),
        None => (path, ""),
    };
    if token == "*" {
        let arr = value.as_array()?;
        let mut out = Vec::new();
        for item in arr {
            match eval_pointer(item, rest) {
                // A `*` that maps to arrays flattens them (§3.7).
                Some(Value::Array(inner)) => out.extend(inner),
                Some(v) => out.push(v),
                None => {}
            }
        }
        Some(Value::Array(out))
    } else {
        let next = match value {
            Value::Object(m) => m.get(token)?,
            Value::Array(a) => a.get(token.parse::<usize>().ok()?)?,
            _ => return None,
        };
        if rest.is_empty() {
            Some(next.clone())
        } else {
            eval_pointer(next, rest)
        }
    }
}

// --- Mailbox --------------------------------------------------------------------------------

fn mailbox_get<S: MailStore>(store: &S, account: &str, args: &Value) -> Result<Value, Value> {
    // Enforce maxObjectsInGet like Email/Thread/SearchSnippet get: an unbounded `ids` array otherwise
    // drives an O(mailboxes × ids) scan (`filter.iter().any(...)` per mailbox), request-size-amplified
    // and with no cap on the mailbox count either (CREATE has no per-account limit). Reject an
    // oversized id list, and index the filter into a set so the scan is O(mailboxes + ids), not the
    // product.
    let id_set: Option<std::collections::HashSet<&str>> = match args
        .get("ids")
        .and_then(|v| v.as_array())
    {
        Some(f) => {
            if f.len() > MAX_OBJECTS_IN_GET {
                return Err(
                    json!({ "type": "requestTooLarge", "description": "more than maxObjectsInGet ids requested" }),
                );
            }
            Some(f.iter().filter_map(|v| v.as_str()).collect())
        }
        None => None,
    };
    let mut list = Vec::new();
    for name in store.mailbox_names() {
        if let Some(set) = &id_set {
            if !set.contains(name.as_str()) {
                continue;
            }
        }
        // `name` came from `mailbox_names()` above; skip rather than unwrap so a store that can
        // mutate between the two calls (e.g. a shared/concurrent backend) never panics here.
        let mb = match store.mailbox(&name) {
            Some(m) => m,
            None => continue,
        };
        list.push(json!({
            "id": name,
            "name": name,
            "parentId": Value::Null,
            "role": mb.special_use.map(|s| s.jmap_role()),
            "sortOrder": 0,
            "totalEmails": mb.exists(),
            "unreadEmails": mb.unseen(),
            "totalThreads": mb.exists(),
            "unreadThreads": mb.unseen(),
            "myRights": {
                "mayReadItems": true, "mayAddItems": true, "mayRemoveItems": true,
                "maySetSeen": true, "maySetKeywords": true, "mayCreateChild": true,
                "mayRename": true, "mayDelete": true, "maySubmit": true
            },
            "isSubscribed": mb.subscribed
        }));
    }
    Ok(json!({
        "accountId": account,
        "state": state_string(store),
        "list": list,
        "notFound": []
    }))
}

fn mailbox_query<S: MailStore>(store: &S, account: &str) -> Value {
    let ids: Vec<String> = store.mailbox_names();
    json!({
        "accountId": account,
        "queryState": state_string(store),
        "canCalculateChanges": true,
        "position": 0,
        "total": ids.len(),
        "ids": ids
    })
}

// --- Email ----------------------------------------------------------------------------------

fn email_id(mailbox: &str, uid: u32) -> String {
    format!("{mailbox}|{uid}")
}

fn parse_email_id(id: &str) -> Option<(String, u32)> {
    let (mb, uid) = id.rsplit_once('|')?;
    Some((mb.to_string(), uid.parse().ok()?))
}

fn keyword_of(flag: &Flag) -> Option<&'static str> {
    match flag {
        Flag::Seen => Some("$seen"),
        Flag::Flagged => Some("$flagged"),
        Flag::Answered => Some("$answered"),
        Flag::Draft => Some("$draft"),
        Flag::Deleted => Some("$deleted"),
        Flag::Recent => None,
        Flag::Keyword(_) => None,
    }
}

fn flag_of_keyword(kw: &str) -> Flag {
    match kw.to_ascii_lowercase().as_str() {
        "$seen" => Flag::Seen,
        "$flagged" => Flag::Flagged,
        "$answered" => Flag::Answered,
        "$draft" => Flag::Draft,
        "$deleted" => Flag::Deleted,
        other => Flag::Keyword(other.to_string()),
    }
}

fn email_query<S: MailStore>(store: &S, account: &str, args: &Value) -> Value {
    let in_mailbox = args
        .get("filter")
        .and_then(|f| f.get("inMailbox"))
        .and_then(|v| v.as_str());
    let mut ids = Vec::new();
    for name in store.mailbox_names() {
        if let Some(target) = in_mailbox {
            if target != name {
                continue;
            }
        }
        let mb = match store.mailbox(&name) {
            Some(m) => m,
            None => continue,
        };
        for m in &mb.messages {
            ids.push(email_id(&name, m.uid));
        }
    }
    json!({
        "accountId": account,
        "queryState": state_string(store),
        "canCalculateChanges": true,
        "position": 0,
        "total": ids.len(),
        "ids": ids
    })
}

fn email_get<S: MailStore>(store: &S, account: &str, args: &Value) -> Result<Value, Value> {
    let want_ids: Vec<String> = match args.get("ids").and_then(|v| v.as_array()) {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        None => {
            // null ids = all emails.
            let mut all = Vec::new();
            for name in store.mailbox_names() {
                let mb = match store.mailbox(&name) {
                    Some(m) => m,
                    None => continue,
                };
                for m in &mb.messages {
                    all.push(email_id(&name, m.uid));
                }
            }
            all
        }
    };
    // RFC 8620 §5.1: bound the number of objects fetched (advertised maxObjectsInGet). Enforced
    // BEFORE the per-id parse loop, so a huge or duplicate-packed id list (or a null-ids "fetch all"
    // over a large store) cannot drive unbounded MIME parses. NOTE: this bounds the object COUNT, not
    // the response BYTES — each object embeds the full decoded body, so the cumulative-byte bound in
    // the loop below is what stops a duplicate-packed id list from copying one large message N times.
    if want_ids.len() > MAX_OBJECTS_IN_GET {
        return Err(
            json!({ "type": "requestTooLarge", "description": "more than maxObjectsInGet ids requested" }),
        );
    }
    let mut list = Vec::new();
    let mut not_found = Vec::new();
    let mut resp_bytes = 0usize;
    for id in &want_ids {
        match parse_email_id(id).and_then(|(mb, uid)| {
            store
                .mailbox(&mb)
                .and_then(|m| m.by_uid(uid))
                .map(|msg| (mb, msg))
        }) {
            Some((mailbox, msg)) => {
                let parsed = ParsedMessage::parse(&msg.raw);
                // Display text goes through the crate's one i18n path: RFC 2047 for headers,
                // CTE+charset for the body — with the honest problem flag, never a hardcoded
                // `false` over U+FFFD soup (a GB18030 body IS an encoding problem for us).
                let (body_text, encoding_problem) = crate::mime::decoded_body_text(&parsed);
                let keywords: serde_json::Map<String, Value> = msg
                    .flags
                    .iter()
                    .filter_map(keyword_of)
                    .map(|k| (k.to_string(), Value::Bool(true)))
                    .collect();
                let obj = json!({
                    "id": id,
                    "blobId": id,
                    "threadId": thread_id(&parsed, msg.uid),
                    "mailboxIds": { mailbox: true },
                    "keywords": keywords,
                    "size": msg.size(),
                    "receivedAt": crate::mime::format_rfc5322_date(msg.internal_date),
                    "subject": crate::mime::decode_encoded_words(parsed.header("Subject").unwrap_or("")),
                    "from": jmap_addresses(&parsed, "From"),
                    "to": jmap_addresses(&parsed, "To"),
                    "cc": jmap_addresses(&parsed, "Cc"),
                    "messageId": parsed.header("Message-ID").map(|s| vec![s.trim_matches(|c| c=='<'||c=='>')]),
                    "preview": preview(&parsed),
                    "hasAttachment": matches!(parsed.structure, crate::mime::BodyPart::Multipart{..}),
                    "bodyValues": {
                        "1": { "value": body_text, "isEncodingProblem": encoding_problem, "isTruncated": false }
                    },
                    "textBody": [ { "partId": "1", "type": "text/plain" } ]
                });
                // Bound cumulative response BYTES during the loop, matching resolve_references. The
                // count cap alone permits N duplicate ids each embedding the full decoded body (or N
                // distinct large messages), materializing GBs before process()'s post-hoc byte check
                // — an OOM. Fail closed once this call's output would exceed the response ceiling.
                resp_bytes = resp_bytes.saturating_add(obj.to_string().len());
                list.push(obj);
                if resp_bytes > MAX_RESPONSE_BYTES {
                    return Err(
                        json!({ "type": "requestTooLarge", "description": "Email/get response exceeds the response-size limit" }),
                    );
                }
            }
            None => not_found.push(id.clone()),
        }
    }
    Ok(json!({
        "accountId": account,
        "state": state_string(store),
        "list": list,
        "notFound": not_found
    }))
}

fn jmap_addresses(p: &ParsedMessage, header: &str) -> Value {
    let addrs = p.addresses(header);
    if addrs.is_empty() {
        return Value::Null;
    }
    Value::Array(
        addrs
            .into_iter()
            .map(|a| {
                let email = match (a.mailbox, a.host) {
                    (Some(m), Some(h)) => format!("{m}@{h}"),
                    (Some(m), None) => m,
                    _ => String::new(),
                };
                // Display-names arrive RFC-2047-encoded from legacy senders; decode at this JMAP
                // boundary (the IMAP ENVELOPE keeps the raw form — those clients self-decode).
                json!({ "name": a.name.map(|n| crate::mime::decode_encoded_words(&n)), "email": email })
            })
            .collect(),
    )
}

fn thread_id(p: &ParsedMessage, uid: u32) -> String {
    p.header("Message-ID")
        .map(|m| format!("T{}", m.trim_matches(|c| c == '<' || c == '>')))
        .unwrap_or_else(|| format!("T{uid}"))
}

fn preview(p: &ParsedMessage) -> String {
    // Same decode as bodyValues, so the preview shows text, not base64/mojibake, for legacy mail.
    let (text, _) = crate::mime::decoded_body_text(p);
    text.chars().take(200).collect()
}

fn email_set<S: MailStore>(store: &mut S, account: &str, args: &Value) -> Result<Value, Value> {
    enforce_set_limit(args)?;
    let mut created = serde_json::Map::new();
    let mut not_created = serde_json::Map::new();
    let mut updated = serde_json::Map::new();
    let mut not_updated = serde_json::Map::new();
    let mut destroyed = Vec::new();
    let mut not_destroyed = serde_json::Map::new();

    // create: compose a brand-new Email object into a MOTE-backed message (RFC 8621 §4.6).
    if let Some(create) = args.get("create").and_then(|v| v.as_object()) {
        for (cid, obj) in create {
            match compose_email(store, obj) {
                Ok((mailbox, uid, size)) => {
                    let id = email_id(&mailbox, uid);
                    created.insert(
                        cid.clone(),
                        json!({
                            "id": id,
                            "blobId": id,
                            "threadId": format!("T{uid}"),
                            "size": size
                        }),
                    );
                }
                Err(e) => {
                    not_created.insert(cid.clone(), e);
                }
            }
        }
    }

    // update: change keywords (flags) on existing emails.
    if let Some(update) = args.get("update").and_then(|v| v.as_object()) {
        for (id, patch) in update {
            match parse_email_id(id) {
                Some((mb, uid)) => {
                    let ok = apply_email_update(store, &mb, uid, patch);
                    if ok {
                        updated.insert(id.clone(), Value::Null);
                    } else {
                        not_updated.insert(id.clone(), json!({ "type": "notFound" }));
                    }
                }
                None => {
                    not_updated.insert(id.clone(), json!({ "type": "notFound" }));
                }
            }
        }
    }

    // destroy: remove emails.
    if let Some(list) = args.get("destroy").and_then(|v| v.as_array()) {
        for id in list.iter().filter_map(|v| v.as_str()) {
            match parse_email_id(id) {
                Some((mb, uid)) => {
                    if let Some(m) = store.mailbox_mut(&mb) {
                        if let Some(pos) = m.index_of_uid(uid) {
                            // remove_at records the vanished UID so /changes can report `destroyed`.
                            m.remove_at(pos);
                            destroyed.push(Value::String(id.to_string()));
                            continue;
                        }
                    }
                    not_destroyed.insert(id.to_string(), json!({ "type": "notFound" }));
                }
                None => {
                    not_destroyed.insert(id.to_string(), json!({ "type": "notFound" }));
                }
            }
        }
    }

    Ok(json!({
        "accountId": account,
        "oldState": "0",
        "newState": state_string(store),
        "created": Value::Object(created),
        "updated": Value::Object(updated),
        "destroyed": destroyed,
        "notCreated": Value::Object(not_created),
        "notUpdated": Value::Object(not_updated),
        "notDestroyed": Value::Object(not_destroyed)
    }))
}

/// Compose a JMAP `Email` create object (RFC 8621 §4.6) into RFC 5322 bytes and file it into the
/// store. Returns `(mailbox, uid, size)` or a JMAP SetError. This is the JMAP→MIME compose path:
/// `from`/`to`/`cc`/`subject` become headers, `bodyValues`+`textBody` become the body, `keywords`
/// become flags. A real node would additionally build a native MOTE draft for outbound send.
fn compose_email<S: MailStore>(store: &mut S, obj: &Value) -> Result<(String, u32, usize), Value> {
    // Destination mailbox: the first (true) entry of mailboxIds, defaulting to Drafts→INBOX.
    let mailbox = obj
        .get("mailboxIds")
        .and_then(|v| v.as_object())
        .and_then(|m| {
            m.iter()
                .find(|(_, v)| v.as_bool() == Some(true))
                .map(|(k, _)| k.clone())
        })
        .filter(|m| store.mailbox(m).is_some())
        .or_else(|| store.mailbox("Drafts").map(|_| "Drafts".to_string()))
        .unwrap_or_else(|| "INBOX".to_string());
    if store.mailbox(&mailbox).is_none() {
        return Err(
            json!({ "type": "notFound", "description": "mailboxIds references no such mailbox" }),
        );
    }

    let mut headers = String::new();
    // `name`/`email` come from the request body (untrusted JSON) — sanitize before splicing into a
    // raw header line, same as `subject` below, so a crafted address can't inject a sibling header
    // or an early blank line via an embedded CR/LF.
    let addr_header = |key: &str, label: &str, out: &mut String| {
        if let Some(list) = obj.get(key).and_then(|v| v.as_array()) {
            let rendered: Vec<String> = list
                .iter()
                .filter_map(|a| {
                    let email = sanitize_header(a.get("email").and_then(Value::as_str)?);
                    Some(match a.get("name").and_then(Value::as_str) {
                        // Encode the display-name for the wire (RFC 2047 for non-ASCII, quoting
                        // for phrase specials) — a raw 8-bit name would be mangled downstream.
                        Some(n) if !n.is_empty() => format!(
                            "{} <{email}>",
                            crate::mime::encode_display_name(&sanitize_header(n))
                        ),
                        _ => email,
                    })
                })
                .collect();
            if !rendered.is_empty() {
                out.push_str(&format!("{label}: {}\r\n", rendered.join(", ")));
            }
        }
    };
    addr_header("from", "From", &mut headers);
    addr_header("to", "To", &mut headers);
    addr_header("cc", "Cc", &mut headers);
    if let Some(subject) = obj.get("subject").and_then(Value::as_str) {
        // RFC 2047-encode non-ASCII subjects at compose time so the stored message is already
        // legacy-wire-legal (Email/get decodes it back, so the round trip is display-lossless).
        headers.push_str(&format!(
            "Subject: {}\r\n",
            crate::mime::encode_header_value(&sanitize_header(subject))
        ));
    }
    headers.push_str(&format!(
        "Date: {}\r\n",
        crate::mime::format_rfc5322_date(0)
    ));
    headers.push_str("MIME-Version: 1.0\r\n");
    headers.push_str("Content-Type: text/plain; charset=utf-8\r\n");

    // Body: the text part referenced by textBody[0].partId, else the first bodyValue.
    let body_values = obj.get("bodyValues").and_then(|v| v.as_object());
    let part_id = obj
        .get("textBody")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|p| p.get("partId"))
        .and_then(Value::as_str);
    let body = body_values
        .and_then(|bv| {
            part_id
                .and_then(|pid| bv.get(pid))
                .or_else(|| bv.values().next())
                .and_then(|v| v.get("value"))
                .and_then(Value::as_str)
        })
        .unwrap_or("")
        .to_string();

    let mut raw = headers.into_bytes();
    raw.extend_from_slice(b"\r\n");
    raw.extend_from_slice(body.as_bytes());
    let size = raw.len();

    let flags: Vec<Flag> = obj
        .get("keywords")
        .and_then(|v| v.as_object())
        .map(|kw| kw.keys().map(|k| flag_of_keyword(k)).collect())
        .unwrap_or_default();

    let uid = store
        .mailbox_mut(&mailbox)
        .map(|mb| mb.append(raw, flags, 0))
        .ok_or_else(|| json!({ "type": "serverFail" }))?;
    Ok((mailbox, uid, size))
}

/// Strip CR/LF from a header value so a composed Email can't inject extra headers (RFC 5322 §2.2).
fn sanitize_header(v: &str) -> String {
    v.chars().filter(|c| *c != '\r' && *c != '\n').collect()
}

fn apply_email_update<S: MailStore>(store: &mut S, mb: &str, uid: u32, patch: &Value) -> bool {
    let mailbox = match store.mailbox_mut(mb) {
        Some(m) => m,
        None => return false,
    };
    let msg = match mailbox.messages.iter_mut().find(|m| m.uid == uid) {
        Some(m) => m,
        None => return false,
    };
    let obj = match patch.as_object() {
        Some(o) => o,
        None => return false,
    };
    // Full keyword replacement: "keywords": { "$seen": true, ... }
    if let Some(kw) = obj.get("keywords").and_then(|v| v.as_object()) {
        let recent = msg.has_flag(&Flag::Recent);
        msg.flags = kw.keys().map(|k| flag_of_keyword(k)).collect();
        if recent {
            msg.set_flag(Flag::Recent);
        }
    }
    // Patch form: "keywords/$seen": true|null
    for (k, v) in obj {
        if let Some(kw) = k.strip_prefix("keywords/") {
            let flag = flag_of_keyword(kw);
            if v.is_null() || v == &Value::Bool(false) {
                msg.clear_flag(&flag);
            } else {
                msg.set_flag(flag);
            }
        }
    }
    mailbox.highest_modseq += 1;
    let ms = mailbox.highest_modseq;
    if let Some(m) = mailbox.messages.iter_mut().find(|m| m.uid == uid) {
        m.modseq = ms;
    }
    true
}

fn thread_get<S: MailStore>(store: &S, account: &str, args: &Value) -> Result<Value, Value> {
    let want: Vec<String> = args
        .get("ids")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    // RFC 8620 §5.1: bound the requested-id count before ANY per-message work. The old form was
    // O(requested_ids × total_messages) FULL MIME parses — a small (or duplicate-packed) id list
    // forced ~1e9 parses, an amplification DoS.
    if want.len() > MAX_OBJECTS_IN_GET {
        return Err(
            json!({ "type": "requestTooLarge", "description": "more than maxObjectsInGet ids requested" }),
        );
    }
    // Build threadId -> [emailId] in a SINGLE pass over all messages, each parsed at most once via
    // the memoized cache (shared with the IMAP path), so answering R ids is O(M + R), not O(R × M).
    let mut by_thread: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for name in store.mailbox_names() {
        let mb = match store.mailbox(&name) {
            Some(m) => m,
            None => continue,
        };
        for m in &mb.messages {
            let tid = thread_id(m.parsed_cached(), m.uid);
            by_thread
                .entry(tid)
                .or_default()
                .push(email_id(&name, m.uid));
        }
    }
    let list: Vec<Value> = want
        .iter()
        .map(
            |tid| json!({ "id": tid, "emailIds": by_thread.get(tid).cloned().unwrap_or_default() }),
        )
        .collect();
    Ok(json!({ "accountId": account, "state": state_string(store), "list": list, "notFound": [] }))
}

fn submission_set(account: &str, args: &Value) -> Value {
    // Accept EmailSubmission/set create requests, echoing them as created (the actual send is the
    // node's outbound MOTE path, §8.2). This records intent; no central relay is implied (§8.5).
    let mut created = serde_json::Map::new();
    if let Some(create) = args.get("create").and_then(|v| v.as_object()) {
        for (cid, sub) in create {
            created.insert(
                cid.clone(),
                json!({
                    "id": format!("sub-{cid}"),
                    "sendAt": crate::mime::format_rfc5322_date(0),
                    "undoStatus": "final",
                    "emailId": sub.get("emailId").cloned().unwrap_or(Value::Null)
                }),
            );
        }
    }
    json!({
        "accountId": account,
        "oldState": "0",
        "newState": "0",
        "created": Value::Object(created),
        "updated": {},
        "destroyed": [],
        "notCreated": {}
    })
}

/// `EmailSubmission/get` (RFC 8621 §7 / spec §19.9): read the sender-retry state (§4.7) of a
/// submission. This projection does not persist submission objects (the node's outbound state
/// machine owns them), so requested ids are reported as `notFound` rather than fabricated.
fn submission_get(account: &str, args: &Value) -> Value {
    let not_found: Vec<Value> = args
        .get("ids")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter(|v| v.is_string()).cloned().collect())
        .unwrap_or_default();
    json!({ "accountId": account, "state": "0", "list": [], "notFound": not_found })
}

/// `Foo/changes` (RFC 8620 §5.2): a real delta computed from the store's modseq change-log
/// ([`MailStore::jmap_changes`]). `sinceState` is required; an unparseable token yields the
/// `cannotCalculateChanges` error so the client falls back to a full `/query`+`/get`.
fn changes<S: MailStore>(store: &S, account: &str, obj: JmapObj, args: &Value) -> Value {
    let since = args
        .get("sinceState")
        .and_then(Value::as_str)
        .unwrap_or("0");
    match store.jmap_changes(obj, since) {
        Some(c) => json!({
            "accountId": account,
            "oldState": c.old_state,
            "newState": c.new_state,
            "hasMoreChanges": c.has_more,
            "created": c.created,
            "updated": c.updated,
            "destroyed": c.destroyed
        }),
        None => json!({
            "type": "cannotCalculateChanges",
            "description": "sinceState is not a recognizable state token"
        }),
    }
}

// --- Mailbox/set (RFC 8621 §2.5) -----------------------------------------------------------

/// `Mailbox/set`: create / update (rename, subscribe) / destroy mailboxes, mapping onto the
/// [`MailStore`] mutators. Roles and `parentId` from the create object are honored where the store
/// supports them (folder names are the ids in this projection).
fn mailbox_set<S: MailStore>(store: &mut S, account: &str, args: &Value) -> Result<Value, Value> {
    enforce_set_limit(args)?;
    let mut created = serde_json::Map::new();
    let mut not_created = serde_json::Map::new();
    let mut updated = serde_json::Map::new();
    let mut not_updated = serde_json::Map::new();
    let mut destroyed = Vec::new();
    let mut not_destroyed = serde_json::Map::new();

    if let Some(create) = args.get("create").and_then(|v| v.as_object()) {
        for (cid, obj) in create {
            let name = obj.get("name").and_then(Value::as_str).unwrap_or("");
            if name.is_empty() {
                not_created.insert(
                    cid.clone(),
                    json!({ "type": "invalidProperties", "properties": ["name"] }),
                );
                continue;
            }
            // A parentId prefixes the child name with the parent's path ("Parent/Child").
            let full = match obj.get("parentId").and_then(Value::as_str) {
                Some(p) if !p.is_empty() => format!("{p}/{name}"),
                _ => name.to_string(),
            };
            match store.create(&full) {
                Ok(()) => {
                    created.insert(cid.clone(), json!({ "id": full, "totalEmails": 0, "unreadEmails": 0, "totalThreads": 0, "unreadThreads": 0, "isSubscribed": true }));
                }
                Err(e) => {
                    not_created.insert(
                        cid.clone(),
                        json!({ "type": "invalidArguments", "description": e.to_string() }),
                    );
                }
            }
        }
    }

    if let Some(update) = args.get("update").and_then(|v| v.as_object()) {
        for (id, patch) in update {
            let mut ok = true;
            if let Some(newname) = patch.get("name").and_then(Value::as_str) {
                ok = store.rename(id, newname).is_ok();
            }
            if ok {
                if let Some(sub) = patch.get("isSubscribed").and_then(Value::as_bool) {
                    // The renamed target may now carry the new name.
                    let target = patch.get("name").and_then(Value::as_str).unwrap_or(id);
                    if let Some(mb) = store.mailbox_mut(target) {
                        mb.subscribed = sub;
                    }
                }
                updated.insert(id.clone(), Value::Null);
            } else {
                not_updated.insert(id.clone(), json!({ "type": "notFound" }));
            }
        }
    }

    if let Some(list) = args.get("destroy").and_then(|v| v.as_array()) {
        for id in list.iter().filter_map(Value::as_str) {
            match store.delete(id) {
                Ok(()) => destroyed.push(Value::String(id.to_string())),
                Err(e) => {
                    not_destroyed.insert(
                        id.to_string(),
                        json!({ "type": "notFound", "description": e.to_string() }),
                    );
                }
            }
        }
    }

    Ok(json!({
        "accountId": account,
        "oldState": "0",
        "newState": state_string(store),
        "created": Value::Object(created),
        "updated": Value::Object(updated),
        "destroyed": destroyed,
        "notCreated": Value::Object(not_created),
        "notUpdated": Value::Object(not_updated),
        "notDestroyed": Value::Object(not_destroyed)
    }))
}

// --- Email/queryChanges (RFC 8621 §4.5) ----------------------------------------------------

/// `Email/queryChanges`: since we compute query results directly from the store, report the added
/// items from the modseq delta and the removed items from the vanished log (RFC 8620 §5.6).
fn email_query_changes<S: MailStore>(store: &S, account: &str, args: &Value) -> Value {
    let since = args
        .get("sinceQueryState")
        .and_then(Value::as_str)
        .unwrap_or("0");
    match store.jmap_changes(JmapObj::Email, since) {
        Some(c) => {
            let added: Vec<Value> = c
                .created
                .iter()
                .chain(c.updated.iter())
                .enumerate()
                .map(|(idx, id)| json!({ "id": id, "index": idx }))
                .collect();
            json!({
                "accountId": account,
                "oldQueryState": c.old_state,
                "newQueryState": c.new_state,
                "removed": c.destroyed,
                "added": added
            })
        }
        None => json!({ "type": "cannotCalculateChanges" }),
    }
}

// --- SearchSnippet/get (RFC 8621 §5) -------------------------------------------------------

/// `SearchSnippet/get`: return subject/preview snippets for the given emails, highlighting the
/// filter's text terms with `<mark>…</mark>` (a reference highlighter over the plaintext body).
fn search_snippet_get<S: MailStore>(
    store: &S,
    account: &str,
    args: &Value,
) -> Result<Value, Value> {
    let terms: Vec<String> = args
        .get("filter")
        .and_then(collect_filter_terms)
        .unwrap_or_default();
    let ids: Vec<String> = args
        .get("emailIds")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    // RFC 8620 §5.1 (same cap as the sibling /get handlers): a snippet request is per-email work, so
    // bound the id count BEFORE the parse loop — a duplicate-packed emailIds list would otherwise
    // force one uncached MIME parse + body decode per id, an amplification DoS.
    if ids.len() > MAX_OBJECTS_IN_GET {
        return Err(
            json!({ "type": "requestTooLarge", "description": "more than maxObjectsInGet emailIds requested" }),
        );
    }
    let mut list = Vec::new();
    for id in ids {
        let msg = parse_email_id(&id)
            .and_then(|(mb, uid)| store.mailbox(&mb).and_then(|m| m.by_uid(uid)));
        let (subject, preview) = match msg {
            Some(m) => {
                let p = m.parsed_cached();
                (
                    // Snippets highlight what the user sees — the decoded subject, not raw 2047.
                    highlight(
                        &crate::mime::decode_encoded_words(p.header("Subject").unwrap_or("")),
                        &terms,
                    ),
                    highlight(&preview(p), &terms),
                )
            }
            None => (Value::Null, Value::Null),
        };
        list.push(json!({ "emailId": id, "subject": subject, "preview": preview }));
    }
    Ok(json!({ "accountId": account, "list": list, "notFound": [] }))
}

/// Maximum filter terms collected from one JMAP filter. A FilterOperator `conditions` array is
/// bounded only by the request body size (serde_json caps nesting depth, not array width), so an
/// uncapped collect would return millions of terms — and highlight() calls find_ci once PER term PER
/// requested email, each rebuilding an O(text) tagged Vec, an O(terms·emails·text) CPU/allocation
/// blow-up. A real search carries a handful; beyond the cap, further terms are ignored.
const MAX_FILTER_TERMS: usize = 64;

/// Collect the text terms from a JMAP Email filter (`text`/`subject`/`body`/`from`/`to`), bounded at
/// [`MAX_FILTER_TERMS`] so a wide `conditions` array cannot amplify snippet highlighting.
fn collect_filter_terms(filter: &Value) -> Option<Vec<String>> {
    let mut out = Vec::new();
    collect_filter_terms_into(filter, &mut out);
    Some(out)
}

fn collect_filter_terms_into(filter: &Value, out: &mut Vec<String>) {
    if out.len() >= MAX_FILTER_TERMS {
        return;
    }
    if let Some(obj) = filter.as_object() {
        for key in ["text", "subject", "body", "from", "to"] {
            if out.len() >= MAX_FILTER_TERMS {
                return;
            }
            if let Some(v) = obj.get(key).and_then(Value::as_str) {
                out.push(v.to_string());
            }
        }
        // FilterOperator (AND/OR/NOT) with nested conditions.
        if let Some(conds) = obj.get("conditions").and_then(|v| v.as_array()) {
            for c in conds {
                if out.len() >= MAX_FILTER_TERMS {
                    return;
                }
                collect_filter_terms_into(c, out);
            }
        }
    }
}

/// Case-insensitively locate `needle_lower` (already lowercased, as a `char` slice) inside
/// `haystack`, returning the match's byte range **in `haystack`** (always on original char
/// boundaries). `str::to_lowercase()` is not length-preserving — `'İ'` (U+0130) lowercases to two
/// chars, `'ß'` to `"ss"` — so searching a lowercased copy and slicing the ORIGINAL with those
/// offsets panics (or mis-slices) on non-ASCII, attacker-controlled subject/body text. This maps
/// every lowercased char back to the byte range of the original char that produced it, so the
/// returned range is always a valid slice of `haystack`.
fn find_ci(haystack: &str, needle_lower: &[char]) -> Option<(usize, usize)> {
    if needle_lower.is_empty() {
        return None;
    }
    // Flatten `haystack` into lowercased chars, each tagged with its source char's byte range.
    let tagged: Vec<(usize, usize, char)> = haystack
        .char_indices()
        .flat_map(|(i, c)| {
            let end = i + c.len_utf8();
            c.to_lowercase().map(move |lc| (i, end, lc))
        })
        .collect();
    let n = needle_lower.len();
    if tagged.len() < n {
        return None;
    }
    (0..=tagged.len() - n)
        .find(|&start| (0..n).all(|k| tagged[start + k].2 == needle_lower[k]))
        .map(|start| (tagged[start].0, tagged[start + n - 1].1))
}

/// Highlight each term (case-insensitive) in `text` with `<mark>…</mark>`; `Null` when text empty.
fn highlight(text: &str, terms: &[String]) -> Value {
    if text.is_empty() {
        return Value::Null;
    }
    // A snippet is a preview, and the subject it highlights is attacker-controlled and unbounded in
    // size (a header value is not capped by MAX_HEADER_LINES). find_ci below builds a per-char tagged
    // Vec (~24 B/char), so a multi-MiB subject would drive a ~24× transient allocation per
    // SearchSnippet/get. Cap the highlighted text to a display-reasonable prefix (char-boundary safe);
    // no real subject/preview approaches this (preview itself is already 200 chars).
    const MAX_HIGHLIGHT_CHARS: usize = 10_000;
    let text: &str = match text.char_indices().nth(MAX_HIGHLIGHT_CHARS) {
        Some((byte_idx, _)) => &text[..byte_idx],
        None => text,
    };
    let mut result = text.to_string();
    for term in terms {
        if term.is_empty() {
            continue;
        }
        let needle: Vec<char> = term.to_lowercase().chars().collect();
        if let Some((pos, end)) = find_ci(&result, &needle) {
            result = format!(
                "{}<mark>{}</mark>{}",
                &result[..pos],
                &result[pos..end],
                &result[end..]
            );
        }
    }
    Value::String(result)
}

// --- Identity/* (RFC 8621 §6) --------------------------------------------------------------

/// The default sending identity for an account — the address the node presents (spec §8.2). A real
/// node lists every app-address/alias; the reference exposes one derived from the account id.
fn default_identity(account: &str) -> Value {
    json!({
        "id": "I0",
        "name": account,
        "email": account,
        "replyTo": Value::Null,
        "bcc": Value::Null,
        "textSignature": "",
        "htmlSignature": "",
        "mayDelete": false
    })
}

fn identity_get(account: &str, args: &Value) -> Value {
    let want = args.get("ids").and_then(|v| v.as_array());
    let id = default_identity(account);
    let list = match want {
        Some(arr) if !arr.iter().any(|v| v.as_str() == Some("I0")) => vec![],
        _ => vec![id],
    };
    json!({ "accountId": account, "state": "0", "list": list, "notFound": [] })
}

fn identity_changes(account: &str, args: &Value) -> Value {
    let since = args
        .get("sinceState")
        .and_then(Value::as_str)
        .unwrap_or("0");
    json!({
        "accountId": account,
        "oldState": since,
        "newState": "0",
        "hasMoreChanges": false,
        "created": [],
        "updated": [],
        "destroyed": []
    })
}

/// `Identity/set`: accept created identities (echoing a server-assigned id) and reject destroying
/// the built-in default. Full persistence lives in the node's identity store (spec §8.2); this is
/// the wire-correct reference projection.
fn identity_set(account: &str, args: &Value) -> Value {
    let mut created = serde_json::Map::new();
    let mut not_destroyed = serde_json::Map::new();
    if let Some(create) = args.get("create").and_then(|v| v.as_object()) {
        for (cid, obj) in create {
            let email = obj.get("email").and_then(Value::as_str).unwrap_or(account);
            created.insert(
                cid.clone(),
                json!({ "id": format!("I-{cid}"), "email": email, "mayDelete": true }),
            );
        }
    }
    if let Some(list) = args.get("destroy").and_then(|v| v.as_array()) {
        for id in list.iter().filter_map(Value::as_str) {
            if id == "I0" {
                not_destroyed.insert(
                    id.to_string(),
                    json!({ "type": "forbidden", "description": "default identity" }),
                );
            }
        }
    }
    json!({
        "accountId": account,
        "oldState": "0",
        "newState": "0",
        "created": Value::Object(created),
        "updated": {},
        "destroyed": [],
        "notCreated": {},
        "notUpdated": {},
        "notDestroyed": Value::Object(not_destroyed)
    })
}

// --- PushSubscription/* (RFC 8620 §7.2) ----------------------------------------------------

/// `PushSubscription/get`: subscriptions are per-connection device state, not account data, so a
/// fresh session lists none (RFC 8620 §7.2 — `accountId` is explicitly absent from the response).
fn push_subscription_get(args: &Value) -> Value {
    let _ = args;
    json!({ "state": "0", "list": [], "notFound": [] })
}

/// `PushSubscription/set`: accept a created subscription, echoing a server id + a `verificationCode`
/// the server would confirm out-of-band before delivering (RFC 8620 §7.2.2).
fn push_subscription_set(args: &Value) -> Value {
    let mut created = serde_json::Map::new();
    if let Some(create) = args.get("create").and_then(|v| v.as_object()) {
        for (cid, obj) in create {
            let url = obj.get("url").and_then(Value::as_str).unwrap_or("");
            let code = crate::util::hex(kotva_core::ContentId::of(url.as_bytes()).digest());
            created.insert(cid.clone(), json!({ "id": format!("P-{cid}"), "keys": Value::Null, "verificationCode": &code[..16] }));
        }
    }
    json!({
        "oldState": "0",
        "newState": "0",
        "created": Value::Object(created),
        "updated": {},
        "destroyed": [],
        "notCreated": {},
        "notUpdated": {},
        "notDestroyed": {}
    })
}

// --- Blob up/download ----------------------------------------------------------------------

/// Handle a blob upload (RFC 8620 §6.1): returns the upload response object. The blob id is the
/// content address of the bytes (spec §2.2 content addressing), tying JMAP blobs to MOTE ids.
pub fn blob_upload(account: &str, bytes: &[u8], content_type: &str) -> Value {
    let blob_id = crate::util::hex(kotva_core::ContentId::of(bytes).digest());
    json!({
        "accountId": account,
        "blobId": blob_id,
        "type": content_type,
        "size": bytes.len()
    })
}

/// Resolve a blob download (RFC 8620 §6.2) for an Email id → the raw RFC 5322 bytes.
pub fn blob_download<S: MailStore>(store: &S, blob_id: &str) -> Option<Vec<u8>> {
    let (mb, uid) = parse_email_id(blob_id)?;
    store.mailbox(&mb)?.by_uid(uid).map(|m| m.raw.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemoryStore;

    fn store_with_mail() -> MemoryStore {
        let mut s = MemoryStore::empty();
        s.deliver_raw(
            "INBOX",
            b"From: Alice <alice@example.com>\r\nSubject: Hello\r\nMessage-ID: <m1@x>\r\n\r\nHi Bob".to_vec(),
            vec![],
            1_752_000_000_000,
        );
        s
    }

    fn call(store: &mut MemoryStore, name: &str, args: Value) -> Value {
        let req = Request {
            using: vec![CAP_MAIL.into()],
            method_calls: vec![(name.into(), args, "c1".into())],
        };
        let resp = process(store, "acct1", &req);
        resp.method_responses[0].1.clone()
    }

    /// M1: a non-ASCII subject/body whose `to_lowercase()` changes byte length (e.g. `'İ'`
    /// U+0130 lowercases to two chars) must not panic when a search term is highlighted. The old
    /// code searched a lowercased copy but sliced the original with the lowercased byte offsets,
    /// panicking (out-of-bounds / non-char-boundary) on attacker-controlled received text.
    #[test]
    fn highlight_non_ascii_subject_does_not_panic() {
        // "AİB": 'İ' is U+0130 (2 bytes) whose lowercase is "i̇" (3 bytes) — a length change that
        // pushes the lowercased offset of "b" past the original string's length.
        let v = highlight("AİB", &["b".to_string()]);
        assert_eq!(v, Value::String("Aİ<mark>B</mark>".to_string()));
        // Matching the multi-byte char itself snaps to its original byte range.
        let v2 = highlight("İstanbul", &["i".to_string()]);
        assert_eq!(v2, Value::String("<mark>İ</mark>stanbul".to_string()));
        // A term that does not occur leaves the text unchanged (and still no panic).
        let v3 = highlight("Grüße über İ", &["zzz".to_string()]);
        assert_eq!(v3, Value::String("Grüße über İ".to_string()));
    }

    #[test]
    fn session_resource_serializes() {
        let s = session_resource("acct1", "https://node.dmtap.local", "0");
        assert_eq!(s["primaryAccounts"][CAP_MAIL], json!("acct1"));
        assert!(s["apiUrl"].as_str().unwrap().ends_with("/jmap/api/"));
        // The advertised call limit is the shared enforced constant (no advertise/enforce drift).
        assert_eq!(
            s["capabilities"]["urn:ietf:params:jmap:core"]["maxCallsInRequest"],
            json!(MAX_CALLS_IN_REQUEST)
        );
    }

    #[test]
    fn get_handlers_enforce_max_objects_in_get() {
        // Security regression (RFC 8620 §5.1): Email/get and Thread/get must reject an id list larger
        // than maxObjectsInGet with requestTooLarge BEFORE any per-id MIME parse — else an unbounded
        // or duplicate-packed id list drives an O(ids) or O(ids×messages) parse DoS.
        let mut store = store_with_mail();
        let over: Vec<Value> = (0..=MAX_OBJECTS_IN_GET)
            .map(|i| json!(format!("x_{i}")))
            .collect();
        for method in ["Email/get", "Thread/get"] {
            let req = Request {
                using: vec![CAP_MAIL.into()],
                method_calls: vec![(method.to_string(), json!({ "ids": over }), "c1".into())],
            };
            let resp = process(&mut store, "acct1", &req);
            assert_eq!(
                resp.method_responses[0].0, "error",
                "{method} over-limit must be an error"
            );
            assert_eq!(
                resp.method_responses[0].1["type"],
                json!("requestTooLarge"),
                "{method}"
            );
        }
        // SearchSnippet/get shares the cap but keys on `emailIds`.
        let snip = Request {
            using: vec![CAP_MAIL.into()],
            method_calls: vec![(
                "SearchSnippet/get".into(),
                json!({ "emailIds": over }),
                "c1".into(),
            )],
        };
        let sr = process(&mut store, "acct1", &snip);
        assert_eq!(
            sr.method_responses[0].0, "error",
            "SearchSnippet/get over-limit must be an error"
        );
        assert_eq!(sr.method_responses[0].1["type"], json!("requestTooLarge"));
        // An at-limit Email/get is processed normally (not an error).
        let at: Vec<Value> = (0..MAX_OBJECTS_IN_GET)
            .map(|i| json!(format!("x_{i}")))
            .collect();
        let req = Request {
            using: vec![CAP_MAIL.into()],
            method_calls: vec![("Email/get".into(), json!({ "ids": at }), "c1".into())],
        };
        assert_eq!(
            process(&mut store, "acct1", &req).method_responses[0].0,
            "Email/get"
        );
    }

    #[test]
    fn set_handlers_enforce_max_objects_in_set() {
        // Security regression (RFC 8620 §5.3): Email/set and Mailbox/set must reject a
        // create/update/destroy batch larger than maxObjectsInSet with requestTooLarge BEFORE
        // mutating anything — else an unbounded create map drives unbounded object construction.
        let mut store = store_with_mail();
        let mut create = serde_json::Map::new();
        for i in 0..=MAX_OBJECTS_IN_SET {
            create.insert(format!("c{i}"), json!({ "name": format!("Folder{i}") }));
        }
        for method in ["Mailbox/set", "Email/set"] {
            let req = Request {
                using: vec![CAP_MAIL.into()],
                method_calls: vec![(method.to_string(), json!({ "create": create }), "c1".into())],
            };
            let resp = process(&mut store, "acct1", &req);
            assert_eq!(
                resp.method_responses[0].0, "error",
                "{method} over-limit /set must error"
            );
            assert_eq!(
                resp.method_responses[0].1["type"],
                json!("requestTooLarge"),
                "{method}"
            );
        }
    }

    #[test]
    fn rejects_request_over_max_calls_in_request() {
        // Security regression (RFC 8620 §3.6): a request with more than maxCallsInRequest method
        // calls must be rejected with a single request-level `requestTooLarge` error, not dispatched
        // call-by-call — unbounded dispatch enables a Core/echo back-reference amplification DoS.
        let mut store = store_with_mail();
        let too_many: Vec<Invocation> = (0..=MAX_CALLS_IN_REQUEST)
            .map(|i| ("Core/echo".to_string(), json!({ "n": i }), format!("c{i}")))
            .collect();
        let req = Request {
            using: vec![CAP_MAIL.into()],
            method_calls: too_many,
        };
        let resp = process(&mut store, "acct1", &req);
        assert_eq!(
            resp.method_responses.len(),
            1,
            "over-limit request yields one request-level error"
        );
        assert_eq!(resp.method_responses[0].0, "error");
        assert_eq!(resp.method_responses[0].1["type"], json!("requestTooLarge"));
        // A request exactly at the limit still runs every call.
        let at_limit: Vec<Invocation> = (0..MAX_CALLS_IN_REQUEST)
            .map(|i| ("Core/echo".to_string(), json!({ "n": i }), format!("c{i}")))
            .collect();
        let resp2 = process(
            &mut store,
            "acct1",
            &Request {
                using: vec![CAP_MAIL.into()],
                method_calls: at_limit,
            },
        );
        assert_eq!(
            resp2.method_responses.len(),
            MAX_CALLS_IN_REQUEST,
            "at-limit request runs all calls"
        );
    }

    #[test]
    fn mailbox_get_lists_folders() {
        let mut s = store_with_mail();
        let r = call(
            &mut s,
            "Mailbox/get",
            json!({ "accountId": "acct1", "ids": null }),
        );
        let list = r["list"].as_array().unwrap();
        assert!(list.iter().any(|m| m["id"] == json!("INBOX")));
        let inbox = list.iter().find(|m| m["id"] == json!("INBOX")).unwrap();
        assert_eq!(inbox["role"], json!("inbox"));
        assert_eq!(inbox["totalEmails"], json!(1));
    }

    #[test]
    fn email_query_and_get() {
        let mut s = store_with_mail();
        let q = call(
            &mut s,
            "Email/query",
            json!({ "accountId": "acct1", "filter": { "inMailbox": "INBOX" } }),
        );
        let ids = q["ids"].as_array().unwrap();
        assert_eq!(ids.len(), 1);
        let g = call(
            &mut s,
            "Email/get",
            json!({ "accountId": "acct1", "ids": ids }),
        );
        let email = &g["list"][0];
        assert_eq!(email["subject"], json!("Hello"));
        assert_eq!(email["from"][0]["email"], json!("alice@example.com"));
    }

    #[test]
    fn email_set_updates_keywords() {
        let mut s = store_with_mail();
        let id = email_id("INBOX", 1);
        let r = call(
            &mut s,
            "Email/set",
            json!({ "accountId": "acct1", "update": { id.clone(): { "keywords": { "$seen": true } } } }),
        );
        assert!(r["updated"].get(&id).is_some());
        assert!(s
            .mailbox("INBOX")
            .unwrap()
            .by_uid(1)
            .unwrap()
            .has_flag(&Flag::Seen));
    }

    #[test]
    fn email_get_decodes_rfc2047_and_base64_body() {
        let mut s = MemoryStore::empty();
        s.deliver_raw(
            "INBOX",
            b"From: =?UTF-8?B?0JDQu9C40YHQsA==?= <alice@example.ru>\r\n\
              Subject: =?UTF-8?B?0J/RgNC40LLQtdGC?=\r\n\
              Content-Type: text/plain; charset=utf-8\r\n\
              Content-Transfer-Encoding: base64\r\n\r\n\
              0J/RgNC40LLQtdGCLCDQvNC40YAh"
                .to_vec(),
            vec![],
            1_752_000_000_000,
        );
        let g = call(
            &mut s,
            "Email/get",
            json!({ "accountId": "acct1", "ids": [email_id("INBOX", 1)] }),
        );
        let email = &g["list"][0];
        // Headers surface decoded, not as =?UTF-8?B?…?= gibberish.
        assert_eq!(email["subject"], json!("Привет"));
        assert_eq!(email["from"][0]["name"], json!("Алиса"));
        // The base64 body is decoded for display, and honestly not-a-problem.
        let bv = &email["bodyValues"]["1"];
        assert_eq!(bv["value"].as_str().unwrap().trim_end(), "Привет, мир!");
        assert_eq!(bv["isEncodingProblem"], json!(false));
        // Preview shows the same decoded text.
        assert!(
            email["preview"].as_str().unwrap().starts_with("Привет"),
            "{email}"
        );
    }

    #[test]
    fn email_get_flags_undecodable_charset_honestly() {
        let mut s = MemoryStore::empty();
        s.deliver_raw(
            "INBOX",
            b"Subject: nihao\r\nContent-Type: text/plain; charset=gb18030\r\n\
              Content-Transfer-Encoding: 8bit\r\n\r\n\xc4\xe3\xba\xc3"
                .to_vec(),
            vec![],
            1_752_000_000_000,
        );
        let g = call(
            &mut s,
            "Email/get",
            json!({ "accountId": "acct1", "ids": [email_id("INBOX", 1)] }),
        );
        let bv = &g["list"][0]["bodyValues"]["1"];
        // We don't ship GB18030 tables (std-only): the decode is lossy and MUST say so, never
        // assert `isEncodingProblem: false` over U+FFFD soup.
        assert_eq!(bv["isEncodingProblem"], json!(true), "{bv}");
    }

    #[test]
    fn email_set_create_encodes_non_ascii_headers_for_the_wire() {
        let mut s = store_with_mail();
        let r = call(
            &mut s,
            "Email/set",
            json!({ "accountId": "acct1", "create": { "c": {
                "mailboxIds": { "INBOX": true },
                "subject": "Привет, мир",
                "from": [{ "name": "Алиса", "email": "alice@dmtap.local" }],
                "bodyValues": { "1": { "value": "b" } },
                "textBody": [{ "partId": "1", "type": "text/plain" }]
            } } }),
        );
        let id = r["created"]["c"]["id"].as_str().unwrap();
        let (mb, uid) = parse_email_id(id).unwrap();
        let raw = s.mailbox(&mb).unwrap().by_uid(uid).unwrap().raw.clone();
        // Stored header block is wire-legal ASCII (2047-encoded), and the JMAP view round-trips.
        let (hdr, _) = crate::mime::header_and_body(&raw);
        assert!(
            hdr.is_ascii(),
            "compose leaked raw 8-bit headers: {:?}",
            String::from_utf8_lossy(&hdr)
        );
        let g = call(
            &mut s,
            "Email/get",
            json!({ "accountId": "acct1", "ids": [id] }),
        );
        assert_eq!(g["list"][0]["subject"], json!("Привет, мир"));
        assert_eq!(g["list"][0]["from"][0]["name"], json!("Алиса"));
    }

    #[test]
    fn search_snippet_highlights_decoded_subject() {
        let mut s = MemoryStore::empty();
        s.deliver_raw(
            "INBOX",
            b"Subject: =?UTF-8?B?0J/RgNC40LLQtdGC?=\r\n\r\nbody".to_vec(),
            vec![],
            1_752_000_000_000,
        );
        let r = call(
            &mut s,
            "SearchSnippet/get",
            json!({ "accountId": "acct1", "filter": { "subject": "привет" }, "emailIds": [email_id("INBOX", 1)] }),
        );
        assert_eq!(r["list"][0]["subject"], json!("<mark>Привет</mark>"), "{r}");
    }

    #[test]
    fn request_deserializes() {
        let raw =
            r#"{"using":["urn:ietf:params:jmap:core"],"methodCalls":[["Mailbox/query",{},"0"]]}"#;
        let req: Request = serde_json::from_str(raw).unwrap();
        assert_eq!(req.method_calls[0].0, "Mailbox/query");
    }

    #[test]
    fn blob_upload_uses_content_address() {
        let up = blob_upload("acct1", b"hello", "text/plain");
        assert_eq!(up["size"], json!(5));
        assert!(up["blobId"].as_str().unwrap().len() >= 32);
    }

    #[test]
    fn changes_reports_created_updated_destroyed() {
        let mut s = store_with_mail(); // INBOX has uid 1
        let state0 = s.jmap_state();

        // Create a new email → appears in `created` since state0.
        call(
            &mut s,
            "Email/set",
            json!({ "accountId": "acct1", "create": { "c1": {
                "mailboxIds": { "INBOX": true },
                "subject": "Fresh",
                "from": [{ "name": "Me", "email": "me@dmtap.local" }],
                "bodyValues": { "b": { "value": "hello there" } },
                "textBody": [{ "partId": "b", "type": "text/plain" }]
            } } }),
        );
        let ch = call(
            &mut s,
            "Email/changes",
            json!({ "accountId": "acct1", "sinceState": state0 }),
        );
        let created: Vec<&str> = ch["created"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(
            created,
            vec!["INBOX|2"],
            "new email must be in created: {ch}"
        );
        assert!(ch["updated"].as_array().unwrap().is_empty());

        // Update a keyword → appears in `updated`.
        let state1 = s.jmap_state();
        call(
            &mut s,
            "Email/set",
            json!({ "accountId": "acct1", "update": { "INBOX|1": { "keywords/$seen": true } } }),
        );
        let ch = call(
            &mut s,
            "Email/changes",
            json!({ "accountId": "acct1", "sinceState": state1 }),
        );
        assert_eq!(ch["updated"][0], json!("INBOX|1"), "updated: {ch}");
        assert!(ch["created"].as_array().unwrap().is_empty());

        // Destroy → appears in `destroyed` (from the vanished log).
        let state2 = s.jmap_state();
        call(
            &mut s,
            "Email/set",
            json!({ "accountId": "acct1", "destroy": ["INBOX|1"] }),
        );
        let ch = call(
            &mut s,
            "Email/changes",
            json!({ "accountId": "acct1", "sinceState": state2 }),
        );
        assert_eq!(ch["destroyed"][0], json!("INBOX|1"), "destroyed: {ch}");
    }

    #[test]
    fn changes_unparseable_state_reports_cannot_calculate() {
        let mut s = store_with_mail();
        let ch = call(
            &mut s,
            "Email/changes",
            json!({ "accountId": "acct1", "sinceState": "!!!not-base64!!!" }),
        );
        assert_eq!(ch["type"], json!("cannotCalculateChanges"), "{ch}");
    }

    #[test]
    fn email_set_create_composes_a_message() {
        let mut s = store_with_mail();
        let r = call(
            &mut s,
            "Email/set",
            json!({ "accountId": "acct1", "create": { "draft1": {
                "mailboxIds": { "INBOX": true },
                "keywords": { "$draft": true },
                "subject": "Compose test",
                "from": [{ "name": "Alice", "email": "alice@dmtap.local" }],
                "to": [{ "email": "bob@example.net" }],
                "bodyValues": { "1": { "value": "the composed body" } },
                "textBody": [{ "partId": "1", "type": "text/plain" }]
            } } }),
        );
        let id = r["created"]["draft1"]["id"].as_str().unwrap();
        assert_eq!(id, "INBOX|2");
        let (mb, uid) = parse_email_id(id).unwrap();
        let msg = s.mailbox(&mb).unwrap().by_uid(uid).unwrap();
        assert!(msg.has_flag(&Flag::Draft));
        let text = String::from_utf8_lossy(&msg.raw);
        assert!(text.contains("Subject: Compose test"), "{text}");
        assert!(text.contains("From: Alice <alice@dmtap.local>"), "{text}");
        assert!(text.contains("To: bob@example.net"), "{text}");
        assert!(text.contains("the composed body"), "{text}");
    }

    #[test]
    fn create_rejects_header_injection() {
        let mut s = store_with_mail();
        let r = call(
            &mut s,
            "Email/set",
            json!({ "accountId": "acct1", "create": { "x": {
                "mailboxIds": { "INBOX": true },
                "subject": "evil\r\nBcc: victim@example.com",
                "bodyValues": { "1": { "value": "b" } },
                "textBody": [{ "partId": "1", "type": "text/plain" }]
            } } }),
        );
        let id = r["created"]["x"]["id"].as_str().unwrap();
        let (mb, uid) = parse_email_id(id).unwrap();
        let raw =
            String::from_utf8_lossy(&s.mailbox(&mb).unwrap().by_uid(uid).unwrap().raw).to_string();
        // The CRLF-injected "Bcc" must be flattened into the Subject, not a real header line.
        assert!(!raw.contains("\r\nBcc:"), "header injection leaked: {raw}");
    }

    #[test]
    fn create_rejects_header_injection_via_address_fields() {
        // The From/To/Cc "name"/"email" fields are just as attacker-reachable as Subject (all come
        // straight off the request JSON) — a crafted display name or address must not be able to
        // splice in a sibling header (e.g. a forged Bcc) via an embedded CR/LF.
        let mut s = store_with_mail();
        let r = call(
            &mut s,
            "Email/set",
            json!({ "accountId": "acct1", "create": { "x": {
                "mailboxIds": { "INBOX": true },
                "subject": "hi",
                "from": [{ "name": "Eve\r\nBcc: victim@example.com", "email": "eve@dmtap.local" }],
                "to": [{ "email": "bob@example.net\r\nX-Injected: yes" }],
                "bodyValues": { "1": { "value": "b" } },
                "textBody": [{ "partId": "1", "type": "text/plain" }]
            } } }),
        );
        let id = r["created"]["x"]["id"].as_str().unwrap();
        let (mb, uid) = parse_email_id(id).unwrap();
        let raw =
            String::from_utf8_lossy(&s.mailbox(&mb).unwrap().by_uid(uid).unwrap().raw).to_string();
        assert!(
            !raw.contains("\r\nBcc:"),
            "From.name must not inject a header: {raw}"
        );
        assert!(
            !raw.contains("\r\nX-Injected"),
            "To.email must not inject a header: {raw}"
        );
        // The composed message must still parse as exactly one From line and one To line.
        let parsed = ParsedMessage::parse(raw.as_bytes());
        assert_eq!(parsed.header("bcc"), None);
        assert_eq!(parsed.header("x-injected"), None);
        assert!(parsed.header("from").unwrap().contains("eve@dmtap.local"));
        assert!(parsed.header("to").unwrap().contains("bob@example.net"));
    }

    #[test]
    fn core_echo_returns_args_verbatim() {
        let mut s = store_with_mail();
        let r = call(&mut s, "Core/echo", json!({ "hello": [1, 2, 3], "x": "y" }));
        assert_eq!(r, json!({ "hello": [1, 2, 3], "x": "y" }));
    }

    #[test]
    fn mailbox_set_create_update_destroy() {
        let mut s = store_with_mail();
        // create
        let r = call(
            &mut s,
            "Mailbox/set",
            json!({ "accountId": "acct1", "create": { "c1": { "name": "Work" } } }),
        );
        assert_eq!(r["created"]["c1"]["id"], json!("Work"));
        assert!(s.mailbox("Work").is_some());
        // update (rename)
        let r = call(
            &mut s,
            "Mailbox/set",
            json!({ "accountId": "acct1", "update": { "Work": { "name": "Projects" } } }),
        );
        assert!(r["updated"].get("Work").is_some(), "{r}");
        assert!(s.mailbox("Projects").is_some());
        // destroy
        let r = call(
            &mut s,
            "Mailbox/set",
            json!({ "accountId": "acct1", "destroy": ["Projects"] }),
        );
        assert_eq!(r["destroyed"][0], json!("Projects"));
        assert!(s.mailbox("Projects").is_none());
    }

    #[test]
    fn identity_get_and_set() {
        let mut s = store_with_mail();
        let g = call(
            &mut s,
            "Identity/get",
            json!({ "accountId": "acct1", "ids": null }),
        );
        assert_eq!(g["list"][0]["email"], json!("acct1"));
        let st = call(
            &mut s,
            "Identity/set",
            json!({ "accountId": "acct1", "create": { "n": { "email": "alias@dmtap.local", "name": "Alias" } } }),
        );
        assert_eq!(st["created"]["n"]["email"], json!("alias@dmtap.local"));
        // The default identity cannot be destroyed.
        let d = call(
            &mut s,
            "Identity/set",
            json!({ "accountId": "acct1", "destroy": ["I0"] }),
        );
        assert!(d["notDestroyed"].get("I0").is_some(), "{d}");
    }

    #[test]
    fn search_snippet_highlights_terms() {
        let mut s = store_with_mail(); // subject "Hello", body "Hi Bob"
        let id = email_id("INBOX", 1);
        let r = call(
            &mut s,
            "SearchSnippet/get",
            json!({ "accountId": "acct1", "filter": { "subject": "hello" }, "emailIds": [id] }),
        );
        let snippet = &r["list"][0];
        assert_eq!(snippet["subject"], json!("<mark>Hello</mark>"), "{r}");
    }

    #[test]
    fn push_subscription_set_echoes_verification() {
        let mut s = store_with_mail();
        let r = call(
            &mut s,
            "PushSubscription/set",
            json!({ "create": { "p": { "deviceClientId": "d1", "url": "https://push.example/x" } } }),
        );
        assert!(r["created"]["p"]["verificationCode"].is_string(), "{r}");
        // get on a fresh session lists none, and omits accountId per RFC 8620.
        let g = call(&mut s, "PushSubscription/get", json!({}));
        assert!(g["list"].as_array().unwrap().is_empty());
        assert!(g.get("accountId").is_none());
    }

    #[test]
    fn email_submission_get_reports_not_found() {
        let mut s = store_with_mail();
        let r = call(
            &mut s,
            "EmailSubmission/get",
            json!({ "accountId": "acct1", "ids": ["sub-1"] }),
        );
        assert_eq!(r["notFound"][0], json!("sub-1"), "{r}");
        assert!(r["list"].as_array().unwrap().is_empty());
    }

    #[test]
    fn unknown_method_errors() {
        let mut s = store_with_mail();
        let r = call(&mut s, "Bogus/frobnicate", json!({}));
        assert_eq!(r["type"], json!("unknownMethod"));
    }

    #[test]
    fn back_reference_chains_query_into_get() {
        let mut s = store_with_mail();
        let req = Request {
            using: vec![CAP_MAIL.into()],
            method_calls: vec![
                (
                    "Email/query".into(),
                    json!({ "accountId": "acct1", "filter": { "inMailbox": "INBOX" } }),
                    "q".into(),
                ),
                (
                    "Email/get".into(),
                    json!({ "accountId": "acct1", "#ids": { "resultOf": "q", "name": "Email/query", "path": "/ids" } }),
                    "g".into(),
                ),
            ],
        };
        let resp = process(&mut s, "acct1", &req);
        let get = &resp.method_responses[1].1;
        assert_eq!(
            get["list"][0]["subject"],
            json!("Hello"),
            "back-ref get: {get}"
        );
    }

    #[test]
    fn resolve_references_bounds_single_call_fanout() {
        // Security regression: a single call carrying many `#`-keys, each cloning a large prior
        // response via an empty-path back-reference (a full clone), allocated N×R bytes before any
        // size check — process()'s MAX_RESPONSE_BYTES runs only after resolve+dispatch and gates only
        // subsequent calls. resolve_references now bounds cumulative resolved size, failing closed.
        let big = "x".repeat(6_000_000); // ~6 MB response body
        let prior: Vec<Invocation> = vec![(
            "Core/echo".to_string(),
            json!({ "data": big }),
            "c1".to_string(),
        )];

        // 12 back-references × ~6 MB ≈ 72 MB > MAX_RESPONSE_BYTES (50 MB) → must fail closed.
        let mut args = serde_json::Map::new();
        for i in 0..12 {
            args.insert(
                format!("#a{i}"),
                json!({ "resultOf": "c1", "name": "Core/echo", "path": "" }),
            );
        }
        let err = resolve_references(&Value::Object(args), &prior).unwrap_err();
        assert_eq!(
            err["type"],
            json!("invalidResultReference"),
            "over-budget fan-out must fail closed"
        );

        // A single in-budget back-reference to the same response still resolves normally.
        let ok = resolve_references(
            &json!({ "#a": { "resultOf": "c1", "name": "Core/echo", "path": "/data" } }),
            &prior,
        )
        .expect("a single in-budget back-reference resolves");
        assert!(
            ok.get("a").is_some(),
            "normal back-reference resolution still works"
        );
    }

    #[test]
    fn email_get_bounds_response_bytes_against_duplicate_ids() {
        // Security regression: email_get capped only the id COUNT (MAX_OBJECTS_IN_GET), not response
        // BYTES — each object embeds the full decoded body, so N duplicate ids each copy one large
        // message, materializing GBs before process()'s post-hoc MAX_RESPONSE_BYTES check. A
        // cumulative-byte bound in the loop now fails closed.
        let mut s = MemoryStore::empty();
        let mut raw = b"From: a@b\r\nSubject: big\r\n\r\n".to_vec();
        raw.extend(std::iter::repeat(b'x').take(6 * 1024 * 1024)); // ~6 MB body
        s.deliver_raw("INBOX", raw, vec![], 1_752_000_000_000);

        // 30 duplicate ids → a ~180 MB would-be response (> MAX_RESPONSE_BYTES = 50 MB) → fail closed.
        let ids: Vec<Value> = std::iter::repeat(Value::String("INBOX|1".into()))
            .take(30)
            .collect();
        let r = call(
            &mut s,
            "Email/get",
            json!({ "accountId": "acct1", "ids": ids }),
        );
        assert_eq!(
            r["type"],
            json!("requestTooLarge"),
            "duplicate-packed large Email/get must fail closed, got: {r}"
        );

        // A single get of the same message (~6 MB < 50 MB) still succeeds.
        let ok = call(
            &mut s,
            "Email/get",
            json!({ "accountId": "acct1", "ids": ["INBOX|1"] }),
        );
        assert_eq!(
            ok["list"][0]["id"],
            json!("INBOX|1"),
            "single get still works: {ok}"
        );
    }

    #[test]
    fn filter_term_count_is_bounded() {
        // Security regression: a wide FilterOperator `conditions` array yielded one term per
        // condition with no cap, amplifying highlight() into O(terms·emails·text). Capped at
        // MAX_FILTER_TERMS.
        let conds: Vec<Value> = (0..(MAX_FILTER_TERMS * 10))
            .map(|_| json!({ "text": "a" }))
            .collect();
        let terms =
            collect_filter_terms(&json!({ "operator": "AND", "conditions": conds })).unwrap();
        assert!(
            terms.len() <= MAX_FILTER_TERMS,
            "filter term count must be bounded, got {}",
            terms.len()
        );
        // A normal small filter still collects its terms.
        let ok = collect_filter_terms(&json!({ "subject": "hi", "from": "bob" })).unwrap();
        assert_eq!(ok.len(), 2);
    }

    #[test]
    fn mailbox_get_enforces_max_objects_in_get() {
        // Security regression: mailbox_get had no maxObjectsInGet cap and scanned every mailbox against
        // the full ids array (O(mailboxes × ids)) — request-size-amplified. It now rejects an oversized
        // id list like the sibling get handlers and uses set membership.
        let mut s = store_with_mail();
        let ids: Vec<Value> = (0..(MAX_OBJECTS_IN_GET + 1))
            .map(|i| Value::String(format!("z{i}")))
            .collect();
        let r = call(
            &mut s,
            "Mailbox/get",
            json!({ "accountId": "acct1", "ids": ids }),
        );
        assert_eq!(
            r["type"],
            json!("requestTooLarge"),
            "oversized Mailbox/get ids must fail closed: {r}"
        );
        // A normal request (filter by real id) still returns that mailbox.
        let ok = call(
            &mut s,
            "Mailbox/get",
            json!({ "accountId": "acct1", "ids": ["INBOX"] }),
        );
        assert_eq!(
            ok["list"][0]["id"],
            json!("INBOX"),
            "normal Mailbox/get works: {ok}"
        );
    }

    #[test]
    fn search_snippet_highlight_is_length_bounded() {
        // Security regression: search_snippet_get highlighted the FULL decoded subject, and
        // find_ci builds a per-char tagged Vec (~24 B/char) — a multi-MiB Subject drove a proportional
        // transient allocation per request. highlight() now caps the text it processes.
        let mut s = MemoryStore::empty();
        let mut raw = b"From: a@b\r\nSubject: ".to_vec();
        raw.extend(std::iter::repeat(b'x').take(4 * 1024 * 1024)); // ~4 MiB Subject header
        raw.extend_from_slice(b"\r\n\r\nbody");
        s.deliver_raw("INBOX", raw, vec![], 1_752_000_000_000);
        let r = call(
            &mut s,
            "SearchSnippet/get",
            json!({ "accountId": "acct1", "filter": { "subject": "x" }, "emailIds": ["INBOX|1"] }),
        );
        // Must complete and return a bounded snippet (the highlighted subject is capped, not ~4 MiB).
        let subj = r["list"][0]["subject"].as_str().unwrap_or("");
        assert!(
            subj.len() < 100_000,
            "highlighted subject must be length-bounded, got {}",
            subj.len()
        );
    }
}
