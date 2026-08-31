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

use crate::value::{EvalBreak, Value};

/// `(:wat::core::List arg1 arg2 ...)` → a `:wat::core::List` holding each
/// argument, in order.
///
/// Evaluates each argument and pushes it to the back of a new
/// `LinkedList<Value>`. Zero args → empty list. No arity restriction
/// (variadic; 0 or more). Arc 220 Stone 220.4.
///
/// Arc 255 Stone O-iv-d — migrated to ALGEBRA (the first *variadic* ALGEBRA verb in the
/// arc): args arrive already evaluated as `&[Value]`; this handler only clones each into the
/// new list. One declaration now feeds both doors — the AST door (built by the macro) still
/// evaluates each argument itself, then reuses this same value-door body.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Transform
/// @arg     vals… :wat::core::Value the elements of the new list, in order
/// @ret     :wat::core::List a `List` holding each argument, in order
/// @example (:wat::core::List 1 2 3) #=> (:wat::core::List 1 2 3)
#[wat_intrinsic(":wat::core::List")]
pub(crate) fn list_of(vals: &[Value]) -> Result<Value, EvalBreak> {
    let mut items = std::collections::LinkedList::new();
    for v in vals {
        items.push_back(v.clone());
    }
    Ok(Value::wat__core__List(std::sync::Arc::new(items)))
}
