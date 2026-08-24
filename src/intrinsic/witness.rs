//! Arc 255 spec-complete witnesses — variadic + @yields examples.
//!
//! Two intrinsics that prove the spec-complete capabilities:
//!
//! * `:wat::intrinsic::variadic-args-measurement` — a variadic intrinsic
//!   (single `&[WatAST]` param) that counts its arguments and returns the
//!   count as `:wat::core::i64`. Proves Part A: `Arity::Variadic`, the `…`
//!   grammar, and the variadic shim path.
//!
//! * `:wat::intrinsic::yields-witness` — a minimal HOF that receives a
//!   `Fn(i64)->i64` callback and hands it the value `42`. Proves Part B:
//!   `@yields` grammar, the singleton directive, and the cross-check that
//!   `@yields` type == the fn-arg's Fn param type.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::runtime::eval_inner;
use crate::span::Span;
use crate::value::{EvalBreak, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value};

/// Count the number of arguments passed — a variadic intrinsic witness.
///
/// Accepts zero or more arguments (any type); evaluates none of them.
/// Returns the argument count as `:wat::core::i64`. Pure and deterministic.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      Reflection
/// @arg xs… :wat::core::Value the args to count
/// @ret :wat::core::i64 the number of arguments passed
/// @example (:wat::intrinsic::variadic-args-measurement 1 2 3) #=> 3
/// @example (:wat::intrinsic::variadic-args-measurement) #=> 0
// `@Category Reflection` is CORRECT here and was weighed (2026-08-15): this verb
// reports a property of its own CALL SITE — how many arguments it was handed —
// and never evaluates them. Interrogating the shape of an invocation is the
// program interrogating itself. Contrast `yields-witness`, which was mislabelled
// `Reflection` and is a plain combinator. `//` not `///` — see the note below.
#[wat_intrinsic(":wat::intrinsic::variadic-args-measurement")]
pub(crate) fn eval_variadic_args_measurement(
    xs: &[WatAST],
    _env: &Environment,
    _sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — infallible — no error path (always `Ok(Value::i64(xs.len()))`)
) -> Result<Value, EvalBreak> {
    Ok(Value::i64(xs.len() as i64))
}

/// A minimal higher-order-function witness for `@yields` (arc 255 spec-complete).
///
/// Applies `f` to the constant value `42` and returns `f(42)`. The yielded
/// value is `:wat::core::i64`; `@yields` documents the type handed to `f`.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Category      ControlFlow
/// @arg f [:wat::core::i64 :-> :wat::core::i64] the fn applied to the yielded value
/// @yields :wat::core::i64 the value handed to f (always 42 for this witness)
/// @ret :wat::core::i64 the result of applying f to 42
/// @example (:wat::intrinsic::yields-witness (fn [x] (:wat::core::+ x 1))) #=> 43
// `@Category ControlFlow` (corrected 2026-08-15; was `Reflection`). This body
// applies a callable — it directs evaluation, exactly as `if` selects a branch.
// It introspects nothing, so `Reflection` was a lie. NOT a new `HigherOrder`
// variant: "takes a fn" is a signature property, while `Category` classifies what
// the computation IS; mixing those axes is the error that produced `Ambient`.
//
// ⚠ THIS IS `//`, NOT `///`, ON PURPOSE. The `///` block above is the
// USER-FACING body that `render-doc` prints and the goldens pin. Maintainer
// rationale in `///` ships to users as API documentation — caught by the
// byte-identical goldens on 2026-08-15 when exactly that was tried here.
#[wat_intrinsic(":wat::intrinsic::yields-witness")]
pub(crate) fn eval_yields_witness(
    f: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::intrinsic::yields-witness";
    // Evaluate f to get the callable.
    let callable = eval_inner(f, env, sym)?.value_owned();
    // Extract the Function arc from the callable value.
    let func = match callable {
        Value::wat__core__fn(f) => f,
        other => {
            return Err(RuntimeError::new(span.clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::core::Fn(:wat::core::i64)->:wat::core::i64",
                    got: Box::new(crate::runtime::ValueSnapshot::of(&other)),
                })
            .into());
        }
    };
    // Apply f(42) — the yielded value is always 42 for this witness.
    let yielded = Value::i64(42);
    crate::runtime::apply_function(func, vec![yielded], sym, span.clone())
        .map_err(EvalBreak::from)
}
