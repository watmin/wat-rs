//! Arc 109 Stone — the record home's ACCESS role: field reads + type predicates.
//!
//! Split by ROLE, never by declaration FORM (see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-record-home.md`). The four
//! read verbs — `struct-field`, `Record/field-at`, `record?`, `List?` — moved
//! verbatim out of `src/runtime.rs` (arc 109 record-home stone). Behaviour is
//! unchanged; only the location moved.
//!
//! `eval_record_q`/`eval_list_q` are `pub(crate)` here (a visibility bump forced
//! by the new module boundary, not a signature change): both were originally called
//! directly, by bare name, from `dispatch_keyword_head_value`'s literal match arm
//! in `runtime.rs`, which reaches across the module boundary to call them. Arc 255 Stone
//! the-seven-that-need-no-extraction homed `eval_record_q` into a `#[wat_intrinsic]` handler
//! (below) — the registry-first check in `dispatch_keyword_head_value` (arc 255.1c-guard)
//! now intercepts `:wat::core::record?` before that match is ever reached, leaving its
//! literal arm there unreachable (left in place — this stone's blast radius is attributes
//! only). `eval_list_q` is untouched and still reached the original way.
//!
//! Siblings: `construct.rs` (the constructors), `project.rs` (surface
//! projection), `update.rs` (record->map / assoc / same-data?).

use crate::ast::WatAST;
use crate::span::Span;
use crate::types::Nature;
use crate::value::{
    Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot,
};

// `eval_inner` is genuinely defined in `crate::runtime` (not a facade re-export of a
// `crate::value` type — see STOP-2); it is the evaluator's own entry point.
use crate::runtime::eval_inner;
use wat_macros::wat_intrinsic;

/// `(:wat::core::struct-field <struct-value> <field-index>)` — the
/// internal primitive every auto-generated `<struct>/<field>` accessor
/// body invokes. Users do not call this directly; they call the
/// per-struct accessor (e.g., `:wat::holon::CapacityExceeded/cost`),
/// which expands to a `struct-field` call with the field's index
/// baked in.
///
/// Validates:
/// - First arg evaluates to a `Value::Aggregate(nature=Struct)`.
/// - Second arg is an integer literal in range `[0, fields.len())`.
///
/// Returns the field value by position. Bounds and type alignment are
/// enforced by the type checker at the `<struct>/<field>` scheme —
/// this primitive trusts the caller for well-typed programs, and
/// raises `MalformedForm` for the ill-typed runtime path.
pub(crate) fn eval_struct_field(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::core::struct-field".into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let struct_val = eval_inner(&args[0], env, sym)?.value_owned();
    // Arc 293.R2.2 — accept ANY Value::Aggregate (unified repr post-R2.1;
    // STOP-3 resolution: the old Nature::Struct guard was a pre-unification
    // artifact; record + holon-record field accessors now use this same
    // primitive via register_aggregate_methods).
    let inner = match struct_val {
        Value::Aggregate(a) => a,
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::core::struct-field".into(),
                    expected: "Aggregate",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let index = match &args[1] {
        WatAST::IntLit(n, _) if *n >= 0 => *n as usize,
        WatAST::IntLit(n, span) => {
            return Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::struct-field".into(),
                    reason: format!("field index must be non-negative; got {}", n),
                },
            )
            .into());
        }
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::struct-field".into(),
                    reason: format!(
                        "second argument must be an integer literal (the field index); got {}",
                        other.variant_name()
                    ),
                },
            )
            .into());
        }
    };
    if index >= inner.fields.len() {
        return Err(RuntimeError::new(
            args[1].span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::struct-field".into(),
                reason: format!(
                    "field index {} out of range for struct {} with {} fields",
                    index,
                    inner.class,
                    inner.fields.len()
                ),
            },
        )
        .into());
    }
    Ok(inner.fields[index].clone())
}

