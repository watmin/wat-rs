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
//!
//! ## The walk yields its violation instead of discarding it (post-6a fix — see
//! BRIEF-the-fence-names-the-head.md)
//!
//! The walk described above always knew, at the instant it falsified an axis, exactly which head did
//! it — and threw that away to return a bare `false`. `classify_expr`/`head_ok`/`classify_fn` now
//! return `Result<(), AxisViolation>` internally; `is_pure_expr`/`is_deterministic_expr` derive their
//! (UNCHANGED) bools from `.is_ok()`, and the new `find_axis_violation` (wat surface:
//! `:wat::rete::axis-violation`, PROVISIONAL name — cast owed) derives from `.err()`. One walk, two
//! surfaces. A `Span` rides along whenever the walk was still inside an inspectable call-site AST at
//! the moment of failure; the one case it is not is a `FunctionBody::Native` head reached through
//! transitive user-fn recursion (no body AST to point into — see `classify_fn`'s `Native` arm).
//!
//! ## A third axis, `Total` — UNARMED (BRIEF-total-t1-the-axis-unarmed.md)
//!
//! `Total` asks a DIFFERENT question than the two above: is the op defined on all its inputs, not
//! merely effect-free and referentially transparent? `first`/`i64::/`/`i64::mod` are all pure AND
//! deterministic — the fence above admits them — yet all three are **partial** (undefined on an
//! empty vector / a zero divisor), so a rule using one compiles clean and then aborts the entire
//! `fire-rules` call the first time a poisoned token reaches it. `is_total_expr`/`eval_total_predicate`
//! mirror the two siblings exactly (same walk, same `OpMeta` shape, same default-deny). **`total?` is
//! callable but `compile-condition` does NOT consult it** — arming the fence needs the `:undefined`-
//! carrying total variants (T2/T3) to exist first, or a refused `first` has nowhere to go. This stone
//! only mints the axis and measures which verbs a live corpus row actually demands be classified.

use crate::ast::WatAST;
use crate::runtime::{
    EvalBreak, Environment, FunctionBody, RuntimeError, RuntimeErrorKind, SymbolTable, Value,
    ValueSnapshot,
};
use crate::span::Span;
use crate::value::value::{AggregateValue, EnumValue};
use std::collections::HashSet;
use std::sync::Arc;

// ─── The two axes ─────────────────────────────────────────────────────────────

/// The property being classified. The structural walk is shared; only the per-head leaf decision
/// (`head_ok`) differs by axis. `pub(crate)` (not private) because `AxisViolation::axis` and
/// `find_axis_violation` — both `pub(crate)`, for the wat-visible `axis-violation` surface — carry
/// it past this module's boundary.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Axis {
    /// Effect-free: no IO/mutation/spawn.
    Pure,
    /// Referentially transparent: same inputs → same output.
    Deterministic,
    /// Defined on all its inputs (domain-total) — UNARMED this stone (see module doc's "A third
    /// axis, `Total`" section). `compile-condition` does not consult it yet.
    Total,
}

/// The offending leaf recorded when `classify_expr`/`head_ok`/`classify_fn` falsifies `axis`.
/// Carries at minimum the violating head's name; `span` is `Some` whenever the walk was still
/// inside an inspectable call-site AST at the moment of failure. `classify_expr`'s general-`List`
/// arm always has the failing call's own AST node (`items.first()`) in hand and stamps its `Span`
/// there — the ONE arm that cannot is `classify_fn`'s `FunctionBody::Native` case (and its sibling
/// "name not registered" case): a native fn stub has no body AST to point into.
///
/// Exists so `(:wat::rete::axis-violation …)` (the new wat-visible diagnostic surface — PROVISIONAL
/// name, cast owed) can name WHAT failed instead of a bare `false`. See
/// `docs/arc/2026/06/278-rules-engine/BRIEF-the-fence-names-the-head.md`.
#[derive(Clone)]
pub(crate) struct AxisViolation {
    pub(crate) head: String,
    pub(crate) axis: Axis,
    pub(crate) span: Option<Span>,
}

impl AxisViolation {
    /// A span-less violation — the caller (typically `classify_expr`'s general-`List` arm) fills in
    /// `span` from the call-site AST it has in hand, when it has one.
    fn new(head: impl Into<String>, axis: Axis) -> Self {
        AxisViolation { head: head.into(), axis, span: None }
    }
}

// ─── The hand-managed per-op metadata map (v1 projection of arc 255) ───────────

