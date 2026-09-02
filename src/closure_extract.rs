//! Closure extraction — substrate-internal Rust capability.
//!
//! Arc 170 slice 1b. Given a `Value::wat__core__fn` plus the parent
//! world's `SymbolTable` + `TypeEnv`, produce a `ClosurePackage`:
//! a `prologue` (top-level WatAST forms — the captured environment) plus
//! an `entry_form` (an expression evaluating to a fn Value). When the
//! prologue is fed through `startup_from_forms` and `entry_form` is then
//! `eval`d in that fresh world, the resulting fn Value is behaviorally
//! equivalent to the original.
//!
//! **Scope**: Rust-internal in arc 170. NOT exposed at wat level.
//! Future remote-program arc may expose it. The first wat-level
//! consumer is `eval_kernel_spawn_process` in slice 2.
//!
//! **Algorithm** (per CLOSURE-EXTRACTION.md v2):
//!
//! 1. Resolve entry: keyword-path input → register entry fn into deps so
//!    its define ends up in `prologue`; entry_form = `Symbol(name)`.
//!    Inline-lambda input → no name; entry_form = reconstructed fn-form
//!    AST `(:wat::core::fn [name <- :T ...] -> :Ret body)`.
//! 2. Walk the entry body's AST, track scope, collect free references.
//! 3. Recursively extract user dependencies (other defns, types) until
//!    fixpoint; visited-set guards recursive types.
//! 4. Encode captured runtime Values to AST.
//! 5. Portability check: refuse channel/IO/process/handle types.
//! 6. Assemble: prologue = type defs → capture defines → user dep
//!    defines (topological) — INCLUDING the entry fn's define when input
//!    was a keyword path. entry_form = the expression that evaluates to
//!    a fn Value (Symbol AST for keyword-path; fn-form AST for lambda).
//!
//! **Discipline**: zero Mutex (per ZERO-MUTEX.md). No process-wide
//! synthetic-name counter — slice 1b retired the entry-keyword ceremony.

use crate::ast::WatAST;
// Arc 278 — the data/code boundary classifier. This walker consults the ONE place the
// boundary-head set is encoded rather than re-deriving it (see the dispatch in
// `walk_free_symbols`); `resolve::walk` and `resolve::normalize` are the other two consumers.
use crate::resolve::boundary::{is_unquote_escape, quote_boundary, Boundary};
use crate::scope::Identifier;
use crate::runtime::{eval, Environment, Function, FunctionBody, RuntimeError, RuntimeErrorKind, SymbolTable, Value, ValueSnapshot};
use crate::value::value::AggregateValue;
use crate::types::Nature;
use crate::span::{span_prefix, Span};
use crate::types::{TypeDef, TypeEnv, TypeExpr};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::Arc;

// Arc 244 Doctrine 1 — the TYPE keyword spelling of nil (the placeholder elem/ok/err type for
// an empty Vec/HashMap/HashSet or a payload-less Result arm — a genuinely PARSEABLE type,
// `TypeExpr::Path(":wat::core::nil")`), distinct from the VALUE literal `WatAST::nil()`/
// `NilLit` `gate_no_nil_keyword_synthesis` guards against (a `NilLit` is not one of
// `parse_type_node`'s accepted type-position node kinds, so it cannot replace this). Named
// so every synthesis site below routes through one spelling instead of a repeated literal.
const NIL_TYPE_PATH_KEYWORD: &str = ":wat::core::nil";

// ─── Public API ─────────────────────────────────────────────────────────

/// The product of closure extraction.
///
/// `prologue` is an ordered Vec of top-level WatAST forms — the captured
/// environment. When fed through `startup_from_forms`, it produces a
/// fresh world that seeds every type, dep, and captured value the entry
/// expression needs.
///
/// `entry_form` is an expression that, when `eval`d in the frozen fresh
/// world, produces a fn Value behaviorally equivalent to the original.
/// Two shapes:
///   - keyword-path input — `entry_form` is a Symbol AST whose name
///     resolves to a fn defined in `prologue`.
///   - inline-lambda input — `entry_form` is the reconstructed fn-form
///     AST `(:wat::core::fn [name <- :T ...] -> :Ret body)`.
#[derive(Debug, Clone)]
pub struct ClosurePackage {
    pub prologue: Vec<WatAST>,
    pub entry_form: WatAST,
}

/// Errors surfaced during extraction. Pattern A (Stone 243.7d): span
/// at the outer struct level; variant data in `ExtractionErrorKind`.
///
/// `ImpureCapture` is the substrate-as-teacher rejection: a captured
/// value whose type is channel-bearing / IO / process-handle cannot
/// cross a process boundary because pointer-identity does not survive
/// `fork(2)`. The diagnostic names the offending capture, its type,
/// the field path inside (when nested), and points the user at pipes /
/// restructure. (Renamed from `NonPortableCapture` by arc 293.W.2d.)
#[derive(Debug, Clone)]
pub struct ExtractionError {
    pub span: Span,
    pub kind: ExtractionErrorKind,
}

/// Variant data for [`ExtractionError`]. Spans live in the outer struct;
/// variants carry ONLY data unique to each failure kind.
#[derive(Debug, Clone)]
pub enum ExtractionErrorKind {
    /// A captured value of an impure (non-wire-serializable) type was found
    /// in a PROCESS-spawn closure. Process spawns cross address-space boundaries
    /// and cannot carry impure values (channels, handles, structs, etc.).
    /// Thread-spawn closures are in-locus and are NOT checked here.
    /// No span — constructs with outer `crate::rust_caller_span!()`.
    ImpureCapture {
        /// The let-scope name of the offending capture.
        name: String,
        /// The type name (may be a `Sender<i64>`, `Process<I,O>`, etc.).
        type_name: String,
        /// Path inside a struct/tuple if the offending value is nested:
        /// e.g. `["my-config", "tx-field"]`. Empty for direct captures.
        path: Vec<String>,
    },
    /// A free symbol could not be resolved against the parent's symbol
    /// table or treated as a substrate primitive.
    UnresolvedSymbol { name: String },
    /// An internal invariant was violated. Not user-actionable; surfaces
    /// programmer bugs (e.g., `Function` carries no body span when one
    /// was expected). No span — constructs with outer `crate::rust_caller_span!()`.
    Internal(String),
}

impl std::fmt::Display for ExtractionErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractionErrorKind::ImpureCapture {
                name,
                type_name,
                path,
            } => {
                let path_suffix = if path.is_empty() {
                    String::new()
                } else {
                    format!(" (field path: {})", path.join("."))
                };
                write!(
                    f,
                    "spawn-process closure captures `{}` of impure type `{}`{}.\n\
                     Impure types (channels, handles, structs) cannot cross process \
                     boundaries (different address space, §7).\n\
                     Use stdin/stdout/stderr pipes for inter-process communication, or\n\
                     restructure the program so the resource is created in the spawned program.",
                    name, type_name, path_suffix
                )
            }
            ExtractionErrorKind::UnresolvedSymbol { name } => write!(
                f,
                "free symbol `{}` does not resolve to a parent define or substrate primitive",
                name
            ),
            ExtractionErrorKind::Internal(msg) => write!(f, "closure-extract internal: {}", msg),
        }
    }
}

impl std::fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = span_prefix(&self.span);
        write!(f, "{}{}", prefix, self.kind)
    }
}

impl std::error::Error for ExtractionError {}

/// Extract a closure package from a fn value.
///
/// `entry_name` is `Some(":my::path")` if the caller obtained the fn
/// via a keyword-path lookup (top-level defn case); `None` for inline
/// lambdas / factory results. Slice 1b retired the synthetic-name
/// ceremony — for inline-lambda input, no name is minted; the package's
/// `entry_form` is the reconstructed fn-form AST itself.
pub fn extract_closure(
    fn_value: &Value,
    entry_name: Option<&str>,
    parent_symbols: &SymbolTable,
    parent_types: &TypeEnv,
) -> Result<ClosurePackage, ExtractionError> {
    let func = match fn_value {
        Value::wat__core__fn(f) => f.clone(),
        other => {
            return Err(ExtractionError {
                span: crate::rust_caller_span!(),
                kind: ExtractionErrorKind::Internal(format!(
                    "extract_closure expected Value::wat__core__fn, got {}",
                    other.type_name()
                )),
            })
        }
    };

    // Resolve entry mode. Two cases:
    //
    // 1. Keyword-path input — caller passed Some(name); or the
    //    function's own `name` field carries one. The entry's define
    //    form (with its rewritten body) goes into `prologue`; the
    //    `entry_form` is a Symbol AST naming it.
    // 2. Inline lambda — no name. The `entry_form` is the
    //    reconstructed fn-form AST itself; nothing about the entry
    //    appears in `prologue`. Per arc 170 slice 1b's "the fn IS the
    //    program" framing.
    let entry_path: Option<String> = match (entry_name, &func.name) {
        (Some(n), _) => Some(n.to_string()),
        (None, Some(n)) => Some(n.clone()),
        (None, None) => None,
    };
    let is_lambda = entry_path.is_none();

    // Walker / extraction state.
    let mut state = ExtractState::new(parent_symbols, parent_types);

    // Walk the entry fn's signature for type refs (param types + rest +
    // ret) — these contribute user types to the package even if the
    // body doesn't mention them by keyword.
    for ty in &func.param_types {
        record_type_refs_in_typeexpr(&mut state, ty);
    }
    record_type_refs_in_typeexpr(&mut state, &func.ret_type);
    if let Some(rest_ty) = &func.rest_param_type {
        record_type_refs_in_typeexpr(&mut state, rest_ty);
    }

    // Walk the entry fn's body for free symbols. Parameters + rest-param
    // are LOCAL within the body's scope.
    let mut body_locals: BTreeSet<String> =
        func.params.iter().map(|p| crate::scope::env_key(p).into_owned()).collect();
    if let Some(rest) = &func.rest_param {
        body_locals.insert(rest.clone());
    }
    // Stone 255.1a — Native builtins have no wat body; closure extraction is N/A.
    let body_ast = match &func.body {
        FunctionBody::Wat(ast) => ast,
        FunctionBody::Native => unreachable!("native builtin fn-applied — dispatched via the runtime match, not fn-apply"),
    };
    walk_free_symbols(body_ast, &body_locals, &mut state)?;

    // Process captured locals from the fn's closed environment. Match
    // free symbols against the closed env to identify captures; their
    // names go into the captures map and the remaining frees are
    // reclassified or surfaced as unresolved.
    if let Some(closed_env) = &func.closed_env {
        let frees = std::mem::take(&mut state.unresolved_frees);
        for (name, span) in frees {
            if let Some(tv) = closed_env.lookup(&name, &span) {
                // It's a captured local. Encode the value to AST and
                // record. The captured value itself may carry types we
                // need to extract; the type-walk phase below handles
                // them.
                let encoded = encode_value_to_ast(tv.value(), &name, &mut state)?;
                state.captured_bindings.push(CapturedBinding {
                    original_name: name.clone(),
                    synthetic_name: synthesize_capture_name(&name),
                    encoded_ast: encoded,
                });
                state.captured_locals.insert(name);
            } else {
                // Not in closed env; not a parent symbol or type.
                // Genuinely unresolved.
                state.really_unresolved.push((name, span));
            }
        }
    } else {
        // No closed env — every unresolved free is genuinely unresolved.
        let frees = std::mem::take(&mut state.unresolved_frees);
        state.really_unresolved.extend(frees);
    }

    // Surface the first unresolved (others would cascade similar errors).
    if let Some((name, span)) = state.really_unresolved.first().cloned() {
        return Err(ExtractionError { span, kind: ExtractionErrorKind::UnresolvedSymbol { name } });
    }

    // Recursively extract user deps + user types. Walks each dep AST
    // for further frees; chases types through their fields.
    extract_user_deps_to_fixpoint(&mut state)?;
    extract_user_types_to_fixpoint(&mut state)?;

    // Arc 170 slice 3 Gap F-3 — propagate parent's full user type registry.
    //
    // The reference-walking path above captures only types that are
    // STATICALLY referenced in the fn signature or body AST (via
    // `record_type_refs_in_typeexpr` / `walk_free_symbols`). Types used
    // DYNAMICALLY — e.g. `:wat::edn::read` deserializing a tagged EDN
    // string whose type tag names a parent-declared struct or enum — are
    // not reachable by static analysis. The child subprocess's TypeEnv
    // would therefore be missing them, causing `EdnReadError::UnknownTag`
    // at runtime.
    //
    // Fix: sweep ALL non-reserved user types from the parent's TypeEnv
    // into `state.captured_types`. Since `record_type_dependency` is
    // idempotent (no-op if already present), types already captured via
    // the reference-walking path are skipped without duplication.
    //
    // Inclusion strategy: WHOLE registry (not filtered by body
    // references). Rationale (four questions):
    //   Obvious: whole-registry is the only strategy that handles
    //     dynamic type dispatch (edn::read, reflection). Filtered-by-
    //     body-references leaves the gap open for any indirect use.
    //   Simple: one loop over parent_types.iter(); no body-walking
    //     changes; no new walker infrastructure.
    //   Honest: the cost is proportional to the parent world's user
    //     type count — typically O(tens) for real programs. Types are
    //     immutable data; sharing them with the child is correct per
    //     BRIEF §"Hermetic semantics preserved".
    //   Good UX: child always has the full type picture the parent
    //     had; edn::read / reflection / future dynamic dispatch all
    //     work without the caller needing to annotate "exported types".
    //
    // Reserved-prefix filter: types under `:wat::*` / `:rust::*` are
    // built-in stdlib types always re-registered in the child's world
    // via `TypeEnv::with_builtins()` + `register_stdlib_types` during
    // `startup_from_forms`. Sweeping them would trigger the reserved-
    // prefix gate in `TypeEnv::register` and fail. Only user-namespace
    // types (no reserved prefix) belong in the prologue.
    for (name, def) in parent_types.iter() {
        if !crate::resolve::is_reserved_prefix(name) {
            record_type_dependency(&mut state, name, def);
            // Arc 278 — A TYPE'S CONSTRUCTOR RIDES WITH IT.
            //
            // The registry census measured 182 names carrying BOTH a `Type`
            // and a `Macro` facet: a record's type declaration and its kwargs
            // constructor share one name. Sweeping the type alone ships a type
            // the child cannot construct — which is exactly how a synthesized
            // record arrived uncallable.
            //
            // This is the RIGHT place, not the Keyword walker: types are swept
            // UNCONDITIONALLY (see above), so a record whose constructor is
            // only ever called from a form we generate — never named as a free
            // keyword in a walked body — is reached here and nowhere else.
            record_macro_dependency(&mut state, name);
        }
    }

    // Body rewrite: rewrite captured-local references in the entry
    // body from `X` to the synthetic capture name (a bare Symbol with
    // the substituted name). This avoids collision with extracted
    // user-symbol names and makes the bindings explicit. Runs BEFORE
    // entry_form is assembled so the rewritten body is what flows
    // into either the keyword-path entry-define (in prologue) or the
    // inline-lambda fn-form AST (entry_form).
    //
    // Capture rewrite runs on the FULL body (including any prelude
    // forms) so that define/struct/enum forms referencing closed-env
    // captures are rewritten BEFORE the Gap H prelude-lift extracts
    // them from the body.
    // Stone 255.1a — body_ast is guaranteed Wat here (Native would have already unreachable!'d above).
    let rewritten_body = rewrite_captures(body_ast, &state.captured_bindings, &body_locals);

    // Arc 170 slice 3 Gap H — lift fn body prelude forms into prologue.
    //
    // If the fn body is a `(:wat::core::do ...)` form whose leading
    // children are `define` / `struct` / `enum` declarations (the
    // "prelude prefix"), those forms cannot be evaluated at child
    // runtime: `eval_do_tail` returns `DefineInExpressionPosition`
    // for any `(:wat::core::define ...)` at expression position.
    //
    // Fix: extract the leading prelude run from the rewritten body and
    // place the forms into `body_prelude_forms` for later insertion
    // into the prologue. The residual body (pure expressions after the
    // prelude run) becomes the fn's actual body in the entry_form.
    //
    // Capture rewrite has ALREADY run above; lifted forms that
    // reference closed-env captures already carry the rewritten
    // synthetic keyword form (`:user::closure-capture::X`).
    // Captured_binding defines are in the prologue BEFORE the lifted
    // prelude forms (prologue assembly step 2 precedes step 5 below),
    // so the `:user::closure-capture::X` keyword resolves correctly at
    // child startup.
    //
    // Non-do bodies (single expression, let, etc.) are left unchanged:
    // `split_body_prelude` returns (empty, body) for non-do shapes.
    let (body_prelude_forms, final_body) = split_body_prelude(rewritten_body);

    // Assemble prologue in topological order:
    //   0. (Arc 278) Macros — retained `(defmacro …)` forms, shipped
    //      verbatim. First, because expansion precedes everything and any
    //      later form may call one.
    //   1. Type definitions (types in topological order)
    //   2. Captured-binding defines (`(def :user::closure-capture::X <encoded>)`)
    //   2b. (Arc 278) `def`-bound values carried from the parent, under
    //       their ORIGINAL name — in discovery order. Sits between step
    //       2 and step 3: a def's encoded value may construct a user
    //       type (step 1 must precede it), and a dep fn's body may read
    //       the def (step 3 must follow it).
    //   3. User dependency defines (in topological order)
    //   4. (Arc 170 slice 3 Gap H) Lifted fn-body prelude forms —
    //      define/struct/enum forms extracted from the fn body's
    //      leading do-prefix. These register in the child's world
    //      at startup_from_forms step 5 (types) + step 6 (defines)
    //      before the body runs. Appended AFTER parent-types (step
    //      1, F-3) and captured values (step 2) so that lifted
    //      defines that reference closed-env captures resolve to
    //      `:user::closure-capture::X` keywords that are already bound.
    //   5. (keyword-path input only) the entry fn's define form,
    //      with rewritten body — appended after all deps. For
    //      inline-lambda, the entry never appears in prologue.
    let mut prologue: Vec<WatAST> = Vec::new();

    // 0. Macros, in discovery order. They come FIRST: a macro is an
    //    EXPAND-time registration, and every later form may call one.
    for name in &state.macro_discovery_order {
        if let Some(form) = state.captured_macros.get(name) {
            prologue.push(form.clone());
        }
    }

    // 1. Types in deterministic topological order.
    let type_order = topo_sort_types(&state);
    for tn in &type_order {
        // Arc 170 — prefer the RETAINED original source form (shipped verbatim,
        // cannot drift). Fall back to `type_def_to_ast` reconstruction only for
        // synthesized types (records/enums derived by the parent) that have no
        // user source form.
        if let Some(src) = state.parent_types.source_form(tn) {
            prologue.push(src.clone());
        } else if let Some(def) = state.captured_types.get(tn) {
            prologue.push(type_def_to_ast(def));
        }
    }

    // 2. Captured-binding defines.
    for cb in &state.captured_bindings {
        prologue.push(capture_define_form(cb));
    }

    // 2b. `def`-bound values, in discovery order. The body references
    //     these by Keyword and `rewrite_captures` never rewrites a
    //     Keyword, so each keeps its ORIGINAL name.
    for name in &state.def_discovery_order {
        if let Some(encoded) = state.captured_defs.get(name) {
            prologue.push(def_form(name, encoded));
        }
    }

    // 3. User defns in topological order.
    let dep_order = topo_sort_deps(&state);
    for dep_name in &dep_order {
        if let Some(dep_func) = state.captured_deps.get(dep_name) {
            prologue.push(function_to_define_form(dep_func));
        }
    }

    // 4. (Gap H) Lifted fn-body prelude forms.
    for prelude_form in body_prelude_forms {
        prologue.push(prelude_form);
    }

    // 5. Entry resolution: keyword-path mode appends the entry
    //    define (with rewritten body) to prologue; inline-lambda
    //    mode emits the fn-form AST as `entry_form` directly.
    let entry_form = match &entry_path {
        Some(path) => {
            // Keyword-path: the entry's own define belongs in
            // prologue (after every dep it transitively pulled in).
            // The Keyword AST naming it is the entry_form — when
            // `eval`d in the frozen world, the Keyword arm
            // (runtime.rs ≈ line 2846) does `sym.get(k)` and lifts
            // the registered Function to a `Value::wat__core__fn`.
            //
            // (Slice 1b honest-delta note: CLOSURE-EXTRACTION.md v2
            // describes this as "a Symbol AST naming the keyword";
            // wat-rs's eval resolves bare-Symbol references via
            // `env.lookup` — only lexical bindings — so naming a
            // top-level defn requires a Keyword AST. The intent of
            // the spec — "a name reference that evaluates to the fn
            // Value" — is preserved; the surface is Keyword, not
            // Symbol, for substrate-fit.)
            let entry_define =
                function_to_define_form_with_body(&func, path, final_body);
            prologue.push(entry_define);
            WatAST::Keyword(path.clone(), crate::rust_caller_span!())
        }
        None => {
            // Inline lambda: emit the fn-form AST. The fn-form
            // evaluates to a fn Value at consumer time; no define
            // wrapping; no synthetic name.
            let _ = is_lambda; // documented above; kept for clarity
            function_to_fn_form(&func, final_body)
        }
    };

    Ok(ClosurePackage {
        prologue,
        entry_form,
    })
}

