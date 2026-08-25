//! `:wat::core::List` — arc 220 Stone 220.4's typed `List` constructor.
//!
//! A FIFTH family `string_ops.rs` held (unnamed by the builder's amendment,
//! which named string/Uuid/char/regex) — `:wat::core::List` is none of
//! those; it is a generic core-collection constructor that happened to sit
//! in the same file, right after `char/of`, because its doc comment says it
//! "mirrors `eval_char_of`'s pattern". Since `string_ops.rs` ceases to
//! exist, this needed a home too. "Own home, same shape" as `bytes.rs` /
//! `char.rs` / `regex.rs`: self-contained, one verb.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::runtime::eval_inner;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::core::List arg1 arg2 ...)` → a `:wat::core::List` holding each
/// argument, in order.
///
/// Evaluates each argument and pushes it to the back of a new
/// `LinkedList<Value>`. Zero args → empty list. No arity restriction
/// (variadic; 0 or more). Arc 220 Stone 220.4.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Transform
/// @arg     args… :wat::core::Value the elements of the new list, in order
/// @ret     :wat::core::List a `List` holding each argument, in order
/// @example (:wat::core::List 1 2 3) #=> (:wat::core::List 1 2 3)
#[wat_intrinsic(":wat::core::List")]
pub(crate) fn eval_list_of(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — located elsewhere: no own error path; the only errors are `?`-propagated from each per-element eval, each carrying its own arg's span
) -> Result<Value, EvalBreak> {
    let mut items = std::collections::LinkedList::new();
    for arg in args {
        items.push_back(eval_inner(arg, env, sym)?.value_owned());
    }
    Ok(Value::wat__core__List(std::sync::Arc::new(items)))
}