/// Declared properties of a known intrinsic. The single hand source of truth until arc 255 lifts it
/// to a queryable registry. DEFAULT-DENY: a head NOT covered here returns `None` ⇒ neither property.
/// `pub(crate)` (arc 278 #55 slice one): `rete::vocabulary::ReteOp` embeds this type directly
/// (its `meta` field) so the table's rows can declare their whitelist entry inline — "reuse
/// purity.rs's type" per the design stone's own sketch, rather than a second, parallel struct.
#[derive(Clone, Copy)]
pub(crate) struct OpMeta {
    pub(crate) pure: bool,
    pub(crate) deterministic: bool,
    /// Domain-total: defined on ALL its inputs? DEFAULT-DENY, same discipline as `pure`/
    /// `deterministic` — `false` unless a live corpus row demonstrated the need (see
    /// BRIEF-total-t1-the-axis-unarmed.md's measurement). NOT derived from `pure`/`deterministic`:
    /// `i64::/` is pure∧deterministic yet undefined at a zero divisor (and, separately, at the one
    /// input pair that overflows i64) — the three axes are genuinely orthogonal.
    pub(crate) total: bool,
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
    // Arc 278 #55 slice one — rete-namespaced heads consult THE ONE TABLE first (STOP-2: no rete
    // op named in more than one file — this is an exact match against `RETE_OPS`'s rows, never a
    // bare `:wat::rete::` prefix per STOP-1, so the engine's own API — `fire-rules`/`insert`/
    // `compile`/`Session`/`AlphaNode`/… — is untouched by this arm; it simply isn't a row).
    if let Some(op) = crate::rete::vocabulary::rete_op_for(head) {
        return Some(op.meta);
    }
    // Pure but NON-deterministic: random. Not corpus-demanded for `total` (Uuid/v4 can never
    // reach a `where` fence today — it already fails the determinism conjunct — so DEFAULT-DENY
    // stands; it is trivially total in the absolute sense but that claim was never measured).
    if head == ":wat::core::Uuid/v4" {
        return Some(OpMeta { pure: true, deterministic: false, total: false });
    }
    // Pure ∧ deterministic by namespace prefix — every op here is referentially transparent.
    // `total` is NOT blanket over the prefix (unlike pure/deterministic): `string::subs` is
    // GENUINELY PARTIAL — verified `string_ops.rs::eval_string_subs` raises `MalformedForm` when
    // `start`/`end` fall outside the string's char-length (out-of-range indices), the exact
    // domain-fault shape this axis exists to catch. The three below are the ones the where-corpus
    // (`where-string.wat`) demonstrates a need for, each verified total by reading its own
    // implementation (`string_ops.rs`): `length`/`trim`/`to-lowercase` always return, for any
    // string input, no raise. Every other `string::`/`regex::` verb (incl. `subs`, `split`,
    // `to-uppercase`, the whole `regex::` family) is left `false` — undemanded, unmeasured.
    if head.starts_with(":wat::core::string::") || head.starts_with(":wat::core::regex::") {
        let total = matches!(
            head,
            ":wat::core::string::length" | ":wat::core::string::trim" | ":wat::core::string::to-lowercase"
        );
        return Some(OpMeta { pure: true, deterministic: true, total });
    }
    // The whole `:wat::edn::` namespace is pure data transforms — parse/serialize/navigate
    // (read, read-foreign, write, write-pretty, write-json, write-json-natural,
    // ForeignRecord/get, ForeignRecord/class, ForeignVariant/variant, ForeignVariant/enum-class,
    // ForeignVariant/fields), no IO, no entropy. Root-level by namespace, not a per-verb
    // hand-list — the next foreign verb slips past a hand-list.
    // `total`: DEFAULT-DENY — no `where` in the corpus calls an edn verb, so nothing measured it
    // (`read`/`read-foreign` are the obvious partial candidates — malformed input — so a blanket
    // `true` here would be exactly the mass-assert this axis's doc forbids).
    if head.starts_with(":wat::edn::") {
        return Some(OpMeta { pure: true, deterministic: true, total: false });
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

    // ── `total` — BRIEF-total-t1-the-axis-unarmed.md's measurement, NOT a mass-assert ──────────
    //
    // DEFAULT-DENY over the WHOLE `pure_det` list above: every verb defaults to `total: false`
    // regardless of membership in `pure_det` (pure∧deterministic says nothing about totality —
    // `i64::/` is both, and undefined at a zero divisor). This sub-list is exactly the verbs the
    // 9-file / 98-row `where`-corpus (`wat-scripts/perf/grid/where-*.wat`) uses inside a `where`
    // (directly or via a transitively-checked user fn), each verified total by READING its own
    // implementation in `runtime.rs` (never inferred from the name):
    //
    //   generic `=`/`not=`/`<`/`and`/`or`/`not`/`if`/`let` — value/control-flow ops with no domain
    //     restriction (a well-typed call always returns; type mismatches are the type checker's
    //     concern, not this axis's, exactly the convention `pure`/`deterministic` already use).
    //   `i64::>` `i64::<` `i64::>=` `i64::<=` — comparisons never overflow (only +/-/*// do).
    //   `i64::to-f64` — a total, lossy-but-never-raising conversion (i64::MAX ≈ 9.2e18 is nowhere
    //     near f64's overflow boundary ≈1.8e308, so the result is always finite, never ±Inf).
    //   `f64::>` — a comparison, not an arithmetic op: `eval_f64_compare` returns a `bool` for any
    //     two f64 inputs including NaN/±Inf (IEEE says `NaN > x` is `false`, never a raise), so the
    //     OUTPUT itself can never be the undefined thing this axis polices — same shape as the
    //     `coincident?`/`presence?` predicates below.
    //   `PersistentVector/length` `/contains?` — always defined.
    //   `PersistentVector/get` — ALREADY total by design (returns `Option`, `None` on
    //     out-of-range — verified `persistentvector_get_inner`, never raises for a valid index).
    //   `String/concat` `/starts-with?` `/ends-with?` `/contains?` `/empty?` — verified
    //     (`string_ops.rs`) total for any two strings, no domain restriction.
    //   `foldl` — CONDITIONALLY total exactly like its pure∧det entry above: the verb ITSELF never
    //     raises (an empty seq returns the seed), so marking the head total and letting
    //     `classify_expr`'s general-list arm recurse into the fn-literal argument (checking ITS
    //     body against `Axis::Total` too) is the same mechanism already built for pure/det, not a
    //     new one. `map`/`filter`/`reduce`/`foldr` are extremely likely total by the identical
    //     argument but are NOT included — no `where` row in the corpus uses them, so nothing
    //     measured the claim. Flagged, not classified.
    //   `:wat::holon::presence?` — see the VSA-seam block below (BRIEF-total-column-honest.md
    //     Direction 2); grouped with the string/holon verbs there, not repeated here.
    //
    // Explicitly and deliberately LEFT `false` (genuinely partial, confirmed by reading the
    // implementation, not assumed from the design stone's guess) even though every one appears
    // inside a `where` in this corpus — these are exactly the T2 mint candidates:
    //   `i64::+` `i64::-` `i64::*` `i64::/` — verified `checked_add`/`checked_sub`/`checked_mul`/
    //     `checked_div` in `runtime.rs`: ALL FOUR raise `IntegerOverflow` at the i64 boundary
    //     (`i64::/` additionally raises `DivisionByZero`) — this generalizes past the design
    //     stone's own guess, which named only `/`/`mod`/`rem`/`quot` as partial; `+`/`-`/`*`
    //     overflow too, and the corpus's own `where-numeric.wat` header already measured this
    //     independently for `+` (documented as "the same class of event" as division-by-zero).
    //   `i64::mod` `i64::rem` `i64::quot` — verified: raise `DivisionByZero` at a zero divisor.
    //   `string::subs` — verified: raises `MalformedForm` on an out-of-range start/end.
    //
    // ── BRIEF-total-column-honest.md Direction 1 (2026-08-02) — a false marked TRUE, removed ────
    //
    //   `f64::*` — WAS `total: true` by T1's default-deny-then-never-revisited sweep; that was the
    //     gap this strike exists to close. `eval_f64_arith` dispatches `f64::*` to a bare `a * b`
    //     (`runtime.rs:4993`) — raw IEEE 754 multiply, NO overflow guard. Under the builder-ruled
    //     stricter definition (total = ordinary value on every input; NaN/±Inf are UNDEFINED, not a
    //     free pass), `f64::*` is NOT total two separate ways: (1) two large finite operands
    //     overflow to `±Inf` (e.g. `1e200 * 1e200`); (2) `0.0 * f64::INFINITY` (both are ordinary,
    //     reachable f64 values — Infinity is a legal f64 the moment ANY prior op produces one) is
    //     `NaN` by IEEE 754. `f64::>` (kept true, above) is unaffected — it is a comparison whose
    //     OUTPUT is a bool, never itself the undefined value. `f64::+`/`f64::-`/`f64::/` were never
    //     marked true (already correctly `false`, same overflow-to-Inf reasoning) — no action needed
    //     there, per STOP-3 (do not widen the audit past entries already `true`).
    let total = matches!(
        head,
        ":wat::core::="
            | ":wat::core::not="
            | ":wat::core::<"
            | ":wat::core::and"
            | ":wat::core::or"
            | ":wat::core::not"
            | ":wat::core::if"
            | ":wat::core::let"
            | ":wat::core::i64::>"
            | ":wat::core::i64::<"
            | ":wat::core::i64::>="
            | ":wat::core::i64::<="
            | ":wat::core::i64::to-f64"
            | ":wat::core::f64::>"
            | ":wat::core::PersistentVector/length"
            | ":wat::core::PersistentVector/contains?"
            | ":wat::core::PersistentVector/get"
            | ":wat::core::String/concat"
            | ":wat::core::String/starts-with?"
            | ":wat::core::String/ends-with?"
            | ":wat::core::String/contains?"
            | ":wat::core::String/empty?"
            | ":wat::core::foldl"
            // ── BRIEF-total-column-honest.md Direction 2 (2026-08-02) — the VSA seam ───────────
            //
            // Only ONE of the four holon verbs opened 2026-08-01 (`purity.rs`'s VSA-seam block,
            // above) is genuinely total. Per-verb evidence, not a blanket grant — the design
            // stone's `MEASUREMENT vs PREDICATE` ruling (`DESIGN-STONE-where-admits-only-rete-ops.md`
            // "RULED 2026-08-02 — THE MEASUREMENT IS FULL; THE PREDICATE IS EXACT") gives the SHAPE
            // of the eventual answer, but that ruling's `coincident?`/`presence? become total` is a
            // FUTURE-TENSE description of a strike this brief's STOP-1 forbids doing now (converting
            // `cosine` to an outcome enum, and correspondingly hardening `coincident?`/`presence?`'s
            // OWN bodies to swallow the degenerate case instead of raising) — so each of the four is
            // graded against what its implementation ACTUALLY does today, not the ruling's target
            // state:
            //
            //   `:wat::holon::presence?` — TRUE. `eval_algebra_presence_q` (`runtime.rs:18623`)
            //     takes both args through `require_holon` (HolonAST only — a raw `Vector` is
            //     rejected as a `TypeMismatch`, the ordinary "type checker's concern" exclusion this
            //     axis already uses elsewhere), then encodes BOTH at the same ambient `d`
            //     (`program_dim` → one `enc`, `runtime.rs:18646-18649`) — so there is no code path by
            //     which its two vectors can disagree in dimension; unlike its three siblings below it
            //     never reaches `pair_values_to_vectors`. Its only float op is `cosine >
            //     enc.presence_floor(sym)` — a comparison, total for the same reason `f64::>` is
            //     (returns `bool`, never raises, never itself NaN/Inf) — so even the `norm < 1e-10 →
            //     0.0` mask in `Similarity::cosine` (`holon-rs/src/kernel/similarity.rs:79-81`) does
            //     not threaten totality here: whatever `cosine` returns, `>` against it is defined.
            //     `presence?` is ALREADY total today, no future strike required.
            //
            //   `:wat::holon::coincident?` — left FALSE, diverging from the design stone's naive
            //     table entry (found by grounding, not by trusting the name — STOP-4). Unlike
            //     `presence?`, `eval_algebra_coincident_q` (`runtime.rs:18677`) is polymorphic over
            //     (HolonAST, Vector) pairs and routes BOTH args through `pair_values_to_vectors`
            //     (`runtime.rs:18699`, same call `cosine`/`dot` make) — which raises
            //     `RuntimeErrorKind::TypeMismatch` when two pre-encoded Vectors disagree in
            //     dimension (`runtime.rs:18539-18546`) — exactly `dot`'s already-grounded
            //     "dimension mismatch, not arithmetic" partiality, on the identical shared helper.
            //
            //     ⚠ REACHABILITY IS **UNPROVEN IN BOTH DIRECTIONS**, and this entry rests on
            //     DEFAULT-DENY, not on a demonstrated hazard. The obvious route was closed HOURS
            //     before this audit ran: `bytes-vector` used to admit a foreign-`d` Vector (its
            //     cross-dim check was VACUOUS — `encoders.get` materializes an encoder at any `d`
            //     it is handed), and `9eb0f4c1` replaced it with `dim != ctx.dim_count` →
            //     `VectorDecodeOutcome::DimensionMismatch` (`runtime.rs:19446`). With
            //     `set-dim-count!` a static, once-only entry-file constant (`config.rs:431`),
            //     every ENCODED Vector in a program shares one `d`. What is NOT proven is whether a
            //     `Value::Vector` can cross a process boundary via the GENERIC EDN record path
            //     rather than `bytes-vector` — nobody has enumerated that, so the world is not
            //     provably closed either. `total: false` is therefore the correct posture for an
            //     UNMEASURED verb, and must not be read as "mismatch is reachable."
            //     The design stone's "`coincident?`...become[s] total" is that FUTURE strike's
            //     target (hardening this verb's own body to swallow the mismatch as `false` instead
            //     of raising) — STOP-1 forbids doing that work now, so marking it `total: true`
            //     today would recreate this exact audit's own defect one level up: a false marked
            //     true, this time ahead of its implementation rather than behind it.
            //
            //   `:wat::holon::cosine`, `:wat::holon::dot` — left FALSE, per the brief's own grounded
            //     citations: `dot`'s arithmetic cannot overflow (`Vector.data: Vec<i8>`, `dot_raw`
            //     accumulates in `i64`, unreachable at real dimensions) and `cosine`'s zero-magnitude
            //     case cannot NaN (guarded to `0.0`) — but BOTH share `coincident?`'s reachable
            //     dimension-mismatch raise via `pair_values_to_vectors`, and `cosine`'s `0.0` is a
            //     live semantic mask on a reachable input (probe
            //     `wat-scripts/scratch-pad/probe-zero-magnitude-reachable.wat`, 2026-08-02: genuine
            //     unrelatedness reads `-0.0086`, never exactly `0.0`) that the ruled fix is an
            //     outcome enum for, not a `total: true` stamp. Unchanged from T1.
            | ":wat::holon::presence?"
    );

    if pure_det {
        Some(OpMeta { pure: true, deterministic: true, total })
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
    // `total`: DEFAULT-DENY, `false` at both sites below. Neither is corpus-demanded — no `where`
    // in the 98-row corpus calls a constructor — so it stays unmeasured rather than inferred, even
    // though "a constructor always builds a value, given well-typed args" is a plausible structural
    // argument (mirroring how `pure` is derived here from `nature.is_pure()` rather than a hand
    // list). Left `false` on discipline, not because a counter-example was found.
    // TypeEnv keys carry the leading colon (e.g. ":p::Rec") — use the head verbatim.
    // 1. Aggregate constructor (record / holon / struct) — the head IS the type name.
    if let Some(crate::types::TypeDef::Aggregate(a)) = types.get(head) {
        return Some(OpMeta { pure: a.nature.is_pure(), deterministic: true, total: false });
    }
    // 2. Enum-variant constructor — the head is `{EnumPath}::{Variant}` (unit or tagged).
    if let Some((enum_path, variant)) = head.rsplit_once("::") {
        if let Some(crate::types::TypeDef::Enum(e)) = types.get(enum_path) {
            let is_variant = e.variants.iter().any(|v| match v {
                crate::types::EnumVariant::Unit(n) => n == variant,
                crate::types::EnumVariant::Tagged { name, .. } => name == variant,
            });
            if is_variant {
                return Some(OpMeta { pure: e.purity.is_pure(), deterministic: true, total: false });
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
            // `total: true` — UNLIKE `constructor_meta`, this one IS corpus-demonstrated:
            // `where-record.wat` calls `Client/rep`, `L2/u`, `L3/w`, `L4/v`, `Client/l2`,
            // `L2/l3`, `L3/l4`, `Client/tags`, `Client/bag`, `Bag/items` directly inside `where`
            // predicates. And it is sound, not inferred from a name: a field declared on an
            // Aggregate exists on EVERY instance of that type by construction of the type itself
            // (there is no partial record — every field is populated at construction), so a
            // well-typed accessor call cannot fail on domain grounds, only on a type mismatch
            // (out of this axis's scope, same convention `pure`/`deterministic` already use).
            // Declaration-derived like `pure` above, not a hand-audited verb list — the exact
            // asymmetry the axis's default-deny doc warns about does not apply here.
            return Some(OpMeta { pure: a.nature.is_pure(), deterministic: true, total: true });
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
fn head_ok(head: &str, axis: Axis, sym: &SymbolTable, seen: &mut HashSet<String>) -> Result<(), AxisViolation> {
    // Data constructor (record/holon/enum-variant pure; struct impure) — recognized BEFORE the
    // sym.functions branch, because tagged-variant constructors are registered there as opaque stubs
    // that classify_fn would default-deny.
    if let Some(m) = constructor_meta(head, sym) {
        let ok = match axis {
            Axis::Pure => m.pure,
            Axis::Deterministic => m.deterministic,
            Axis::Total => m.total,
        };
        return if ok { Ok(()) } else { Err(AxisViolation::new(head, axis)) };
    }
    // Generated field accessor (`Type/field`) — same declaration-read as constructors, and likewise
    // BEFORE the sym.functions branch: accessors register there as Native stubs that classify_fn
    // default-denies, so we MUST intercept the accessor here.
    if let Some(m) = accessor_meta(head, sym) {
        let ok = match axis {
            Axis::Pure => m.pure,
            Axis::Deterministic => m.deterministic,
            Axis::Total => m.total,
        };
        return if ok { Ok(()) } else { Err(AxisViolation::new(head, axis)) };
    }
    // User-defined fn → transitive check of its body on the SAME axis.
    if sym.functions.contains_key(head) {
        return classify_fn(head, axis, sym, seen);
    }
    // Arc 278 #55 slice one — THE ADMISSION TEST, a FOURTH consideration alongside the three
    // above (additive, never a replacement — the design stone's own framing). A head inside a
    // declared rete-vocabulary sub-namespace (module-set membership — `rete_vocabulary_admitted`;
    // STOP-1: NOT a bare `:wat::rete::` prefix, which would wrongly admit the engine's own API)
    // is judged by THE ONE TABLE's declared meta directly. Admission is necessary, not
    // sufficient: an admitted namespace whose specific verb is not yet a `RETE_OPS` row still
    // default-denies here, rather than falling through to the generic effectful-namespace /
    // `intrinsic_meta` logic below (which does not know about rete-namespaced heads at all,
    // beyond the `intrinsic_meta` table consultation this arm makes redundant for admitted
    // heads specifically). Every non-rete-namespaced head (i.e. the entire 99-form `where`
    // corpus today) never reaches this branch — STOP-6's unmoved-corpus proof.
    if crate::rete::vocabulary::rete_vocabulary_admitted(head) {
        let ok = crate::rete::vocabulary::rete_op_for(head).is_some_and(|op| match axis {
            Axis::Pure => op.meta.pure,
            Axis::Deterministic => op.meta.deterministic,
            Axis::Total => op.meta.total,
        });
        return if ok { Ok(()) } else { Err(AxisViolation::new(head, axis)) };
    }
    match axis {
        // Pure: effectful namespaces are an explicit deny; otherwise the metadata must declare pure.
        Axis::Pure => {
            if crate::runtime::is_effectful_op(head) {
                return Err(AxisViolation::new(head, axis));
            }
            if intrinsic_meta(head).is_some_and(|m| m.pure) {
                Ok(())
            } else {
                Err(AxisViolation::new(head, axis))
            }
        }
        // Deterministic: the metadata must declare deterministic (effectful/unknown ⇒ None ⇒ deny,
        // which is correct — IO and unknown ops are not referentially transparent).
        Axis::Deterministic => {
            if intrinsic_meta(head).is_some_and(|m| m.deterministic) {
                Ok(())
            } else {
                Err(AxisViolation::new(head, axis))
            }
        }
        // Total (BRIEF-total-t1-the-axis-unarmed.md) — same default-deny discipline as
        // Deterministic: the metadata must declare total, unknown ⇒ None ⇒ deny.
        Axis::Total => {
            if intrinsic_meta(head).is_some_and(|m| m.total) {
                Ok(())
            } else {
                Err(AxisViolation::new(head, axis))
            }
        }
    }
}

// ─── Shared structural walk (parameterized by axis) ─────────────────────────────

/// Recursively classify an AST node against `axis`. The structure (quote-as-data, clause-aware
/// `cond`/`match`, element-wise vectors/maps/sets) is identical for both axes; only `head_ok` differs.
fn classify_expr(ast: &WatAST, axis: Axis, sym: &SymbolTable, seen: &mut HashSet<String>) -> Result<(), AxisViolation> {
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
        | WatAST::Symbol(_, _) => Ok(()),

        // quote / quasiquote / holon-literal sub-forms are DATA — do not recurse into them as calls.
        // Arc 294.b: `:wat::holon::literal` is pure (it captures data, no side-effects).
        WatAST::List(items, _) if matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::core::quote" || k == ":wat::core::quasiquote" || k == ":wat::holon::literal") => {
            Ok(())
        }

        // `cond` — clause-aware: (cond (test body…) …). A clause is NOT a call; every element
        // (test AND body forms) is an expression that must satisfy the axis. (cond ≡ chained `if`.)
        WatAST::List(items, _) if matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::core::cond") => {
            for clause in &items[1..] {
                match clause {
                    WatAST::List(parts, _) => {
                        for e in parts {
                            classify_expr(e, axis, sym, seen)?;
                        }
                    }
                    // malformed clause → deny, naming the malformed clause's own span.
                    other => return Err(AxisViolation { head: "<malformed cond clause>".into(), axis, span: Some(other.span().clone()) }),
                }
            }
            Ok(())
        }

        // `match` — clause-aware: (match scrut (pattern body…) …). The scrutinee and every arm
        // BODY must satisfy the axis; the PATTERN is structural (destructures/binds, never calls — wat
        // match has no guards). Arc 258.5 — bare match: scrutinee = items[1], arms = items[2..]
        // (the `-> :T` ascription is retired). Skip the pattern (arm element 0), check the body
        // (arm elements 1..).
        WatAST::List(items, list_span) if matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::core::match") => {
            let scrut = items.get(1).ok_or_else(|| AxisViolation {
                head: "<malformed match: no scrutinee>".into(),
                axis,
                span: Some(list_span.clone()),
            })?;
            classify_expr(scrut, axis, sym, seen)?;
            let arms = items.get(2..).ok_or_else(|| AxisViolation {
                head: "<malformed match: no arms>".into(),
                axis,
                span: Some(list_span.clone()),
            })?;
            for arm in arms {
                match arm {
                    // skip pattern (element 0); check body forms (1..).
                    WatAST::List(parts, _) => {
                        for e in parts.iter().skip(1) {
                            classify_expr(e, axis, sym, seen)?;
                        }
                    }
                    other => return Err(AxisViolation { head: "<malformed match arm>".into(), axis, span: Some(other.span().clone()) }),
                }
            }
            Ok(())
        }

        // `:wat::core::fn` lambda literal — NOT a call. Layout: (fn [params…] -> :ret body…).
        // The param vector + return-type are not evaluated; only the BODY forms (after the `-> :ret`
        // ascription) carry effects, so classify exactly those. Mirror the `match`-arm's logic:
        // locate the top-level `->` symbol, then body = items[i+2..] (skip `->` and :ret).
        WatAST::List(items, list_span) if matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::core::fn") => {
            match items.iter().position(|it| matches!(it, WatAST::Symbol(s, _) if s.as_str() == "->")) {
                Some(i) => {
                    let body = items.get(i + 2..).ok_or_else(|| AxisViolation {
                        head: "<malformed fn: no body>".into(),
                        axis,
                        span: Some(list_span.clone()),
                    })?;
                    for e in body {
                        classify_expr(e, axis, sym, seen)?;
                    }
                    Ok(())
                }
                // malformed fn (no `->`) → deny
                None => Err(AxisViolation { head: "<malformed fn: no `->`>".into(), axis, span: Some(list_span.clone()) }),
            }
        }

        // General list: head decision + recurse into args (same axis).
        WatAST::List(items, _) => {
            let head_node = items.first();
            let head = match head_node {
                None => return Ok(()), // empty list — no call
                Some(WatAST::Keyword(k, _)) => k.as_str(),
                Some(WatAST::Symbol(id, _)) => id.as_str(),
                // non-keyword/symbol head — unknown → deny, naming the offending node's own span.
                Some(other) => return Err(AxisViolation { head: "<non-keyword/symbol head>".into(), axis, span: Some(other.span().clone()) }),
            };
            if let Err(mut v) = head_ok(head, axis, sym, seen) {
                // Fill in the call-site span iff a deeper frame hasn't already stamped a more
                // precise one (see `AxisViolation`'s doc — the innermost failing call wins).
                if v.span.is_none() {
                    v.span = head_node.map(|h| h.span().clone());
                }
                return Err(v);
            }
            for a in &items[1..] {
                classify_expr(a, axis, sym, seen)?;
            }
            Ok(())
        }

        // Vectors / maps / sets → recurse element-wise.
        WatAST::Vector(elems, _) => {
            for e in elems {
                classify_expr(e, axis, sym, seen)?;
            }
            Ok(())
        }
        WatAST::Map(pairs, _) => {
            for (k, v) in pairs {
                classify_expr(k, axis, sym, seen)?;
                classify_expr(v, axis, sym, seen)?;
            }
            Ok(())
        }
        WatAST::Set(elems, _) => {
            for e in elems {
                classify_expr(e, axis, sym, seen)?;
            }
            Ok(())
        }
    }
}

/// Classify a named user fn against `axis` by inspecting its body transitively. `seen` detects cycles;
/// a back-edge (fqdn already in `seen`) returns `true` (fixpoint: the cycle adds no new violation).
fn classify_fn(fqdn: &str, axis: Axis, sym: &SymbolTable, seen: &mut HashSet<String>) -> Result<(), AxisViolation> {
    if seen.contains(fqdn) {
        return Ok(()); // back-edge — no new violation from the recursive call
    }
    seen.insert(fqdn.to_string());

    let func = match sym.functions.get(fqdn) {
        Some(f) => Arc::clone(f),
        None => return Err(AxisViolation::new(fqdn, axis)), // name not registered → deny
    };
    match &func.body {
        FunctionBody::Wat(body_ast) => classify_expr(body_ast.as_ref(), axis, sym, seen),
        // A native builtin registered in sym.functions is opaque — its body cannot be inspected —
        // so consult the hand-managed intrinsic_meta on the requested axis. This is load-bearing
        // for the HOF combinators (foldl/map/…): they are native AND registered in sym.functions,
        // so head_ok reaches classify_fn FIRST (before intrinsic_meta on the Pure/Det fallthrough).
        // Unproven natives (not in intrinsic_meta) still default-deny. NOTE: this is the one arm
        // that CANNOT carry a Span — a native stub has no body AST to point into (see
        // `AxisViolation`'s doc).
        FunctionBody::Native => {
            let ok = intrinsic_meta(fqdn).is_some_and(|m| match axis {
                Axis::Pure => m.pure,
                Axis::Deterministic => m.deterministic,
                Axis::Total => m.total,
            });
            if ok { Ok(()) } else { Err(AxisViolation::new(fqdn, axis)) }
        }
    }
}

/// Classify a NATIVE (bodiless) fn against `axis` by consulting `intrinsic_meta` on its OWN
/// path — the exact logic `classify_fn`'s `FunctionBody::Native` arm uses, extracted so a
/// caller holding a bare `Function` (not reached through `sym.functions` transitive recursion)
/// can apply the identical default-deny rule. Exposed for `freeze.rs`'s sigma-fn install-time
/// purity/determinism/totality gate (arc 278,
/// `docs/arc/2026/06/278-rules-engine/BRIEF-sigma-fn-must-be-pure-total-deterministic.md`,
/// STOP-1): a sigma fn's `Function` value is always `FunctionBody::Wat` today — nothing in
/// this codebase constructs a `Function` with `FunctionBody::Native` (see
/// `value::environment::FunctionBody`'s doc) — so this arm is presently unreachable for a
/// sigma fn in practice. Kept, not `unreachable!()`, because a purity GATE default-denying
/// is a controlled refusal; a gate that panics instead of refusing is worse than one that
/// never fires.
pub(crate) fn classify_native_fn(path: &str, axis: Axis) -> Result<(), AxisViolation> {
    let ok = intrinsic_meta(path).is_some_and(|m| match axis {
        Axis::Pure => m.pure,
        Axis::Deterministic => m.deterministic,
        Axis::Total => m.total,
    });
    if ok { Ok(()) } else { Err(AxisViolation::new(path, axis)) }
}

/// STOP-1's defensive code path has NO wat-surface fixture (see
/// `tests/program/wat_arc278_sigma_fn_purity_gate.rs`'s doc comment) — nothing in the crate
/// constructs a `Function` with `FunctionBody::Native`, so no wat program can drive a sigma fn
/// into this arm. Exercised directly at the Rust level instead, proving the extracted helper
/// itself agrees with `classify_fn`'s `FunctionBody::Native` arm it mirrors.
#[cfg(test)]
mod classify_native_fn_tests {
    use super::*;

