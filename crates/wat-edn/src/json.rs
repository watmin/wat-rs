//! EDN ↔ JSON conversion with type-fidelity round-trip.
//!
//! JSON has fewer types than EDN; we use sentinel keys (`#tag`,
//! `#set`, `#bigint`, etc.) on objects to preserve EDN type
//! information when converting to JSON. Reading JSON back, the
//! sentinels are recognized and the original EDN types reconstruct.
//!
//! # Wire convention
//!
//! ```text
//! EDN value          JSON shape
//! ─────────────────  ──────────────────────────────────────────
//! nil                null
//! true / false       true / false
//! i64 (in range)     number
//! i64 (> 2^53)       string  "9007199254740993"
//! bigint             {"#bigint": "123N"}
//! f64                number
//! NaN / ±Inf         {"#float": "nan" | "inf" | "neg-inf"}
//! bigdec             {"#bigdec": "3.14M"}
//! rational           {"#rational": "1/2"}
//! string             string
//! char               {"#char": "X"}
//! keyword            ":foo"  /  ":ns/foo"  (colon prefix)
//! symbol             {"#symbol": "foo"} | {"#symbol": "ns/foo"}
//! list / vector      array (round-trips as vector)
//! map (string keys)  object {"k": v, ...}
//! map (other keys)   object — non-string keys serialized as EDN
//! set                {"#set": [...]}
//! tagged             {"#tag": "ns/name", "body": ...}
//! inst               {"#inst": "2026-04-28T16:00:00.000000000Z"}  (WriteOpts default: 9 digits)
//! uuid               {"#uuid": "550e8400-..."}
//! ```
//!
//! Round-trip identity holds for every spec EDN type.

use crate::vocab::{is_canonical_uuid, split_namespaced};
use crate::value::{Keyword, Symbol, Tag, Value};
use crate::OwnedValue;
use bigdecimal::BigDecimal;
use chrono::{DateTime, SecondsFormat, Utc};
use num_bigint::BigInt;
use num_rational::BigRational;
use serde_json::{Map, Number, Value as JV};
use std::str::FromStr;
use thiserror::Error;

/// JSON's largest safely-representable integer (2^53 - 1).
const SAFE_INT_MAX: i64 = (1_i64 << 53) - 1;
const SAFE_INT_MIN: i64 = -(1_i64 << 53);

#[derive(Debug, Clone, Error, PartialEq)]
pub enum JsonError {
    #[error("JSON parse error: {0}")]
    Parse(String),

    #[error("number out of i64/f64 range: {0}")]
    NumberOutOfRange(String),

    #[error("invalid #tag: {0}")]
    InvalidTag(String),

    #[error("invalid #inst: {0}")]
    InvalidInst(String),

    #[error("invalid #uuid: {0}")]
    InvalidUuid(String),

    #[error("invalid #bigint: {0}")]
    InvalidBigInt(String),

    #[error("invalid #bigdec: {0}")]
    InvalidBigDec(String),

    #[error("invalid #rational: {0}")]
    InvalidRational(String),

    #[error("invalid #char: {0}")]
    InvalidChar(String),

    #[error("invalid #symbol: {0}")]
    InvalidSymbol(String),

    #[error("invalid #float sentinel: {0}")]
    InvalidFloat(String),

    #[error("invalid keyword from string: {0}")]
    InvalidKeyword(String),

    #[error("invalid map: {0}")]
    InvalidMap(String),

    #[error("invalid set: {0}")]
    InvalidSet(String),

    #[error("invalid map key '{key}': {reason}")]
    InvalidMapKey { key: String, reason: String },
}

pub type JsonResult<T> = std::result::Result<T, JsonError>;

/// Options for JSON rendering. A VALUE the caller passes — not a global,
/// not a `digits` argument on the serializer. `inst_digits` is clamped
/// to `[0, 9]` at construction (same bounds as `:wat::time::to-iso8601`).
/// Default is 9: constant-width nanoseconds, the sane default a caller
/// never needs to touch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WriteOpts {
    /// Fractional-second digits on `#inst` / ISO-8601 Inst strings. Always in `0..=9`.
    pub inst_digits: u32,
}

