//! Arc 109 Stone — the record home's CONSTRUCT role: the aggregate constructors.
//!
//! Split by ROLE, never by declaration FORM (see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-record-home.md`). The five
//! construction verbs/helpers — `struct-new`, `variant`, `aggregate-new`,
//! `construct_aggregate` (the shared constructor tail both `aggregate-new` and
//! `kwargs-construct` reduce to), and `kwargs-construct` — moved verbatim out of
//! `src/runtime.rs` (arc 109 record-home stone). Behaviour is unchanged; only the
//! location moved.
//!
//! `construct_aggregate` stays private: both its callers (`eval_aggregate_new`,
//! `eval_kwargs_construct`) moved into this same file with it — nothing outside
//! this cluster ever called it directly.
//!
//! Siblings: `access.rs` (field reads + predicates), `project.rs` (surface
//! projection), `update.rs` (record->map / assoc / same-data?).

use std::sync::Arc;

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{
    AggregateValue, EnumValue, Environment, EvalBreak, RuntimeError, RuntimeErrorKind,
    SymbolTable, Value,
};

// `eval_inner`/`no_field_names`/`require_encoding_ctx` are genuinely defined in
// `crate::runtime` (not facade re-exports of `crate::value` types — see STOP-2);
// `eval_inner` is the evaluator's own entry point, `no_field_names` is the shared
// empty-names singleton, `require_encoding_ctx` resolves the ambient `EncodingCtx`
// a `HolonRecord` construction needs.
use crate::runtime::{eval_inner, no_field_names, require_encoding_ctx};

// `build_holon_hologram` is `crate::holon::ast::build_holon_hologram`, re-exported
// at `crate::holon` (the `ast` submodule itself is private) — the canonical path,
// not a facade.
use crate::holon::build_holon_hologram;

/// `(:wat::core::struct-new <type-name-keyword> <v1> <v2> ...)` — the
/// internal primitive every auto-generated `<struct>/new` constructor
/// body invokes. Users do not call this directly; they call the
/// per-struct constructor, which expands to a `struct-new` call with
/// the right type name baked in.
///
/// Validates:
/// - First arg is a keyword (the struct's type name).
/// - Remaining args evaluate; their count becomes the field count.
///
/// Emits `Value::Aggregate(nature=Struct)` with the class FQDN and positional fields.
/// Arity vs field-count mismatch is enforced by the type checker at
/// the bare `<struct>` ctor scheme — this primitive trusts the caller.
pub(crate) fn eval_struct_new(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::struct-new";
    if args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: 0,
            },
        )
        .into());
    }
    let type_name = match &args[0] {
        WatAST::Keyword(k, _) => k.clone(),
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!(
                        "first argument must be a keyword literal (the struct's type name); got {}",
                        other.variant_name()
                    ),
                },
            )
            .into());
        }
    };
    let mut fields = Vec::with_capacity(args.len() - 1);
    for arg in &args[1..] {
        fields.push(eval_inner(arg, env, sym)?.value_owned());
    }
    // Arc 293.R2.1 — AggregateValue::struct_ strips leading ':' from type_name.
    let class = type_name.trim_start_matches(':').to_string();
    // Arc 296 G-1 — `struct-new` is a SECOND generic constructor (`eval_aggregate_new`,
    // ~:15772, is the primary one that already guards against an unregistered class). It can
    // reach either a `TypeDef::Aggregate(Nature::Struct)` — registered, so its declared
    // `names_arc()` is available — or a `TypeDef::Newtype`, which declares no field name at
    // all (arc 049: exactly one inner value, referred to positionally, `<Type>/0`, the SAME
    // convention `register_newtype_methods` bakes into the accessor path a few hundred lines
    // above). An unregistered class raises rather than falling back to a positional guess —
    // mirrors `eval_aggregate_new`'s `:15812` guard.
    let type_key = format!(":{}", class);
    let names: Arc<Vec<String>> = match sym.types().and_then(|types| types.get(&type_key)) {
        Some(crate::types::TypeDef::Aggregate(a)) => a.names_arc(),
        Some(crate::types::TypeDef::Newtype(_)) => Arc::new(vec!["0".to_string()]),
        _ => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!("type {} is not a registered struct or newtype", type_key),
                },
            )
            .into());
        }
    };
    Ok(Value::Aggregate(Arc::new(AggregateValue::struct_(
        class, names, fields,
    ))))
}