    #[test]
    fn a_proven_pure_deterministic_native_head_passes_both_axes() {
        assert!(classify_native_fn(":wat::core::+", Axis::Pure).is_ok());
        assert!(classify_native_fn(":wat::core::+", Axis::Deterministic).is_ok());
    }

    #[test]
    fn an_unproven_head_default_denies_every_axis() {
        let head = ":wat::core::this-op-does-not-exist";
        assert!(classify_native_fn(head, Axis::Pure).is_err());
        assert!(classify_native_fn(head, Axis::Deterministic).is_err());
        assert!(classify_native_fn(head, Axis::Total).is_err());
    }

    #[test]
    fn pure_but_nondeterministic_native_head_fails_only_the_deterministic_axis() {
        // Uuid/v4 — the one hand-documented pure-but-random op.
        assert!(classify_native_fn(":wat::core::Uuid/v4", Axis::Pure).is_ok());
        assert!(classify_native_fn(":wat::core::Uuid/v4", Axis::Deterministic).is_err());
    }
}

// ─── Public axis classifiers (fresh `seen` per call) — also for stone 6b+ ──────

/// Is `ast` effect-free (no IO/mutation/spawn)? `:wat::core::Uuid/v4` is pure (it does no IO).
pub(crate) fn is_pure_expr(ast: &WatAST, sym: &SymbolTable) -> bool {
    classify_expr(ast, Axis::Pure, sym, &mut HashSet::new()).is_ok()
}

/// Is `ast` referentially transparent (same inputs → same output)? `:wat::core::Uuid/v4` is NOT.
pub(crate) fn is_deterministic_expr(ast: &WatAST, sym: &SymbolTable) -> bool {
    classify_expr(ast, Axis::Deterministic, sym, &mut HashSet::new()).is_ok()
}

/// Is `ast` domain-total (defined on all its inputs)? UNARMED (BRIEF-total-t1-the-axis-unarmed.md) —
/// callable, but `compile-condition` does not consult it. `:wat::core::i64::/` is NOT (undefined at
/// a zero divisor, and separately at the one input pair that overflows i64).
pub(crate) fn is_total_expr(ast: &WatAST, sym: &SymbolTable) -> bool {
    classify_expr(ast, Axis::Total, sym, &mut HashSet::new()).is_ok()
}

/// Run the SAME walk `is_pure_expr`/`is_deterministic_expr` use, but keep the violation instead of
/// collapsing it to `false`. `None` ⟺ `ast` satisfies `axis` (agrees with the bool predicates above
/// by construction — same function, same recursion, only the return type differs). Backs the
/// wat-visible `:wat::rete::axis-violation` diagnostic surface (PROVISIONAL name, cast owed).
pub(crate) fn find_axis_violation(ast: &WatAST, axis: Axis, sym: &SymbolTable) -> Option<AxisViolation> {
    classify_expr(ast, axis, sym, &mut HashSet::new()).err()
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

/// `(:wat::rete::total? <quoted-expr>) -> :bool` — domain-total (defined on all its inputs)?
/// UNARMED (BRIEF-total-t1-the-axis-unarmed.md): callable, but NOT consulted by
/// `compile-condition` this stone.
pub(crate) fn eval_total_predicate(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    eval_axis_predicate(":wat::rete::total?", is_total_expr, args, list_span, env, sym)
}

/// `(:wat::rete::axis-violation <quoted-expr> <axis: :wat::rete::Axis>) ->
/// :wat::core::Option<wat::rete::AxisViolation>`
///
/// **PLACEHOLDER NAME** — orchestrator's scaffolding, cast owed (see
/// `BRIEF-the-fence-names-the-head.md`). The SAME walk `pure?`/`deterministic?` run, surfacing the
/// violation instead of discarding it: `:wat::core::None` ⟺ `(pure? e)` / `(deterministic? e)` would
/// be `true` for the requested axis; `Some(v)` names the offending head (`v/head`), echoes the axis
/// back (`v/axis`), and carries a `:wat::kernel::Location` when the walk was still inside an
/// inspectable AST at the point of failure (`v/span`), `:wat::core::None` otherwise.
///
/// Builder-ruled (CLOSED-SET RULE, REALIZATIONS.md:2676): the axis argument is the
/// `:wat::rete::Axis` enum (a `defenum` in `wat/rete.wat`), decoded/encoded here directly as a
/// `Value::Enum` — no keyword string map. `pure?`/`deterministic?` are UNCHANGED by this addition
/// (STOP-1) — this is purely additive.
pub(crate) fn eval_axis_violation(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::axis-violation";
    const AXIS_TYPE: &str = ":wat::rete::Axis";
    if args.len() != 2 {
        return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
        })
        .into());
    }
    let val = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let ast = match val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError::new(args[0].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::WatAST (a quoted expr from :wat::core::quote)",
                got: Box::new(ValueSnapshot::of(&other)),
            })
            .into());
        }
    };
    let axis_val = crate::runtime::eval_inner(&args[1], env, sym)?.value_owned();
    let axis = match &axis_val {
        Value::Enum(ev) if ev.type_path == AXIS_TYPE && ev.variant_name == "Pure" => Axis::Pure,
        Value::Enum(ev) if ev.type_path == AXIS_TYPE && ev.variant_name == "Deterministic" => Axis::Deterministic,
        Value::Enum(ev) if ev.type_path == AXIS_TYPE && ev.variant_name == "Total" => Axis::Total,
        other => {
            return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::rete::Axis (Pure, Deterministic, or Total)",
                got: Box::new(ValueSnapshot::of(other)),
            })
            .into());
        }
    };
    let out = match find_axis_violation(&ast, axis, sym) {
        None => Value::Option(Arc::new(None)),
        Some(v) => {
            let span_val = match v.span {
                Some(sp) => Value::Option(Arc::new(Some(crate::runtime::value_from_span(sp)))),
                None => Value::Option(Arc::new(None)),
            };
            let axis_variant = match v.axis {
                Axis::Pure => "Pure",
                Axis::Deterministic => "Deterministic",
                Axis::Total => "Total",
            };
            let record = Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::rete::AxisViolation".to_string(),
                Arc::new(vec![
                    Value::String(Arc::new(v.head)),
                    Value::Enum(Arc::new(EnumValue {
                        type_path: AXIS_TYPE.to_string(),
                        variant_name: axis_variant.to_string(),
                        fields: vec![],
                    })),
                    span_val,
                ]),
            )));
            Value::Option(Arc::new(Some(record)))
        }
    };
    Ok(out)
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

    /// ★ THE LEDGER — the frozen NAMES of every dispatched verb that has no purity ruling yet.
    ///
    /// This replaced a bare count (`UNREVIEWED_BASELINE: usize = 214`) on 2026-08-02, at the
    /// builder's ruling: *"shouldn't this just be > 0, not some static ref? measuring an exact
    /// count is foolish."* He is right, and the hole was worse than imprecision — a count cannot
    /// tell these two worlds apart:
    ///
    ///     rule on one verb  +  add one unruled verb   =  the SAME number, gate stays GREEN
    ///
    /// A brand-new unruled verb walked in free whenever a strike also ruled on one, which is the
    /// normal case for a strike. The gate wanted SET MEMBERSHIP and measured CARDINALITY. It also
    /// could not name the offender — the old message said "Newly unreviewed, first 20" and then
    /// printed the first 20 of ALL of them, alphabetically, because it never knew which was new.
    ///
    /// `> 0` is the real goal and cannot be the gate today: these 215 are genuinely unruled, so a
    /// zero-tolerance check is permanently red. This list IS the debt, by name, and it is a
    /// RATCHET IN BOTH DIRECTIONS:
    ///
    /// - a verb NOT in this list  ⇒ RED. A new dispatch verb needs a ruling, always.
    /// - a verb in this list that is no longer unreviewed ⇒ RED. Rule on it, delete its line.
    ///
    /// So the list can only shrink, and it cannot silently rot. Never add a line to make a red
    /// gate green — that is the laundering this gate exists to prevent; CLASSIFY the verb in
    /// `intrinsic_meta`, or give its namespace a disposition in `RULES` with the reason.
    const KNOWN_UNREVIEWED: &[&str] = &[
    ":wat::core::Option/expect",
    ":wat::core::Option/try",
    ":wat::core::Record/assoc",
    ":wat::core::Record/field-at",
    ":wat::core::Record/same-data?",
    ":wat::core::Record::of",
    ":wat::core::Result/expect",
    ":wat::core::Result/try",
    ":wat::core::Tuple",
    ":wat::core::aggregate-new",
    ":wat::core::ann-form",
    ":wat::core::apply",
    ":wat::core::assoc",
    ":wat::core::char/of",
    ":wat::core::conforms?",
    ":wat::core::conj",
    ":wat::core::def",
    ":wat::core::defclause",
    ":wat::core::derive",
    ":wat::core::drop",
    ":wat::core::find-last-index",
    ":wat::core::fn",
    ":wat::core::forms",
    ":wat::core::keyword/from-string",
    ":wat::core::keyword/to-string",
    ":wat::core::kwargs-construct",
    ":wat::core::last",
    ":wat::core::macroexpand",
    ":wat::core::macroexpand-1",
    ":wat::core::match",
    ":wat::core::quasiquote",
    ":wat::core::quote",
    ":wat::core::range",
    ":wat::core::record->map",
    ":wat::core::rest",
    ":wat::core::reverse",
    ":wat::core::seqable->stream",
    ":wat::core::show",
    ":wat::core::sort'",  // rune:lint(retired-name) — live prime (arc 251 comparator-sort primitive); wat-level sort/sort-by wrap it
    ":wat::core::struct->form",
    ":wat::core::struct-field",
    ":wat::core::struct-new",
    ":wat::core::subtype?",
    ":wat::core::take",
    ":wat::core::to-record",
    ":wat::core::type",
    ":wat::core::use!",
    ":wat::core::variant",
    ":wat::form::matches?",
    ":wat::holon::Atom",
    ":wat::holon::Bind",
    ":wat::holon::Bind/left",
    ":wat::holon::Bind/right",
    ":wat::holon::Blend",
    ":wat::holon::Bundle",
    ":wat::holon::Bundle/children",
    ":wat::holon::Bundle/first",
    ":wat::holon::Engram/eigenvalue-signature",
    ":wat::holon::Engram/n",
    ":wat::holon::Engram/name",
    ":wat::holon::Engram/residual",
    ":wat::holon::EngramLibrary/add",
    ":wat::holon::EngramLibrary/contains",
    ":wat::holon::EngramLibrary/len",
    ":wat::holon::EngramLibrary/match-vec",
    ":wat::holon::EngramLibrary/names",
    ":wat::holon::EngramLibrary/new",
    ":wat::holon::Hologram/capacity",
    ":wat::holon::Hologram/find",
    ":wat::holon::Hologram/get",
    ":wat::holon::Hologram/len",
    ":wat::holon::Hologram/make",
    ":wat::holon::Hologram/put",
    ":wat::holon::Hologram/remove",
    ":wat::holon::List",
    ":wat::holon::Map",
    ":wat::holon::OnlineSubspace/dim",
    ":wat::holon::OnlineSubspace/eigenvalues",
    ":wat::holon::OnlineSubspace/k",
    ":wat::holon::OnlineSubspace/n",
    ":wat::holon::OnlineSubspace/new",
    ":wat::holon::OnlineSubspace/project",
    ":wat::holon::OnlineSubspace/reconstruct",
    ":wat::holon::OnlineSubspace/residual",
    ":wat::holon::OnlineSubspace/threshold",
    ":wat::holon::OnlineSubspace/update",
    ":wat::holon::Permute",
    ":wat::holon::Reckoner/curve",
    ":wat::holon::Reckoner/dims",
    ":wat::holon::Reckoner/labels",
    ":wat::holon::Reckoner/new-continuous",
    ":wat::holon::Reckoner/new-discrete",
    ":wat::holon::Reckoner/observe",
    ":wat::holon::Reckoner/predict",
    ":wat::holon::Reckoner/resolve",
    ":wat::holon::Record::of",
    ":wat::holon::Set",
    ":wat::holon::Thermometer",
    ":wat::holon::Tuple",
    ":wat::holon::Vector",
    ":wat::holon::bytes-vector",
    ":wat::holon::coincident-explain",
    ":wat::holon::coincident-floor",
    ":wat::holon::encode",
    ":wat::holon::eval-coincident?",
    ":wat::holon::eval-digest-coincident?",
    ":wat::holon::eval-digest-string-coincident?",
    ":wat::holon::eval-edn-coincident?",
    ":wat::holon::eval-signed-coincident?",
    ":wat::holon::eval-signed-string-coincident?",
    ":wat::holon::extract-classifier",
    ":wat::holon::from-holon",
    ":wat::holon::from-wat",
    ":wat::holon::is-Keyword?",
    ":wat::holon::is-List?",
    ":wat::holon::is-Map?",
    ":wat::holon::is-Nil?",
    ":wat::holon::is-Set?",
    ":wat::holon::is-Symbol?",
    ":wat::holon::is-Tag?",
    ":wat::holon::is-Tuple?",
    ":wat::holon::is-Vector?",
    ":wat::holon::is?",
    ":wat::holon::leaf",
    ":wat::holon::literal",
    ":wat::holon::presence-floor",
    ":wat::holon::simhash",
    ":wat::holon::statement-length",
    ":wat::holon::term::matches?",
    ":wat::holon::term::ranges",
    ":wat::holon::term::slots",
    ":wat::holon::term::template",
    ":wat::holon::therm-form",
    ":wat::holon::to-holon",
    ":wat::holon::to-record",
    ":wat::holon::to-wat",
    ":wat::holon::vector-bind",
    ":wat::holon::vector-blend",
    ":wat::holon::vector-bundle",
    ":wat::holon::vector-bytes",
    ":wat::holon::vector-permute",
    ":wat::rete::alpha-match",
    ":wat::rete::axis-violation",
    ":wat::rete::collect-rules",
    ":wat::rete::deterministic?",
    ":wat::rete::eval-insert",
    ":wat::rete::eval-test",
    ":wat::rete::fire-once'",  // rune:lint(retired-name) — rete dual-impl: unprimed is the wat ORACLE, primed the native kernel; never collapsed
    ":wat::rete::fire-rules'",  // rune:lint(retired-name) — rete dual-impl: unprimed is the wat ORACLE, primed the native kernel; never collapsed
    ":wat::rete::fire-rules-explain'",  // rune:lint(retired-name) — rete dual-impl: unprimed is the wat ORACLE, primed the native kernel; never collapsed
    ":wat::rete::insert'",  // rune:lint(retired-name) — rete dual-impl: unprimed is the wat ORACLE, primed the native kernel; never collapsed
    ":wat::rete::insert-all'",  // rune:lint(retired-name) — rete dual-impl: unprimed is the wat ORACLE, primed the native kernel; never collapsed
    ":wat::rete::pure?",
    ":wat::rete::step-payload'",  // rune:lint(retired-name) — rete dual-impl: unprimed is the wat ORACLE, primed the native kernel; never collapsed
    ":wat::rete::total?",
    ":wat::rete::vocabulary-admitted?",
    ":wat::std::list::map-with-index",
    ":wat::std::list::remove-at",
    ":wat::std::list::window",
    ":wat::std::list::zip",
    ":wat::std::math::cos",
    ":wat::std::math::exp",
    ":wat::std::math::ln",
    ":wat::std::math::log",
    ":wat::std::math::pi",
    ":wat::std::math::sin",
    ":wat::std::math::sqrt",
    ":wat::std::stat::mean",
    ":wat::std::stat::stddev",
    ":wat::std::stat::variance",
    ":wat::stdlib::sources",
    ":wat::stream::cons",
    ":wat::stream::empty",
    ":wat::stream::lazy",
    ":wat::time::+",
    ":wat::time::-",
    ":wat::time::Day",
    ":wat::time::Hour",
    ":wat::time::Microsecond",
    ":wat::time::Millisecond",
    ":wat::time::Minute",
    ":wat::time::Nanosecond",
    ":wat::time::Second",
    ":wat::time::ago",
    ":wat::time::at",
    ":wat::time::at-millis",
    ":wat::time::at-nanos",
    ":wat::time::days",
    ":wat::time::days-ago",
    ":wat::time::days-from-now",
    ":wat::time::epoch-millis",
    ":wat::time::epoch-nanos",
    ":wat::time::epoch-seconds",
    ":wat::time::from-iso8601",
    ":wat::time::from-now",
    ":wat::time::hours",
    ":wat::time::hours-ago",
    ":wat::time::hours-from-now",
    ":wat::time::microseconds",
    ":wat::time::microseconds-ago",
    ":wat::time::microseconds-from-now",
    ":wat::time::milliseconds",
    ":wat::time::milliseconds-ago",
    ":wat::time::milliseconds-from-now",
    ":wat::time::minutes",
    ":wat::time::minutes-ago",
    ":wat::time::minutes-from-now",
    ":wat::time::nanoseconds",
    ":wat::time::nanoseconds-ago",
    ":wat::time::nanoseconds-from-now",
    ":wat::time::now",
    ":wat::time::seconds",
    ":wat::time::seconds-ago",
    ":wat::time::seconds-from-now",
    ":wat::time::to-iso8601",
    ];

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
             \x20 UNREVIEWED (the worklist)         {:>4}   ledger {}\n\
             \x20 note: constructors + field accessors are NOT here — `constructor_meta` and\n\
             \x20 `accessor_meta` DERIVE from the frozen TypeEnv, so they cannot go stale. Only the\n\
             \x20 hand-managed `intrinsic_meta` needs a gate.\n",
            verbs.len(),
            unreviewed.len(),
            KNOWN_UNREVIEWED.len(),
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

        let known: std::collections::BTreeSet<&str> = KNOWN_UNREVIEWED.iter().copied().collect();
        let live: std::collections::BTreeSet<&str> = unreviewed.iter().map(|v| v.as_str()).collect();

        let newly: Vec<&&str> = live.difference(&known).collect();
        assert!(
            newly.is_empty(),
            "{} dispatch verb(s) have NO purity ruling and are NOT in the ledger:\n{}\n\n\
             That is not cosmetic: `compile-condition` PANICS on `pure? = false`, so every rule \
             using one of these CANNOT COMPILE, and nothing else in the suite will say so. This is \
             the exact defect that hid 35 verbs — including the whole `String/` family and the VSA \
             seam — for months.\n\n\
             Fix it by CLASSIFYING the verb in `intrinsic_meta` (if it is genuinely pure — a \
             ruling, not an inference from its name), or by giving its namespace a disposition in \
             `RULES` with the reason. Adding it to `KNOWN_UNREVIEWED` is the LAST resort and is \
             only honest for a verb whose ruling is genuinely open — say why in the commit.\n",
            newly.len(),
            newly.iter().map(|v| format!("  {v}")).collect::<Vec<_>>().join("\n"),
        );

        let stale: Vec<&&str> = known.difference(&live).collect();
        assert!(
            stale.is_empty(),
            "{} verb(s) in `KNOWN_UNREVIEWED` are no longer unreviewed — they have been ruled on \
             (or no longer dispatched). DELETE their lines; the ledger must shrink as the debt is \
             paid, or it rots into a list nobody trusts:\n{}\n",
            stale.len(),
            stale.iter().map(|v| format!("  {v}")).collect::<Vec<_>>().join("\n"),
        );
    }
}
