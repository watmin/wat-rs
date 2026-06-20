//! Arc 278 Stone 6a — purity classifier: `is_pure_expr` / `is_pure_fn`.
//!
//! Default-deny: a head is pure ONLY if it can be *proven* pure — a known-pure
//! intrinsic, or a user fn whose body is transitively pure. Anything unproven is
//! rejected. "Pure" = a deterministic function of the facts; impurity has two
//! sources: (1) effects (IO/spawn/mutation — `is_effectful_op`) and
//! (2) non-determinism (randomness — `:wat::core::Uuid/v4`).
//!
//! ## Entry point
//!
//! `(:wat::rete::pure? <quoted-expr>) -> :bool`
//! Dispatched from `runtime.rs` beside the sibling rete primitives.
//!
//! ## Cycle handling
//!
//! `is_pure_fn` threads a `seen: &mut HashSet<String>` of fqdns mid-evaluation.
//! A back-edge to an fqdn already in `seen` contributes no new impurity (purity
//! fixpoint: assume-pure on the cycle, falsify on any concrete impure leaf).

use crate::ast::WatAST;
use crate::runtime::{
    EvalBreak, Environment, FunctionBody, RuntimeError, RuntimeErrorKind, SymbolTable, Value,
    ValueSnapshot,
};
use crate::span::Span;
use std::collections::HashSet;
use std::sync::Arc;

// ─── Non-determinism seed ─────────────────────────────────────────────────────

/// ONLY `:wat::core::Uuid/v4` is random (non-deterministic).
/// `Uuid/v5` (SHA1 of namespace+name), `Uuid/from-string`, `Uuid/to-string`,
/// and `Uuid/nil` are all deterministic ⇒ pure; they belong on the allow-list.
fn is_nondeterministic(head: &str) -> bool {
    matches!(head, ":wat::core::Uuid/v4")
}

// ─── Pure-intrinsic allow-list ────────────────────────────────────────────────

