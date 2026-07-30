//! JSON interchange for the canonical-CBOR value space — **optional**, behind the
//! `json` feature.
//!
//! This is not a KOTVA/DMTAP wire format. It is a human-readable projection of
//! [`Value`], normative to evermesh's spec 001 §11, carried here so the byte-level
//! codec and its JSON projection cannot drift apart in separate repositories. The
//! DMTAP wire never touches it and `kotva-core` does not enable the feature.
//!
//! # The mapping
//!
//! [`Value::Uint`] / [`Value::Nint`] become plain JSON integers. Byte strings
//! become the quoted string `"hex:<lowercase hex>"`. Arrays and maps map
//! naturally, except that JSON object keys must be strings, so map keys are
//! rendered as: integers as a decimal string, `Bytes` as `"hex:<hex>"`, `Text` as
//! itself.
//!
//! That creates two ambiguities, resolved by escaping. Both are documented here
//! because the mapping does not round-trip without them:
//!
//! * A `Text` value (key or plain value) whose content itself starts with `"hex:"`
//!   or `"txt:"` would be indistinguishable from an escaped byte string.
//!   Resolution: render such text with an extra `"txt:"` prefix. On decode, a
//!   string starting `"hex:"` decodes to `Bytes`, a string starting `"txt:"` has
//!   that one prefix stripped and decodes to `Text` (whatever remains, verbatim),
//!   and anything else decodes to `Text` as-is.
//! * A `Text` **map key** whose content is itself a bare canonical decimal integer
//!   (`"5"`, `"-3"`) would, after decoding, be indistinguishable from an
//!   integer-valued key rendering to the same digits. Resolution: apply the same
//!   `"txt:"` escape whenever a text key's raw content would otherwise be
//!   re-parsed as a decimal integer key. Plain (non-key) text values never need
//!   this, because integers never appear as bare JSON strings in value position.
//!
//! Integer map keys support the full `Uint`/`Nint` magnitude range. Bare JSON
//! *number* tokens in value position are narrower: positive values fit `u64`, but
//! negative values must fit `i64`. That asymmetry is spec 001 §11's, kept as-is.
//!
//! # Known gap, stated rather than hidden
//!
//! [`Value::Map`] admits keys of **any** type (see the crate docs), and a
//! `Bool` / `Null` / `Array` / `Map` key has no JSON rendering. [`to_json`]
//! cannot fail, so it renders such a key as `"unsupported-key:<debug>"`, which
//! **does not round-trip**: [`from_json`] reads it back as ordinary text and
//! [`crate::encode_canonical`] then produces different bytes — a different
//! content address for what was the same value. Callers that both accept
//! untrusted CBOR and expose a JSON form must reject non-scalar map keys at their
//! own schema layer first. [`is_json_representable`] answers that question.

use crate::{CborError, Value, MAX_DEPTH};

/// Whether every map key in this value, at every depth, has a faithful JSON
/// rendering — i.e. is an integer, byte string, or text string.
///
/// Check this before exposing a value's [`to_json`] form as though it were
/// equivalent to the value; see the module's "Known gap".
pub fn is_json_representable(v: &Value) -> bool {
    match v {
        Value::Array(items) => items.iter().all(is_json_representable),
        Value::Map(entries) => entries.iter().all(|(k, val)| {
            matches!(
                k,
                Value::Uint(_) | Value::Nint(_) | Value::Bytes(_) | Value::Text(_)
            ) && is_json_representable(k)
                && is_json_representable(val)
        }),
        _ => true,
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

fn hex_digit(b: u8) -> Result<u8, CborError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        _ => Err(CborError::Json("invalid hex digit")),
    }
}

fn decode_hex(s: &str) -> Result<Vec<u8>, CborError> {
    let b = s.as_bytes();
    if b.len() % 2 != 0 {
        return Err(CborError::Json("odd-length hex string"));
    }
    let mut out = Vec::with_capacity(b.len() / 2);
    for pair in b.chunks_exact(2) {
        out.push((hex_digit(pair[0])? << 4) | hex_digit(pair[1])?);
    }
    Ok(out)
}

fn nint_decimal(n: u64) -> String {
    (-1i128 - n as i128).to_string()
}

fn bytes_json_string(b: &[u8]) -> String {
    format!("hex:{}", hex_lower(b))
}

