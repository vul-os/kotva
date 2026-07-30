//! Canonical deterministic CBOR — RFC 8949 §4.2.1 core deterministic encoding,
//! as profiled by KOTVA/DMTAP §18.1.1.
//!
//! # What this crate is for
//!
//! A signature is a statement about *bytes*. If two honest implementations can
//! serialize the same value to different bytes, the signature says nothing and a
//! content address is not an address. So the codec that signed and
//! content-addressed objects flow through must be **byte-identical across
//! independent implementations**, and its decoder must **reject** non-canonical
//! input rather than normalize it — a decoder that silently re-canonicalizes lets
//! a sender hand two different verifiers two different byte strings for the same
//! object.
//!
//! [`encode_canonical`] emits exactly one encoding per value. [`decode_canonical`]
//! accepts *only* that encoding, so
//!
//! ```text
//! encode_canonical(decode_canonical(b)?)? == b     for every accepted b
//! ```
//!
//! which is the malleability guarantee the whole thing exists to provide. That
//! property is asserted by test over a frozen 183-vector corpus, not argued.
//!
//! # Provenance — this is a consolidation, not a new implementation
//!
//! The Vulos family independently grew **four** canonical-CBOR codecs, all
//! claiming the same rules, with no cross-check between any two:
//! `kotva_core::cbor`, `kotva_sync::detcbor`, `evermesh_kernel::codec` and
//! `magnetite_seams::cbor`. This crate is seeded from **evermesh's**, which was
//! the most tested of the four (189 conformance vectors replayed across three
//! runtimes, plus 201 unit, 7 property and 4 frozen tests) and the only one whose
//! module docs already treated cross-implementation byte identity as
//! consensus-critical. evermesh's suite is MIT OR Apache-2.0; the vectors it
//! produced are frozen here as
//! `tests/vectors/evermesh-canonical-cbor.txt`.
//!
//! Before the extraction, a differential harness drove all four codecs from those
//! vectors plus 20 000 generated values and 200 000 random byte strings. **No
//! byte-level encoding divergence was found on the shared subset.** The
//! differences that do exist are all in *which values each can represent* — see
//! "Scope of the value space" below — and are stated rather than papered over.
//!
//! # Rules enforced by [`decode_canonical`]
//!
//! 1. **Definite lengths only.** Indefinite-length items (additional info 31 on
//!    major types 2–5) and the `break` code are rejected.
//! 2. **Shortest-form arguments.** Values 0–23 are inlined in the initial byte;
//!    larger values use the smallest of the 1/2/4/8-byte argument forms that can
//!    hold them. A widened head (24 spelled with a 2-byte argument, a length of 5
//!    spelled `0x59 0x00 0x05`) is rejected.
//! 3. **Map keys strictly ascending** by the bytewise order of their own
//!    canonical encodings. One check, two rules: a duplicate key cannot be
//!    strictly greater than itself, so [`CborError::DuplicateMapKey`] and
//!    [`CborError::MapKeyOrder`] are distinguished for the caller's benefit while
//!    both being refusals.
//! 4. **No floating point** (major 7, additional info 25/26/27) and no simple
//!    values beyond `false`/`true`/`null` (20/21/22). `undefined` (23) and
//!    one-byte simple values (24) are rejected.
//! 5. **No tags** (major 6).
//! 6. **Exactly one top-level item.** Trailing bytes are an error — otherwise
//!    re-encoding would silently drop the tail.
//!
//! Two further checks are resource guards rather than canonical-encoding rules,
//! and are called out as such because they are the one place a conforming
//! implementation may legitimately differ:
//!
//! * Nesting depth is capped at [`MAX_DEPTH`] (64). Without it, ~50 KB of `0x81`
//!   overflows the native stack and aborts the process from any parse of
//!   untrusted bytes. 64 is far above any real object and far below the overflow
//!   threshold.
//! * Every length header is checked against the bytes actually remaining before
//!   anything is allocated, so a crafted 8-byte length cannot force a huge
//!   allocation out of a nine-byte input.
//!
//! # Scope of the value space — and one known gap
//!
//! [`Value`] spans unsigned and negative integers, byte and text strings,
//! arrays, maps, `bool` and `null`. It has **no float and no tag arm at all**, so
//! those are unrepresentable rather than merely rejected.
//!
//! Map keys may be **any** `Value`. That is inherited from evermesh deliberately,
//! so this crate accepts exactly what evermesh accepts and a migration cannot
//! change any verdict. It has a consequence worth stating plainly: a map keyed by
//! a `Bool`, `Null`, `Array` or `Map` is accepted here but has **no faithful JSON
//! interchange form** — the optional [`json`] module renders such a key as
//! `"unsupported-key:…"`, which does not round-trip. Callers with a schema (KOTVA
//! objects are integer-keyed, §18.1.2; `Headers.ext` is text-keyed, §18.3.6)
//! should reject key types their schema does not name, and `kotva_core::cbor::Cv`
//! does exactly that.
//!
//! # Choosing an encoder
//!
//! [`encode_canonical`] refuses a map holding duplicate keys, because the bytes it
//! would otherwise emit are non-canonical and its own decoder would reject them.
//! [`encode_canonical_unchecked`] skips that check and is infallible; it exists
//! only for callers whose public API predates the check and cannot start
//! returning a `Result`. Prefer the checked one.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]

