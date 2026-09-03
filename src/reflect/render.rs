//! Arc 109 Stone — the reflect home's RENDER role: internal state → AST.
//!
//! Split by ROLE, never by declaration FORM (see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-reflect-home.md`). This file builds
//! `WatAST` signature/define forms from the runtime's own internal representations —
//! `Function`, `TypeScheme`, `MacroDef`, `TypeDef` — the emission half of reflection.
//! `lookup.rs`/`verbs.rs` call into this file to render what they find; this file never
//! calls back into them. Moved verbatim out of `src/runtime.rs` (arc 109 reflect stone).
//! Behaviour is unchanged; only the location moved.
//!
//! `type_expr_to_ast` and `binder_head_nodes` stay private — every caller measured is
//! inside this same file. The other ten items are `pub(crate)`: `eval_struct_to_form`
//! is called from `runtime.rs`'s own `dispatch_keyword_head_value` (a special form with
//! no `#[wat_intrinsic]` entry); `name_from_keyword_or_fn` and the eight
//! `*_to_signature_ast`/`*_to_define_ast` builders are called from `lookup.rs` and
//! `verbs.rs` — a visibility bump forced by the new module boundary, not a signature
//! change.
//!
//! Siblings: `lookup.rs` (find a binding), `verbs.rs` (the `*-of` API surface),
//! `match.rs` (form matching), `expand.rs` (macroexpand).

use crate::ast::WatAST;
use crate::span::Span;
use crate::types::Nature;
use crate::value::{
    Environment, EvalBreak, Function, FunctionBody, RuntimeError, RuntimeErrorKind, SymbolTable,
    Value, ValueSnapshot,
};
use std::sync::Arc;
use wat_macros::wat_special_form_impl;

// `eval_inner` (the evaluator's own entry point) and `value_to_watast` are genuinely defined
// in `crate::runtime` (not a facade re-export of a `crate::value` type — see STOP-2);
// `value_to_watast` sits just above this stone's moved range and stays there.
use crate::runtime::{eval_inner, value_to_watast};

