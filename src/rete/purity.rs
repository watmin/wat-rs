//! Arc 278 Stone 6a — the rete condition fence: TWO orthogonal classifiers, `pure?` + `deterministic?`.
//!
//! A rete condition (a `where`/`:test` predicate, an accumulator fn) must be a **deterministic,
//! effect-free function of the facts**. Those are two INDEPENDENT properties:
//!
//! - **pure** — effect-free: no IO/mutation/spawn (seed: the negation of `is_effectful_op`).
//! - **deterministic** — referentially transparent: same inputs → same output (no randomness/clock).
//!
//! They are genuinely orthogonal. `:wat::core::Uuid/v4` does no IO and mutates nothing → it is PURE,
//! yet it is random → NON-deterministic. The exposed rete check is therefore `(and (pure? f)
//! (deterministic? f))`; each axis is its own predicate.
//!
//! ## Default-deny, and the hand-managed metadata map
//!
//! Both classifiers are DEFAULT-DENY: a head's property holds only if PROVEN (a known intrinsic whose
//! metadata declares it, or a user fn whose body transitively holds it); anything unproven is rejected.
//! The per-op metadata is a small HAND-MANAGED map (`intrinsic_meta`) — the explicit v1 projection of
//! the queryable registry that arc 255 will eventually own (see
//! `docs/arc/2026/06/255-builtin-registry/NOTE-purity-is-definition-time-queryable-metadata.md`). When
//! 255 lands, delete this map and have the predicates query `metadata-of` instead.
//!
//! ## Entry points
//!
//! `(:wat::rete::pure? <quoted-expr>) -> :bool` · `(:wat::rete::deterministic? <quoted-expr>) -> :bool`
//! Dispatched from `runtime.rs` beside the sibling rete primitives.
//!
//! ## Cycle handling
//!
//! `classify_fn` threads a `seen: &mut HashSet<String>` of fqdns mid-evaluation. A back-edge to an fqdn
//! already in `seen` returns `true` (purity/determinism fixpoint: the cycle contributes no new
//! violation; the property is falsified only by a concrete violating leaf, which short-circuits up).

use crate::ast::WatAST;
use crate::runtime::{
    EvalBreak, Environment, FunctionBody, RuntimeError, RuntimeErrorKind, SymbolTable, Value,
    ValueSnapshot,
};
use crate::span::Span;
use std::collections::HashSet;
use std::sync::Arc;

// ─── The two axes ─────────────────────────────────────────────────────────────

/// The property being classified. The structural walk is shared; only the per-head leaf decision
/// (`head_ok`) differs by axis.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Axis {
    /// Effect-free: no IO/mutation/spawn.
    Pure,
    /// Referentially transparent: same inputs → same output.
    Deterministic,
}

// ─── The hand-managed per-op metadata map (v1 projection of arc 255) ───────────

/// Declared properties of a known intrinsic. The single hand source of truth until arc 255 lifts it
/// to a queryable registry. DEFAULT-DENY: a head NOT covered here returns `None` ⇒ neither property.
#[derive(Clone, Copy)]
struct OpMeta {
    pure: bool,
    deterministic: bool,
}