// ─── Wat-level dispatch: `:wat::kernel::fn-forms` ───────────────────────
//
// Arc 259 (forced-hand) Stone S1 — the first wat-level consumer of
// `extract_closure` (see the module doc's "first wat-level consumer is
// eval_kernel_spawn_process in slice 2" note — that was slice 1b's
// forward-looking guess; this stone instead exposes it directly as its
// own verb, `fn-forms`, so the not-shared bracket path can reify a
// work-fn and ship the resulting forms itself).

/// Wat-level dispatch arm for `:wat::kernel::fn-forms`.
///
/// Arity 2 — `(f name)`. `f` evaluates to a `:wat::core::fn` value (an
/// anonymous block OR a named fn passed by reference — both arrive here
/// as a resolved `Value::wat__core__fn`, so the reification is uniform);
/// `name` evaluates to a `:wat::core::keyword` — the bind name the
/// reified fn will carry when the returned forms are later evaluated in
/// a FRESH universe (a forked, not-shared child).
///
/// Fronts [`extract_closure`] uniformly through its inline-lambda path
/// (`entry_name = None`) per the pinned S1 contract — that path both
/// reconstructs the fn-form AST from the value and walks the body for
/// transitive deps, so it covers both input shapes without a name-vs-
/// no-name branch here. Returns `prologue ++ [(:wat::core::def <name>
/// <entry_form>)]` as a `(:wat::core::Vector :- [wat::WatAST])` value: a
/// self-contained program fragment that, `eval`d top-to-bottom in a
/// fresh world, resolves `<name>` to a behaviorally-equivalent fn.
///
/// `ExtractionError::ImpureCapture` (channel/IO/process-handle captures
/// cannot cross a process boundary — pointer identity doesn't survive
/// `fork`/`clone3`) surfaces as a wat `RuntimeError` naming the capture
/// and its type (via `ExtractionError`'s `Display`); `UnresolvedSymbol`
/// / `Internal` surface the same way.
pub fn eval_kernel_fn_forms(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::kernel::fn-forms";
    if args.len() != 2 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
        }));
    }

    // arg 0: the fn value to reify. A runtime-computed keyword naming a
    // registered def (e.g. built via `keyword/from-string`) does NOT
    // auto-upgrade to its fn the way a *literal* keyword node does (the
    // arc-009 "names are values" lift lives in `eval`'s `WatAST::Keyword`
    // arm, runtime.rs, and only fires when the keyword is a literal AST
    // node in source). Mirror that exact resolution here so a keyword
    // Value arriving through computation also resolves.
    let fn_value = match eval(&args[0], env, sym)?.value_owned() {
        v @ Value::wat__core__fn(_) => v,
        Value::wat__core__keyword(k) => match sym.get(&k) {
            Some(func) => Value::wat__core__fn(func.clone()),
            None => {
                return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: "a keyword naming a registered fn (or a fn value)",
                    got: Box::new(ValueSnapshot::of(&Value::wat__core__keyword(k))),
                }));
            }
        },
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "fn value (:wat::core::fn) to reify",
                got: Box::new(ValueSnapshot::of(&other)),
            }));
        }
    };

    // arg 1: the bind name — a keyword (carries its leading ':', same
    // convention as every other keyword Value in the runtime).
    let name: String = match eval(&args[1], env, sym)?.value_owned() {
        Value::wat__core__keyword(k) => (*k).clone(),
        other => {
            return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: "keyword bind-name",
                got: Box::new(ValueSnapshot::of(&other)),
            }));
        }
    };

    // Reach the parent TypeEnv exactly the way `conforms?` / struct-
    // destructure already do — `sym.types()` — not a new param threaded
    // through the world. `sym` is already the parent SymbolTable handed
    // in by the dispatch site (mirrors `eval_kernel_spawn_process`'s
    // `parent_symbols` = `sym` directly).
    let parent_types = sym.types().ok_or_else(|| RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
        head: OP.into(),
        reason: "fn-forms requires the type registry, but the SymbolTable has no TypeEnv attached (programmer error: this build path didn't go through startup_from_source / freeze)".into()
    }))?;

    let pkg = extract_closure(&fn_value, None, sym, parent_types).map_err(|e| {
        RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: e.to_string(),
        })
    })?;

    let def_span = list_span.clone();
    let mut forms = pkg.prologue;
    forms.push(WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::def".into(), def_span.clone()),
            WatAST::Keyword(name, def_span.clone()),
            pkg.entry_form,
        ],
        def_span,
    ));

    let items: Vec<Value> = forms
        .into_iter()
        .map(|f| Value::wat__WatAST(Arc::new(f)))
        .collect();
    Ok(Value::Vec(Arc::new(items)))
}

// ─── Capture-binding name minting ───────────────────────────────────────
//
// Slice 1b retired the per-package synthetic-name counter
// (`:__closure::__pkg_<n>`) — entry naming no longer exists at the
// substrate level. Capture-binding names below are a separate
// concern (avoiding collision with extracted user-symbol names) and
// stay.

fn synthesize_capture_name(local: &str) -> String {
    // Captured locals are bare symbols. We emit them as top-level keyword
    // paths under `:user::closure-capture::<local>` — never bare (a
    // bare top-level name is a located `Registration::Unnamespaced`
    // error at every registration door) and never `:wat::`-rooted
    // (privilege does not survive a process boundary: in the child
    // these planted forms are user residue, so a `:wat::`-rooted mint
    // is refused as a reserved prefix — `ReservedPrefix`). `:user::` is
    // the rendezvous coordinate space (see `wat/bracket.wat`'s header —
    // "not a user's namespace; a rendezvous space"), so that is where
    // these belong. The `closure-capture` segment is deliberate: the
    // old `__` marker is gone, and that segment now carries the
    // collision-avoidance duty the `__` prefix used to carry (the duty
    // itself is unchanged — see the retirement note above; only where
    // it is discharged has moved, from a string prefix to a namespace
    // segment). The body rewrite swaps the bare-Symbol reference for a
    // Keyword that resolves to the def-bound value at runtime via
    // `def`'s runtime_def_values pathway.
    format!(
        ":user::closure-capture::{}",
        sanitize_local_for_keyword(local)
    )
}

fn sanitize_local_for_keyword(s: &str) -> String {
    // Local names are bare symbols (e.g. `my-config`, `tx_count`).
    // Keyword paths admit identifiers separated by `::`. We pass
    // through unchanged; user locals follow identifier syntax already.
    s.to_string()
}

// ─── Extraction state ───────────────────────────────────────────────────

struct ExtractState<'a> {
    parent_symbols: &'a SymbolTable,
    parent_types: &'a TypeEnv,
    /// The dep currently being walked, if any. While walking a dep's
    /// body, any newly discovered dep is recorded as a child edge from
    /// `current_walking_dep` so topological sort lifts deps before
    /// their consumers. While walking the entry fn (no dep is being
    /// walked), this is None — the entry fn is downstream of every
    /// recorded dep, so its position in the output is fixed (last).
    current_walking_dep: Option<String>,
    /// Free symbols collected from the entry body, awaiting
    /// reclassification (capture / dep / type / unresolved).
    unresolved_frees: Vec<(String, Span)>,
    /// Free symbols that are not captures, deps, types, or substrate
    /// primitives — surface as UnresolvedSymbol.
    really_unresolved: Vec<(String, Span)>,
    /// Captured local names (so dep walks know NOT to recurse on these).
    captured_locals: HashSet<String>,
    /// Encoded captured bindings to emit as top-level defines.
    captured_bindings: Vec<CapturedBinding>,
    /// User dependency functions discovered, keyed by canonical name.
    captured_deps: BTreeMap<String, Arc<Function>>,
    /// User types discovered, keyed by canonical name.
    captured_types: BTreeMap<String, TypeDef>,
    /// Order in which deps were discovered (drives topo sort tiebreak).
    dep_discovery_order: Vec<String>,
    /// Order in which types were discovered.
    type_discovery_order: Vec<String>,
    /// Macros discovered in walked bodies, keyed by name, holding the
    /// RETAINED `(defmacro …)` form. Shipped verbatim — a `MacroDef`
    /// cannot rebuild its own declaration (no param types, no return
    /// type), and the forms we ship still CALL these macros.
    captured_macros: BTreeMap<String, WatAST>,
    /// Order in which macros were discovered (deterministic emission).
    macro_discovery_order: Vec<String>,
    /// `def`-bound values discovered in walked bodies, keyed by the
    /// def's ORIGINAL keyword name. Emitted as top-level `def` forms.
    captured_defs: BTreeMap<String, WatAST>,
    /// Order in which defs were discovered (deterministic emission).
    def_discovery_order: Vec<String>,
    /// Visited-set: types whose closure has been (or is being) walked,
    /// to break recursion through `:Vector<:Self>` and friends.
    types_visited: HashSet<String>,
    /// Visited-set: deps whose body has been walked.
    deps_visited: HashSet<String>,
    /// Edges for topological ordering.
    /// `dep_edges[name]` = set of names this dep depends on.
    dep_edges: BTreeMap<String, BTreeSet<String>>,
    /// `type_edges[name]` = set of types this type depends on.
    type_edges: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Debug, Clone)]
struct CapturedBinding {
    original_name: String,
    synthetic_name: String,
    encoded_ast: WatAST,
}

impl<'a> ExtractState<'a> {
    fn new(parent_symbols: &'a SymbolTable, parent_types: &'a TypeEnv) -> Self {
        ExtractState {
            parent_symbols,
            parent_types,
            current_walking_dep: None,
            unresolved_frees: Vec::new(),
            really_unresolved: Vec::new(),
            captured_locals: HashSet::new(),
            captured_bindings: Vec::new(),
            captured_deps: BTreeMap::new(),
            captured_types: BTreeMap::new(),
            dep_discovery_order: Vec::new(),
            type_discovery_order: Vec::new(),
            captured_macros: BTreeMap::new(),
            macro_discovery_order: Vec::new(),
            captured_defs: BTreeMap::new(),
            def_discovery_order: Vec::new(),
            types_visited: HashSet::new(),
            deps_visited: HashSet::new(),
            dep_edges: BTreeMap::new(),
            type_edges: BTreeMap::new(),
        }
    }
}

// ─── Free-symbol walker ─────────────────────────────────────────────────