/// Arc 255 Stone 1a-gamma-i — the `role = eval` pointer for `:wat::core::struct->form`.
/// Annotated IN PLACE (unlike `quote`/`forms`, this fn's signature already fits the canonical
/// `NativeHandler` shape, so no thin delegate is needed — see `intrinsic/special/
/// struct_to_form.rs` for the doc-only struct and the `role = check` pointer).
#[wat_special_form_impl(":wat::core::struct->form", role = eval)]
pub(crate) fn eval_struct_to_form(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::struct->form";
    if args.len() != 1 {
        // arc 138: no span — leaf helper without list_span; threading
        // would require touching the entire dispatcher arm chain.
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
    // Arc 293.R2.1 — Aggregate with nature==Struct.
    let s = match v {
        Value::Aggregate(a) if a.nature == Nature::Struct => a,
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "struct value (e.g. the output of bare `:my::Foo` ctor)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    // Build constructor keyword: `:class::Foo'` from class `class::Foo` (colon-free, no /new — arc 293.R2.3).
    // Arc 294 item 9a: this is a GENERATED positional re-construction form — the bare
    // name is now the kwargs companion macro, so generated/machinery code must use the
    // positional PRIME (mirrors the `encode_struct` fix in closure_extract.rs).
    let constructor = format!(":{}'", s.class);
    let span = list_span.clone();
    let mut items = Vec::with_capacity(s.fields.len() + 1);
    items.push(WatAST::Keyword(constructor, span.clone()));
    for f in s.fields.iter() {
        items.push(value_to_watast(OP, f.clone(), span.clone())?);
    }
    Ok(Value::wat__WatAST(Arc::new(WatAST::List(items, span))))
}

// ─── Arc 143 slice 1 — runtime introspection helpers ────────────────────────
//
// Four Rust-internal helpers that reconstruct WatAST from stored runtime
// data. Used by `eval_lookup_define`, `eval_signature_of_defn`, `eval_body_of`.
// None of these are exposed to wat; they are pure implementation detail.

/// Arc 201 slice 1 — render a `TypeExpr` as a STRUCTURED WatAST node
/// for signature emission. Replaces arc 143's `type_expr_to_kw` (which
/// flattened every shape to a single Keyword via `format_type`).
///
/// Emission rules:
/// - `TypeExpr::Path(p)` → `WatAST::Keyword(p)` — `p` already carries
///   its leading `:` (e.g. `:wat::core::i64`); atomic.
/// - `TypeExpr::Parametric { head, args }` →
///   `WatAST::List [ Keyword(":"+head), ...recurse(args) ]` — head is
///   stored WITHOUT the leading colon (e.g. `wat::core::Option`), so
///   we prepend `:` to match keyword convention. Args recurse so nested
///   generics stay structured all the way down.
/// - `TypeExpr::Tuple(args)` →
///   `WatAST::List [ Keyword(":Tuple"), ...recurse(args) ]` — `:Tuple`
///   is the synthetic head marker (no head string is carried in the
///   variant). Empty tuple `:()` lowers to `(:Tuple)`.
/// - `TypeExpr::Fn { args, ret }` →
///   `WatAST::List [ Keyword(":Fn"), ...recurse(args), Symbol("->"),
///                   recurse(ret) ]` — `->` is a Symbol (not a Keyword)
///   so it round-trips through the same shape the existing
///   `extract-arg-names` walker recognises (`HolonAST::Symbol("->")`).
/// - `TypeExpr::Var(id)` → `WatAST::Keyword(":?{id}")` — mirrors
///   `format_type`'s Var spelling; type variables stay atomic.
///
/// Downstream `watast_to_holon` lowers `WatAST::List` → `HolonAST::Bundle`
/// and `WatAST::Keyword(":Foo")` → `HolonAST::Keyword("Foo")` uniformly
/// (arc 221 Stone 221.4b — leading colon stripped by `HolonAST::keyword()`
/// constructor per Stone 221.3; the pre-Stone-221.4b `→ HolonAST::Symbol(":Foo")`
/// convention is retired).
/// The reflection consumer sees `HolonAST::Bundle` for every Parametric / Tuple
/// / Fn type and `HolonAST::Keyword` for every Path / Var keyword. Type-driven
/// macros walk the structure via Bundle accessors; leaf keywords via `as_keyword()`.
///
/// NOTE: this does NOT replace `format_type`. `format_type` remains the
/// canonical source for DIAGNOSTIC / error-message spellings; only
/// SIGNATURE emission paths (the helpers below + `dispatch_to_signature_ast`'s
/// ret-type slot) get the structured form.
fn type_expr_to_ast(ty: &crate::types::TypeExpr) -> WatAST {
    let span = crate::rust_caller_span!();
    match ty {
        // Arc 294.f — the reflection surface emits canonical `wat.type/` type
        // nodes (`WatAST::Symbol` for leaves, `WatAST::List`/`Vector` for
        // Parametric/Tuple/Fn), so the generic WatAST→plain-EDN bridge
        // serializes them faithfully. The pre-294.f rust-scheme
        // `:wat::core::i64` KEYWORD leaked through the bridge as the mangled
        // `:wat.core/i64`; delegating to the canonicalizer produces the
        // reserved `wat.type/i64` symbol instead. Reflection is now ZERO-holon.
        //
        // `type_expr_to_clojure_form` panics on `Var` (it only ever sees
        // parsed-from-source types); `type_scheme_to_signature_ast` genuinely
        // carries `Var` for a generic primitive's params, so handle it here as
        // a bare-symbol type-var (the same shape the canonicalizer's Path
        // type-var — Case 4 — produces), never a panic.
        crate::types::TypeExpr::Var(id) => {
            WatAST::Symbol(crate::scope::Identifier::bare(format!("t{id}")), span)
        }
        other => match crate::edn::render::type_expr_to_clojure_form(other, crate::edn::render::TypeFormHeadMode::Clojure) {
            Ok(node) => node,
            // Unmodeled shape (malformed trailing-`::` path, or a
            // bare/higher-kinded parametric head) — never reachable from a
            // type-checked signature, but fall back to a faithful bare symbol
            // rather than panic (mirrors the canonicalizer's own clean-`Err`
            // discipline).
            Err(_) => WatAST::Symbol(
                crate::scope::Identifier::bare(crate::check::format_type(other)),
                span,
            ),
        },
    }
}

/// STONE-defservice-emits-the-binder (arc 109) — the ONE builder for a reflection
/// signature's HEAD sequence: the bare name keyword, plus (when `type_params` is
/// non-empty) the `:-` marker + a `Vector` of bare type-param symbols as SIBLINGS
/// immediately after it — position 4, live since `69933d362`. Retires the three
/// `format!("{}<{}>", name, type_params.join(","))` call sites that used to splice the
/// retired angle spelling into a single `WatAST::Keyword`, which cannot express a
/// binder at all (a Keyword is an atom; `(Head :- [T U])` is a compound FORM). Callers
/// `.extend()` this into their own `items` vector in place of pushing one Keyword node.
fn binder_head_nodes(head_kw: String, type_params: &[String], span: &Span) -> Vec<WatAST> {
    if type_params.is_empty() {
        vec![WatAST::Keyword(head_kw, span.clone())]
    } else {
        vec![
            WatAST::Keyword(head_kw, span.clone()),
            WatAST::Keyword(":-".into(), span.clone()),
            WatAST::Vector(
                type_params
                    .iter()
                    .map(|t| WatAST::Symbol(crate::scope::Identifier::bare(t.clone()), span.clone()))
                    .collect(),
                span.clone(),
            ),
        ]
    }
}

/// Build `(<name> :- [type_params] (param0 :Type0) (param1 :Type1) ... -> :Ret)`
/// from a user-defined `Function`. This is the signature HEAD as it would
/// appear in a `:wat::core::define` form.
///
/// The name keyword is followed by the `:- [T U]` binder as SIBLINGS when the function
/// is generic (never a re-serialized `:my::fn<T,U>`). Each parameter pair is a
/// two-element list `(param-name :Type)`. The `->` arrow and return type come last.
pub(crate) fn function_to_signature_ast(f: &Function) -> WatAST {
    let span = crate::rust_caller_span!();
    let head_kw = f.name.clone().unwrap_or_else(|| ":anonymous".into());
    let mut items: Vec<WatAST> = Vec::with_capacity(3 + f.params.len() * 2 + 4);
    items.extend(binder_head_nodes(head_kw, &f.type_params, &span));
    for (param, ty) in f.params.iter().zip(f.param_types.iter()) {
        items.push(WatAST::List(
            vec![
                // Arc 170 — REUSE the binder node; rebuilding it from a name
                // is what HygieneScopeDivergence exists to reject.
                WatAST::Symbol(param.clone(), span.clone()),
                type_expr_to_ast(ty),
            ],
            span.clone(),
        ));
    }
    // Arc 150 — variadic defines render their rest-binder as
    // `& (rest (wat.type/Vector [T]))` between the fixed params and the arrow.
    // Mirrors `macrodef_to_signature_ast`'s shape so reflection
    // consumers see a uniform variadic surface across functions and
    // macros. Strict-arity defines skip this block (`rest_param.is_none()`).
    if let (Some(rest_name), Some(rest_ty)) = (f.rest_param.as_ref(), f.rest_param_type.as_ref()) {
        items.push(WatAST::Symbol(
            crate::scope::Identifier::bare("&"),
            span.clone(),
        ));
        items.push(WatAST::List(
            vec![
                WatAST::Symbol(
                    crate::scope::Identifier::bare(rest_name.clone()),
                    span.clone(),
                ),
                type_expr_to_ast(rest_ty),
            ],
            span.clone(),
        ));
    }
    // Arrow + return type.
    items.push(WatAST::Symbol(
        crate::scope::Identifier::bare("->"),
        span.clone(),
    ));
    items.push(type_expr_to_ast(&f.ret_type));
    WatAST::List(items, span)
}

/// Build the full `(:wat::core::defn <head> <body>)` AST for a
/// user-defined function. The `body` field is the stored WatAST verbatim.
/// Stone 241.16 — head keyword updated from `:wat::core::define` to `:wat::core::defn`.
/// `:wat::core::define` is HARD CUT (eval-time residue completed); reflection
/// now labels user-function declarations with the canonical `:wat::core::defn` head.
pub(crate) fn function_to_define_ast(f: &Function) -> WatAST {
    let head = function_to_signature_ast(f);
    let body = match &f.body {
        FunctionBody::Wat(ast) => (**ast).clone(),
        FunctionBody::Native => unreachable!(
            "native builtin fn-applied — dispatched via the runtime match, not fn-apply"
        ),
    };
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::defn".into(), crate::rust_caller_span!()),
            head,
            body,
        ],
        crate::rust_caller_span!(),
    )
}

