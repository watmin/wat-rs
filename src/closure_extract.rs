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
use crate::identifier::Identifier;
use crate::runtime::{Function, StructValue, SymbolTable, Value};
use crate::span::Span;
use crate::types::{TypeDef, TypeEnv, TypeExpr};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::Arc;

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

/// Errors surfaced during extraction.
///
/// `NonPortableCapture` is the substrate-as-teacher rejection: a
/// captured value whose type is channel-bearing / IO / process-handle
/// cannot cross a process boundary because pointer-identity does not
/// survive `fork(2)`. The diagnostic names the offending capture, its
/// type, the field path inside (when nested), and points the user at
/// pipes / restructure.
#[derive(Debug, Clone)]
pub enum ExtractionError {
    /// A captured value of a non-portable type was found.
    NonPortableCapture {
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
    UnresolvedSymbol { name: String, span: Span },
    /// An internal invariant was violated. Not user-actionable; surfaces
    /// programmer bugs (e.g., `Function` carries no body span when one
    /// was expected).
    Internal(String),
}

impl std::fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractionError::NonPortableCapture {
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
                    "spawn-process closure captures `{}` of type `{}`{}.\n\
                     Channel-bearing types cannot cross process boundaries (different memory).\n\
                     Use stdin/stdout/stderr pipes for inter-process communication, or\n\
                     restructure the program so the channel is created in the spawned program.",
                    name, type_name, path_suffix
                )
            }
            ExtractionError::UnresolvedSymbol { name, span } => write!(
                f,
                "free symbol `{}` does not resolve to a parent define or substrate primitive (span: {})",
                name, span
            ),
            ExtractionError::Internal(msg) => write!(f, "closure-extract internal: {}", msg),
        }
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
            return Err(ExtractionError::Internal(format!(
                "extract_closure expected Value::wat__core__fn, got {}",
                other.type_name()
            )))
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
    let mut body_locals: BTreeSet<String> = func.params.iter().cloned().collect();
    if let Some(rest) = &func.rest_param {
        body_locals.insert(rest.clone());
    }
    walk_free_symbols(&func.body, &body_locals, &mut state)?;

    // Process captured locals from the fn's closed environment. Match
    // free symbols against the closed env to identify captures; their
    // names go into the captures map and the remaining frees are
    // reclassified or surfaced as unresolved.
    if let Some(closed_env) = &func.closed_env {
        let frees = std::mem::take(&mut state.unresolved_frees);
        for (name, span) in frees {
            if let Some(value) = closed_env.lookup(&name) {
                // It's a captured local. Encode the value to AST and
                // record. The captured value itself may carry types we
                // need to extract; the type-walk phase below handles
                // them.
                let encoded = encode_value_to_ast(&value, &name, &mut state)?;
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
        return Err(ExtractionError::UnresolvedSymbol { name, span });
    }

    // Recursively extract user deps + user types. Walks each dep AST
    // for further frees; chases types through their fields.
    extract_user_deps_to_fixpoint(&mut state)?;
    extract_user_types_to_fixpoint(&mut state)?;

    // Body rewrite: rewrite captured-local references in the entry
    // body from `X` to the synthetic capture name (a bare Symbol with
    // the substituted name). This avoids collision with extracted
    // user-symbol names and makes the bindings explicit. Runs BEFORE
    // entry_form is assembled so the rewritten body is what flows
    // into either the keyword-path entry-define (in prologue) or the
    // inline-lambda fn-form AST (entry_form).
    let rewritten_body = rewrite_captures(&func.body, &state.captured_bindings, &body_locals);

    // Assemble prologue in topological order:
    //   1. Type definitions (types in topological order)
    //   2. Captured-binding defines (`(def :__captured_X <encoded>)`)
    //   3. User dependency defines (in topological order)
    //   4. (keyword-path input only) the entry fn's define form, with
    //      rewritten body — appended after all deps. For inline-lambda,
    //      the entry never appears in prologue.
    let mut prologue: Vec<WatAST> = Vec::new();

    // 1. Types in deterministic topological order.
    let type_order = topo_sort_types(&state);
    for tn in &type_order {
        if let Some(def) = state.captured_types.get(tn) {
            prologue.push(type_def_to_ast(def));
        }
    }

    // 2. Captured-binding defines.
    for cb in &state.captured_bindings {
        prologue.push(capture_define_form(cb));
    }

    // 3. User defns in topological order.
    let dep_order = topo_sort_deps(&state);
    for dep_name in &dep_order {
        if let Some(dep_func) = state.captured_deps.get(dep_name) {
            prologue.push(function_to_define_form(dep_func));
        }
    }

    // 4. Entry resolution: keyword-path mode appends the entry
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
                function_to_define_form_with_body(&func, path, rewritten_body);
            prologue.push(entry_define);
            WatAST::Keyword(path.clone(), Span::unknown())
        }
        None => {
            // Inline lambda: emit the fn-form AST. The fn-form
            // evaluates to a fn Value at consumer time; no define
            // wrapping; no synthetic name.
            let _ = is_lambda; // documented above; kept for clarity
            function_to_fn_form(&func, rewritten_body)
        }
    };

    Ok(ClosurePackage {
        prologue,
        entry_form,
    })
}