/// The hand-managed map (enumerated from `dispatch_keyword_head_value` in `runtime.rs`).
/// Almost every pure op is also deterministic; `Uuid/v4` is the lone pure-but-non-deterministic op.
fn intrinsic_meta(head: &str) -> Option<OpMeta> {
    // Pure but NON-deterministic: random.
    if head == ":wat::core::Uuid/v4" {
        return Some(OpMeta { pure: true, deterministic: false });
    }
    // Pure ∧ deterministic by namespace prefix — every op here is referentially transparent.
    if head.starts_with(":wat::core::string::") || head.starts_with(":wat::core::regex::") {
        return Some(OpMeta { pure: true, deterministic: true });
    }
    // The whole `:wat::edn::` namespace is pure data transforms — parse/serialize/navigate
    // (read, read-foreign, write, write-pretty, write-json, write-json-natural,
    // ForeignRecord/get, ForeignRecord/class, ForeignVariant/variant, ForeignVariant/enum-class,
    // ForeignVariant/fields), no IO, no entropy. Root-level by namespace, not a per-verb
    // hand-list — the next foreign verb slips past a hand-list.
    if head.starts_with(":wat::edn::") {
        return Some(OpMeta { pure: true, deterministic: true });
    }
    // Pure ∧ deterministic explicit `:wat::core::` ops.
    let pure_det = matches!(
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
            // Arc 300 stone C1 — bigint arithmetic + conversions.
            | ":wat::core::bigint::+"
            | ":wat::core::bigint::-"
            | ":wat::core::bigint::*"
            | ":wat::core::bigint::/"
            | ":wat::core::i64::to-bigint"
            | ":wat::core::bigint::to-f64"
            // Arc 300 stone C2 — rational arithmetic + conversions.
            | ":wat::core::rational::+"
            | ":wat::core::rational::-"
            | ":wat::core::rational::*"
            | ":wat::core::rational::/"
            | ":wat::core::i64::to-rational"
            | ":wat::core::bigint::to-rational"
            | ":wat::core::rational::to-f64"
            | ":wat::core::rational/numerator"
            | ":wat::core::rational/denominator"
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
            // Control flow whose sub-items are ALL plain exprs (or symbol/expr binding vectors)
            // — safe to recurse element-wise. (`cond`/`match` are handled with dedicated
            // clause-aware arms in classify_expr, NOT here, because their clauses are not calls.)
            | ":wat::core::if"
            | ":wat::core::let"
            | ":wat::core::do"
            | ":wat::core::when"
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
            // Deterministic Uuid ops (v5 = SHA1(ns,name); from-string/to-string/nil)
            | ":wat::core::Uuid/v5"
            | ":wat::core::Uuid/from-string"
            | ":wat::core::Uuid/to-string"
            | ":wat::core::Uuid/nil"
            // Higher-order fold combinators — CONDITIONALLY pure∧det: the combinator itself is
            // referentially transparent + effect-free; its purity/determinism falls out of the
            // arg-recursion over its fn-argument (classify_expr recurses every arg, incl. the
            // fn-literal, whose body is classified by the `:wat::core::fn` arm). An impure fn-arg
            // therefore still fails — conditional purity, not blanket-allow.
            | ":wat::core::foldl"
            | ":wat::core::foldr"
            | ":wat::core::map"
            | ":wat::core::filter"
            | ":wat::core::reduce"
    );
    if pure_det {
        Some(OpMeta { pure: true, deterministic: true })
    } else {
        None
    }
}

// ─── Per-head leaf decision ─────────────────────────────────────────────────────

/// Data constructors are pure∧deterministic BY CONSTRUCTION — they build a value, no effects, no
/// entropy — EXCEPT a struct constructor: a struct can hold a live resource (the wire-wall, arc 293.W),
/// so it is NOT pure (still deterministic). Mirrors the canonical `is_pure_type` (check.rs): an
/// Aggregate's purity is `Nature::is_pure()` (Record/HolonRecord pure, Struct impure); an enum's is its
/// declared `:wat::enum::*` marker (`EnumDef.purity`). INTERIM recognizer keyed on the frozen TypeEnv,
/// until arc 255's builtin-registry becomes the single queryable purity source and subsumes it.
fn constructor_meta(head: &str, sym: &SymbolTable) -> Option<OpMeta> {
    let types = sym.types.as_deref()?;
    // TypeEnv keys carry the leading colon (e.g. ":p::Rec") — use the head verbatim.
    // 1. Aggregate constructor (record / holon / struct) — the head IS the type name.
    if let Some(crate::types::TypeDef::Aggregate(a)) = types.get(head) {
        return Some(OpMeta { pure: a.nature.is_pure(), deterministic: true });
    }
    // 2. Enum-variant constructor — the head is `{EnumPath}::{Variant}` (unit or tagged).
    if let Some((enum_path, variant)) = head.rsplit_once("::") {
        if let Some(crate::types::TypeDef::Enum(e)) = types.get(enum_path) {
            let is_variant = e.variants.iter().any(|v| match v {
                crate::types::EnumVariant::Unit(n) => n == variant,
                crate::types::EnumVariant::Tagged { name, .. } => name == variant,
            });
            if is_variant {
                return Some(OpMeta { pure: e.purity.is_pure(), deterministic: true });
            }
        }
    }
    None
}