/// Build the signature HEAD for a substrate primitive from its `TypeScheme`.
/// Because `TypeScheme` carries no param names, we synthesise `:_a0`,
/// `:_a1`, ... as standin names.
///
/// Shape: `(<name> :- [type_params] (_a0 :Type0) ... -> :Ret)`
pub(crate) fn type_scheme_to_signature_ast(name: &str, scheme: &crate::check::TypeScheme) -> WatAST {
    let span = crate::rust_caller_span!();
    let mut items: Vec<WatAST> = Vec::with_capacity(3 + scheme.params.len() * 2);
    items.extend(binder_head_nodes(name.to_string(), &scheme.type_params, &span));
    for (i, ty) in scheme.params.iter().enumerate() {
        items.push(WatAST::List(
            vec![
                WatAST::Symbol(
                    crate::scope::Identifier::bare(format!("_a{}", i)),
                    span.clone(),
                ),
                type_expr_to_ast(ty),
            ],
            span.clone(),
        ));
    }
    items.push(WatAST::Symbol(
        crate::scope::Identifier::bare("->"),
        span.clone(),
    ));
    items.push(type_expr_to_ast(&scheme.ret));
    WatAST::List(items, span)
}

/// Build the full `(:wat::core::defn <head> <sentinel-body>)` for a
/// substrate primitive. The sentinel body
/// `(:wat::core::__internal/primitive <name>)` is a marker — it is
/// NEVER evaluated; substrate primitives use Rust dispatch, not wat
/// bodies. `body-of` returns `:None` for primitives rather than this
/// sentinel; it is exposed only through `lookup-define`.
/// Stone 241.16 — head keyword updated from `:wat::core::define` to `:wat::core::defn`.
pub(crate) fn primitive_to_define_ast(name: &str, scheme: &crate::check::TypeScheme) -> WatAST {
    let span = crate::rust_caller_span!();
    let head = type_scheme_to_signature_ast(name, scheme);
    let sentinel = WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::__internal/primitive".into(), span.clone()),
            WatAST::Keyword(name.to_string(), span.clone()),
        ],
        span.clone(),
    );
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::defn".into(), span.clone()),
            head,
            sentinel,
        ],
        span,
    )
}