#[cfg(feature = "json")]
pub mod json;

use core::fmt;

/// Maximum nesting depth accepted by [`decode_canonical`] (and by the optional
/// [`json`] parser). A resource-exhaustion guard, not a canonical-encoding rule:
/// each nested container costs a native stack frame, so an unbounded decoder lets
/// a few tens of KB of `0x81` overflow the stack and abort.
///
/// Deliberately the same number every codec in this family applies, so one
/// encoder cannot mint a value a second decoder refuses.
pub const MAX_DEPTH: u32 = 64;

/// A canonical-CBOR value. No float arm and no tag arm exist, so neither is
/// representable — they are refused at the type level, not merely at the decoder.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    /// Major type 0: an unsigned integer.
    Uint(u64),
    /// Major type 1: a negative integer. The payload is the CBOR *argument* `n`,
    /// which encodes the number `-1 - n`, so the full 64-bit negative range is
    /// representable without an `i128`.
    Nint(u64),
    /// Major type 2: a byte string.
    Bytes(Vec<u8>),
    /// Major type 3: a UTF-8 text string.
    Text(String),
    /// Major type 4: a definite-length array.
    Array(Vec<Value>),
    /// Major type 5: a map. Entries are in canonical (ascending, unique-key)
    /// order once produced by [`decode_canonical`]; the encoders sort and
    /// validate on the way out regardless of the order they are handed.
    Map(Vec<(Value, Value)>),
    /// Major type 7, additional info 20/21: a boolean.
    Bool(bool),
    /// Major type 7, additional info 22: null.
    Null,
}

impl Value {
    /// A signed integer as its canonical value: non-negative uses [`Value::Uint`],
    /// negative uses [`Value::Nint`] (RFC 8949 §3.1).
    pub fn from_i64(v: i64) -> Value {
        if v >= 0 {
            Value::Uint(v as u64)
        } else {
            // i128 avoids overflowing `-1 - v` at `v == i64::MIN`.
            Value::Nint((-1i128 - v as i128) as u64)
        }
    }

