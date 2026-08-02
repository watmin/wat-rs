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
///
/// ⚠ **HAND-MANAGED IS THE DEFECT, and it is a STEM fix — the root is arc 255.** This is one list
/// transcribed from another, so a verb minted in `dispatch_keyword_head_value` is silently
/// *unclassified* here, and unclassified means a rule that uses it **cannot compile**
/// (`compile-condition` panics on `pure? = false`, `wat/rete.wat:566`). Nothing detects that; only
/// a user hitting it does. The 2026-08-01 sweep below closed 35 such verbs, including the entire
/// `String/` family. **The wall is purity declared where the verb is DEFINED**, so a verb cannot
/// exist unclassified — arc 255's builtin registry, already named as this recognizer's successor in
/// `constructor_meta`'s doc below. Until then, adding a verb to the dispatch table means adding it
/// here, and nothing enforces that.
///
/// **Deliberately left UNCLASSIFIED by the 2026-08-01 sweep** (named so the omission is visible
/// rather than silent — each wants a ruling, not an assumption):
/// `apply` (purity depends on the fn it is handed); `struct-new` / `struct-field` / `struct->form`
/// (a struct may hold a live resource — arc 293.W); `Record/assoc` / `Record/field-at` /
/// `record->map` / `to-record`; the AST/meta family (`read-string`, `macroexpand`, `forms`,
/// `ast->*`, `with-children`, `write-forms`); `Option/expect` / `Result/expect` (total but they
/// raise); and the generic sequence verbs (`range`, `take`, `drop`, `rest`, `last`, `assoc`,
/// `conj`, `find-last-index`) which are very likely pure but were not measured this session.
///
/// **The other 101 `:wat::holon::` verbs are also deliberately unclassified.** Three groups, and
/// the middle one is the interesting question, not an oversight:
/// - *the threshold siblings* — `coincident-floor`, `presence-floor`, `coincident-explain`: the
///   same reads with an explicit floor / an explanation payload. Almost certainly pure; the builder
///   ruled the four above as the set to open NOW, and these are the obvious next ask if a user
///   wants a per-rule threshold rather than the configured one.
/// - *the LEARNING ops* — `OnlineSubspace/update`, `EngramLibrary/add`, `Hologram/put|remove`,
///   `Reckoner/observe`: they return new values rather than mutating, so they may well be pure,
///   but a learning step inside a re-fired rete predicate is a semantics question before it is a
///   purity question. Wants a ruling, not a classification.
/// - *the `eval-*-coincident?` family* — these evaluate submitted forms; purity depends on what
///   they are handed, exactly like `apply`.
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
            // ── 2026-08-01: the EXPRESSIVITY GAP, closed by hand ──────────────────────────────
            //
            // Found by measuring the fence against what a USER MAY WRITE rather than against our
            // own corpus (the builder: "do not optimize for our code — optimize for the users we
            // have not encountered yet"). Diffing `dispatch_keyword_head_value`'s arms against this
            // list: 221 dispatch arms, 96 classified. Our rulesets happened to use only the
            // classified subset — generic `>` everywhere, never a String verb in a `where` — so
            // nothing ever tripped it.
            //
            // The cost was real and hard: `compile-condition` PANICS on `pure? = false`
            // (`wat/rete.wat:566`), so `(:wat::rete::where (:wat::core::i64::> ?bytes 10000))` and
            // `(:wat::rete::where (:wat::core::String/starts-with? ?path "/adm"))` were BOTH
            // uncompilable — and the same gate fences the sift `Sieve::Predicate` form, so this
            // constrained the chaos engine's server-side filter too, not just rete rules. It also
            // propagates: a user's own `defn` predicate inherits the gap invisibly through its body.
            //
            // Each entry below is a value operation over already-evaluated arguments: no IO, no
            // entropy, no mutation, same inputs → same output. Grouped by the family whose ABSENCE
            // was the actual defect — the transcription had added the per-Type ARITHMETIC leaves
            // and the Persistent* containers, then stopped.

            // Per-Type COMPARISON leaves. `i64::+ - * /` were here; `i64::< > <= >=` were not —
            // an inconsistency inside one family, and the single most common thing a rule says.
            | ":wat::core::i64::<"  | ":wat::core::i64::<=" | ":wat::core::i64::>" | ":wat::core::i64::>="
            | ":wat::core::f64::<"  | ":wat::core::f64::<=" | ":wat::core::f64::>" | ":wat::core::f64::>="
            // Per-Type integer division family (`i64::/` was already here; its siblings were not).
            | ":wat::core::i64::mod" | ":wat::core::i64::quot" | ":wat::core::i64::rem"
            // f64 numeric readers/roundings — total functions of their argument.
            | ":wat::core::f64::round" | ":wat::core::f64::clamp"
            | ":wat::core::f64::max-of" | ":wat::core::f64::min-of"
            | ":wat::core::f64::to-i64" | ":wat::core::f64::to-string"
            // The `String/` family — ENTIRELY absent. Note `:wat::core::string::` (lowercase, a
            // namespace) is whitelisted by prefix above; `String/` is the per-Type family users
            // actually call, and it is a different namespace, so the prefix never covered it.
            | ":wat::core::String/concat"      | ":wat::core::String/contains?"
            | ":wat::core::String/empty?"      | ":wat::core::String/starts-with?"
            | ":wat::core::String/ends-with?"
            // `Vector/`, `List/`, `HashSet/` — the value containers. `PersistentVector/`,
            // `PersistentMap/` and `HashMap/` were classified; these three were skipped.
            | ":wat::core::Vector"        | ":wat::core::Vector/length"   | ":wat::core::Vector/get"
            | ":wat::core::Vector/conj"   | ":wat::core::Vector/contains?" | ":wat::core::Vector/empty?"
            | ":wat::core::Vector/concat" | ":wat::core::Vector/extend"
            | ":wat::core::List?"         | ":wat::core::List/of"         | ":wat::core::List/length"
            | ":wat::core::List/get"      | ":wat::core::List/conj"       | ":wat::core::List/contains?"
            | ":wat::core::List/empty?"
            | ":wat::core::HashSet"       | ":wat::core::HashSet/length"  | ":wat::core::HashSet/conj"
            | ":wat::core::HashSet/contains?" | ":wat::core::HashSet/empty?"
            // The persistent sibling the `into` stone minted; its `Vector/extend` twin is above.
            | ":wat::core::PersistentVector/concat"
            // Scalar conversions — total, same-in-same-out.
            | ":wat::core::bool::to-string"
            | ":wat::core::i64/to-f64" | ":wat::core::i64/to-string"
            // Uuid READERS (contrast `Uuid/v4`, which is pure but NON-deterministic and is handled
            // by its own arm at the top): these read bits out of a value already in hand.
            | ":wat::core::Uuid/version" | ":wat::core::Uuid/rfc4122-variant?"

            // ── The VSA SEAM — `:wat::holon::` (builder-ruled, 2026-08-01: these four) ─────────
            //
            // ZERO of the 105 `:wat::holon::` verbs were classified, so R4's designed seam — *"swap
            // RETE's exact test for COINCIDENCE, similarity over a floor, so rules fire on
            // resemblance, not equality"* — was welded shut: a rule doing a VSA op mid-fire could
            // not compile. `DESIGN-sift-server-side-filter.md` calls for exactly this ("a holon
            // fact carries its Hologram → a rule can do a VSA op mid-fire"), and the same fence
            // gates the sift `Sieve::Predicate`, so it was shut on both paths.
            //
            // All four READ two values already in hand and return a scalar or a bool — no IO, no
            // entropy, no mutation. `coincident?`/`presence?` are the bool predicates a `where`
            // can use directly; `cosine`/`dot` are the scalars, usable because the f64 comparisons
            // above landed in the same sweep — `(:wat::core::f64::> (:wat::holon::cosine ?a ?b) 0.9)`
            // now composes, where before BOTH halves were unclassified.
            | ":wat::holon::cosine" | ":wat::holon::dot"
            | ":wat::holon::coincident?" | ":wat::holon::presence?"
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
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch { op: op.into(), expected: 1, got: args.len() })
        .into());
    }
    let val = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let ast = match val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                    op: op.into(),
                    expected: ":wat::WatAST (a quoted expr from :wat::core::quote)",
                    got: Box::new(ValueSnapshot::of(&other)),
                })
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