/// A generated field ACCESSOR (`{TypePath}/{field}`) is as pure as the aggregate it reads: a
/// Record/HolonRecord accessor is pure ∧ deterministic, a Struct accessor is impure (a struct can
/// hold a live resource, arc 293.W) — the exact declaration `constructor_meta` / `is_pure_type`
/// reads. Declaration-read from the frozen TypeEnv (resolve the type, don't string-match), so it
/// covers every user record; NOT a hand-list. INTERIM recognizer keyed on the frozen TypeEnv, until
/// arc 255's builtin-registry becomes the single queryable purity source and subsumes it.
fn accessor_meta(head: &str, sym: &SymbolTable) -> Option<OpMeta> {
    let types = sym.types.as_deref()?;
    // Accessors register as `{agg.name}/{field}` (runtime.rs); `agg.name` carries the leading
    // colon (e.g. ":wat::telemetry::Log"), so the type-path splits off verbatim for `types.get`.
    let (type_path, field) = head.rsplit_once('/')?;
    if let Some(crate::types::TypeDef::Aggregate(a)) = types.get(type_path) {
        if a.field_names().any(|n| n == field) {
            return Some(OpMeta { pure: a.nature.is_pure(), deterministic: true });
        }
    }
    // Enum-variant field accessors (tagged-variant field readers) are left to None here — their
    // head shape is not the flat `Type/field` form resolved above (STOP-3); the RED gate covers
    // records, and forcing the enum case is out of scope.
    None
}

/// Does `head` satisfy `axis`? Data constructors and field accessors are recognized first
/// (pure-by-declaration, interim pre-255); then user fns recurse transitively; intrinsics consult
/// `intrinsic_meta`; unknown heads default-deny.
fn head_ok(head: &str, axis: Axis, sym: &SymbolTable, seen: &mut HashSet<String>) -> bool {
    // Data constructor (record/holon/enum-variant pure; struct impure) — recognized BEFORE the
    // sym.functions branch, because tagged-variant constructors are registered there as opaque stubs
    // that classify_fn would default-deny.
    if let Some(m) = constructor_meta(head, sym) {
        return match axis {
            Axis::Pure => m.pure,
            Axis::Deterministic => m.deterministic,
        };
    }
    // Generated field accessor (`Type/field`) — same declaration-read as constructors, and likewise
    // BEFORE the sym.functions branch: accessors register there as Native stubs that classify_fn
    // default-denies, so we MUST intercept the accessor here.
    if let Some(m) = accessor_meta(head, sym) {
        return match axis {
            Axis::Pure => m.pure,
            Axis::Deterministic => m.deterministic,
        };
    }
    // User-defined fn → transitive check of its body on the SAME axis.
    if sym.functions.contains_key(head) {
        return classify_fn(head, axis, sym, seen);
    }
    match axis {
        // Pure: effectful namespaces are an explicit deny; otherwise the metadata must declare pure.
        Axis::Pure => {
            if crate::runtime::is_effectful_op(head) {
                return false;
            }
            intrinsic_meta(head).is_some_and(|m| m.pure)
        }
        // Deterministic: the metadata must declare deterministic (effectful/unknown ⇒ None ⇒ deny,
        // which is correct — IO and unknown ops are not referentially transparent).
        Axis::Deterministic => intrinsic_meta(head).is_some_and(|m| m.deterministic),
    }
}

// ─── Shared structural walk (parameterized by axis) ─────────────────────────────

