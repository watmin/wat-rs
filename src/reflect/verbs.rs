//! Arc 109 Stone — the reflect home's VERBS role: the `*-of` API surface.
//!
//! Split by ROLE, never by declaration FORM (see
//! `docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-reflect-home.md`). The
//! `:wat::runtime::*-of` / `rename-callable-name` / `extract-arg-*` verbs — the bulk of
//! the reflection API surface, all built on `lookup.rs`'s `Binding`/`lookup_form` and
//! `render.rs`'s AST builders. Moved verbatim out of `src/runtime.rs` (arc 109 reflect
//! stone). Behaviour is unchanged; only the location moved.
//!
//! `require_ast_children`, `resolve_type_keyword_arg`, and
//! `resolve_aggregate_def_for_reflection` stay private — every caller measured is inside
//! this same file. The other nine items are `pub(crate)`: each carries `#[wat_intrinsic]`,
//! and every such verb living outside `runtime.rs` in this codebase is `pub(crate)` (see
//! `crates/wat-macros/src/wat_intrinsic.rs`'s own doc example) — a visibility bump forced
//! by the new module boundary, not a signature change.
//!
//! ⛔ **STOP-4 finding — `eval_metadata_of` did NOT move.** DESIGN's function list placed
//! it here (line 9281, between `eval_body_of` and `require_bundle`), but its body calls
//! the top-level `intrinsic` module's own registry accessor and constructs that module's
//! `ToEnumValue`/`DefinedIn`/`Layer`/`Arity` values throughout — that registry itself, not
//! this stone's edge (`src/intrinsic/reflect.rs`). Moving it would put a same-crate path to
//! that module into this file, which the acceptance check (a textual grep for that exact
//! path, over `src/reflect/*.rs`, must read 0) forbids categorically — no import shape
//! avoids it, since the grep is textual and a `use` of that path still contains the same
//! text. Per STOP-4, `eval_metadata_of` stays in `runtime.rs`, same as `require_bundle`
//! (STOP-1) — both are DESIGN's-list members proximity put in this range but which do not
//! belong to this concern.
//!
//! Siblings: `render.rs` (internal state → AST), `lookup.rs` (find a binding), `match.rs`
//! (form matching), `expand.rs` (macroexpand).

use crate::ast::WatAST;
use crate::span::Span;
use crate::value::{
    Environment, EvalBreak, FunctionBody, RuntimeError, RuntimeErrorKind, SymbolTable, Value,
    ValueSnapshot,
};
use std::sync::Arc;
use wat_macros::wat_intrinsic;

// `eval_inner` is genuinely defined in `crate::runtime` (not a facade re-export of a
// `crate::value` type — see STOP-2); it is the evaluator's own entry point.
use crate::runtime::eval_inner;

use crate::holon::holon_to_watast;

use crate::reflect::lookup::{lookup_form, Binding};
use crate::reflect::render::{
    function_to_signature_ast, macrodef_to_signature_ast, name_from_keyword_or_fn,
    type_scheme_to_signature_ast, typedef_to_signature_ast,
};