    /// This value as `i64`, if it fits: `Uint` up to `i64::MAX`, or `Nint(n)`
    /// whose represented value `-1 - n` is at least `i64::MIN`.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Uint(v) if *v <= i64::MAX as u64 => Some(*v as i64),
            Value::Nint(n) if *n <= i64::MAX as u64 => Some((-1i128 - *n as i128) as i64),
            _ => None,
        }
    }

    /// This value as `u64`, if it is a [`Value::Uint`].
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Uint(v) => Some(*v),
            _ => None,
        }
    }

    /// This value's bytes, if it is a [`Value::Bytes`].
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(b) => Some(b),
            _ => None,
        }
    }

    /// This value's text, if it is a [`Value::Text`].
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Value::Text(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// This value's elements, if it is a [`Value::Array`].
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(a) => Some(a.as_slice()),
            _ => None,
        }
    }

    /// This value's entries, if it is a [`Value::Map`].
    pub fn as_map(&self) -> Option<&[(Value, Value)]> {
        match self {
            Value::Map(m) => Some(m.as_slice()),
            _ => None,
        }
    }

    /// This value as `bool`, if it is a [`Value::Bool`].
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Look up a text key in a map. `None` if this is not a map or the key is absent.
    pub fn map_get(&self, key: &str) -> Option<&Value> {
        self.as_map()?
            .iter()
            .find(|(k, _)| k.as_text() == Some(key))
            .map(|(_, v)| v)
    }

    /// Look up an unsigned-integer key in a map. `None` if this is not a map or
    /// the key is absent.
    pub fn map_get_int(&self, key: u64) -> Option<&Value> {
        self.as_map()?
            .iter()
            .find(|(k, _)| k.as_u64() == Some(key))
            .map(|(_, v)| v)
    }

    /// This value with every nested map's entries reordered into canonical order
    /// — sorted by the bytewise order of their canonically encoded keys, exactly
    /// the key the encoders use.
    ///
    /// Only ordering changes; the set of entries is untouched, so this never
    /// alters the canonical encoding. Its purpose is to make a hand-built value
    /// compare equal to one [`decode_canonical`] produced from the same bytes.
    pub fn into_canonical(self) -> Value {
        match self {
            Value::Array(items) => {
                Value::Array(items.into_iter().map(Value::into_canonical).collect())
            }
            Value::Map(entries) => {
                let mut canon: Vec<(Vec<u8>, (Value, Value))> = entries
                    .into_iter()
                    .map(|(k, v)| {
                        let k = k.into_canonical();
                        let v = v.into_canonical();
                        let mut kbuf = Vec::new();
                        // Unchecked: this needs a TOTAL ordering function, and a
                        // duplicate key must surface at encode time, not here.
                        let _ = enc(&k, &mut kbuf, false);
                        (kbuf, (k, v))
                    })
                    .collect();
                canon.sort_by(|a, b| a.0.cmp(&b.0));
                Value::Map(canon.into_iter().map(|(_, kv)| kv).collect())
            }
            other => other,
        }
    }

    /// This value's canonical encoding, or the reason it has none.
    /// Sugar for [`encode_canonical`].
    pub fn to_canonical(&self) -> Result<Vec<u8>, CborError> {
        encode_canonical(self)
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// A refusal. Every variant means the input is not the canonical encoding of any
/// value (or, for [`CborError::DuplicateMapKey`] on the encode side, that the
/// value has no canonical encoding). Nothing here is ever repaired.
///
/// The variants are deliberately finer-grained than "malformed": a consumer with
/// its own error taxonomy — `kotva_core::cbor::CborError` is one — needs to map
/// each refusal onto its own code without re-parsing the bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CborError {
    /// Input ended inside an item.
    Truncated,
    /// A complete top-level item, followed by more bytes.
    TrailingBytes,
    /// An integer or length head longer than necessary (rule 2).
    NonShortestForm,
    /// An indefinite-length item, or a `break` code (rule 1).
    IndefiniteLength,
    /// Additional info 28, 29 or 30 — reserved by RFC 8949, assigned to nothing.
    ReservedAdditionalInfo,
    /// A half-, single- or double-precision float (rule 4).
    Float,
    /// The `undefined` simple value, additional info 23 (rule 4).
    Undefined,
    /// A one-byte simple value, additional info 24 (rule 4).
    SimpleValue,
    /// A tag, major type 6 (rule 5).
    Tag,
    /// Map keys not in strictly ascending encoded-byte order (rule 3).
    MapKeyOrder,
    /// The same key twice in one map (rule 3). Carries the offending key so a
    /// consumer can report it without re-decoding; boxed to keep the enum small.
    DuplicateMapKey(Box<Value>),
    /// A text string whose bytes are not valid UTF-8.
    InvalidUtf8,
    /// Container nesting beyond [`MAX_DEPTH`].
    DepthExceeded,
    /// A length header larger than the bytes actually remaining in the input.
    /// Refused *before* allocating, so a crafted length cannot exhaust memory.
    LengthExceedsInput,
    /// A length that does not fit this platform's `usize`.
    LengthTooLarge,
    /// A JSON interchange document that cannot be represented as canonical CBOR.
    ///
    /// Only ever produced by the optional [`json`] module. The variant is present
    /// unconditionally so that enabling the `json` feature never changes this
    /// enum's shape — a feature that alters a public type is not additive.
    Json(&'static str),
}

impl fmt::Display for CborError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let CborError::Json(what) = self {
            return write!(f, "json: {what}");
        }
        let s = match self {
            CborError::Truncated => "truncated input",
            CborError::TrailingBytes => "trailing bytes after the top-level item",
            CborError::NonShortestForm => "non-shortest-form integer/length head",
            CborError::IndefiniteLength => "indefinite-length item or break code",
            CborError::ReservedAdditionalInfo => "reserved additional info (28-30)",
            CborError::Float => "floating-point value",
            CborError::Undefined => "the `undefined` simple value",
            CborError::SimpleValue => "a one-byte simple value",
            CborError::Tag => "a tag (major type 6)",
            CborError::MapKeyOrder => "map keys not in strictly ascending encoded-byte order",
            CborError::DuplicateMapKey(_) => "duplicate map key",
            CborError::InvalidUtf8 => "invalid UTF-8 in a text string",
            CborError::DepthExceeded => "nesting depth exceeds the limit",
            CborError::LengthExceedsInput => "length header exceeds the remaining input",
            CborError::LengthTooLarge => "length does not fit usize",
            // Handled above; unreachable, and cheaper than an unwrap.
            CborError::Json(_) => "json",
        };
        f.write_str(s)
    }
}

impl std::error::Error for CborError {}

// ---------------------------------------------------------------------------
// Encoding
// ---------------------------------------------------------------------------

/// Write a CBOR head — 3-bit major type plus the shortest form of `arg` (rule 2).
fn write_head(out: &mut Vec<u8>, major: u8, arg: u64) {
    let top = major << 5;
    if arg < 24 {
        out.push(top | arg as u8);
    } else if arg <= 0xff {
        out.push(top | 24);
        out.push(arg as u8);
    } else if arg <= 0xffff {
        out.push(top | 25);
        out.extend_from_slice(&(arg as u16).to_be_bytes());
    } else if arg <= 0xffff_ffff {
        out.push(top | 26);
        out.extend_from_slice(&(arg as u32).to_be_bytes());
    } else {
        out.push(top | 27);
        out.extend_from_slice(&arg.to_be_bytes());
    }
}