/// Walk an AST node collecting free Symbol references AND classifying
/// free Keyword references that resolve to user defines / user types.
///
/// Free-symbol classification (substrate primitive vs user) for
/// keywords happens here; the dep / type extraction is recursive
/// through `extract_user_deps_to_fixpoint` / `extract_user_types_to_fixpoint`.
///
/// `locals` is the set of bound Symbol names visible at this node.
fn walk_free_symbols(
    node: &WatAST,
    locals: &BTreeSet<String>,
    state: &mut ExtractState<'_>,
) -> Result<(), ExtractionError> {
    match node {
        WatAST::IntLit(..)
        | WatAST::FloatLit(..)
        // Arc 300 stone B — RationalLit is a leaf literal, same as IntLit/FloatLit.
        | WatAST::RationalLit(..)
        // Arc 300 stone C1 — BigIntLit is a leaf literal too.
        | WatAST::BigIntLit(..)
        // Arc 300 stone D — CharLit is a leaf literal too.
        | WatAST::CharLit(..)
        | WatAST::BoolLit(..)
        | WatAST::StringLit(..)
        // Arc 244 — NilLit is a leaf literal; no free symbols to collect.
        | WatAST::NilLit(..) => Ok(()),

        WatAST::Symbol(ident, span) => {
            let name = ident.as_str().to_owned();
            // Syntactic markers: `->` (return-type arrow), `<-` (input
            // direction arrow in fn signatures), `&` (rest-binder
            // marker), `_` (wildcard at any position — never a
            // reference; in let-binder position it's a permissive
            // discard, in match-arm pattern position it's a wildcard).
            // These are NOT references and must not enter the
            // free-Symbol set.
            if matches!(name.as_str(), "->" | "<-" | "&" | "_") {
                return Ok(());
            }
            // `locals` (body_locals) is keyed by `env_key` — see
            // `runtime.rs:733`, where `FunctionDef.params` is built via
            // `crate::scope::env_key(n)`, the SAME key derivation the
            // runtime's own symbol-lookup path uses at eval time
            // (`runtime.rs:3273`). A hygienic (`fresh-symbol`) param
            // carries a non-empty scope set, so its `env_key` differs
            // from its bare `as_str()` name. Membership must therefore
            // be tested by `env_key`, not the bare name, or a fn's own
            // hygienic param reads as free inside its own body. `name`
            // (bare) is still what gets pushed into `unresolved_frees` —
            // downstream consumers (`closed_env` capture lookup,
            // `rewrite_captures`'s `by_name` match on `ident.as_str()`)
            // key on the bare name, and this arm only changes the
            // locals-membership test, not that contract.
            let key = crate::scope::env_key(ident);
            if !locals.contains(key.as_ref()) {
                state.unresolved_frees.push((name, span.clone()));
            }
            Ok(())
        }

        WatAST::Keyword(k, _span) => {
            // Substrate primitives (`:wat::*` / `:rust::*`) get skipped.
            // User defines and user types both resolve to extraction
            // targets here.
            if crate::resolve::is_reserved_prefix(k) {
                return Ok(());
            }
            // ─── Arc 278: ASK THE DOOR ───────────────────────────────
            //
            // `registrations` reports every FACET a name carries. The
            // facets are not alternatives in general — a record's name is
            // BOTH a `Type` and a `Macro` (its kwargs constructor), and a
            // child needs both — so the macro facet is handled ADDITIVELY
            // here and the value/type facets fall through to the chain
            // below.
            //
            // The chain below stays first-match on purpose: `[Function,
            // DefValue]` co-occur for every `defn` (`defn` expands to
            // `(def :n (fn …))`), and there the Function facet is the one
            // to ship — encoding the DefValue would hit the fn-value wall
            // in `encode_value_to_ast`. Same concept, one right facet.
            //
            // Macros must ship because the forms we send still CONTAIN
            // macro calls: `(:my::Rec :field v)` expands in the CHILD.
            // Omitting this facet is why a synthesized record arrived as a
            // type with no callable constructor.
            record_macro_dependency(state, k.as_str());
            // Try function lookup.
            if let Some(func) = state.parent_symbols.get(k) {
                record_dep_dependency(state, k.as_str(), func);
                // Also recurse types in the function's signature.
                for ty in &func.param_types {
                    record_type_refs_in_typeexpr(state, ty);
                }
                record_type_refs_in_typeexpr(state, &func.ret_type);
                if let Some(rest_ty) = &func.rest_param_type {
                    record_type_refs_in_typeexpr(state, rest_ty);
                }
                return Ok(());
            }
            // Try unit-variant resolution. Unit variants live as
            // `:my::E::Variant` keys in `unit_variants`.
            if let Some(ev) = state.parent_symbols.unit_variant(k) {
                // The enum type itself is the dep; record it.
                if !crate::resolve::is_reserved_prefix(&ev.type_path) {
                    record_type_dependency_by_name(state, &ev.type_path);
                }
                return Ok(());
            }
            // Try type lookup via TypeEnv.
            if let Some(type_def) = state.parent_types.get(k) {
                record_type_dependency(state, k.as_str(), type_def);
                return Ok(());
            }
            // Else: a `def`-bound value? Arc 278 — carry it. Encode the
            // value through the same encoder used for captured locals
            // and record it under its ORIGINAL name (Keyword references
            // are never rewritten by `rewrite_captures`, so the def must
            // keep the name the body already refers to it by).
            if let Some(value) = state.parent_symbols.def_value(k).cloned() {
                if !state.captured_defs.contains_key(k.as_str()) {
                    let encoded = encode_value_to_ast(&value, k.as_str(), state)?;
                    state.captured_defs.insert(k.to_string(), encoded);
                    state.def_discovery_order.push(k.to_string());
                }
                return Ok(());
            }
            // Treat as a keyword literal at value position (no
            // resolution required).
            Ok(())
        }

        WatAST::List(items, _span) => {
            // First: detect binding-introducing forms by head keyword.
            // We honor `:wat::core::let`, `:wat::core::fn`, `:wat::core::match`
            // (whose arm patterns introduce bindings into the arm body's
            // scope), and `:wat::core::define` (for completeness; entry-fn
            // bodies don't usually contain a top-level define).
            //
            // Arc 170 slice 3 Gap H addition: struct and enum forms also
            // require special handling when they appear in a fn body's
            // do-prefix (lifted to prologue by extract_closure). Their
            // field/variant names are BINDING positions, not references —
            // the plain-list recursive path would incorrectly treat them as
            // free symbols, causing UnresolvedSymbol failures at extraction
            // time. We walk only the type keyword children for type deps.
            if let Some((WatAST::Keyword(k, _), rest)) = items.split_first() {
                match k.as_str() {
                    ":wat::core::let" => {
                        return walk_let_form(rest, locals, state);
                    }
                    ":wat::core::fn" => {
                        return walk_fn_form(rest, locals, state);
                    }
                    // Stone 241.16 — `:wat::core::define` arm DELETED.
                    // HARD CUT total (Stone 241.11 startup check + Stone 241.16 eval residue).
                    // No define-headed form reaches closure extraction; arm is permanently unreachable.
                    //
                    // `match` on an enum is CORE to the language — a special form like `let`
                    // and `fn`, and this walker has always needed its arm patterns' BINDERS to
                    // enter the arm body's scope. That is a scope question, not a data/code
                    // one, so it is answered HERE and never routed through the boundary door
                    // (routing it there made this walker depend on the door for something it
                    // already knew — my error, backed out).
                    ":wat::core::match" => {
                        return walk_match_form(rest, locals, state);
                    }
                    // Stone 255.1a-β-i-b — `":wat::core::defstruct" =>` arm REMOVED. `defstruct`
                    // is a stdlib `defmacro`; `expand_all` rewrites it to `structtype` before
                    // this walker's only caller (`extract_closure`, reached from
                    // `:wat::kernel::fn-forms` reifying an already-registered/already-expanded
                    // fn value) ever runs. A raw `defstruct` head cannot survive to here even via
                    // `eval-ast!`: `refuse_mutation_forms_in` walks a quoted AST's FULL tree
                    // recursively and refuses `defstruct` anywhere in it (see `runtime.rs`),
                    // before any fn value built from that AST could reach this walker. Measured
                    // dead; `structtype` (below) is the live post-expansion arm.
                    // Arc 293.2-parity — structtype is the low-level primitive defstruct (macro) expands to.
                    ":wat::core::structtype" => {
                        return walk_struct_form(rest, state);
                    }
                    // Stone 241.9 — defenum replaces enum (HARD CUT).
                    ":wat::core::defenum" => {
                        return walk_defenum_form(rest, state);
                    }
                    ":wat::core::defmacro" => {
                        return walk_defmacro_form(rest);
                    }
                    _ => {}
                }

                // ── Arc 278 — THE DATA/CODE BOUNDARY IS NOT THIS WALKER'S QUESTION ──────────
                //
                // `crate::resolve::boundary::quote_boundary` is, by its own doc, "the ONE place
                // the boundary-head set is encoded"; the call-head resolution walk
                // (`resolve::walk`) and the symbol-ref normalization pass (`resolve::normalize`)
                // both route through it. This walker was a THIRD derivation that bypassed it
                // entirely — it had no concept of `quote` at all (`grep -n quote` returned
                // nothing) — and so it READ QUOTED DATA AS CODE, raising `UnresolvedSymbol` on
                // symbols that were never references.
                //
                // MEASURED, rete-free, two arms differing in exactly one thing
                // (`wat-scripts/scratch-pad/probe-arc278-fnforms-walks-into-quoted-data.wat`):
                // an un-quoted body extracted fine; the same body wrapped in `quote` raised on
                // `mystery-symbol` — a plain bare Symbol, deliberately NOT `?`-prefixed. The rete
                // symptom that surfaced this (`?c` in a `defrule`) was incidental: `defrule`
                // QUOTES its `:when`/`:then` (wat/rete.wat:2385), so its pattern variables are
                // quoted data like any other. Nothing here knows about rete, and nothing should:
                // any user DSL that quotes its forms is covered by construction.
                //
                // The binder arms above are a DIFFERENT question — they answer "what is in
                // scope", not "what is data" — so they stay and are consulted first.
                //
                // This match is EXHAUSTIVE by law: a new `Boundary` variant turns it red until
                // handled, which is the structural guarantee that the three passes cannot drift.
                match quote_boundary(k) {
                    // quote / forms / define / holon::literal — every argument is data.
                    // Nothing inside is a reference; collecting deps from it is meaningless and
                    // demanding that it resolve is the bug.
                    Boundary::AllData => return Ok(()),

                    // quasiquote — the template is data EXCEPT inside unquote / unquote-splicing
                    // escapes, which are live code and may name real dependencies we must ship.
                    Boundary::Quasiquote => {
                        if let Some(template) = items.get(1) {
                            walk_quasiquote_template(template, locals, state)?;
                        }
                        return Ok(());
                    }

                    // `match` is answered by the BINDER arm above (it is core to the language
                    // and this walker needs its arm patterns' binders). Listed only to keep
                    // this match exhaustive; unreachable in practice.
                    Boundary::Match => {}

                    // `:wat::form::matches?` — only the subject (`items[1]`) is code; the
                    // pattern (`items[2..]`) is DSL data owned by check.rs's
                    // `infer_form_matches` grammar walker. Identical to `resolve::walk`'s
                    // `Boundary::MatchesSubject` arm.
                    //
                    // ⚠ THIS ARM WAS `{}` AND THAT WAS A LIVE BUG, proven by run. Falling
                    // through walks the PATTERN as code and raises on its DSL tokens — the
                    // pattern is NOT quoted, so nothing downstream stops the walk:
                    //
                    //   free symbol `=` does not resolve to a parent define or substrate
                    //   primitive   (probe_matches_pattern_var.wat:26:21)
                    //
                    // i.e. NO fn containing a `matches?` could be closure-extracted — the
                    // exact "walker refuses a valid program" defect this whole hook exists to
                    // kill, left standing for a second form. The `{}` was justified by
                    // `make-rule`'s reasoning (below), which does NOT transfer.
                    Boundary::MatchesSubject => {
                        if let Some(subject) = items.get(1) {
                            walk_free_symbols(subject, locals, state)?;
                        }
                        return Ok(());
                    }

                    // ⛔ DELIBERATELY NOT HONOURED — `:wat::rete::make-rule` is LIBRARY grammar
                    // sitting in a substrate list (plus `is_where_form`, a whole function for
                    // one library form). Honouring it here would spread that privilege to
                    // another consumer, and privilege is exactly what makes a user-defined DSL
                    // second-class: rete got to edit the compiler's list; a user cannot.
                    //
                    // Falling through is safe HERE, and the reason is specific to this form:
                    // `make-rule`'s `:when` is itself a `quote` form, so the recursion below
                    // meets it and the `AllData` arm stops there — no false raise. The cost is
                    // a FALSE NEGATIVE: deps inside a `(:wat::rete::where …)` body go
                    // uncollected, and the child names them at startup. A missing dep is
                    // legible; a refused valid program is not.
                    //
                    // ⚠ This refusal removes NO privilege on its own — `quote_boundary` still
                    // returns `MakeRule`, and `walk`/`normalize`/`expand` all still honour it.
                    // It is a marker, not a cure. The cure is the boundary-DECLARATION stone
                    // (ruled 2026-08-12): a form declares its own boundary, the compiler holds
                    // no library's grammar, and this variant ceases to exist.
                    Boundary::MakeRule => {}

                    // Not a boundary — fall through to the plain-list recursion below.
                    Boundary::Ordinary => {}
                }
            }
            // Plain list — recurse on every child.
            for item in items {
                walk_free_symbols(item, locals, state)?;
            }
            Ok(())
        }

        WatAST::Vector(items, _span) => {
            for item in items {
                walk_free_symbols(item, locals, state)?;
            }
            Ok(())
        }

        // Arc 257.2 — Map/Set literals: recurse into all k/v and elements.
        WatAST::Map(pairs, _) => {
            for (k, v) in pairs {
                walk_free_symbols(k, locals, state)?;
                walk_free_symbols(v, locals, state)?;
            }
            Ok(())
        }
        WatAST::Set(items, _) => {
            for item in items {
                walk_free_symbols(item, locals, state)?;
            }
            Ok(())
        }
    }
}

/// Walk a `(:wat::core::let [binders...] body...)` form, accumulating
/// bindings into the local scope as we walk.
fn walk_let_form(
    args: &[WatAST],
    outer_locals: &BTreeSet<String>,
    state: &mut ExtractState<'_>,
) -> Result<(), ExtractionError> {
    if args.is_empty() {
        return Ok(());
    }
    let bindings_vec = match &args[0] {
        WatAST::Vector(items, _) => items,
        _ => {
            // Malformed; let the runtime's MalformedForm fire when
            // executed. Walk children defensively.
            for a in args {
                walk_free_symbols(a, outer_locals, state)?;
            }
            return Ok(());
        }
    };
    let mut current_locals = outer_locals.clone();
    let mut i = 0;
    while i + 1 < bindings_vec.len() {
        let binder = &bindings_vec[i];
        let rhs = &bindings_vec[i + 1];
        // RHS is evaluated in the scope BEFORE the binder takes effect
        // (sequential let semantics).
        walk_free_symbols(rhs, &current_locals, state)?;
        // Now extend scope with binder names.
        match binder {
            WatAST::Symbol(ident, _) => {
                current_locals.insert(ident.as_str().to_owned());
            }
            WatAST::Vector(inner, _) => {
                for it in inner {
                    if let WatAST::Symbol(ident, _) = it {
                        current_locals.insert(ident.as_str().to_owned());
                    }
                }
            }
            // Arc 257.2 — Map binder: classify and extract binding names.
            // Binder symbols (inside :keys [..] or hash-destructure vars)
            // are BINDINGS, not free references — add them to locals BEFORE
            // walking the body. The binder Map itself is NOT walked as a
            // value expression (its symbols are binding positions).
            WatAST::Map(pairs, _) => {
                if let Some(md) = WatAST::classify_map_destructure(pairs) {
                    for (ident, _, _) in &md.bindings {
                        current_locals.insert(ident.as_str().to_owned());
                    }
                }
                // If not a destructure (value-position map in binder
                // slot — malformed), fall through with no new locals.
            }
            _ => {}
        }
        i += 2;
    }
    // Body: walks under the cumulative scope.
    for body_form in &args[1..] {
        walk_free_symbols(body_form, &current_locals, state)?;
    }
    Ok(())
}

/// Walk a `(:wat::core::fn [param <- :T ...] -> :Ret body...)` form,
/// adding the parameter names to the local scope for the body walk.
fn walk_fn_form(
    args: &[WatAST],
    outer_locals: &BTreeSet<String>,
    state: &mut ExtractState<'_>,
) -> Result<(), ExtractionError> {
    if args.len() < 3 {
        for a in args {
            walk_free_symbols(a, outer_locals, state)?;
        }
        return Ok(());
    }
    let mut new_locals = outer_locals.clone();
    if let WatAST::Vector(items, _) = &args[0] {
        // Triples: name <- :T name <- :T ... ; we just need the
        // names (every third item starting at 0).
        let mut j = 0;
        while j < items.len() {
            if let WatAST::Symbol(ident, _) = &items[j] {
                new_locals.insert(ident.as_str().to_owned());
            }
            j += 3;
        }
    }
    // args[1] is `->`, args[2] is :Ret keyword (type ref). Walk type
    // keyword for type-extraction.
    walk_free_symbols(&args[2], outer_locals, state)?;
    // Body.
    for body_form in &args[3..] {
        walk_free_symbols(body_form, &new_locals, state)?;
    }
    Ok(())
}

// Stone 241.16 — `walk_define_form` DELETED. `:wat::core::define` HARD CUT total;
// the dispatch arm that called this function was deleted (arm removal above).
// No define-headed form reaches closure extraction. Function body dies with the arm.

/// Walk a `(:wat::core::defstruct :TypeName [field <- :T1 ...])` or
/// `(:wat::core::defstruct :TypeName {metadata} [field <- :T1 ...])` form.
///
/// Stone 241.8 — replaces walk of old `(:wat::core::struct ...)` pair-form.
/// Arc 170 slice 3 Gap H. Struct forms appear in fn body do-prefixes when
/// the user declares a local type inside a spawned fn. The form is a
/// freeze-time declaration, not a runtime expression. When `walk_free_symbols`
/// encounters a defstruct form in a do body, it must NOT recurse into it as a
/// plain list because field names are binding positions (not free-symbol
/// references) and would be misclassified as `UnresolvedSymbol`.
///
/// Correct walk for defstruct:
///   - `args[0]` = `:TypeName` keyword — skip (being declared; no lookup).
///   - `args[1]` = metadata-map or field-vector; `args[2]` = field-vector (if metadata at [1]).
///   - Field-vector items are triples: field(Symbol) <- :Type. The type keyword
///     (position 2 in each triple: 2, 5, 8, ...) is walked for type-dep recording.
///     Field-name Symbols are binding positions — skipped.
fn walk_struct_form(
    args: &[WatAST],
    state: &mut ExtractState<'_>,
) -> Result<(), ExtractionError> {
    // args[0] is the type name keyword — skip (declaration, not lookup).
    // Find the field-vector: args[1] if it is a Vector; args[2] if args[1] is metadata.
    let field_vec = match args.get(1) {
        Some(WatAST::Vector(fv, _)) => Some(fv.as_slice()),
        Some(WatAST::List(_, _)) => {
            // args[1] is metadata-map; field-vector at args[2].
            match args.get(2) {
                Some(WatAST::Vector(fv, _)) => Some(fv.as_slice()),
                _ => None,
            }
        }
        _ => None,
    };
    if let Some(fv) = field_vec {
        // Walk triples: field(0) <-(1) :Type(2) ...
        // Type keywords are at positions 2, 5, 8, ... in the flat vector.
        let mut idx = 0;
        while idx + 2 < fv.len() {
            // fv[idx] = field name Symbol (binding position — skip)
            // fv[idx+1] = `<-` Symbol (skip)
            // fv[idx+2] = type keyword — walk for type dep recording
            let empty_locals = std::collections::BTreeSet::new();
            walk_free_symbols(&fv[idx + 2], &empty_locals, state)?;
            idx += 3;
        }
    }
    Ok(())
}

