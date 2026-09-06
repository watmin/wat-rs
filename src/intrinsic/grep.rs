//! `:wat::grep::canonical-name` — the one seam that folds three head spellings
//! onto the clojure target. Algorithm lives next to `wat_keyword_to_clojure_symbol`.

use std::sync::Arc;

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::runtime::eval_inner;
use crate::span::Span;
use crate::value::{
    Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot,
};

fn arg_string(
    op: &str,
    arg: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Arc<String>, EvalBreak> {
    match eval_inner(arg, env, sym)?.value_owned() {
        Value::String(s) => Ok(s),
        other => Err(RuntimeError::new(
            arg.span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "String",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// `(:wat::grep::canonical-name n)` → the clojure-target spelling of a head name.
///
/// Three arms, one result: FQDN keyword, dotted keyword, clojure symbol.
/// Bare data (`:else`, `<-`, `x`) is identity. The FQDN arm is
/// [`crate::edn::render::wat_keyword_to_clojure_symbol`] — not a second parser.
///
/// **Purity ground:** one string in, one string out; no IO.
/// **Totality ground:** every String maps to a String; TypeMismatch on a non-String.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Transform
/// @arg     n :wat::core::String a head spelling, any of the three flavors
/// @ret     :wat::core::String the clojure-target spelling (`wat.core/defn`)
/// @example (:wat::grep::canonical-name ":wat::core::defn") #=> "wat.core/defn"
/// @example (:wat::grep::canonical-name ":wat.core/defn") #=> "wat.core/defn"
/// @example (:wat::grep::canonical-name "wat.core/defn") #=> "wat.core/defn"
/// @see     :wat::core::ast-name
#[wat_intrinsic(":wat::grep::canonical-name")]
pub(crate) fn eval_canonical_name(
    n: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — TypeMismatch locates at `n`'s own span
) -> Result<Value, EvalBreak> {
    let s = arg_string(":wat::grep::canonical-name", n, env, sym)?;
    Ok(Value::String(Arc::new(
        crate::edn::render::canonical_head_name(s.as_str()),
    )))
}