/// Parse a decimal-integer map key (`"5"`, `"-3"`; not `"-0"`, not `"007"`) into
/// `Uint`/`Nint`, over the full magnitude range so any key [`to_json`] produces
/// parses back exactly. `None` — not an error — for anything outside that
/// grammar; the caller falls through to text/bytes handling.
fn try_parse_int_key(s: &str) -> Option<Value> {
    let (negative, digits) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s),
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if digits.len() > 1 && digits.starts_with('0') {
        return None;
    }
    if negative && digits == "0" {
        return None;
    }
    // u128 so the extreme Nint magnitude (2^64, one more than u64::MAX) is
    // representable during the subtraction below.
    let magnitude: u128 = digits.parse().ok()?;
    if negative {
        if magnitude > (u64::MAX as u128) + 1 {
            return None;
        }
        Some(Value::Nint((magnitude - 1) as u64))
    } else {
        if magnitude > u64::MAX as u128 {
            return None;
        }
        Some(Value::Uint(magnitude as u64))
    }
}

/// Render a `Text` value's content, escaping the hex:/txt: collision only. Used
/// for plain (non-key) values.
fn text_json_string(s: &str) -> String {
    if s.starts_with("hex:") || s.starts_with("txt:") {
        format!("txt:{s}")
    } else {
        s.to_string()
    }
}

/// Render a map key's content: §11's rule plus the integer-lookalike escape.
fn key_json_string(k: &Value) -> String {
    match k {
        Value::Uint(n) => n.to_string(),
        Value::Nint(n) => nint_decimal(*n),
        Value::Bytes(b) => bytes_json_string(b),
        Value::Text(s) => {
            if s.starts_with("hex:") || s.starts_with("txt:") || try_parse_int_key(s).is_some() {
                format!("txt:{s}")
            } else {
                s.clone()
            }
        }
        // No JSON rendering exists. `to_json` cannot fail, so emit something
        // panic-free and non-colliding. This does NOT round-trip — see the
        // module's "Known gap" and `is_json_representable`.
        Value::Array(_) | Value::Map(_) | Value::Bool(_) | Value::Null => {
            format!("unsupported-key:{k:?}")
        }
    }
}

fn push_json_escaped(out: &mut String, s: &str) {
    use core::fmt::Write;
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            // `write!` to a `String` never fails.
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
}

fn value_to_json(v: &Value, out: &mut String) {
    match v {
        Value::Uint(n) => out.push_str(&n.to_string()),
        Value::Nint(n) => out.push_str(&nint_decimal(*n)),
        Value::Bytes(b) => {
            out.push('"');
            push_json_escaped(out, &bytes_json_string(b));
            out.push('"');
        }
        Value::Text(s) => {
            out.push('"');
            push_json_escaped(out, &text_json_string(s));
            out.push('"');
        }
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                value_to_json(item, out);
            }
            out.push(']');
        }
        Value::Map(entries) => {
            out.push('{');
            for (i, (k, val)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('"');
                push_json_escaped(out, &key_json_string(k));
                out.push('"');
                out.push(':');
                value_to_json(val, out);
            }
            out.push('}');
        }
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Null => out.push_str("null"),
    }
}

/// The JSON interchange form of a value. Infallible; see the module's "Known
/// gap" for the one shape whose rendering does not round-trip.
pub fn to_json(v: &Value) -> String {
    let mut out = String::new();
    value_to_json(v, &mut out);
    out
}

/// A JSON string in *value* position: only the hex:/txt: rule applies.
fn string_value_from_json(raw: &str) -> Result<Value, CborError> {
    if let Some(hex) = raw.strip_prefix("hex:") {
        Ok(Value::Bytes(decode_hex(hex)?))
    } else if let Some(rest) = raw.strip_prefix("txt:") {
        Ok(Value::Text(rest.to_string()))
    } else {
        Ok(Value::Text(raw.to_string()))
    }
}

/// A JSON object key: integer-lookalikes become integers, then the value rule.
fn key_value_from_json(raw: &str) -> Result<Value, CborError> {
    if let Some(v) = try_parse_int_key(raw) {
        return Ok(v);
    }
    string_value_from_json(raw)
}