// ─── Arc 144 slice 1 — emission helpers for Macro + Type variants ───────────
//
// Slice 1 extends the arc-143 emission helpers
// (`function_to_signature_ast`, `function_to_define_ast`,
// `type_scheme_to_signature_ast`, `primitive_to_define_ast`) with
// sibling helpers for the two NEW kinds reflection now covers:
// `Macro` and `Type`. Each helper renders a WatAST whose shape matches
// what its declaration form looks like in source:
//
// - `macrodef_to_define_ast` → `(:wat::core::defmacro <head> <template>)`
// - `macrodef_to_signature_ast` → just the head Bundle
// - `typedef_to_define_ast` → `(:wat::core::struct|enum|newtype|typealias :Name ...)`
//   with a sentinel body slot (real field emission is a future arc); type params ride
//   the separate `:- [T...]` binder siblings (`typedef_to_signature_ast`), never in the name
// - `typedef_to_signature_ast` → the bare type head as a single-element Bundle
//
// Honest-sentinel discipline: the substrate doesn't preserve every
// detail (e.g. defmacro's per-param `:AST<T>` type isn't tracked
// separately from the template), so we emit a clearly-marked sentinel
// shape rather than a half-rendered fiction.

/// Build the signature HEAD for a registered defmacro from its
/// `MacroDef`. The substrate doesn't track per-parameter type
/// annotations (they're lost after defmacro registration), so every
/// param gets the honest sentinel `:wat::WatAST` — the param IS an
/// AST; the specific shape isn't tracked. (An earlier design tracked
/// this as a fictional `:AST<T>` sentinel; STONE-close-the-last-two-
/// channels (arc 109) retired the angle-bracket grammar it was spelled
/// in — the reader refuses it outright — and repointed this to the
/// real, non-parametric `:wat::WatAST` below.)
///
/// Shape: `(<name> (p1 :wat::WatAST) ... [& (rest (:wat::core::Vector :- [:wat::WatAST]))] -> :wat::WatAST)`.
/// Build the canonical argspec Vector `[name <- :AST ... & rest <- :AST]`
/// for a registered defmacro. Stone 241.17 — mirrors the canonical argspec
/// form that `parse_argspec_triples` parses.
pub(crate) fn macrodef_to_signature_ast(def: &crate::macros::MacroDef) -> WatAST {
    let span = crate::rust_caller_span!();
    // STONE-close-the-last-two-channels (arc 109) — `:AST<wat::WatAST>` was a
    // fictional sentinel spelled in the retired angle-bracket grammar; the reader
    // refuses it outright. `:wat::WatAST` is the real, already-registered, NON-parametric
    // type (a form IS its own type — no `<T>` wrapper was ever needed).
    let ast_kw = WatAST::Keyword(":wat::WatAST".into(), span.clone());
    let mut items: Vec<WatAST> = Vec::new();
    for p in def.params.iter() {
        items.push(WatAST::Symbol(
            crate::scope::Identifier::bare(p.clone()),
            span.clone(),
        ));
        items.push(WatAST::Symbol(
            crate::scope::Identifier::bare("<-"),
            span.clone(),
        ));
        items.push(ast_kw.clone());
    }
    if let Some(rest) = &def.rest_param {
        items.push(WatAST::Symbol(
            crate::scope::Identifier::bare("&"),
            span.clone(),
        ));
        items.push(WatAST::Symbol(
            crate::scope::Identifier::bare(rest.clone()),
            span.clone(),
        ));
        items.push(WatAST::Symbol(
            crate::scope::Identifier::bare("<-"),
            span.clone(),
        ));
        // STONE-close-the-last-two-channels — same retirement as `ast_kw` above, for the
        // rest-param's Vector-of-forms type: the real reference spelling is the surviving
        // `:-` binder form, `(:wat::core::Vector :- [:wat::WatAST])`, not the fictional
        // `:AST<Vec<wat::WatAST>>` keyword the reader would refuse to read back.
        items.push(WatAST::List(
            vec![
                WatAST::Keyword(":wat::core::Vector".into(), span.clone()),
                WatAST::Keyword(":-".into(), span.clone()),
                WatAST::Vector(
                    vec![WatAST::Keyword(":wat::WatAST".into(), span.clone())],
                    span.clone(),
                ),
            ],
            span.clone(),
        ));
    }
    WatAST::Vector(items, span)
}