/// `(:wat::core::Record/field-at <record: :wat::core::Record> <index: i64>)` → field value
/// — arc 234 Stone 234.2a.
///
/// Positional accessor for a Record/HolonRecord Aggregate. Returns `fields[index]`.
/// Out-of-bounds index (negative or >= fields.len()) → TypeMismatch error.
/// Consumed by the Stone 234.2b defrecord macro's per-field accessor codegen.
///
/// Arc 255 Stone A-2-ii-b-0 — `pub(crate)` so `src/intrinsic/record.rs`'s thin
/// `#[wat_intrinsic]` delegate can call straight into this unchanged body.
pub(crate) fn eval_record_field_at(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::Record/field-at";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let record_val = eval_inner(&args[0], env, sym)?.value_owned();
    let index_val = eval_inner(&args[1], env, sym)?.value_owned();

    // Arc 293.R2.1 — Aggregate (Record/HolonRecord): positional field store.
    let fields = match record_val {
        Value::Aggregate(a) if a.nature != Nature::Struct => a.fields.clone(),
        other => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::core::Record instance",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };

    let index = match index_val {
        Value::i64(n) => n,
        other => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "i64 positional index",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };

    if index < 0 || (index as usize) >= fields.len() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "i64 index within bounds of fields",
                got: Box::new(ValueSnapshot::described(
                    "i64",
                    format!(
                        "index {} out of bounds (fields.len() = {})",
                        index,
                        fields.len()
                    ),
                )),
            },
        )
        .into());
    }

    Ok(fields[index as usize].clone())
}

/// `(:wat::core::record? v)` — arc 234 Stone 234.3a.
///
/// Polymorphic predicate: true iff `v` is `Value::Aggregate` (Record or HolonRecord nature).
/// Accepts any value (∀T) and returns bool. Mirrors `:wat::core::vector?` / `:wat::core::map?` family.
///
/// **Purity ground —** the sole arg is evaluated by ordinary call-by-value (`eval_inner`, not
/// itself an effect); past that the body only pattern-matches the resulting `Value`'s variant
/// and `Aggregate::nature` — no `eval_inner`/`apply_function` on caller-supplied code beyond
/// the initial evaluation.
///
/// **Totality ground —** `matches!(v, Value::Aggregate(ref a) if a.nature != Nature::Struct)`
/// is a `bool`-valued pattern test over every `Value` variant — every possible `v` (of any
/// type `T`, per the ∀T domain) either matches or doesn't; there is no third outcome and no
/// raise anywhere in the body.
///
/// **Expand-time ground —** NOT on `macros/eval.rs`'s `is_expand_time_legal` residue list
/// today (checked by name — absent from every group), so it is currently REFUSED inside a
/// macro body; there is no existing legality to preserve. Grounded fresh: a pure,
/// deterministic polymorphic predicate that reads no state and performs no effect — safe to
/// evaluate while a `defmacro` body is being expanded, the same shape `:wat::core::length`
/// (also ∀T, also a capability-gated predicate/probe) already declares `Legal` for. Declaring
/// `Legal` here.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Total
/// @ExpandTime    Legal
/// @Category      Probe
/// @arg     args :T the value to classify
/// @ret     :wat::core::bool `true` iff `args` is a `Value::Aggregate` with Record or HolonRecord nature (never `Struct`)
/// @example (:wat::core::do (:wat::core::defrecord :probe::RecordQExample [sk <- :wat::core::i64]) (:wat::core::record? (:probe::RecordQExample :sk 1))) #=> true
#[wat_intrinsic(":wat::core::record?")]
pub(crate) fn eval_record_q(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::record?";
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    // Arc 293.R2.1 — Aggregate with Record or HolonRecord nature is a record.
    Ok(Value::bool(
        matches!(v, Value::Aggregate(ref a) if a.nature != Nature::Struct),
    ))
}

/// `(:wat::core::List? v)` — arc 249 Stone 249.3a.
///
/// Pure-total form-shape predicate: true iff `v` is a `Value::wat__WatAST` wrapping
/// a `WatAST::List`. Used in macro programs that branch on step shape (list step →
/// splice; bare symbol → wrap), enabling threading (`->`/`->>`) as wat code.
///
/// // core form-shape predicate over WatAST::List; distinct from
/// // :wat::holon::is-List? (a classifier over HolonAST). The name
/// // diverges on purpose — the form-vs-holon distinction is the
/// // reason this exists. Do not "harmonize" the two names.
pub(crate) fn eval_list_q(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::List?";
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let v = eval_inner(&args[0], env, sym)?.value_owned();
    Ok(Value::bool(
        matches!(v, Value::wat__WatAST(ref ast) if matches!(&**ast, WatAST::List(..))),
    ))
}
