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

/// `(:wat::core::List arg1 arg2 ...)`, now also `(:wat::core::List :- [T] arg1 arg2 ...)`
/// (STONE-255.70) → a `:wat::core::List` holding each argument, in order.
///
/// Evaluates each remaining argument and pushes it to the back of a new
/// `LinkedList<Value>`. Zero args (with or without a bracket) → empty list. No arity
/// restriction on the elements (variadic; 0 or more). Arc 220 Stone 220.4.
///
/// STONE-255.70 — `:wat::core::List` moves off ALGEBRA back onto BINDING (an `&[WatAST]`
/// leading param, the same shape `:wat::core::PersistentVector`/`PersistentMap`/`Tuple` use),
/// exactly as those three do: the dispatch call site strips an optional leading `:- [T]` bracket
/// via `crate::check::split_type_param_bracket` — a pure syntactic peel, discarding the declared
/// type — BEFORE evaluating the rest as ordinary call-by-value elements. ALGEBRA's
/// auto-generated AST door evaluated every arg (including the `:-` keyword and the `[T]` bracket
/// itself) as an ordinary value BEFORE this fn ever saw it — the bracket forms were never legal
/// input for List's old, bracket-blind shape (bracket-less `List` was `infer_linked_list_constructor`'s
/// only supported shape until this stone), so no existing bracket-less call is affected: the
/// splice is a no-op (`None` branch) when no `:-` marker is present, same as `PersistentVector`.
///
/// The value door `list_of_value_door` (below) preserves this fn's EXACT prior ALGEBRA-era value
/// door behavior byte-for-byte (`apply`'s substrate fallback, `dispatch_substrate_impl` —
/// `:wat::core::apply` of a bound `:wat::core::List` reference): a `Value` has no AST, so there is
/// no bracket to strip there — this was already true before this stone (the old value door body,
/// carried forward unchanged).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     args… :wat::core::Value an optional leading `(List :- [T])` type declaration,
///   followed by the elements of the new list, in order
/// @ret     :wat::core::List a `List` holding each argument, in order
/// @example (:wat::core::List 1 2 3) #=> (:wat::core::List 1 2 3)
#[wat_intrinsic(":wat::core::List", value = list_of_value_door)]
pub(crate) fn eval_list_ctor(
    args: &[WatAST],
    _list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    let values = match crate::check::split_type_param_bracket(args) {
        Some((_inner, _bspan, rest)) => rest,
        None => args,
    };
    let mut items = std::collections::LinkedList::new();
    for arg in values {
        let v = eval_inner(arg, env, sym)?.value_owned();
        items.push_back(v);
    }
    Ok(Value::wat__core__List(std::sync::Arc::new(items)))
}

/// Arc 255 Stone O-iv-d's original `list_of` body, unchanged, now the explicit value door
/// (`value = list_of_value_door`) `dispatch_substrate_impl` reaches for `apply` of a bound
/// `:wat::core::List` reference — STONE-255.70 carries this forward byte-for-byte (STOP-2: moving
/// the AST door off ALGEBRA must not silently drop `apply`'s substrate fallback for this ctor,
/// the one BINDING sibling among Vector/HashMap/HashSet/PersistentVector/PersistentMap/Tuple that
/// had one before this stone).
fn list_of_value_door(vals: &[Value], _span: &Span) -> Result<Value, EvalBreak> {
    let mut items = std::collections::LinkedList::new();
    for v in vals {
        items.push_back(v.clone());
    }
    Ok(Value::wat__core__List(std::sync::Arc::new(items)))
}