impl WriteOpts {
    pub const DEFAULT: Self = Self { inst_digits: 9 };

    /// Clamp `n` to `[0, 9]` — match `to-iso8601`, do not reject.
    pub fn from_inst_digits(n: i64) -> Self {
        Self {
            inst_digits: n.clamp(0, 9) as u32,
        }
    }

    /// RFC-3339 / ISO-8601 UTC, `Z`-suffixed, `inst_digits` fractional digits.
    /// Digit counts other than 0/3/6/9 are hand-formatted: chrono's
    /// `SecondsFormat` only offers those four.
    pub fn format_inst(self, dt: &DateTime<Utc>) -> String {
        let digits = self.inst_digits;
        if digits == 0 {
            dt.to_rfc3339_opts(SecondsFormat::Secs, true)
        } else {
            let secs_part = dt.format("%Y-%m-%dT%H:%M:%S");
            let nanos = dt.timestamp_subsec_nanos();
            let scaled = nanos / 10_u32.pow(9 - digits);
            format!(
                "{}.{:0>width$}Z",
                secs_part,
                scaled,
                width = digits as usize
            )
        }
    }
}

impl Default for WriteOpts {
    fn default() -> Self {
        Self::DEFAULT
    }
}

// ─── EDN → JSON ─────────────────────────────────────────────────

/// Convert an EDN `Value` to a `serde_json::Value`.
pub(crate) fn edn_to_json(v: &Value<'_>, opts: &WriteOpts) -> JV {
    match v {
        Value::Nil => JV::Null,
        Value::Bool(b) => JV::Bool(*b),
        Value::Integer(i) => {
            if (*i) >= SAFE_INT_MIN && (*i) <= SAFE_INT_MAX {
                JV::Number((*i).into())
            } else {
                // Outside JSON's safe-integer range — promote to string.
                JV::String(i.to_string())
            }
        }
        Value::BigInt(n) => single_key_object("#bigint", JV::String(format!("{}N", n))),
        Value::Float(f) => {
            if f.is_nan() {
                single_key_object("#float", JV::String("nan".into()))
            } else if f.is_infinite() {
                let tag = if f.is_sign_negative() { "neg-inf" } else { "inf" };
                single_key_object("#float", JV::String(tag.into()))
            } else {
                Number::from_f64(*f)
                    .map(JV::Number)
                    .expect("finite f64 must convert to serde_json::Number per from_f64 contract")
            }
        }
        Value::BigDec(n) => single_key_object("#bigdec", JV::String(format!("{}M", n))),
        Value::Rational(n) => {
            single_key_object("#rational", JV::String(format!("{}/{}", n.numer(), n.denom())))
        }
        Value::String(s) => JV::String(s.to_string()),
        Value::Char(c) => single_key_object("#char", JV::String(c.to_string())),
        Value::Keyword(k) => JV::String(format!("{}", k)), // includes leading `:`
        Value::Symbol(s) => single_key_object("#symbol", JV::String(format!("{}", s))),
        Value::List(items) | Value::Vector(items) => {
            JV::Array(items.iter().map(|x| edn_to_json(x, opts)).collect())
        }
        Value::Map(entries) => {
            let mut map = Map::with_capacity(entries.len());
            for (k, v) in entries {
                let key_str = match k {
                    // String keys: use the raw string content.
                    Value::String(s) => s.to_string(),
                    // Other keys: serialize as EDN. Round-trip-able
                    // because `json->edn` parses keys back as EDN.
                    other => crate::write(other),
                };
                map.insert(key_str, edn_to_json(v, opts));
            }
            JV::Object(map)
        }
        Value::Set(items) => single_key_object(
            "#set",
            JV::Array(items.iter().map(|x| edn_to_json(x, opts)).collect()),
        ),
        Value::Tagged(tag, body) => {
            let mut map = Map::with_capacity(2);
            map.insert(
                "#tag".to_string(),
                JV::String(format!("{}/{}", tag.namespace(), tag.name())),
            );
            map.insert("body".to_string(), edn_to_json(body, opts));
            JV::Object(map)
        }
        Value::Inst(dt) => single_key_object("#inst", JV::String(opts.format_inst(dt))),
        Value::Uuid(u) => single_key_object("#uuid", JV::String(u.to_string())),
    }
}