/// `(:wat::runtime::signature-of-defn name) -> (:wat::core::Option :- [:wat::WatAST])`. Returns
/// ONLY the signature HEAD: `(<name> :- [type_params] (param0 :Type0) (param1 :Type1) ... ->
/// :Ret)`. Dispatches on the same uniform `Binding` enum as `lookup-define`: `UserFunction`
/// reconstructs from the `Function`; `Primitive` synthesises from the `TypeScheme`; `Macro`
/// reconstructs from the `MacroDef`; `Type` reconstructs the type's declaration head;
/// `SpecialForm` lowers its pre-built (arc 144 slice 2) signature sketch to `:wat::WatAST`;
/// `Registered` (arc 255 Stone 3a-i) emits the same marker sentinel `lookup-define` does — a
/// registry row carries no synthesized signature shape to lower. An unregistered name returns
/// `:None`.
///
/// Arc 143 slice 1.
///
/// ★ Doc correction (arc 255 Stone P6-c-W3): same stale return-type annotation `eval_lookup_define`
/// carried (`(:Option :- [wat::holon::HolonAST])`) — arc 201/251/294.f retired that representation;
/// the wrapped value is `:wat::WatAST` (confirmed against `check.rs`'s registered `TypeScheme`).
/// The prior prose also named only the UserFunction/Primitive/unknown arms; Macro/Type/SpecialForm
/// are real arms too (see the match below) — completed here.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Reflection
/// @arg     name_ast :wat::core::keyword the binding name whose signature head is reconstructed
/// @ret     (:wat::core::Option :- [:wat::WatAST]) the signature head, or `:None` if unregistered
/// @example (:wat::core::= (:wat::runtime::signature-of-defn :wat::core::if) (:wat::runtime::signature-of-defn :wat::core::if)) #=> true
/// @see     :wat::runtime::lookup-define
/// @see     :wat::runtime::extract-arg-names
#[wat_intrinsic(":wat::runtime::signature-of-defn")]
pub(crate) fn eval_signature_of_defn(
    name_ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::runtime::signature-of-defn";
    // Arc 150 — mirror of the arc 166 pattern in `eval_lookup_define`:
    // when the argument is a literal keyword AST, use the keyword string
    // directly instead of going through `eval`. `eval_inner` on a keyword
    // that is a registered user define resolves to the fn VALUE stored in
    // `runtime_def_values` — which carries `name: None` (eval_fn sets
    // `name: None`; only `sym.functions` has the named version). Using
    // the keyword literal directly avoids the unnamed-fn fallback.
    // The eval path remains correct for non-literal callers (e.g. a
    // variable holding a fn value with `name: Some(...)`).
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
    // Arc 144 slice 1 — dispatch on uniform Binding. SpecialForm
    // returns its pre-built signature directly (slice 2 populated it).
    match lookup_form(&name, sym) {
        Some(Binding::UserFunction { f, .. }) => {
            let ast = function_to_signature_ast(f);
            Ok(Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(
                ast,
            ))))))
        }
        Some(Binding::Primitive {
            name: n, scheme, ..
        }) => {
            let ast = type_scheme_to_signature_ast(&n, &scheme);
            Ok(Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(
                ast,
            ))))))
        }
        Some(Binding::Macro { def, .. }) => {
            let ast = macrodef_to_signature_ast(def);
            Ok(Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(
                ast,
            ))))))
        }
        Some(Binding::Type { def, .. }) => {
            let ast = typedef_to_signature_ast(def);
            Ok(Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(
                ast,
            ))))))
        }
        Some(Binding::SpecialForm { signature, .. }) => {
            // Slice 2 populates SpecialForm.signature with a synthetic
            // HolonAST at registration time; lower it to WatAST so the
            // reflection surface emits plain EDN (arc 294.f) — unlike the
            // other arms whose `*_to_signature_ast` builders already return
            // WatAST, this arm carries a stored HolonAST field.
            Ok(Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(
                holon_to_watast(&signature),
            ))))))
        }
        // ⛔ Arc 255 Stone 3a-i FIX — with a scheme, render EXACTLY what the displaced
        // `Primitive` arm above renders. The registry consult sits ahead of the `CheckEnv`
        // step it displaced, so answering with a bare sentinel here was a measured
        // reflection regression, not a neutral change of head word.
        Some(Binding::Registered {
            name: n,
            scheme: Some(scheme),
            ..
        }) => {
            let ast = type_scheme_to_signature_ast(&n, &scheme);
            Ok(Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(
                ast,
            ))))))
        }
        // Arc 255 Stone 1a-α — a row that DECLARES a grammar answers with that grammar.
        //
        // ⛔ PLACED AFTER THE SCHEME ARM, DELIBERATELY. A `TypeScheme` and an `@syntax`
        // answer DIFFERENT questions — the first is a typed signature, the second a
        // grammar — and where a row has a real typed signature that is the richer answer.
        // Stone 3a-i paid for this exact lesson: an arm hoisted above the scheme step
        // silently downgraded `:wat::vec::length` from a full typed signature to a bare
        // marker. Measured today: ZERO rows carry both (3 declare `@syntax`, all
        // `Kind::SpecialForm`, none registers a scheme), so the order costs nothing now and
        // forecloses that regression when one eventually does. `@syntax` is prose the substrate's own
        // reader can parse (proved by `wat-scripts/scratch-pad/
        // 255-can-the-reader-parse-a-syntax-grammar.wat`), and it is rendered VERBATIM —
        // no FQDN splice — which is `render-doc`'s own precedence
        // (`src/intrinsic/reflect.rs:456-470`), adopted here so the two renderers stop
        // disagreeing. `@syntax` names the form with its short head (`let`, not
        // `:wat.core/let`); re-authoring the string here would mint a third rendering of a
        // question the row already answers.
        //
        // The `.expect` below is load-bearing BECAUSE it is unreachable:
        // `every_registered_syntax_parses` (`src/intrinsic/mod.rs`, this stone) walks every
        // registered `@syntax` through this same parser and turns a malformed one into a red
        // floor at the moment it is authored, not a silent fallback discovered here.
        Some(Binding::Registered { entry, .. }) if !entry.syntax.is_empty() => {
            let ast = wat_reader::parser::parse_one_with_file(
                entry.syntax,
                "<registry @syntax>",
            )
            .expect(
                "every_registered_syntax_parses guarantees every registered @syntax reads clean",
            );
            Ok(Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(
                ast,
            ))))))
        }
        // Arc 255 Stone 3a-i, updated by Stone 1a-α — the registry answers everything it CAN
        // for this verb, and says plainly what it cannot. This arm is reached only when the
        // `@syntax` arm above did NOT fire, i.e. `entry.syntax` is empty: `entry.args` gives
        // the slot names and `is_rest` gives the `+`, so a row with `@arg` (a TYPED slot,
        // e.g. `if`'s three `@arg exprs :wat::core::bool …`) renders its own sketch here.
        // `@syntax` is the correct vehicle for `let`/`fn`/`match` precisely because their
        // slots (`<binder>`, `<params>`, `<scrutinee>`) are syntactic positions with no type
        // to declare — authoring `@arg` for them would mint a type claim these forms don't
        // have, which is why they moved to the arm above instead of into this one.
        //
        // A row with NEITHER `@syntax` nor `@arg` still falls through to the
        // `special_forms.rs` deferral below (arm after this one) — that deferral is what
        // still answers for the 23 rows Stone 1a-α does not touch, until each is registered
        // with its own vehicle.
        Some(Binding::Registered {
            name: n, entry, ..
        }) if !entry.args.is_empty() => {
            // Built through the SAME HolonAST helpers `special_forms.rs`'s `sketch()` uses,
            // then `holon_to_watast` — one shape, not a second hand-rolled one.
            let mut children = Vec::with_capacity(1 + entry.args.len());
            children.push(holon::HolonAST::keyword(&n));
            for (arg_name, _, _, is_rest) in entry.args {
                let slot = if *is_rest {
                    format!("<{arg_name}>+")
                } else {
                    format!("<{arg_name}>")
                };
                children.push(holon::HolonAST::symbol(slot.as_str()));
            }
            let sketch = holon::HolonAST::bundle(children);
            Ok(Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(
                holon_to_watast(&sketch),
            ))))))
        }
        Some(Binding::Registered { name: n, entry, .. })
            if crate::special_forms::lookup_special_form(&n).is_some() =>
        {
            let def = crate::special_forms::lookup_special_form(&n).expect("guard above");
            let _ = entry;
            Ok(Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(
                holon_to_watast(&def.signature),
            ))))))
        }
        Some(Binding::Registered { name: n, entry, .. }) => {
            // ⛔ Arc 255 Stone 3a-i FIX — the sentinel HEAD is derived from `entry.kind`,
            // not fixed to "registered". The registry carries the SpecialForm/Intrinsic
            // distinction the displaced sources encoded in their sentinel words, so answering
            // first must answer with the SAME vocabulary — a registered `if` is still a
            // special-form to reflection. Flattening both to one word would discard a
            // distinction the registry itself holds, which is the opposite of sole authority.
            let head = match entry.kind {
                crate::intrinsic::Kind::SpecialForm => ":wat::core::__internal/special-form",
                _ => ":wat::core::__internal/registered",
            };

            // Arc 255 Stone 3a-i — a registry row with NO scheme (the 89 in GAP_A): no
            // synthesized TypeScheme/HolonAST signature exists for a bare registry row
            // (building one would mean parsing `@arg`/`@ret` doc-text into `TypeExpr`, which
            // is not this stone's job). Marker, never evaluated as a real signature shape.
            let span = name_ast.span().clone();
            let sentinel = WatAST::List(
                vec![
                    WatAST::Keyword(head.into(), span.clone()),
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

/// `(:wat::runtime::signature-of-fn fn-expr) -> :wat::WatAST`. The fn-input sibling of
/// `signature-of-defn` (which takes a NAME keyword and looks up a defined callable in the symbol
/// table). This primitive operates on a FN VALUE — typically the result of evaluating an inline
/// `(:wat::core::fn [...] -> :T body)` form at the call site, or a fn value bound to a local.
///
/// Returns the structured signature AST in the SAME SHAPE `signature-of-defn` returns for named
/// user defines (`function_to_signature_ast`'s output, shared directly — anonymous fn values
/// carry the same `Function` struct as named defines, only `f.name` is `None`, so the signature
/// head spells out as `:anonymous`): `(:anonymous (param0 type0) (param1 type1) ... -> ret-type)`,
/// a `:wat::WatAST` `List`.
///
/// Unlike `signature-of-defn` (which can fail to find a name → `:None`), this primitive's input is
/// a structurally-validated fn value — absence is impossible, so the return is the bare AST, not
/// wrapped in `Option`. Type mismatches at the input slot surface as `RuntimeError::TypeMismatch`.
///
/// Arc 201 slice 3. Used by type-driven macros that receive a coordinator fn as a call-site
/// argument and need to extract per-arg types structurally without symbol-table lookup
/// (originating consumer was arc 170 Stone D2's `run-threads`, since retired — this primitive is
/// shared infra, not run-threads-specific).
///
/// ★ Doc correction (arc 255 Stone P6-c-W3): the prior header said "Return type is
/// `:wat::holon::HolonAST` (NOT `(:Option :- [HolonAST])`)" — the parenthetical about `Option`
/// wrapping was right, but the TYPE NAME was already wrong when it was written: the checker's own
/// registered `TypeScheme` for this FQDN (`check.rs::register_builtins`) has always read
/// `ret: TypeExpr::Path(":wat::WatAST".into())`, and its neighbouring comment even LABELS it
/// "HolonAST" one line above that literal — the same stale terminology, corroborated twice over
/// (this doc and that comment), never the actual registered type.
///
/// **Expand-time ground —** shared infra for type-driven macros to reflect on fn signatures
/// at macro expand time (arc 249 stone 249.2b-i); `fn → HolonAST`, pure — reads from the fn
/// value, no IO. Ruling relocated from `macros/eval.rs`'s expand-time allow-list (arc 255
/// expand-T4a), from its "Runtime reflection" group; the verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
/// @Category      Reflection
/// @arg     fn_expr :wat::core::fn the fn value whose signature is reconstructed
/// @ret     :wat::WatAST the signature head `(:anonymous (param type)... -> ret-type)`
/// @example (:wat::runtime::extract-arg-names (:wat::runtime::signature-of-fn (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64/+ x 1)))) #=> [:x]
/// @see     :wat::runtime::signature-of-defn
/// @see     :wat::runtime::return-type-of
#[wat_intrinsic(":wat::runtime::signature-of-fn")]
pub(crate) fn eval_signature_of_fn(
    fn_expr: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::runtime::signature-of-fn";
    let v = eval_inner(fn_expr, env, sym)?.value_owned();
    let f = match v {
        Value::wat__core__fn(f) => f,
        other => {
            return Err(RuntimeError::new(
                fn_expr.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected:
                        "wat::core::fn value (e.g., from `(:wat::core::fn [...] -> :T body)`)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let ast = function_to_signature_ast(&f);
    Ok(Value::wat__WatAST(Arc::new(ast)))
}

/// `(:wat::runtime::return-type-of <fn-value>) -> :wat::core::String`
///
/// Arc 278. The STATIC sibling of `(:wat::core::type <value>)`: where `type`
/// returns a runtime value's declared type FQDN (colon-free), `return-type-of`
/// returns a fn's DECLARED RETURN type FQDN — colon-free, in the SAME convention,
/// so the two are directly comparable.
///
/// Motivating use (rete `query`): in the types-as-forms surface a bare type name
/// (`:weather::ColdAndWindy`) evaluates to that type's CONSTRUCTOR fn, whose
/// `ret_type` IS the record type. So `return-type-of` on the constructor yields
/// the record's FQDN in ONE step — replacing a multi-step `signature-of-fn` +
/// `extract-arg-types`-style AST walk. General-purpose: works on any fn value.
///
/// Reads `Function.ret_type` directly. Path / Parametric return types yield their (head) FQDN.
/// Tuple / Fn / Var return types have no single nominal name → `RuntimeError::TypeMismatch`. A
/// bare `wat::core::keyword` input (arc 278 query (a) de-mask: a type name whose positional
/// constructor moved to the prime `:T'` evaluates to a plain keyword, not a ctor fn, when the
/// prime is UNDEFINED — a defined prime resolves to its ctor fn and hits the `fn` arm instead)
/// RAISES naming the unknown type, rather than echoing the keyword text back as if it were a
/// resolved type (the pre-arc-278 behaviour, which masked a typo'd prime as a "successful" call).
///
/// ★ Doc correction (arc 255 Stone P6-c-W3): the prior header's "5-step `signature-of-fn`
/// HolonAST Bundle reflection" line named a representation (`HolonAST` `Bundle`) this surface
/// stopped emitting at arc 201/251/294.f — `signature-of-fn` returns a `:wat::WatAST` `List`
/// (see its own doc's correction). Reworded above to avoid repeating the stale name. The prior
/// prose also omitted the `wat__core__keyword` de-mask arm entirely — added above.
///
/// Arc 278.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Reflection
/// @arg     fn_expr :wat::core::fn the fn value whose declared return type is read
/// @ret     :wat::core::String the return type's FQDN, colon-free
/// @example (:wat::runtime::return-type-of (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64/+ x 1))) #=> "wat::core::i64"
/// @see     :wat::runtime::signature-of-fn
#[wat_intrinsic(":wat::runtime::return-type-of")]
pub(crate) fn eval_return_type_of(
    fn_expr: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::runtime::return-type-of";
    let v = eval_inner(fn_expr, env, sym)?.value_owned();
    let f = match v {
        Value::wat__core__fn(f) => f,
        // Arc 278 query (a) de-mask — arc 294 item 9a's construction flip made a bare
        // aggregate type name a MACRO (its positional ctor moved to the prime `:T'`), so a
        // bare type name in VALUE position now evaluates to a KEYWORD, not the ctor fn it
        // used to be. This branch used to ECHO the colon-stripped keyword text back as if it
        // were the resolved type — that masked unknown types: a typo'd `:my::Typo'` echoed
        // its own (wrong) name instead of failing. A DEFINED prime `:T'` resolves to its
        // ctor fn and hits the `fn` branch above, unchanged; an UNDEFINED prime stays a bare
        // keyword and reaches THIS branch — so reaching it IS proof the type is unknown.
        // RAISE instead of echo (rete `query`'s macro-emitted prime is the sole caller —
        // see wat/rete.wat's `query`).
        Value::wat__core__keyword(k) => {
            return Err(RuntimeError::new(
                fn_expr.span().clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason: format!(
                        "unknown type: `{k}` (return-type-of: no such registered type)"
                    ),
                },
            )
            .into());
        }
        other => {
            return Err(RuntimeError::new(
                fn_expr.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::core::fn value (e.g. a record constructor or an inline fn)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    // ret_type FQDN, colon-free to match `(:wat::core::type x)`'s convention.
    let fqdn = match &f.ret_type {
        crate::types::TypeExpr::Path(p) => p.strip_prefix(':').unwrap_or(p).to_string(),
        crate::types::TypeExpr::Parametric { head, .. } => {
            head.strip_prefix(':').unwrap_or(head).to_string()
        }
        _ => {
            return Err(RuntimeError::new(
                fn_expr.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "a fn with a nominal (Path/Parametric) return type",
                    got: Box::new(ValueSnapshot::of(&Value::wat__core__fn(f.clone()))),
                },
            )
            .into());
        }
    };
    Ok(Value::String(Arc::new(fqdn)))
}

/// `(:wat::runtime::body-of <name :keyword>) -> (:Option :- [wat::holon::HolonAST])`
///
/// Arc 143 slice 1. Returns the body AST only — the `wat` body of a `UserFunction` (`None` for a
/// `FunctionBody::Native` builtin, per Stone 255.1a: it has no wat-level body) or the template
/// body of a `Macro`. `Primitive`, `Type`, `SpecialForm`, and `Registered` (arc 255 Stone 3a-i)
/// bindings are ALL body-less in the wat sense (primitives are Rust-implemented; types declare
/// shapes; special forms are semantic operations, not data with a body; a registry row's
/// `handler`/`value_handler` are likewise Rust, not a wat body) and return `:None`, same as an
/// unregistered name. (The sentinel `lookup-define` emits for these cases is for the FULL define
/// structure only; `body-of` is honest about the absence of a body specifically.)
///
/// ★ Doc correction (arc 255 Stone P6-c-W3): the prior header named only the UserFunction and
/// Primitive arms ("For user defines... For substrate primitives... For unknown names") and the
/// return-type annotation said `(:Option :- [wat::holon::HolonAST])` — stale on both counts. The
/// `Macro` arm (a real `Some(body)` case, omitted entirely from the prior prose) is completed
/// above, and the wrapped type is `:wat::WatAST` (arc 201/251/294.f; confirmed against
/// `check.rs`'s registered `TypeScheme`, which returns `Option<:wat::WatAST>`).
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Reflection
/// @arg     name_ast :wat::core::keyword the binding name whose body is read
/// @ret     (:wat::core::Option :- [:wat::WatAST]) the wat body (function or macro template), or `:None` when body-less or unregistered
/// @example (:wat::core::match (:wat::runtime::body-of :wat::core::cond) ((:wat::core::Some ast) (:wat::core::ast-kind ast)) (:wat::core::None "none")) #=> "list"
/// @example (:wat::core::match (:wat::runtime::body-of :wat::core::if) ((:wat::core::Some _) true) (:wat::core::None false)) #=> false
/// @see     :wat::runtime::lookup-define
/// @see     :wat::runtime::signature-of-defn
#[wat_intrinsic(":wat::runtime::body-of")]
pub(crate) fn eval_body_of(
    name_ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::runtime::body-of";
    // Mirror of eval_signature_of_defn (arc 150): when the argument is a literal keyword
    // AST, use the keyword string directly instead of going through `eval_inner`. Evaluating
    // a keyword that is a registered user define resolves to the fn VALUE stored in
    // `runtime_def_values` which carries `name: None` — `name_from_keyword_or_fn` would then
    // fail with the TypeMismatch below. The literal bypass preserves the name for lookup_form.
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
    // Arc 144 slice 1 — dispatch on uniform Binding. Bodies exist for
    // UserFunction (the wat body) + Macro (the template). Primitive,
    // Type, SpecialForm, and Registered (arc 255 Stone 3a-i) are all body-less in the
    // wat sense: primitives are Rust-implemented; types declare shapes (no body);
    // special forms are semantic operations, not data with a body; a registry row's
    // handler is likewise Rust, not a wat body.
    match lookup_form(&name, sym) {
        Some(Binding::UserFunction { f, .. }) => {
            // Stone 255.1a — Native builtins have no wat body; return None.
            let ast = match &f.body {
                FunctionBody::Wat(ast) => ast,
                FunctionBody::Native => return Ok(Value::Option(Arc::new(None))),
            };
            let body = (**ast).clone();
            Ok(Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(
                body,
            ))))))
        }
        Some(Binding::Macro { def, .. }) => {
            let body = def.body.clone();
            Ok(Value::Option(Arc::new(Some(Value::wat__WatAST(Arc::new(
                body,
            ))))))
        }
        Some(Binding::Primitive { .. }) => Ok(Value::Option(Arc::new(None))),
        Some(Binding::Type { .. }) => Ok(Value::Option(Arc::new(None))),
        Some(Binding::SpecialForm { .. }) => Ok(Value::Option(Arc::new(None))),
        // Arc 255 Stone 3a-i — a registry row is Rust-implemented (a `handler`/
        // `value_handler` fn pointer), never a wat body; body-less same as the three arms
        // above, for the same reason.
        Some(Binding::Registered { .. }) => Ok(Value::Option(Arc::new(None))),
        None => Ok(Value::Option(Arc::new(None))),
    }
}

/// Arc 294.f — the WatAST analogue of [`require_bundle`]. Reflection signature
/// heads are now `WatAST` compound nodes (a `List` for function/primitive
/// signatures, a `Vector` for macro/variadic argspecs), NOT lowered
/// `HolonAST::Bundle`s. Borrow the child forms, or error on a leaf node.
fn require_ast_children<'a>(
    op: &'static str,
    ast: &'a WatAST,
    arg_span: &Span,
) -> Result<&'a Vec<WatAST>, EvalBreak> {
    match ast {
        WatAST::List(children, _) | WatAST::Vector(children, _) => Ok(children),
        _ => Err(RuntimeError::new(
            arg_span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "List/Vector (signature head :wat::WatAST)",
                got: Box::new(ValueSnapshot::unavailable("non-compound WatAST variant")),
            },
        )
        .into()),
    }
}

/// `(:wat::runtime::rename-callable-name head from to) -> :wat::WatAST`. Takes a signature head
/// AST (a `List`/`Vector` `:wat::WatAST` whose first `Keyword` child is the bare callable name —
/// the shape `signature-of-defn`/`signature-of-fn` return) and returns a new head with the
/// function-name part replaced.
///
/// Arc 143 slice 3.
///
/// ★ Doc correction (arc 255 Stone P6-c-W3): the prior header called the head a `Bundle`
/// (`HolonAST`'s compound variant) and the return type `:wat::holon::HolonAST`. Both are stale:
/// arc 294.f moved reflection signature heads to plain `:wat::WatAST` `List`/`Vector` nodes —
/// `require_ast_children` below destructures exactly those two variants, never `HolonAST::Bundle`
/// — and the checker's registered `TypeScheme` for this FQDN (`check.rs::register_builtins`,
/// `watast_ty()`) has params/ret of `:wat::WatAST` throughout, not `HolonAST`.
///
/// Steps:
/// 1. Eval all three args; verify arg types.
/// 2. Destructure `head` into its `List`/`Vector` children (error if neither).
/// 3. Verify children[0] is a Keyword — the bare name (STONE reap-the-angle-machinery,
///    arc 109: a generic head's type params are `children[1..]`, the `:- [T ...]` binder
///    siblings `binder_head_nodes` emits; children[0] never carries a suffix).
/// 4. Verify children[0] == `from` keyword string. On
///    mismatch, error "rename-callable-name: head name does not match `from`".
/// 5. Construct new first keyword: `to`.
/// 6. Rebuild the head (same compound kind — List stays List, Vector stays Vector) with
///    [new_keyword, children[1..]] — the `:- [...]` binder, if any, rides along unchanged.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Reflection
/// @arg     head :wat::WatAST the signature head whose name is replaced
/// @arg     from :wat::core::keyword the expected current name (verified against `head`'s own name)
/// @arg     to   :wat::core::keyword the replacement name
/// @ret     :wat::WatAST the rebuilt head, same compound kind as `head`, name replaced
/// @example (:wat::core::ast-name (:wat::core::first (:wat::runtime::rename-callable-name (:wat::runtime::signature-of-fn (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64/+ x 1))) :anonymous :probe::renamed))) #=> ":probe::renamed"
/// @see     :wat::runtime::signature-of-fn
/// @see     :wat::runtime::extract-arg-names
#[wat_intrinsic(":wat::runtime::rename-callable-name")]
pub(crate) fn eval_rename_callable_name(
    head: &WatAST,
    from: &WatAST,
    to: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::runtime::rename-callable-name";
    // Eval all three args.
    let head_val = eval_inner(head, env, sym)?.value_owned();
    // Arc 166 — mirror the literal-keyword shortcut from `eval_lookup_define`:
    // when the `from`/`to` arg is a literal keyword AST (e.g. `:user::my-double`
    // passed directly as a keyword literal), use the keyword string as the name
    // directly instead of eval'ing it. Without this, a literal name keyword whose
    // defn-bound fn has `name: None` (eval_fn writes no name) causes
    // `name_from_keyword_or_fn` to return None → spurious TypeMismatch.
    // The eval path remains the fallback for non-literal callers.
    let from_val = if let WatAST::Keyword(k, _) = from {
        Value::wat__core__keyword(Arc::new(k.clone()))
    } else {
        eval_inner(from, env, sym)?.value_owned()
    };
    let to_val = if let WatAST::Keyword(k, _) = to {
        Value::wat__core__keyword(Arc::new(k.clone()))
    } else {
        eval_inner(to, env, sym)?.value_owned()
    };

    // Extract WatAST from head arg (arc 294.f — signature heads are now WatAST).
    let ast_arc = match head_val {
        Value::wat__WatAST(a) => a,
        other => {
            return Err(RuntimeError::new(
                head.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::WatAST (signature head)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };

    // Extract keyword strings from `from` and `to`.
    // Arc-009: known function names evaluate to their Function value, not
    // a keyword literal. Use `name_from_keyword_or_fn` to handle both.
    let from_str = match name_from_keyword_or_fn(&from_val) {
        Some(n) => n,
        None => {
            return Err(RuntimeError::new(
                from.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::core::keyword or named function (from name)",
                    got: Box::new(ValueSnapshot::of(&from_val)),
                },
            )
            .into());
        }
    };
    let to_str = match name_from_keyword_or_fn(&to_val) {
        Some(n) => n,
        None => {
            return Err(RuntimeError::new(
                to.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "wat::core::keyword or named function (to name)",
                    got: Box::new(ValueSnapshot::of(&to_val)),
                },
            )
            .into());
        }
    };

    // Destructure the signature head; children[0] is the name Keyword.
    // Preserve the compound kind (List for fn/primitive signatures, Vector for
    // macro argspecs) so the rebuilt head round-trips faithfully.
    let children = require_ast_children(OP, &ast_arc, head.span())?;
    if children.is_empty() {
        return Err(RuntimeError::new(
            head.span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "compound WatAST with at least one Keyword child (the function name)",
                got: Box::new(ValueSnapshot::unavailable("empty signature head")),
            },
        )
        .into());
    }
    // children[0] is a `WatAST::Keyword` — its stored string INCLUDES the leading colon
    // (e.g. `:user::add-two`). STONE reap-the-angle-machinery (arc 109) — this used to
    // split the name at `<` to preserve a type-param SUFFIX embedded in this same Keyword
    // (`:user::add-two<T>`). That spelling is retired: `binder_head_nodes` (this signature
    // head's own builder, for both native primitives and user functions) emits a generic
    // name's type params as SIBLINGS after the bare name Keyword — `[Keyword(name),
    // Keyword(":-"), Vector([T ...])]` — never folded into the name string. So `children[0]`
    // is always bare, and `children[1..]` (the `:-` binder, when present) already rides
    // along unchanged through the `new_children.extend_from_slice(&children[1..])` rebuild
    // below — confirmed against the live `foldl`→`reduce` golden
    // (`wat_arc143_manipulation__reduce_head.edn`: `(:wat.list/reduce :- [T Acc] ...)`, type
    // params preserved with NO suffix on the renamed head).
    let base = if let WatAST::Keyword(s, _) = &children[0] {
        s.as_str()
    } else {
        return Err(RuntimeError::new(
            head.span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "Keyword as first child (the function name)",
                got: Box::new(ValueSnapshot::unavailable("non-Keyword first child")),
            },
        )
        .into());
    };

    // `base` (from the WatAST Keyword) and `from_str` (from name_from_keyword_or_fn /
    // Value::wat__core__keyword) carry the leading colon; compare them directly.
    if base != from_str.as_str() {
        return Err(RuntimeError::new(
            from.span().clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: format!(
                    "rename-callable-name: head base name '{}' does not match `from` argument '{}'",
                    base, from_str
                ),
            },
        )
        .into());
    }

    // Construct the new first Keyword. `to_str` (from name_from_keyword_or_fn) already
    // includes the leading colon (e.g. `:bar::fn`); the WatAST Keyword payload keeps the
    // colon, so `to_str` alone is the correct stored form (no suffix to re-attach).
    let new_name = to_str;
    let head_span = children[0].span().clone();
    let new_first = WatAST::Keyword(new_name, head_span);

    // Rebuild the head, preserving its compound kind: [new_first] ++ children[1..].
    let mut new_children: Vec<WatAST> = Vec::with_capacity(children.len());
    new_children.push(new_first);
    new_children.extend_from_slice(&children[1..]);
    let rebuilt = match &*ast_arc {
        WatAST::Vector(_, span) => WatAST::Vector(new_children, span.clone()),
        WatAST::List(_, span) => WatAST::List(new_children, span.clone()),
        // require_ast_children already rejected leaf nodes.
        other => other.clone(),
    };

    Ok(Value::wat__WatAST(Arc::new(rebuilt)))
}