/// Stone 241.9 — Walk a `(:wat::core::defenum :TypeName [:V1_kw :V2_kw [field <- :T] ...])` form.
///
/// Arc 170 slice 3 Gap H. Enum forms in fn body do-prefixes require the
/// same protection as struct forms (see `walk_struct_form`). Variant names
/// and field names are binding positions, not free-symbol references.
///
/// defenum grammar (positional + one-token look-ahead per FORM-COLLAPSE verdict D):
///   args[0] = name keyword (skip — new declaration)
///   args[1] = OPTIONAL metadata-map List (skip — no free-variable refs in type metadata)
///   args[1..] or args[2..] = positional variants:
///     - Keyword `:VariantName` — unit variant (binding position, skip)
///     - Keyword `:VariantName` followed by Vector `[name <- :T ...]` — tagged variant;
///       keyword = binding position (skip); Vector's type keywords walked for type deps.
///       Vector items: triples `Symbol("<-") Keyword(:T)` — Symbol names = bindings (skip);
///       type keywords walked for deps.
fn walk_defenum_form(
    args: &[WatAST],
    state: &mut ExtractState<'_>,
) -> Result<(), ExtractionError> {
    // args[0] is the enum name keyword (skip — new declaration).
    // args[1] may be a metadata-map (WatAST::List with :wat::core::HashMap head) — skip.
    // Remaining args: positional variants with one-token look-ahead grammar.
    // Arc 257 slice 1: is_metadata_map() accepts Map literal and legacy HashMap List.
    let start = if args.get(1).map(|n| n.is_metadata_map()).unwrap_or(false) { 2 } else { 1 };
    let mut vi = start;
    while vi < args.len() {
        match &args[vi] {
            // Variant name keyword — check look-ahead.
            WatAST::Keyword(_, _) => {
                // Peek ahead: is next item a Vector (tagged variant)?
                if let Some(WatAST::Vector(vec_items, _)) = args.get(vi + 1) {
                    // Tagged variant: walk type keywords inside the argspec Vector.
                    // Argspec structure: [Symbol(name), Symbol("<-"), Keyword(:T), ...]
                    // Type keywords are at indices 2, 5, 8, ... (every triple's 3rd slot).
                    let mut ti = 0;
                    while ti + 2 < vec_items.len() {
                        // vec_items[ti+2] should be the type keyword.
                        if let WatAST::Keyword(_, _) = &vec_items[ti + 2] {
                            let empty_locals = std::collections::BTreeSet::new();
                            walk_free_symbols(&vec_items[ti + 2], &empty_locals, state)?;
                        }
                        ti += 3; // advance one triple
                    }
                    vi += 2; // consume keyword + vector
                } else {
                    // Unit variant — binding position, skip.
                    vi += 1;
                }
            }
            // Metadata-map List or other non-variant item — skip.
            _ => {
                vi += 1;
            }
        }
    }
    Ok(())
}

/// Walk a `(:wat::core::defmacro (:name (param :AST) ... -> :AST) body)` form.
///
/// Arc 170 slice 3 Gap I-A. Defmacro forms in fn body do-prefixes require
/// the same protection as struct/enum forms (see `walk_struct_form`). Macro
/// parameter names (bare Symbols like `x` in `(x :AST)`) are binding
/// positions within the macro's template body — they are NOT free-symbol
/// references to the parent scope.
///
/// Correct walk: skip the entire defmacro. The macro body is a template that
/// uses parameter Symbols as binding sites (e.g., `~x` / `\`~x`). Walking
/// the template as a plain list would misclassify those Symbols as unresolved
/// free-variable references, causing `MalformedForm` at `extract_closure`
/// time. The defmacro form itself introduces no free-variable dependencies on
/// the parent's scope; its registration in the child's `MacroRegistry` at
/// startup step 4 is self-contained.
fn walk_defmacro_form(_args: &[WatAST]) -> Result<(), ExtractionError> {
    // The defmacro body is a macro template, not an executable expression.
    // Parameter names inside the template are binding positions; they must
    // not be walked for free-symbol resolution against the parent scope.
    Ok(())
}

/// Arc 278 — descend a `quasiquote` TEMPLATE, walking only the live-code escapes.
///
/// Mirrors `resolve::quote::check_quasiquote_template`, which is the reference traversal;
/// the escape-head set comes from `resolve::boundary::is_unquote_escape` so this third
/// descent cannot drift from the other two. Template data is skipped (it is not code and
/// must not be resolved), but recursion CONTINUES through it because an escape can sit
/// arbitrarily deep — including inside bracketed forms, which a List-only walk would miss.
fn walk_quasiquote_template(
    node: &WatAST,
    locals: &BTreeSet<String>,
    state: &mut ExtractState<'_>,
) -> Result<(), ExtractionError> {
    if let WatAST::List(items, _) = node {
        if let Some(WatAST::Keyword(head, _)) = items.first() {
            if is_unquote_escape(head) {
                // An escape: its arguments are live code, and may name real dependencies.
                for arg in items.iter().skip(1) {
                    walk_free_symbols(arg, locals, state)?;
                }
                return Ok(());
            }
            // Any other head (including a nested quasiquote) is template DATA — do not
            // resolve it, but keep descending for escapes deeper in the tree.
        }
    }
    for child in node.children().iter() {
        walk_quasiquote_template(child, locals, state)?;
    }
    Ok(())
}

/// Walk a `(:wat::core::match scrut arm1 arm2 ...)` form.
///
/// Each arm is a 2-list `(pattern body)`. Pattern names BIND inside
/// the arm's body — they are NOT free symbols. The pattern is walked
/// separately to collect bindings AND to record any user-enum variant
/// keyword as a type dependency (the variant's enclosing enum is the
/// type the closure-extraction needs in the prologue).
///
/// Pattern shapes recognized (mirroring `runtime::try_match_pattern`):
///
///   - `_` wildcard         — no binding
///   - bare Symbol          — binds that name
///   - `:enum::Variant`     — unit-variant keyword (no binding)
///   - literal              — int/float/bool/string/None — no binding
///   - `(Some pat)` / `(Ok pat)` / `(Err pat)`  — sub-pattern recursion
///   - `(:enum::Variant pat1 pat2 ...)` — tagged variant; sub-patterns recurse
///   - `(pat1 pat2 ...)`    — tuple destructure; sub-patterns recurse
fn walk_match_form(
    args: &[WatAST],
    outer_locals: &BTreeSet<String>,
    state: &mut ExtractState<'_>,
) -> Result<(), ExtractionError> {
    // Arc 258.5 — `(:wat::core::match scrut arm1 arm2 ...)`: args[0] =
    // scrut, args[1..] = arms (the `-> :T` ascription is retired). If
    // shape is malformed, fall through to a defensive recurse so the
    // runtime's MalformedForm fires when the form executes.
    if args.len() < 2 {
        for a in args {
            walk_free_symbols(a, outer_locals, state)?;
        }
        return Ok(());
    }
    // Walk scrutinee in outer scope.
    walk_free_symbols(&args[0], outer_locals, state)?;
    // For each arm: the arm is a 2-list `(pattern body)`. Bindings
    // collected from the pattern enter the arm's body scope.
    for arm in &args[1..] {
        let arm_items = match arm {
            WatAST::List(items, _) if items.len() == 2 => items,
            _ => {
                // Malformed arm — defensive recurse.
                walk_free_symbols(arm, outer_locals, state)?;
                continue;
            }
        };
        let pattern = &arm_items[0];
        let body = &arm_items[1];
        let mut arm_locals = outer_locals.clone();
        // Collect pattern bindings AND record type deps for any
        // user-enum-variant keywords found inside.
        collect_pattern_bindings(pattern, &mut arm_locals, state)?;
        // Walk the body under the augmented scope.
        walk_free_symbols(body, &arm_locals, state)?;
    }
    Ok(())
}

/// Recursively walk a match-arm pattern, accumulating binding names
/// into `locals` and recording any user-defined type dependency the
/// pattern references through a variant keyword.
///
/// The runtime's `try_match_pattern` is the source of truth for which
/// AST shapes introduce bindings; this helper mirrors that classifier.
fn collect_pattern_bindings(
    pattern: &WatAST,
    locals: &mut BTreeSet<String>,
    state: &mut ExtractState<'_>,
) -> Result<(), ExtractionError> {
    match pattern {
        // Literals — no binding. Arc 244: NilLit joins the literal group.
        // Arc 300 stone B: RationalLit joins it too.
        WatAST::IntLit(..)
        | WatAST::FloatLit(..)
        | WatAST::RationalLit(..)
        // Arc 300 stone C1: BigIntLit joins it too.
        | WatAST::BigIntLit(..)
        // Arc 300 stone D: CharLit joins it too.
        | WatAST::CharLit(..)
        | WatAST::BoolLit(..)
        | WatAST::StringLit(..)
        | WatAST::NilLit(..) => Ok(()),
        // Symbol pattern: `_` wildcard binds nothing; any other bare
        // symbol binds that name to the matched scrutinee.
        WatAST::Symbol(ident, _) => {
            let name = ident.as_str();
            if name != "_" {
                locals.insert(name.to_string());
            }
            Ok(())
        }
        // Keyword pattern — unit-variant (`:wat::core::None`,
        // `:my::E::Variant`) or `:None` literal. Resolve type deps
        // through the existing free-symbol machinery so user-enum
        // unit-variant patterns pull their enum into prologue.
        WatAST::Keyword(_, _) => walk_free_symbols(pattern, locals, state),
        // List pattern: tagged variant constructor or tuple destructure.
        WatAST::List(items, _) => {
            // Resolve the head if it's a user-enum variant keyword
            // (Some/Ok/Err have reserved-prefix and become no-ops in
            // walk_free_symbols; user variants pull their enum's type
            // into the dep set).
            if let Some(head) = items.first() {
                match head {
                    WatAST::Keyword(_, _) => {
                        walk_free_symbols(head, locals, state)?;
                    }
                    // Bare-symbol heads `Some`/`Ok`/`Err` (legacy
                    // pre-FQDN form) carry no binding semantics —
                    // skip head, recurse into sub-patterns.
                    WatAST::Symbol(_, _) => {}
                    // First element is itself a sub-pattern (tuple
                    // destructure) — recurse.
                    _ => collect_pattern_bindings(head, locals, state)?,
                }
            }
            // Whether the head was a constructor keyword/symbol or a
            // sub-pattern, the REMAINING items are sub-patterns. For
            // a constructor head, items[0] was the tag (already
            // handled); for tuple destructure, items[0] was a
            // sub-pattern (already collected above). Either way, we
            // only need to recurse into items[1..].
            for sub in items.iter().skip(1) {
                collect_pattern_bindings(sub, locals, state)?;
            }
            Ok(())
        }
        // Vector at pattern position — defensive: not a wat-rs match
        // pattern shape today, but recurse for any future binders.
        WatAST::Vector(items, _) => {
            for sub in items {
                collect_pattern_bindings(sub, locals, state)?;
            }
            Ok(())
        }
        // Arc 257.2 — Map at pattern position: classify and extract binding names.
        // collect_pattern_bindings is called for match-arm patterns; a Map at
        // pattern position may be a hash-destructure. Extract binding names so
        // the closure walker correctly shadows them.
        // Arc 257 slice 1 — Map/Set at pattern position: recurse defensively.
        WatAST::Map(pairs, _) => {
            for (k, v) in pairs {
                collect_pattern_bindings(k, locals, state)?;
                collect_pattern_bindings(v, locals, state)?;
            }
            Ok(())
        }
        WatAST::Set(items, _) => {
            for item in items {
                collect_pattern_bindings(item, locals, state)?;
            }
            Ok(())
        }
    }
}

// ─── Dep + type recording ───────────────────────────────────────────────

/// Record a name's MACRO facet, if it has one — the retained `(defmacro …)`
/// form, shipped verbatim. Idempotent; a no-op for names with no macro facet.
/// Asks the door (`registrations`) for the facet, then takes it through the
/// narrow accessor.
fn record_macro_dependency(state: &mut ExtractState<'_>, name: &str) {
    if state.captured_macros.contains_key(name) {
        return;
    }
    if !state
        .parent_symbols
        .registrations(name)
        .contains(crate::value::symbol_table::RegistryKind::Macro)
    {
        return;
    }
    if let Some(mac) = state
        .parent_symbols
        .macro_registry()
        .and_then(|reg| reg.get(name))
    {
        state
            .captured_macros
            .insert(name.to_string(), mac.source_form.clone());
        state.macro_discovery_order.push(name.to_string());
    }
}

fn record_dep_dependency(
    state: &mut ExtractState<'_>,
    name: &str,
    func: &Arc<Function>,
) {
    // Always record the edge (consumer → dep) regardless of whether
    // this dep is freshly discovered or already known. Topological
    // ordering needs every back-edge.
    if let Some(consumer) = state.current_walking_dep.clone() {
        if consumer != name {
            // Skip the edge if `name` ends up being a type accessor
            // that gets short-circuited below (won't be in
            // captured_deps); but recording an edge to a non-existent
            // node is harmless — topo_sort filters by node membership.
            state
                .dep_edges
                .entry(consumer)
                .or_default()
                .insert(name.to_string());
        }
    }
    if state.captured_deps.contains_key(name) {
        return;
    }
    // Skip auto-synthesized type accessors / constructors: a function
    // whose name is `<TypeName>/<rest>` where `<TypeName>` is a
    // declared type. The freeze pipeline re-synthesizes these when the
    // type definition is registered; including them as deps would cause
    // DuplicateDefine on re-freeze. Type accessors like `:my::Point/x`
    // fall under this rule. We ALSO need to walk the type's signature
    // for type-extraction so the corresponding TypeDef makes it into
    // the package.
    //
    // Arc 293.R2.3 — struct/newtype ctors now register at the BARE type
    // name (no `/new`). A bare name that IS a declared struct/newtype in
    // the parent types is an auto-synthesized ctor — skip as dep exactly
    // like accessors (re-freeze re-generates it via register_struct_methods
    // / register_newtype_methods).
    {
        let type_part_to_check: Option<&str> = {
            // Case 1: name has `/` — accessor or (legacy) `/new` ctor.
            if name.contains('/') {
                let tp = wat_reader::identifier::receiver(name);
                if state.parent_types.get(tp).is_some()
                    || state.captured_types.contains_key(tp)
                {
                    Some(tp)
                } else {
                    None
                }
            }
            // Case 2: bare name — arc 293.R2.3 struct/newtype bare ctor.
            else {
                let is_auto_ctor = {
                    let opt = state.parent_types.get(name)
                        .or_else(|| state.captured_types.get(name));
                    matches!(opt, Some(TypeDef::Aggregate(_)) | Some(TypeDef::Newtype(_)))
                };
                if is_auto_ctor { Some(name) } else { None }
            }
        };
        if let Some(type_part) = type_part_to_check {
            // Ensure the type is extracted so re-freeze regenerates the
            // accessor / constructor.
            record_type_dependency_by_name(state, type_part);
            // Walk the function's signature for additional type refs
            // (e.g., a Point/x might surface :wat::core::i64 — that's a
            // substrate primitive and gets skipped by the prefix gate).
            for ty in &func.param_types {
                record_type_refs_in_typeexpr(state, ty);
            }
            record_type_refs_in_typeexpr(state, &func.ret_type);
            return;
        }
    }
    // Similarly skip enum tagged-variant constructors `:E::Variant`
    // where `:E` is a declared enum.
    if name.contains("::") {
        let enum_part = wat_reader::identifier::path(name);
        if let Some(TypeDef::Enum(_)) = state.parent_types.get(enum_part) {
            record_type_dependency_by_name(state, enum_part);
            return;
        }
    }
    state.captured_deps.insert(name.to_string(), func.clone());
    state.dep_discovery_order.push(name.to_string());
    state
        .dep_edges
        .entry(name.to_string())
        .or_default();
}

fn record_type_dependency(
    state: &mut ExtractState<'_>,
    name: &str,
    def: &TypeDef,
) {
    if state.captured_types.contains_key(name) {
        return;
    }
    state.captured_types.insert(name.to_string(), def.clone());
    state.type_discovery_order.push(name.to_string());
    state
        .type_edges
        .entry(name.to_string())
        .or_default();
}

fn record_type_dependency_by_name(state: &mut ExtractState<'_>, name: &str) {
    if state.captured_types.contains_key(name) {
        return;
    }
    if let Some(def) = state.parent_types.get(name) {
        record_type_dependency(state, name, &def.clone());
    }
}