/// Convert an EDN `Value` to a JSON string.
pub fn to_json_string(v: &Value<'_>) -> String {
    // rune:temperare(serde-api-shape) — serde_json::to_string takes
    // &impl Serialize; we materialize via edn_to_json into a full
    // serde_json::Value tree before serializing. Alternative is a
    // custom serde::Serialize impl on Value (full visitor). No caller
    // has surfaced JSON-throughput pressure; the simpler shape wins
    // until measurement disagrees.
    to_json_string_with(v, &WriteOpts::DEFAULT)
}

/// Convert an EDN `Value` to a JSON string with explicit [`WriteOpts`].
pub fn to_json_string_with(v: &Value<'_>, opts: &WriteOpts) -> String {
    serde_json::to_string(&edn_to_json(v, opts)).expect("serde_json::to_string on Value")
}

// rune:purgare(public-api) — symmetric pretty variant paired with
// to_json_string (which IS actively consumed by src/edn/render.rs's
// `eval_edn_write_json` / `eval_edn_write_json_natural`
// for WAT_TEST_OUTPUT cargo integration per arc 116). to_json_string_pretty
// itself has no current direct caller; justification is symmetric-
// completeness with the live compact variant + future Clojure-IPC bridge
// surface (crates/wat-edn/docs/IPC-BRIDGE.md). Impressive JSON bridges
// ship both compact and pretty forms; the pretty variant is the natural
// API for human-readable output (debug logs, error envelopes, REPL).
/// Convert an EDN `Value` to a pretty-printed JSON string.
pub fn to_json_string_pretty(v: &Value<'_>) -> String {
    // rune:temperare(serde-api-shape) — serde_json::to_string_pretty
    // takes &impl Serialize; we materialize via edn_to_json into a full
    // serde_json::Value tree before serializing. Alternative is a
    // custom serde::Serialize impl on Value (full visitor). No caller
    // has surfaced JSON-throughput pressure; the simpler shape wins
    // until measurement disagrees.
    serde_json::to_string_pretty(&edn_to_json(v, &WriteOpts::DEFAULT))
        .expect("serde_json::to_string_pretty on Value")
}

// ─── JSON → EDN ─────────────────────────────────────────────────

/// Convert a `serde_json::Value` to an EDN `OwnedValue`.
pub(crate) fn json_to_edn(v: &JV) -> JsonResult<OwnedValue> {
    match v {
        JV::Null => Ok(Value::Nil),
        JV::Bool(b) => Ok(Value::Bool(*b)),
        JV::Number(n) => parse_number(n),
        JV::String(s) => Ok(string_to_edn(s)?),
        JV::Array(items) => {
            let parsed: JsonResult<Vec<_>> = items.iter().map(json_to_edn).collect();
            Ok(Value::Vector(parsed?))
        }
        JV::Object(map) => object_to_edn(map),
    }
}

/// Convert a JSON string to an EDN `OwnedValue`.
pub fn from_json_string(s: &str) -> JsonResult<OwnedValue> {
    let json: JV = serde_json::from_str(s).map_err(|e| JsonError::Parse(e.to_string()))?;
    json_to_edn(&json)
}

// ─── Helpers ────────────────────────────────────────────────────

fn single_key_object(key: &str, body: JV) -> JV {
    let mut map = Map::with_capacity(1);
    map.insert(key.to_string(), body);
    JV::Object(map)
}

fn parse_number(n: &Number) -> JsonResult<OwnedValue> {
    if let Some(i) = n.as_i64() {
        Ok(Value::Integer(i))
    } else if let Some(f) = n.as_f64() {
        Ok(Value::Float(f))
    } else if let Some(u) = n.as_u64() {
        if u <= i64::MAX as u64 {
            Ok(Value::Integer(u as i64))
        } else {
            Err(JsonError::NumberOutOfRange(n.to_string()))
        }
    } else {
        Err(JsonError::NumberOutOfRange(n.to_string()))
    }
}