/// `(:wat::runtime::extract-arg-names head) -> (:wat::core::Vector :- [:wat::core::keyword])`.
/// Takes a signature head AST (the `:wat::WatAST` `List`/`Vector` `signature-of-fn`/
/// `signature-of-defn` return) and returns a `Vector` of the arg-name keywords (`:_a0`, `:_a1`,
/// ... for a substrate primitive, or the user-declared names for a `defn`/macro).
///
/// Arc 143 slice 3.
///
/// ★ Doc correction (arc 255 Stone P6-c-W3): the prior header's "Algorithm" step 1 said "destructure
/// head as Bundle" — stale (`HolonAST::Bundle`, not what this walks). The code below has always
/// destructured via [`require_ast_children`], which matches `WatAST::List`/`WatAST::Vector`
/// exclusively; there is no `Bundle` anywhere on this path (arc 294.f retired that representation
/// here). Reworded below to match what the body actually does.
///
/// Algorithm:
/// 1. Eval `head`; destructure into its `List`/`Vector` children.
/// 2. Skip children[0] (the function name Keyword — arc 221 Stone 221.4b) and peel a `:- [T
///    U…]` generic binder, if present (`peel_param_spec` — STONE-defservice-emits-the-binder;
///    without this, a binder Vector's own contents can misread as an arg-name pair).
/// 3. For each remaining child:
///    - `Symbol("->")`: STOP collecting (return-type sentinel).
///    - a bare `Symbol` (e.g. `&` variadic marker): skip.
///    - a 2-element `List`/`Vector` `(arg_name <type>)`: collect `arg_name` as `:arg_name`.
///    - anything else: skip.
/// 4. Return the collected keywords as a `Vector`.
///
/// **Expand-time ground —** shared infra for type-driven macros to reflect on fn signatures
/// at macro expand time (arc 249 stone 249.2b-i); `HolonAST → Vector<Keyword>`, pure
/// structural walk. Ruling relocated from `macros/eval.rs`'s expand-time allow-list (arc 255
/// expand-T4a), from its "Runtime reflection" group; the verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
/// @Category      Reflection
/// @arg     head :wat::WatAST the signature head walked
/// @ret     (:wat::core::Vector :- [:wat::core::keyword]) one keyword per declared arg name, in order
/// @example (:wat::runtime::extract-arg-names (:wat::runtime::signature-of-fn (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64/+ x 1)))) #=> [:x]
/// @see     :wat::runtime::extract-arg-types
/// @see     :wat::runtime::signature-of-fn
#[wat_intrinsic(":wat::runtime::extract-arg-names")]
pub(crate) fn eval_extract_arg_names(
    head: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::runtime::extract-arg-names";
    let head_val = eval_inner(head, env, sym)?.value_owned();
    let ast_arc = match head_val {
        Value::wat__WatAST(a) => a,
        other => {
            return Err(RuntimeError::new(
                head.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::WatAST (signature head)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let children = require_ast_children(OP, &ast_arc, head.span())?;

    let mut names: Vec<Value> = Vec::new();
    // Skip children[0] (function name keyword); walk from index 1. STONE-defservice-
    // emits-the-binder — `binder_head_nodes` (this file) now inserts `:- [T U…]` as
    // SIBLINGS right after a generic signature's head, so index 1 may be the `:-`
    // marker + a Vector of bare type-param symbols, not the first arg pair. Peel it
    // through the ONE door (`peel_param_spec`) before walking, or the type-param
    // Vector's own contents (which also happen to satisfy the 2-element pair shape
    // whenever there are exactly two params) get misread as an arg-name pair —
    // measured: `foldl<T,Acc>`'s `[T Acc]` binder produced a spurious `:T` name.
    let (_binder, rest) = crate::types::peel_param_spec(&children[1..]);
    for child in rest {
        if let WatAST::Symbol(ident, _) = child {
            if ident.as_str() == "->" {
                break; // Arrow sentinel — stop collecting.
            }
            // Bare symbol at top level (`&` variadic marker, etc.) — skip.
            continue;
        }
        // Arg-pair `(arg_name <type>)` — a two-element List (function
        // signatures) or Vector (macro/variadic surfaces). pair[0] is the
        // arg-name Symbol; pair[1] is the (canonical `wat.type/`) type node.
        if let WatAST::List(pair, _) | WatAST::Vector(pair, _) = child {
            if pair.len() == 2 {
                if let WatAST::Symbol(arg_name, _) = &pair[0] {
                    // Return the bare name as a plain keyword, e.g. `logger`
                    // -> `:logger` — the leading-colon convention every other
                    // keyword VALUE in this file follows.
                    names.push(Value::wat__core__keyword(Arc::new(format!(
                        ":{}",
                        arg_name.as_str()
                    ))));
                }
            }
        }
        // Any other shape: skip.
    }

    Ok(Value::Vec(Arc::new(names)))
}

/// `(:wat::runtime::extract-arg-types head) -> (:wat::core::Vector :- [:wat::WatAST])`. Direct
/// sibling of `extract-arg-names` (arc 143 slice 3) — same walk, collecting pair[1] (the type AST)
/// instead of pair[0] (the name keyword) from each 2-element arg-pair child.
///
/// Arc 201 slice 5.
///
/// ★ Doc correction (arc 255 Stone P6-c-W3): the prior header described an elaborate
/// HolonAST-carrier pipeline — `head` as `Value::holon__HolonAST`, a "hand-built Bundle", and a
/// render step through a named helper `holon_type_ast_to_wat_type_form` — that does not exist in
/// this codebase (`grep` finds the name only in comments: this one, a `check.rs` comment, and a
/// test-file comment — never a function definition). The body below has never called anything by
/// that name; it destructures via [`require_ast_children`] (`List`/`Vector` only) and returns
/// `pair[1].clone()` VERBATIM — no re-canonicalization, no HolonAST bridge, matching the inline
/// comment already sitting at the call site (arc 294.f: "the signature already carries canonical
/// `wat.type/` WatAST type nodes... we RETURN the sub-node verbatim"). Reworded below to match.
///
/// Algorithm:
/// 1. Eval `head`; destructure into its `List`/`Vector` children.
/// 2. Skip children[0] (the function name Keyword) and peel a `:- [T U…]` generic binder, if
///    present (`peel_param_spec`, same as `extract-arg-names`).
/// 3. For each remaining child:
///    - `Symbol("->")`: STOP collecting (everything after is return type).
///    - a 2-element `List`/`Vector` `(arg_name type_ast)`: collect `type_ast` verbatim.
///    - anything else: skip.
/// 4. Return the collected type ASTs as a `Vector`.
///
/// **Expand-time ground —** shared infra for type-driven macros to reflect on fn signatures
/// at macro expand time (arc 249 stone 249.2b-i); `HolonAST → Vector<Keyword>`, pure
/// structural walk. Ruling relocated from `macros/eval.rs`'s expand-time allow-list (arc 255
/// expand-T4a), from its "Runtime reflection" group; the verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
/// @Category      Reflection
/// @arg     head :wat::WatAST the signature head walked
/// @ret     (:wat::core::Vector :- [:wat::WatAST]) one type AST per declared arg, in order
/// @example (:wat::core::ast->source (:wat::core::first (:wat::runtime::extract-arg-types (:wat::runtime::signature-of-fn (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64/+ x 1)))))) #=> "wat.type/i64"
/// @see     :wat::runtime::extract-arg-names
/// @see     :wat::runtime::signature-of-fn
#[wat_intrinsic(":wat::runtime::extract-arg-types")]
pub(crate) fn eval_extract_arg_types(
    head: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::runtime::extract-arg-types";
    let head_val = eval_inner(head, env, sym)?.value_owned();
    let ast_arc = match head_val {
        Value::wat__WatAST(a) => a,
        other => {
            return Err(RuntimeError::new(
                head.span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::WatAST (signature head)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    let children = require_ast_children(OP, &ast_arc, head.span())?;

    let mut types: Vec<Value> = Vec::new();
    // Skip children[0] (function name keyword); walk from index 1. STONE-defservice-
    // emits-the-binder — see the identical note in `eval_extract_arg_names`, just
    // above: peel a `:- [T U…]` binder (if present) through the ONE door before
    // walking, or its own Vector contents get misread as an arg-pair.
    let (_binder, rest) = crate::types::peel_param_spec(&children[1..]);
    for child in rest {
        if let WatAST::Symbol(ident, _) = child {
            if ident.as_str() == "->" {
                break; // Arrow sentinel — stop collecting.
            }
            continue; // Bare symbol at top level (`&`, etc.) — skip.
        }
        // Arg-pair `(arg_name <type>)` — a two-element List/Vector; pair[1] is
        // the type node. Post arc-294.f the signature already carries canonical
        // `wat.type/` WatAST type nodes (emitted by `type_expr_to_ast` via
        // `type_expr_to_clojure_form`), so we RETURN the sub-node verbatim — no
        // re-canonicalization, no HolonAST bridge.
        if let WatAST::List(pair, _) | WatAST::Vector(pair, _) = child {
            if pair.len() == 2 {
                types.push(Value::wat__WatAST(Arc::new(pair[1].clone())));
            }
        }
        // Any other shape: skip.
    }

    Ok(Value::Vec(Arc::new(types)))
}

/// `(:wat::runtime::field-names-of type-kw) -> (:wat::core::Vector :- [:wat::core::keyword])`
///
/// Arc 170 Strike B — struct-field reflection, the type-direction sibling
/// of `extract-arg-names` (which reflects a *callable's* argspec; this
/// reflects a *type's* field list). `type-kw` is a type keyword
/// (`:probe::Bag`) evaluated at call time; resolved through the runtime
/// type registry (`sym.types`, an `Option<Arc<TypeEnv>>` populated at
/// freeze time — the same registry `lookup_form`'s `Binding::Type` arm
/// reads) to its `AggregateDef`. `AggregateDef.fields: Vec<(String,
/// TypeExpr)>` is always in declaration order (Vec preserves insertion
/// order; no re-sorting anywhere in the parse path), so no extra sort is
/// needed here.
///
/// Value representation: a plain KEYWORD per field name — the substrate's
/// own "struct fields as data" form (`closure_extract.rs:2521,2542` ships
/// a struct's field types as `WatAST::Keyword(format_type(fty))`; a type
/// IS a keyword). NOT the retired `HolonAST` carrier `extract-arg-types`
/// uses — we do not mint new HolonAST-returning intrinsics. Each bare
/// field name (`kv`) becomes the keyword `:kv` via
/// `Value::wat__core__keyword(Arc::new(format!(":{}", name)))` — the
/// leading-colon convention every other keyword-value construction in
/// this file follows (e.g. runtime.rs:8150,14834).
///
/// Arg handling mirrors the arc-166 pattern in `eval_lookup_define`: a
/// literal `WatAST::Keyword` is used directly, without going through
/// `eval_inner`. This matters here even more than for `lookup-define` —
/// arc 009 "names are values" lifts a type keyword whose bare name also
/// carries a defstruct-synthesized ctor (or, per `sym.get`, ANY
/// registered callable including a defrecord's constructor) to a
/// `Value::wat__core__fn`, not a keyword, when evaluated. Going through
/// `eval_inner` would make every struct/record type keyword fail this
/// primitive. The eval fallback remains for non-literal callers (a var
/// holding a keyword).
///
/// ★ Doc read against body (arc 255 Stone P6-c-W4): the prior informal return-type shorthand
/// (`wat::core::Keyword`, capitalized, colonless) named the right element type — matched, only
/// reformatted to the canonical `:wat::core::keyword` spelling this stone's structured `@ret`
/// requires; not a lie.
///
/// **Expand-time ground —** type-kw → the frozen runtime type registry (`sym.types`,
/// populated once at freeze time) → `AggregateDef.fields`; same category as `signature-of-fn`
/// (read-only reflection off already-frozen registry state, no IO, no mutation, deterministic).
/// Ruling relocated from `macros/eval.rs`'s expand-time allow-list (arc 255 expand-T4a; arc 170
/// Strike B); the verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
/// @Category      Reflection
/// @arg     type_kw_ast :wat::core::keyword the struct/record type name whose field names are read (a literal keyword; a non-literal keyword-valued expression also resolves via `resolve_type_keyword_arg`)
/// @ret     (:wat::core::Vector :- [:wat::core::keyword]) each field name as a keyword, in declaration order
/// @example (:wat::runtime::field-names-of :wat::program::Env) #=> [:started-at :peer-started-at :process-id :os-thread-id :peer-kind :cpu-count :user-data]
/// @see     :wat::runtime::field-types-of
/// @see     :wat::runtime::extract-arg-names
#[wat_intrinsic(":wat::runtime::field-names-of")]
pub(crate) fn eval_field_names_of(
    type_kw_ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::runtime::field-names-of";
    let type_kw = resolve_type_keyword_arg(OP, type_kw_ast, env, sym)?;
    let agg = resolve_aggregate_def_for_reflection(OP, &type_kw, type_kw_ast.span(), sym)?;

    let names: Vec<Value> = agg
        .fields
        .iter()
        .map(|(name, _)| Value::wat__core__keyword(Arc::new(format!(":{}", name))))
        .collect();
    Ok(Value::Vec(Arc::new(names)))
}

/// `(:wat::runtime::field-types-of type-kw) -> (:wat::core::Vector :- [:wat::WatAST])`
///
/// Arc 170 Strike B — direct sibling of `eval_field_names_of` above (same
/// resolution: `type-kw` → runtime type registry → `AggregateDef`).
/// Positionally aligned with `field-names-of`'s output (both walk
/// `AggregateDef.fields` in the same declaration order) so a downstream
/// consumer can zip the two vectors.
///
/// Value representation (post arc-251 type-form rewire): each field's
/// `TypeExpr` is rendered directly via
/// [`crate::edn::render::type_expr_to_clojure_form`] to the canonical
/// `wat.type/` `WatAST` form and wrapped as `Value::wat__WatAST` — NOT the
/// old `format_type` keyword flattening, which mangled parametric types
/// (`Peer'<probe::Kv::Op,probe::Kv::Reply>` → the broken, non-reparseable
/// keyword `:wat.kernel.Peer'<probe.Kv.Op_probe.Kv/Reply>`). The new form
/// is plain-EDN and decomposable: an atomic type renders to
/// `WatAST::Symbol("wat.type/i64")`; a parametric type renders to a
/// `WatAST::List`, e.g. `(wat.kernel/Peer' probe.Kv/Op probe.Kv/Reply)`.
///
/// ★ Doc read against body (arc 255 Stone P6-c-W4): the prior informal return-type shorthand
/// (`wat::WatAST`, colonless) named the right element type — matched, only reformatted to the
/// canonical `:wat::WatAST` spelling this stone's structured `@ret` requires; not a lie.
///
/// **Expand-time ground —** type-kw → the frozen runtime type registry (`sym.types`,
/// populated once at freeze time) → `AggregateDef.fields`; same category as `signature-of-fn`
/// (read-only reflection off already-frozen registry state, no IO, no mutation, deterministic).
/// Ruling relocated from `macros/eval.rs`'s expand-time allow-list (arc 255 expand-T4a; arc 170
/// Strike B); the verdict is that list's.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Legal
/// @Category      Reflection
/// @arg     type_kw_ast :wat::core::keyword the struct/record type name whose field types are read (a literal keyword; a non-literal keyword-valued expression also resolves via `resolve_type_keyword_arg`)
/// @ret     (:wat::core::Vector :- [:wat::WatAST]) each field's type, rendered as a canonical `wat.type/` WatAST node, in declaration order (positionally aligned with `field-names-of`)
/// @example (:wat::core::ast->source (:wat::core::first (:wat::runtime::field-types-of :wat::program::Env))) #=> "wat.time/Instant"
/// @see     :wat::runtime::field-names-of
/// @see     :wat::runtime::extract-arg-types
#[wat_intrinsic(":wat::runtime::field-types-of")]
pub(crate) fn eval_field_types_of(
    type_kw_ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::runtime::field-types-of";
    let type_kw = resolve_type_keyword_arg(OP, type_kw_ast, env, sym)?;
    let agg = resolve_aggregate_def_for_reflection(OP, &type_kw, type_kw_ast.span(), sym)?;

    let mut types: Vec<Value> = Vec::with_capacity(agg.fields.len());
    for (_, ty) in agg.fields.iter() {
        let node = crate::edn::render::type_expr_to_clojure_form(ty, crate::edn::render::TypeFormHeadMode::Clojure).map_err(|reason| {
            RuntimeError::new(
                list_span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: OP.into(),
                    reason,
                },
            )
        })?;
        types.push(Value::wat__WatAST(Arc::new(node)));
    }
    Ok(Value::Vec(Arc::new(types)))
}

/// Shared arg-resolution step for `field-names-of` / `field-types-of`:
/// extracts the type keyword string (colon-included, e.g. `":probe::Bag"`)
/// from the call's sole argument.
///
/// Mirrors the arc-166 pattern in `eval_lookup_define` exactly: a literal
/// `WatAST::Keyword` is read directly, bypassing `eval_inner`. This is not
/// merely stylistic here — `eval_inner`'s `WatAST::Keyword` arm implements
/// arc 009 "names are values" (runtime.rs:3737: `if let Some(func) =
/// sym.get(k) { return ...Value::wat__core__fn(func.clone()) }`), and a
/// struct/record type's bare name IS a registered callable (its
/// constructor). Evaluating `:probe::Bag` would therefore yield a
/// `Value::wat__core__fn`, not a keyword — silently breaking this
/// primitive for every struct/record type. The eval fallback (via
/// `name_from_keyword_or_fn`) stays for non-literal callers, e.g. a local
/// variable bound to a keyword.
fn resolve_type_keyword_arg(
    op: &str,
    arg: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<String, EvalBreak> {
    if let WatAST::Keyword(k, _) = arg {
        return Ok(k.clone());
    }
    let v = eval_inner(arg, env, sym)?.value_owned();
    match name_from_keyword_or_fn(&v) {
        Some(n) => Ok(n),
        None => Err(RuntimeError::new(
            arg.span().clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "wat::core::keyword (type name)",
                got: Box::new(ValueSnapshot::of(&v)),
            },
        )
        .into()),
    }
}

/// Shared resolution step for `field-names-of` / `field-types-of`:
/// `type-kw` (as evaluated, colon-included — e.g. `":probe::Bag"`) →
/// the runtime type registry's `AggregateDef`. Unlike the
/// extract-arg-names/-types pair (which walk near-identical HolonAST
/// shapes and are kept as two independent walkers per
/// `feedback_simple_is_uniform_composition`), this lookup step is
/// genuinely one mechanical operation (registry get + variant match),
/// not a second near-duplicate walker — sharing it removes duplication
/// without hiding any per-verb logic.
///
/// `sym.types` (`Option<Arc<TypeEnv>>`) is populated at freeze time
/// (`FrozenWorld::freeze` → `symbols.set_types`, see runtime.rs:5309)
/// from the same `TypeEnv` the checker resolved `:probe::Bag` against —
/// the runtime-reachable type registry `lookup_form`'s `Binding::Type`
/// arm also reads (runtime.rs:10463/10509).
///
/// GROUNDED (not assumed): `TypeEnv` keys are colon-INCLUDED, despite
/// `parse_declared_name`'s misleading "strip the colon" comment — that
/// function's actual return value in the non-parametric branch is `Ok((raw,
/// ...))` (`raw`, the ORIGINAL colon-included string; `types.rs:2769`), and
/// in the parametric branch it re-adds the colon (`format!(":{}", base)`;
/// `types.rs:2801`). Confirmed independently by `register_aggregate_methods`
/// (runtime.rs:1165): `agg.name = ":myapp::Voltage"` per its own comment,
/// and by an empirical probe (`lookup-define :probe::Bag` round-tripped
/// correctly with the colon kept). So the lookup key is `type_kw` UNCHANGED
/// — no trim. (`eval_struct_new`'s `trim_start_matches(':')` is unrelated:
/// it strips for `AggregateValue::struct_`'s VALUE-level naming convention,
/// not a `TypeEnv` HashMap key.)
fn resolve_aggregate_def_for_reflection<'a>(
    op: &str,
    type_kw: &str,
    span: &Span,
    sym: &'a SymbolTable,
) -> Result<&'a crate::types::AggregateDef, EvalBreak> {
    match sym.types().and_then(|t| t.get(type_kw)) {
        Some(crate::types::TypeDef::Aggregate(a)) => Ok(a),
        Some(_) => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: format!("'{}' is not a struct/record type (no fields)", type_kw),
            },
        )
        .into()),
        None => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: format!("unknown type '{}'", type_kw),
            },
        )
        .into()),
    }
}