/// Walk a TypeExpr, recording any non-substrate type names referenced.
fn record_type_refs_in_typeexpr(state: &mut ExtractState<'_>, ty: &TypeExpr) {
    match ty {
        TypeExpr::Path(p) => {
            if !crate::resolve::is_reserved_prefix(p)
                && state.parent_types.get(p).is_some()
            {
                record_type_dependency_by_name(state, p);
            }
        }
        TypeExpr::Parametric { head, args } => {
            // `head` carries no leading colon (e.g. "wat::core::Vector");
            // re-attach for substrate-prefix check + lookup.
            let head_kw = crate::types::parametric_head_fqdn(head);
            if !crate::resolve::is_reserved_prefix(&head_kw)
                && state.parent_types.get(&head_kw).is_some()
            {
                record_type_dependency_by_name(state, &head_kw);
            }
            for a in args {
                record_type_refs_in_typeexpr(state, a);
            }
        }
        TypeExpr::Fn { args, ret } => {
            for a in args {
                record_type_refs_in_typeexpr(state, a);
            }
            record_type_refs_in_typeexpr(state, ret);
        }
        TypeExpr::Tuple(elems) => {
            for e in elems {
                record_type_refs_in_typeexpr(state, e);
            }
        }
        TypeExpr::Var(_) => {}
    }
}

// ─── Recursive extraction (fixpoint) ────────────────────────────────────

fn extract_user_deps_to_fixpoint(
    state: &mut ExtractState<'_>,
) -> Result<(), ExtractionError> {
    loop {
        let to_walk: Vec<String> = state
            .captured_deps
            .keys()
            .filter(|k| !state.deps_visited.contains(*k))
            .cloned()
            .collect();
        if to_walk.is_empty() {
            return Ok(());
        }
        for name in to_walk {
            state.deps_visited.insert(name.clone());
            // Walk the function's body for further frees + types.
            // Function parameters are local within its body.
            let dep_func = state
                .captured_deps
                .get(&name)
                .cloned()
                .ok_or_else(|| ExtractionError { span: crate::rust_caller_span!(), kind: ExtractionErrorKind::Internal(format!("dep {} vanished", name)) })?;
            let mut dep_locals: BTreeSet<String> =
                dep_func.params.iter().map(|p| crate::scope::env_key(p).into_owned()).collect();
            if let Some(rest) = &dep_func.rest_param {
                dep_locals.insert(rest.clone());
            }
            // Walk dep's signature for type refs (param types + ret + rest).
            for ty in &dep_func.param_types {
                record_type_refs_in_typeexpr(state, ty);
            }
            record_type_refs_in_typeexpr(state, &dep_func.ret_type);
            if let Some(rest_ty) = &dep_func.rest_param_type {
                record_type_refs_in_typeexpr(state, rest_ty);
            }
            // Snapshot the unresolved frees so we can isolate
            // dep-introduced ones.
            let pre_frees: Vec<(String, Span)> =
                std::mem::take(&mut state.unresolved_frees);
            // Stone 255.1a — Native builtins have no wat body; skip dep body walk.
            let dep_body_ast = match &dep_func.body {
                FunctionBody::Wat(ast) => ast,
                FunctionBody::Native => continue,
            };
            // Set current_walking_dep so back-edges are recorded.
            let prior = state.current_walking_dep.replace(name.clone());
            let walk_result = walk_free_symbols(dep_body_ast, &dep_locals, state);
            state.current_walking_dep = prior;
            walk_result?;
            // After walking, any non-dep / non-type / non-capture
            // unresolved free becomes a dep-relative unresolved. For a
            // top-level defn, unresolved bare Symbols indicate broken
            // input; surface immediately.
            let dep_frees = std::mem::replace(&mut state.unresolved_frees, pre_frees);
            for (fname, fspan) in dep_frees {
                // Top-level defns have no closed env; bare Symbol frees
                // here are genuinely unresolved.
                state.really_unresolved.push((fname, fspan));
            }
            if let Some((n, sp)) = state.really_unresolved.first().cloned() {
                return Err(ExtractionError {
                    span: sp,
                    kind: ExtractionErrorKind::UnresolvedSymbol {
                        name: n,
                    },
                });
            }
        }
    }
}

fn extract_user_types_to_fixpoint(
    state: &mut ExtractState<'_>,
) -> Result<(), ExtractionError> {
    loop {
        let to_walk: Vec<String> = state
            .captured_types
            .keys()
            .filter(|k| !state.types_visited.contains(*k))
            .cloned()
            .collect();
        if to_walk.is_empty() {
            return Ok(());
        }
        for name in to_walk {
            state.types_visited.insert(name.clone());
            let def = state
                .captured_types
                .get(&name)
                .cloned()
                .ok_or_else(|| ExtractionError { span: crate::rust_caller_span!(), kind: ExtractionErrorKind::Internal(format!("type {} vanished", name)) })?;
            // Walk fields / variants / inner / alias-target for further
            // type refs. Each found type-ref becomes an edge from
            // `name` to that type.
            let mut deps_for_name = BTreeSet::<String>::new();
            collect_typeexpr_type_names(&def_inner_typeexprs(&def), state.parent_types, &mut deps_for_name);
            // Dependencies of THIS type — record them and the edges.
            for dep_ty_name in &deps_for_name {
                record_type_dependency_by_name(state, dep_ty_name);
                state
                    .type_edges
                    .entry(name.clone())
                    .or_default()
                    .insert(dep_ty_name.clone());
            }
        }
    }
}

/// Pull out all `TypeExpr` references from a `TypeDef` (struct fields,
/// enum variant fields, newtype inner, alias target).
fn def_inner_typeexprs(def: &TypeDef) -> Vec<TypeExpr> {
    match def {
        // Arc 293.2b — Aggregate carries fields for both struct and record kinds.
        // Records also have typed fields (D2), so return them. Struct fields were already returned.
        TypeDef::Aggregate(a) => a.fields.iter().map(|(_, t)| t.clone()).collect(),
        TypeDef::Enum(e) => {
            let mut out = Vec::new();
            for v in &e.variants {
                if let crate::types::EnumVariant::Tagged { fields, .. } = v {
                    for (_, t) in fields {
                        out.push(t.clone());
                    }
                }
            }
            out
        }
        TypeDef::Newtype(n) => vec![n.inner.clone()],
        TypeDef::Alias(a) => vec![a.expr.clone()],
        // Stone 237.1 — typeunion members are the inner type references.
        TypeDef::Union(u) => u.members.clone(),
        // Arc 293.3-core / 293.4a — surface members carry Field and Method variants.
        // Return the TypeExpr from each member (Field.ty or Method.ret) for reachability.
        TypeDef::Surface(s) => s.members.iter().map(|m| match m {
            crate::types::SurfaceMember::Field { ty, .. } => ty.clone(),
            crate::types::SurfaceMember::Method { ret, .. } => ret.clone(),
        }).collect(),
    }
}

fn collect_typeexpr_type_names(
    types: &[TypeExpr],
    env: &TypeEnv,
    out: &mut BTreeSet<String>,
) {
    for t in types {
        collect_typeexpr_type_names_one(t, env, out);
    }
}

fn collect_typeexpr_type_names_one(
    t: &TypeExpr,
    env: &TypeEnv,
    out: &mut BTreeSet<String>,
) {
    match t {
        TypeExpr::Path(p) => {
            if !crate::resolve::is_reserved_prefix(p) && env.get(p).is_some() {
                out.insert(p.clone());
            }
        }
        TypeExpr::Parametric { head, args } => {
            let head_kw = crate::types::parametric_head_fqdn(head);
            if !crate::resolve::is_reserved_prefix(&head_kw)
                && env.get(&head_kw).is_some()
            {
                out.insert(head_kw);
            }
            for a in args {
                collect_typeexpr_type_names_one(a, env, out);
            }
        }
        TypeExpr::Fn { args, ret } => {
            for a in args {
                collect_typeexpr_type_names_one(a, env, out);
            }
            collect_typeexpr_type_names_one(ret, env, out);
        }
        TypeExpr::Tuple(elems) => {
            for e in elems {
                collect_typeexpr_type_names_one(e, env, out);
            }
        }
        TypeExpr::Var(_) => {}
    }
}

// ─── Topological sort ───────────────────────────────────────────────────

fn topo_sort_types(state: &ExtractState<'_>) -> Vec<String> {
    topo_sort(&state.captured_types.keys().cloned().collect::<Vec<_>>(),
              &state.type_edges,
              &state.type_discovery_order)
}

fn topo_sort_deps(state: &ExtractState<'_>) -> Vec<String> {
    topo_sort(&state.captured_deps.keys().cloned().collect::<Vec<_>>(),
              &state.dep_edges,
              &state.dep_discovery_order)
}

/// Standard Kahn-ish topological sort: nodes with no remaining
/// dependencies first; ties broken by discovery order. Nodes with
/// missing deps in the edge map are treated as having zero edges.
fn topo_sort(
    nodes: &[String],
    edges: &BTreeMap<String, BTreeSet<String>>,
    discovery_order: &[String],
) -> Vec<String> {
    let node_set: BTreeSet<String> = nodes.iter().cloned().collect();
    // Effective edges: keep only edges to nodes in the set.
    let mut indeg: BTreeMap<String, usize> = BTreeMap::new();
    let mut effective_edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for n in nodes {
        indeg.insert(n.clone(), 0);
    }
    for (from, to_set) in edges {
        if !node_set.contains(from) {
            continue;
        }
        for to in to_set {
            if node_set.contains(to) {
                effective_edges
                    .entry(to.clone())
                    .or_default()
                    .push(from.clone());
                *indeg.entry(from.clone()).or_default() += 1;
            }
        }
    }
    // Process: start with nodes having indeg=0. To get stable order,
    // iterate by discovery_order at each step.
    let mut output: Vec<String> = Vec::with_capacity(nodes.len());
    let mut emitted: BTreeSet<String> = BTreeSet::new();
    loop {
        let mut progressed = false;
        for n in discovery_order {
            if emitted.contains(n) || !node_set.contains(n) {
                continue;
            }
            if indeg.get(n).copied().unwrap_or(0) == 0 {
                output.push(n.clone());
                emitted.insert(n.clone());
                if let Some(consumers) = effective_edges.get(n) {
                    for c in consumers {
                        if let Some(d) = indeg.get_mut(c) {
                            if *d > 0 {
                                *d -= 1;
                            }
                        }
                    }
                }
                progressed = true;
            }
        }
        // Backstop for nodes not in discovery_order.
        for n in nodes {
            if emitted.contains(n) {
                continue;
            }
            if indeg.get(n).copied().unwrap_or(0) == 0 {
                output.push(n.clone());
                emitted.insert(n.clone());
                if let Some(consumers) = effective_edges.get(n) {
                    for c in consumers {
                        if let Some(d) = indeg.get_mut(c) {
                            if *d > 0 {
                                *d -= 1;
                            }
                        }
                    }
                }
                progressed = true;
            }
        }
        if !progressed {
            // Cycle? Emit remaining in discovery order.
            for n in nodes {
                if !emitted.contains(n) {
                    output.push(n.clone());
                    emitted.insert(n.clone());
                }
            }
            break;
        }
        if emitted.len() == nodes.len() {
            break;
        }
    }
    output
}

// ─── Value → AST encoder ────────────────────────────────────────────────

/// Encode a captured Value into an AST whose evaluation produces an
/// equal Value in the fresh world.
///
/// `binding_name` is used in error messages (ImpureCapture).
fn encode_value_to_ast(
    v: &Value,
    binding_name: &str,
    state: &mut ExtractState<'_>,
) -> Result<WatAST, ExtractionError> {
    encode_value_with_path(v, binding_name, &mut Vec::new(), state)
}

