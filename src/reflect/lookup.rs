//! Arc 109 Stone — the reflect home's LOOKUP role: find a binding.
//!
//! Split by ROLE, never by declaration FORM (see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-reflect-home.md`). `Binding` is the
//! uniform enum every known wat form (user define, macro, substrate primitive, special
//! form, type) resolves to; `lookup_form` walks the five registries in dispatch order;
//! `eval_lookup_define` is the `:wat::runtime::lookup-define` verb built on top of both.
//! Calls into `render.rs` to build the define AST for whatever it finds. Moved verbatim
//! out of `src/runtime.rs` (arc 109 reflect stone). Behaviour is unchanged; only the
//! location moved.
//!
//! `Binding` and `lookup_form` were already `pub` (the external edge,
//! `src/intrinsic/reflect.rs`, reaches both); `eval_lookup_define` is bumped from private
//! to `pub(crate)` — it carries `#[wat_intrinsic]`, and every such verb living outside
//! `runtime.rs` in this codebase is `pub(crate)` (see `crates/wat-macros/src/
//! wat_intrinsic.rs`'s own doc example).
//!
//! Siblings: `render.rs` (internal state → AST), `verbs.rs` (the `*-of` API surface),
//! `match.rs` (form matching), `expand.rs` (macroexpand).

use crate::ast::WatAST;
use crate::value::{
    Environment, EvalBreak, Function, RuntimeError, RuntimeErrorKind, SymbolTable, Value,
    ValueSnapshot,
};
use holon::HolonAST;
use std::sync::Arc;
use wat_macros::wat_intrinsic;

// `eval_inner` is genuinely defined in `crate::runtime` (not a facade re-export of a
// `crate::value` type — see STOP-2); it is the evaluator's own entry point.
use crate::runtime::eval_inner;

use crate::reflect::render::{
    function_to_define_ast, macrodef_to_define_ast, name_from_keyword_or_fn,
    primitive_to_define_ast, typedef_to_define_ast,
};

// ─── Arc 144 slice 1 — uniform reflection: Binding + lookup_form ────────────
//
// Replaces arc 143 slice 1's `LookupResult` (UserDefine / Primitive only)
// with a uniform `Binding<'a>` enum covering the five known wat form
// kinds. Every variant carries `name` + the form's backing data (or
// derived shape) + a `doc_string: Option<String>` slot reserved for
// arc 141 (always `None` until that arc populates it).
//
// The 'a lifetime ties UserFunction / Macro / Type bindings to their
// SymbolTable-borrowed data; Primitive + SpecialForm own their data
// (TypeScheme + HolonAST respectively).
//
// SpecialForm is the slice-2 territory — slice 1 carries its shape so
// the dispatch is structurally complete; `lookup_form`'s SpecialForm
// path returns None today (no registry to walk yet).

/// Arc 144 slice 1 — uniform reflection binding. Every kind of known
/// wat form (user defines, macros, substrate primitives, special forms,
/// types) produces a `Binding` when looked up. The reflection-layer
/// consumers (`lookup-define`, `signature-of-defn`, `body-of`) dispatch on
/// the variant uniformly — the consumer doesn't case the kinds it
/// cares about; the data flows through one shape.
///
/// Each variant carries `doc_string: Option<String>` as the paved road
/// for arc 141 (docstrings on user defines + macros + substrate
/// primitives + special forms). Always `None` here in slice 1; arc 141
/// populates the `Some` cases as docstring sources arrive.
// ⛔ THESE FIELDS WERE ALWAYS DEAD; THE MOVE ONLY MADE IT VISIBLE.
// At HEAD `Binding` was `pub` inside `pub mod runtime`, so it was publicly reachable and
// rustc does not report unread fields on public API. It now lives in `pub(crate) mod reflect`,
// so dead-code analysis applies for the first time. The enum body is byte-identical across the
// move (diffed) and every reader destructures with `..` — `Binding::Macro { def, .. }`,
// `Binding::Primitive { .. }`, `Binding::SpecialForm { signature, .. }` — so `name` and
// `doc_string` are written at construction and never read.
//
// `#[expect]`, NOT `#[allow]`, per this repo's convention (`src/intrinsic/mod.rs:40`): it is
// silent while the fields are genuinely dead and FIRES the moment one is read, so it cannot rot
// into a stale exemption. Deleting the fields is the real fix and is a SEPARATE stone — this one
// is a relocation whose contract is "bodies move verbatim", and changing a type's shape is not
// that. `[[feedback_an_exemption_is_earned_when_the_alternative_is_worse]]`
#[expect(
    dead_code,
    reason = "arc 109 the-reflect-home: pre-existing dead fields, newly VISIBLE because the move \
              took `Binding` out of a `pub` module into a `pub(crate)` one. Deleting them is a \
              separate stone; this expect fires if any becomes read."
)]
pub enum Binding<'a> {
    UserFunction {
        name: String,
        f: &'a Arc<Function>,
        doc_string: Option<String>,
    },
    Macro {
        name: String,
        def: &'a crate::macros::MacroDef,
        doc_string: Option<String>,
    },
    Primitive {
        name: String,
        scheme: crate::check::TypeScheme,
        doc_string: Option<String>,
    },
    SpecialForm {
        name: String,
        /// Slice 2 will populate this with synthetic signature ASTs at
        /// registration time. Slice 1 carries the shape so the
        /// dispatch arm is structurally present; until slice 2 ships,
        /// the SpecialForm path of `lookup_form` returns `None` and
        /// this variant is unreachable in practice.
        signature: HolonAST,
        doc_string: Option<String>,
    },
    Type {
        name: String,
        def: &'a crate::types::TypeDef,
        doc_string: Option<String>,
    },
}