/// Default-deny allow-list: a head is a known-pure intrinsic only if it appears
/// here. Enumerated from `dispatch_keyword_head_value` in `runtime.rs`.
///
/// Categories included:
/// - Pure namespace prefixes: `:wat::core::string::`, `:wat::core::regex::`.
/// - Pure `:wat::core::` ops: arithmetic, comparison, boolean, collection
///   readers/predicates, type predicates, and the deterministic Uuid ops.
/// - Pure control-flow forms: `if`, `let`, `do`, `match`, `cond`, `when` —
///   these recurse into their sub-forms element-wise.
/// - `quote` and `quasiquote` are handled separately (data, not calls).
fn is_pure_intrinsic(head: &str) -> bool {
    // Pure namespace prefixes — every op in these namespaces is pure.
    if head.starts_with(":wat::core::string::") {
        return true;
    }
    if head.starts_with(":wat::core::regex::") {
        return true;
    }
    // Explicit pure `:wat::core::` operations.
    matches!(
        head,
        // Arithmetic
        ":wat::core::+"
            | ":wat::core::-"
            | ":wat::core::*"
            | ":wat::core::/"
            | ":wat::core::i64::+"
            | ":wat::core::i64::-"
            | ":wat::core::i64::*"
            | ":wat::core::i64::/"
            | ":wat::core::i64::to-string"
            | ":wat::core::i64::to-f64"
            | ":wat::core::f64::+"
            | ":wat::core::f64::-"
            | ":wat::core::f64::*"
            | ":wat::core::f64::/"
            | ":wat::core::f64::abs"
            | ":wat::core::f64::max"
            | ":wat::core::f64::min"
            | ":wat::core::u8"
            // Comparison
            | ":wat::core::="
            | ":wat::core::not="
            | ":wat::core::<"
            | ":wat::core::>"
            | ":wat::core::<="
            | ":wat::core::>="
            // Boolean
            | ":wat::core::not"
            | ":wat::core::and"
            | ":wat::core::or"
            // Control flow whose sub-items are ALL plain exprs (or symbol/expr binding
            // vectors) — safe to recurse element-wise.
            | ":wat::core::if"
            | ":wat::core::let"
            | ":wat::core::do"
            | ":wat::core::when"
            // NOTE: `match`/`cond` are deterministic+pure control flow but their CLAUSE
            // sub-structure is not all-expr (a clause `(pattern body)` / `(test body)` has a
            // list as its head, which the element-wise walk would misclassify as impure).
            // Default-deny them until a clause-aware walk is added (6a follow-on, on a real
            // consumer need) — the safe direction: a `where` using them errors loudly rather
            // than the allow-list lying about handling them.
            // Collection/map/vector readers and predicates
            | ":wat::core::get"
            | ":wat::core::length"
            | ":wat::core::empty?"
            | ":wat::core::contains?"
            | ":wat::core::first"
            | ":wat::core::second"
            | ":wat::core::third"
            | ":wat::core::record?"
            | ":wat::core::str"
            // PersistentVector ops
            | ":wat::core::PersistentVector"
            | ":wat::core::PersistentVector/length"
            | ":wat::core::PersistentVector/empty?"
            | ":wat::core::PersistentVector/contains?"
            | ":wat::core::PersistentVector/get"
            | ":wat::core::PersistentVector/conj"
            // PersistentMap ops
            | ":wat::core::PersistentMap"
            | ":wat::core::PersistentMap/length"
            | ":wat::core::PersistentMap/empty?"
            | ":wat::core::PersistentMap/contains-key?"
            | ":wat::core::PersistentMap/get"
            | ":wat::core::PersistentMap/assoc"
            | ":wat::core::PersistentMap/dissoc"
            | ":wat::core::PersistentMap/keys"
            | ":wat::core::PersistentMap/values"
            // HashMap ops
            | ":wat::core::HashMap"
            | ":wat::core::HashMap/length"
            | ":wat::core::HashMap/empty?"
            | ":wat::core::HashMap/contains-key?"
            | ":wat::core::HashMap/get"
            | ":wat::core::HashMap/assoc"
            | ":wat::core::HashMap/dissoc"
            | ":wat::core::HashMap/keys"
            | ":wat::core::HashMap/values"
            // Deterministic Uuid ops (v5 = SHA1-based, from-string, to-string, nil are deterministic)
            | ":wat::core::Uuid/v5"
            | ":wat::core::Uuid/from-string"
            | ":wat::core::Uuid/to-string"
            | ":wat::core::Uuid/nil"
    )
}

// ─── Core classifier ──────────────────────────────────────────────────────────

/// Recursively classify an AST node as pure.
///
/// - Literals / keywords / symbols (incl. `?vars`) → pure (data, not a call).
/// - `quote`/`quasiquote` sub-forms → pure (data; do NOT recurse as calls).
/// - List with a keyword/symbol head H: apply the per-head decision.
/// - Vectors / maps / sets → recurse element-wise.
pub(crate) fn is_pure_expr(ast: &WatAST, sym: &SymbolTable, seen: &mut HashSet<String>) -> bool {
    match ast {
        // Non-list forms are pure data.
        WatAST::IntLit(_, _)
        | WatAST::FloatLit(_, _)
        | WatAST::BoolLit(_, _)
        | WatAST::StringLit(_, _)
        | WatAST::NilLit(_)
        | WatAST::Keyword(_, _)
        | WatAST::Symbol(_, _) => true,

        // quote / quasiquote sub-forms are DATA — pure (do not recurse into them as calls).
        WatAST::List(items, _) if matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::core::quote" || k == ":wat::core::quasiquote") => {
            true
        }

        // General list: extract head and apply per-head decision.
        WatAST::List(items, _) => {
            let head = match items.first() {
                // Empty list — pure (no head).
                None => return true,
                Some(WatAST::Keyword(k, _)) => k.as_str(),
                Some(WatAST::Symbol(id, _)) => id.as_str(),
                // Non-keyword/symbol head — treat as unknown → DENY.
                _ => return false,
            };

            // Step 1: effectful namespace seed.
            if crate::runtime::is_effectful_op(head) {
                return false;
            }

            // Step 2: non-deterministic intrinsics.
            if is_nondeterministic(head) {
                return false;
            }

            // Step 3: user-defined function → transitive check.
            if sym.functions.contains_key(head) {
                if !is_pure_fn(head, sym, seen) {
                    return false;
                }
                // Also recurse into the args (the call-site sub-expressions).
                return items[1..].iter().all(|a| is_pure_expr(a, sym, seen));
            }

            // Step 4: known-pure intrinsic → recurse into args.
            if is_pure_intrinsic(head) {
                return items[1..].iter().all(|a| is_pure_expr(a, sym, seen));
            }

            // Step 5: unknown head → DEFAULT-DENY.
            false
        }

        // Vectors → recurse element-wise.
        WatAST::Vector(elems, _) => elems.iter().all(|e| is_pure_expr(e, sym, seen)),

        // Maps → recurse element-wise over key-value pairs.
        WatAST::Map(pairs, _) => pairs
            .iter()
            .all(|(k, v)| is_pure_expr(k, sym, seen) && is_pure_expr(v, sym, seen)),

        // Sets → recurse element-wise.
        WatAST::Set(elems, _) => elems.iter().all(|e| is_pure_expr(e, sym, seen)),
    }
}

