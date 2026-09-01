//! Arc 109 Stone — the record home's UPDATE role: map extraction, write, and
//! type-blind equality.
//!
//! Split by ROLE, never by declaration FORM (see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-record-home.md`). The five
//! items — `record_field_map` (the shared `record->map` core both `record->map`
//! and `Record/same-data?` reduce to), `record->map`, `Record/same-data?`,
//! `record_assoc_inner` (the shared write tail `Record/assoc` and `assoc`'s
//! Record arm both reduce to), and `Record/assoc` — moved verbatim out of
//! `src/runtime.rs` (arc 109 record-home stone). Behaviour is unchanged; only
//! the location moved.
//!
//! `record_field_map` stays private: both its callers (`eval_record_to_map`,
//! `eval_record_same_data`) moved into this same file with it. `record_assoc_inner`
//! is `pub(crate)` here (a visibility bump forced by the new module boundary, not
//! a signature change): `runtime.rs`'s `eval_assoc` (the polymorphic `assoc`
//! verb's Record arm, out of scope for this stone) calls it directly, by bare
//! name, on already-evaluated args to avoid double evaluation — that call site
//! now reaches across the module boundary.
//!
//! Siblings: `construct.rs` (the constructors), `access.rs` (field reads +
//! predicates), `project.rs` (surface projection).

use std::sync::Arc;

use crate::ast::WatAST;
use crate::span::Span;
use crate::types::Nature;
use crate::value::{
    AggregateValue, Environment, EvalBreak, HolonForm, RuntimeError, RuntimeErrorKind,
    SymbolTable, Value, ValueSnapshot,
};

// `eval_inner`/`values_equal` are genuinely defined in `crate::runtime` (not
// facade re-exports of `crate::value` types — see STOP-2): `eval_inner` is the
// evaluator's own entry point; `values_equal` is arc 238's total map/value
// equality, reused here rather than re-implemented.
use crate::runtime::{eval_inner, values_equal};

// `to_holon_inner` is `crate::holon::ast::to_holon_inner`, re-exported at
// `crate::holon` (the `ast` submodule itself is private) — the canonical path,
// not a facade.
use crate::holon::to_holon_inner;

// `HolonAST` is the external `holon` crate's AST type (not `crate::holon`).
use holon::HolonAST;

/// `(:wat::core::record->map r)` — arc 234 Stone 234.3a.
///
/// Core of `record->map`: given an already-evaluated `Value` that must be a record (base or
/// holonic), returns `Value::wat__std__HashMap` mapping `:<field-name>` keywords to values.
///
/// Called by `eval_record_to_map` (single public path, behavior unchanged) and by
/// `eval_record_same_data` (arc 237 Stone S-C.2d) to avoid code duplication.
///
/// `op` is the calling verb name (used in error messages); `span` is the call-site span.
// Stone 216.5c — suppress `mutable_key_type` for `HashMap<Value, Value>`.
// Value implements Hash + Eq; keywords are stable keys (Arc<String>, no mutation path).
#[allow(clippy::mutable_key_type)]
fn record_field_map(
    v: Value,
    op: &str,
    span: &Span,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    // Arc 293.R2.1 — Aggregate (Record/HolonRecord).
    match v {
        Value::Aggregate(a) if a.nature != Nature::Struct => {
            let type_key = format!(":{}", a.class);
            let types = sym.types().ok_or_else(|| {
                RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: op.into(),
                        reason: "record->map requires the type registry".into(),
                    },
                )
            })?;
            let record_def = match types.get(&type_key) {
                Some(crate::types::TypeDef::Aggregate(agg))
                    if agg.nature != crate::types::Nature::Struct =>
                {
                    agg
                }
                _ => {
                    return Err(RuntimeError::new(
                        span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: op.into(),
                            reason: format!(
                                "record class :{} is not registered in the TypeEnv",
                                a.class
                            ),
                        },
                    )
                    .into());
                }
            };
            let mut map: std::collections::HashMap<Value, Value> =
                std::collections::HashMap::with_capacity(record_def.fields.len());
            for (i, (field_name, _)) in record_def.fields.iter().enumerate() {
                let key = Value::wat__core__keyword(Arc::new(format!(":{}", field_name)));
                let val = a.fields[i].clone();
                map.insert(key, val);
            }
            Ok(Value::wat__std__HashMap(Arc::new(map)))
        }
        other => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::core::Record instance",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// Extracts a `(HashMap :- [:wat::core::keyword value])` from a `Value::Aggregate` (Record/HolonRecord).