// ─── The purity-COMPLETENESS gate (arc 278, 2026-08-01) ─────────────────────────

/// Every builtin verb the runtime dispatches is either CLASSIFIED by `intrinsic_meta`, or carries
/// an explicit DISPOSITION saying why nobody has ruled on it yet. A verb that is neither is the
/// state this gate exists to make impossible.
///
/// ## Why this gate exists — the defect it catches, which shipped
///
/// `intrinsic_meta` is, by its own doc, "hand-managed (enumerated from
/// `dispatch_keyword_head_value`)" — one list transcribed from another. A verb minted in the
/// dispatch table is therefore silently *unclassified* here, and unclassified is not a harmless
/// default: `compile-condition` **panics** on `pure? = false` (`wat/rete.wat:566`), so a rule using
/// it **cannot compile**. Nothing detected that. On 2026-08-01 it had accumulated to 35 verbs,
/// including every `i64`/`f64` comparison, the entire `String/` family, and all 105
/// `:wat::holon::` verbs — which welded shut R4's designed VSA seam. It was found by a user-shaped
/// probe, not by any test, and it had been true for months.
///
/// ## The safety asymmetry — PURE is never inferred
///
/// A wrong "impure" costs expressivity and is visible the moment someone tries to write the rule.
/// A wrong "pure" lets an effectful call into a predicate that gets re-fired and sandboxed. So:
/// **`Pure` comes only from `intrinsic_meta`, per verb, by hand.** Dispositions below may say
/// `Impure` or `Unreviewed` — never `Pure`. The gate cannot be satisfied by widening a prefix.
///
/// ## Why it is a RATCHET, not a zero-floor
///
/// Classifying the remaining verbs is a per-verb semantic ruling — the builder's, not a bulk
/// inference from reading names, which is the exact error that produced the 35. So the gate asserts
/// the unreviewed count **does not GROW**: a verb added to the dispatch table without a
/// classification or a disposition pushes it over the baseline and goes red. The list is printed so
/// the worklist is visible, the same shape `no_inlined_edn` and the clippy campaign used.
/// `[[feedback_no_consumers_does_not_mean_dead]]` applies: this reports an inventory needing
/// dispositions; it never proposes deleting a verb.
///
/// The root remains arc 255 — purity declared where the verb is *defined*, so the transcription
/// step disappears. This gate is what holds the line until then.
#[cfg(test)]
mod completeness_gate {
    use super::intrinsic_meta;