// ─── Capture-binding name minting ───────────────────────────────────────
//
// Slice 1b retired the per-package synthetic-name counter
// (`:__closure::__pkg_<n>`) — entry naming no longer exists at the
// substrate level. Capture-binding names below are a separate
// concern (avoiding collision with extracted user-symbol names) and
// stay.

fn synthesize_capture_name(local: &str) -> String {
    // Captured locals are bare symbols. We emit them as bare-name
    // top-level keyword paths under `:__captured_<local>`. The body
    // rewrite swaps the bare-Symbol reference for a Keyword that
    // resolves to the def-bound value at runtime via `def`'s
    // runtime_def_values pathway.
    format!(":__captured_{}", sanitize_local_for_keyword(local))
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
        | WatAST::BoolLit(..)
        | WatAST::StringLit(..) => Ok(()),

        WatAST::Symbol(ident, span) => {
            let name = ident.name.clone();
            // Syntactic markers: `->` (return-type arrow), `<-` (input
            // direction arrow in fn signatures), `&` (rest-binder
            // marker), `:else` would never appear here as a Symbol.
            // These are NOT references and must not enter the
            // free-Symbol set.
            if matches!(name.as_str(), "->" | "<-" | "&") {
                return Ok(());
            }
            if !locals.contains(&name) {
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
            if let Some(ev) = state.parent_symbols.unit_variants.get(k) {
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
            // Else: a `def`-bound value? Unbound? For arc 170 slice 1 we
            // do not chase top-level defs (`runtime_def_values`); a
            // future arc opens IFF a caller surfaces wanting it. Check:
            // if it sits in runtime_def_values, surface as Internal so
            // tests reveal the gap; else silently skip — this keyword
            // may be a user-typed keyword literal at value position.
            if state.parent_symbols.runtime_def_values.contains_key(k) {
                return Err(ExtractionError::Internal(format!(
                    "captured `def`-bound name {} not yet supported by closure extraction (slice 1)",
                    k
                )));
            }
            // Treat as a keyword literal at value position (no
            // resolution required).
            Ok(())
        }

        WatAST::List(items, _span) => {
            // First: detect binding-introducing forms by head keyword.
            // We honor `:wat::core::let` and `:wat::core::fn` (and
            // `:wat::core::define` for completeness, though entry-fn
            // bodies don't usually contain a top-level define).
            if let Some((head, rest)) = items.split_first() {
                if let WatAST::Keyword(k, _) = head {
                    match k.as_str() {
                        ":wat::core::let" => {
                            return walk_let_form(rest, locals, state);
                        }
                        ":wat::core::fn" => {
                            return walk_fn_form(rest, locals, state);
                        }
                        ":wat::core::define" => {
                            return walk_define_form(rest, locals, state);
                        }
                        _ => {}
                    }
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

        WatAST::StructPattern(items, _span) => {
            // Field-name bare Symbols inside `{...}` are bindings, not
            // references. They're accumulated by the surrounding `let`
            // form's binder pass; we don't recurse into them here.
            // (But arc 169's StructPattern only legally appears as a
            // let binder, so this branch is defensive.)
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
                current_locals.insert(ident.name.clone());
            }
            WatAST::Vector(inner, _) => {
                for it in inner {
                    if let WatAST::Symbol(ident, _) = it {
                        current_locals.insert(ident.name.clone());
                    }
                }
            }
            WatAST::StructPattern(inner, _) => {
                for it in inner {
                    if let WatAST::Symbol(ident, _) = it {
                        current_locals.insert(ident.name.clone());
                    }
                }
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
                new_locals.insert(ident.name.clone());
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

/// Walk a `(:wat::core::define <signature> <body>)` form. Parameters
/// in the signature introduce scope for the body.
fn walk_define_form(
    args: &[WatAST],
    outer_locals: &BTreeSet<String>,
    state: &mut ExtractState<'_>,
) -> Result<(), ExtractionError> {
    if args.len() != 2 {
        for a in args {
            walk_free_symbols(a, outer_locals, state)?;
        }
        return Ok(());
    }
    let signature = &args[0];
    let body = &args[1];
    let mut new_locals = outer_locals.clone();
    if let WatAST::List(sig_items, _) = signature {
        // sig_items[0] is the function-name keyword; rest are
        // (param :Type) pairs and the trailing -> :Ret.
        for item in sig_items.iter().skip(1) {
            if let WatAST::List(pair, _) = item {
                if let Some(WatAST::Symbol(ident, _)) = pair.first() {
                    new_locals.insert(ident.name.clone());
                }
                // Walk type keywords inside the pair for type-ref extraction.
                for pi in pair.iter().skip(1) {
                    walk_free_symbols(pi, outer_locals, state)?;
                }
            } else if let WatAST::Keyword(_, _) = item {
                // Probably the `:Ret` type after `->`; walk for type-ref extraction.
                walk_free_symbols(item, outer_locals, state)?;
            }
        }
    }
    walk_free_symbols(body, &new_locals, state)?;
    Ok(())
}

// ─── Dep + type recording ───────────────────────────────────────────────

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
                .or_insert_with(BTreeSet::new)
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
    // and constructors like `:my::Point/new` fall under this rule. We
    // ALSO need to walk the type's signature for type-extraction so
    // the corresponding TypeDef makes it into the package.
    if let Some(slash_idx) = name.rfind('/') {
        let type_part = &name[..slash_idx];
        if state.parent_types.get(type_part).is_some()
            || state.captured_types.contains_key(type_part)
        {
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
    if let Some(colon2_idx) = name.rfind("::") {
        let enum_part = &name[..colon2_idx];
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
        .or_insert_with(BTreeSet::new);
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
        .or_insert_with(BTreeSet::new);
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
            let head_kw = format!(":{}", head);
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
                .ok_or_else(|| ExtractionError::Internal(format!("dep {} vanished", name)))?;
            let mut dep_locals: BTreeSet<String> =
                dep_func.params.iter().cloned().collect();
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
            // Set current_walking_dep so back-edges are recorded.
            let prior = state.current_walking_dep.replace(name.clone());
            let walk_result = walk_free_symbols(&dep_func.body, &dep_locals, state);
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
                return Err(ExtractionError::UnresolvedSymbol {
                    name: n,
                    span: sp,
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
                .ok_or_else(|| ExtractionError::Internal(format!("type {} vanished", name)))?;
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
                    .or_insert_with(BTreeSet::new)
                    .insert(dep_ty_name.clone());
            }
        }
    }
}

/// Pull out all `TypeExpr` references from a `TypeDef` (struct fields,
/// enum variant fields, newtype inner, alias target).
fn def_inner_typeexprs(def: &TypeDef) -> Vec<TypeExpr> {
    match def {
        TypeDef::Struct(s) => s.fields.iter().map(|(_, t)| t.clone()).collect(),
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
            let head_kw = format!(":{}", head);
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
/// `binding_name` is used in error messages (NonPortableCapture).
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
    let span = Span::unknown();
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
        Value::wat__core__keyword(k) => {
            // A wat-level keyword value is constructed via
            // `(:wat::core::keyword "literal-text")` — but the simpler
            // surface is to emit the bare keyword token. Emit literal
            // keyword token; eval's keyword arm produces the same value
            // for a stand-alone keyword that doesn't resolve to a
            // function.
            Ok(WatAST::Keyword((**k).clone(), span))
        }
        Value::Unit => Ok(WatAST::Keyword(":wat::core::nil".into(), span)),

        // ─── containers ────────────────────────────────────────────────
        Value::Vec(items) => {
            // `(:wat::core::Vector :T elem1 elem2 ...)` — infer T from
            // the first element. Empty Vec falls back to `:wat::core::nil`
            // (the singleton type), which type-checks against any
            // surface that doesn't dispatch on element type.
            let elem_kw = if let Some(first) = items.first() {
                value_static_type_keyword(first, state)?
            } else {
                ":wat::core::nil".to_string()
            };
            let mut out = Vec::with_capacity(items.len() + 2);
            out.push(WatAST::Keyword(":wat::core::Vector".into(), span.clone()));
            out.push(WatAST::Keyword(elem_kw, span.clone()));
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
            // `(:wat::core::HashMap :(K,V) k1 v1 k2 v2 ...)`. Determine K, V
            // from first entry; empty map falls back to `:(:wat::core::nil,:wat::core::nil)`.
            let pair_kw = if let Some((_canon, (k, vv))) = map.iter().next() {
                let kkw = value_static_type_keyword(k, state)?;
                let vkw = value_static_type_keyword(vv, state)?;
                format!(":({},{})", kkw, vkw)
            } else {
                ":(:wat::core::nil,:wat::core::nil)".to_string()
            };
            let mut out = Vec::with_capacity(map.len() * 2 + 2);
            out.push(WatAST::Keyword(":wat::core::HashMap".into(), span.clone()));
            out.push(WatAST::Keyword(pair_kw, span.clone()));
            // Iterate by sorted canonical key for determinism.
            let mut entries: Vec<(&String, &(Value, Value))> = map.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            for (canon_key, (k, vv)) in entries {
                path.push(format!("{{{}}}", canon_key));
                let kk = encode_value_with_path(k, binding_name, path, state)?;
                let vv2 = encode_value_with_path(vv, binding_name, path, state)?;
                path.pop();
                out.push(kk);
                out.push(vv2);
            }
            Ok(WatAST::List(out, span))
        }
        Value::wat__std__HashSet(set) => {
            let elem_kw = if let Some((_canon, v)) = set.iter().next() {
                value_static_type_keyword(v, state)?
            } else {
                ":wat::core::nil".to_string()
            };
            let mut out = Vec::with_capacity(set.len() + 2);
            out.push(WatAST::Keyword(":wat::core::HashSet".into(), span.clone()));
            out.push(WatAST::Keyword(elem_kw, span.clone()));
            let mut entries: Vec<(&String, &Value)> = set.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            for (canon_key, vv) in entries {
                path.push(format!("{{{}}}", canon_key));
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
        Value::Struct(sv) => {
            // `(:my::Type/new f1 f2 ...)`. Extract the struct's TypeDef
            // for inclusion in package.forms.
            ensure_type_extracted(state, &sv.type_name);
            encode_struct(sv, binding_name, path, state, span)
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

        // ─── non-portable arms ────────────────────────────────────────
        Value::crossbeam_channel__Sender(_)
        | Value::crossbeam_channel__Receiver(_)
        | Value::wat__kernel__ProgramHandle(_)
        | Value::wat__kernel__HandlePool { .. }
        | Value::wat__kernel__ChildHandle(_)
        | Value::io__IOReader(_)
        | Value::io__IOWriter(_)
        | Value::OnlineSubspace(_)
        | Value::Reckoner(_)
        | Value::Engram(_)
        | Value::EngramLibrary(_)
        | Value::Hologram(_) => Err(ExtractionError::NonPortableCapture {
            name: binding_name.to_string(),
            type_name: v.type_name().to_string(),
            path: path.clone(),
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
        | Value::Duration(_) => Err(ExtractionError::Internal(format!(
            "encoding for captured Value of kind {} not implemented in slice 1",
            v.type_name()
        ))),
    }
}

fn encode_struct(
    sv: &StructValue,
    binding_name: &str,
    path: &mut Vec<String>,
    state: &mut ExtractState<'_>,
    span: Span,
) -> Result<WatAST, ExtractionError> {
    // Pull field names from the TypeEnv (if available) for nicer path
    // diagnostics; positional order is what `<Type>/new` expects.
    let field_names: Option<Vec<String>> = state.parent_types.get(&sv.type_name).and_then(|td| {
        if let TypeDef::Struct(sd) = td {
            Some(sd.fields.iter().map(|(n, _)| n.clone()).collect())
        } else if let TypeDef::Newtype(_) = td {
            Some(vec!["0".to_string()])
        } else {
            None
        }
    });
    let constructor = format!("{}/new", sv.type_name);
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

/// The static type-keyword to emit as the type-arg for a Vec/HashMap
/// constructor based on a sample Value's tag. Conservatively returns
/// the FQDN of the value's runtime tag — the type-checker will
/// reconcile this against the captured-binding's downstream uses.
fn value_static_type_keyword(
    v: &Value,
    state: &mut ExtractState<'_>,
) -> Result<String, ExtractionError> {
    Ok(match v {
        Value::bool(_) => ":wat::core::bool".into(),
        Value::i64(_) => ":wat::core::i64".into(),
        Value::u8(_) => ":wat::core::u8".into(),
        Value::f64(_) => ":wat::core::f64".into(),
        Value::String(_) => ":wat::core::String".into(),
        Value::wat__core__keyword(_) => ":wat::core::keyword".into(),
        Value::Unit => ":wat::core::nil".into(),
        Value::Vec(items) => {
            let inner = if let Some(first) = items.first() {
                value_static_type_keyword(first, state)?
            } else {
                ":wat::core::nil".to_string()
            };
            // Vector head is the parametric form.
            format!(":wat::core::Vector<{}>", inner)
        }
        Value::Tuple(items) => {
            let mut parts = Vec::with_capacity(items.len());
            for it in items.iter() {
                parts.push(value_static_type_keyword(it, state)?);
            }
            format!(":({})", parts.join(","))
        }
        Value::Option(opt) => {
            let inner = match &**opt {
                Some(v) => value_static_type_keyword(v, state)?,
                None => ":wat::core::nil".to_string(),
            };
            format!(":wat::core::Option<{}>", inner)
        }
        Value::Result(res) => match &**res {
            Ok(v) => format!(":wat::core::Result<{},:wat::core::nil>", value_static_type_keyword(v, state)?),
            Err(e) => format!(":wat::core::Result<:wat::core::nil,{}>", value_static_type_keyword(e, state)?),
        },
        Value::Struct(sv) => {
            ensure_type_extracted(state, &sv.type_name);
            sv.type_name.clone()
        }
        Value::Enum(ev) => {
            ensure_type_extracted(state, &ev.type_path);
            ev.type_path.clone()
        }
        Value::wat__std__HashMap(_) => ":wat::core::HashMap".to_string(),
        Value::wat__std__HashSet(_) => ":wat::core::HashSet".to_string(),
        // Non-portable types — they should not be reaching here through
        // a portable container, but if they do, encoding fails through
        // the value-level path.
        other => format!(":{}", other.type_name()),
    })
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
        | WatAST::BoolLit(_, _)
        | WatAST::StringLit(_, _)
        | WatAST::Keyword(_, _) => node.clone(),

        WatAST::Symbol(ident, span) => {
            if !locals.contains(&ident.name) {
                if let Some(cb) = by_name.get(ident.name.as_str()) {
                    return WatAST::Keyword(cb.synthetic_name.clone(), span.clone());
                }
            }
            node.clone()
        }

        WatAST::List(items, span) => {
            // Recognize binding-introducing forms; preserve scope rules.
            if let Some((head, _)) = items.split_first() {
                if let WatAST::Keyword(k, _) = head {
                    if k == ":wat::core::let" {
                        return rewrite_let(items, by_name, locals, span.clone());
                    }
                    if k == ":wat::core::fn" {
                        return rewrite_fn(items, by_name, locals, span.clone());
                    }
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

        WatAST::StructPattern(items, span) => {
            // Field-name positions stay verbatim (they are bindings).
            WatAST::StructPattern(items.clone(), span.clone())
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
                        current_locals.insert(ident.name.clone());
                    }
                    WatAST::Vector(bv, _) => {
                        for it in bv {
                            if let WatAST::Symbol(ident, _) = it {
                                current_locals.insert(ident.name.clone());
                            }
                        }
                    }
                    WatAST::StructPattern(bv, _) => {
                        for it in bv {
                            if let WatAST::Symbol(ident, _) = it {
                                current_locals.insert(ident.name.clone());
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
                    new_locals.insert(ident.name.clone());
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
    // Use `(:wat::core::def :__captured_X <encoded>)` to bind the
    // captured value at top level. Per arc 157, def-bound names
    // resolve at the keyword arm of `eval` after unit_variants.
    let span = Span::unknown();
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::def".into(), span.clone()),
            WatAST::Keyword(cb.synthetic_name.clone(), span.clone()),
            cb.encoded_ast.clone(),
        ],
        span,
    )
}

fn type_def_to_ast(def: &TypeDef) -> WatAST {
    // Reconstruct the source-form for a TypeDef.
    let span = Span::unknown();
    match def {
        TypeDef::Struct(s) => {
            let mut items = vec![
                WatAST::Keyword(":wat::core::struct".into(), span.clone()),
                WatAST::Keyword(format_type_decl_name(&s.name, &s.type_params), span.clone()),
            ];
            for (fname, fty) in &s.fields {
                items.push(WatAST::List(
                    vec![
                        WatAST::Symbol(Identifier::bare(fname.clone()), span.clone()),
                        WatAST::Keyword(crate::check::format_type(fty), span.clone()),
                    ],
                    span.clone(),
                ));
            }
            WatAST::List(items, span)
        }
        TypeDef::Enum(e) => {
            let mut items = vec![
                WatAST::Keyword(":wat::core::enum".into(), span.clone()),
                WatAST::Keyword(format_type_decl_name(&e.name, &e.type_params), span.clone()),
            ];
            for variant in &e.variants {
                match variant {
                    crate::types::EnumVariant::Unit(name) => {
                        items.push(WatAST::Keyword(format!(":{}", name), span.clone()));
                    }
                    crate::types::EnumVariant::Tagged { name, fields } => {
                        let mut v_items = Vec::with_capacity(fields.len() + 1);
                        v_items.push(WatAST::Keyword(format!(":{}", name), span.clone()));
                        for (fname, fty) in fields {
                            v_items.push(WatAST::List(
                                vec![
                                    WatAST::Symbol(
                                        Identifier::bare(fname.clone()),
                                        span.clone(),
                                    ),
                                    WatAST::Keyword(
                                        crate::check::format_type(fty),
                                        span.clone(),
                                    ),
                                ],
                                span.clone(),
                            ));
                        }
                        items.push(WatAST::List(v_items, span.clone()));
                    }
                }
            }
            WatAST::List(items, span)
        }
        TypeDef::Newtype(n) => WatAST::List(
            vec![
                WatAST::Keyword(":wat::core::newtype".into(), span.clone()),
                WatAST::Keyword(format_type_decl_name(&n.name, &n.type_params), span.clone()),
                WatAST::Keyword(crate::check::format_type(&n.inner), span.clone()),
            ],
            span,
        ),
        TypeDef::Alias(a) => WatAST::List(
            vec![
                WatAST::Keyword(":wat::core::typealias".into(), span.clone()),
                WatAST::Keyword(format_type_decl_name(&a.name, &a.type_params), span.clone()),
                WatAST::Keyword(crate::check::format_type(&a.expr), span.clone()),
            ],
            span,
        ),
    }
}

fn format_type_decl_name(name: &str, type_params: &[String]) -> String {
    if type_params.is_empty() {
        name.to_string()
    } else {
        format!("{}<{}>", name, type_params.join(","))
    }
}

/// Build a `(:wat::core::define <signature> <body>)` AST for a stored
/// Function, using the function's existing body.
fn function_to_define_form(func: &Function) -> WatAST {
    let body = (*func.body).clone();
    let name = func
        .name
        .clone()
        .unwrap_or_else(|| ":wat::kernel::__closure::__anon".to_string());
    function_to_define_form_with_body(func, &name, body)
}

/// Same as `function_to_define_form` but lets the caller pass in a
/// rewritten body (used for the entry fn after capture-rewriting).
fn function_to_define_form_with_body(
    func: &Function,
    name: &str,
    body: WatAST,
) -> WatAST {
    let span = Span::unknown();
    let head_kw = if func.type_params.is_empty() {
        name.to_string()
    } else {
        format!("{}<{}>", name, func.type_params.join(","))
    };
    let mut sig_items: Vec<WatAST> = Vec::with_capacity(3 + func.params.len() * 2 + 4);
    sig_items.push(WatAST::Keyword(head_kw, span.clone()));
    for (param, ty) in func.params.iter().zip(func.param_types.iter()) {
        sig_items.push(WatAST::List(
            vec![
                WatAST::Symbol(Identifier::bare(param.clone()), span.clone()),
                WatAST::Keyword(crate::check::format_type(ty), span.clone()),
            ],
            span.clone(),
        ));
    }
    if let (Some(rname), Some(rty)) =
        (func.rest_param.as_ref(), func.rest_param_type.as_ref())
    {
        sig_items.push(WatAST::Symbol(Identifier::bare("&"), span.clone()));
        sig_items.push(WatAST::List(
            vec![
                WatAST::Symbol(Identifier::bare(rname.clone()), span.clone()),
                WatAST::Keyword(crate::check::format_type(rty), span.clone()),
            ],
            span.clone(),
        ));
    }
    sig_items.push(WatAST::Symbol(Identifier::bare("->"), span.clone()));
    sig_items.push(WatAST::Keyword(
        crate::check::format_type(&func.ret_type),
        span.clone(),
    ));
    let signature = WatAST::List(sig_items, span.clone());
    WatAST::List(
        vec![
            WatAST::Keyword(":wat::core::define".into(), span.clone()),
            signature,
            body,
        ],
        span,
    )
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
    let span = Span::unknown();
    // Build flat-vector args: [name <- :T name <- :T ...].
    let mut args_items: Vec<WatAST> =
        Vec::with_capacity(func.params.len() * 3 + func.rest_param.iter().count() * 3);
    for (param, ty) in func.params.iter().zip(func.param_types.iter()) {
        args_items.push(WatAST::Symbol(Identifier::bare(param.clone()), span.clone()));
        args_items.push(WatAST::Symbol(Identifier::bare("<-"), span.clone()));
        args_items.push(WatAST::Keyword(crate::check::format_type(ty), span.clone()));
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
                WatAST::Keyword(crate::check::format_type(&func.ret_type), span.clone()),
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
            WatAST::Keyword(crate::check::format_type(&func.ret_type), span.clone()),
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
    fn synthetic_capture_name_prefixes_double_underscore() {
        let n = synthesize_capture_name("my-config");
        assert!(n.starts_with(":__captured_"));
    }
}