/// The single encoding pass. `reject_dups` selects between the checked and
/// unchecked public entry points; everything else is identical, so the two can
/// never drift into producing different bytes for a value that has a canonical
/// encoding.
fn enc(v: &Value, out: &mut Vec<u8>, reject_dups: bool) -> Result<(), CborError> {
    match v {
        Value::Uint(n) => write_head(out, 0, *n),
        Value::Nint(n) => write_head(out, 1, *n),
        Value::Bytes(b) => {
            write_head(out, 2, b.len() as u64);
            out.extend_from_slice(b);
        }
        Value::Text(s) => {
            // `str::len` is the UTF-8 byte length, which is what the head counts.
            write_head(out, 3, s.len() as u64);
            out.extend_from_slice(s.as_bytes());
        }
        Value::Array(items) => {
            write_head(out, 4, items.len() as u64);
            for item in items {
                enc(item, out, reject_dups)?;
            }
        }
        Value::Map(entries) => {
            // Encode each entry, then sort by the encoded KEY bytes (rule 3).
            // `sort_by` is stable, so two entries with equal keys keep their
            // relative order — which only matters when `reject_dups` is off.
            let mut pairs: Vec<(Vec<u8>, Vec<u8>, &Value)> = Vec::with_capacity(entries.len());
            for (k, val) in entries {
                let mut kbuf = Vec::new();
                enc(k, &mut kbuf, reject_dups)?;
                let mut vbuf = Vec::new();
                enc(val, &mut vbuf, reject_dups)?;
                pairs.push((kbuf, vbuf, k));
            }
            pairs.sort_by(|a, b| a.0.cmp(&b.0));
            if reject_dups {
                for w in pairs.windows(2) {
                    if w[0].0 == w[1].0 {
                        return Err(CborError::DuplicateMapKey(Box::new(w[0].2.clone())));
                    }
                }
            }
            write_head(out, 5, pairs.len() as u64);
            for (k, val, _) in &pairs {
                out.extend_from_slice(k);
                out.extend_from_slice(val);
            }
        }
        Value::Bool(b) => out.push(if *b { 0xf5 } else { 0xf4 }),
        Value::Null => out.push(0xf6),
    }
    Ok(())
}

/// Encode canonically (RFC 8949 §4.2.1 + §18.1.1).
///
/// Map entries are sorted by the bytewise order of their canonically encoded
/// keys, at every depth, so an out-of-order map still encodes canonically. A map
/// holding the same key twice has **no** canonical encoding and is refused with
/// [`CborError::DuplicateMapKey`] rather than emitted as bytes this crate's own
/// decoder would reject.
pub fn encode_canonical(v: &Value) -> Result<Vec<u8>, CborError> {
    let mut out = Vec::new();
    enc(v, &mut out, true)?;
    Ok(out)
}

/// Encode canonically **without** the duplicate-key check, and therefore
/// infallibly.
///
/// For every value that has a canonical encoding this is byte-for-byte
/// [`encode_canonical`]. For a map holding a duplicate key it emits a map with
/// that key twice — non-canonical bytes that [`decode_canonical`] will reject.
///
/// It exists for one reason: callers whose published API is an infallible
/// `fn encode(&V) -> Vec<u8>` cannot start returning a `Result` without a
/// breaking change, and silently dropping an entry would be worse than either.
/// New code should call [`encode_canonical`].
pub fn encode_canonical_unchecked(v: &Value) -> Vec<u8> {
    let mut out = Vec::new();
    // Infallible: `enc` only ever errors on the duplicate-key check, which is off.
    let _ = enc(v, &mut out, false);
    out
}

// ---------------------------------------------------------------------------
// Decoding
// ---------------------------------------------------------------------------

struct Decoder<'a> {
    b: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    fn byte(&mut self) -> Result<u8, CborError> {
        let b = *self.b.get(self.pos).ok_or(CborError::Truncated)?;
        self.pos += 1;
        Ok(b)
    }

    /// Read exactly `n` bytes, advancing the cursor. Bounds-checked with `get`,
    /// so it never panics.
    fn take(&mut self, n: usize) -> Result<&'a [u8], CborError> {
        let end = self.pos.checked_add(n).ok_or(CborError::LengthTooLarge)?;
        let s = self.b.get(self.pos..end).ok_or(CborError::Truncated)?;
        self.pos = end;
        Ok(s)
    }

    fn be<const N: usize>(&mut self) -> Result<[u8; N], CborError> {
        let s = self.take(N)?;
        let mut arr = [0u8; N];
        arr.copy_from_slice(s);
        Ok(arr)
    }

    /// Read a head argument for majors 0–5, enforcing shortest form (rule 2) and
    /// rejecting indefinite lengths and reserved additional-info values (rule 1).
    fn argument(&mut self, ai: u8) -> Result<u64, CborError> {
        match ai {
            0..=23 => Ok(ai as u64),
            24 => {
                let v = self.byte()? as u64;
                if v < 24 {
                    return Err(CborError::NonShortestForm); // fits the inline form
                }
                Ok(v)
            }
            25 => {
                let v = u16::from_be_bytes(self.be::<2>()?) as u64;
                if v <= 0xff {
                    return Err(CborError::NonShortestForm);
                }
                Ok(v)
            }
            26 => {
                let v = u32::from_be_bytes(self.be::<4>()?) as u64;
                if v <= 0xffff {
                    return Err(CborError::NonShortestForm);
                }
                Ok(v)
            }
            27 => {
                let v = u64::from_be_bytes(self.be::<8>()?);
                if v <= 0xffff_ffff {
                    return Err(CborError::NonShortestForm);
                }
                Ok(v)
            }
            28..=30 => Err(CborError::ReservedAdditionalInfo),
            _ => Err(CborError::IndefiniteLength), // 31 = indefinite / break
        }
    }

    /// Read a length argument and check it against the bytes actually remaining,
    /// so an attacker-chosen length cannot force an oversized allocation. Note
    /// this bounds array/map *counts* too: an element is at least one byte, so a
    /// count larger than the remaining input is already impossible.
    fn length(&mut self, ai: u8) -> Result<usize, CborError> {
        let n = self.argument(ai)?;
        let n = usize::try_from(n).map_err(|_| CborError::LengthTooLarge)?;
        if n > self.b.len().saturating_sub(self.pos) {
            return Err(CborError::LengthExceedsInput);
        }
        Ok(n)
    }

    fn value(&mut self, depth: u32) -> Result<Value, CborError> {
        if depth > MAX_DEPTH {
            return Err(CborError::DepthExceeded);
        }
        let ib = self.byte()?;
        let major = ib >> 5;
        let ai = ib & 0x1f;
        match major {
            0 => Ok(Value::Uint(self.argument(ai)?)),
            1 => Ok(Value::Nint(self.argument(ai)?)),
            2 => {
                let n = self.length(ai)?;
                Ok(Value::Bytes(self.take(n)?.to_vec()))
            }
            3 => {
                let n = self.length(ai)?;
                let s = self.take(n)?;
                let s = core::str::from_utf8(s).map_err(|_| CborError::InvalidUtf8)?;
                Ok(Value::Text(s.to_owned()))
            }
            4 => {
                let n = self.length(ai)?;
                let mut items = Vec::with_capacity(n);
                for _ in 0..n {
                    items.push(self.value(depth + 1)?);
                }
                Ok(Value::Array(items))
            }
            5 => {
                let n = self.length(ai)?;
                let mut entries = Vec::with_capacity(n);
                let mut prev: Option<&[u8]> = None;
                for _ in 0..n {
                    let start = self.pos;
                    let key = self.value(depth + 1)?;
                    let key_bytes = &self.b[start..self.pos];
                    if let Some(p) = prev {
                        match key_bytes.cmp(p) {
                            core::cmp::Ordering::Greater => {}
                            // Same encoded bytes ⇒ same key (rule 3).
                            core::cmp::Ordering::Equal => {
                                return Err(CborError::DuplicateMapKey(Box::new(key)))
                            }
                            core::cmp::Ordering::Less => return Err(CborError::MapKeyOrder),
                        }
                    }
                    prev = Some(key_bytes);
                    let val = self.value(depth + 1)?;
                    entries.push((key, val));
                }
                Ok(Value::Map(entries))
            }
            6 => Err(CborError::Tag),
            _ => match ai {
                20 => Ok(Value::Bool(false)),
                21 => Ok(Value::Bool(true)),
                22 => Ok(Value::Null),
                23 => Err(CborError::Undefined),
                24 => Err(CborError::SimpleValue),
                25..=27 => Err(CborError::Float),
                28..=30 => Err(CborError::ReservedAdditionalInfo),
                _ => Err(CborError::IndefiniteLength), // 31 = break
            },
        }
    }
}

