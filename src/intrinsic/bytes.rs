//! core::Bytes intrinsics — arc 255 first home, carved to the
//! `#[wat_intrinsic]` fixed-arg form (255.1b-ii).
//!
//! The two Bytes ↔ hex bridges (arc 063) live here as fixed-arg handlers:
//! each takes its single wat arg as a typed `&WatAST` param plus the
//! `env/sym/span` context tail. The `#[wat_intrinsic("<fqdn>")]` attribute
//! sniffs the arity (Exact-1), emits the arity-checking `NativeHandler`
//! shim, and `inventory::submit!`s the (fqdn → shim) into the registry —
//! no explicit `register()` call. This module stays `mod`-declared in
//! `intrinsic/mod.rs` so its submissions are linked.
//!
//! ## Text bridge for `:wat::core::Bytes` (arc 063)
//!
//! The substrate's hermetic stdout/stdin (and any future log-file or
//! string-field channel) is `:wat::core::Vector<String>` — raw `:Bytes`
//! doesn't ride that without an encoding. Hex is the universally-readable
//! choice: 1:2 byte-to-char, trivially encodable, debuggable in dumps.
//! Base64 / base32 ship later under the same `:wat::core::Bytes::to-X` /
//! `from-X` shape if a consumer surfaces.

use std::sync::Arc;

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::runtime::eval_inner;
use crate::span::Span;
use crate::value::{
    Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot,
};

/// Encode a `:wat::core::Bytes` into its lowercase-hex `:String`.
///
/// Markdown prose, GFM — flows straight to the wiki page body.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     bs :wat::core::Bytes the bytes to encode
/// @ret     :wat::core::String the lowercase hex string, two chars per byte, no separators
/// @example (:wat::core::Bytes::to-hex (:wat::core::Vector :u8 (:wat::core::u8 255) (:wat::core::u8 0) (:wat::core::u8 16))) #=> "ff0010"
/// @see     :wat::core::Bytes::from-hex
#[wat_intrinsic(":wat::core::Bytes::to-hex")]
pub(crate) fn eval_bytes_to_hex(
    bs: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,

) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::Bytes::to-hex";
    let xs = match eval_inner(bs, env, sym)?.value_owned() {
        Value::Vec(xs) => xs,
        other => {
            return Err(RuntimeError::new(bs.span().clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::core::Bytes, i.e. (Vector :- [u8])",
                    got: Box::new(ValueSnapshot::of(&other)),
                })
            .into());
        }
    };
    let mut out = String::with_capacity(xs.len() * 2);
    for v in xs.iter() {
        let b = match v {
            Value::u8(b) => *b,
            other => {
                // No per-ELEMENT AST exists (the element came from a Vec value), but the
                // CALL's span does — and "this Bytes::to-hex call got a bad element" is a
                // location the author can act on. A Rust line is not.
                return Err(RuntimeError::new(span.clone(), RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: "wat::core::Bytes, i.e. (Vector :- [u8])",
                        got: Box::new(ValueSnapshot::of(other)),
                    })
                .into());
            }
        };
        // Lowercase hex (matches Rust's hex::encode default + git /
        // file-checksum conventions). Two chars per byte, no padding.
        out.push(NIBBLE[(b >> 4) as usize]);
        out.push(NIBBLE[(b & 0x0f) as usize]);
    }
    Ok(Value::String(Arc::new(out)))
}

/// Lowercase hex digit table — 16 entries, indexed by nibble.
const NIBBLE: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

/// Decode a lowercase-hex `:String` back into `(:wat::core::Option :- [:wat::core::Bytes])`.
///
/// Mixed case accepted (`a-f` and `A-F` both decode); raw hex only (no
/// separators, no `0x` prefix); the empty string round-trips to an empty
/// Bytes. Returns `:None` on odd input length or any non-hex character.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg        s :wat::core::String the hex-encoded string to decode
/// @ret        (:wat::core::Option :- [:wat::core::Bytes]) Some(Bytes) on success, None on malformed input
/// @example    (:wat::core::Bytes::from-hex "gg") #=> :None
/// @example-norun (:wat::core::Bytes::from-hex "ff0010") #=> Some(Bytes[255, 0, 16])
/// @see        :wat::core::Bytes::to-hex
#[wat_intrinsic(":wat::core::Bytes::from-hex")]
pub(crate) fn eval_bytes_from_hex(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: the arg type error locates at `arg_span` (`s.span()`); bad hex is a non-error `Ok(None)`

) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::Bytes::from-hex";
    let arg_span = s.span().clone();
    let s = match eval_inner(s, env, sym)?.value_owned() {
        Value::String(s) => s,
        other => {
            return Err(RuntimeError::new(arg_span, RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "String",
                    got: Box::new(ValueSnapshot::of(&other)),
                })
            .into());
        }
    };
    let bytes_in = s.as_bytes();
    if !bytes_in.len().is_multiple_of(2) {
        return Ok(Value::Option(Arc::new(None)));
    }
    let mut out: Vec<Value> = Vec::with_capacity(bytes_in.len() / 2);
    let mut i = 0;
    while i < bytes_in.len() {
        let hi = match decode_nibble(bytes_in[i]) {
            Some(n) => n,
            None => return Ok(Value::Option(Arc::new(None))),
        };
        let lo = match decode_nibble(bytes_in[i + 1]) {
            Some(n) => n,
            None => return Ok(Value::Option(Arc::new(None))),
        };
        out.push(Value::u8((hi << 4) | lo));
        i += 2;
    }
    Ok(Value::Option(Arc::new(Some(Value::Vec(Arc::new(out))))))
}

/// Decode an ASCII byte to a hex nibble. Accepts `0-9`, `a-f`,
/// `A-F`; everything else returns `None`.
fn decode_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}