/// JSON strings starting with `:` are EDN keywords; otherwise plain
/// strings. Stringified large-integers are also strings — at this
/// layer we can't disambiguate from text strings, so they round-trip
/// as `Value::String`. Callers needing strict round-trip for huge
/// integers should use the `#bigint` sentinel instead.
fn string_to_edn(s: &str) -> JsonResult<OwnedValue> {
    if let Some(body) = s.strip_prefix(':') {
        if body.is_empty() {
            return Err(JsonError::InvalidKeyword(s.into()));
        }
        if let Some((ns, name)) = split_namespaced(body) {
            let kw = Keyword::try_ns(ns, name)
                .map_err(|m| JsonError::InvalidKeyword(format!("{}: {}", s, m)))?;
            Ok(Value::Keyword(kw))
        } else {
            let kw = Keyword::try_new(body)
                .map_err(|m| JsonError::InvalidKeyword(format!("{}: {}", s, m)))?;
            Ok(Value::Keyword(kw))
        }
    } else {
        Ok(Value::String(s.to_string().into()))
    }
}

fn object_to_edn(map: &Map<String, JV>) -> JsonResult<OwnedValue> {
    // Single-key sentinel objects come first.
    if map.len() == 1 {
        let (k, v) = map.iter().next().unwrap();
        match k.as_str() {
            "#bigint" => return decode_bigint(v),
            "#bigdec" => return decode_bigdec(v),
            "#rational" => return decode_rational(v),
            "#float" => return decode_float_sentinel(v),
            "#char" => return decode_char(v),
            "#symbol" => return decode_symbol(v),
            "#set" => return decode_set(v),
            "#inst" => return decode_inst(v),
            "#uuid" => return decode_uuid(v),
            _ => {}
        }
    }

    // Two-key tagged-element ({"#tag": "...", "body": ...})
    if map.len() == 2 {
        if let (Some(tag_v), Some(body_v)) = (map.get("#tag"), map.get("body")) {
            return decode_tagged(tag_v, body_v);
        }
    }

    // Plain map. Non-string keys (those parseable as EDN) recover.
    let mut entries = Vec::with_capacity(map.len());
    for (k, v) in map {
        let key = decode_map_key(k)?;
        let val = json_to_edn(v)?;
        entries.push((key, val));
    }
    Ok(Value::Map(entries))
}

fn decode_map_key(k: &str) -> JsonResult<OwnedValue> {
    // Heuristic: keys that LOOK like EDN (start with `:`, `[`, `{`,
    // `(`, `#`, `"`) are parsed as EDN; otherwise treated as plain
    // strings. This matches what edn_to_json produced.
    let looks_like_edn = k
        .chars()
        .next()
        .map(|c| matches!(c, ':' | '[' | '{' | '(' | '#' | '"'))
        .unwrap_or(false);
    if looks_like_edn {
        match crate::parse(k) {
            Ok(v) => return Ok(v.into_owned()),
            Err(e) => {
                return Err(JsonError::InvalidMapKey {
                    key: k.to_string(),
                    reason: e.to_string(),
                })
            }
        }
    }
    Ok(Value::String(k.to_string().into()))
}

fn decode_bigint(v: &JV) -> JsonResult<OwnedValue> {
    let s = v
        .as_str()
        .ok_or_else(|| JsonError::InvalidBigInt(v.to_string()))?;
    let trimmed = s.strip_suffix('N').unwrap_or(s);
    let n = BigInt::from_str(trimmed).map_err(|_| JsonError::InvalidBigInt(s.into()))?;
    Ok(Value::BigInt(Box::new(n)))
}