/// Arc 048 — `(:wat::core::variant <type-path> <variant-name> field1 field2 ...)`
/// — the internal primitive that auto-synthesized tagged-variant
/// constructors invoke. Users do not call this directly; they call
/// `(:Enum::Variant arg1 arg2)` which dispatches to a Function whose
/// body is a single `variant` call with the type path + variant
/// name baked in via keyword literals.
///
/// Unit variants do NOT route through this primitive — they're stored
/// as pre-built `EnumValue`s in `SymbolTable.unit_variants` and
/// returned directly when the bare keyword evaluates.
///
/// Validates:
/// - First arg is a keyword literal (the enum's type path,
///   `:trading::types::PhaseLabel`).
/// - Second arg is a keyword literal (the variant identifier with
///   leading `:`, e.g. `:Valley`). The leading colon is stripped to
///   yield `variant_name = "Valley"`.
/// - Remaining args evaluate; their count becomes the variant's
///   field count. Arity vs declared variant arity is enforced by
///   the type checker at the synthesized constructor scheme.
pub(crate) fn eval_variant(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() < 2 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::core::variant".into(),
                expected: 2,
                got: args.len(),
            },
        )
        .into());
    }
    let type_path = match &args[0] {
        WatAST::Keyword(k, _) => k.clone(),
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::variant".into(),
                    reason: format!(
                        "first argument must be a keyword literal (the enum's type path); got {}",
                        other.variant_name()
                    ),
                },
            )
            .into());
        }
    };
    let variant_name = match &args[1] {
        WatAST::Keyword(k, _) => {
            // Strip the leading `:` — variant_name stores the bare
            // identifier (e.g., "Valley"), not the `:Valley` keyword form.
            k.strip_prefix(':').unwrap_or(k.as_str()).to_string()
        }
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::variant".into(),
                    reason: format!(
                    "second argument must be a keyword literal (the variant identifier); got {}",
                    other.variant_name()
                ),
                },
            )
            .into());
        }
    };
    let mut fields = Vec::with_capacity(args.len() - 2);
    for arg in &args[2..] {
        fields.push(eval_inner(arg, env, sym)?.value_owned());
    }
    // Arc 296 G′ — the generic constructor: names come from the registry, never
    // invented. STOP-2: an unregistered type/variant RAISES, it does not fall back
    // to a positional guess.
    let types = sym.types().ok_or_else(|| RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
        head: ":wat::core::variant".into(),
        reason: "variant construction requires the type registry, but the SymbolTable has no TypeEnv attached (programmer error: this build path didn't go through startup_from_source / freeze)".into()
    }))?;
    let enum_def = match types.get(&type_path) {
        Some(crate::types::TypeDef::Enum(e)) => e,
        _ => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::variant".into(),
                    reason: format!("{} is not a registered enum type", type_path),
                },
            )
            .into())
        }
    };
    let names = match enum_def.variant_names_arc(&variant_name) {
        Some(n) => n,
        None if enum_def
            .variants
            .iter()
            .any(|v| matches!(v, crate::types::EnumVariant::Unit(n) if n == &variant_name)) =>
        {
            no_field_names()
        }
        None => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::variant".into(),
                    reason: format!("enum {} has no variant named {}", type_path, variant_name),
                },
            )
            .into())
        }
    };
    Ok(Value::Enum(Arc::new(EnumValue {
        type_path,
        variant_name,
        names,
        fields,
    })))
}

