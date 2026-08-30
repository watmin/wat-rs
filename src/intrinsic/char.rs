//! `:wat::core::char` intrinsics — arc 255 carve (builder amendment to
//! home #4 phase 2, the string carve): "own home, same shape" as
//! `intrinsic/bytes.rs`. One verb: arc 220 slice 2's `char` constructor
//! (renamed from `char/of` in this arc's four-homes stone; before that,
//! from `Char/of` in stone 242.1, scalar types lowercase per Doctrine 2).

use crate::ast::WatAST;
use crate::runtime::eval_inner;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use wat_macros::wat_intrinsic;

/// `(:wat::core::char s)` → the single `:wat::core::char` in the length-1
/// String `s`.
///
/// BMP-only: codepoints above U+FFFF (supplementary-plane) are rejected
/// with a clear diagnostic, inheriting the Stone 218.6b discipline from
/// wat-edn's BMP-only strictness. Errors: `s` is not length-1 (empty or
/// multi-char), or its single char is a supplementary-plane codepoint. Arc
/// 220 slice 2.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     s :wat::core::String a length-1 BMP string
/// @ret     :wat::core::char the single character in `s`
/// @example (:wat::core::char "x") #=> (:wat::core::char "x")
#[wat_intrinsic(":wat::core::char")]
pub(crate) fn eval_char_of(
    s: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: every error (TypeMismatch/MalformedForm) locates at `s`'s own span
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::char";
    let val = eval_inner(s, env, sym)?.value_owned();
    let text = match val {
        Value::String(v) => (*v).clone(),
        other => {
            return Err(RuntimeError::new(s.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::core::String",
                got: Box::new(ValueSnapshot::of(&other)),
            })
            .into());
        }
    };
    let mut chars = text.chars();
    let c = match chars.next() {
        None => {
            return Err(RuntimeError::new(s.span().clone(), RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "expected a length-1 String; got empty string".into(),
            })
            .into());
        }
        Some(c) => c,
    };
    if chars.next().is_some() {
        let len = text.chars().count();
        return Err(RuntimeError::new(s.span().clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!("expected a length-1 String; got length-{} string {:?}", len, text),
        })
        .into());
    }
    if (c as u32) > 0xFFFF {
        return Err(RuntimeError::new(s.span().clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!(
                "supplementary-plane codepoint U+{:X} not supported; \
                 wat::core::char is BMP-only (U+0000-U+FFFF)",
                c as u32
            ),
        })
        .into());
    }
    Ok(Value::wat__core__Char(c))
}