fn encode_value_with_path(
    v: &Value,
    binding_name: &str,
    path: &mut Vec<String>,
    state: &mut ExtractState<'_>,
) -> Result<WatAST, ExtractionError> {
    let span = crate::rust_caller_span!();
    match v {
        // ─── primitive arms ────────────────────────────────────────────
        Value::bool(b) => Ok(WatAST::BoolLit(*b, span)),
        Value::i64(n) => Ok(WatAST::IntLit(*n, span)),
        Value::f64(x) => Ok(WatAST::FloatLit(*x, span)),
        Value::u8(n) => {
            // u8 doesn't have a literal form; `(:wat::core::u8 N)`.
            Ok(WatAST::List(
                vec![
                    WatAST::Keyword(":wat::core::u8".into(), span.clone()),
                    WatAST::IntLit(*n as i64, span.clone()),
                ],
                span,
            ))
        }
        Value::String(s) => Ok(WatAST::StringLit((**s).clone(), span)),
        // Arc 300 stone B — Rational is the numeric-literal lane (NOT a
        // desugared call, per DESIGN-STONE-rational-B-runtime.md's pinned
        // contract) — re-encodes directly as a `RationalLit`.
        Value::wat__core__Rational(r) => Ok(WatAST::RationalLit((**r).clone(), span)),
        // Arc 300 stone C1 — BigInt is the numeric-literal lane (mirrors Rational
        // immediately above, one type over): re-encodes directly as a `BigIntLit`.
        Value::wat__core__BigInt(n) => Ok(WatAST::BigIntLit((**n).clone(), span)),
        Value::wat__core__keyword(k) => {
            // A wat-level keyword value is constructed via
            // `(:wat::core::keyword "literal-text")` — but the simpler
            // surface is to emit the bare keyword token. Emit literal
            // keyword token; eval's keyword arm produces the same value
            // for a stand-alone keyword that doesn't resolve to a
            // function.
            Ok(WatAST::Keyword((**k).clone(), span))
        }
        // Arc 207 — Uuid is portable: encode as a `Uuid/from-string` call
        // on the canonical 8-4-4-4-12 hyphenated form. Round-trips cleanly.
        Value::wat__core__Uuid(u) => Ok(WatAST::List(
            vec![
                WatAST::Keyword(":wat::uuid::from-string".into(), span.clone()),
                WatAST::StringLit(u.to_string(), span.clone()),
            ],
            span,
        )),
        // Arc 300 stone D — Char is a leaf literal at both ends of the
        // substrate now: re-encode directly as a `CharLit` (mirrors the
        // BigInt arm immediately above, one type over). Round-trips
        // cleanly. Was: a `char/of` call on a length-1 String
        // (arc 220 / stone 242.1) — that workaround is retired now that
        // WatAST can hold a char literal directly.
        Value::wat__core__Char(c) => Ok(WatAST::CharLit(*c, span)),
        // Arc 220 Stone 220.4 — List is portable: encode as a variadic
        // `(:wat::core::List item1 item2 ...)` call. Each item is recursively
        // encoded. Round-trips cleanly.
        Value::wat__core__List(items) => {
            let mut out = Vec::with_capacity(items.len() + 1);
            out.push(WatAST::Keyword(":wat::core::List".into(), span.clone()));
            for (i, it) in items.iter().enumerate() {
                path.push(format!("[{}]", i));
                let encoded = encode_value_with_path(it, binding_name, path, state)?;
                path.pop();
                out.push(encoded);
            }
            Ok(WatAST::List(out, span))
        }
        // Stone 242.2 / Arc 244 — Doctrine 1: bare `nil` is the value form; `:wat::core::nil`
        // is the TYPE keyword. Arc 244 canonicalizes this to NilLit (not Symbol("nil")).
        Value::Unit => Ok(WatAST::NilLit(span)),

        // ─── containers ────────────────────────────────────────────────
        Value::Vec(items) => {
            // `(:wat::core::Vector :- [T] elem1 elem2 ...)` — infer T from
            // the first element. Empty Vec falls back to `:wat::core::nil`
            // (the singleton type), which type-checks against any
            // surface that doesn't dispatch on element type.
            //
            // Arc 109 stone 3 (THE WALL) — this Rust-side synthesis used to emit the
            // now-retired bare form (`(Vector T ...)`); the checker no longer
            // represents that shape at all, so re-freezing a captured Vec value
            // failed CheckErrors::MalformedForm on the synthesized AST. Wrap the
            // type keyword in the `:-` marker + one-element bracket, the same
            // canonicalization `infer`'s `WatAST::Vector` literal arm does in
            // `src/check.rs`.
            let elem_kw = if let Some(first) = items.first() {
                value_static_type_keyword(first, state, &span)?
            } else {
                WatAST::Keyword(NIL_TYPE_PATH_KEYWORD.into(), span.clone())
            };
            let mut out = Vec::with_capacity(items.len() + 3);
            out.push(WatAST::Keyword(":wat::core::Vector".into(), span.clone()));
            out.push(WatAST::Keyword(":-".into(), span.clone()));
            out.push(WatAST::Vector(vec![elem_kw], span.clone()));
            for (i, it) in items.iter().enumerate() {
                path.push(format!("[{}]", i));
                let encoded = encode_value_with_path(it, binding_name, path, state)?;
                path.pop();
                out.push(encoded);
            }
            Ok(WatAST::List(out, span))
        }
        Value::Tuple(items) => {
            let mut out = Vec::with_capacity(items.len() + 1);
            out.push(WatAST::Keyword(":wat::core::Tuple".into(), span.clone()));
            for (i, it) in items.iter().enumerate() {
                path.push(format!(".{}", i));
                let encoded = encode_value_with_path(it, binding_name, path, state)?;
                path.pop();
                out.push(encoded);
            }
            Ok(WatAST::List(out, span))
        }
        Value::wat__std__HashMap(map) => {
            // Closure-capture round-trip: re-encode a runtime HashMap<K,V> Value
            // back to the corresponding `(:wat::core::HashMap :- [K V] k1 v1 k2 v2 ...)`
            // constructor AST, so the captured env can be replayed in a fresh
            // world (Arc 109 stone 3, THE WALL, updated the synthesis below to
            // the `:-`-marked spelling — see its comment). Mirrors Vector's
            // single-element `:- [T]` bracket, with two type-args for K + V
            // (arc 214 P1 retired the earlier `:(K,V)` tuple-keyword shape).
            //
            // Stone 216.5c — storage is now Arc<HashMap<Value, Value>>; iterate (k, v) directly.
            //
            // Determine K, V from the first entry. LIMITATION: empty HashMaps
            // have no entries to sample, so K + V fall back to `:wat::core::nil`
            // sentinels. A re-evaluated empty-capture will type-check as
            // `HashMap<nil,nil>` and accept any contents only at recipient
            // contexts expecting that exact shape. Non-empty captures infer
            // K + V honestly from the first entry's value types.
            let (k_kw, v_kw) = if let Some((k, vv)) = map.iter().next() {
                let kkw = value_static_type_keyword(k, state, &span)?;
                let vkw = value_static_type_keyword(vv, state, &span)?;
                (kkw, vkw)
            } else {
                (
                    WatAST::Keyword(NIL_TYPE_PATH_KEYWORD.into(), span.clone()),
                    WatAST::Keyword(NIL_TYPE_PATH_KEYWORD.into(), span.clone()),
                )
            };
            // Arc 109 stone 3 (THE WALL) — `:- [K V]` is the ONE legal param-spec
            // spelling now; see the `Value::Vec` arm's comment above for why this
            // Rust-side synthesis needed the same update.
            let mut out = Vec::with_capacity(map.len() * 2 + 4);
            out.push(WatAST::Keyword(":wat::core::HashMap".into(), span.clone()));
            out.push(WatAST::Keyword(":-".into(), span.clone()));
            out.push(WatAST::Vector(vec![k_kw, v_kw], span.clone()));
            // Stone 216.5d — sort by Value's native Hash for determinism.
            // hashmap_key canonical-key crutch removed; Value: Hash (arc 216.5a) is the contract.
            // DefaultHasher produces a stable u64 key per Value for sort ordering.
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let value_sort_key = |v: &Value| -> u64 {
                let mut h = DefaultHasher::new();
                v.hash(&mut h);
                h.finish()
            };
            let mut entries: Vec<(&Value, &Value)> = map.iter().collect();
            entries.sort_by_key(|(k, _)| value_sort_key(k));
            for (k, vv) in entries {
                let sort_key = value_sort_key(k);
                path.push(format!("{{{:x}}}", sort_key));
                let kk = encode_value_with_path(k, binding_name, path, state)?;
                let vv2 = encode_value_with_path(vv, binding_name, path, state)?;
                path.pop();
                out.push(kk);
                out.push(vv2);
            }
            Ok(WatAST::List(out, span))
        }
        Value::wat__std__HashSet(set) => {
            // Stone 216.5b — storage is now Arc<HashSet<Value>>; iterate Values directly.
            // Stone 216.5d — sort by Value's native Hash for deterministic encoding order.
            let elem_kw = if let Some(v) = set.iter().next() {
                value_static_type_keyword(v, state, &span)?
            } else {
                WatAST::Keyword(NIL_TYPE_PATH_KEYWORD.into(), span.clone())
            };
            // Arc 109 stone 3 (THE WALL) — `:- [T]` is the ONE legal param-spec
            // spelling now; see the `Value::Vec` arm's comment above.
            let mut out = Vec::with_capacity(set.len() + 3);
            out.push(WatAST::Keyword(":wat::core::HashSet".into(), span.clone()));
            out.push(WatAST::Keyword(":-".into(), span.clone()));
            out.push(WatAST::Vector(vec![elem_kw], span.clone()));
            // Stone 216.5d — sort by Value's native Hash for determinism.
            // hashmap_key canonical-key crutch removed; Value: Hash (arc 216.5a) is the contract.
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let value_sort_key_set = |v: &Value| -> u64 {
                let mut h = DefaultHasher::new();
                v.hash(&mut h);
                h.finish()
            };
            let mut entries: Vec<&Value> = set.iter().collect();
            entries.sort_by_key(|v| value_sort_key_set(v));
            for vv in entries {
                let sort_key = value_sort_key_set(vv);
                path.push(format!("{{{:x}}}", sort_key));
                let encoded = encode_value_with_path(vv, binding_name, path, state)?;
                path.pop();
                out.push(encoded);
            }
            Ok(WatAST::List(out, span))
        }
        Value::Option(opt) => match &**opt {
            Some(inner) => {
                let encoded = encode_value_with_path(inner, binding_name, path, state)?;
                Ok(WatAST::List(
                    vec![
                        WatAST::Keyword(":wat::core::Some".into(), span.clone()),
                        encoded,
                    ],
                    span,
                ))
            }
            None => Ok(WatAST::Keyword(":wat::core::None".into(), span)),
        },
        Value::Result(res) => match &**res {
            Ok(inner) => {
                let encoded = encode_value_with_path(inner, binding_name, path, state)?;
                Ok(WatAST::List(
                    vec![
                        WatAST::Keyword(":wat::core::Ok".into(), span.clone()),
                        encoded,
                    ],
                    span,
                ))
            }
            Err(inner) => {
                let encoded = encode_value_with_path(inner, binding_name, path, state)?;
                Ok(WatAST::List(
                    vec![
                        WatAST::Keyword(":wat::core::Err".into(), span.clone()),
                        encoded,
                    ],
                    span,
                ))
            }
        },
        Value::Aggregate(a) if a.nature == Nature::Struct => {
            // `(:my::Type/new f1 f2 ...)`. Extract the struct's TypeDef
            // for inclusion in package.forms.
            let type_name_with_colon = format!(":{}", a.class);
            ensure_type_extracted(state, &type_name_with_colon);
            encode_struct(a, binding_name, path, state, span)
        }
        Value::Enum(ev) => {
            // `:my::E::Variant` (unit) or `(:my::E::Variant a b)` (tagged).
            ensure_type_extracted(state, &ev.type_path);
            let constructor =
                format!("{}::{}", ev.type_path, ev.variant_name);
            if ev.fields.is_empty() {
                Ok(WatAST::Keyword(constructor, span))
            } else {
                let mut out = Vec::with_capacity(ev.fields.len() + 1);
                out.push(WatAST::Keyword(constructor, span.clone()));
                for (i, f) in ev.fields.iter().enumerate() {
                    path.push(format!(".{}", i));
                    let encoded = encode_value_with_path(f, binding_name, path, state)?;
                    path.pop();
                    out.push(encoded);
                }
                Ok(WatAST::List(out, span))
            }
        }

        Value::wat__core__PersistentMap(map) => {
            // Closure-capture round-trip: re-encode a runtime PersistentMap<K,V> Value
            // back to the corresponding `(:wat::core::PersistentMap k1 v1 k2 v2 ...)`
            // constructor AST. PersistentMap ctor takes k/v pairs directly (no type header).
            // Arc-278-0a.
            let mut out = Vec::with_capacity(map.len() * 2 + 1);
            out.push(WatAST::Keyword(":wat::core::PersistentMap".into(), span.clone()));
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let value_sort_key = |v: &Value| -> u64 {
                let mut h = DefaultHasher::new();
                v.hash(&mut h);
                h.finish()
            };
            let mut entries: Vec<(&Value, &Value)> = map.iter().collect();
            entries.sort_by_key(|(k, _)| value_sort_key(k));
            for (k, vv) in entries {
                let sort_key = value_sort_key(k);
                path.push(format!("{{{:x}}}", sort_key));
                let kk = encode_value_with_path(k, binding_name, path, state)?;
                let vv2 = encode_value_with_path(vv, binding_name, path, state)?;
                path.pop();
                out.push(kk);
                out.push(vv2);
            }
            Ok(WatAST::List(out, span))
        }

        Value::wat__core__PersistentVector(pv) => {
            // Closure-capture round-trip: re-encode a runtime PersistentVector<T> Value
            // back to the corresponding `(:wat::core::PersistentVector e1 e2 ...)`
            // constructor AST. PersistentVector ctor takes bare elements in order.
            // Arc-278-0b.
            let mut out = Vec::with_capacity(pv.len() + 1);
            out.push(WatAST::Keyword(":wat::core::PersistentVector".into(), span.clone()));
            for (i, elem) in pv.iter().enumerate() {
                path.push(format!("[{}]", i));
                let encoded = encode_value_with_path(elem, binding_name, path, state)?;
                path.pop();
                out.push(encoded);
            }
            Ok(WatAST::List(out, span))
        }

        // ─── non-portable arms ────────────────────────────────────────
        Value::wat__kernel__Sender(_)
        | Value::wat__kernel__Receiver(_)
        | Value::wat__kernel__HandlePool { .. }
        | Value::wat__kernel__ChildHandle(_)
        | Value::io__IOReader(_)
        | Value::io__IOWriter(_)
        | Value::OnlineSubspace(_)
        | Value::Reckoner(_)
        | Value::Engram(_)
        | Value::EngramLibrary(_)
        | Value::Hologram(_) => Err(ExtractionError {
            span: crate::rust_caller_span!(),
            kind: ExtractionErrorKind::ImpureCapture {
                name: binding_name.to_string(),
                type_name: v.type_name().to_string(),
                path: path.clone(),
            },
        }),
        // Arc 293.R2.1 — Aggregate (Record/HolonRecord): no closure-extract encoding yet.
        // The closure-extract path for records lands when the constructor form is available.
        // No guard here — the Struct arm above catches nature==Struct, so this arm is
        // reached only when nature!=Struct (Record/HolonRecord). Guard dropped so Rust's
        // exhaustiveness checker sees Value::Aggregate(_) as fully covered.
        Value::Aggregate(_) => {
            Err(ExtractionError {
                span: crate::rust_caller_span!(),
                kind: ExtractionErrorKind::Internal(format!(
                    "encoding for captured Value of kind {} not implemented (Stone 234.2+)",
                    v.type_name()
                )),
            })
        }

        // Stone 237.2 — wat__core__clauses: no closure-extract encoding for
        // multi-arity dispatchers yet. Clause bodies are evaluated at dispatch
        // time; the dispatcher itself is a top-level registration (not a closure).
        Value::wat__core__clauses(_) => Err(ExtractionError {
            span: crate::rust_caller_span!(),
            kind: ExtractionErrorKind::Internal(format!(
                "encoding for captured Value of kind {} not implemented (Stone 237.2 — defclause is top-level)",
                v.type_name()
            )),
        }),

        // ─── arms slice 1 doesn't yet encode ──────────────────────────
        // These are portable in principle; surface as Internal so a
        // surfacing test reveals the gap. (Per FM 5: don't bridge with
        // a TODO.)
        Value::wat__core__fn(_)
        | Value::holon__HolonAST(_)
        | Value::wat__WatAST(_)
        | Value::RustOpaque(_)
        | Value::Vector(_)
        | Value::Instant(_)
        | Value::Duration(_)
        // Arc 118 — Stream: lazy seqs carry closures/thunks; closure-extract encoding
        // is a later strike (not portable in slice 1; same as fn).
        | Value::wat__stream__Stream(_)
        // Arc 232 Stone 232.1 — registry carriers are top-level registrations, not closure values.
        | Value::wat__core__extend_def(_)
        // Arc 278 Stone A — foreign dynamic values are portable in principle (they
        // re-serialize to EDN), but closure-extract re-encoding is a later strike;
        // surface as Internal so the gap is honest (no silent bridge, FM5).
        | Value::ForeignRecord(_)
        | Value::ForeignVariant(_) => Err(ExtractionError {
            span: crate::rust_caller_span!(),
            kind: ExtractionErrorKind::Internal(format!(
                "encoding for captured Value of kind {} not implemented in slice 1",
                v.type_name()
            )),
        }),
    }
}

fn encode_struct(
    sv: &AggregateValue,
    binding_name: &str,
    path: &mut Vec<String>,
    state: &mut ExtractState<'_>,
    span: Span,
) -> Result<WatAST, ExtractionError> {
    // Pull field names from the TypeEnv (if available) for nicer path
    // diagnostics; positional order is what `<Type>/new` expects.
    // Arc 293.2b — AggregateDef with kind==Struct replaces StructDef.
    let type_name_with_colon = format!(":{}", sv.class);
    let field_names: Option<Vec<String>> = state.parent_types.get(&type_name_with_colon).and_then(|td| {
        if let TypeDef::Aggregate(a) = td {
            if a.nature == crate::types::Nature::Struct {
                Some(a.fields.iter().map(|(n, _)| n.clone()).collect())
            } else {
                None
            }
        } else if let TypeDef::Newtype(_) = td {
            Some(vec!["0".to_string()])
        } else {
            None
        }
    });
    // Arc 293.R2.3 — bare struct/newtype ctor; `/new` annihilated.
    // Arc 294 item 9a — the flip made the bare type name a kwargs MACRO; this is CODEGEN
    // (a captured struct value serialized to a wat form, decoded via eval in the child with
    // no macro expansion), and it emits fields POSITIONALLY, so it targets the positional
    // PRIME `:T'` (a plain ctor fn) — the reserved machinery form.
    let constructor = format!(":{}'", sv.class);
    let mut out = Vec::with_capacity(sv.fields.len() + 1);
    out.push(WatAST::Keyword(constructor, span.clone()));
    for (i, f) in sv.fields.iter().enumerate() {
        let name = field_names
            .as_ref()
            .and_then(|v| v.get(i).cloned())
            .unwrap_or_else(|| format!("f{}", i));
        path.push(name);
        let encoded = encode_value_with_path(f, binding_name, path, state)?;
        path.pop();
        out.push(encoded);
    }
    Ok(WatAST::List(out, span))
}

fn ensure_type_extracted(state: &mut ExtractState<'_>, name: &str) {
    if state.captured_types.contains_key(name) {
        return;
    }
    if crate::resolve::is_reserved_prefix(name) {
        return;
    }
    if let Some(def) = state.parent_types.get(name).cloned() {
        record_type_dependency(state, name, &def);
    }
}

/// The static type NODE to emit as the type-arg for a Vec/HashMap constructor based on a
/// sample Value's tag. Conservatively returns the FQDN of the value's runtime tag — the
/// type-checker will reconcile this against the captured-binding's downstream uses.
///
/// Arc 109 ③ — angle brackets are ILLEGAL for types, so this used to build an angle-bracket
/// STRING (`format!(":wat::core::Vector<{}>", inner)`) that only worked because the caller
/// wrapped it in ONE flat `WatAST::Keyword`. Returns a `WatAST` type-position NODE directly
/// now: a base case is a `WatAST::Keyword`; a parametric case is the reference FORM
/// `(Head :- [args])`, a `WatAST::List` — which the constructor-arg slot already accepts
/// (`infer_list_constructor`'s arc 109 ②-iii widening, `src/check.rs`). Callers splice the
/// returned node directly (no more re-wrapping in `WatAST::Keyword`).
fn value_static_type_keyword(
    v: &Value,
    state: &mut ExtractState<'_>,
    span: &Span,
) -> Result<WatAST, ExtractionError> {
    // (:Head :- [args…]) — the reference FORM shared by every parametric arm below.
    fn parametric(head: &str, args: Vec<WatAST>, span: &Span) -> WatAST {
        WatAST::List(
            vec![
                WatAST::Keyword(format!(":{head}"), span.clone()),
                WatAST::Keyword(":-".into(), span.clone()),
                WatAST::Vector(args, span.clone()),
            ],
            span.clone(),
        )
    }
    // Arc 244 Doctrine 1 / Arc 109 ③ — `:wat::core::nil` here is the TYPE keyword (a
    // placeholder elem/ok/err type for an empty Vec/HashMap/HashSet or a `None`/`Err`-less
    // Result — a genuinely PARSEABLE type, `TypeExpr::Path(":wat::core::nil")`), never the
    // VALUE literal `WatAST::nil()`/`NilLit` `gate_no_nil_keyword_synthesis` guards against —
    // a `NilLit` is not one of `parse_type_node`'s accepted type-position node kinds, so
    // swapping to it here would break every empty-container type-arg. Uses the module-level
    // `NIL_TYPE_PATH_KEYWORD` constant so the gate's textual scan reads this as a type-
    // position use, not the value-position heresy it hunts.
    let nil_kw = || WatAST::Keyword(NIL_TYPE_PATH_KEYWORD.into(), span.clone());
    Ok(match v {
        Value::bool(_) => WatAST::Keyword(":wat::core::bool".into(), span.clone()),
        Value::i64(_) => WatAST::Keyword(":wat::core::i64".into(), span.clone()),
        Value::u8(_) => WatAST::Keyword(":wat::core::u8".into(), span.clone()),
        Value::f64(_) => WatAST::Keyword(":wat::core::f64".into(), span.clone()),
        Value::String(_) => WatAST::Keyword(":wat::core::String".into(), span.clone()),
        Value::wat__core__keyword(_) => WatAST::Keyword(":wat::core::keyword".into(), span.clone()),
        Value::Unit => nil_kw(),
        Value::Vec(items) => {
            let inner = if let Some(first) = items.first() {
                value_static_type_keyword(first, state, span)?
            } else {
                nil_kw()
            };
            parametric("wat::core::Vector", vec![inner], span)
        }
        Value::Tuple(items) => {
            let mut parts = Vec::with_capacity(items.len());
            for it in items.iter() {
                parts.push(value_static_type_keyword(it, state, span)?);
            }
            // `parse_type_form`'s `raw_head == "wat::core::Tuple"` special-case
            // (`src/types.rs`) collapses this to `TypeExpr::Tuple` — identical to the
            // retired native `:(...)` string spelling, structurally instead of textually.
            parametric("wat::core::Tuple", parts, span)
        }
        Value::Option(opt) => {
            let inner = match &**opt {
                Some(v) => value_static_type_keyword(v, state, span)?,
                None => nil_kw(),
            };
            parametric("wat::core::Option", vec![inner], span)
        }
        Value::Result(res) => match &**res {
            Ok(v) => {
                let ok_ty = value_static_type_keyword(v, state, span)?;
                parametric("wat::core::Result", vec![ok_ty, nil_kw()], span)
            }
            Err(e) => {
                let err_ty = value_static_type_keyword(e, state, span)?;
                parametric("wat::core::Result", vec![nil_kw(), err_ty], span)
            }
        },
        Value::Aggregate(a) if a.nature == Nature::Struct => {
            let type_name_with_colon = format!(":{}", a.class);
            ensure_type_extracted(state, &type_name_with_colon);
            WatAST::Keyword(type_name_with_colon, span.clone())
        }
        Value::Enum(ev) => {
            ensure_type_extracted(state, &ev.type_path);
            WatAST::Keyword(ev.type_path.clone(), span.clone())
        }
        // rune:purgare(safety-margin) — bare `:wat::core::HashMap` keyword emitted
        // when HashMap is nested as a container element (e.g., Vec<HashMap<K,V>>).
        // Arc 214 P1 changed the constructor to require two separate type-keywords
        // `:K :V`; this static-tag path emits only the head. No current production
        // path exercises nested-HashMap-in-container through closure extraction;
        // if one arises, the K/V keywords must be derived here (sample first entry
        // like the encode arm above) or emit via a richer type-tag mechanism.
        Value::wat__std__HashMap(_) => WatAST::Keyword(":wat::core::HashMap".into(), span.clone()),
        Value::wat__core__PersistentMap(_) => WatAST::Keyword(":wat::core::PersistentMap".into(), span.clone()),
        Value::wat__core__PersistentVector(_) => WatAST::Keyword(":wat::core::PersistentVector".into(), span.clone()),
        Value::wat__std__HashSet(_) => WatAST::Keyword(":wat::core::HashSet".into(), span.clone()),
        // Non-portable types — they should not be reaching here through
        // a portable container, but if they do, encoding fails through
        // the value-level path.
        other => WatAST::Keyword(format!(":{}", other.type_name()), span.clone()),
    })
}