/// Classify a named user fn as pure by inspecting its body transitively.
///
/// `seen` is threaded to detect cycles; a back-edge (fqdn already in `seen`)
/// returns `true` (assume-pure: the cycle contributes no new impurity; the
/// purity fixpoint is falsified only if a concrete impure leaf is found).
pub(crate) fn is_pure_fn(fqdn: &str, sym: &SymbolTable, seen: &mut HashSet<String>) -> bool {
    // Back-edge: no new impurity from the recursive call.
    if seen.contains(fqdn) {
        return true;
    }
    seen.insert(fqdn.to_string());

    let func = match sym.functions.get(fqdn) {
        Some(f) => Arc::clone(f),
        // Name not in functions — unknown → DENY.
        None => return false,
    };

    match &func.body {
        FunctionBody::Wat(body_ast) => is_pure_expr(body_ast.as_ref(), sym, seen),
        // Native builtins registered in sym.functions are handled as pure
        // only if they also appear on is_pure_intrinsic; reaching here means
        // the fn is registered as Native without an intrinsic entry → DENY.
        FunctionBody::Native => false,
    }
}

// ─── Public 2-arg wrappers (for callers that don't thread `seen`) ─────────────

/// Classify an AST node as pure, seeding a fresh `seen` set.
/// Called by `eval_pure_predicate` and available to stone 6b+.
pub(crate) fn is_pure_expr_top(ast: &WatAST, sym: &SymbolTable) -> bool {
    is_pure_expr(ast, sym, &mut HashSet::new())
}

/// Classify a named user fn as pure, seeding a fresh `seen` set.
/// Stone 6b+ will call this from the rule-compiler.
#[allow(dead_code)]
pub(crate) fn is_pure_fn_top(fqdn: &str, sym: &SymbolTable) -> bool {
    is_pure_fn(fqdn, sym, &mut HashSet::new())
}

// ─── WAT surface: `(:wat::rete::pure? <quoted-expr>) -> :bool` ───────────────

/// Entry point dispatched by `runtime.rs` beside the sibling rete primitives.
///
/// Arity 1. Evaluates `args[0]` → expects `Value::wat__WatAST(a)` (the result
/// of a `quote` in the caller). Returns `Value::bool(is_pure_expr_top(a))`.
/// Pattern copied from `eval_alpha_match` in `matcher.rs`.
pub(crate) fn eval_pure_predicate(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::pure?";
    if args.len() != 1 {
        return Err(RuntimeError {
            span: list_span.clone(),
            kind: RuntimeErrorKind::ArityMismatch {
                op: OP.into(),
                expected: 1,
                got: args.len(),
            },
        }
        .into());
    }

    // Evaluate args[0] → must be Value::wat__WatAST (the quoted expr).
    let val = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let ast = match val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError {
                span: args[0].span().clone(),
                kind: RuntimeErrorKind::TypeMismatch {
                    op: OP.into(),
                    expected: ":wat::WatAST (a quoted expr from :wat::core::quote)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            }
            .into());
        }
    };

    Ok(Value::bool(is_pure_expr_top(&ast, sym)))
}