struct Parser<'a> {
    chars: core::iter::Peekable<core::str::Chars<'a>>,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Parser {
            chars: s.chars().peekable(),
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn next_char(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek_char(), Some(' ' | '\t' | '\n' | '\r')) {
            self.chars.next();
        }
    }

    fn expect(&mut self, c: char) -> Result<(), CborError> {
        match self.next_char() {
            Some(x) if x == c => Ok(()),
            _ => Err(CborError::Json("unexpected character")),
        }
    }

    fn read_hex4(&mut self) -> Result<u16, CborError> {
        let mut v: u16 = 0;
        for _ in 0..4 {
            let c = self
                .next_char()
                .ok_or(CborError::Json("truncated unicode escape"))?;
            let d = c
                .to_digit(16)
                .ok_or(CborError::Json("invalid unicode escape"))?;
            v = v * 16 + d as u16;
        }
        Ok(v)
    }

    /// A JSON string literal, opening `"` already consumed. Handles every
    /// standard escape including `\uXXXX` surrogate pairs; rejects lone
    /// surrogates and raw control characters.
    fn parse_string_raw(&mut self) -> Result<String, CborError> {
        let mut s = String::new();
        loop {
            let c = self
                .next_char()
                .ok_or(CborError::Json("unterminated string"))?;
            match c {
                '"' => return Ok(s),
                '\\' => {
                    let esc = self
                        .next_char()
                        .ok_or(CborError::Json("unterminated escape"))?;
                    match esc {
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        '/' => s.push('/'),
                        'b' => s.push('\u{08}'),
                        'f' => s.push('\u{0c}'),
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        'u' => {
                            let cp = self.read_hex4()?;
                            if (0xD800..=0xDBFF).contains(&cp) {
                                if self.next_char() != Some('\\') || self.next_char() != Some('u') {
                                    return Err(CborError::Json("lone surrogate"));
                                }
                                let low = self.read_hex4()?;
                                if !(0xDC00..=0xDFFF).contains(&low) {
                                    return Err(CborError::Json("invalid surrogate pair"));
                                }
                                let combined =
                                    0x10000u32 + ((cp as u32 - 0xD800) << 10) + (low as u32 - 0xDC00);
                                s.push(
                                    char::from_u32(combined)
                                        .ok_or(CborError::Json("invalid unicode scalar"))?,
                                );
                            } else if (0xDC00..=0xDFFF).contains(&cp) {
                                return Err(CborError::Json("lone surrogate"));
                            } else {
                                s.push(
                                    char::from_u32(cp as u32)
                                        .ok_or(CborError::Json("invalid unicode scalar"))?,
                                );
                            }
                        }
                        _ => return Err(CborError::Json("invalid escape sequence")),
                    }
                }
                c if (c as u32) < 0x20 => {
                    return Err(CborError::Json("control character in string"))
                }
                c => s.push(c),
            }
        }
    }

    fn parse_literal(&mut self, lit: &str, value: Value) -> Result<Value, CborError> {
        for expected in lit.chars() {
            match self.next_char() {
                Some(c) if c == expected => {}
                _ => return Err(CborError::Json("invalid literal")),
            }
        }
        Ok(value)
    }

    /// A JSON number. Integers only: optional `-`, digits with no leading zero
    /// (except a bare `0`), no fraction, no exponent. Positive values fit `u64`;
    /// negative values must also fit `i64`.
    fn parse_number(&mut self) -> Result<Value, CborError> {
        let mut s = String::new();
        if self.peek_char() == Some('-') {
            s.push('-');
            self.next_char();
        }
        let mut any_digit = false;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                s.push(c);
                any_digit = true;
                self.next_char();
            } else {
                break;
            }
        }
        if !any_digit {
            return Err(CborError::Json("invalid number"));
        }
        if matches!(self.peek_char(), Some('.') | Some('e') | Some('E')) {
            return Err(CborError::Json("fractional or exponential number"));
        }
        let negative = s.starts_with('-');
        let digits = if negative { &s[1..] } else { s.as_str() };
        if digits.len() > 1 && digits.starts_with('0') {
            return Err(CborError::Json("leading zero in number"));
        }
        if negative {
            if digits == "0" {
                return Err(CborError::Json("negative zero"));
            }
            let magnitude: u64 = digits
                .parse()
                .map_err(|_| CborError::Json("integer out of range"))?;
            if magnitude > 9_223_372_036_854_775_808u64 {
                return Err(CborError::Json("integer out of range"));
            }
            Ok(Value::Nint(magnitude - 1))
        } else {
            Ok(Value::Uint(
                digits
                    .parse()
                    .map_err(|_| CborError::Json("integer out of range"))?,
            ))
        }
    }

    fn parse_array(&mut self, depth: u32) -> Result<Value, CborError> {
        self.next_char(); // '['
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek_char() == Some(']') {
            self.next_char();
            return Ok(Value::Array(items));
        }
        loop {
            items.push(self.parse_value(depth + 1)?);
            self.skip_ws();
            match self.next_char() {
                Some(',') => {
                    self.skip_ws();
                    continue;
                }
                Some(']') => break,
                _ => return Err(CborError::Json("expected ',' or ']'")),
            }
        }
        Ok(Value::Array(items))
    }

    fn parse_object(&mut self, depth: u32) -> Result<Value, CborError> {
        self.next_char(); // '{'
        let mut entries: Vec<(Value, Value)> = Vec::new();
        self.skip_ws();
        if self.peek_char() == Some('}') {
            self.next_char();
            return Ok(Value::Map(entries));
        }
        loop {
            self.skip_ws();
            self.expect('"')?;
            let raw_key = self.parse_string_raw()?;
            let key = key_value_from_json(&raw_key)?;
            self.skip_ws();
            self.expect(':')?;
            let val = self.parse_value(depth + 1)?;
            if entries.iter().any(|(k, _)| *k == key) {
                return Err(CborError::Json("duplicate object key"));
            }
            entries.push((key, val));
            self.skip_ws();
            match self.next_char() {
                Some(',') => continue,
                Some('}') => break,
                _ => return Err(CborError::Json("expected ',' or '}'")),
            }
        }
        Ok(Value::Map(entries))
    }

    fn parse_value(&mut self, depth: u32) -> Result<Value, CborError> {
        if depth > MAX_DEPTH {
            return Err(CborError::Json("nesting depth exceeds limit"));
        }
        self.skip_ws();
        match self.peek_char() {
            Some('{') => self.parse_object(depth),
            Some('[') => self.parse_array(depth),
            Some('"') => {
                self.next_char();
                let raw = self.parse_string_raw()?;
                string_value_from_json(&raw)
            }
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            Some('t') => self.parse_literal("true", Value::Bool(true)),
            Some('f') => self.parse_literal("false", Value::Bool(false)),
            Some('n') => self.parse_literal("null", Value::Null),
            _ => Err(CborError::Json("unexpected character or end of input")),
        }
    }
}

