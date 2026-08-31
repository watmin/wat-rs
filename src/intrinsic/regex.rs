//! `:wat::regex::*` intrinsics — arc 255 carve (builder amendment to
//! home #4 phase 2, the string carve): "own home, same shape" as
//! `intrinsic/bytes.rs`. One verb: `matches?`. Lives in its own namespace
//! (not folded into `:wat::string::*`) since the `regex` crate is its own
//! concern — a wat-rs deployment that didn't want the regex dep could
//! feature-gate this module separately in a future refactor.

use crate::ast::WatAST;
use crate::runtime::eval_inner;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use wat_macros::wat_intrinsic;

/// `(:wat::regex::matches? pattern haystack)` → whether `pattern`
/// matches anywhere in `haystack`.
///
/// Not anchored — use `^...$` inside the pattern for full-string match.
/// Pattern compile failure surfaces as MalformedForm; typical user errors
/// (unbalanced bracket, invalid escape) get the regex crate's own
/// diagnostic.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     pattern  :wat::core::String the regex pattern (not anchored)
/// @arg     haystack :wat::core::String the string searched
/// @ret     :wat::core::bool true iff `pattern` matches anywhere in `haystack`
/// @example (:wat::regex::matches? "wor" "hello world") #=> true
#[wat_intrinsic(":wat::regex::matches?")]
pub(crate) fn eval_regex_matches(
    pattern: &WatAST,
    haystack: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: TypeMismatch locates at the offending arg's own span; a bad pattern locates at `pattern`'s own span too
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::regex::matches?";
    let pattern_val = eval_inner(pattern, env, sym)?.value_owned();
    let pattern_str = match &pattern_val {
        Value::String(s) => s.as_str().to_string(),
        other => {
            return Err(RuntimeError::new(pattern.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "String",
                got: Box::new(ValueSnapshot::of(other)),
            })
            .into());
        }
    };
    let haystack_val = eval_inner(haystack, env, sym)?.value_owned();
    let haystack_str = match &haystack_val {
        Value::String(s) => s.as_str().to_string(),
        other => {
            return Err(RuntimeError::new(haystack.span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "String",
                got: Box::new(ValueSnapshot::of(other)),
            })
            .into());
        }
    };
    let re = ::regex::Regex::new(pattern_str.as_str()).map_err(|e| {
        RuntimeError::new(pattern.span().clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!("invalid regex: {}", e),
        })
    })?;
    Ok(Value::bool(re.is_match(haystack_str.as_str())))
}