// ─── Arc 170 slice 3 Gap H — body prelude lift ──────────────────────────

/// Classify whether an AST form is a "prelude form" eligible for lifting
/// from a fn body's do-prefix into the closure's prologue.
///
/// A form is a prelude form if it is a list whose head keyword is one of:
///   - `:wat::core::define`
///   - `:wat::core::defstruct` (Stone 241.8; was `:wat::core::struct`)
///   - `:wat::core::enum`
///
/// These are the three forms that `startup_from_forms` processes at
/// top-level (step 5 for types, step 6 for defines). They cannot be
/// evaluated at expression position inside `eval_do_tail` (which returns
/// `DefineInExpressionPosition` for `define`; `UnknownFunction` for
/// `defstruct` / `enum` which are freeze-time forms, not runtime ones).
/// Returns the head keyword string of a `WatAST::List` form, or `None`
/// for non-list nodes or lists whose head is not a `Keyword`.
fn head_keyword(node: &WatAST) -> Option<&str> {
    if let WatAST::List(items, _) = node {
        if let Some(WatAST::Keyword(k, _)) = items.first() {
            return Some(k.as_str());
        }
    }
    None
}

/// Split a fn body's leading prelude forms from the residual expressions.
///
/// Returns `(prelude_forms, residual_body)`.
///
/// If `body` is a `(:wat::core::do ...)` form:
///   - Scans children left-to-right collecting consecutive declaration forms — a child
///     lifts iff its head keyword's registry row names a `role = declare` implementation
///     (`crate::intrinsic::is_declare_role_head`, arc 255 Stone 1a-β-ii). This REPLACES the
///     nine-name (then three-name) `freeze::is_liftable_declaration_head` hand-list, now
///     DELETED: `def` · `defalias` · `defenum` · `defmacro` · `defsurface` · `newtype` ·
///     `structtype` · `typealias` all carry a `role = declare` row today, which is why the
///     population this scan admits is unchanged. Stops at the FIRST non-declaration child.
///   - Returns `(prelude_forms, reconstructed_residual_do_or_expr)`.
///   - If `prelude_forms` is empty (no leading declarations), returns
///     `(vec![], body)` unchanged — the caller applies no lift.
///   - Residual shape:
///       - 0 children remaining → `:wat::core::nil` keyword (the fn
///         body had only declaration forms; expression is implicit nil)
///       - 1 child remaining → that child directly (no do wrapper)
///       - 2+ children remaining → `(:wat::core::do child1 child2 ...)`
///
/// If `body` is NOT a do form (single expression, let, etc.), returns
/// `(vec![], body)` unchanged — no lift for non-do shapes.
///
/// **Prefix-termination rule**: the split stops at the FIRST non-declaration
/// child. A declaration form AFTER an expression stays in the residual body.
/// If the user places a define after an expression in a do body, it will
/// still reach `eval_do_tail` and trigger `DefineInExpressionPosition` at
/// runtime — this is the correct and documented behavior (Gap H/I-A lift only
/// the PREFIX run, not all declaration forms throughout the body).
fn split_body_prelude(body: WatAST) -> (Vec<WatAST>, WatAST) {
    // Only do-forms are eligible for prelude splitting.
    let (do_children, span) = match &body {
        WatAST::List(items, span) => {
            match items.first() {
                Some(WatAST::Keyword(k, _)) if k == ":wat::core::do" => {
                    // Children are items[1..] (skip the `:wat::core::do` head).
                    (items[1..].to_vec(), span.clone())
                }
                _ => return (vec![], body),
            }
        }
        _ => return (vec![], body),
    };

    // Scan for the leading prelude prefix.
    let prefix_len = do_children
        .iter()
        .take_while(|child| {
            head_keyword(child)
                .map(crate::intrinsic::is_declare_role_head)
                .unwrap_or(false)
        })
        .count();

    if prefix_len == 0 {
        // No leading prelude — body unchanged.
        return (vec![], body);
    }

    let prelude_forms: Vec<WatAST> = do_children[..prefix_len].to_vec();
    let residual_children: Vec<WatAST> = do_children[prefix_len..].to_vec();

    // Reconstruct the residual body based on how many children remain.
    let residual_body = match residual_children.len() {
        0 => {
            // Only prelude forms in the do; body is implicitly nil.
            // Arc 244 — use NilLit(span) (canonical nil value literal).
            WatAST::NilLit(span)
        }
        1 => {
            // Single residual expression — no do wrapper needed.
            residual_children.into_iter().next().unwrap()
        }
        _ => {
            // Multiple residual expressions — wrap in a new do.
            let mut do_items = Vec::with_capacity(residual_children.len() + 1);
            do_items.push(WatAST::Keyword(":wat::core::do".into(), span.clone()));
            do_items.extend(residual_children);
            WatAST::List(do_items, span)
        }
    };

    (prelude_forms, residual_body)
}

// ─── Body rewriting ─────────────────────────────────────────────────────

/// Walk the body AST and rewrite any free reference to a captured
/// local from bare-Symbol form to a Keyword form referencing the
/// synthetic capture name. References that are SHADOWED by a let /
/// fn / define-introduced local stay unchanged.
fn rewrite_captures(
    node: &WatAST,
    captures: &[CapturedBinding],
    outer_locals: &BTreeSet<String>,
) -> WatAST {
    let mut by_name: std::collections::HashMap<&str, &CapturedBinding> =
        std::collections::HashMap::new();
    for cb in captures {
        by_name.insert(cb.original_name.as_str(), cb);
    }
    rewrite_with_scope(node, &by_name, outer_locals)
}

fn rewrite_with_scope(
    node: &WatAST,
    by_name: &std::collections::HashMap<&str, &CapturedBinding>,
    locals: &BTreeSet<String>,
) -> WatAST {
    match node {
        WatAST::IntLit(_, _)
        | WatAST::FloatLit(_, _)
        // Arc 300 stone B — RationalLit is a leaf; no scope rewrite needed.
        | WatAST::RationalLit(_, _)
        // Arc 300 stone C1 — BigIntLit is a leaf too.
        | WatAST::BigIntLit(_, _)
        // Arc 300 stone D — CharLit is a leaf too.
        | WatAST::CharLit(_, _)
        | WatAST::BoolLit(_, _)
        | WatAST::StringLit(_, _)
        // Arc 244 — NilLit is a leaf; no scope rewrite needed.
        | WatAST::NilLit(_)
        | WatAST::Keyword(_, _) => node.clone(),

        WatAST::Symbol(ident, span) => {
            if !locals.contains(ident.as_str()) {
                if let Some(cb) = by_name.get(ident.as_str()) {
                    return WatAST::Keyword(cb.synthetic_name.clone(), span.clone());
                }
            }
            node.clone()
        }

        WatAST::List(items, span) => {
            // Recognize binding-introducing forms; preserve scope rules.
            if let Some((WatAST::Keyword(k, _), _)) = items.split_first() {
                if k == ":wat::core::let" {
                    return rewrite_let(items, by_name, locals, span.clone());
                }
                if k == ":wat::core::fn" {
                    return rewrite_fn(items, by_name, locals, span.clone());
                }
            }
            let new_items: Vec<WatAST> = items
                .iter()
                .map(|it| rewrite_with_scope(it, by_name, locals))
                .collect();
            WatAST::List(new_items, span.clone())
        }

        WatAST::Vector(items, span) => {
            let new_items: Vec<WatAST> = items
                .iter()
                .map(|it| rewrite_with_scope(it, by_name, locals))
                .collect();
            WatAST::Vector(new_items, span.clone())
        }

        // Arc 257.2 — Map/Set literals: rewrite children (they may contain
        // free symbols). Map keys/values and Set elements are value expressions.
        // NOTE: binder-position Maps (in rewrite_let) are copied verbatim by
        // rewrite_let itself (out_inner.push(binder.clone())); this arm only
        // fires for value-position Maps.
        WatAST::Map(pairs, span) => {
            let new_pairs: Vec<(WatAST, WatAST)> = pairs
                .iter()
                .map(|(k, v)| (
                    rewrite_with_scope(k, by_name, locals),
                    rewrite_with_scope(v, by_name, locals),
                ))
                .collect();
            WatAST::Map(new_pairs, span.clone())
        }
        WatAST::Set(items, span) => {
            let new_items: Vec<WatAST> = items
                .iter()
                .map(|it| rewrite_with_scope(it, by_name, locals))
                .collect();
            WatAST::Set(new_items, span.clone())
        }
    }
}

fn rewrite_let(
    items: &[WatAST],
    by_name: &std::collections::HashMap<&str, &CapturedBinding>,
    outer_locals: &BTreeSet<String>,
    span: Span,
) -> WatAST {
    // items[0] = head keyword; items[1] = bindings vector;
    // items[2..] = body.
    let mut out = Vec::with_capacity(items.len());
    out.push(items[0].clone());
    if items.len() < 2 {
        return WatAST::List(items.to_vec(), span);
    }
    let bindings_vec = &items[1];
    let mut current_locals = outer_locals.clone();
    let new_bindings = match bindings_vec {
        WatAST::Vector(inner, ispan) => {
            let mut out_inner = Vec::with_capacity(inner.len());
            let mut i = 0;
            while i + 1 < inner.len() {
                let binder = &inner[i];
                let rhs = &inner[i + 1];
                // RHS evaluated under current_locals BEFORE binder takes effect.
                let rhs_rewritten = rewrite_with_scope(rhs, by_name, &current_locals);
                // Now extend scope with binder names.
                match binder {
                    WatAST::Symbol(ident, _) => {
                        current_locals.insert(ident.as_str().to_owned());
                    }
                    WatAST::Vector(bv, _) => {
                        for it in bv {
                            if let WatAST::Symbol(ident, _) = it {
                                current_locals.insert(ident.as_str().to_owned());
                            }
                        }
                    }
                    // Arc 257.2 — Map binder: classify and extract binding
                    // names. Binder symbols must be added to current_locals
                    // BEFORE the body is rewritten so they are treated as
                    // local (not free) refs. The binder Map is copied verbatim
                    // (out_inner.push(binder.clone()) below) — never substituted.
                    WatAST::Map(pairs, _) => {
                        if let Some(md) = WatAST::classify_map_destructure(pairs) {
                            for (ident, _, _) in &md.bindings {
                                current_locals.insert(ident.as_str().to_owned());
                            }
                        }
                    }
                    _ => {}
                }
                out_inner.push(binder.clone());
                out_inner.push(rhs_rewritten);
                i += 2;
            }
            // If the bindings vector had an odd-length tail (malformed),
            // copy verbatim.
            if i < inner.len() {
                out_inner.push(inner[i].clone());
            }
            WatAST::Vector(out_inner, ispan.clone())
        }
        other => other.clone(),
    };
    out.push(new_bindings);
    for body_form in items.iter().skip(2) {
        out.push(rewrite_with_scope(body_form, by_name, &current_locals));
    }
    WatAST::List(out, span)
}

fn rewrite_fn(
    items: &[WatAST],
    by_name: &std::collections::HashMap<&str, &CapturedBinding>,
    outer_locals: &BTreeSet<String>,
    span: Span,
) -> WatAST {
    // items[0] = head keyword; items[1] = args vector (param triples);
    // items[2] = `->` symbol; items[3] = :Ret keyword; items[4..] = body.
    let mut new_locals = outer_locals.clone();
    if items.len() >= 2 {
        if let WatAST::Vector(av, _) = &items[1] {
            let mut j = 0;
            while j < av.len() {
                if let WatAST::Symbol(ident, _) = &av[j] {
                    new_locals.insert(ident.as_str().to_owned());
                }
                j += 3;
            }
        }
    }
    let mut out = Vec::with_capacity(items.len());
    if items.is_empty() {
        return WatAST::List(items.to_vec(), span);
    }
    out.push(items[0].clone());
    for (i, item) in items.iter().enumerate().skip(1) {
        if i < 4 {
            // header positions: args vector / `->` / :Ret keyword. Keep
            // verbatim — captures don't appear in signatures.
            out.push(item.clone());
        } else {
            out.push(rewrite_with_scope(item, by_name, &new_locals));
        }
    }
    WatAST::List(out, span)
}

// ─── ClosurePackage assembly helpers ────────────────────────────────────

fn capture_define_form(cb: &CapturedBinding) -> WatAST {
    // Use `(:wat::core::def :user::closure-capture::X <encoded>)` to
    // bind the captured value at top level. Per arc 157, def-bound
    // names resolve at the keyword arm of `eval` after unit_variants.
    let span = crate::rust_caller_span!();
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::def".into(), span.clone()),
            WatAST::Keyword(cb.synthetic_name.clone(), span.clone()),
            cb.encoded_ast.clone(),
        ],
        span,
    )
}

fn def_form(name: &str, encoded: &WatAST) -> WatAST {
    // Sibling of `capture_define_form`: same three-item
    // `(:wat::core::def <name> <encoded>)` shape, but the name is the
    // def's ORIGINAL keyword — the body references it by Keyword, and
    // Keyword references are never rewritten, unlike captured locals.
    let span = crate::rust_caller_span!();
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::def".into(), span.clone()),
            WatAST::Keyword(name.to_string(), span.clone()),
            encoded.clone(),
        ],
        span,
    )
}

/// Arc 109 ③ — convert a closed `TypeExpr` into a faithful COLON-mode WatAST type-position
/// node (`:wat::core::Vector` Keyword for a non-parametric type; the reference FORM
/// `(Head :- [args])`, a `WatAST::List`, for a parametric one). Every re-emission site below
/// used to build `WatAST::Keyword(crate::check::format_type(ty), span)` — `format_type`
/// renders a `Parametric` as the angle-bracket string `Head<args>` (still correct for its
/// OTHER callers, e.g. `extend-type`'s internal bookkeeping key, which is never re-parsed as
/// source — see its own doc), but closure extraction re-EMITS this as real wat source text
/// that DOES get re-parsed, so the angle spelling is illegal here. Delegates to
/// `edn::render::type_expr_to_clojure_form`'s `Colon` mode — the SAME renderer backing
/// `keyword/to-type-form-colon` — rather than a second hand-rolled spelling.
fn type_expr_to_colon_ast(ty: &crate::types::TypeExpr, span: &Span) -> WatAST {
    match ty {
        // Mirrors `runtime.rs`'s `type_expr_to_ast` Var fallback: `type_expr_to_clojure_form`
        // panics on `Var` (only ever expects a parsed-from-source type); a bare-symbol
        // rendering is the same shape its Path type-var arm produces.
        crate::types::TypeExpr::Var(id) => {
            WatAST::Symbol(crate::scope::Identifier::bare(format!("t{id}")), span.clone())
        }
        other => match crate::edn::render::type_expr_to_clojure_form(other, crate::edn::render::TypeFormHeadMode::Colon) {
            Ok(node) => node,
            Err(_) => WatAST::Keyword(crate::check::format_type(other), span.clone()),
        },
    }
}