/// Walk every form-kind registry in dispatch order, returning the
/// first match wrapped in a `Binding`. Lookup precedence mirrors the
/// runtime's call dispatch:
///
/// 1. **User defines** (`sym.functions`) — shadow builtins per
///    call-dispatch precedent.
/// 2. **Macros** (`sym.macro_registry`) — only consulted when the
///    SymbolTable carries a registry (test harnesses sometimes don't).
/// 3. **Substrate primitives** (`CheckEnv::with_builtins()`) — built
///    on demand from the canonical scheme registry.
/// 4. **Types** (`sym.types`) — only consulted when the SymbolTable
///    carries a type registry.
/// 5. **Special forms** — slice 2's territory; returns `None` today.
///
/// Returns `None` only when every registry misses.
pub fn lookup_form<'a>(name: &str, sym: &'a SymbolTable) -> Option<Binding<'a>> {
    // Arc 293.R2.3 — struct/newtype ctors now register at the bare type name.
    // When `name` IS a declared Struct or Newtype type, return the Type binding
    // FIRST so that reflection primitives (lookup-define, body-of, signature-of-defn)
    // see the type definition rather than the synthesized ctor function that lives
    // under the same key in sym.functions.
    if let Some(types) = sym.types() {
        if let Some(def) = types.get(name) {
            let is_auto_ctor = match def {
                crate::types::TypeDef::Aggregate(a) => a.nature == crate::types::Nature::Struct,
                crate::types::TypeDef::Newtype(_) => true,
                _ => false,
            };
            if is_auto_ctor {
                return Some(Binding::Type {
                    name: name.to_string(),
                    def,
                    doc_string: None,
                });
            }
        }
    }
    // 1. User defines shadow builtins (call-dispatch precedent).
    if let Some(f) = sym.get(name) {
        return Some(Binding::UserFunction {
            name: name.to_string(),
            f,
            doc_string: None,
        });
    }
    // 2. Macros — only when a registry is attached.
    if let Some(reg) = sym.macro_registry() {
        if let Some(def) = reg.get(name) {
            return Some(Binding::Macro {
                name: name.to_string(),
                def,
                doc_string: None,
            });
        }
    }
    // 3. Substrate primitives via on-demand CheckEnv.
    // Stone 243.3.1 — with_builtins() removed; caller binds TypeEnv first.
    let _builtin_types = crate::types::TypeEnv::with_builtins();
    let env = crate::check::CheckEnv::with_builtins_and_types(&_builtin_types);
    if let Some(scheme) = env.get(name) {
        return Some(Binding::Primitive {
            name: name.to_string(),
            scheme: scheme.clone(),
            doc_string: None,
        });
    }
    // 4. Types — only when a type registry is attached.
    if let Some(types) = sym.types() {
        if let Some(def) = types.get(name) {
            return Some(Binding::Type {
                name: name.to_string(),
                def,
                doc_string: None,
            });
        }
    }
    // 5. SpecialForm registry — arc 144 slice 2 populated. Cloning
    //    the HolonAST per lookup is acceptable on the reflection-only
    //    path (clone is O(1) — Arc-wrapped recursive payloads).
    if let Some(def) = crate::special_forms::lookup_special_form(name) {
        return Some(Binding::SpecialForm {
            name: def.name.clone(),
            signature: def.signature.clone(),
            doc_string: def.doc_string.clone(),
        });
    }
    None
}