/// Decode a `{"#rational": "<n>/<d>"}` sentinel. Re-applies the same
/// Clojure-faithful normalization as the EDN parser (a denominator
/// that reduces to 1 is an Integer, not a Rational) so a hand-authored
/// JSON payload (e.g. `"4/2"`) can't smuggle in a non-canonical Value
/// that `crate::parse` would never itself produce.
fn decode_rational(v: &JV) -> JsonResult<OwnedValue> {
    let s = v
        .as_str()
        .ok_or_else(|| JsonError::InvalidRational(v.to_string()))?;
    let ratio =
        BigRational::from_str(s).map_err(|_| JsonError::InvalidRational(s.into()))?;
    if ratio.is_integer() {
        let numer = ratio.numer();
        match numer.to_string().parse::<i64>() {
            Ok(i) => Ok(Value::Integer(i)),
            Err(_) => Ok(Value::BigInt(Box::new(numer.clone()))),
        }
    } else {
        Ok(Value::Rational(Box::new(ratio)))
    }
}

fn decode_bigdec(v: &JV) -> JsonResult<OwnedValue> {
    let s = v
        .as_str()
        .ok_or_else(|| JsonError::InvalidBigDec(v.to_string()))?;
    let trimmed = s.strip_suffix('M').unwrap_or(s);
    let n = BigDecimal::from_str(trimmed).map_err(|_| JsonError::InvalidBigDec(s.into()))?;
    Ok(Value::BigDec(Box::new(n)))
}

fn decode_float_sentinel(v: &JV) -> JsonResult<OwnedValue> {
    let s = v
        .as_str()
        .ok_or_else(|| JsonError::InvalidFloat(v.to_string()))?;
    Ok(match s {
        "nan" => Value::Float(f64::NAN),
        "inf" => Value::Float(f64::INFINITY),
        "neg-inf" => Value::Float(f64::NEG_INFINITY),
        _ => return Err(JsonError::InvalidFloat(s.into())),
    })
}

fn decode_char(v: &JV) -> JsonResult<OwnedValue> {
    let s = v
        .as_str()
        .ok_or_else(|| JsonError::InvalidChar(v.to_string()))?;
    let mut chars = s.chars();
    let c = chars
        .next()
        .ok_or_else(|| JsonError::InvalidChar("empty".into()))?;
    if chars.next().is_some() {
        return Err(JsonError::InvalidChar(format!("multi-char: {}", s)));
    }
    Ok(Value::Char(c))
}

fn decode_symbol(v: &JV) -> JsonResult<OwnedValue> {
    let s = v
        .as_str()
        .ok_or_else(|| JsonError::InvalidSymbol(v.to_string()))?;
    let sym = if let Some((ns, name)) = split_namespaced(s) {
        Symbol::try_ns(ns, name)
            .map_err(|m| JsonError::InvalidSymbol(format!("{}: {}", s, m)))?
    } else {
        Symbol::try_new(s).map_err(|m| JsonError::InvalidSymbol(format!("{}: {}", s, m)))?
    };
    Ok(Value::Symbol(sym))
}

fn decode_set(v: &JV) -> JsonResult<OwnedValue> {
    let arr = v
        .as_array()
        .ok_or_else(|| JsonError::InvalidSet(format!("#set body must be array: {}", v)))?;
    let parsed: JsonResult<Vec<_>> = arr.iter().map(json_to_edn).collect();
    Ok(Value::Set(parsed?))
}

fn decode_inst(v: &JV) -> JsonResult<OwnedValue> {
    let s = v
        .as_str()
        .ok_or_else(|| JsonError::InvalidInst(v.to_string()))?;
    let dt = chrono::DateTime::parse_from_rfc3339(s)
        .map_err(|e| JsonError::InvalidInst(format!("{}: {}", s, e)))?
        .with_timezone(&chrono::Utc);
    Ok(Value::Inst(dt))
}

fn decode_uuid(v: &JV) -> JsonResult<OwnedValue> {
    let s = v
        .as_str()
        .ok_or_else(|| JsonError::InvalidUuid(v.to_string()))?;
    if !is_canonical_uuid(s) {
        return Err(JsonError::InvalidUuid(format!(
            "{}: not canonical 8-4-4-4-12 lowercase form",
            s
        )));
    }
    let u = uuid::Uuid::parse_str(s).map_err(|e| JsonError::InvalidUuid(format!("{}: {}", s, e)))?;
    Ok(Value::Uuid(u))
}