    #[derive(Clone, Copy, PartialEq)]
    enum Disp {
        /// Structurally cannot be pure. Only used where the namespace's whole reason for existing
        /// is the effect.
        Impure,
        /// Nobody has ruled. Counts toward the worklist — this is the honest default, and saying
        /// "impure" instead would hide a capability gap exactly the way the 35 were hidden.
        Unreviewed,
    }

    /// Namespace rules. Conservative by construction: `Impure` only where the namespace IS the
    /// effect; everything else is `Unreviewed` with what needs checking named.
    const RULES: &[(&str, Disp, &str)] = &[
        (":wat::io::", Disp::Impure, "IO is the namespace's entire purpose"),
        (":wat::kernel::", Disp::Impure, "the effect surface — println/eprintln/readln/assertion-failed!"),
        (":wat::config::", Disp::Impure, "mutates per-runtime config (set-*! family)"),
        (":wat::runtime::", Disp::Impure, "runtime introspection + mutation"),
        (":wat::program::", Disp::Impure, "reads process env"),
        (":wat::eval", Disp::Impure, "evaluates arbitrary submitted forms — purity is the form's, like `apply`"),
        // Unreviewed: each names the question, not a guess.
        (":wat::time::", Disp::Unreviewed, "MIXED — `now` reads the clock (non-deterministic), but `epoch-nanos` of an Instant in hand is a pure read. Needs per-verb review, and a blanket `impure` here would hide the pure readers the way the 35 were hidden"),
        (":wat::holon::", Disp::Unreviewed, "4 ruled pure 2026-08-01 (cosine/dot/coincident?/presence?). The rest split into threshold siblings (likely pure), LEARNING ops (update/add/put — a semantics question before a purity one), and the eval-* family (purity is the argument's)"),
        (":wat::rete::", Disp::Unreviewed, "engine verbs — insert/query/fire are pure value transforms over a Session, but a rete verb inside a rete predicate wants a ruling on recursion before a ruling on purity"),
        (":wat::stream::", Disp::Unreviewed, "laziness — a Stream's purity is its producer's"),
        (":wat::std::", Disp::Unreviewed, "arc 109 is annihilating this namespace; classify only what survives"),
        (":wat::verify::", Disp::Unreviewed, "signature verification — reads keys; needs review"),
        (":wat::form::", Disp::Unreviewed, "form/AST manipulation; kin to the `:wat::core::ast->*` family"),
        (":wat::stdlib::", Disp::Unreviewed, "single verb; unexamined"),
        (":wat::core::", Disp::Unreviewed, "the classified majority live here via `intrinsic_meta`; what falls through is the real worklist — the AST/meta family, apply, struct/Record ops, and the generic seq verbs named in `intrinsic_meta`'s doc"),
    ];

    /// ★ THE RATCHET. Lower it when verbs get ruled on; NEVER raise it to make a red gate green —
    /// raising it is the laundering this gate exists to prevent. A new dispatch verb that is
    /// neither classified nor disposed pushes the count over and goes red.
    const UNREVIEWED_BASELINE: usize = 212;

