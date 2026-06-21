//! `:wat::intrinsic::examples` — the iv-b2-a reflection seam (arc 255.1b).
//!
//! Walks the intrinsic registry and returns every registered intrinsic's
//! carried `@example`s to wat as data, so a wat verifier can run them.
//! Mirrors `:wat::stdlib::sources` (io.rs:1454): return plain Vectors, let
//! wat wrap. The wat verifier (`verify-examples`, iv-b2-b) `eval-ast!`s them.
//!
//! Tuple shape per element:
//!   `[fqdn, expr, expected, run, pure, deterministic]`
//! — `fqdn` a keyword, `expr`/`expected` quoted forms (`Value::wat__WatAST`),
//!   `expected` nil for a markerless `@example-norun`, `run`/`pure`/`det` bools.
//!
//! This read satisfies iv-b1's `#[expect(dead_code)]` on
//! `IntrinsicEntry.examples` and `ExampleSubmission` — both removed in
//! `src/intrinsic/mod.rs` this strike.

use std::sync::Arc;

use wat_macros::wat_intrinsic;

use crate::parser::parse_one_with_file;
use crate::span::Span;
use crate::value::{EvalBreak, Environment, RuntimeError, RuntimeErrorKind, SymbolTable, Value};

/// Walk the intrinsic registry and return every registered intrinsic's
/// carried `@example`s as a `Vector` of `[fqdn, expr, expected, run, pure, det]`
/// tuples — the iv-b2-a reflection seam. The wat verifier (`verify-examples`,
/// iv-b2-b) iterates this to run or skip each example.
///
/// Each element is a six-element heterogeneous Vector:
/// - index 0 `fqdn`: the intrinsic's FQDN as a keyword
/// - index 1 `expr`: the example expression, parsed into a quoted form
/// - index 2 `expected`: the `#=>` value, parsed; nil for `@example-norun`
/// - index 3 `run`: bool — true for `@example`, false for `@example-norun`
/// - index 4 `pure`: bool — derived from `is_effectful_op`
/// - index 5 `deterministic`: bool — derived from pure ∧ ∉ NONDETERMINISTIC
///
/// A parse failure of an example string is a loud seam error (acceptable —
/// a malformed example is a real defect; the macro enforced the doc SHAPE,
/// not that `expr` parses as wat).
///
/// @added 1.0.0
/// @ret a Vector of [fqdn, expr, expected, run, pure, deterministic] tuples, one per @example/@example-norun across all registered intrinsics
/// @example-norun (:wat::intrinsic::examples)
#[wat_intrinsic(":wat::intrinsic::examples")]
pub(crate) fn eval_intrinsic_examples(
    env: &Environment,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    // Suppress unused-param warnings for the context tail — zero wat args,
    // but the macro requires the tail and the NativeHandler ABI needs them.
    let _ = (env, sym);

    let mut tuples: Vec<Value> = Vec::new();

    for entry in crate::intrinsic::registry().all_entries() {
        let fqdn_kw = Value::wat__core__keyword(Arc::new(entry.name.to_string()));
        let (pure, det) = crate::runtime::derive_pure_deterministic(entry.name);

        for ex in entry.examples {
            // Parse `expr` into a quoted form — loud on failure.
            let expr_ast = parse_one_with_file(ex.expr, "<intrinsic-example>")
                .map_err(|e| RuntimeError {
                    span: span.clone(),
                    kind: RuntimeErrorKind::MalformedForm {
                        head: ":wat::intrinsic::examples".into(),
                        reason: format!(
                            "intrinsic {} @example expr failed to parse: {:?}",
                            entry.name, e
                        ),
                    },
                })?;
            let expr_q = Value::wat__WatAST(Arc::new(expr_ast));

            // Parse `expected` only when this is a runnable example (`run=true`).
            // For `@example-norun`, the `#=>` text may be pseudo-code (human
            // doc only, not wat syntax) — the verifier skips it, so yield nil.
            // Nil is also yielded for a markerless `@example-norun` (None).
            let expected_q = if ex.run {
                match ex.expected {
                    Some(s) => {
                        let expected_ast =
                            parse_one_with_file(s, "<intrinsic-example-expected>").map_err(
                                |e| RuntimeError {
                                    span: span.clone(),
                                    kind: RuntimeErrorKind::MalformedForm {
                                        head: ":wat::intrinsic::examples".into(),
                                        reason: format!(
                                            "intrinsic {} @example expected failed to parse: {:?}",
                                            entry.name, e
                                        ),
                                    },
                                },
                            )?;
                        Value::wat__WatAST(Arc::new(expected_ast))
                    }
                    None => Value::Unit,
                }
            } else {
                // @example-norun: expected is human-doc pseudo-code, not wat; yield nil.
                Value::Unit
            };

            let tuple = Value::Vec(Arc::new(vec![
                fqdn_kw.clone(),
                expr_q,
                expected_q,
                Value::bool(ex.run),
                Value::bool(pure),
                Value::bool(det),
            ]));
            tuples.push(tuple);
        }
    }

    Ok(Value::Vec(Arc::new(tuples)))
}