/// Recursively classify an AST node against `axis`. The structure (quote-as-data, clause-aware
/// `cond`/`match`, element-wise vectors/maps/sets) is identical for both axes; only `head_ok` differs.
fn classify_expr(ast: &WatAST, axis: Axis, sym: &SymbolTable, seen: &mut HashSet<String>) -> bool {
    match ast {
        // Non-list forms are pure, deterministic data.
        WatAST::IntLit(_, _)
        | WatAST::FloatLit(_, _)
        // Arc 300 stone B — rational literal is pure, deterministic data.
        | WatAST::RationalLit(_, _)
        // Arc 300 stone C1 — bigint literal is pure, deterministic data too.
        | WatAST::BigIntLit(_, _)
        | WatAST::BoolLit(_, _)
        | WatAST::StringLit(_, _)
        | WatAST::NilLit(_)
        | WatAST::Keyword(_, _)
        | WatAST::Symbol(_, _) => true,

        // quote / quasiquote / holon-literal sub-forms are DATA — do not recurse into them as calls.
        // Arc 294.b: `:wat::holon::literal` is pure (it captures data, no side-effects).
        WatAST::List(items, _) if matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::core::quote" || k == ":wat::core::quasiquote" || k == ":wat::holon::literal") => {
            true
        }

        // `cond` — clause-aware: (cond (test body…) …). A clause is NOT a call; every element
        // (test AND body forms) is an expression that must satisfy the axis. (cond ≡ chained `if`.)
        WatAST::List(items, _) if matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::core::cond") => {
            items[1..].iter().all(|clause| match clause {
                WatAST::List(parts, _) => parts.iter().all(|e| classify_expr(e, axis, sym, seen)),
                _ => false, // malformed clause → deny
            })
        }

        // `match` — clause-aware: (match scrut (pattern body…) …). The scrutinee and every arm
        // BODY must satisfy the axis; the PATTERN is structural (destructures/binds, never calls — wat
        // match has no guards). Arc 258.5 — bare match: scrutinee = items[1], arms = items[2..]
        // (the `-> :T` ascription is retired). Skip the pattern (arm element 0), check the body
        // (arm elements 1..).
        WatAST::List(items, _) if matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::core::match") => {
            let scrut_ok = items.get(1).is_some_and(|s| classify_expr(s, axis, sym, seen));
            scrut_ok
                && items.get(2..).is_some_and(|arms| {
                    arms.iter().all(|arm| match arm {
                        // skip pattern (element 0); check body forms (1..).
                        WatAST::List(parts, _) => {
                            parts.iter().skip(1).all(|e| classify_expr(e, axis, sym, seen))
                        }
                        _ => false, // malformed arm → deny
                    })
                })
        }

        // `:wat::core::fn` lambda literal — NOT a call. Layout: (fn [params…] -> :ret body…).
        // The param vector + return-type are not evaluated; only the BODY forms (after the `-> :ret`
        // ascription) carry effects, so classify exactly those. Mirror the `match`-arm's logic:
        // locate the top-level `->` symbol, then body = items[i+2..] (skip `->` and :ret).
        WatAST::List(items, _) if matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::core::fn") => {
            match items.iter().position(|it| matches!(it, WatAST::Symbol(s, _) if s.as_str() == "->")) {
                Some(i) => items
                    .get(i + 2..)
                    .is_some_and(|body| body.iter().all(|e| classify_expr(e, axis, sym, seen))),
                None => false, // malformed fn (no `->`) → deny
            }
        }

        // General list: head decision + recurse into args (same axis).
        WatAST::List(items, _) => {
            let head = match items.first() {
                None => return true, // empty list — no call
                Some(WatAST::Keyword(k, _)) => k.as_str(),
                Some(WatAST::Symbol(id, _)) => id.as_str(),
                _ => return false, // non-keyword/symbol head — unknown → deny
            };
            head_ok(head, axis, sym, seen)
                && items[1..].iter().all(|a| classify_expr(a, axis, sym, seen))
        }

        // Vectors / maps / sets → recurse element-wise.
        WatAST::Vector(elems, _) => elems.iter().all(|e| classify_expr(e, axis, sym, seen)),
        WatAST::Map(pairs, _) => pairs
            .iter()
            .all(|(k, v)| classify_expr(k, axis, sym, seen) && classify_expr(v, axis, sym, seen)),
        WatAST::Set(elems, _) => elems.iter().all(|e| classify_expr(e, axis, sym, seen)),
    }
}