    /// Pull every verb the runtime dispatches, from BOTH doors: `dispatch_keyword_head_value` (the
    /// keyword-head path) and `dispatch_substrate_impl` (the `apply`-reachable substrate table).
    /// Located by NAME, not line number, so the scan cannot silently drift to the wrong region —
    /// and a floor-assert below catches it going vacuous if a rename ever breaks the anchors.
    fn dispatch_verbs(src: &str) -> Vec<String> {
        let lines: Vec<&str> = src.lines().collect();
        let mut out = Vec::new();
        for anchor in ["fn dispatch_keyword_head_value(", "fn dispatch_substrate_impl("] {
            let start = match lines.iter().position(|l| l.contains(anchor)) {
                Some(i) => i,
                None => continue,
            };
            let end = lines[start + 1..]
                .iter()
                .position(|l| l.starts_with("fn ") || l.starts_with("pub fn ") || l.starts_with("pub(crate) fn "))
                .map(|i| start + 1 + i)
                .unwrap_or(lines.len());
            for line in &lines[start..end] {
                let mut rest = *line;
                while let Some(i) = rest.find("\":wat::") {
                    rest = &rest[i + 1..];
                    if let Some(j) = rest.find('"') {
                        out.push(rest[..j].to_string());
                        rest = &rest[j + 1..];
                    } else {
                        break;
                    }
                }
            }
        }
        out.sort();
        out.dedup();
        out
    }

    #[test]
    fn every_dispatched_verb_is_classified_or_disposed() {
        let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/runtime.rs"))
            .expect("runtime.rs must be readable — it IS the source of truth for what verbs exist");
        let verbs = dispatch_verbs(&src);

        // Non-vacuity FIRST. If a rename broke the anchors this returns a handful of verbs, every
        // count below collapses, and a green gate would mean "we scanned nothing".
        assert!(
            verbs.len() > 400,
            "the dispatch scan found only {} verbs — the `fn dispatch_*` anchors have drifted and \
             this gate is measuring nothing. Fix the anchors; do NOT lower the floor.",
            verbs.len()
        );

        let (mut classified, mut impure, mut unreviewed) = (0usize, 0usize, Vec::new());
        for v in &verbs {
            if intrinsic_meta(v).is_some() {
                classified += 1;
                continue;
            }
            match RULES.iter().find(|(p, _, _)| v.starts_with(p)) {
                Some((_, Disp::Impure, _)) => impure += 1,
                Some((_, Disp::Unreviewed, _)) | None => unreviewed.push(v.clone()),
            }
        }

        println!(
            "\nPURITY COMPLETENESS — {} dispatched verbs\n\
             \x20 classified pure (intrinsic_meta)  {classified:>4}\n\
             \x20 disposed impure (namespace rule)  {impure:>4}\n\
             \x20 UNREVIEWED (the worklist)         {:>4}   baseline {UNREVIEWED_BASELINE}\n\
             \x20 note: constructors + field accessors are NOT here — `constructor_meta` and\n\
             \x20 `accessor_meta` DERIVE from the frozen TypeEnv, so they cannot go stale. Only the\n\
             \x20 hand-managed `intrinsic_meta` needs a gate.\n",
            verbs.len(),
            unreviewed.len(),
        );

        // The worklist, grouped by namespace — the INVENTORY is the deliverable, not the count
        // (`UNADOPTED.md`). A bare number tells nobody where to start.
        let mut by_ns: std::collections::BTreeMap<String, Vec<&String>> = Default::default();
        for v in &unreviewed {
            let ns = v.rsplit_once("::").map(|(a, _)| a.to_string()).unwrap_or_else(|| v.clone());
            by_ns.entry(ns).or_default().push(v);
        }
        let mut rows: Vec<(usize, String, Vec<&String>)> =
            by_ns.into_iter().map(|(k, v)| (v.len(), k, v)).collect();
        rows.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        println!("  THE WORKLIST — unreviewed by namespace (ratchet DOWN by ruling on these):");
        for (n, ns, vs) in rows.iter().take(12) {
            let sample: Vec<&str> = vs.iter().take(4).map(|s| s.as_str()).collect();
            println!("   {n:>4}  {ns}   e.g. {}", sample.join(", "));
        }
        println!();

        assert!(
            unreviewed.len() <= UNREVIEWED_BASELINE,
            "UNREVIEWED grew to {} (baseline {UNREVIEWED_BASELINE}).\n\n\
             A verb was added to the runtime's dispatch table without a purity classification or a \
             disposition. That is not cosmetic: `compile-condition` PANICS on `pure? = false`, so \
             every rule using this verb CANNOT COMPILE, and nothing else in the suite will say so. \
             This is the exact defect that hid 35 verbs — including the whole `String/` family and \
             the VSA seam — for months.\n\n\
             Fix it by CLASSIFYING the verb in `intrinsic_meta` (if it is genuinely pure — a ruling, \
             not an inference from its name) or by adding a disposition in `RULES` with the reason. \
             Do NOT raise the baseline.\n\n\
             Newly unreviewed, first 20:\n{}\n",
            unreviewed.len(),
            unreviewed.iter().take(20).map(|v| format!("  {v}")).collect::<Vec<_>>().join("\n"),
        );
    }
}
