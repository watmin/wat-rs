//! Arc 109 Stone — the record home's PROJECT role: surface projection.
//!
//! Split by ROLE, never by declaration FORM (see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-record-home.md`). The three
//! projection verbs/helpers — `project_surface_attrs`, `parse_projection_args`
//! (shared by both `to-record` verbs), and `to-record` itself — moved verbatim
//! out of `src/runtime.rs` (arc 109 record-home stone). Behaviour is unchanged;
//! only the location moved.
//!
//! `project_surface_attrs`/`parse_projection_args` are ALSO called directly, by
//! bare name, from `src/intrinsic/holon/atom.rs`'s `:wat::holon::to-record`
//! handler (the PAIR's other half — the `$holon-record` tier) — that call site
//! now points at `crate::record::project::{project_surface_attrs,
//! parse_projection_args}` instead of the `crate::runtime::` facade.
//!
//! Siblings: `construct.rs` (the constructors), `access.rs` (field reads +
//! predicates), `update.rs` (record->map / assoc / same-data?).

use std::sync::Arc;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{
    AggregateValue, Environment, EvalBreak, RuntimeError, RuntimeErrorKind, SymbolTable, Value,
};

// `eval_inner`/`apply_function` are genuinely defined in `crate::runtime` (not
// facade re-exports of `crate::value` types — see STOP-2): `eval_inner` is the
// evaluator's own entry point; `apply_function` applies an already-resolved
// `Function` to already-evaluated args (the registry-resolved surface accessor
// this file calls is not caller-supplied code — see the Purity ground in
// `src/intrinsic/record.rs`'s `to-record` doc).
use crate::runtime::{apply_function, eval_inner};

/// Extract surface S's Field-member attributes off `x_val`, returning the field NAMES
/// (declaration order, from the surface itself — class A: the `SurfaceDef` is already in
/// hand) alongside one `Value` per field, built in the SAME loop so the two can never
/// disagree in length (arc 296 G-1 STOP-3). Reuses the surface-accessor routing (5282
/// pattern): derives `x_val`'s concrete FQDN, then looks up `:<T>/<field>` in `sym` and
/// calls it. Works for any satisfier whose field accessors are registered (Struct, Record,
/// HolonRecord, or a foreign type with extend-type).
pub(crate) fn project_surface_attrs(
    x_val: &Value,
    surface: &crate::types::SurfaceDef,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<(Arc<Vec<String>>, Vec<Value>), EvalBreak> {
    // Mirror of 5282's concrete_type_fqdn derivation.
    let concrete_type_fqdn: String = match x_val {
        Value::Aggregate(a) => format!(":{}", a.class),
        Value::RustOpaque(inner) => inner.type_path.to_string(),
        other_val => format!(":{}", other_val.type_name()),
    };
    let mut field_names = Vec::new();
    let mut field_values = Vec::new();
    for member in &surface.members {
        if let crate::types::SurfaceMember::Field { name: fname, .. } = member {
            // STONE reap-the-angle-machinery (arc 109) — `method_key` used to be stripped
            // via `canonical_callable_name`; angle syntax is unexpressible now, so neither
            // `concrete_type_fqdn` nor `fname` can carry a suffix — look it up directly.
            let method_key = format!("{}/{}", concrete_type_fqdn, fname);
            let func = match sym.get(&method_key) {
                Some(f) => f.clone(),
                None => {
                    return Err(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::UnknownFunction(format!(
                            "to-record: type `{}` does not have accessor `{}`",
                            concrete_type_fqdn, method_key
                        )),
                    )
                    .into())
                }
            };
            let v = apply_function(func, vec![x_val.clone()], sym, list_span.clone())
                .map_err(EvalBreak::from)?;
            field_names.push(fname.clone());
            field_values.push(v);
        }
    }
    Ok((Arc::new(field_names), field_values))
}

/// Parse the two-arg form `(verb x :S)` for the three projection verbs.
/// Returns `(x_val, surface_name_keyword, surface_def)`.
pub(crate) fn parse_projection_args(
    op: &'static str,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<(Value, String, crate::types::SurfaceDef), EvalBreak> {
    if args.len() != 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: op.into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    // args[1]: literal surface keyword (not evaluated — exactly like aggregate-new's args[0]).
    let surface_kw = match &args[1] {
        WatAST::Keyword(k, _) => k.clone(),
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: op.into(),
                    reason: format!(
                    "second argument must be a surface keyword literal (e.g. :my::Surface); got {}",
                    other.variant_name()
                ),
                },
            )
            .into())
        }
    };
    // Look up the surface in the TypeEnv.
    let types = sym.types().ok_or_else(|| {
        RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: "projection verbs require the type registry (startup via freeze)".into(),
            },
        )
    })?;
    let surf = match types.get(&surface_kw) {
        Some(crate::types::TypeDef::Surface(s)) => s.clone(),
        _ => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: op.into(),
                    reason: format!("{} is not a registered surface", surface_kw),
                },
            )
            .into())
        }
    };
    // Evaluate args[0] (x).
    let x_val = eval_inner(&args[0], env, sym)?.value_owned();
    Ok((x_val, surface_kw, surf))
}

/// Arc 293 K3-revise — `(:wat::core::to-record x :S)` → `:S$core-record` (Record nature).
pub(crate) fn eval_to_core_record(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::to-record";
    let (x_val, surface_kw, surf) = parse_projection_args(OP, args, list_span, env, sym)?;
    let (field_names, field_values) = project_surface_attrs(&x_val, &surf, sym, list_span)?;
    let class = format!("{}$core-record", surface_kw.trim_start_matches(':'));
    Ok(Value::Aggregate(Arc::new(AggregateValue::record(
        class,
        field_names,
        Arc::new(field_values),
    ))))
}