/// Strict canonical decode: accepts *only* the canonical encoding of a value.
///
/// Rejects truncation, trailing bytes, indefinite lengths, non-shortest heads,
/// floats, `undefined`, simple values, tags, map keys that are unsorted or
/// duplicated, invalid UTF-8, nesting beyond [`MAX_DEPTH`], and any length header
/// exceeding the remaining input. It never normalizes and accepts.
pub fn decode_canonical(bytes: &[u8]) -> Result<Value, CborError> {
    let mut d = Decoder { b: bytes, pos: 0 };
    let v = d.value(0)?;
    if d.pos != bytes.len() {
        return Err(CborError::TrailingBytes);
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enc(v: &Value) -> Vec<u8> {
        encode_canonical(v).unwrap()
    }

    fn err(bytes: &[u8]) -> CborError {
        decode_canonical(bytes).expect_err("expected a refusal")
    }

    // -- Value helpers ---------------------------------------------------

    #[test]
    fn from_i64_as_i64_round_trip() {
        for v in [0i64, 1, -1, 23, -24, 255, -256, i64::MAX, i64::MIN, i64::MIN + 1] {
            assert_eq!(Value::from_i64(v).as_i64(), Some(v), "round trip for {v}");
        }
    }

    #[test]
    fn from_i64_matches_expected_variant() {
        assert_eq!(Value::from_i64(0), Value::Uint(0));
        assert_eq!(Value::from_i64(-1), Value::Nint(0));
        assert_eq!(Value::from_i64(-24), Value::Nint(23));
        assert_eq!(Value::from_i64(-25), Value::Nint(24));
        assert_eq!(Value::from_i64(i64::MIN), Value::Nint(i64::MAX as u64));
    }

    #[test]
    fn as_i64_rejects_out_of_range() {
        assert_eq!(Value::Nint(i64::MAX as u64 + 1).as_i64(), None);
        assert_eq!(Value::Uint(u64::MAX).as_i64(), None);
    }

    #[test]
    fn accessors() {
        assert_eq!(Value::Uint(5).as_u64(), Some(5));
        assert_eq!(Value::Nint(5).as_u64(), None);
        assert_eq!(Value::Bytes(vec![1, 2]).as_bytes(), Some(&[1u8, 2][..]));
        assert_eq!(Value::Text("hi".into()).as_text(), Some("hi"));
        assert_eq!(Value::Bool(true).as_bool(), Some(true));
        assert_eq!(Value::Null.as_bool(), None);
        assert_eq!(
            Value::Array(vec![Value::Uint(1)]).as_array(),
            Some(&[Value::Uint(1)][..])
        );
    }

    #[test]
    fn map_get_helpers() {
        let map = Value::Map(vec![
            (Value::Text("a".into()), Value::Uint(1)),
            (Value::Uint(9), Value::Text("nine".into())),
        ]);
        assert_eq!(map.map_get("a"), Some(&Value::Uint(1)));
        assert_eq!(map.map_get("missing"), None);
        assert_eq!(map.map_get_int(9), Some(&Value::Text("nine".into())));
        assert_eq!(map.map_get_int(1), None);
        assert_eq!(Value::Uint(1).map_get("a"), None);
    }

    #[test]
    fn into_canonical_matches_decoded_order_without_changing_bytes() {
        let unsorted = Value::Map(vec![
            (Value::Uint(24), Value::Uint(4)),
            (Value::Uint(1), Value::Uint(1)),
            (Value::Uint(10), Value::Uint(3)),
        ]);
        let bytes = enc(&unsorted);
        let decoded = decode_canonical(&bytes).unwrap();
        assert_eq!(unsorted.clone().into_canonical(), decoded);
        assert_eq!(enc(&unsorted.into_canonical()), bytes);
    }

    // -- Known-answer encodings (RFC 8949 appendix A) ---------------------

    #[test]
    fn kat_shortest_form_heads() {
        assert_eq!(enc(&Value::Uint(0)), vec![0x00]);
        assert_eq!(enc(&Value::Uint(1)), vec![0x01]);
        assert_eq!(enc(&Value::Uint(23)), vec![0x17]);
        assert_eq!(enc(&Value::Uint(24)), vec![0x18, 0x18]);
        assert_eq!(enc(&Value::Uint(255)), vec![0x18, 0xff]);
        assert_eq!(enc(&Value::Uint(256)), vec![0x19, 0x01, 0x00]);
        assert_eq!(enc(&Value::Uint(65_536)), vec![0x1a, 0x00, 0x01, 0x00, 0x00]);
        assert_eq!(
            enc(&Value::Uint(u64::MAX)),
            vec![0x1b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn kat_negative_ints() {
        assert_eq!(enc(&Value::from_i64(-1)), vec![0x20]);
        assert_eq!(enc(&Value::from_i64(-24)), vec![0x37]);
        assert_eq!(enc(&Value::from_i64(-25)), vec![0x38, 0x18]);
    }

    #[test]
    fn kat_strings_arrays_maps_simple() {
        assert_eq!(enc(&Value::Text(String::new())), vec![0x60]);
        assert_eq!(enc(&Value::Text("IETF".into())), vec![0x64, b'I', b'E', b'T', b'F']);
        assert_eq!(enc(&Value::Bytes(vec![1, 2, 3, 4])), vec![0x44, 1, 2, 3, 4]);
        assert_eq!(
            enc(&Value::Array(vec![
                Value::Uint(1),
                Value::Array(vec![Value::Uint(2), Value::Uint(3)])
            ])),
            vec![0x82, 0x01, 0x82, 0x02, 0x03]
        );
        assert_eq!(
            enc(&Value::Map(vec![
                (Value::Text("a".into()), Value::Uint(1)),
                (
                    Value::Text("b".into()),
                    Value::Array(vec![Value::Uint(2), Value::Uint(3)])
                ),
            ])),
            vec![0xa2, 0x61, 0x61, 0x01, 0x61, 0x62, 0x82, 0x02, 0x03]
        );
        assert_eq!(enc(&Value::Bool(false)), vec![0xf4]);
        assert_eq!(enc(&Value::Bool(true)), vec![0xf5]);
        assert_eq!(enc(&Value::Null), vec![0xf6]);
    }

    #[test]
    fn multibyte_text_head_counts_utf8_bytes_not_chars() {
        // "é漢🙂" is 3 chars but 9 UTF-8 bytes: the head must say 9 (0x69).
        let v = Value::Text("é漢🙂".into());
        let bytes = enc(&v);
        assert_eq!(bytes[0], 0x69);
        assert_eq!(bytes.len(), 10);
        assert_eq!(decode_canonical(&bytes).unwrap(), v);
    }

    // -- Map key ordering -------------------------------------------------

    #[test]
    fn map_sorts_by_encoded_key_bytes_not_string_order() {
        // "b" encodes 61 62; "aa" encodes 62 61 61. 0x61 < 0x62, so "b" sorts
        // FIRST even though "aa" < "b" as a string.
        let v = Value::Map(vec![
            (Value::Text("aa".into()), Value::Uint(1)),
            (Value::Text("b".into()), Value::Uint(2)),
        ]);
        assert_eq!(enc(&v), vec![0xa2, 0x61, 0x62, 0x02, 0x62, 0x61, 0x61, 0x01]);
    }

    #[test]
    fn map_keys_emitted_ascending_regardless_of_insertion_order() {
        let a = Value::Map(vec![
            (Value::Uint(10), Value::Uint(1)),
            (Value::Uint(2), Value::Uint(2)),
            (Value::Uint(1), Value::Uint(3)),
            (Value::Uint(24), Value::Uint(4)),
        ]);
        // 24 needs a two-byte head, so it sorts AFTER single-byte 10.
        assert_eq!(
            enc(&a),
            vec![0xa4, 0x01, 0x03, 0x02, 0x02, 0x0a, 0x01, 0x18, 0x18, 0x04]
        );
    }

    #[test]
    fn duplicate_map_key_refused_on_encode_at_any_depth() {
        let flat = Value::Map(vec![
            (Value::Uint(1), Value::Uint(1)),
            (Value::Uint(1), Value::Uint(2)),
        ]);
        assert_eq!(
            encode_canonical(&flat),
            Err(CborError::DuplicateMapKey(Box::new(Value::Uint(1))))
        );
        // Nested inside an array inside a map value.
        let nested = Value::Map(vec![(
            Value::Uint(7),
            Value::Array(vec![Value::Uint(0), flat.clone()]),
        )]);
        assert!(matches!(
            encode_canonical(&nested),
            Err(CborError::DuplicateMapKey(_))
        ));
        // And the unchecked encoder emits bytes this crate's own decoder rejects
        // — the documented, deliberate escape hatch.
        let bytes = encode_canonical_unchecked(&flat);
        assert_eq!(bytes, vec![0xa2, 0x01, 0x01, 0x01, 0x02]);
        assert!(matches!(
            decode_canonical(&bytes),
            Err(CborError::DuplicateMapKey(_))
        ));
    }

    // -- Round trips ------------------------------------------------------

    fn round_trip(v: &Value) {
        let bytes = enc(v);
        assert_eq!(&decode_canonical(&bytes).unwrap(), v);
        assert_eq!(enc(&decode_canonical(&bytes).unwrap()), bytes);
    }

    #[test]
    fn round_trip_every_variant() {
        round_trip(&Value::Uint(0));
        round_trip(&Value::Uint(u64::MAX));
        round_trip(&Value::Nint(0));
        round_trip(&Value::Nint(u64::MAX));
        round_trip(&Value::Bytes(vec![]));
        round_trip(&Value::Bytes(vec![0xde, 0xad, 0xbe, 0xef]));
        round_trip(&Value::Text(String::new()));
        round_trip(&Value::Text("hello, world".into()));
        round_trip(&Value::Array(vec![]));
        round_trip(&Value::Map(vec![]));
        round_trip(&Value::Bool(true));
        round_trip(&Value::Bool(false));
        round_trip(&Value::Null);
    }

    #[test]
    fn round_trip_nested_structure() {
        round_trip(&Value::Map(vec![
            (
                Value::Uint(1),
                Value::Array(vec![Value::Uint(1), Value::Nint(0)]),
            ),
            (
                Value::Uint(2),
                Value::Map(vec![(Value::Text("x".into()), Value::Bool(true))]),
            ),
            (Value::Uint(3), Value::Bytes(vec![1, 2, 3])),
            (Value::Uint(4), Value::Null),
        ]));
    }

    // -- Refusals, one per rule -------------------------------------------

    #[test]
    fn rejects_truncation_and_trailing_bytes() {
        assert_eq!(err(&[]), CborError::Truncated);
        assert_eq!(err(&[0x18]), CborError::Truncated);
        assert_eq!(err(&[0x00, 0x00]), CborError::TrailingBytes);
    }

    #[test]
    fn rejects_non_shortest_integer_and_length_heads() {
        assert_eq!(err(&[0x18, 0x01]), CborError::NonShortestForm);
        assert_eq!(err(&[0x18, 0x17]), CborError::NonShortestForm);
        assert_eq!(err(&[0x18, 0x0a]), CborError::NonShortestForm);
        assert_eq!(err(&[0x19, 0x00, 0xc8]), CborError::NonShortestForm);
        assert_eq!(err(&[0x1b, 0, 0, 0, 0, 0, 0, 0, 0]), CborError::NonShortestForm);
        assert_eq!(err(&[0x58, 0x01, 0xaa]), CborError::NonShortestForm);
        assert_eq!(
            err(&[0x59, 0x00, 0x05, 0, 0, 0, 0, 0]),
            CborError::NonShortestForm
        );
        // A non-shortest NEGATIVE head is caught in the head, before the value.
        assert_eq!(err(&[0x38, 0x17]), CborError::NonShortestForm);
    }

    #[test]
    fn accepts_genuine_shortest_forms() {
        assert_eq!(decode_canonical(&[0x18, 0x18]).unwrap(), Value::Uint(24));
        assert_eq!(decode_canonical(&[0x19, 0x01, 0x00]).unwrap(), Value::Uint(256));
    }

    #[test]
    fn rejects_indefinite_lengths_and_break() {
        assert_eq!(err(&[0x5f, 0xff]), CborError::IndefiniteLength);
        assert_eq!(err(&[0x7f, 0xff]), CborError::IndefiniteLength);
        assert_eq!(err(&[0x9f, 0xff]), CborError::IndefiniteLength);
        assert_eq!(err(&[0xbf, 0xff]), CborError::IndefiniteLength);
        assert_eq!(err(&[0xff]), CborError::IndefiniteLength);
    }

    #[test]
    fn rejects_reserved_additional_info() {
        assert_eq!(err(&[0x1c]), CborError::ReservedAdditionalInfo);
        assert_eq!(err(&[0x1d]), CborError::ReservedAdditionalInfo);
        assert_eq!(err(&[0x1e]), CborError::ReservedAdditionalInfo);
        assert_eq!(err(&[0xfc]), CborError::ReservedAdditionalInfo);
    }

    #[test]
    fn rejects_floats_undefined_simple_and_tags() {
        assert_eq!(err(&[0xf9, 0x00, 0x00]), CborError::Float);
        assert_eq!(err(&[0xf9, 0x7e, 0x00]), CborError::Float); // NaN
        assert_eq!(err(&[0xfa, 0, 0, 0, 0]), CborError::Float);
        assert_eq!(err(&[0xfb, 0, 0, 0, 0, 0, 0, 0, 0]), CborError::Float);
        assert_eq!(err(&[0xf7]), CborError::Undefined);
        assert_eq!(err(&[0xf8, 0xff]), CborError::SimpleValue);
        assert_eq!(err(&[0xc0, 0x00]), CborError::Tag);
        assert_eq!(err(&[0xc0, 0x61, 0x41]), CborError::Tag);
    }

    #[test]
    fn rejects_unsorted_and_duplicate_map_keys() {
        assert_eq!(err(&[0xa2, 0x02, 0x00, 0x01, 0x00]), CborError::MapKeyOrder);
        assert_eq!(
            err(&[0xa2, 0x61, 0x62, 0x00, 0x61, 0x61, 0x00]),
            CborError::MapKeyOrder
        );
        assert_eq!(
            err(&[0xa2, 0x01, 0x00, 0x01, 0x01]),
            CborError::DuplicateMapKey(Box::new(Value::Uint(1)))
        );
        assert_eq!(
            err(&[0xa2, 0x61, 0x61, 0x00, 0x61, 0x61, 0x00]),
            CborError::DuplicateMapKey(Box::new(Value::Text("a".into())))
        );
    }

    #[test]
    fn accepts_ascending_map_keys() {
        assert_eq!(
            decode_canonical(&[0xa2, 0x01, 0x00, 0x02, 0x00]).unwrap(),
            Value::Map(vec![
                (Value::Uint(1), Value::Uint(0)),
                (Value::Uint(2), Value::Uint(0))
            ])
        );
    }

    #[test]
    fn rejects_invalid_utf8() {
        assert_eq!(err(&[0x61, 0xff]), CborError::InvalidUtf8);
    }

    #[test]
    fn length_headers_are_checked_before_allocating() {
        // A byte-string head claiming u64::MAX bytes, with nothing following.
        assert_eq!(
            err(&[0x5b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]),
            CborError::LengthExceedsInput
        );
        // An array head claiming ~2^32 elements, with nothing following: refused
        // on the COUNT, not after 4 billion iterations.
        assert_eq!(err(&[0x9a, 0xff, 0xff, 0xff, 0xff]), CborError::LengthExceedsInput);
        // Same for a map.
        assert_eq!(err(&[0xba, 0xff, 0xff, 0xff, 0xff]), CborError::LengthExceedsInput);
    }

    #[test]
    fn nesting_depth_is_bounded_at_the_documented_limit() {
        let nest = |n: usize| {
            let mut b = vec![0x81u8; n];
            b.push(0x00);
            b
        };
        assert!(decode_canonical(&nest(MAX_DEPTH as usize)).is_ok());
        assert_eq!(
            decode_canonical(&nest(MAX_DEPTH as usize + 1)),
            Err(CborError::DepthExceeded)
        );
        // And a large hostile input fails closed rather than overflowing the stack.
        assert_eq!(err(&vec![0x81u8; 50_000]), CborError::DepthExceeded);
    }

    #[test]
    fn accepted_bytes_re_encode_to_themselves() {
        // The malleability guarantee, on a shape with every head width and both
        // key majors.
        let v = Value::Map(vec![
            (Value::Uint(1), Value::Uint(24)),
            (Value::Uint(2), Value::Bytes(vec![0xde, 0xad, 0xbe, 0xef])),
            (Value::Uint(3), Value::Text("hi".into())),
            (
                Value::Uint(7),
                Value::Array(vec![Value::Uint(256), Value::Bool(true), Value::Null]),
            ),
            (Value::Uint(24), Value::Uint(1_700_000_000_000)),
        ]);
        let bytes = enc(&v);
        assert_eq!(enc(&decode_canonical(&bytes).unwrap()), bytes);
    }

    #[test]
    fn error_display_is_not_empty() {
        for e in [
            CborError::Truncated,
            CborError::TrailingBytes,
            CborError::NonShortestForm,
            CborError::IndefiniteLength,
            CborError::ReservedAdditionalInfo,
            CborError::Float,
            CborError::Undefined,
            CborError::SimpleValue,
            CborError::Tag,
            CborError::MapKeyOrder,
            CborError::DuplicateMapKey(Box::new(Value::Uint(1))),
            CborError::InvalidUtf8,
            CborError::DepthExceeded,
            CborError::LengthExceedsInput,
            CborError::LengthTooLarge,
        ] {
            assert!(!e.to_string().is_empty());
        }
    }

    /// The one place this crate is looser than every schema-bearing consumer, and
    /// it is documented at module level rather than silently inherited.
    #[test]
    fn non_scalar_map_keys_are_accepted_deliberately() {
        // Inherited from evermesh so a migration cannot change any verdict.
        // Consumers with a schema (kotva_core::cbor::Cv) refuse these themselves.
        assert!(decode_canonical(&[0xa1, 0xf5, 0x00]).is_ok()); // {true: 0}
        assert!(decode_canonical(&[0xa1, 0xf6, 0x00]).is_ok()); // {null: 0}
        assert!(decode_canonical(&[0xa1, 0x80, 0x00]).is_ok()); // {[]: 0}
        assert!(decode_canonical(&[0xa1, 0xa0, 0x00]).is_ok()); // {{}: 0}
    }
}