/// Arc 296 M — `(:Enum::Variant {:field v …})` / `(:Enum::Unit {})`.
///
/// Returns `None` when `head` is not a registered enum variant constructor,
/// so the generic dispatch can proceed. The map NAMES FIELDS; it is never
/// the payload. Positional `(:Enum::Variant v…)` is refused.
///
/// Option/Result native `Value::Option` / `Value::Result` wrappers are a
/// representation mapping of those two TypeDef enums, not a construction
/// grammar exception — field names still come from the declaration.
pub(crate) fn try_eval_enum_map_ctor(
    head: &str,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Option<Result<Value, EvalBreak>> {
    if !head.contains("::") {
        return None;
    }
    let type_path = wat_reader::identifier::path(head);
    let variant_name = wat_reader::identifier::leaf(head);
    let types = sym.types()?;
    let enum_def = match types.get(type_path) {
        Some(crate::types::TypeDef::Enum(e)) => e,
        _ => return None,
    };
    let declared = enum_def.variant_fields(variant_name)?;

    let refuse = || {
        RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: head.into(),
                reason: format!(
                    "positional variant construction is retired; write `({head} {{:field value …}})` \
                     or `({head} {{}})` for a unit variant"
                ),
            },
        )
        .into()
    };

    if args.len() != 1 {
        return Some(Err(refuse()));
    }
    let WatAST::Map(pairs, map_span) = &args[0] else {
        return Some(Err(refuse()));
    };
    let parsed = match crate::match_arm::parse_key_first_pairs(pairs, map_span) {
        Ok(p) => p,
        Err(e) => {
            return Some(Err(RuntimeError::new(
                e.span,
                RuntimeErrorKind::MalformedForm {
                    head: head.into(),
                    reason: e.reason,
                },
            )
            .into()));
        }
    };
    if parsed.len() != declared.len() {
        return Some(Err(RuntimeError::new(
            map_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: head.into(),
                reason: format!(
                    "map ctor has {} key(s), variant {type_path}::{variant_name} declares {}",
                    parsed.len(),
                    declared.len()
                ),
            },
        )
        .into()));
    }
    let mut fields = Vec::with_capacity(declared.len());
    let mut names = Vec::with_capacity(declared.len());
    for (decl_name, _) in declared {
        let Some((_, val_ast)) = parsed.iter().find(|(n, _)| n == decl_name) else {
            return Some(Err(RuntimeError::new(
                map_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: head.into(),
                    reason: format!(
                        "map ctor is missing declared field `:{decl_name}` of {type_path}::{variant_name}"
                    ),
                },
            )
            .into()));
        };
        match eval_inner(val_ast, env, sym) {
            Ok(tv) => {
                names.push(decl_name.clone());
                fields.push(tv.value_owned());
            }
            Err(e) => return Some(Err(e)),
        }
    }
    for (key, _) in &parsed {
        if !declared.iter().any(|(n, _)| n == key) {
            return Some(Err(RuntimeError::new(
                map_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: head.into(),
                    reason: format!(
                        "map-ctor key `:{key}` is not a field of {type_path}::{variant_name}"
                    ),
                },
            )
            .into()));
        }
    }
    Some(Ok(enum_runtime_value(
        type_path,
        variant_name,
        Arc::new(names),
        fields,
    )))
}

fn enum_runtime_value(
    type_path: &str,
    variant_name: &str,
    names: Arc<Vec<String>>,
    fields: Vec<Value>,
) -> Value {
    // Native representations: Option/Result are TypeDef enums whose values
    // are `Value::Option` / `Value::Result`, not `Value::Enum`. Variant
    // identity is the declaration; field names still come from TypeDef.
    if type_path == ":wat::core::Option" {
        return Value::Option(Arc::new(if fields.is_empty() {
            None
        } else {
            Some(fields.into_iter().next().expect("Some has one field"))
        }));
    }
    if type_path == ":wat::core::Result" {
        return Value::Result(Arc::new(if variant_name == "Err" {
            Err(fields.into_iter().next().expect("Err has one field"))
        } else {
            Ok(fields.into_iter().next().expect("Ok has one field"))
        }));
    }
    Value::Enum(Arc::new(EnumValue {
        type_path: type_path.to_string(),
        variant_name: variant_name.to_string(),
        names,
        fields,
    }))
}