fn decode_tagged(tag_v: &JV, body_v: &JV) -> JsonResult<OwnedValue> {
    let tag_s = tag_v
        .as_str()
        .ok_or_else(|| JsonError::InvalidTag(format!("#tag must be string: {}", tag_v)))?;
    let (ns, name) = split_namespaced(tag_s)
        .ok_or_else(|| JsonError::InvalidTag(format!("user tag must have namespace: {}", tag_s)))?;
    let tag = Tag::try_ns(ns, name)
        .map_err(|m| JsonError::InvalidTag(format!("{}: {}", tag_s, m)))?;
    let body = json_to_edn(body_v)?;
    Ok(Value::Tagged(tag, Box::new(body)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse, write};

    /// Parse EDN, convert to JSON, parse JSON back, convert to EDN,
    /// write EDN — assert byte-equivalence at every step.
    fn round_trip(edn: &str) {
        let v1 = parse(edn).unwrap().into_owned();
        let json = to_json_string(&v1);
        let v2 = from_json_string(&json).unwrap();
        assert_eq!(v1, v2, "edn → json → edn for {}\n  json was: {}", edn, json);
        let edn2 = write(&v2);
        let v3 = parse(&edn2).unwrap().into_owned();
        assert_eq!(v1, v3, "edn → json → edn → write → parse for {}", edn);
    }

    #[test]
    fn primitives() {
        round_trip("nil");
        round_trip("true");
        round_trip("false");
        round_trip("42");
        round_trip("-7");
        round_trip("3.14");
        round_trip(r#""hello""#);
    }

    #[test]
    fn keywords_round_trip_via_colon_prefix() {
        round_trip(":foo");
        round_trip(":ns/foo");
    }

    #[test]
    fn symbols_round_trip_via_sentinel() {
        round_trip("foo");
        round_trip("ns/foo");
    }

    #[test]
    fn collections() {
        round_trip("[1 2 3]");
        round_trip("[]");
        round_trip("#{1 2 3}");
        round_trip(r#"{:a 1 :b 2}"#);
    }

    #[test]
    fn list_collapses_to_vector_through_json() {
        // EDN distinguishes list `(1 2 3)` from vector `[1 2 3]`; JSON
        // has only arrays. List→JSON loses the list type; round-trip
        // back yields a Vector (ordering preserved). Documented lossy.
        let v_list = parse("(1 2 3)").unwrap().into_owned();
        let json = to_json_string(&v_list);
        let v_back = from_json_string(&json).unwrap();
        assert_eq!(
            v_back,
            Value::Vector(vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
            ])
        );
    }

    #[test]
    fn nested() {
        round_trip(r#"[{:a 1} {:b 2}]"#);
        round_trip(r#"{"key" [1 2 3]}"#);
        round_trip(r#"#{[1 2] [3 4]}"#);
    }

    #[test]
    fn tagged() {
        round_trip(r#"#myapp/Order {:id 1}"#);
        round_trip(r#"#a/B #c/D 42"#);
    }

    #[test]
    fn inst_and_uuid() {
        round_trip(r#"#inst "2026-04-28T16:00:00Z""#);
        round_trip(r#"#uuid "550e8400-e29b-41d4-a716-446655440000""#);
    }

    #[test]
    fn inst_default_is_nine_digits() {
        let v = parse(r#"#inst "2026-04-28T16:00:00Z""#).unwrap();
        assert_eq!(
            to_json_string(&v),
            r##"{"#inst":"2026-04-28T16:00:00.000000000Z"}"##
        );
    }

    #[test]
    fn inst_digits_zero_has_no_fraction() {
        let v = parse(r#"#inst "2026-04-28T16:00:00Z""#).unwrap();
        assert_eq!(
            to_json_string_with(&v, &WriteOpts::from_inst_digits(0)),
            r##"{"#inst":"2026-04-28T16:00:00Z"}"##
        );
    }

    #[test]
    fn inst_digits_clamps_to_0_9() {
        assert_eq!(WriteOpts::from_inst_digits(-1).inst_digits, 0);
        assert_eq!(WriteOpts::from_inst_digits(99).inst_digits, 9);
        let v = parse(r#"#inst "2026-04-28T16:00:00.123456789Z""#).unwrap();
        let lo = to_json_string_with(&v, &WriteOpts::from_inst_digits(-1));
        let hi = to_json_string_with(&v, &WriteOpts::from_inst_digits(99));
        assert_eq!(lo, to_json_string_with(&v, &WriteOpts::from_inst_digits(0)));
        assert_eq!(hi, to_json_string_with(&v, &WriteOpts::from_inst_digits(9)));
    }

    #[test]
    fn bigint_and_bigdec() {
        round_trip("123456789012345678901234567890N");
        round_trip("-3.14M");
    }

    #[test]
    fn nan_and_infinity() {
        let v1 = Value::Float(f64::NAN);
        let json = to_json_string(&v1);
        let v2 = from_json_string(&json).unwrap();
        assert_eq!(v1, v2);

        let v1 = Value::Float(f64::INFINITY);
        let v2 = from_json_string(&to_json_string(&v1)).unwrap();
        assert_eq!(v1, v2);

        let v1 = Value::Float(f64::NEG_INFINITY);
        let v2 = from_json_string(&to_json_string(&v1)).unwrap();
        assert_eq!(v1, v2);
    }

    #[test]
    fn out_of_range_integer_uses_string() {
        let v1 = Value::Integer(i64::MAX);
        let json = to_json_string(&v1);
        // Should serialize as string, not number
        assert!(json.contains('\"'));
    }

    #[test]
    fn map_with_keyword_keys() {
        round_trip(r#"{:asset :BTC, :side :Buy, :size 0.025}"#);
    }

    #[test]
    fn map_with_vector_keys() {
        round_trip(r#"{[1 2] :pair, [3 4] :other}"#);
    }

    #[test]
    fn realistic_blob() {
        round_trip(
            r#"#enterprise.observer.market/TradeSignal
               {:asset       :BTC
                :side        :Buy
                :size        0.025
                :confidence  0.73
                :proposed-at #inst "2026-04-26T14:30:00Z"
                :id          #uuid "550e8400-e29b-41d4-a716-446655440000"}"#,
        );
    }

    #[test]
    fn json_to_edn_with_string_starting_with_colon_is_keyword() {
        // String value ":foo" in JSON deliberately maps to EDN keyword
        // — the colon prefix is the discriminator.
        let v = from_json_string("\":foo\"").unwrap();
        assert!(matches!(v, Value::Keyword(_)));
    }

    #[test]
    fn empty_collections_round_trip() {
        round_trip("[]");
        round_trip("#{}");
        round_trip("{}");
    }

    #[test]
    fn decode_map_key_strict_edn_looking_invalid_returns_error() {
        // An EDN-looking key (starts with ':') that fails to parse must
        // return JsonError::InvalidMapKey, not silently fall through to
        // Value::String. Probe: ":" is an EDN-looking key (colon prefix)
        // that the EDN parser rejects (bare colon is not a valid keyword).
        let json = r#"{":": 1}"#;
        let result = from_json_string(json);
        assert!(
            matches!(result, Err(JsonError::InvalidMapKey { .. })),
            "expected InvalidMapKey, got {:?}",
            result
        );
    }

    #[test]
    fn decode_uuid_rejects_uppercase_via_json_bridge() {
        // The JSON bridge must apply the same canonical-strict check as the
        // EDN path. Uppercase hex (e.g., "550E8400-...") must return
        // JsonError::InvalidUuid — not silently parse via uuid::Uuid::parse_str
        // which accepts both cases.
        let json = "{\"#uuid\": \"550E8400-E29B-41D4-A716-446655440000\"}";
        let result = from_json_string(json);
        assert!(
            matches!(result, Err(JsonError::InvalidUuid(_))),
            "expected InvalidUuid for uppercase UUID via JSON bridge, got {:?}",
            result
        );
    }
}