/// Build the full `(:wat::core::defmacro :name [argspec] -> :Ret body)` AST
/// for a registered defmacro. Stone 241.17 — canonical 6-item form.
/// The template is the stored `def.body` WatAST verbatim (the same value
/// the expander uses).
pub(crate) fn macrodef_to_define_ast(def: &crate::macros::MacroDef) -> WatAST {
    let span = crate::rust_caller_span!();
    let argvec = macrodef_to_signature_ast(def);
    let body = def.body.clone();
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::defmacro".into(), span.clone()),
            WatAST::Keyword(def.name.clone(), span.clone()),
            argvec,
            WatAST::Symbol(crate::scope::Identifier::bare("->"), span.clone()),
            // STONE-close-the-last-two-channels — same retirement as `macrodef_to_signature_ast`'s
            // `ast_kw`: the real, non-parametric `:wat::WatAST`, not the fictional `:AST<wat::WatAST>`.
            WatAST::Keyword(":wat::WatAST".into(), span.clone()),
            body,
        ],
        span,
    )
}

/// Build the signature HEAD for a `TypeDef`. Unlike functions and
/// macros, type "signatures" are just the type's name keyword + its
/// optional `:- [T U …]` binder siblings — types declare a name
/// shape, not a callable arity. Wrapping the head in a single-element
/// List keeps the surface uniform with the function/macro helpers
/// (always a List around a head Keyword + zero-or-more sub-forms).
pub(crate) fn typedef_to_signature_ast(def: &crate::types::TypeDef) -> WatAST {
    let span = crate::rust_caller_span!();
    let (base, type_params) = match def {
        // Arc 293.2b — Aggregate: record kind has no type params (emit name only);
        // struct kind may have type params (fall through to normal path).
        crate::types::TypeDef::Aggregate(a) => {
            if a.nature != crate::types::Nature::Struct {
                // Record | HolonRecord — no type params.
                return WatAST::List(vec![WatAST::Keyword(a.name.clone(), span.clone())], span);
            }
            (a.name.clone(), &a.type_params)
        }
        crate::types::TypeDef::Enum(e) => (e.name.clone(), &e.type_params),
        crate::types::TypeDef::Newtype(n) => (n.name.clone(), &n.type_params),
        crate::types::TypeDef::Alias(a) => (a.name.clone(), &a.type_params),
        // Stone 237.1 — typeunion is a type-level grouping; signature is its name.
        crate::types::TypeDef::Union(u) => (u.name.clone(), &u.type_params),
        // Arc 293.3-core — surface signature: name + optional type params.
        crate::types::TypeDef::Surface(s) => (s.name.clone(), &s.type_params),
    };
    let head_nodes = binder_head_nodes(base, type_params, &span);
    WatAST::List(head_nodes, span)
}