/// Field names come from the `AggregateDef` in the TypeEnv; values from `fields` by index
/// (positional match — field i in declaration order corresponds to fields[i]).
///
/// Keys in the returned HashMap are `Value::wat__core__keyword(":<field-name>")`.
///
/// Zero-field record: returns empty HashMap.
/// Non-record input: TypeMismatch error.
pub(crate) fn eval_record_to_map(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::record->map";
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
    record_field_map(v, OP, list_span, sym)
}

/// `(:wat::core::Record/same-data? a b)` — arc 237 Stone S-C.2d.
///
/// Type-BLIND record data equality. Compares the field-name→value maps of two records,
/// ignoring class (type) AND flavor (base vs holonic). The clean complement to `=` (arc 238,
/// which is type-strict): `Pt[x:0,y:0] same-data? Coord[x:0,y:0]` → `true`.
///
/// Semantics (name-keyed, not positional): extracts each record's field map via
/// `record_field_map` (the `record->map` core), then delegates to `values_equal` on the
/// two `HashMap` values — reusing arc 238's total map equality, never re-implementing it.
///
/// Errors:
/// - `ArityMismatch` — not exactly 2 args
/// - `TypeMismatch`  — either arg is not a record
pub(crate) fn eval_record_same_data(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::Record/same-data?";
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
    let a = eval_inner(&args[0], env, sym)?.value_owned();
    let b = eval_inner(&args[1], env, sym)?.value_owned();
    let map_a = record_field_map(a, OP, list_span, sym)?;
    let map_b = record_field_map(b, OP, list_span, sym)?;
    Ok(Value::bool(values_equal(&map_a, &map_b) == Some(true)))
}