/// `(:wat::runtime::lookup-define name) -> (:wat::core::Option :- [:wat::WatAST])`. The FULL
/// define AST for any known binding, dispatched on the uniform `Binding` enum (`lookup_form`):
/// `UserFunction` reconstructs `(:wat::core::defn <head> <body>)` from the stored `Function`
/// (params/param_types/ret_type/body all preserved); `Primitive` synthesises the same shape from
/// the registered `TypeScheme` with `:_a0`, `:_a1`, ... stand-in param names and the sentinel body
/// `(:wat::core::__internal/primitive <name>)`; `Macro` reconstructs from the stored `MacroDef`
/// template; `Type` reconstructs the type's own declaration form; `SpecialForm` emits the
/// sentinel `(:wat::core::__internal/special-form <name>)` (arc 144 slice 2 populated this
/// registry — `if`/`let`/`match`/… all resolve here, not only substrate-primitive names). An
/// unregistered name returns `:None`.
///
/// Arc 143 slice 1 / Arc 144 slice 1.
///
/// ★ Doc correction (arc 255 Stone P6-c-W3): the prior header (and a matching inline comment on
/// the `SpecialForm` arm below, claiming that arm was "unreachable... until slice 2") both
/// predate slice 2 landing — the registry has been populated since, `tests/wat_lang/
/// wat_arc144_special_forms.rs` exercises it directly, and `(lookup-define :wat::core::if)`
/// returns `Some` today. The prior return-type annotation (`(:Option :- [wat::holon::HolonAST])`)
/// was also stale: arc 201/251/294.f already retired that representation on this whole surface —
/// the wrapped value is a plain `:wat::WatAST` (confirmed against the registered `TypeScheme` for
/// this FQDN, `check.rs`'s `register_builtins`, which returns `Option<:wat::WatAST>`).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Reflection
/// @arg     name_ast :wat::core::keyword the binding name looked up (a literal keyword; a named fn value also resolves via its stored name)
/// @ret     (:wat::core::Option :- [:wat::WatAST]) the FULL define AST for `name_ast`, or `:None` if unregistered
/// @example (:wat::core::match (:wat::runtime::lookup-define :wat::core::if) ((:wat::core::Some _) true) (:wat::core::None false)) #=> true
/// @example (:wat::core::match (:wat::runtime::lookup-define :probe::totally-unknown-xyz) ((:wat::core::Some _) true) (:wat::core::None false)) #=> false
/// @see     :wat::runtime::signature-of-defn
/// @see     :wat::runtime::body-of
#[wat_intrinsic(":wat::runtime::lookup-define")]
pub(crate) fn eval_lookup_define(
    name_ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::runtime::lookup-define";
    // Arc 166 — when the argument is a literal keyword AST, use it
    // directly instead of going through `eval`. Without this, a literal
    // `:user::add` (a defn-bound fn) eval-resolves to the unnamed fn
    // sitting in `runtime_def_values` (eval_fn writes `name: None`
    // because the fn-form itself doesn't carry the def's name), and
    // `name_from_keyword_or_fn` returns None. Reflection on a literal
    // keyword should always resolve to the keyword's name without
    // requiring the runtime value's `name` field to be populated.
    //
    // The eval path remains the fallback for non-literal callers
    // (e.g., `(lookup-define some-var)` where the var holds a Function
    // value with `name: Some(...)` from a sym.functions lookup).
    let name = if let WatAST::Keyword(k, _) = name_ast {
        k.clone()
    } else {
        let v = eval_inner(name_ast, env, sym)?.value_owned();
        match name_from_keyword_or_fn(&v) {
            Some(n) => n,
            None => {
                return Err(RuntimeError::new(
                    name_ast.span().clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: ":wat::core::keyword or named function (e.g. :my::fn)",
                        got: Box::new(ValueSnapshot::of(&v)),
                    },
                )
                .into());
            }
        }
    };
    // Arc 144 slice 1 — dispatch on uniform Binding. Each variant
    // emits its declaration form via the matching helper. SpecialForm is
    // POPULATED (arc 144 slice 2) — reachable today, not a future arm.
    match lookup_form(&name, sym) {
        Some(Binding::UserFunction { f, .. }) => {
            let ast = function_to_define_ast(f);
            Ok(Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(
                ast,
            ))))))
        }
        Some(Binding::Primitive {
            name: n, scheme, ..
        }) => {
            let ast = primitive_to_define_ast(&n, &scheme);
            Ok(Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(
                ast,
            ))))))
        }
        Some(Binding::Macro { def, .. }) => {
            let ast = macrodef_to_define_ast(def);
            Ok(Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(
                ast,
            ))))))
        }
        Some(Binding::Type { def, .. }) => {
            let ast = typedef_to_define_ast(def);
            Ok(Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(
                ast,
            ))))))
        }
        Some(Binding::SpecialForm { name: n, .. }) => {
            // Slice 2 populated the SpecialForm registry (special_forms.rs) —
            // this arm is reachable today for every registered special form
            // (`if`, `let`, `match`, …), not a placeholder for a future slice.
            let span = name_ast.span().clone();
            let sentinel = WatAST::List(
                vec![
                    WatAST::Keyword(":wat::core::__internal/special-form".into(), span.clone()),
                    WatAST::Keyword(n, span.clone()),
                ],
                span,
            );
            Ok(Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(
                sentinel,
            ))))))
        }
        None => Ok(Value::Option(Arc::new(None))),
    }
}