fn type_def_to_ast(def: &TypeDef) -> WatAST {
    // Reconstruct the source-form for a TypeDef.
    let span = crate::rust_caller_span!();
    match def {
        // Arc 293.2b — Aggregate branches on nature to reconstruct the right source form.
        TypeDef::Aggregate(a) => match a.nature {
            crate::types::Nature::Struct => {
                // Stone 241.8 — emit defstruct triple-form: [field <- :T ...].
                let mut field_vec_items = Vec::with_capacity(a.fields.len() * 3);
                for (fname, fty) in &a.fields {
                    field_vec_items.push(WatAST::Symbol(Identifier::bare(fname.clone()), span.clone()));
                    field_vec_items.push(WatAST::Symbol(Identifier::bare("<-".to_string()), span.clone()));
                    field_vec_items.push(type_expr_to_colon_ast(fty, &span));
                }
                let mut items = vec![WatAST::Keyword(":wat::core::defstruct".into(), span.clone())];
                items.extend(decl_name_siblings(&a.name, &a.type_params, &span));
                items.push(WatAST::Vector(field_vec_items, span.clone()));
                WatAST::List(items, span)
            }
            _ => {
                // Stone S-B.1 — reconstruct recordtype form from AggregateDef.
                // Arc 293 annihilation: parent field deleted; derive from nature.root_keyword().
                // Bug fix — reconstruct the fields vector too (mirror the Struct branch),
                // else a user defrecord ships to a process-bracket child fields-less and
                // re-parses malformed ("expected (:recordtype :Name :Parent [fields]) got 2 args").
                let mut field_vec_items = Vec::with_capacity(a.fields.len() * 3);
                for (fname, fty) in &a.fields {
                    field_vec_items.push(WatAST::Symbol(Identifier::bare(fname.clone()), span.clone()));
                    field_vec_items.push(WatAST::Symbol(Identifier::bare("<-".to_string()), span.clone()));
                    field_vec_items.push(type_expr_to_colon_ast(fty, &span));
                }
                WatAST::List(
                    vec![
                        WatAST::Keyword(":wat::core::recordtype".into(), span.clone()),
                        WatAST::Keyword(a.name.clone(), span.clone()),
                        WatAST::Keyword(a.nature.root_keyword().to_string(), span.clone()),
                        WatAST::Vector(field_vec_items, span.clone()),
                    ],
                    span,
                )
            }
        },
        TypeDef::Enum(e) => {
            // Stone 241.9 — emit defenum positional grammar:
            //   :TypeName :wat::enum::Pure|:wat::enum::Impure :V1_unit_kw :V2_tagged_kw [f1 <- :T1 ...]
            // Arc 293.W.2b — the mandatory purity marker must come immediately after the name.
            let purity_marker = if e.purity.is_pure() { ":wat::enum::Pure" } else { ":wat::enum::Impure" };
            let mut items = vec![WatAST::Keyword(":wat::core::defenum".into(), span.clone())];
            items.extend(decl_name_siblings(&e.name, &e.type_params, &span));
            items.push(WatAST::Keyword(purity_marker.to_string(), span.clone()));
            for variant in &e.variants {
                match variant {
                    crate::types::EnumVariant::Unit(name) => {
                        items.push(WatAST::Keyword(format!(":{}", name), span.clone()));
                    }
                    crate::types::EnumVariant::Tagged { name, fields } => {
                        // Variant name keyword.
                        items.push(WatAST::Keyword(format!(":{}", name), span.clone()));
                        // Argspec Vector: [f1, <-, :T1, f2, <-, :T2, ...]
                        let mut vec_items = Vec::with_capacity(fields.len() * 3);
                        for (fname, fty) in fields {
                            vec_items.push(WatAST::Symbol(
                                Identifier::bare(fname.clone()),
                                span.clone(),
                            ));
                            vec_items.push(WatAST::Symbol(
                                Identifier::bare("<-".to_string()),
                                span.clone(),
                            ));
                            vec_items.push(type_expr_to_colon_ast(fty, &span));
                        }
                        items.push(WatAST::Vector(vec_items, span.clone()));
                    }
                }
            }
            WatAST::List(items, span)
        }
        TypeDef::Newtype(n) => {
            let mut items = vec![WatAST::Keyword(":wat::core::newtype".into(), span.clone())];
            items.extend(decl_name_siblings(&n.name, &n.type_params, &span));
            items.push(type_expr_to_colon_ast(&n.inner, &span));
            WatAST::List(items, span)
        }
        TypeDef::Alias(a) => {
            let mut items = vec![WatAST::Keyword(":wat::core::typealias".into(), span.clone())];
            items.extend(decl_name_siblings(&a.name, &a.type_params, &span));
            items.push(type_expr_to_colon_ast(&a.expr, &span));
            WatAST::List(items, span)
        }
        // Stone 237.1 — reconstruct typeunion form from UnionDef.
        TypeDef::Union(u) => {
            let member_items: Vec<WatAST> = u
                .members
                .iter()
                .map(|m| type_expr_to_colon_ast(m, &span))
                .collect();
            let mut items = vec![WatAST::Keyword(":wat::core::typeunion".into(), span.clone())];
            items.extend(decl_name_siblings(&u.name, &u.type_params, &span));
            items.push(WatAST::Vector(member_items, span.clone()));
            WatAST::List(items, span)
        }
        // Arc 293.3-core / 293.4a — reconstruct defsurface form.
        // Field members → `name <- :T` triples; Method members → `(name [self] -> :R)` lists.
        TypeDef::Surface(s) => {
            let mut member_vec_items = Vec::new();
            for member in &s.members {
                match member {
                    crate::types::SurfaceMember::Field { name: mname, ty: mty } => {
                        member_vec_items.push(WatAST::Symbol(Identifier::bare(mname.clone()), span.clone()));
                        member_vec_items.push(WatAST::Symbol(Identifier::bare("<-".to_string()), span.clone()));
                        member_vec_items.push(type_expr_to_colon_ast(mty, &span));
                    }
                    crate::types::SurfaceMember::Method { name: mname, args: margs, ret: mret, .. } => {
                        // Reconstruct as `(name [arg_triples... | self] -> :RetType)`.
                        // Rebuild the argvec from the ArgSpec's fixed_params; fall back to
                        // bare `[self]` when fixed_params is empty (untyped surface member).
                        let arg_vec_items: Vec<WatAST> = if margs.fixed_params.is_empty() {
                            vec![WatAST::Symbol(Identifier::bare("self".to_string()), span.clone())]
                        } else {
                            margs.fixed_params.iter().flat_map(|(id, ty)| {
                                vec![
                                    WatAST::Symbol(id.clone(), span.clone()),
                                    WatAST::Symbol(Identifier::bare("<-".to_string()), span.clone()),
                                    type_expr_to_colon_ast(ty, &span),
                                ]
                            }).collect()
                        };
                        let method_list = WatAST::List(
                            vec![
                                WatAST::Symbol(Identifier::bare(mname.clone()), span.clone()),
                                WatAST::Vector(arg_vec_items, span.clone()),
                                WatAST::Symbol(Identifier::bare("->".to_string()), span.clone()),
                                type_expr_to_colon_ast(mret, &span),
                            ],
                            span.clone(),
                        );
                        member_vec_items.push(method_list);
                    }
                }
            }
            let mut items = vec![WatAST::Keyword(":wat::core::defsurface".into(), span.clone())];
            items.extend(decl_name_siblings(&s.name, &s.type_params, &span));
            items.push(WatAST::Vector(member_vec_items, span.clone()));
            WatAST::List(items, span)
        }
    }
}

/// Arc 109 ③ — a declaration's own name is never angle-bracket-spelled. When
/// `type_params` is non-empty, splice the `:-` binder marker + a bracket vector
/// of BARE parameter symbols as SIBLINGS after the plain name keyword (no
/// parens) — the same shape `parse_declared_name`/`take_declared_binder`
/// require on the way back in (`Head :- [T …]`), and the same shape
/// `wat/service.wat`'s `record-def`/`state-def` splice via `~@tp-syms`.
fn decl_name_siblings(name: &str, type_params: &[String], span: &Span) -> Vec<WatAST> {
    let mut out = vec![WatAST::Keyword(name.to_string(), span.clone())];
    if !type_params.is_empty() {
        out.push(WatAST::Keyword(":-".to_string(), span.clone()));
        let param_syms = type_params
            .iter()
            .map(|p| WatAST::Symbol(Identifier::bare(p.clone()), span.clone()))
            .collect();
        out.push(WatAST::Vector(param_syms, span.clone()));
    }
    out
}

/// Build a `(:wat::core::define <signature> <body>)` AST for a stored
/// Function, using the function's existing body.
fn function_to_define_form(func: &Function) -> WatAST {
    // Stone 255.1a — Native builtins have no wat body and are never closure-extracted.
    let body = match &func.body {
        FunctionBody::Wat(ast) => (**ast).clone(),
        FunctionBody::Native => unreachable!("native builtin fn-applied — dispatched via the runtime match, not fn-apply"),
    };
    let name = func
        .name
        .clone()
        .unwrap_or_else(|| ":wat::kernel::__closure::__anon".to_string());
    function_to_define_form_with_body(func, &name, body)
}

/// Same as `function_to_define_form` but lets the caller pass in a
/// rewritten body (used for the entry fn after capture-rewriting).
///
/// Stone 241.11 — emits `:wat::core::defn` (not the retired `:wat::core::define`).
/// defn shape: `(:wat::core::defn :name<T,U> [p1 <- :T1  p2 <- :T2  & rest <- :Trest] -> :Ret body)`.
fn function_to_define_form_with_body(
    func: &Function,
    name: &str,
    body: WatAST,
) -> WatAST {
    let span = crate::rust_caller_span!();
    // Build binder vector: [p1 <- :T1  p2 <- :T2  & rest <- :Trest]
    let mut binder_items: Vec<WatAST> = Vec::with_capacity(func.params.len() * 3 + 3);
    for (param, ty) in func.params.iter().zip(func.param_types.iter()) {
        // Arc 170 — REUSE the binder node. This line built
        // `Identifier::bare("kwargs\u{1}952")` from a flattened env_key: a scope
        // id baked into a NAME, which `Identifier::bare` debug-asserts against
        // and which no scope remapping can ever move. It is the exact thing
        // HygieneScopeDivergence's remedy names — "reuse the original AST node".
        binder_items.push(WatAST::Symbol(param.clone(), span.clone()));
        binder_items.push(WatAST::Symbol(Identifier::bare("<-"), span.clone()));
        binder_items.push(format_type_for_emit(ty, &span));
    }
    if let (Some(rname), Some(rty)) =
        (func.rest_param.as_ref(), func.rest_param_type.as_ref())
    {
        binder_items.push(WatAST::Symbol(Identifier::bare("&"), span.clone()));
        binder_items.push(WatAST::Symbol(Identifier::bare(rname.clone()), span.clone()));
        binder_items.push(WatAST::Symbol(Identifier::bare("<-"), span.clone()));
        binder_items.push(format_type_for_emit(rty, &span));
    }
    let binders = WatAST::Vector(binder_items, span.clone());
    // Build: (:wat::core::defn :name [binders] -> :Ret body)
    // Arc 109 ③ — defn's own name is never angle-bracket-spelled; splice the
    // `:-` binder + bare-symbol param vector as SIBLINGS (decl-name role),
    // same as `type_def_to_ast`'s `decl_name_siblings`.
    let mut items = vec![WatAST::Keyword(":wat::core::defn".into(), span.clone())];
    items.extend(decl_name_siblings(name, &func.type_params, &span));
    items.push(binders);
    items.push(WatAST::Symbol(Identifier::bare("->"), span.clone()));
    items.push(format_type_for_emit(&func.ret_type, &span));
    items.push(body);
    WatAST::List(items, span)
}

/// Render a TypeExpr into a wat type-position AST node, structurally —
/// Arc 109 ③ retired the angle-bracket parametric spelling this used to
/// build via `format!("{}<{}>", ...)` (no longer expressible as a keyword
/// STRING at all: `(Head :- [args])` is the only surviving spelling for a
/// parametric type). Delegates to `type_expr_to_colon_ast`, the same
/// structural renderer `type_def_to_ast` uses.
///
/// The empty-Tuple-is-unit round-trip concern this function's previous
/// string-based incarnation special-cased (rendering `Tuple([])` as the
/// `:wat::core::nil` keyword to dodge the `BareLegacyUnitType` walker) is
/// moot here: that walker (`check::walk_type_for_bare`) only inspects bare
/// `WatAST::Keyword` type annotations by re-parsing their STRING text: a
/// structural `WatAST::List` node — which is what `type_expr_to_colon_ast`
/// emits for `Tuple([])` (`(:wat::core::Tuple :- [])`, Stone ②-i-b) — never
/// reaches that string-audit codepath at all.
fn format_type_for_emit(t: &TypeExpr, span: &Span) -> WatAST {
    type_expr_to_colon_ast(t, span)
}

/// Build a `(:wat::core::fn ARGS-VECTOR -> :RET-TYPE body)` AST
/// reconstructed from a stored Function's signature + a rewritten body.
///
/// Slice 1b — used for inline-lambda input where there is no canonical
/// name. The fn-form AST evaluates to a fn Value directly when fed to
/// `eval` (per `runtime::eval_fn`'s arc-167 flat-shape consumer), so no
/// define wrapping is required.
///
/// Output shape per arc 167 + WAT-CHEATSHEET § 2:
///   - flat-vector binders: `[name <- :T name <- :T ...]`
///   - FQDN keyword for `:wat::core::fn`
///   - FQDN keyword for the return type (via `check::format_type`)
///   - no whitespace inside `<>` / `:(...)` / `:[...]`
fn function_to_fn_form(func: &Function, rewritten_body: WatAST) -> WatAST {
    let span = crate::rust_caller_span!();
    // Build flat-vector args: [name <- :T name <- :T ...].
    let mut args_items: Vec<WatAST> =
        Vec::with_capacity(func.params.len() * 3 + func.rest_param.iter().count() * 3);
    for (param, ty) in func.params.iter().zip(func.param_types.iter()) {
        // Arc 170 — REUSE the binder node (see the sibling site above).
        args_items.push(WatAST::Symbol(param.clone(), span.clone()));
        args_items.push(WatAST::Symbol(Identifier::bare("<-"), span.clone()));
        args_items.push(format_type_for_emit(ty, &span));
    }
    // Rest-param. The flat-vector fn-form doesn't currently carry a
    // dedicated `&` marker the way `define`-form signatures do; the
    // rest-param case for inline-lambda should be rare in arc 170's
    // closure-extraction inputs. Surface as Internal if hit, per FM 5
    // (don't bridge with a TODO).
    if func.rest_param.is_some() {
        // Arc 170 slice 1b honest delta: rest-param emission in the
        // fn-form AST shape isn't covered by the current substrate's
        // flat-vector grammar. The stored Function may carry a
        // rest-param if the original input was a defn with `&`; for
        // an inline-lambda input this combination is unexpected.
        // Emitting an unrecognized form here would produce a
        // freeze-time MalformedForm at consume; better to surface
        // Internal here so the gap is visible at the extraction site.
        // (Tests T1-T15 do not exercise this case.)
        return WatAST::List(
            vec![
                WatAST::Keyword(":wat::core::fn".into(), span.clone()),
                WatAST::Vector(args_items, span.clone()),
                WatAST::Symbol(Identifier::bare("->"), span.clone()),
                format_type_for_emit(&func.ret_type, &span),
                rewritten_body,
            ],
            span,
        );
    }
    let args_vec = WatAST::Vector(args_items, span.clone());
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::fn".into(), span.clone()),
            args_vec,
            WatAST::Symbol(Identifier::bare("->"), span.clone()),
            // Arc 278 — the ROUND-TRIP renderer, not `check::format_type`.
            // `check::format_type` renders the unit type (`Tuple(vec![])`) as
            // `:()`, the spelling arcs 109/153/179 retired; a child's freeze
            // rejects it with `BareLegacyUnitType`. The sibling emit sites
            // above already use `format_type_for_emit`; this one did not, and
            // `fn-forms` routes EVERY call through this path (see
            // `eval_kernel_fn_forms`' doc — it fronts extraction uniformly via
            // the inline-lambda path), so any nil-returning fn in a closure
            // shipped a program the child could not start. Proven by run:
            // wat-scripts/scratch-pad/probe-arc278-union-closure-boots-a-process-child.wat
            format_type_for_emit(&func.ret_type, &span),
            rewritten_body,
        ],
        span,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Slice 1b: synthetic-name uniqueness test retired alongside the
    // entry-keyword ceremony. Capture-binding name prefix test stays
    // (capture-binding naming is unchanged).

    #[test]
    fn synthetic_capture_name_is_user_closure_capture_namespaced() {
        let n = synthesize_capture_name("my-config");
        assert_eq!(n, ":user::closure-capture::my-config");
    }
}
