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

/// Encode a `:wat::core::Bytes` into its lowercase hex `:String`.
///
/// `(:wat::core::Bytes::to-hex bs)` → `:String` (arc 063). Lowercase
/// hex, two chars per byte, no separators and no `0x` prefix.
/// Deterministic: the same Bytes always produce the same String.
#[wat_intrinsic(":wat::core::Bytes::to-hex")]
pub(crate) fn eval_bytes_to_hex(
    bs: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::Bytes::to-hex";
    let xs = match eval_inner(bs, env, sym)?.value_owned() {
        Value::Vec(xs) => xs,
        other => {
            return Err(RuntimeError {
                span: bs.span().clone(),
                kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::core::Bytes (Vec<u8>)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            }
            .into());
        }
    };
    let mut out = String::with_capacity(xs.len() * 2);
    for v in xs.iter() {
        let b = match v {
            Value::u8(b) => *b,
            other => {
                return Err(RuntimeError {
                    span: Span::unknown(),
                    kind: RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: "wat::core::Bytes (Vec<u8>)",
                        got: Box::new(ValueSnapshot::of(&other)),
                        // arc 138: no — element from Vec value, not AST
                    },
                }
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

/// Decode a hex `:String` back into `:Option<wat::core::Bytes>`.
///
/// `(:wat::core::Bytes::from-hex s)` → `:Option<wat::core::Bytes>`
/// (arc 063). Mixed case accepted (a-f and A-F both decode); raw hex
/// only (no separators, no `0x` prefix); the empty string round-trips
/// to an empty Bytes.
///
/// Returns `:None` on:
///   - odd input length (can't pair into bytes)
///   - any non-hex character (`[^0-9a-fA-F]`)
///
/// Same `:None`-on-structural-failure posture as arc 056's
/// `from-iso8601` and arc 061's `bytes-vector`.
#[wat_intrinsic(":wat::core::Bytes::from-hex")]
pub(crate) fn eval_bytes_from_hex(
    s_arg: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::Bytes::from-hex";
    let s = match eval_inner(s_arg, env, sym)?.value_owned() {
        Value::String(s) => s,
        other => {
            return Err(RuntimeError {
                span: s_arg.span().clone(),
                kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "String",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            }
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