/// Classify a named user fn against `axis` by inspecting its body transitively. `seen` detects cycles;
/// a back-edge (fqdn already in `seen`) returns `true` (fixpoint: the cycle adds no new violation).
fn classify_fn(fqdn: &str, axis: Axis, sym: &SymbolTable, seen: &mut HashSet<String>) -> bool {
    if seen.contains(fqdn) {
        return true; // back-edge — no new violation from the recursive call
    }
    seen.insert(fqdn.to_string());

    let func = match sym.functions.get(fqdn) {
        Some(f) => Arc::clone(f),
        None => return false, // name not registered → deny
    };
    match &func.body {
        FunctionBody::Wat(body_ast) => classify_expr(body_ast.as_ref(), axis, sym, seen),
        // A native builtin registered in sym.functions is opaque — its body cannot be inspected —
        // so consult the hand-managed intrinsic_meta on the requested axis. This is load-bearing
        // for the HOF combinators (foldl/map/…): they are native AND registered in sym.functions,
        // so head_ok reaches classify_fn FIRST (before intrinsic_meta on the Pure/Det fallthrough).
        // Unproven natives (not in intrinsic_meta) still default-deny.
        FunctionBody::Native => intrinsic_meta(fqdn).is_some_and(|m| match axis {
            Axis::Pure => m.pure,
            Axis::Deterministic => m.deterministic,
        }),
    }
}

// ─── Public axis classifiers (fresh `seen` per call) — also for stone 6b+ ──────

/// Is `ast` effect-free (no IO/mutation/spawn)? `:wat::core::Uuid/v4` is pure (it does no IO).
pub(crate) fn is_pure_expr(ast: &WatAST, sym: &SymbolTable) -> bool {
    classify_expr(ast, Axis::Pure, sym, &mut HashSet::new())
}

/// Is `ast` referentially transparent (same inputs → same output)? `:wat::core::Uuid/v4` is NOT.
pub(crate) fn is_deterministic_expr(ast: &WatAST, sym: &SymbolTable) -> bool {
    classify_expr(ast, Axis::Deterministic, sym, &mut HashSet::new())
}

// ─── WAT surfaces ───────────────────────────────────────────────────────────────

/// Shared body for the two single-arg WatAST predicates: arity 1, eval `args[0]` to a quoted
/// `WatAST`, apply `classify`. Pattern copied from `eval_alpha_match` in `matcher.rs`.
fn eval_axis_predicate(
    op: &'static str,
    classify: fn(&WatAST, &SymbolTable) -> bool,
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError {
            span: list_span.clone(),
            kind: RuntimeErrorKind::ArityMismatch { op: op.into(), expected: 1, got: args.len() },
        }
        .into());
    }
    let val = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let ast = match val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError {
                span: args[0].span().clone(),
                kind: RuntimeErrorKind::TypeMismatch {
                    op: op.into(),
                    expected: ":wat::WatAST (a quoted expr from :wat::core::quote)",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            }
            .into());
        }
    };
    Ok(Value::bool(classify(&ast, sym)))
}

/// `(:wat::rete::pure? <quoted-expr>) -> :bool` — effect-free?
pub(crate) fn eval_pure_predicate(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    eval_axis_predicate(":wat::rete::pure?", is_pure_expr, args, list_span, env, sym)
}

/// `(:wat::rete::deterministic? <quoted-expr>) -> :bool` — referentially transparent?
pub(crate) fn eval_deterministic_predicate(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    eval_axis_predicate(":wat::rete::deterministic?", is_deterministic_expr, args, list_span, env, sym)
}