/// Arc 294.c.2a — `(:wat::core::aggregate-new :T field…)`.
///
/// The ONE nature-dispatched aggregate constructor. Looks up `:T`'s `AggregateDef`
/// in the TypeEnv, reads `a.nature` + field names, validates arity, evaluates each
/// field expression, then builds the appropriate `AggregateValue`:
///   Struct      → `AggregateValue::struct_(class, fields)`
///   Record      → `AggregateValue::record(class, Arc::new(fields))`
///   HolonRecord → `AggregateValue::holon_record(class, Arc::new(fields), hologram)`
///                  where `hologram` is derived by `build_holon_hologram` (internal,
///                  no precomputed arg).
///
/// `struct-new` stays registered as a legacy ctor. `Record::of` / `holon::Record::of`
/// were the other two legacy ctors this comment used to name; arc 296 G-1b deleted both
/// (finish the kill, arc 294.c.2a — zero/one live callers, both superseded by this fn).
/// This is intentional runtime-only dispatch — no check-side scheme registered (mirrors
/// `:wat::core::struct-new`; the checker's fresh-TypeVar fallthrough handles callers silently).
pub(crate) fn eval_aggregate_new(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::aggregate-new";
    if args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: 0,
            },
        )
        .into());
    }

    // arg[0]: the type keyword `:T` (literal keyword, not evaluated).
    let type_name = match &args[0] {
        WatAST::Keyword(k, _) => k.clone(),
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!(
                    "first argument must be a keyword literal (the aggregate's type name); got {}",
                    other.variant_name()
                ),
                },
            )
            .into());
        }
    };
    // args[1..] are the positional field values in declared order.
    construct_aggregate(&type_name, &args[1..], OP, list_span, env, sym)
}

/// Shared aggregate-constructor tail — evaluates `value_asts` in declared order and
/// builds the nature-appropriate `AggregateValue` for the type named by `type_name`
/// (a keyword like `:ns::T`). Single-sourced across the two arms: `eval_aggregate_new`
/// (positional / prime path) and `eval_kwargs_construct` (after the kwargs reorder).
/// Arc 294 item (C).
fn construct_aggregate(
    type_name: &str,
    value_asts: &[WatAST],
    op: &'static str,
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    // Strip leading ':' → colon-free bare class for AggregateValue.class and TypeEnv
    // lookup. The macro-expanded form's `~fqdn` (defrecord/defstruct template,
    // `wat/Record.wat`) splices the raw source keyword verbatim, but `<K,V>` is
    // unexpressible (arc 109 ③'s wall, `src/types.rs:4688`) so it only ever splices a
    // BASE name — `type_name` is used directly, never stripped for a `<...>` suffix
    // (arc 109 "reap the twelve" — measured 1,024,489 calls, 0 type-heads).
    let bare_name: &str = type_name;
    let class = bare_name.trim_start_matches(':').to_string();
    // TypeEnv key has leading ':'.
    let type_key = format!(":{}", class);

    // Evaluate the value ASTs as the field values.
    let mut fields: Vec<Value> = Vec::with_capacity(value_asts.len());
    for arg in value_asts {
        fields.push(eval_inner(arg, env, sym)?.value_owned());
    }

    // Look up the TypeDef in the TypeEnv.
    let types = sym.types().ok_or_else(|| {
        RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: "aggregate construction requires the type registry (startup via freeze)"
                    .into(),
            },
        )
    })?;
    let agg = match types.get(&type_key) {
        Some(crate::types::TypeDef::Aggregate(a)) => a,
        _ => {
            return Err(RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: op.into(),
                    reason: format!(
                        "type {} is not a registered aggregate (struct, record, or holon record)",
                        type_key
                    ),
                },
            )
            .into());
        }
    };

    // Validate arity: fields.len() must equal agg.field_names() count.
    let expected_count = agg.fields.len();
    if fields.len() != expected_count {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: format!("{} (constructing {})", op, type_key),
                expected: expected_count,
                got: fields.len(),
            },
        )
        .into());
    }

    match agg.nature {
        crate::types::Nature::Struct => Ok(Value::Aggregate(Arc::new(AggregateValue::struct_(
            class,
            agg.names_arc(),
            fields,
        )))),
        crate::types::Nature::Record => Ok(Value::Aggregate(Arc::new(AggregateValue::record(
            class,
            agg.names_arc(),
            Arc::new(fields),
        )))),
        crate::types::Nature::HolonRecord => {
            let field_names: Vec<String> = agg.field_names().map(|s| s.to_string()).collect();
            let ctx = require_encoding_ctx(op, sym, list_span)?;
            let hologram = build_holon_hologram(&class, &field_names, &fields, ctx, list_span)?;
            Ok(Value::Aggregate(Arc::new(AggregateValue::holon_record(
                class,
                agg.names_arc(),
                Arc::new(fields),
                hologram,
            ))))
        }
        // Arc 293 S3-Nature-2 — `Peer` is never registered as a `TypeDef::Aggregate` (it is the
        // nature-root for `:nature`-bound surfaces, satisfied by a dialed `Peer'`, not constructed
        // via aggregate-new); exhaustiveness only, unreachable at runtime.
        crate::types::Nature::Peer => unreachable!("TypeDef::Aggregate never carries Nature::Peer"),
    }
}