/// Strict inverse of [`to_json`]: a minimal, dependency-free JSON parser
/// (objects, arrays, strings with standard escapes including surrogate pairs,
/// integers only, `true`/`false`/`null`), depth-limited to [`MAX_DEPTH`].
///
/// Floats, exponents, leading zeros, `-0`, duplicate object keys and trailing
/// data are all refused — a JSON document that cannot be represented as canonical
/// CBOR is rejected rather than silently coerced.
pub fn from_json(s: &str) -> Result<Value, CborError> {
    let mut p = Parser::new(s);
    let v = p.parse_value(0)?;
    p.skip_ws();
    if p.peek_char().is_some() {
        return Err(CborError::Json("trailing data after value"));
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(v: &Value) {
        let json = to_json(v);
        assert_eq!(&from_json(&json).unwrap(), v, "json was: {json}");
    }

    #[test]
    fn round_trip_scalars() {
        round_trip(&Value::Uint(0));
        round_trip(&Value::Uint(u64::MAX));
        round_trip(&Value::from_i64(-1));
        round_trip(&Value::from_i64(i64::MIN));
        round_trip(&Value::Bool(true));
        round_trip(&Value::Bool(false));
        round_trip(&Value::Null);
        round_trip(&Value::Text("hello".into()));
        round_trip(&Value::Text(String::new()));
        round_trip(&Value::Bytes(vec![0xde, 0xad, 0xbe, 0xef]));
        round_trip(&Value::Bytes(vec![]));
    }

    #[test]
    fn hex_and_txt_escapes_round_trip() {
        let bytes = Value::Bytes(vec![0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(to_json(&bytes), "\"hex:deadbeef\"");
        round_trip(&bytes);
        let text = Value::Text("hex:deadbeef".into());
        assert_eq!(to_json(&text), "\"txt:hex:deadbeef\"");
        round_trip(&text);
        let text2 = Value::Text("txt:something".into());
        assert_eq!(to_json(&text2), "\"txt:txt:something\"");
        round_trip(&text2);
    }

    #[test]
    fn integer_and_lookalike_text_keys_are_disambiguated() {
        let int_keyed = Value::Map(vec![(Value::Uint(5), Value::Bool(true))]);
        assert_eq!(to_json(&int_keyed), "{\"5\":true}");
        round_trip(&int_keyed);

        let text_keyed = Value::Map(vec![(Value::Text("5".into()), Value::Bool(true))]);
        assert_eq!(to_json(&text_keyed), "{\"txt:5\":true}");
        round_trip(&text_keyed);

        round_trip(&Value::Map(vec![(Value::from_i64(-3), Value::Null)]));
        let neg_text = Value::Map(vec![(Value::Text("-3".into()), Value::Null)]);
        assert_eq!(to_json(&neg_text), "{\"txt:-3\":null}");
        round_trip(&neg_text);

        // Leading-zero text keys are never produced by the integer path, so they
        // are never ambiguous and pass through unescaped.
        let lz = Value::Map(vec![(Value::Text("007".into()), Value::Null)]);
        assert_eq!(to_json(&lz), "{\"007\":null}");
        round_trip(&lz);
    }

    #[test]
    fn nested_structures_round_trip() {
        round_trip(&Value::Map(vec![
            (Value::Text("text".into()), Value::Text("hi".into())),
            (
                Value::Uint(4),
                Value::Array(vec![Value::Array(vec![
                    Value::Uint(0),
                    Value::Bytes(vec![0xaa; 32]),
                ])]),
            ),
        ]));
    }

    #[test]
    fn rejects_what_canonical_cbor_cannot_hold() {
        assert!(from_json("1.5").is_err());
        assert!(from_json("1e5").is_err());
        assert!(from_json("1E5").is_err());
        assert!(from_json("01").is_err());
        assert!(from_json("-01").is_err());
        assert!(from_json("-0").is_err());
        assert!(from_json("\"\\q\"").is_err());
        assert!(from_json("\"\\uD800\"").is_err());
        assert!(from_json("\"\\uDC00\"").is_err());
        assert!(from_json("1 2").is_err());
        assert!(from_json("{\"a\":1,\"a\":2}").is_err());
        assert!(from_json("{\"1\":1.0}").is_err());
        // u64::MAX + 1
        assert!(from_json("18446744073709551616").is_err());
        // Negative values are i64-bounded even though Nint could hold more.
        assert!(from_json("-9223372036854775809").is_err());
        assert!(from_json("-9223372036854775808").is_ok());
    }

    #[test]
    fn accepts_surrogate_pairs() {
        assert_eq!(
            from_json("\"\\uD83D\\uDE00\"").unwrap(),
            Value::Text("\u{1F600}".to_string())
        );
    }

    #[test]
    fn rejects_depth_beyond_the_limit() {
        let mut s = String::new();
        for _ in 0..(MAX_DEPTH + 2) {
            s.push('[');
        }
        s.push('0');
        for _ in 0..(MAX_DEPTH + 2) {
            s.push(']');
        }
        assert!(from_json(&s).is_err());
    }

    #[test]
    fn non_scalar_keys_are_flagged_and_do_not_round_trip() {
        // The documented gap: `is_json_representable` is how a caller finds out
        // BEFORE relying on the JSON form.
        let bad = Value::Map(vec![(Value::Bool(false), Value::Uint(0))]);
        assert!(!is_json_representable(&bad));
        let json = to_json(&bad);
        assert_eq!(json, "{\"unsupported-key:Bool(false)\":0}");
        // It parses, but to something else entirely — hence the guard function.
        assert_ne!(from_json(&json).unwrap(), bad);

        // Everything with scalar keys is representable, at any depth.
        let good = Value::Map(vec![(
            Value::Uint(1),
            Value::Array(vec![Value::Map(vec![(
                Value::Text("k".into()),
                Value::Bytes(vec![1]),
            )])]),
        )]);
        assert!(is_json_representable(&good));
        round_trip(&good);
    }
}