/// `(:wat::core::Record/assoc record key new-value)` — arc 234 Stone 234.3b.
///
/// Write verb in the polymorphic record-y family. Returns a NEW `Value::Aggregate` (same nature)
/// with the field named by `key` replaced by `new-value`. The original record is
/// unchanged (immutable; Arc-functional).
///
/// Errors:
/// - `ArityMismatch` — not exactly 3 args
/// - `TypeMismatch`  — first arg is not a record, or second arg is not a keyword
/// - `UnknownField`  — key does not match any field name in the record
/// - `TypeMismatch`  — new value's type variant differs from the original field's type variant
///
/// HolonAST rebuild: clone the outer Bind + its Bundle, replace child at the matching
/// index with `Bind(Atom(String(name)), coerce_to_holon_ast(new_val))`.
/// Value-level inner for record assoc — accepts pre-evaluated values.
/// Called by `eval_record_assoc` (thin wrapper) and by `eval_assoc`'s Record arm
/// (which already evaluated all args to avoid double evaluation).
pub(crate) fn record_assoc_inner(
    record_val: Value,
    key_val: Value,
    new_val: Value,
    list_span: &Span,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::Record/assoc";

    // Arc 293.R2.1 — unified Aggregate path (Record + HolonRecord).
    let agg = match record_val {
        Value::Aggregate(a) if a.nature != Nature::Struct => a,
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

    // Extract the bare field name from the keyword (strip leading colon per D5 / T2).
    let key_name = match key_val {
        Value::wat__core__keyword(k) => {
            let s = k.as_ref().as_str();
            s.strip_prefix(':').unwrap_or(s).to_string()
        }
        other => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::core::keyword field name",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };

    let type_key = format!(":{}", agg.class);
    let types = sym.types().ok_or_else(|| {
        RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "record assoc requires the type registry".into(),
            },
        )
    })?;
    // Arc 293.2b — record aggregates (kind != Struct) replace TypeDef::Record.
    let record_def = match types.get(&type_key) {
        Some(crate::types::TypeDef::Aggregate(a)) if a.nature != crate::types::Nature::Struct => a,
        _ => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!(
                        "record class :{} is not registered in the TypeEnv",
                        agg.class
                    ),
                },
            )
            .into());
        }
    };
    let available: Vec<String> = record_def.field_names().map(|s| s.to_string()).collect();
    let field_index = match record_def
        .field_names()
        .position(|n| n == key_name.as_str())
    {
        Some(i) => i,
        None => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::UnknownField {
                    record_class: agg.class.to_string(),
                    field: key_name,
                    available,
                },
            )
            .into());
        }
    };

    // Type check: new value variant must match original field's variant.
    let old_type = agg.fields[field_index].type_name();
    let new_type = new_val.type_name();
    if old_type != new_type {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: old_type,
                got: Box::new(ValueSnapshot::of(&new_val)),
            },
        )
        .into());
    }

    // Rebuild fields: clone Vec, replace at field_index.
    let mut new_fields: Vec<Value> = (*agg.fields).clone();
    new_fields[field_index] = new_val.clone();
    let new_fields_arc = Arc::new(new_fields);

    // For HolonRecord: also rebuild hologram. For base Record: Empty stays Empty.
    let new_holon = match &agg.holon {
        HolonForm::Empty => HolonForm::Empty,
        HolonForm::Hologram(hologram) => {
            // Hoist field_binds from hologram (PARITY invariant: holonic rebuilds BOTH).
            let field_binds = match hologram.as_ref() {
                HolonAST::Bind(_, right) => match right.as_ref() {
                    HolonAST::Bundle(children) => children.clone(),
                    _ => {
                        return Err(RuntimeError::new(
                            list_span.clone(),
                            RuntimeErrorKind::TypeMismatch {
                                op: OP.into(),
                                expected: "hologram outer Bind right to be Bundle(field-binds)",
                                got: Box::new(ValueSnapshot::unavailable(
                                    "non-Bundle hologram inner",
                                )),
                            },
                        )
                        .into());
                    }
                },
                _ => {
                    return Err(RuntimeError::new(
                        list_span.clone(),
                        RuntimeErrorKind::TypeMismatch {
                            op: OP.into(),
                            expected: "hologram to be Bind(class, Bundle)",
                            got: Box::new(ValueSnapshot::unavailable("non-Bind hologram")),
                        },
                    )
                    .into());
                }
            };

            let new_val_holon = match to_holon_inner(new_val, list_span)? {
                Value::holon__HolonAST(h) => (*h).clone(),
                _ => unreachable!("to_holon_inner always returns holon__HolonAST on Ok"),
            };
            let field_name_holon = HolonAST::Atom(Arc::new(HolonAST::String(Arc::from(
                available[field_index].as_str(),
            ))));
            let new_field_bind = HolonAST::Bind(
                Arc::new(field_name_holon),
                Arc::new(HolonAST::Atom(Arc::new(new_val_holon))),
            );
            let mut new_children: Vec<HolonAST> = (*field_binds).clone();
            new_children[field_index] = new_field_bind;
            let new_bundle = HolonAST::Bundle(Arc::new(new_children));
            let class_atom = match hologram.as_ref() {
                HolonAST::Bind(left, _) => left.clone(),
                _ => unreachable!("guarded above"),
            };
            HolonForm::Hologram(Arc::new(HolonAST::Bind(class_atom, Arc::new(new_bundle))))
        }
    };

    Ok(Value::Aggregate(Arc::new(AggregateValue::from_parts(
        agg.class.clone(),
        agg.names.clone(),
        new_fields_arc,
        agg.nature,
        new_holon,
    ))))
}

/// Thin wrapper: evaluates args then delegates to `record_assoc_inner`.
/// Callers that already have evaluated values (e.g. `eval_assoc`) call `record_assoc_inner` directly.
pub(crate) fn eval_record_assoc(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::Record/assoc";
    if args.len() != 3 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 3,
                got: args.len(),
            },
        )
        .into());
    }
    let record_val = eval_inner(&args[0], env, sym)?.value_owned();
    let key_val = eval_inner(&args[1], env, sym)?.value_owned();
    let new_val = eval_inner(&args[2], env, sym)?.value_owned();
    record_assoc_inner(record_val, key_val, new_val, list_span, sym)
}