/// Build the full declaration form for a `TypeDef`. Slice 1 emits a
/// MINIMAL, honest shape: the correct declaration head keyword
/// (`:wat::core::struct` / `:wat::core::enum` / `:wat::core::newtype` /
/// `:wat::core::typealias`) + the type's name + a sentinel body slot
/// `(:wat::core::__internal/type-decl :Name)` declaring "the real
/// fields/variants/inner/expr aren't rendered yet — readers know the
/// declaration head + the type's name; grep for the actual decl in
/// source." Real field emission is deferred to a future arc; honest
/// sentinel beats a half-rendered struct.
pub(crate) fn typedef_to_define_ast(def: &crate::types::TypeDef) -> WatAST {
    let span = crate::rust_caller_span!();
    let head_kw = match def {
        // Arc 293.2b — Aggregate branches on kind for the correct declaration head.
        crate::types::TypeDef::Aggregate(a) => {
            if a.nature == crate::types::Nature::Struct {
                // Stone 241.8 — defstruct replaces struct.
                ":wat::core::defstruct"
            } else {
                // Stone S-B.1 — record class declaration form.
                ":wat::core::recordtype"
            }
        }
        // Stone 241.9 — defenum replaces enum (HARD CUT).
        crate::types::TypeDef::Enum(_) => ":wat::core::defenum",
        crate::types::TypeDef::Newtype(_) => ":wat::core::newtype",
        crate::types::TypeDef::Alias(_) => ":wat::core::typealias",
        // Stone 237.1 — typeunion is type-only; no runtime artifact.
        crate::types::TypeDef::Union(_) => ":wat::core::typeunion",
        // Arc 293.3-core — structural surface declaration form.
        crate::types::TypeDef::Surface(_) => ":wat::core::defsurface",
    };
    let name_kw = WatAST::Keyword(def.name().to_string(), span.clone());
    let sentinel = WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::__internal/type-decl".into(), span.clone()),
            WatAST::Keyword(def.name().to_string(), span.clone()),
        ],
        span.clone(),
    );
    WatAST::List(
        vec![
            WatAST::Keyword(head_kw.into(), span.clone()),
            name_kw,
            sentinel,
        ],
        span,
    )
}

/// Extract the name string from a value that may be either a bare keyword
/// or a function value (arc 009 "names are values" means keywords that
/// refer to defined functions evaluate to their Function value). Returns
/// `Some(name)` for both; `None` for any other value shape.
pub(crate) fn name_from_keyword_or_fn(v: &Value) -> Option<String> {
    match v {
        Value::wat__core__keyword(k) => Some((**k).clone()),
        Value::wat__core__fn(f) => f.name.clone(),
        _ => None,
    }
}