/// Arc 294 item (C) — `:wat::core::kwargs-construct` is the LIVE kwargs-construction
/// form the defrecord/defstruct companion now emits (replacing the expand-time
/// `kwargs-lower` forward whose baked field-vector is WRONG for a SPLICED record).
///
/// `(:wat::core::kwargs-construct :T :f1 v1 :f2 v2 …)` — arg[0] is the bare `:T`
/// keyword; args[1..] are KWARGS. It resolves `:T`'s declared field order from the
/// (splice-merged, post-register) `sym.types()`, reorders the value-ASTs into that
/// order via the shared `reorder_kwargs_by_field_name`, then constructs exactly like
/// `aggregate-new`. Coverage is free: `eval` already traverses every construction at
/// every depth in every residue, so a spliced ctor resolves wherever eval finds it.
///
/// If args[1..] are POSITIONAL (the prime path / generated code — no leading-keyword
/// kv shape), they pass straight through to positional construction, mirroring
/// `build_insert_fact`'s kwargs-vs-positional test (`matcher.rs`).
pub(crate) fn eval_kwargs_construct(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::kwargs-construct";
    if args.is_empty() {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: 0,
            },
        )
        .into());
    }

    // arg[0]: the type keyword `:T` (literal keyword, not evaluated).
    let type_name = match &args[0] {
        WatAST::Keyword(k, _) => k.clone(),
        other => {
            return Err(RuntimeError::new(
                other.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!(
                    "first argument must be a keyword literal (the aggregate's type name); got {}",
                    other.variant_name()
                ),
                },
            )
            .into());
        }
    };

    let rest = &args[1..];
    // Distinguish kwargs (`:f v :f v …`) from positional values — the SAME test
    // `build_insert_fact` uses (`matcher.rs`): an even count of args whose every
    // slot-0-of-pair is a keyword is kwargs; anything else is positional.
    let is_kwargs = rest.len() >= 2
        && rest.len().is_multiple_of(2)
        && rest
            .iter()
            .step_by(2)
            .all(|a| matches!(a, WatAST::Keyword(_, _)));

    if is_kwargs {
        // Resolve declared field order from the (splice-merged) registry. `<K,V>` is
        // unexpressible (arc 109 ③'s wall, `src/types.rs:4688`), so `type_name` is
        // already the base name; used directly, never stripped (arc 109 "reap the
        // twelve" — measured 982,474 calls, 0 type-heads).
        let bare_name: &str = type_name.as_str();
        let class = bare_name.trim_start_matches(':').to_string();
        let type_key = format!(":{}", class);
        let types = sym.types().ok_or_else(|| {
            RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: "kwargs-construct requires the type registry (startup via freeze)"
                        .into(),
                },
            )
        })?;
        let agg = match types.get(&type_key) {
            Some(crate::types::TypeDef::Aggregate(a)) => a,
            _ => {
                return Err(RuntimeError::new(
                    list_span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: format!(
                        "type {} is not a registered aggregate (struct, record, or holon record)",
                        type_key
                    ),
                    },
                )
                .into());
            }
        };
        let field_order: Vec<&str> = agg.field_names().collect();
        // Build (field-name-without-colon, value-AST) pairs.
        let mut kv: Vec<(&str, WatAST)> = Vec::with_capacity(rest.len() / 2);
        for pair in rest.chunks(2) {
            let fname = match &pair[0] {
                WatAST::Keyword(k, _) => k.strip_prefix(':').unwrap_or(k.as_str()),
                // is_kwargs guarantees every slot-0-of-pair is a keyword.
                _ => unreachable!("is_kwargs guarantees a keyword at each kv key"),
            };
            kv.push((fname, pair[1].clone()));
        }
        let reordered = crate::rete::validate::reorder_kwargs_by_field_name(&field_order, &kv, list_span)
            .map_err(|bad| {
                RuntimeError::new(
                    bad.span,
                    RuntimeErrorKind::MalformedForm {
                        head: OP.into(),
                        reason: format!(
                            "unknown field :{} for aggregate {} (declared fields: {})",
                            bad.field,
                            type_key,
                            field_order.join(", ")
                        ),
                    },
                )
            })?;
        construct_aggregate(&type_name, &reordered, OP, list_span, env, sym)
    } else if rest.len() <= 1 {
        // Zero- or single-arg construction — baseline `kwargs-lower`'s "is-pt" passthrough:
        // a lone value/record (or empty) flows straight to positional construction, so a
        // wrong-arity/wrong-type single arg produces the proper located error (NOT a bare-
        // positional rejection). `(:T v)`, `(:T)`, `(:T some-record)`.
        construct_aggregate(&type_name, rest, OP, list_span, env, sym)
    } else {
        // Arc 294 item 9a — the bare aggregate name is the KWARGS macro; raw positional
        // construction is RETIRED (positional belongs to the prime `:T'`, which routes
        // through `aggregate-new` directly, NOT through this form). Reaching here means a
        // user wrote `(:T v1 v2 …)` at the bare name — a LOCATED rejection, preserving the
        // flip doctrine (kwargs everywhere a human writes; the prime for generated code).
        //
        // Arc 198 strike 2 (BRIEF-198-companion-propagation-A1-B2) — A1 gates `T'` behind
        // T's own `:restricted-to` whitelist (it no longer bypasses it), so unconditionally
        // offering "or use the positional prime `T'`" would walk a non-whitelisted caller
        // straight into that wall. Look the type up and drop the offer when it is restricted.
        // `<K,V>` is unexpressible (arc 109 ③'s wall, `src/types.rs:4688`), so `type_name`
        // is already the base name; used directly, never stripped (arc 109 "reap the
        // twelve" — this positional-rejection arm was measured 0 calls over a full floor
        // run: no corpus `.wat` file currently reaches the retired bare-positional spelling
        // this branch rejects).
        let bare_name: &str = type_name.as_str();
        let class = bare_name.trim_start_matches(':').to_string();
        let type_key = format!(":{}", class);
        let is_restricted = sym.types()
            .and_then(|types| types.get(&type_key))
            .is_some_and(|def| matches!(def, crate::types::TypeDef::Aggregate(a) if a.restrictions.is_some()));
        let reason = if is_restricted {
            format!(
                "bare-positional construction of {} is retired (the bare name is the kwargs \
                 macro); write kwargs `({} :field value …)` from a caller in its `:restricted-to` \
                 whitelist — the positional prime `{}'` is gated to that SAME whitelist (arc 198 \
                 strike 2), not an unrestricted alternative",
                type_name, type_name, type_name
            )
        } else {
            format!(
                "bare-positional construction of {} is retired (the bare name is the kwargs \
                 macro); write kwargs `({} :field value …)` or use the positional prime `{}'`",
                type_name, type_name, type_name
            )
        };
        Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason,
            },
        )
        .into())
    }
}
