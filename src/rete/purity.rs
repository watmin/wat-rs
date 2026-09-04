//! Arc 278 Stone 6a — the rete condition fence: four orthogonal classifiers,
//! `pure?` + `deterministic?` + `total?` + `primitive?`.
//!
//! A rete condition (a `where`/`:test` predicate, an accumulator fn) must be a **deterministic,
//! effect-free function of the facts**. Those are two INDEPENDENT properties:
//!
//! - **pure** — effect-free: no IO/mutation/spawn (seed: the negation of `is_effectful_op`).
//! - **deterministic** — referentially transparent: same inputs → same output (no randomness/clock).
//!
//! They are genuinely orthogonal. `:wat::uuid::v4` does no IO and mutates nothing → it is PURE,
//! yet it is random → NON-deterministic. The exposed rete check is therefore
//! `(and (pure? f) (deterministic? f) (total? f) (primitive? f))`; each axis is
//! its own predicate.
//!
//! ## Default-deny, and the hand-managed metadata map
//!
//! The four classifiers are DEFAULT-DENY: a head's property holds only if PROVEN (a known intrinsic whose
//! metadata declares it, or a user fn whose body transitively holds it); anything unproven is rejected.
//! The per-op metadata is a small HAND-MANAGED map (`intrinsic_meta`) — the explicit v1 projection of
//! the queryable registry (arc 255; see
//! `docs/arc/2026/06/255-builtin-registry/NOTE-purity-is-definition-time-queryable-metadata.md`).
//! This file is the v1 hand map. rune:exigere(attested-arc) — registry is arc 255.
//!
//! ## Entry points
//!
//! `(:wat::rete::pure? <quoted-expr>) -> :bool` · `(:wat::rete::deterministic? <quoted-expr>) -> :bool`
//! · `(:wat::rete::total? <quoted-expr>) -> :bool` · `(:wat::rete::primitive? <quoted-expr>) -> :bool`
//! Dispatched from `runtime.rs` beside the sibling rete primitives. Fact-pattern Law A is the
//! freeze wall plus intern `compile_condition_local` (CoreGeneric → none).
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
//! `:wat::rete::axis-violation`) derives from `.err()`. One walk, two
//! surfaces. A `Span` rides along whenever the walk was still inside an inspectable call-site AST at
//! the moment of failure; the one case it is not is a `FunctionBody::Native` head reached through
//! transitive user-fn recursion (no body AST to point into — see `classify_fn`'s `Native` arm).
//!
//! ## A third axis, `Total` — ARMED (BRIEF-total-t1-the-axis-unarmed.md minted the axis)
//!
//! `Total` asks a DIFFERENT question than the two above: is the op defined on all its inputs, not
//! merely effect-free and referentially transparent? `first`/`i64::/`/`i64::mod` are all pure AND
//! deterministic — yet all three are **partial** (undefined on an empty vector / a zero divisor),
//! so a rule using a core-spelled one would compile clean and then abort `fire-rules` the first
//! time a poisoned token reaches it. `is_total_expr` mirrors its two siblings
//! (`is_pure_expr`/`is_deterministic_expr`) exactly — same walk, same `OpMeta` shape, same
//! default-deny; its wat entry point is `#[wat_intrinsic]`-homed in `src/intrinsic/rete.rs`
//! (arc 255 Stone P6-c-W5a), not a hand-rolled dispatch fn in this file any more.
//! **`compile-condition`
//! consults `total?`** as the third conjunct of the four-axis fence (pure ∧ det ∧ total ∧ rete).
//! Partial core ops enter rete only as `OpClass::Fallback` + a mandatory `:undefined`.
//!
//! ## `effectful_by_prefix` / `is_effectful_op` — arc 109 Stone the-last-two-map-items
//!
//! Moved verbatim out of `src/runtime.rs`'s megafile
//! (`docs/arc/2026/04/109-kill-std/DESIGN-STONE-the-last-two-map-items.md`), item 7 of the map:
//! a two-tier classifier — `is_effectful_op` consults `crate::intrinsic::registry()` first and
//! falls back to `effectful_by_prefix` — that moves as a pair (splitting them would put one tier
//! of a two-tier classifier in each of two homes). This file already calls
//! `crate::intrinsic::registry()` (the `intrinsic_meta`/`constructor_meta` machinery above), so
//! the move adds no new dependency; `Axis::Pure`'s leaf decision, just below, is the pair's own
//! first caller in this file.

use wat_macros::wat_intrinsic;

use crate::ast::WatAST;
use crate::runtime::{
    EvalBreak, Environment, Function, FunctionBody, RuntimeError, RuntimeErrorKind, SymbolTable,
    Value, ValueSnapshot,
};
use crate::span::Span;
use crate::value::value::{AggregateValue, EnumValue};
use std::collections::HashSet;
use std::sync::Arc;

// ─── The four-axis fence ──────────────────────────────────────────────────────

/// The property being classified. The structural walk is shared; only the per-head leaf decision
/// (`head_ok`) differs by axis. `pub(crate)` (not private) because `AxisViolation::axis` and
/// `find_axis_violation` — both `pub(crate)`, for the wat-visible `axis-violation` surface — carry
/// it past this module's boundary.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Axis {
    /// Effect-free: no IO/mutation/spawn.
    Pure,
    /// Referentially transparent: same inputs → same output.
    Deterministic,
    /// Defined on all its inputs (domain-total). `compile-condition` consults it as the
    /// third conjunct of the four-axis fence.
    Total,
    /// #57 LAW A — the head is a rete primitive. The builder's law, stated: *"the entire rete
    /// query language may only be composed from rete primitives."*
    ///
    /// It needs its own variant because reusing `Pure`/`Deterministic`/`Total` would make the
    /// refusal LIE: `:wat::core::>` IS pure, IS deterministic, and IS total — it is refused for
    /// one reason only, that it is not from rete. Rejecting it with "is not pure" sends every
    /// reader hunting a purity defect that does not exist.
    /// `[[feedback_a_gates_name_is_where_the_lie_lives]]`
    ///
    /// ★ THE NAME IS A WORD IN A SENTENCE, not a label. `axis-violation-message`
    /// (`wat/rete/compile.wat`) builds the user-facing refusal by literal `string::concat` per
    /// arm — "is not pure" / "is not deterministic" / "is not total" — so this variant's fourth
    /// arm reads **"'<head>' is not a rete primitive"**, which is the law itself, and tells the
    /// author what to do without a lookup. (An earlier spelling, `Vocabulary`, was cast to
    /// intueri and failed exactly here: "is not vocabulary" does not parse as English and names
    /// the table we check rather than the law we hold. It also invented a category mismatch —
    /// "a property of the fence, not the op" — that this name dissolves: *being a rete primitive
    /// IS a property of the head*, the same kind as its three siblings.)
    ///
    /// A head reaches this only after the three declaration-derived doors have all declined it:
    /// not a constructor, not an accessor, not a user fn (the composition door recurses), and not
    /// an admitted rete-vocabulary member. What is left is a core-spelled op inside a `where` —
    /// the one thing law A exists to refuse.
    RetePrimitive,
}

impl Axis {
    /// The wat-side `:wat::rete::Axis` variant name for this axis.
    ///
    /// ★ THE ONE DOOR for the Rust↔wat name mapping, in BOTH directions —
    /// `from_variant_name` is defined in terms of this, so the two can no longer disagree.
    ///
    /// ⛔ WHY THIS EXISTS, and it cost 39 tests to learn. The mapping used to live at two
    /// independent sites in `eval_axis_violation`: an ENCODE (`match` on `Axis`) and a DECODE
    /// (`match` on `ev.variant_name`, a `&str`). Adding `RetePrimitive` to the enum made the
    /// compiler enumerate seven consumers and rewrite the encode — and it said **nothing** about
    /// the decode, because *no exhaustiveness check can see a match on a string*. So the fourth
    /// axis was encodable and undecodable: `axis-violation-message`'s new arm called
    /// `(:wat::rete::axis-violation expr :RetePrimitive)` and the native side answered
    /// `TypeMismatch: expected :wat::rete::Axis (Pure, Deterministic, or Total)`.
    ///
    /// This is the class `holon/CLAUDE.md` names as recurring — *suspect a string comparison
    /// before you suspect the type system* — and the mirror rule
    /// `[[feedback_the_mirror_is_an_instrument_not_a_fix]]`: enumerate ONE side of a pair and
    /// demand a twin for each state. The encode had four states; the decode had three.
    pub(crate) fn variant_name(self) -> &'static str {
        match self {
            Axis::Pure => "Pure",
            Axis::Deterministic => "Deterministic",
            Axis::Total => "Total",
            Axis::RetePrimitive => "RetePrimitive",
        }
    }

    /// Every `Axis`. Kept honest by `axis_variant_names_round_trip`, whose `match` is over `Axis`
    /// itself — so a new variant makes that test non-exhaustive and the compiler names it there,
    /// three lines from this list.
    pub(crate) const ALL: [Axis; 4] =
        [Axis::Pure, Axis::Deterministic, Axis::Total, Axis::RetePrimitive];

    /// Decode a wat-side variant name. Derived from `variant_name`, never a second spelling.
    pub(crate) fn from_variant_name(name: &str) -> Option<Axis> {
        Axis::ALL.into_iter().find(|a| a.variant_name() == name)
    }

    /// The `expected:` text a `TypeMismatch` shows when a decode fails — DERIVED, so it can never
    /// again advertise a smaller set than the decode actually accepts. The old text was the literal
    /// `":wat::rete::Axis (Pure, Deterministic, or Total)"`, which stayed accurate right up until
    /// it wasn't, and then lied in the one message a reader would trust.
    ///
    /// Built once into a `OnceLock` rather than `String::leak`-ed per call: the set is fixed at
    /// compile time, so leaking a fresh allocation on every rejected decode would be a real leak on
    /// a path a program can hit in a loop.
    pub(crate) fn expected_list() -> &'static str {
        static EXPECTED: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        EXPECTED.get_or_init(|| {
            let names: Vec<&str> = Axis::ALL.iter().map(|a| a.variant_name()).collect();
            format!(":wat::rete::Axis (one of: {})", names.join(", "))
        })
    }
}

/// The offending leaf recorded when `classify_expr`/`head_ok`/`classify_fn` falsifies `axis`.
/// Pattern A: `span` is required. Call-site AST span when the walk is inside an inspectable
/// form; `classify_native_fn` / unregistered names use `rust_caller_span` (no body AST).
///
/// Exists so `(:wat::rete::axis-violation …)` can name WHAT failed instead of a bare `false`. See
/// `docs/arc/2026/06/278-rules-engine/BRIEF-the-fence-names-the-head.md`.
#[derive(Clone)]
pub(crate) struct AxisViolation {
    pub(crate) span: Span,
    pub(crate) head: String,
    pub(crate) axis: Axis,
}

impl AxisViolation {
    fn at(span: Span, head: impl Into<String>, axis: Axis) -> Self {
        AxisViolation { span, head: head.into(), axis }
    }
}

// ─── The hand-managed per-op metadata map (v1 projection of arc 255) ───────────

/// Declared properties of a known intrinsic. The v1 hand source of truth
/// (arc 255 is the registry). DEFAULT-DENY: a head NOT covered here returns `None` ⇒ neither property.
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
/// (`compile-condition` panics on `pure? = false`, `wat/rete/compile.wat`). Nothing detects that; only
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
/// `ast->*`, `with-children`, `write-forms`); `Option/expect` / `Result/expect` (⛔ RULED
/// `Partial` 2026-08-30 — this line used to read "total but they raise"; a raise is NOT a
/// matchable outcome, so "but they raise" was the whole answer, not a parenthetical. See
/// `docs/arc/2026/06/255-builtin-registry/RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`); and the generic sequence verbs (`range`, `take`, `drop`, `rest`, `last`, `assoc`,
/// `conj`, `find-last-index`) which are very likely pure but were not measured this session.
///
/// **The other 101 `:wat::holon::` verbs are also deliberately unclassified.** Three groups, and
/// the middle one is the interesting question, not an oversight:
/// - *the threshold siblings* — `coincident-floor`, `presence-floor`, `coincident-explain`: the
///   same reads with an explicit floor / an explanation payload. Unclassified (present
///   default-deny). If a caller surfaces a per-rule floor, a new arc opens; this comment
///   does not commit to one.
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
    // Arc 255 Stone the-registry-answers-first-wave-2 — RETIRED, a pure duplicate: `:wat::uuid::v4`'s
    // own registration (`src/intrinsic/uuid.rs:57`) already declares `@Purity Pure` /
    // `@Determinism Nondeterministic` / `@Totality Unreviewed`, byte-identical to this guard's
    // `{pure: true, deterministic: false, total: false}`. Deleted rather than moved — there was no
    // fact to move, only a copy to remove. The registry consult below now answers for it directly.
    // Keyed-collection ITERATION is pure and total but NOT deterministic — measured
    // 2026-08-26, three consecutive processes, three different orders, for BOTH containers:
    //
    //   HashMap/keys        [:c :a :b :d :e] · [:c :d :a :e :b] · [:e :b :c :d :a]
    //   PersistentMap/keys  [:i :d :a :f :j …] · [:f :e :g :a :j …] · [:e :h :d :a :g …]
    //
    // Both were previously in `pure_det`, i.e. classified DETERMINISTIC. That was a lie in a
    // first-class axis: the fence admits a head into a `where` on the strength of this claim.
    // `src/value/pmap.rs` says it plainly for the persistent side — *"iteration order is
    // deliberately NOT part of the contract (the trie has no meaningful order, so promising one
    // would be a lie the array arm could keep and the trie arm could not)"* — and the same is
    // true of `Arc<std::HashMap>`, whose default hasher is seeded per process.
    //
    // ⚠ NON-VACUITY, stated because the floor did not move when this landed: ZERO `.wat` rules
    // use these verbs inside a `where`/`then`/accumulator today (39 call sites, none in a fence),
    // so nothing could break. This is prophylaxis — it closes the lie BEFORE the persistent-backend
    // swap makes it load-bearing, at which point changing which container a literal produces would
    // change observed key order while this axis said that was impossible.
    // Scrutiny of the wider class: `docs/arc/2026/06/255-builtin-registry/NOTE-the-registry-asserts-properties-nothing-verifies.md`
    // Arc 255 Stone E-i — the maps get their homes. The old `:wat::core::{HashMap,
    // PersistentMap}/{keys,values}` spelling retired this stone; `:wat::hashmap::{keys,values}`
    // and `:wat::map::{keys,values}` carry the identical non-deterministic classification
    // (name-only rename; the iteration-order argument is unchanged).
    // Arc 255 Stone the-registry-answers-first-wave-2 — RETIRED. The fact this guard carried
    // (`total: true`, alongside the `pure`/`deterministic` above it) now lives at each of the
    // four verbs' own registration (`src/intrinsic/hashmap.rs:206,233`, `src/intrinsic/map.rs:
    // 172,190`) as `@Totality Total` — re-derived from `hashmap_keys_inner`/`hashmap_values_inner`/
    // `persistentmap_keys_inner`/`persistentmap_values_inner` (`src/collection/eval.rs`): each
    // verb's `other =>` `TypeMismatch` arm is checker-impossible for a well-typed container
    // argument. Confirmed unchanged. The registry consult below now answers for all four directly.
    // arc 255 Stone the-registry-answers-first — the `:wat::string::`/`:wat::regex::` prefix
    // guess and the `:wat::edn::` prefix guess (each a PREFIX GUESS outranking the registry,
    // DESIGN-STONE-the-registry-answers-first.md) are RETIRED. Both shadowed 34 registered
    // verbs that already declared `@Purity Pure`/`@Determinism Deterministic` at their own
    // registration; the eleven the guesses additionally claimed `total: true` for
    // (`:wat::string::{length,trim,to-lowercase,contains?,starts-with?,ends-with?,empty?}`,
    // `:wat::edn::{read-foreign,ForeignRecord/get,ForeignRecord/class}`) now carry that fact as
    // `@Totality` at their own registration instead — re-derived from each body, not
    // transcribed, and `:wat::string::concat` came back `Partial` (not the guess's `Total`: its
    // own body raises `ArityMismatch` on a zero-arg call, which `check.rs::infer_string_concat`
    // confirms the checker accepts as well-typed). The registry consult below now answers for
    // all 34 directly.
    // Arc 255 Stone the-registry-answers-first-wave-3 — RETIRED. The fact this guard carried
    // (`pure: true, deterministic: true, total: true` for both verbs — construction is
    // assignment: it evaluates already-supplied field VALUES and binds them into a new
    // container, opening nothing, reading no ambient state, mutating nothing) now lives at each
    // verb's own registration (`src/intrinsic/record.rs`) as `@Purity Pure` /
    // `@Determinism Deterministic` / `@Totality Total` — re-derived from `construct_aggregate`
    // (`src/runtime.rs`) unchanged. Both named gaps this classification originally found are
    // still CLOSED, confirmed unchanged: the `aggregate-new`-only checker hole
    // (`infer_aggregate_new_check`, check.rs) and the `HolonRecord` freeze-time capacity hole
    // (`freeze::validate_holon_record_capacity`). `total?` stays ARMED at both
    // `compile-condition` and `then-item-fence`. The registry consult below now answers for both
    // directly.
    // Arc 255 Stone the-registry-answers-first-wave-2 — RETIRED, and the re-derivation OVERTURNED
    // this guard's `total: true` for both verbs. `type-params-used-in`'s `@Totality` now lives at
    // its own registration (`src/intrinsic/reflect.rs:640`) as `Partial`: `param_name_of` raises
    // `TypeMismatch` on a well-typed but non-Symbol/Keyword `params` element, and this verb is
    // additionally on `intrinsic/mod.rs`'s `FROZEN_CHECKER_DEBT_LEDGER` (no TypeScheme at all), so
    // nothing stops that shape from reaching a well-typed call. `type-equal?`'s `@Totality` now
    // lives at `src/intrinsic/reflect.rs:745` as `Partial` too — its own doc already said outright
    // "given a node that does not parse as a type at all, this RAISES rather than returning
    // `false`," which `RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md` makes
    // `Partial`, not `Total`. Both confirmed empirically against the pre-stone binary: a call
    // passing a malformed node passes `--check` and raises at run. The registry consult below now
    // answers for both directly.
    // Arc 255 Stone the-registry-answers-first-wave-2 — RETIRED. The fact this guard carried
    // (`total: true` for both) now lives at each verb's own registration
    // (`src/intrinsic/stream.rs:91,120`) as `@Totality Total` — re-derived: `empty` is a zero-arg
    // constructor with a single unconditional `Ok`; `cons`'s `other =>` `TypeMismatch` arm on
    // `tail` is checker-impossible (checked normally, not on the debt ledger, and `tail`'s
    // declared type is exactly `(Stream :- [T])`). Confirmed unchanged. The registry consult
    // below now answers for both directly.
    // Arc 255 Stone P6-c-W2 — `:wat::stream::next` (`src/intrinsic/stream.rs`) FORCES a thunk:
    // `crate::stream::realize` calls `apply_function` on a captured wat closure (a `Thunk`, the
    // body of `(:wat::stream::lazy <body>)`) or runs a Rust closure (a `NativeThunk`, backing
    // the lazy `map`/`filter`/`take`/`drop` family) — either can run ARBITRARY code this verb has
    // no way to bound: I/O, a clock read, randomness, a raise, another `next` on an unrelated
    // stream. Declaring it `pure`/`deterministic` here would be exactly the lie `apply`/`eval`
    // are deliberately left UNCLASSIFIED to avoid (see this fn's own doc, "purity is the form's,
    // like apply") — so this is a RULING of `false`/`false`/`false`, not a placeholder: someone
    // read the body and the answer is "no, on all three axes", which is why it is CLASSIFIED
    // (removed from `KNOWN_UNREVIEWED`) rather than left unreviewed. Independent corroboration:
    // `src/macros/eval.rs`'s `is_pure_total` expand-time-safe allowlist already listed
    // `cons`/`empty`/`lazy` and already did NOT list `next`, before this stone touched either
    // file — the same conclusion, reached by an unrelated mechanism built for an unrelated
    // reason.
    // Arc 255 Stone the-registry-answers-first-wave-2 — RETIRED, a pure duplicate:
    // `:wat::stream::next`'s own registration (`src/intrinsic/stream.rs:177`) already declares
    // `@Purity Effectful` / `@Determinism Nondeterministic` / `@Totality Unreviewed`,
    // byte-identical to this guard's `{pure: false, deterministic: false, total: false}`. Deleted
    // rather than moved — no fact to move, only a copy to remove. The registry consult below now
    // answers for it directly.
    // Arc 255 Stone the-registry-answers-first-wave-2 — RETIRED, and the re-derivation OVERTURNED
    // this guard's `total: true` for `vocabulary-admitted?`. `pure?`/`deterministic?`/`total?`/
    // `primitive?`/`cond-has-deferred-constraint?`'s `@Totality` now live at their own
    // registrations (`src/intrinsic/rete.rs:139,164,190,219,308`) as `Total`, confirmed unchanged:
    // each unwraps its `Value::wat__WatAST` arg (checker-impossible to miss — the WHOLE type has
    // one runtime representation regardless of AST shape) and delegates to a walk that collapses
    // every malformed-AST arm to `false` rather than raising. `vocabulary-admitted?`'s `@Totality`
    // (`src/intrinsic/rete.rs:249`) now reads `Partial` instead — its body destructures the
    // unwrapped AST a SECOND level, requiring specifically `WatAST::Keyword`, and nothing in the
    // declared `:wat::WatAST` type rules out a quoted List/Symbol/number reaching that `other =>`
    // `TypeMismatch` arm on a well-typed call (confirmed empirically against the pre-stone
    // binary). The registry consult below now answers for all six directly.
    // Arc 255 Stone the-registry-answers-first-wave-2 — RETIRED. The fact this guard carried
    // (`total: true` for all three) now lives at each verb's own registration
    // (`src/intrinsic/rete.rs:407,436,464`) as `@Totality Total` — re-derived: `cond`'s
    // `Value::wat__WatAST` unwrap is checker-impossible to miss; `fact`'s declared
    // `:wat::core::Record` type roots `Nature::Struct` at a DIFFERENT keyword with no subtype
    // edge to `:wat::core::Record` (`Nature::root_keyword`, `src/types.rs`), so `fact_from_value`'s
    // `None` (Struct/non-Aggregate) arm is checker-impossible for a well-typed argument; and
    // `alpha_match_inner`'s malformed-`cond` paths degrade to `None` via `alpha_pattern`'s `?`,
    // with the lone `unreachable!()` in `eval_clause` guarding an invariant the SAME table
    // (`classify_rete_clause`/`classify_constraint_head`) enforces on both sides, not a reachable
    // external input. Confirmed unchanged. The registry consult below now answers for all three
    // directly.
    // ── STONE meter-2 — seven verbs the widened `dispatch_verbs` scan newly finds, dispatched
    // all along from `dispatch_keyword_head` (one word off the anchored
    // `dispatch_keyword_head_value` — the exact miss `DESIGN-STONE-meter-2` names) and from
    // `resolve_verify_payload` (never anchored at all). Each ruling below is read from the
    // implementation the widened scan exposed, not guessed to keep the count down (STOP-2).
    //
    // Arc 255 Stone the-registry-answers-first-wave-3 — RETIRED, both re-derivations UNCHANGED.
    // `write-forms`'s `@Totality` now lives at its own registration (`src/intrinsic/ast.rs`) —
    // `eval_write_forms` (`src/edn/render.rs`) evaluates its one already-typed `:wat::WatAST`
    // argument (ordinary call-by-value, not itself an effect) and runs a pure structural
    // transform (`watast_to_edn` + `wat_edn::write`): no IO, no ambient state. `Partial`,
    // conservatively — the serializer's behavior over every WatAST variant was not independently
    // verified. `with-children`'s `@Totality` now lives at its own registration
    // (`src/intrinsic/ast.rs`) as `Partial` too — verified: a leaf template given non-empty
    // children, or a `Map` template given an odd child count, both raise `MalformedForm`
    // (`src/edn/render.rs`), the same well-typed-domain-restriction shape `i64::/` is `Partial`
    // for. The registry consult below now answers for both directly.
    //
    // ★ `macro-error`'s `@Totality` now lives at its own registration
    // (`src/intrinsic/macro_error.rs`) as `Partial`, RULED (not transcribed) against
    // `RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`: the body evaluates its
    // one String argument and UNCONDITIONALLY returns
    // `Err(EvalBreak::Diagnostic(Box::new(RuntimeError::new(.., RuntimeErrorKind::MacroAbort
    // {..}))))` — the SAME `Diagnostic` variant an ordinary `TypeMismatch`/`ArityMismatch` raise
    // uses, whose own doc (`src/value/signal.rs:70-72`) says it plainly: "carries a source
    // location and surfaces to user code as an error." That is the opposite variant from
    // `Option/try`/`Result/try`'s `EvalBreak::Signal(EvalSignal::OptionPropagate |
    // TryPropagate(_))` (`src/value/signal.rs:78-81`: "Caught at function boundaries; never
    // surfaces to user code"), which `apply_function` catches and repackages as the ENCLOSING
    // function's own checker-guaranteed `Option`/`Result` return — a real value the caller
    // `match`es. `macro-error`'s `Diagnostic` is caught nowhere at the wat-value level: it
    // unwinds past every enclosing wat form and is caught only by `macro_eval_pre_validated`
    // (`src/macros/eval.rs:109-116`), which matches on `e.kind()` and repackages it as a Rust
    // `MacroError` — a macro-EXPANSION-time (compile-time) failure, never a `Value` any wat code
    // receives or branches on. Confirmed empirically against the pre-stone binary: a direct call
    // passes `--check` (exit 0) and raises at run (`RuntimeError`/`MacroAbort`, exit 1) — the
    // same "passes check, raises at run" signature every other `Partial` raise in this file has.
    // `try`'s word "signal" and this verb's own doc comment calling it "first-class macro-abort"
    // resemble each other; the operative Rust type does not — it is `Diagnostic`, never `Signal`.
    // `Partial`. The registry consult below now answers for it directly.
    // `:wat::verify::string` / `:wat::verify::http-path` / `:wat::verify::s3-path`
    // (`resolve_verify_payload`, `src/runtime.rs:24271-24306` — never anchored, invisible to the
    // old two-anchor scan). `string` evaluates its one argument and returns it unchanged if it is
    // already a `String` (`TypeMismatch` otherwise — nothing ambient touched either way);
    // `http-path`/`s3-path` don't even evaluate their argument — the arm unconditionally raises
    // "reserved but not implemented in this build". All three: no IO, no ambient state (pure,
    // deterministic); `total: false` since each has a raise path on every reachable input class.
    if matches!(
        head,
        ":wat::verify::string" | ":wat::verify::http-path" | ":wat::verify::s3-path"
    ) {
        return Some(OpMeta { pure: true, deterministic: true, total: false });
    }
    // `:wat::verify::file-path` (`resolve_verify_payload`, `src/runtime.rs:24281-24297`) — reads
    // a FILE FROM DISK (`sym.source_loader()...fetch_payload_file`), the same class of externally
    // observable effect `:wat::io::`'s whole namespace is blanket `Impure` for. Ruled Impure
    // per-verb here (there is no `:wat::verify::` = Impure namespace rule — its siblings above
    // are pure), not via a namespace prefix, because the effect is this one verb's, not the
    // namespace's.
    if head == ":wat::verify::file-path" {
        return Some(OpMeta { pure: false, deterministic: true, total: false });
    }
    // ── arc 255 Stone total-T5 — THE REGISTRY ANSWERS ALL THREE AXES ────────────────────────────
    //
    // Every `#[wat_intrinsic]`-registered verb ALREADY declares `@Purity`/`@Determinism`/`@Totality`
    // at its registration site (mandatory on the first two since long before this campaign; T4b
    // armed the third). For any such verb, consult the registry directly instead of re-answering
    // via a second, hand-transcribed copy of the same fact — the exact shape `total-T4b` already
    // uses for the `total` axis alone (see this fn's own `total-T4b` comment below the `pure_det`
    // list), now extended to all three axes.
    //
    // `Purity::Preserving`/`Determinism::Preserving`/`Totality::Preserving` all satisfy their axis
    // — "I contribute no impurity/nondeterminism/partiality of my own; my sub-forms carry theirs" —
    // the same reading `intrinsic/mod.rs:1038`'s `purity_mandated_examples` already gives
    // `Purity`/`Determinism`, and `total-T4b` already gave `Totality`.
    //
    // ORDER MATTERS: this sits AFTER `rete_op_for` and every early-return special case above (each
    // already verified to agree with its own registration — retiring them is a follow-up, not this
    // stone), and BEFORE the residual `pure_det`/`total` hand-rulings below, which now answer ONLY
    // for a head this lookup misses (`lookup_entry(head) == None` — unregistered, not a name-list).
    //
    // DESIGN-STONE-total-t5-the-registry-answers-all-three-axes.md measured this change moves 275
    // verdicts (unregistered-by-this-fn's-old-reckoning verbs go from `None` to a declared ruling)
    // and that ZERO of them also pass `total` — every one carries `@Totality Unreviewed` — so the
    // four-axis `where` fence (pure ∧ det ∧ total ∧ primitive) admits exactly none of them.
    if let Some(e) = crate::intrinsic::registry().lookup_entry(head) {
        return Some(OpMeta {
            pure: matches!(e.purity, wat_doc::Purity::Pure | wat_doc::Purity::Preserving),
            deterministic: matches!(
                e.determinism,
                wat_doc::Determinism::Deterministic | wat_doc::Determinism::Preserving
            ),
            total: matches!(e.totality, wat_doc::Totality::Total | wat_doc::Totality::Preserving),
        });
    }

    // Pure ∧ deterministic explicit `:wat::core::` ops.
    //
    // arc 255 Stone total-T6 — this list used to carry ~103 additional names (per-type
    // arithmetic/comparison, the map/hashmap/vec/vector/linkedlist/hashset `/`-verb families,
    // uuid readers, the holon:: VSA seam, …) that `total-T5`'s registry consult (above) had
    // already made UNREACHABLE: every one of those names is `#[wat_intrinsic]`-registered, so
    // the registry branch answers before this `matches!` is ever reached. DERIVED and DELETED —
    // `registry().lookup_entry(name).is_some()` for every deleted name, verified against every
    // verb's `intrinsic_meta` verdict before and after (identical). What remains below is the
    // genuine backlog: dispatched verbs with NO registration yet, so this hand-list is still the
    // only place carrying their ruling. `:wat::core::when` — a ruling for a verb the language
    // does not have (`unknown function: :wat::core::when`) — was deleted in the same stone, by
    // name, not by this derivation (`lookup_entry` returns `None` for it too, same as every
    // survivor, but it resolves to nothing at all).
    //
    // ~~not/and/or, get/contains?/first/second/third/stream->vec, record?,
    //   PersistentVector/PersistentMap, foldl/map/mapv/filter, bool::to-string~~ — DELETED arc
    //   255 Stone `1c-c-the-residues-cannot-shadow-the-registry`. All 17 are now
    //   `#[wat_intrinsic]`-registered, so the `total-T5` registry consult above already answers
    //   first for every one of them and these arms were unreachable dead text — the identical
    //   "shadowed by a copy" defect `total-T6`'s own header names, this time found by a GATE
    //   (`the_residues_cannot_shadow_the_registry`, `src/intrinsic/mod.rs`) that asserts the
    //   rule stated above, rather than by a rider noticing. 38 named FQDNs down to 21.
    // ⛔ Arc 255 Stone 1c-c — `:wat::core::u8` and `:wat::core::do` LEFT this list. Both are
    // registered now (`u8` via `#[wat_intrinsic]`, `do` via `#[wat_special_form]` at Stone 1a-ζ),
    // so the registry consult above answers for them and these arms were unreachable. They
    // survived the stone's first sweep because the gate's arm-detector could not see an arm
    // followed by a COMMENT line — fixed in the same stone.
    // ⛔ Arc 255 Stone 1c-b-ii — `<`/`>`/`<=`/`>=` LEFT this list: all four are registered now
    // (`#[wat_intrinsic]` wrappers in `src/runtime.rs`, each `@Totality Total`), so the registry
    // consult above answers for them and these arms were unreachable. `=`/`not=` STAY — they are
    // deliberately UNregistered (held; see
    // `docs/arc/2026/06/255-builtin-registry/NOTE-equality-is-argued-proven-partial-and-held.md`),
    // so `lookup_entry` still returns `None` for them and the `total` fallback below is still the
    // only thing that can answer. That is the residue working as designed, not a shadow.
    let pure_det = matches!(
        head,
        // Arithmetic
        ":wat::core::+"
            | ":wat::core::-"
            | ":wat::core::*"
            | ":wat::core::/"
            // Comparison
            | ":wat::core::="
            | ":wat::core::not="
            // Control flow whose sub-items are ALL plain exprs — safe to recurse element-wise.
            // (`cond`/`match` are handled with dedicated clause-aware arms in classify_expr, NOT
            // here, because their clauses are not calls. `if`/`let` are registered — Stone
            // total-T6 deleted them from here; `when` never existed as a verb and was deleted by
            // name, not derivation.)
            // Collection/map/vector readers and predicates — UNHOMED (no registration).
            | ":wat::core::stream->pvec"
            | ":wat::core::str"
            // Bare TYPE constructors — the numerics/container HOME campaigns (arc 255 Stones
            // C/D/E-i/E-ii/E-iii) moved every `/`-verb OP to its own per-type namespace and
            // registered it (`:wat::vec::*`, `:wat::vector::*`, `:wat::linkedlist::*`,
            // `:wat::hashset::*`, `:wat::map::*`, `:wat::hashmap::*`, `:wat::uuid::*`) — those
            // are all deleted from here this stone (the registry answers them now). The bare
            // constructor keyword for each type (STOP-3 in each of those stones) stays unhomed.
            // `:wat::core::List` (the bare type, no `?`) IS registered and was deleted; `List?`
            // (the predicate) is not.
            | ":wat::core::HashMap"
            | ":wat::core::Vector"
            | ":wat::core::List?"
            | ":wat::core::HashSet"
            // Higher-order fold combinators — CONDITIONALLY pure∧det: the combinator itself is
            // referentially transparent + effect-free; its purity/determinism falls out of the
            // arg-recursion over its fn-argument (classify_expr recurses every arg, incl. the
            // fn-literal, whose body is classified by the `:wat::core::fn` arm). An impure fn-arg
            // therefore still fails — conditional purity, not blanket-allow.
            | ":wat::core::reduce"
            // Scalar conversions — total, same-in-same-out.
            | ":wat::core::i64/to-f64" | ":wat::core::i64/to-string"
    );

    // ── `total` — arc 255 total-T4b: DERIVED from the registry, an 11-name backlog left ────────
    //
    // DEFAULT-DENY over the WHOLE `pure_det` list above: every verb defaults to `total: false`
    // regardless of membership in `pure_det` (pure∧deterministic says nothing about totality —
    // `i64::/` is both, and undefined at a zero divisor).
    //
    // BRIEF-total-t1-the-axis-unarmed.md through total-T4a built and verified a 38-name hand-list
    // here, each verdict earned by READING the verb's own implementation (never inferred from the
    // name) against the 9-file / 98-row `where`-corpus (`wat-scripts/perf/grid/where-*.wat`).
    // Total-T4a then moved 27 of those 38 verdicts to their own registration site (an `@Totality`
    // line in the verb's doc block) — the reasoning lives there now, not here. Total-T4b (this
    // stone) makes the lookup below CONSULT those 27 sites instead of keeping a second copy of a
    // fact the registry already holds, the exact shape 255.1c retired as "a gate reading a copy
    // of the truth" (`intrinsic/mod.rs:988`, `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`).
    // `Totality::Total` and `Totality::Preserving` BOTH satisfy the axis — `Preserving` means "I
    // contribute no partiality of my own; my sub-forms carry theirs," the same reading
    // `pure`/`deterministic` already give it (`intrinsic/mod.rs:1038`'s
    // `matches!(purity, Pure | Preserving)`) — `Totality::Partial` fails it, and
    // `Totality::Unreviewed`/no registration at all falls through to the `matches!` below.
    //
    // ★ THAT `matches!` IS A HOMING BACKLOG, NOT THE HAND-LIST IT REPLACED. Each of its 3 names
    // is unhomed — no registration exists yet to carry its ruling — so the verdict for exactly
    // these three stays HERE until one exists. Homing a name retires its row: move its reasoning
    // to the registration site (the same motion `if`/`let` and their 25 siblings already made)
    // and delete the arm. A verb that IS registered does not belong in this list (row 5's own
    // gate) — if one shows up here alongside a registration, the derivation above is being
    // shadowed by a copy, which is the exact defect this stone exists to remove.
    //
    //   ~~and/or/not/bool::to-string, map/mapv/filter/foldl~~ — DELETED arc 255 Stone
    //     `1c-c-the-residues-cannot-shadow-the-registry`. All 8 are now `#[wat_intrinsic]`-
    //     registered, so the registry consult above already answers first for every one of
    //     them and these arms were unreachable dead text — the identical "shadowed by a copy"
    //     defect this list's own header names, this time found by a GATE
    //     (`the_residues_cannot_shadow_the_registry`, `src/intrinsic/mod.rs`) that asserts the
    //     rule stated above, rather than by a rider noticing. 11 named FQDNs down to 3.
    //
    //   `=`/`not=` — the remaining P6-c dispatch population: value ops with no domain
    //     restriction (a well-typed call always returns; type mismatches are the type
    //     checker's concern, not this axis's, exactly the convention `pure`/`deterministic`
    //     already use). Their typed siblings (`i64::=`/`i64::not=`/`i64::to-string`/
    //     `f64::to-string`, …) are homed and registered `@Totality Total`; these
    //     generic/untyped forms are not yet.
    //   `reduce` — the last of the W7 HOF family still unhomed. A combinator's totality is
    //     CONDITIONAL on its fn-argument, and `classify_expr`'s general-list arm already
    //     resolves that conditionality by recursing into the fn-literal body and checking IT
    //     against `Axis::Total` too — so `total: true` on the HEAD means exactly what
    //     `pure: true`/`deterministic: true` already mean: "the combinator itself adds no
    //     partiality of its own," proven by run on `foldl` before IT homed:
    //
    //       (total? '(foldl (fn [a b] (rete i64::+ a b :undefined 0)) 0 xs))  -> TRUE
    //       (total? '(foldl (fn [a b] (core i64::/ a b))              0 xs))  -> FALSE
    //
    //     `foldr` is retired (arc 118.B6b — it was `reverse`+`foldl` wearing a name borrowed from
    //     Haskell, where the verb is distinct only because it is LAZY, a property strict wat
    //     cannot have); its replacement `(reduce f init (reverse coll))` is covered via `reduce`.
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
    let total = match crate::intrinsic::registry().lookup_entry(head).map(|e| e.totality) {
        Some(wat_doc::Totality::Total) | Some(wat_doc::Totality::Preserving) => true,
        Some(wat_doc::Totality::Partial) => false,
        // No registration to consult: the verb is not homed yet. These three keep their ruling
        // here until they have a registration site to carry it — see the comment block above
        // this match for the per-verb reasoning (the remaining P6-c dispatch population, then
        // the last unhomed member of the W7 HOF family).
        Some(wat_doc::Totality::Unreviewed) | None => matches!(
            head,
            ":wat::core::reduce" | ":wat::core::=" | ":wat::core::not="
        ),
    };

    if pure_det {
        Some(OpMeta { pure: true, deterministic: true, total })
    } else {
        None
    }
}

// ─── Per-head leaf decision ─────────────────────────────────────────────────────

/// INTERIM recognizer keyed on the frozen TypeEnv. Arc 255 is the registry
/// (`docs/arc/2026/06/255-builtin-registry/`).
/// Is `items` a MACRO-LOWERED construction of a **declared** aggregate?
///
/// `(:wat::core::kwargs-construct :cg::Rate :count c …)` / `(:wat::core::aggregate-new :cg::Rate c …)`
/// — the two forms a record/struct's surface constructor lowers to, where the TYPE is argument 0
/// rather than the head. `constructor_meta` recognises the pre-lowering shape (type AS head); this
/// recognises the post-lowering one, so the declaration-derived door covers both spellings of the
/// same act. See the call site's comment for why that is the law working, not an exception to it.
///
/// ⛔ TIGHT BY CONSTRUCTION: the verb alone is NOT enough — argument 0 must resolve to a declared
/// `TypeDef::Aggregate`, the identical test `constructor_meta` applies to a head. A bare
/// `(:wat::core::kwargs-construct x 1)` over an undeclared name stays refused, and the gate proves
/// that direction too. Without it this door would admit anything wearing the verb.
fn is_declaration_derived_construction(items: &[WatAST], sym: &SymbolTable) -> bool {
    let Some(WatAST::Keyword(head, _)) = items.first() else { return false };
    if head != ":wat::core::kwargs-construct" && head != ":wat::core::aggregate-new" {
        return false;
    }
    let Some(WatAST::Keyword(type_name, _)) = items.get(1) else { return false };
    let Some(types) = sym.types_deref() else { return false };
    matches!(types.get(type_name.as_str()), Some(crate::types::TypeDef::Aggregate(_)))
}

/// `constructor_meta`'s two sites, AUDITED (BRIEF-constructor-meta-audit.md) — closing the
/// inconsistency `b98cf189` named and did not touch: that strike classified the EXPANDED
/// constructor verbs (`aggregate-new` / `kwargs-construct`, the block above) `pure ∧
/// deterministic ∧ total` and left THIS function's two return sites — the SURFACE form (a bare
/// `(:T arg…)` / `(:Enum::Variant arg…)` written directly, e.g. inside a quoted `:then`/`:when`
/// item, never macro-expanded there — on the OLD default-deny discipline. Ground, per site,
/// per axis, independently; do not average the two sites and do not inherit the expanded
/// verdict without re-deriving it (the surface and expanded forms are walked by DIFFERENT
/// code, so agreement is not automatic).
///
/// ## `pure` — UNIFIED with the expanded verdict, now independently grounded (not inherited)
///
/// `constructor_meta` used to derive `pure` from the target's declared purity marker
/// (`Nature::is_pure()` for an aggregate — Record/HolonRecord pure, Struct impure;
/// `Purity::is_pure()` for an enum's `:wat::enum::Pure`/`Impure` marker) — i.e. purity of the
/// THING BUILT, not of the ACT of building it. `b98cf189` dropped that dependency for the
/// EXPANDED forms, arguing construction is assignment: it binds already-evaluated field values
/// into a shape, acquires nothing, and any actual resource ACQUISITION inside an argument is
/// caught independently, because `classify_expr`'s "General list" arm (below) recurses into
/// EVERY argument of EVERY call form on the SAME axis, `head_ok`'s verdict at the outer head
/// notwithstanding (`classify_expr`, the `WatAST::List` arm: `head_ok(...)?` then
/// `for a in &items[1..] { classify_expr(a, axes, sym, seen)?; }` — unconditional, common code,
/// not specific to any one `head_ok` branch).
///
/// That recursion is IDENTICAL regardless of which branch of `head_ok` admitted the outer head —
/// this function's aggregate/enum-variant branches recurse through the exact same call site as
/// the `aggregate-new`/`kwargs-construct` branch above. So an acquisition nested in a surface
/// constructor's argument (`(:usr::Handle :conn (:wat::io::IOReader/open-file "…"))`) is refused
/// at the ARGUMENT's own head regardless of what this function returns for the outer `:usr::Handle`
/// — STOP-2's question (a route by which a resource reaches a constructor argument, bypassing the
/// fence) has no answer: `?var`/literal arguments (`(:usr::Handle :conn ?bound-resource)`) are
/// values that already exist (the legitimate, intended case: read a resource out of one fact,
/// carry it into another), `let`/`fn`/`cond`/`match` sub-forms are walked element-wise by their
/// own dedicated `classify_expr` arms (each sub-expression re-enters the same recursive walk), and
/// a user-fn argument recurses through `classify_fn` into that fn's own body on the same axis. No
/// gap found by reading the walk.
///
/// The nature/purity-marker dependence is ALSO vacuous on the "declared pure" side for both
/// sites: `validate_aggregate_containment` (check.rs:12578) is a post-registration freeze pass
/// that rejects STARTUP for a `Nature::Record`/`HolonRecord` aggregate declaring an impure field
/// (`TypeErrorKind::ImpureFieldInPureAggregate`) — AND, arc 293.W.2b, for its "enum counterpart":
/// a `:wat::enum::Pure` enum may declare only pure variant fields (check.rs:12598). So
/// `nature.is_pure()`/`e.purity.is_pure()` can only ever discriminate the "declared impure"
/// side (`Nature::Struct`, `Purity::Impure`) — exactly the side the argument above already covers
/// (`eval_variant`, runtime.rs:13608-13658, mirrors `construct_aggregate`'s pure "evaluate args,
/// wrap into a value" shape — no IO, no mutation, on either path). Both sites: `pure: true`,
/// unconditionally, matching the expanded forms — the reasoning applies to the ACT of
/// construction regardless of which of the two syntactic forms (surface or expanded) performs it.
///
/// ## `total` — NOW `true` at BOTH sites. BRIEF-construction-total-three-walls.md closed the
/// three fire-time failures `d6c32cf5`'s audit measured (below, for the record — this doc used
/// to justify `false` with them; it now records what CLOSED each one, so it never ages into a
/// stale justification the way the pre-audit comment did).
///
/// `b98cf189` earned `total: true` for the EXPANDED forms by closing two checker gaps — (a)
/// `infer_aggregate_new_check`/`infer_kwargs_construct_check` (check.rs) validate arity/field
/// names, and (b) `freeze::validate_holon_record_capacity` validates a HolonRecord's dim budget
/// — both running whenever the CONSTRUCTING CODE (a `defn` body) is itself type-checked. The
/// surface form's problem was structural, not a missing check to port: a `:then`/`:when` item is
/// captured under `(:wat::core::quote …)` by `defrule`'s macro template (`wat/rete/syntax.wat`),
/// and `expand_form`'s recursive macro-expansion walk stops dead at that `quote` boundary
/// (`src/macros/expand.rs:436-444`) — so the bare surface head is what a `:then`/`:when` item
/// carries forever, and `--check`'s `infer` does not recurse into quoted data either
/// (`check.rs:3076-3088`, `:wat::core::quote` returns `:wat::WatAST` without inspecting its
/// argument). NEITHER of gap (a)'s checkers, nor `--check` generally, ever sees this form. Gap
/// (b) alone transferred cleanly (it is keyed on the TYPE, at freeze, independent of which call
/// form constructs it) — it was (a)'s closure that did not carry over, and it did not carry over
/// identically for the two sites, so each needed its OWN wall:
///
/// **Aggregate site** — closed by TWO new/widened freeze-time walls, both in
/// `src/rete/validate.rs`, plus one runtime wiring fix (`src/runtime.rs`):
///   1. (#1, the one WIRED rather than rejected) A nested surface aggregate constructor (an
///      operand's VALUE, not the `:then` item's own head — e.g. `:then [(:usr::Outer :inner
///      (:usr::Inner :x 1))]`) used to compile clean and die at FIRE time with
///      `RuntimeErrorKind::UnknownFunction`, unconditionally, regardless of arity —
///      `dispatch_keyword_head_value`'s fallback (`runtime.rs`, ahead of its final
///      `UnknownFunction`/keyword-accessor arms) had no arm recognizing a bare aggregate-type
///      keyword as a constructor. Nothing about the form was illegal — it was simply never
///      wired: the fallback now recognizes a bare keyword resolving to `TypeDef::Aggregate` and
///      delegates to `eval_kwargs_construct` (the SAME dispatch the macro-expanded
///      `:wat::core::kwargs-construct` verb already used), so a nested constructor now
///      evaluates identically to its expanded-form twin.
///   2. (#2) `validate_then_form`'s kwargs branch used to check every SUPPLIED field name
///      is real but never that ALL declared fields were supplied — `reorder_kwargs_by_field_name`
///      itself still doesn't require full coverage (see its own doc), but its callers now do.
///      STOP-A audited the whole corpus for a `:then` that under-supplies before closing this:
///      NONE found (every kwargs `:then` in `wat/`, `wat-scripts/`, and `tests/` already supplies
///      every declared field) — the doc line calling under-supply "pre-existing, unchanged" was
///      describing an accident nobody depended on, so closing it was free. A new
///      `RhsMissingFields` finding now names the rule, the type, and the missing fields.
///   3. A THIRD failure mode surfaced only once #1 was WIRED (it could not be measured before,
///      since any nested constructor died `UnknownFunction` regardless of its own shape): a
///      nested constructor operand can itself be malformed (unknown/missing field, or a
///      multi-arg RAW POSITIONAL call — `eval_kwargs_construct` unconditionally retires that
///      shape at a bare aggregate name). `validate_then_form` now walks every `:then`
///      value-position operand RECURSIVELY (`walk_nested_constructors`, unbounded depth, mirroring
///      `classify_expr`'s own unconditional per-argument recursion for `pure`, above) and
///      validates any nested aggregate-constructor call it finds the same way it validates a
///      `:then` item's own top-level shape.
///
/// **Enum-variant site** — closed by resolving a bare `:Enum::Variant` head at freeze, which
/// `lookup_fields` never did (it resolves only `TypeDef::Aggregate`). A tagged-variant
/// constructor IS a real, directly-callable `FunctionBody::Wat` function
/// (`register_enum_methods`, runtime.rs — its body is literally `(:wat::core::variant :Enum
/// :Variant p1…pn)`), reached through the SAME `sym.get(canonical)` / `apply_function` path as
/// any ordinary fn call, so — unlike the aggregate site's dead/unwired shape — every call always
/// reached `apply_function`'s unconditional arity gate; that gate just fired at RUNTIME, not at
/// `--check`/freeze, and the ordinary `--check` path that would catch this in NON-quoted code
/// never ran here either (same `quote` boundary as the aggregate site). `walk_nested_constructors`
/// (the SAME recursive walk #1's third failure mode needed) now also resolves a bare
/// `{EnumPath}::{Variant}` head against the `TypeEnv`'s `TypeDef::Enum` and compares the
/// variant's declared field count (`Unit` → 0, `Tagged` → `fields.len()`) against the supplied
/// arg count, naming the rule, the full variant path, and the actual/expected arity.
///
/// Both sites: `total: true`, EARNED — every failure mode the audit measured (and the one #1's
/// own wiring newly exposed) now has a freeze-time wall naming it, in place of a fire-time
/// surprise naming only the reader that tripped over it.
///
fn constructor_meta(head: &str, sym: &SymbolTable) -> Option<OpMeta> {
    let types = sym.types_deref()?;
    // TypeEnv keys carry the leading colon (e.g. ":p::Rec") — use the head verbatim.
    // 1. Aggregate constructor (record / holon / struct) — the head IS the type name.
    //    `total: true` — EARNED, arc 278 BRIEF-construction-total-three-walls.md: #1 wired a
    //    nested surface constructor to the same dispatch the expanded form uses, #2 walled
    //    top-level kwargs under-supply, and the recursive `walk_nested_constructors` walls a
    //    nested constructor's own shape (unknown/missing field, retired raw-positional) too.
    if let Some(crate::types::TypeDef::Aggregate(_)) = types.get(head) {
        return Some(OpMeta { pure: true, deterministic: true, total: true });
    }
    // 2. Enum-variant constructor — the head is `{EnumPath}::{Variant}` (unit or tagged).
    //    `total: true` — EARNED, #3: `walk_nested_constructors` now resolves a bare
    //    `:Enum::Variant` head against the TypeEnv and walls a wrong-arity call at freeze.
    if let Some((enum_path, variant)) = head.rsplit_once("::") {
        if let Some(crate::types::TypeDef::Enum(e)) = types.get(enum_path) {
            let is_variant = e.variants.iter().any(|v| match v {
                crate::types::EnumVariant::Unit(n) => n == variant,
                crate::types::EnumVariant::Tagged { name, .. } => name == variant,
            });
            if is_variant {
                return Some(OpMeta { pure: true, deterministic: true, total: true });
            }
        }
    }
    None
}

/// A generated field ACCESSOR (`{TypePath}/{field}`) is as pure as the aggregate it reads: a
/// Record/HolonRecord accessor is pure ∧ deterministic, a Struct accessor is impure (a struct can
/// hold a live resource, arc 293.W) — the exact declaration `constructor_meta` / `is_pure_type`
/// reads. Declaration-read from the frozen TypeEnv (resolve the type, don't string-match), so it
/// covers every user record; NOT a hand-list. INTERIM recognizer keyed on the frozen TypeEnv.
/// Arc 255 is the registry.
fn accessor_meta(head: &str, sym: &SymbolTable) -> Option<OpMeta> {
    let types = sym.types_deref()?;
    // Accessors register as `{agg.name}/{field}` (runtime.rs); `agg.name` carries the leading
    // colon (e.g. ":wat::telemetry::Log"), so the type-path splits off verbatim for `types.get`.
    if !head.contains('/') {
        return None;
    }
    let (type_path, field) = (wat_reader::identifier::receiver(head), wat_reader::identifier::method(head));
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
    // head shape is not the flat `Type/field` form resolved above (STOP-3). The RED gate
    // covers records. Enum accessors default-deny until a row names them.
    None
}

/// Arc 255 Stone A-2-i — which WORLD is this classification happening in? Every call site
/// names it explicitly; there is no default to omit and nothing to forget.
///
/// `Option<&Environment>` was rejected mid-flight: `None` would have meant two different
/// things at a call site — "there is genuinely no environment here" (static context, check
/// time, no values exist yet) vs. "I did not bother threading one" — and nothing at the call
/// site would have told a reader which. That conflation is the exact defect this arc exists to
/// kill (the same reason `Totality` carries an `Unreviewed` variant instead of a guessed pole).
#[derive(Clone, Copy)]
pub(crate) enum ClassifyCtx<'a> {
    /// No values exist yet — check time, a quoted form, a rule body being proved at
    /// definition time. Captured-fn resolution is IMPOSSIBLE here, not merely skipped: there is
    /// no environment to look a name up in. `head_ok`'s new door is simply not tried.
    Static,
    /// Values exist; a bare head naming a local binding may be resolved through this
    /// environment (arc 255 Stone A-2-i — `head_ok`'s new door, immediately before its final
    /// default-deny).
    Runtime(&'a Environment),
}

/// Does `head` satisfy `axis`? Data constructors and field accessors are recognized first
/// (pure-by-declaration); then user fns recurse transitively; intrinsics consult
/// `intrinsic_meta`; under `ClassifyCtx::Runtime`, a local binding holding a closure is resolved
/// (arc 255 Stone A-2-i) immediately before the final default-deny; unknown heads default-deny.
fn head_ok(
    head: &str,
    axis: Axis,
    sym: &SymbolTable,
    seen: &mut HashSet<String>,
    closure_seen: &mut HashSet<*const Function>,
    at: &Span,
    ctx: ClassifyCtx,
) -> Result<(), AxisViolation> {
    // Data constructor (record/holon/enum-variant pure; struct impure) — recognized BEFORE the
    // sym.functions branch, because tagged-variant constructors are registered there as opaque stubs
    // that classify_fn would default-deny.
    if let Some(m) = constructor_meta(head, sym) {
        let ok = match axis {
            Axis::Pure => m.pure,
            Axis::Deterministic => m.deterministic,
            Axis::Total => m.total,
            // LAW A — a DECLARATION-DERIVED head is admissible whatever its namespace. A record's
            // constructor and its field accessors exist by construction of the type; `Client/rep`
            // will never be rete-namespaced and must never need to be. The design stone says so
            // outright ("Error 1 — it would kill composition"): declaration-derived is a STRONGER
            // warrant than a namespace, so law A does not reach these two doors.
            Axis::RetePrimitive => true,
        };
        return if ok { Ok(()) } else { Err(AxisViolation::at(at.clone(), head, axis)) };
    }
    // Generated field accessor (`Type/field`) — same declaration-read as constructors, and likewise
    // BEFORE the sym.functions branch: accessors register there as Native stubs that classify_fn
    // default-denies, so we MUST intercept the accessor here.
    if let Some(m) = accessor_meta(head, sym) {
        let ok = match axis {
            Axis::Pure => m.pure,
            Axis::Deterministic => m.deterministic,
            Axis::Total => m.total,
            // LAW A — a DECLARATION-DERIVED head is admissible whatever its namespace. A record's
            // constructor and its field accessors exist by construction of the type; `Client/rep`
            // will never be rete-namespaced and must never need to be. The design stone says so
            // outright ("Error 1 — it would kill composition"): declaration-derived is a STRONGER
            // warrant than a namespace, so law A does not reach these two doors.
            Axis::RetePrimitive => true,
        };
        return if ok { Ok(()) } else { Err(AxisViolation::at(at.clone(), head, axis)) };
    }
    // User-defined fn → classify_fn (below). Arc 278 #88 — THE MEMBRANE lives INSIDE
    // classify_fn's `FunctionBody::Wat` arm now, not here: a Wat-bodied fn is admitted on the
    // strength of its DECLARATION (`Function::rete`), never by re-walking its body. This branch
    // itself is unchanged so the `FunctionBody::Native` arm (native HOF combinators —
    // `foldl`/`map`/… — registered in `sym.functions` and judged by `intrinsic_meta`, same as
    // always) keeps working exactly as before; only the Wat arm's admission rule flipped.
    // Arc 278 #88 — THE MEMBRANE, unchanged: `sym.has_function` resolves a NAMED (defn-registered)
    // fn. Its body is never lexically inside whatever scope is being classified here (a `defn`'s
    // `closed_env` is always `None` — it resolves symbols via the global `sym`, not a captured
    // env), so `ClassifyCtx::Static` is deliberately forced here instead of forwarding `ctx`:
    // passing `Static` is not a "safe default", it is the ONLY correct world for this recursion,
    // on scope grounds alone — whatever `ctx` this call was made under, a NAMED fn's own body is
    // never lexically nested in it.
    if sym.has_function(head) {
        // ⛔ `ClassifyCtx::Static` is FORCED here, NOT forwarded — and forwarding the caller's
        // `ctx` would be a silent SCOPE bug, not a safe-default question. A `defn`-registered
        // function's body is never lexically inside the scope that triggered this
        // classification: `Function` carries `closed_env = None` for named fns (they resolve
        // through the global `SymbolTable` at call time — `src/value/environment.rs:44`), so the
        // caller's environment is not this body's environment. Handing it down would let a named
        // fn's body resolve a head through bindings it can never actually see.
        // Captured-closure resolution belongs to `classify_closure`, reached from the
        // `ClassifyCtx::Runtime` door below, which carries the closure's OWN `closed_env`.
        return classify_fn(head, axis, sym, seen, closure_seen, at, ClassifyCtx::Static);
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
            // A head with a RETE_OPS row IS a rete primitive by construction — that is what the
            // row means. An admitted module whose specific verb has no row still default-denies
            // via `is_some_and`, exactly as on the other three axes.
            Axis::RetePrimitive => true,
        });
        return if ok { Ok(()) } else { Err(AxisViolation::at(at.clone(), head, axis)) };
    }
    // Arc 255 Stone A-2-i — THE NEW DOOR, immediately before the final default-deny: every prior
    // door has declined (not a constructor, accessor, registered fn, or admitted rete-vocabulary
    // member), so ask whether `head` names a LOCAL BINDING holding a closure — the shape
    // `sort-by`'s comparator `(fn [a b] (< (keyfn a) (keyfn b)))` needs for `keyfn`. Under
    // `ClassifyCtx::Static` no values exist yet, so this door is simply not tried — that is the
    // whole point of the enum, not a special case of it. Under `Runtime(env)`, if `env` resolves
    // `head` to a `Value::wat__core__fn(f)`, classify `f`'s body against the SAME axis, carrying
    // `f`'s OWN `closed_env` — the scope the closure was CREATED in, never the caller's (`ctx`
    // itself is not forwarded past this point; only a fresh ctx built from `f.closed_env` is).
    if let ClassifyCtx::Runtime(env) = ctx {
        if let Some(bound) = env.lookup(head, at) {
            if let Value::wat__core__fn(f) = bound.value() {
                return classify_closure(f, axis, sym, seen, closure_seen, at);
            }
            // Bound to a non-fn value — nothing to classify; fall through unchanged.
        }
        // No binding of this name in scope — fall through unchanged.
    }
    match axis {
        // Pure: effectful namespaces are an explicit deny; otherwise the metadata must declare pure.
        Axis::Pure => {
            if is_effectful_op(head) {
                return Err(AxisViolation::at(at.clone(), head, axis));
            }
            if intrinsic_meta(head).is_some_and(|m| m.pure) {
                Ok(())
            } else {
                Err(AxisViolation::at(at.clone(), head, axis))
            }
        }
        // Deterministic: the metadata must declare deterministic (effectful/unknown ⇒ None ⇒ deny,
        // which is correct — IO and unknown ops are not referentially transparent).
        Axis::Deterministic => {
            if intrinsic_meta(head).is_some_and(|m| m.deterministic) {
                Ok(())
            } else {
                Err(AxisViolation::at(at.clone(), head, axis))
            }
        }
        // Total (BRIEF-total-t1-the-axis-unarmed.md) — same default-deny discipline as
        // Deterministic: the metadata must declare total, unknown ⇒ None ⇒ deny.
        Axis::Total => {
            if intrinsic_meta(head).is_some_and(|m| m.total) {
                Ok(())
            } else {
                Err(AxisViolation::at(at.clone(), head, axis))
            }
        }
        // ★★ LAW A, ARMED. Reaching this line means every prior door declined: not a constructor,
        // not an accessor, not a user fn (that door RECURSES, so a composed fn is judged by its
        // CONTENTS, never its name), and not an admitted rete-vocabulary member. What is left is a
        // core-spelled operation inside a `where` — the one thing law A exists to refuse.
        //
        // The builder's law: *"the entire rete query language may only be composed from rete
        // primitives."* No `intrinsic_meta` consultation here, deliberately: being pure,
        // deterministic and total does not make an op rete. `:wat::core::>` is all three and is
        // still refused — which is exactly why this axis exists instead of borrowing one of theirs.
        Axis::RetePrimitive => Err(AxisViolation::at(at.clone(), head, axis)),
    }
}

/// Arc 255 Stone A-2-i — classify an ANONYMOUS closure resolved through `ClassifyCtx::Runtime`
/// (`head_ok`'s new door). An anonymous closure has `name: None` and is absent from `sym`, so the
/// FQDN-keyed `seen` cannot hold it (`classify_fn`'s guard, unusable here) — recursion is guarded
/// on the `Arc<Function>` POINTER ADDRESS instead, in `closure_seen`, a set kept separate from
/// `seen`'s FQDN keys, mirroring `src/value/value.rs:684`'s existing `Arc::ptr_eq` fn-identity
/// idiom. A back-edge returns `Ok(())`, exactly as `classify_fn`'s FQDN back-edge does.
///
/// ⛔ NOT a depth bound: this classifier's `false` must mean *proven not*, never *gave up* — a
/// recursion-depth limit would silently return the wrong answer on a deep-but-finite capture
/// chain (`[[feedback_an_error_names_where_it_gave_up_not_what_is_missing]]`).
fn classify_closure(
    f: &Arc<Function>,
    axis: Axis,
    sym: &SymbolTable,
    seen: &mut HashSet<String>,
    closure_seen: &mut HashSet<*const Function>,
    at: &Span,
) -> Result<(), AxisViolation> {
    let ptr: *const Function = Arc::as_ptr(f);
    if closure_seen.contains(&ptr) {
        return Ok(()); // back-edge — no new violation from the recursive call
    }
    closure_seen.insert(ptr);
    // Carry `f`'s OWN captured environment — the scope the closure was CREATED in — not
    // whatever `ClassifyCtx` this recursion was reached under. Shared by both arms below (arc
    // 255 Stone A-2-ii-a widened this from the Wat arm alone): a Native fn carries no captured
    // env today — nothing in this codebase constructs one with `closed_env: Some(..)` — so this
    // reduces to `ClassifyCtx::Static` there in practice, but the shape stays uniform rather
    // than special-cased per body kind.
    let child_ctx = match &f.closed_env {
        Some(closed_env) => ClassifyCtx::Runtime(closed_env),
        None => ClassifyCtx::Static,
    };
    match &f.body {
        FunctionBody::Wat(body_ast) => {
            classify_expr(body_ast.as_ref(), std::slice::from_ref(&axis), sym, seen, closure_seen, child_ctx)
        }
        // Arc 255 Stone A-2-ii-a — REACH-INDEPENDENCE: a named native gets the SAME verdict a
        // head named the same way would get, routed through `head_ok`'s one door ladder
        // (constructor_meta -> accessor_meta -> sym.has_function/classify_fn -> intrinsic_meta
        // -> deny) instead of consulting `intrinsic_meta` alone — the asymmetry
        // DESIGN-STONE-A-2-ii-a-a-resolved-name-gets-the-same-doors-as-a-head.md measured. Both
        // recursion guards ride along in the exact calling convention `head_ok` already takes
        // everywhere else: `seen` (FQDN back-edge, inside `classify_fn` via the
        // `sym.has_function` door) and `closure_seen` (pointer back-edge, already inserted
        // above — so a native reachable from its own resolution, e.g. via the `ClassifyCtx::
        // Runtime` env-lookup door at the tail of `head_ok`, hits THIS fn's own guard at the
        // top and returns `Ok(())` rather than recursing again). This is not a second thread of
        // recursion to guard independently; it is the SAME `head_ok` recursion, just entered
        // with a name instead of a call-site head string.
        //
        // An **anonymous** native (`name: None`) keeps A-2-i's exact behaviour: default-deny.
        // Nothing names it, so `head_ok` — which classifies a NAME — is never consulted; there
        // is no second ladder here; the two arms cannot drift because there is only one.
        FunctionBody::Native => match f.name.as_deref() {
            Some(name) => head_ok(name, axis, sym, seen, closure_seen, at, child_ctx),
            None => Err(AxisViolation::at(at.clone(), "<anonymous native fn>", axis)),
        },
    }
}

// ─── Shared structural walk (parameterized by axis) ─────────────────────────────

/// Refuse `cond` / `match` / `fn` as a RETE PRIMITIVE when more than one axis is
/// being asked at once — a narrow, non-recursive guard on `items.first()`.
///
/// It fires only for a multi-axis query that includes `RetePrimitive`: those three
/// core heads are pure, deterministic and total, so a single-axis walk must keep
/// admitting them; it is only as a rete primitive that they are refused. The
/// recursive walk this guards is [`classify_expr`], directly below.
fn refuse_core_structural_on_multi(axes: &[Axis], items: &[WatAST]) -> Result<(), AxisViolation> {
    if axes.len() <= 1 || !axes.contains(&Axis::RetePrimitive) {
        return Ok(());
    }
    if let Some(WatAST::Keyword(k, s)) = items.first() {
        if crate::rete::vocabulary::rete_op_for(k).is_none()
            && matches!(
                crate::rete::vocabulary::resolve_core_name(k),
                ":wat::core::cond" | ":wat::core::match" | ":wat::core::fn"
            )
        {
            return Err(AxisViolation::at(s.clone(), k.clone(), Axis::RetePrimitive));
        }
    }
    Ok(())
}

/// Recursively classify an AST node against `axes` (one or all four). One structural
/// walk; `head_ok` per axis at each call head (Pure → Det → Total → Rete).
///
/// The `WatAST::List` arm recurses into EVERY argument of EVERY call form on the
/// same axis, whatever `head_ok` returned at the outer head — so a resource
/// acquisition buried in an argument is caught even under a head the axis admits.
fn classify_expr(
    ast: &WatAST,
    axes: &[Axis],
    sym: &SymbolTable,
    seen: &mut HashSet<String>,
    closure_seen: &mut HashSet<*const Function>,
    ctx: ClassifyCtx,
) -> Result<(), AxisViolation> {
    match ast {
        // Non-list forms are pure, deterministic data.
        WatAST::IntLit(_, _)
        | WatAST::FloatLit(_, _)
        // Arc 300 stone B — rational literal is pure, deterministic data.
        | WatAST::RationalLit(_, _)
        // Arc 300 stone C1 — bigint literal is pure, deterministic data too.
        | WatAST::BigIntLit(_, _)
        // Arc 300 stone D — char literal is pure, deterministic data too.
        | WatAST::CharLit(_, _)
        | WatAST::BoolLit(_, _)
        | WatAST::StringLit(_, _)
        | WatAST::NilLit(_)
        | WatAST::Keyword(_, _)
        | WatAST::Symbol(_, _) => Ok(()),

        // ★★ LAW A AND THE DECLARATION-DERIVED CONSTRUCTOR, POST-LOWERING.
        //
        // ⛔ THE FAILURE THIS FIXES, and the user's mistake was NOTHING. This program is refused:
        //
        //     (:wat::core::defrecord :cg::Rate [count <- :wat::core::i64  window <- :wat::core::i64])
        //     (:wat::core::defn :cg::make-rate [c <- …  w <- …] -> :cg::Rate
        //       (:cg::Rate :count c :window w))                        ; ← the only way to build one
        //     (:wat::rete::defrule :cg::gather
        //       :when [(:cg::Anchor (?x <- :x))]
        //       :then [(:cg::make-rate 7 9)])
        //
        // Every field supplied, types right, fn pure ∧ det ∧ total. But `defrecord`'s macro lowers
        // `(:cg::Rate :count c :window w)` to `(:wat::core::kwargs-construct :cg::Rate …)` at
        // DEFINITION time, and law A then refused a `:wat::core::` head **the user never typed, in
        // a fn body rete never expanded**, offering a rete twin that cannot exist. The fence was
        // firing on its own lowering.
        //
        // ★ WHY THIS IS NOT AN EXCEPTION TO THE LAW. `head_ok`'s FIRST door is
        // `constructor_meta` — declaration-derived — and the design stone is explicit that it stays
        // open precisely because *"declaration-derived is a stronger warrant than a namespace"*
        // (the same reason `Client/rep` and `:usr::risk-score` need no rete name). A construction
        // of a DECLARED aggregate was always admissible; the door simply matched the pre-lowering
        // spelling, where the type is the HEAD, and could not see the post-lowering spelling, where
        // the type is ARGUMENT 0. This teaches the existing door the second shape — it does not add
        // a fourth conjunct, a vocabulary row, or a special case.
        //
        // Two verbs, because a record/struct lowers two ways and the substrate already names them
        // as one class (`intrinsic_meta`, `:283-286`; `BRIEF-construction-inside-a-fn.md`):
        // `kwargs-construct` (kwargs) and `aggregate-new` (positional).
        //
        // ⛔ THE PREDICATE IS TIGHT ON PURPOSE — argument 0 must RESOLVE to a declared
        // `TypeDef::Aggregate`, exactly as `constructor_meta` requires of a head. Accepting the
        // verb alone would make this door a hole wide enough for anything, so the gate proves BOTH
        // directions: a declared type is admitted, an undeclared one is still refused.
        //
        // Scoped to `RetePrimitive`: on Pure/Deterministic/Total these verbs are already classified
        // by `intrinsic_meta` and this arm must not disturb that. Field-value arguments still
        // recurse on the axis, so an impure or partial value inside a construction is caught at its
        // OWN head, exactly as before.
        WatAST::List(items, _)
            if axes.contains(&Axis::RetePrimitive) && is_declaration_derived_construction(items, sym) =>
        {
            for arg in &items[1..] {
                classify_expr(arg, axes, sym, seen, closure_seen, ctx)?;
            }
            Ok(())
        }

        // ★★ LAW A FOR THE STRUCTURAL-GUARD FORMS — `cond` / `match` / `fn`.
        //
        // ⛔ THE HOLE THIS CLOSES (found 2026-08-05 by the builder, proven by run in one probe):
        //
        //     (:wat::rete::primitive? '(:wat::core::cond  (true 1) (:else 2)))  -> TRUE
        //     (:wat::rete::primitive? '(:wat::core::match x (:wat::core::None 1))) -> TRUE
        //     (:wat::rete::primitive? '(:wat::core::fn [a <- …] -> … a))       -> TRUE
        //     (:wat::rete::primitive? '(:wat::core::> 1 0))                    -> false   (control)
        //
        // Law A ADMITTED the core spelling of all three, so a `where` could legally contain
        // `:wat::core::cond` and their rete twins were decorative. The builder's ruling is flat:
        // *"it may not — only rete forms and primitives are allowed in rete expressions."*
        //
        // MECHANISM: these three have STRUCTURAL arms below, which match before `head_ok` and
        // therefore NEVER REACH IT — and `head_ok`'s fallthrough is the only place law A's deny
        // lives. Their guards resolve through `resolve_core_name`, which normalises BOTH spellings
        // to the core name. That is CORRECT for the WALK (the form must be understood either way,
        // and `pure?`/`deterministic?`/`total?` are general predicates over ordinary core code) and
        // WRONG for ADMISSION. S5 widened the guard to accept the rete name and never closed the
        // core one; widening added, it did not replace.
        //
        // THE SPLIT THIS INTRODUCES, and it is the point: RECOGNITION stays spelling-agnostic;
        // ADMISSION is asked separately, here, and only on the `RetePrimitive` axis. So nothing
        // changes for `pure?`/`deterministic?`/`total?` over core code — verified by run, not
        // reasoned — while a core-spelled form inside a `where` is now refused by name.
        //
        // Third instance today of ONE class: a match on a literal STRING, which no exhaustiveness
        // check can see (`axis-violation`'s native decode; `matcher.rs`'s inline LHS `=`; these
        // guards). `holon/CLAUDE.md`: suspect a string comparison before the type system.
        WatAST::List(items, _)
            if axes == [Axis::RetePrimitive]
                && matches!(items.first(), Some(WatAST::Keyword(k, _))
                    if crate::rete::vocabulary::rete_op_for(k).is_none()
                        && matches!(
                            crate::rete::vocabulary::resolve_core_name(k),
                            ":wat::core::cond" | ":wat::core::match" | ":wat::core::fn"
                        )) =>
        {
            let (head, span) = match items.first() {
                Some(WatAST::Keyword(k, s)) => (k.clone(), s.clone()),
                _ => (String::from("<structural form>"), ast.span().clone()),
            };
            Err(AxisViolation::at(span, head, Axis::RetePrimitive))
        }

        // quote / quasiquote / holon-literal sub-forms are DATA — do not recurse into them as calls.
        // Arc 294.b: `:wat::holon::literal` is pure (it captures data, no side-effects).
        WatAST::List(items, _) if matches!(items.first(), Some(WatAST::Keyword(k, _)) if k == ":wat::core::quote" || k == ":wat::core::quasiquote" || k == ":wat::holon::literal") => {
            Ok(())
        }

        // `cond` — clause-aware: (cond (test body…) …). A clause is NOT a call; every element
        // (test AND body forms) is an expression that must satisfy the axis. (cond ≡ chained `if`.)
        //
        // BRIEF-cond-the-first-macro-backed-rete-row.md (2026-08-05) — widened through
        // `resolve_core_name` (THE ONE discriminator, `rete/vocabulary.rs`) exactly as `match`'s
        // and `fn`'s guards were widened (STOP-5: the structural-guard widening is this one
        // indirection, never a duplicated arm), so `:wat::rete::core::cond` is recognised here
        // too. A non-rete head (the entire core corpus) round-trips through `resolve_core_name`
        // unchanged — zero behavior change for anything not in `RETE_OPS`.
        WatAST::List(items, _) if matches!(items.first(), Some(WatAST::Keyword(k, _)) if crate::rete::vocabulary::resolve_core_name(k) == ":wat::core::cond") => {
            for clause in &items[1..] {
                match clause {
                    WatAST::List(parts, _) => {
                        for e in parts {
                            classify_expr(e, axes, sym, seen, closure_seen, ctx)?;
                        }
                    }
                    // malformed clause → deny, naming the malformed clause's own span.
                    other => return Err(AxisViolation::at(other.span().clone(), "<malformed cond clause>", axes[0])),
                }
            }
            refuse_core_structural_on_multi(axes, items)?;
            Ok(())
        }

        // `match` — clause-aware: (match scrut (pattern body…) …). The scrutinee and every arm
        // BODY must satisfy the axis; the PATTERN is structural (destructures/binds, never calls — wat
        // match has no guards). Arc 258.5 — bare match: scrutinee = items[1], arms = items[2..]
        // (the `-> :T` ascription is retired). Skip the pattern (arm element 0), check the body
        // (arm elements 1..).
        //
        // Arc 278 #56 phase 2 — the guard resolves through `resolve_core_name` (THE ONE
        // discriminator, `rete/vocabulary.rs`) so `:wat::rete::core::match` is recognised here
        // too, without a second copy of this arm's body keyed on the rete name (STOP-4: the
        // structural-guard widening is this one indirection, never a duplicated arm). A
        // non-rete head (the entire core corpus) round-trips through `resolve_core_name`
        // unchanged — zero behavior change for anything not in `RETE_OPS`.
        WatAST::List(items, list_span) if matches!(items.first(), Some(WatAST::Keyword(k, _)) if crate::rete::vocabulary::resolve_core_name(k) == ":wat::core::match") => {
            let scrut = items.get(1).ok_or_else(|| {
                AxisViolation::at(list_span.clone(), "<malformed match: no scrutinee>", axes[0])
            })?;
            classify_expr(scrut, axes, sym, seen, closure_seen, ctx)?;
            let arms = items.get(2..).ok_or_else(|| {
                AxisViolation::at(list_span.clone(), "<malformed match: no arms>", axes[0])
            })?;
            for arm in arms {
                match arm {
                    // skip pattern (element 0); check body forms (1..).
                    WatAST::List(parts, _) => {
                        for e in parts.iter().skip(1) {
                            classify_expr(e, axes, sym, seen, closure_seen, ctx)?;
                        }
                    }
                    other => {
                        return Err(AxisViolation::at(
                            other.span().clone(),
                            "<malformed match arm>",
                            axes[0],
                        ))
                    }
                }
            }
            refuse_core_structural_on_multi(axes, items)?;
            Ok(())
        }

        // `:wat::core::fn` lambda literal — NOT a call. Layout: (fn [params…] -> :ret body…).
        // The param vector + return-type are not evaluated; only the BODY forms (after the `-> :ret`
        // ascription) carry effects, so classify exactly those. Mirror the `match`-arm's logic:
        // locate the top-level `->` symbol, then body = items[i+2..] (skip `->` and :ret).
        //
        // Arc 278 (S5, closing #56's leftover) — widened through `resolve_core_name` (THE ONE
        // discriminator, `rete/vocabulary.rs`) exactly as `match`'s guard above was widened, so
        // `:wat::rete::core::fn` is recognised here too, without a second copy of this arm's body
        // keyed on the rete name (STOP-4: one indirection, never a duplicated arm). A non-rete
        // head (the entire core corpus) round-trips through `resolve_core_name` unchanged —
        // zero behavior change for anything not in `RETE_OPS`.
        WatAST::List(items, list_span) if matches!(items.first(), Some(WatAST::Keyword(k, _)) if crate::rete::vocabulary::resolve_core_name(k) == ":wat::core::fn") => {
            match items.iter().position(|it| matches!(it, WatAST::Symbol(s, _) if s.as_str() == "->")) {
                Some(i) => {
                    let body = items.get(i + 2..).ok_or_else(|| {
                        AxisViolation::at(list_span.clone(), "<malformed fn: no body>", axes[0])
                    })?;
                    for e in body {
                        classify_expr(e, axes, sym, seen, closure_seen, ctx)?;
                    }
                    refuse_core_structural_on_multi(axes, items)?;
                    Ok(())
                }
                // malformed fn (no `->`) → deny
                None => Err(AxisViolation::at(list_span.clone(), "<malformed fn: no `->`>", axes[0])),
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
                Some(other) => {
                    return Err(AxisViolation::at(
                        other.span().clone(),
                        "<non-keyword/symbol head>",
                        axes[0],
                    ))
                }
            };
            let at = head_node.map(|h| h.span().clone()).unwrap_or_else(|| ast.span().clone());
            for &axis in axes {
                let mut axis_seen = seen.clone();
                let mut axis_closure_seen = closure_seen.clone();
                head_ok(head, axis, sym, &mut axis_seen, &mut axis_closure_seen, &at, ctx)?;
            }
            for a in &items[1..] {
                classify_expr(a, axes, sym, seen, closure_seen, ctx)?;
            }
            Ok(())
        }

        // Vectors / maps / sets → recurse element-wise.
        WatAST::Vector(elems, _) => {
            for e in elems {
                classify_expr(e, axes, sym, seen, closure_seen, ctx)?;
            }
            Ok(())
        }
        WatAST::Map(pairs, _) => {
            for (k, v) in pairs {
                classify_expr(k, axes, sym, seen, closure_seen, ctx)?;
                classify_expr(v, axes, sym, seen, closure_seen, ctx)?;
            }
            Ok(())
        }
        WatAST::Set(elems, _) => {
            for e in elems {
                classify_expr(e, axes, sym, seen, closure_seen, ctx)?;
            }
            Ok(())
        }
    }
}

/// Classify a named user fn against `axis` by inspecting its body transitively. `seen` detects cycles;
/// a back-edge (fqdn already in `seen`) returns `true` (fixpoint: the cycle adds no new violation).
fn classify_fn(
    fqdn: &str,
    axis: Axis,
    sym: &SymbolTable,
    seen: &mut HashSet<String>,
    closure_seen: &mut HashSet<*const Function>,
    at: &Span,
    ctx: ClassifyCtx,
) -> Result<(), AxisViolation> {
    if seen.contains(fqdn) {
        return Ok(()); // back-edge — no new violation from the recursive call
    }
    seen.insert(fqdn.to_string());

    let func = match sym.get(fqdn) {
        Some(f) => Arc::clone(f),
        None => return Err(AxisViolation::at(at.clone(), fqdn, axis)), // name not registered → deny
    };
    match &func.body {
        // Arc 278 #88 — THE MEMBRANE. Pre-#88 this arm was
        // `classify_expr(body_ast.as_ref(), axis, sym, seen)` — a transitive walk of the
        // callee's body on EVERY reach, for EVERY axis, chased from whatever rule happened to
        // call it. `Function::rete` is `Some(_)` iff this fn was declared
        // `(:wat::rete::core::defn …)` and its body already PROVED, once, at THAT declaration,
        // against all four axes (`apply_rete_defn_contracts`) — so admission here is
        // unconditional on `axis`, the same way the declaration was unconditional on all four.
        // An ordinary `:wat::core::defn` (`rete: None`) is refused ON LAW A: nobody declared the
        // boundary, so nothing is proven — exactly the gap #88 closes
        // (DESIGN-STONE-the-rete-defn.md's "reproduced live": editing one op inside an undeclared
        // helper used to fail naming the RULE, with not one frame naming the helper; refusing here
        // names the HELPER's own fqdn, `fqdn`, directly).
        //
        // ⚠ THE MEMBRANE IS SCOPED TO `RetePrimitive`, AND THE SCOPING IS LOAD-BEARING. An earlier
        // cut of this arm denied an undeclared fn on EVERY axis, reasoning that the declaration
        // proves all four so its absence should deny all four. That inverts wrongly: `Some(_)`
        // may be admitted on any axis (all four WERE proven at the declaration), but `None` means
        // only "undeclared" — it says nothing about purity. `:wat::rete::pure?` /
        // `deterministic?` / `total?` are GENERAL predicates over any expression, not rete-
        // admission tests, so denying them for a missing rete marker made `pure?` answer FALSE
        // for an ordinary, genuinely pure fn. `Axis::RetePrimitive`'s own doc names the rule this
        // violates: it exists as a separate variant precisely because reusing Pure/Deterministic/
        // Total "would make the refusal LIE". So: declared → admitted anywhere; undeclared →
        // refused on law A only, and the other three axes keep their pre-#88 body walk unchanged.
        FunctionBody::Wat(body_ast) => {
            if func.rete.is_some() {
                Ok(())
            } else if matches!(axis, Axis::RetePrimitive) {
                Err(AxisViolation::at(at.clone(), fqdn, axis))
            } else {
                classify_expr(body_ast.as_ref(), std::slice::from_ref(&axis), sym, seen, closure_seen, ctx)
            }
        }
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
                // A NATIVE fn has no body to walk and its name is core-spelled — it is not a
                // rete primitive. This is the arm the core HOFs (`foldl`/`map`/…) take: native AND
                // in `sym.functions`, so they reach here before the admission door. Their rete
                // twins are separate rows and reach the vocabulary door instead.
                Axis::RetePrimitive => false,
            });
            if ok { Ok(()) } else { Err(AxisViolation::at(at.clone(), fqdn, axis)) }
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
        // A NATIVE fn is opaque and core-spelled — not a rete primitive. Same rule as
        // `classify_fn`'s `FunctionBody::Native` arm, which this fn exists to mirror.
        Axis::RetePrimitive => false,
    });
    if ok {
        Ok(())
    } else {
        Err(AxisViolation::at(crate::rust_caller_span!(), path, axis))
    }
}

/// STOP-1's defensive code path has NO wat-surface fixture (see
/// `tests/program/wat_arc278_sigma_fn_purity_gate.rs`'s doc comment) — nothing in the crate
/// constructs a `Function` with `FunctionBody::Native`, so no wat program can drive a sigma fn
/// into this arm. Exercised directly at the Rust level instead, proving the extracted helper
/// itself agrees with `classify_fn`'s `FunctionBody::Native` arm it mirrors.
#[cfg(test)]
mod axis_name_round_trip_tests {
    use super::*;

    /// THE WALL for the class that cost 39 tests: an `Axis` variant that the wat surface can
    /// ENCODE but cannot DECODE.
    ///
    /// ⛔ Why this `match` is the gate and not the `assert`s: the match is over **`Axis` itself**,
    /// so adding a variant makes it non-exhaustive and *the compiler names the new variant right
    /// here* — three lines from `Axis::ALL`, which is the list the decode is built from. That is
    /// the only mechanism available: the thing that broke was a `match` on a **`&str`**, and no
    /// exhaustiveness check can see a string.
    ///
    /// It is deliberately NOT a count assert. A count cannot tell "+1 added, −1 removed" from
    /// "nothing happened", and its failure text cannot name the offender
    /// (`[[feedback_a_gate_freezes_names_never_a_count]]`).
    #[test]
    fn axis_variant_names_round_trip_through_one_door() {
        for axis in Axis::ALL {
            // A new variant lands here as a compile error, naming itself.
            let expected_name = match axis {
                Axis::Pure => "Pure",
                Axis::Deterministic => "Deterministic",
                Axis::Total => "Total",
                Axis::RetePrimitive => "RetePrimitive",
            };
            assert_eq!(axis.variant_name(), expected_name);
            assert_eq!(
                Axis::from_variant_name(expected_name),
                Some(axis),
                "{expected_name} encodes but does not decode — the exact shape that made \
                 axis-violation answer 'expected Pure, Deterministic, or Total' to a \
                 :RetePrimitive it had itself produced",
            );
        }
    }

    /// NON-VACUITY, both directions — without this the test above would still pass if
    /// `from_variant_name` returned `Some(Axis::Pure)` for literally anything.
    #[test]
    fn an_unknown_axis_name_decodes_to_none() {
        assert_eq!(Axis::from_variant_name("Vocabulary"), None);
        assert_eq!(Axis::from_variant_name(""), None);
        assert_eq!(Axis::from_variant_name("pure"), None, "the decode is case-sensitive");
    }

    /// The refusal message is DERIVED from the same list, so it can never again advertise a
    /// smaller set than the decode accepts — which is what made the old failure actively
    /// misleading rather than merely unhelpful.
    #[test]
    fn the_expected_list_names_every_decodable_axis() {
        let msg = Axis::expected_list();
        for axis in Axis::ALL {
            assert!(
                msg.contains(axis.variant_name()),
                "the TypeMismatch text {msg:?} omits {}, which the decode accepts",
                axis.variant_name(),
            );
        }
    }
}

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
        assert!(classify_native_fn(":wat::uuid::v4", Axis::Pure).is_ok());
        assert!(classify_native_fn(":wat::uuid::v4", Axis::Deterministic).is_err());
    }
}

// ─── Public axis classifiers (fresh `seen` per call) — also for stone 6b+ ──────

/// Is `ast` effect-free (no IO/mutation/spawn)? `:wat::uuid::v4` is pure (it does no IO).
///
/// Arc 255 Stone A-2-i — `ctx` names the world explicitly (`ClassifyCtx::Static` at check time /
/// over a quoted form with no live environment; `ClassifyCtx::Runtime(env)` when `env` is a real,
/// evaluated environment a local binding might resolve through). `:wat::rete::pure?` passes its
/// own `env` as `Runtime` — the sole reason this predicate can now see a captured comparator like
/// `sort-by`'s `(fn [a b] (< (keyfn a) (keyfn b)))`.
pub(crate) fn is_pure_expr(ast: &WatAST, sym: &SymbolTable, ctx: ClassifyCtx) -> bool {
    classify_expr(ast, &[Axis::Pure], sym, &mut HashSet::new(), &mut HashSet::new(), ctx).is_ok()
}

/// Is `ast` referentially transparent (same inputs → same output)? `:wat::uuid::v4` is NOT.
/// `ctx` — see [`is_pure_expr`]'s doc.
pub(crate) fn is_deterministic_expr(ast: &WatAST, sym: &SymbolTable, ctx: ClassifyCtx) -> bool {
    classify_expr(ast, &[Axis::Deterministic], sym, &mut HashSet::new(), &mut HashSet::new(), ctx).is_ok()
}

/// Is `ast` domain-total (defined on all its inputs)? ARMED: `compile-condition` consults
/// this as the third fence conjunct. `:wat::i64::/` is NOT (undefined at a zero
/// divisor, and separately at the one input pair that overflows i64). `ctx` — see
/// [`is_pure_expr`]'s doc.
pub(crate) fn is_total_expr(ast: &WatAST, sym: &SymbolTable, ctx: ClassifyCtx) -> bool {
    classify_expr(ast, &[Axis::Total], sym, &mut HashSet::new(), &mut HashSet::new(), ctx).is_ok()
}

/// LAW A — is every head in `ast`'s transitive walk a rete primitive? Armed on the
/// `where` / accumulate / `:then` fences (`compile-condition`); fact-pattern Law A is
/// the freeze wall plus intern `compile_condition_local` (CoreGeneric → none). Same
/// walk as the three predicates above; only the axis differs — a user fn is admitted
/// iff its BODY is, at any depth.
///
/// Arc 255 Stone A-2-i deliberately did NOT extend `:wat::rete::primitive?` to accept a
/// `ClassifyCtx::Runtime` — nothing in this stone consumes that for the `RetePrimitive` axis, so
/// this predicate keeps calling the walk under `ClassifyCtx::Static`, byte-identical to before.
pub(crate) fn is_rete_primitive_expr(ast: &WatAST, sym: &SymbolTable) -> bool {
    classify_expr(ast, &[Axis::RetePrimitive], sym, &mut HashSet::new(), &mut HashSet::new(), ClassifyCtx::Static).is_ok()
}

/// Run the SAME walk `is_pure_expr`/`is_deterministic_expr` use, but keep the violation instead of
/// collapsing it to `false`. `None` ⟺ `ast` satisfies `axis` (agrees with the bool predicates above
/// by construction — same function, same recursion, only the return type differs). Backs the
/// wat-visible `:wat::rete::axis-violation` diagnostic surface. Thin `ClassifyCtx::Static` wrapper
/// over [`find_axis_violation_ctx`] — this signature and its `freeze.rs` call site are unchanged.
pub(crate) fn find_axis_violation(ast: &WatAST, axis: Axis, sym: &SymbolTable) -> Option<AxisViolation> {
    find_axis_violation_ctx(ast, axis, sym, ClassifyCtx::Static)
}

/// Arc 255 Stone A-2-i — the env-carrying sibling `find_axis_violation` keeps beside it: same
/// walk, but the caller NAMES the world (`ClassifyCtx::Static` vs. `Runtime(env)`) rather than
/// getting `Static` by default. First consumed by arc 255 Stone A-2-ii-b, at `sort$native`'s
/// door (`eval_vec_sort_by`, `src/collection/transform.rs`): the comparator's own `closed_env`
/// names the world there — `Runtime(env)` when it has one, `Static` when it does not.
pub(crate) fn find_axis_violation_ctx(
    ast: &WatAST,
    axis: Axis,
    sym: &SymbolTable,
    ctx: ClassifyCtx,
) -> Option<AxisViolation> {
    classify_expr(ast, std::slice::from_ref(&axis), sym, &mut HashSet::new(), &mut HashSet::new(), ctx).err()
}

/// Arc 278 #88 v2 — THE DEFINITION-SITE CHECK's OUTCOME, as a matchable VALUE rather than a
/// raise. Shaped like `SiftRulesResponse` (`wat/query.wat:154-157`'s convention: one good
/// result, N named bad ones, each carrying located structured fields), because
/// THE DEPLOYMENT MODEL (DESIGN-STONE-the-rete-defn.md) rules that rule compilation is
/// runtime-only, from forms arriving over a wire from a host we must never trust to have
/// validated its own input — so a refused declaration must be a RESPONSE a caller can
/// `match`, never something that unwinds across that boundary and takes the service down
/// for every other client (the arc already paid for that mistake once: `28701476`→`b9d61bd6`).
///
/// This is the VALUE SHAPE ONLY — no service, no wire, no transport wraps it here (that is
/// the #7 chaos engine / #17's contract). `apply_rete_defn_contracts` hands this back and
/// leaves the decision of "raise at boot" vs "reply on a wire" to its caller, instead of
/// deciding it here by returning something that IS itself a raise.
pub(crate) enum ReteDefnCheckOutcome {
    /// Every declared rete-defn in this registration batch proved all four axes
    /// and has no call-graph cycle; `Function::rete` has been stamped for each.
    Ok,
    Err(ReteDefnCheckError),
}

/// Pattern A: location on the outer error, kind variants carry no span.
pub(crate) struct ReteDefnCheckError {
    pub span: Span,
    pub kind: ReteDefnCheckErrorKind,
}

pub(crate) enum ReteDefnCheckErrorKind {
    /// The 400-class sibling of `RequestMalformed`: a declared rete-defn's body failed
    /// `axis`, at `head` — located (`span` on the outer error), structured.
    AxisViolation {
        name: String,
        axis: &'static str,
        head: String,
    },
    /// #87 — the body (transitively) calls itself. Not an axis: a cycle is still
    /// pure ∧ det ∧ total ∧ rete. eBPF-shaped static refusal at LOAD.
    Recursive {
        name: String,
        head: String,
    },
}

/// Arc 278 #88 — THE DEFINITION-SITE CHECK. Called from `register_runtime_defs`
/// (`runtime.rs`) — the ONE door both the boot path (`freeze.rs`'s `FrozenWorld::freeze`)
/// and the live-session path (`runtime.rs`'s `eval_form_against_defs`) already call — after
/// `sym.types` is attached (running any earlier would false-positive every helper that reads
/// an aggregate field or calls an aggregate constructor, since `constructor_meta`/
/// `accessor_meta` need `sym.types` to recognize them; `sym.types` is attached in
/// `build_env` well before `register_runtime_defs` ever runs, so that precondition still
/// holds at the new call site).
///
/// `declared` is the set of canonical (`<T,…>`-stripped) names whose SURFACE form was
/// `(:wat::rete::core::defn …)` — collected pre-macro-expansion
/// (`freeze::env::extract_rete_defn_names`), because by the time this runs the head has
/// already been rewritten to plain `:wat::core::defn` and registered through that ORDINARY
/// path: same parse (`crate::function::parse_fn_signature`, unchanged), same registration
/// (`register_defines`'s existing fn-shape-def branch, unchanged), same symbol binding — the
/// design stone's own framing, "does everything `defn` does."
///
/// For each declared name, one `classify_expr` walk over `Axis::ALL` (STOP-1: no second
/// implementation). First failing axis at each call head names the violating head.
/// `find_axis_violation` is the wat `axis-violation` door, not this stamp. A clean pass re-stamps
/// the SAME `Function` with `rete: Some(ReteContract {})`; `classify_fn`'s `Wat` arm (above)
/// consults that marker instead of re-walking — that consultation is the membrane.
pub(crate) fn apply_rete_defn_contracts(
    sym: &mut SymbolTable,
    declared: &std::collections::HashSet<String>,
) -> ReteDefnCheckOutcome {
    for name in declared {
        // A name collected pre-expansion but absent from `sym.functions` means registration
        // refused it for an unrelated reason upstream (reserved prefix, unnamespaced, a
        // collision) — that error already surfaced from `register_defines`; nothing to stamp.
        let Some(func) = sym.get(name).cloned() else {
            continue;
        };
        let body_ast = match &func.body {
            FunctionBody::Wat(ast) => Arc::clone(ast),
            // No `(:wat::rete::core::defn …)` construction path produces a Native body today
            // (see `Function::body`'s doc); kept as a controlled refusal, never a panic, mirroring
            // `check_sigma_fn_contract`'s identical defensive arm (freeze.rs).
            FunctionBody::Native => continue,
        };
        // SEED `seen` WITH THE NAME BEING DECLARED. A self-call inside the very body we are
        // proving would otherwise reach `classify_fn` while this name is still `rete: None`
        // (the stamp happens below, after all four axes pass) and be denied on law A for
        // calling ITSELF — which would LIE ("is not a rete primitive" about a rete-defn).
        // The four axes stay silent on cycles; #87's refusal is the walk AFTER this check.
        //
        // AND SEED IT WITH EVERY OTHER DECLARED NAME, for the same reason one step out.
        // `declared` is a HashSet, so `for name in declared` runs in ARBITRARY, run-varying
        // order. Seeding only `name` leaves a MUTUAL reference order-dependent: `where-nesting`
        // declares `c1` then `c2`, and `c2` calls `c1` — if the loop happens to reach `c2`
        // first, `c1` is not yet stamped and `c2` is refused; reach `c1` first and both pass.
        // A check that answers differently depending on hash iteration order is not a check.
        //
        // Every member of `declared` is being proven in THIS pass, each independently against
        // its own body, so a call from one to another is a back-edge within the declaration
        // group — the identical assumption `classify_fn` already makes for a cycle, widened
        // from self to the group. Soundness is unchanged: nobody is admitted without its own
        // body passing all four axes; only the ORDER of proving stops mattering.
        //
        // One AST walk; first-failing axis at each call head (Pure → Det → Total → Rete).
        let mut seen: HashSet<String> = declared.clone();
        seen.insert(name.clone());
        let mut closure_seen: HashSet<*const Function> = HashSet::new();
        // Definition-time check: no live `Environment` exists yet for a not-yet-invoked rete-defn
        // body — `ClassifyCtx::Static` is the only world this call can honestly claim.
        if let Some(v) =
            classify_expr(body_ast.as_ref(), &Axis::ALL, sym, &mut seen, &mut closure_seen, ClassifyCtx::Static).err()
        {
            return ReteDefnCheckOutcome::Err(ReteDefnCheckError {
                span: v.span,
                kind: ReteDefnCheckErrorKind::AxisViolation {
                    name: name.clone(),
                    axis: v.axis.variant_name(),
                    head: v.head,
                },
            });
        }
        // rune:temperare(simplicity-win) — cycle is a second question (#87 recursion),
        // not a fifth fence axis; merging it into classify_expr would complect them.
        if let Some((head, span)) = rete_defn_cycle(name, body_ast.as_ref(), sym) {
            return ReteDefnCheckOutcome::Err(ReteDefnCheckError {
                span,
                kind: ReteDefnCheckErrorKind::Recursive {
                    name: name.clone(),
                    head,
                },
            });
        }
        let mut declared_func = (*func).clone();
        declared_func.rete = Some(crate::value::ReteContract::default());
        sym.register_function(name.clone(), Arc::new(declared_func));
    }
    ReteDefnCheckOutcome::Ok
}

/// #87 — a rete-defn may not recurse. Gray-node DFS over named `FunctionBody::Wat`
/// callees reachable from `root_name`'s body. A back-edge is a cycle. Natives and
/// rete primitives have no Wat body and are not followed. `pure?` is unchanged:
/// this walk is a LOAD refusal, not a fifth axis.
fn rete_defn_cycle(root_name: &str, body: &WatAST, sym: &SymbolTable) -> Option<(String, Span)> {
    let mut gray: HashSet<String> = HashSet::new();
    gray.insert(root_name.to_string());
    let mut black: HashSet<String> = HashSet::new();
    walk_rete_defn_callees(body, &mut gray, &mut black, sym)
}

fn walk_rete_defn_callees(
    ast: &WatAST,
    gray: &mut HashSet<String>,
    black: &mut HashSet<String>,
    sym: &SymbolTable,
) -> Option<(String, Span)> {
    match ast {
        WatAST::List(items, list_span) => {
            let head = match items.first() {
                Some(WatAST::Keyword(k, _)) => Some(k.as_str()),
                Some(WatAST::Symbol(id, _)) => Some(id.as_str()),
                _ => None,
            };
            if let Some(head) = head {
                if matches!(head, ":wat::core::quote" | ":wat::core::quasiquote" | ":wat::holon::literal") {
                    return None;
                }
                let core = crate::rete::vocabulary::resolve_core_name(head);
                if core == ":wat::core::fn" {
                    if let Some(i) = items
                        .iter()
                        .position(|it| matches!(it, WatAST::Symbol(s, _) if s.as_str() == "->"))
                    {
                        for e in items.get(i + 2..).unwrap_or(&[]) {
                            if let Some(hit) = walk_rete_defn_callees(e, gray, black, sym) {
                                return Some(hit);
                            }
                        }
                    }
                    return None;
                }
                if core == ":wat::core::match" {
                    if let Some(scrut) = items.get(1) {
                        if let Some(hit) = walk_rete_defn_callees(scrut, gray, black, sym) {
                            return Some(hit);
                        }
                    }
                    for arm in items.get(2..).unwrap_or(&[]) {
                        if let WatAST::List(parts, _) = arm {
                            for e in parts.iter().skip(1) {
                                if let Some(hit) = walk_rete_defn_callees(e, gray, black, sym) {
                                    return Some(hit);
                                }
                            }
                        }
                    }
                    return None;
                }
                if let Some(func) = sym.get(head) {
                    if let FunctionBody::Wat(callee_body) = &func.body {
                        if gray.contains(head) {
                            return Some((head.to_string(), list_span.clone()));
                        }
                        if !black.contains(head) {
                            gray.insert(head.to_string());
                            if let Some(hit) =
                                walk_rete_defn_callees(callee_body.as_ref(), gray, black, sym)
                            {
                                return Some(hit);
                            }
                            gray.remove(head);
                            black.insert(head.to_string());
                        }
                    }
                }
            }
            for a in items.iter().skip(1) {
                if let Some(hit) = walk_rete_defn_callees(a, gray, black, sym) {
                    return Some(hit);
                }
            }
            None
        }
        WatAST::Vector(elems, _) | WatAST::Set(elems, _) => {
            for e in elems {
                if let Some(hit) = walk_rete_defn_callees(e, gray, black, sym) {
                    return Some(hit);
                }
            }
            None
        }
        WatAST::Map(pairs, _) => {
            for (k, v) in pairs {
                if let Some(hit) = walk_rete_defn_callees(k, gray, black, sym) {
                    return Some(hit);
                }
                if let Some(hit) = walk_rete_defn_callees(v, gray, black, sym) {
                    return Some(hit);
                }
            }
            None
        }
        _ => None,
    }
}

// ─── WAT surfaces ───────────────────────────────────────────────────────────────
//
// Arc 255 Stone P6-c-W5a — `eval_pure_predicate`/`eval_deterministic_predicate`/
// `eval_total_predicate`/`eval_rete_primitive_predicate` (the `:wat::rete::pure?` /
// `deterministic?` / `total?` / `primitive?` dispatch entry points) and their shared
// `eval_axis_predicate` helper (the hand-rolled `args.len() != 1` arity guard + the
// `WatAST` type-check) are DELETED — moved to `#[wat_intrinsic]` handlers in
// `src/intrinsic/rete.rs`, each taking a typed `expr: &WatAST` leading param (arity 1,
// shim-owned) and calling `is_pure_expr`/`is_deterministic_expr`/`is_total_expr`/
// `is_rete_primitive_expr` (below) directly. `eval_axis_predicate` had exactly these four
// callers (`grep -n "eval_axis_predicate(" src/rete/purity.rs` at pre-image), so nothing
// else goes dead by its removal.

::wat_source_derive::wat_field_names_from!(AXIS_VIOLATION_FIELDS, "wat/rete/compile.wat", ":wat::rete::AxisViolation");
fn axis_violation_names() -> crate::rete::kernel::FieldNames {
    static N: std::sync::OnceLock<crate::rete::kernel::FieldNames> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(AXIS_VIOLATION_FIELDS)).clone()
}

/// `(:wat::rete::axis-violation expr axis) -> (:wat::core::Option :- [wat::rete::AxisViolation])`
///
/// The SAME walk `pure?`/`deterministic?`/`total?`/`primitive?` run, surfacing the
/// violation instead of discarding it: `:wat::core::None` ⟺ `(pure? e)` / `(deterministic? e)` would
/// be `true` for the requested axis; `Some(v)` names the offending head (`v/head`), echoes the axis
/// back (`v/axis`), and carries a `:wat::kernel::Location` at `v/span` (native stubs use
/// `rust_caller_span` so the field is never omitted).
///
/// Builder-ruled (CLOSED-SET RULE, REALIZATIONS.md:2676): the axis argument is the
/// `:wat::rete::Axis` enum (a `defenum` in `wat/rete/compile.wat`), decoded/encoded here directly as a
/// `Value::Enum` — no keyword string map. `pure?`/`deterministic?` are UNCHANGED by this addition
/// (STOP-1) — this is purely additive.
///
/// Arc 255 Stone P6-c-W5c — moved verbatim into `#[wat_intrinsic]` with its real (2) arity
/// declared; the hand-rolled `args.len() != 2` guard this wave retires lived right here.
///
/// **Purity ground:** both args are evaluated by ordinary call-by-value (not itself an effect).
/// `find_axis_violation` → `classify_expr` is the exact same read-only structural walk that
/// `is_pure_expr`/`is_deterministic_expr`/`is_total_expr`/`is_rete_primitive_expr` run (W5a,
/// `Pure`/`Deterministic`) — no `eval_inner`/`apply_function` on `expr`, only a transitive AST
/// walk with a per-call `gray`/`black` `HashSet` (local, dropped on return) for cycle detection.
/// The one `OnceLock` here (`axis_violation_names`) caches a fixed, compile-time-constant
/// field-name table process-wide, the same boilerplate every record constructor in this file uses
/// — infrastructure, not a per-call effect. The `Option<AxisViolation>` returned is freshly built
/// and handed to the caller; nothing outlives the call beyond that one constant cache.
///
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Probe
/// @arg     expr :wat::WatAST the quoted expression form (from `:wat::core::quote`), walked structurally, never evaluated
/// @arg     axis :wat::rete::Axis which of the four fence axes to check (`Pure`/`Deterministic`/`Total`/`RetePrimitive`)
/// @ret     (:wat::core::Option :- [:wat::rete::AxisViolation]) `None` if `expr` satisfies `axis`; `Some(v)` naming the offending head, the axis, and its span otherwise
/// @example (:wat::rete::axis-violation (:wat::core::quote (:wat::rete::i64::> ?c 5)) :wat::rete::Axis::Pure) #=> :None
#[wat_intrinsic(":wat::rete::axis-violation")]
pub(crate) fn eval_axis_violation(
    expr: &WatAST,
    axis: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::rete::axis-violation";
    const AXIS_TYPE: &str = ":wat::rete::Axis";
    let expr_span = expr.span().clone();
    let axis_span = axis.span().clone();
    let val = crate::runtime::eval_inner(expr, env, sym)?.value_owned();
    let ast = match val {
        Value::wat__WatAST(ref a) => (**a).clone(),
        other => {
            return Err(RuntimeError::new(expr_span, RuntimeErrorKind::TypeMismatch {
                op: OP.into(),
                expected: ":wat::WatAST (a quoted expr from :wat::core::quote)",
                got: Box::new(ValueSnapshot::of(&other)),
            })
            .into());
        }
    };
    let axis_val = crate::runtime::eval_inner(axis, env, sym)?.value_owned();
    // ONE DOOR (`Axis::from_variant_name`) — never a second, hand-spelled variant list here.
    // See `Axis::variant_name`'s doc for the 39-test failure the old duplicate decode caused.
    let axis = match &axis_val {
        Value::Enum(ev) if ev.type_path == AXIS_TYPE => Axis::from_variant_name(&ev.variant_name),
        _ => None,
    };
    let Some(axis) = axis else {
        return Err(RuntimeError::new(axis_span, RuntimeErrorKind::TypeMismatch {
            op: OP.into(),
            // Leaked deliberately: the accepted set is DERIVED from `Axis::ALL`, so this message
            // cannot again name fewer variants than the decode accepts.
            expected: Axis::expected_list(),
            got: Box::new(ValueSnapshot::of(&axis_val)),
        })
        .into());
    };
    let out = match find_axis_violation(&ast, axis, sym) {
        None => Value::Option(Arc::new(None)),
        Some(v) => {
            let span_val = crate::runtime::value_from_span(v.span);
            // ONE DOOR, the encode half — same `variant_name` the decode is derived from.
            let axis_variant = v.axis.variant_name();
            let record = Value::Aggregate(Arc::new(AggregateValue::record(
                "wat::rete::AxisViolation".to_string(),
                axis_violation_names(),
                Arc::new(vec![
                    Value::String(Arc::new(v.head)),
                    Value::Enum(Arc::new(EnumValue {
                        type_path: AXIS_TYPE.to_string(),
                        variant_name: axis_variant.to_string(),
                        // Arc 296 G′ — `Axis` is a Unit-only enum (no variant carries a
                        // payload); `fields` is always `vec![]` above.
                        names: crate::runtime::no_field_names(),
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

// ─── The effectful-op classifier (arc 109 Stone the-last-two-map-items) ─────────

/// Prefix-based effectful guess — the pre-arc-255 classifier, kept as a
/// named fallback for verbs not yet carved into the intrinsic registry.
/// Anything under `:wat::kernel::*`, `:wat::io::*`, `:wat::holon::*`, or the
/// eval/load family is rejected in step mode — the consumer falls back to
/// `:wat::eval-ast!` for those sub-forms.
///
/// This is a GUESS about a namespace, not a fact about a body — a
/// registered row's declared purity is a stronger signal (`is_effectful_op`
/// consults it first, and every `:wat::holon::*` verb is registered as of
/// Stone HOME-8, so this prefix never actually fires for one of them today).
/// Kept `pub(crate)` for `src/intrinsic/mod.rs`'s test module (arc 255.1c
/// site 3's census).
///
/// `:wat::holon::*` joined this list at Stone HOME-8: several verbs
/// (`Hologram/put`/`remove`, `OnlineSubspace/new`/`update`,
/// `Reckoner/new-discrete`/`new-continuous`/`observe`/`resolve`/`curve`,
/// `EngramLibrary/new`/`add`/`match-vec`, `Engram/residual`, and the six
/// `eval-*-coincident?` forms) are honestly `@Purity Effectful` — they mutate
/// a native `ThreadOwnedCell`-backed handle via `with_mut`, or (the
/// `eval-*-coincident?` family) evaluate arbitrary embedded wat source.
/// `declared_purity_vs_effectful_by_prefix_census` asserts `Effectful ⇒
/// effectful_by_prefix` for every registered row — leaving `:wat::holon::`
/// off this list would have left that assertion failing not because the
/// verbs are mis-declared, but because the fallback oracle hadn't caught up
/// to a namespace that used to have zero registry presence. Unlike
/// `string.rs`'s `declare-acronyms` (Stone HOME-4), which had the honest
/// escape hatch of a genuinely side-effect-free body at eval time, these
/// verbs do not: reclassifying them `Pure` would be the dishonest fix.
///
/// `:wat::stream::` joined this list at arc 255 Stone P6-c-W2, for the identical reason:
/// `:wat::stream::next` (`src/intrinsic/stream.rs`) is honestly `@Purity Effectful` — forcing
/// a thunk calls `apply_function` on a captured wat closure (or runs a native closure for the
/// lazy `map`/`filter`/`take`/`drop` family), which can run arbitrary code. No escape hatch:
/// `next`'s body genuinely can have a side effect, so `Pure` would be the dishonest fix, the
/// same non-choice `:wat::holon::`'s effectful members faced. `:wat::stream::empty`/`cons`
/// (same wave, same file) stay `Pure` and simply add two more entries to this census's
/// tolerated Pure-declared-under-an-effectful-prefix inventory — the same shape
/// `:wat::config::*` (four rows) already established below.
///
/// `:wat::rete::` joined this list at arc 255 Stone P6-c-W5b, for six verbs that mutate a
/// rete session: `arm-session`/`release-session` (`src/rete/kernel/arm.rs`) take/drop a lease
/// on the thread-local `ARM_TABLE` intern cache; `export`/`import` (`src/rete/export.rs`)
/// touch the same table on a build-and-intern MISS; `eval-insert`/`eval-test`
/// (`src/rete/eval_insert.rs`, `src/rete/eval_test.rs`) can run a caller-supplied expression
/// via `eval_inner`/`apply_function` — arbitrary code this verb has no way to bound, the same
/// shape `:wat::stream::next` is effectful for. All six are honestly `@Purity Effectful`; no
/// escape hatch, `Pure` would be the dishonest fix. ⚠ Unlike `:wat::stream::`'s two tolerated
/// Pure rows, this widening also puts W5a's NINE already-homed PURE `:wat::rete::` verbs
/// (`pure?`/`deterministic?`/`total?`/`primitive?`/`vocabulary-admitted?`/
/// `cond-has-deferred-constraint?`/`alpha-match`/`alpha-match-local`/`alpha-match-under`)
/// under this prefix — legal (the census's surviving assertion is one-directional, `Effectful
/// ⇒ effectful_by_prefix`; `prefix ⇒ Effectful` is a counted census, not a rule), but it is
/// the reason `declared_purity_vs_effectful_by_prefix_census`'s disagreement count rises by
/// about nine at this stone: those nine now disagree (declared Pure, prefix says effectful)
/// the same way `:wat::config::`'s four Pure rows already did.
pub(crate) fn effectful_by_prefix(head: &str) -> bool {
    head.starts_with(":wat::kernel::")
        || head.starts_with(":wat::io::")
        || head.starts_with(":wat::holon::")
        || head.starts_with(":wat::eval-")
        || head.starts_with(":wat::load")
        || head.starts_with(":wat::config::")
        || head.starts_with(":wat::stream::")
        || head.starts_with(":wat::rete::")
}

/// Effectful-op classifier — the registry is the authority (arc 255.1c). A
/// registered row DECLARED its purity from its body; the prefix cannot see
/// inside one. `Pure` and `Preserving` both mean not-effectful, so
/// `matches!(.., Effectful)` is the whole test. Falls back to the prefix
/// guess only for verbs not yet carved into the registry.
pub(crate) fn is_effectful_op(head: &str) -> bool {
    if let Some(e) = crate::intrinsic::registry().lookup_entry(head) {
        return matches!(e.purity, wat_doc::Purity::Effectful);
    }
    effectful_by_prefix(head)
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
/// default: `compile-condition` **panics** on `pure? = false` (`wat/rete/compile.wat`), so a rule using
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
/// step disappears. This gate holds the line while the hand map is v1.
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
        (":wat::std::", Disp::Unreviewed, "arc 255 Stone HOME-9 (2026-08-27) retired the last 14 dispatched verbs that survived arc 109's sweep (math/stat/list) — this scan currently finds NOTHING under this prefix; the rule is kept as insurance against a stray future re-add, not because anything lives here"),
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
        // ── RESTORED 2026-08-20 (255.1c-io-writer). These 48 were removed from this ledger by
        // the carve stones that moved their verbs OUT of `runtime.rs` — `:wat::time::` ×41 by
        // 255.1c-time (`25c1f4521`), `Bytes::` ×2 by home #1, and the reflect/intrinsic rows.
        // That commit recorded it as *"unreviewed debt 214 -> 173"*. IT WAS NOT A REVIEW.
        // The ratchet said "41 verb(s) are no longer unreviewed" because they had left the
        // SCAN's sight, not because anyone ruled on them — and the honest response to a
        // population that shrank was to ask why, not to delete the names that fell out of it.
        // `dispatch_verbs` now scans the `#[wat_intrinsic]` homes too, so they are back in the
        // population and their debt is visible again. Nothing here is classified; 255.3 owns
        // that. [[feedback_a_gate_freezes_names_never_a_count]] — the gate froze the names
        // correctly; the DISPOSITION of its red is what went wrong.
    // Arc 255 Stone 1a-zeta — `:wat::core::ann-form` LEAVES. `intrinsic_meta`'s registry-first
    // consult now answers `Some` for it (the `@Purity Preserving` / `@Determinism Preserving` /
    // `@Totality Preserving` this stone registered — `intrinsic/special/ann_form.rs`), so
    // `dispatch_verbs`'s scan (which still finds `eval_ann_form`'s literal arm RETIRED in
    // `runtime.rs`'s `dispatch_keyword_head_value`, STOP-1 unchanged past the arm deletion the
    // registry-first door itself demanded) classifies it instead of leaving it unreviewed.
    // Leaving the name here after registering would fail this ledger's own STALE check.
    // Arc 255 Stone 1c-a-ii — `:wat::core::Tuple` / `:wat::core::apply` / `:wat::core::conforms?`
    // LEAVE together. `intrinsic_meta`'s registry-first consult now answers `Some` for all three
    // (`@Purity Pure`/`Preserving`, `@Totality Total`/`Partial`/`Preserving` — registered at
    // `src/runtime.rs`: `eval_tuple`, `eval_apply_intrinsic`, `eval_conforms`), so `dispatch_verbs`'s
    // scan (whose literal arms were RETIRED in `dispatch_keyword_head_value` this same stone) no
    // longer finds them dispatched-but-unclassified. `:wat::core::get`/`:wat::core::contains?`
    // (registered the same stone) were never on this ledger to begin with — nothing to remove.
    // Arc 255 Stone 1a-β-ii — `:wat::core::def` LEAVES. `intrinsic_meta`'s registry-first
    // consult (`:473` above) now answers `Some` for it (the `@Purity Unevaluated` /
    // `@Determinism Deterministic` / `@Totality Partial` this stone registered), so
    // `dispatch_verbs`'s scan (which still finds the `":wat::core::def" => Err(...)` refusal
    // arm at `runtime.rs:2132` — that arm is unchanged by this stone, STOP-1) classifies it
    // instead of leaving it unreviewed. Leaving the name here after registering would fail
    // this ledger's own STALE check.
    // ⛔ Arc 255 Stone the-hand-rolled-arms-retire — `:wat::core::defclause` LEAVES, and NOT
    // because it was ruled on. Its literal `runtime.rs` arm was deleted with `def`'s, so
    // `dispatch_verbs`'s scan no longer finds it and this ledger's own STALE check demanded the
    // removal. ⚠ THAT IS A LOSS, RECORDED RATHER THAN ABSORBED: `defclause` has NO
    // `#[wat_special_form]` row, so the replacement guard — keyed on `@Purity Unevaluated` —
    // cannot see it either. Its named `DeclarationInExpressionPosition` refusal on the raw-AST
    // path (the only path that reached the arm; `check.rs`'s resolve pass refuses the literal head
    // before eval otherwise) is GONE until `defclause` is registered.
    //
    // ★ The fix is to register it, not to restore the arm: a single surviving hand-rolled arm is
    // the `const DECLARATION_FORMS` anti-pattern the 2026-06-24 position-class NOTE refused by
    // name, and restoring it would re-create the exact drift that left seven forms saying
    // "unknown function" for months. Tracked in the campaign worklist as a registration.
    //
    // Arc 255 Stone 1c-d — `:wat::core::derive` LEAVES. `intrinsic_meta`'s registry-first
    // consult (`:472` above) now answers `Some` for it (the `@Purity Unevaluated` /
    // `@Determinism Deterministic` / `@Totality Partial` this stone registered —
    // `intrinsic/special/derive_form.rs`), so this ledger's own STALE check (below) demands the
    // removal — classified, not merely no-longer-dispatched. `defclause` and `extend-type` are
    // registered the SAME stone but were never on this ledger to begin with (no dispatch arm
    // for either predates this list — `dispatch_verbs`'s scan never found them here) — nothing
    // to remove for those two.
    // Arc 255 Stone 1c-a-i — `:wat::core::find-last-index` LEAVES. `intrinsic_meta`'s
    // registry-first consult now answers `Some` for it (`@Purity Preserving` /
    // `@Determinism Preserving` / `@Totality Preserving`, registered at
    // `src/collection/transform.rs`), so `dispatch_verbs`'s scan (whose literal arm was
    // RETIRED in `runtime.rs`'s `dispatch_keyword_head_value` this same stone) no longer finds
    // it dispatched-but-unclassified — it is classified, not merely no-longer-dispatched.
    // Arc 255 Stone 1a-gamma-i — `:wat::core::forms` / `:wat::core::macroexpand` /
    // `:wat::core::macroexpand-1` / `:wat::core::quasiquote` / `:wat::core::quote` /
    // `:wat::core::struct->form` LEAVE together. `intrinsic_meta`'s registry-first consult
    // now answers `Some` for all six (the `@Purity`/`@Determinism`/`@Totality` this stone
    // registered — `Pure`/`Deterministic`/`Total` for `quote`/`forms`, `Pure`/`Deterministic`/
    // `Partial` for `struct->form`, `Preserving` across all three for `quasiquote`, `Pure`/
    // `Nondeterministic`/`Partial` for `macroexpand`/`macroexpand-1`), so `dispatch_verbs`'s
    // scan (which still finds each literal arm RETIRED in `runtime.rs`'s
    // `dispatch_keyword_head_value`, STOP-1 unchanged past the arm deletion the registry-first
    // door itself demanded) classifies all six instead of leaving them unreviewed. Leaving
    // any name here after registering would fail this ledger's own STALE check.
    ":wat::core::seqable->stream",
    ":wat::core::subtype?",
    // Arc 255 Stone 1a-ε — `:wat::core::use!` LEAVES. `intrinsic_meta`'s registry-first
    // consult (`:473` above) now answers `Some` for it (the `@Purity Pure` /
    // `@Determinism Deterministic` / `@Totality Total` this stone registered), so
    // `dispatch_verbs`'s scan classifies it instead of leaving it unreviewed. Leaving the name
    // here after registering would fail this ledger's own STALE check.
    //
    // ⛔ CORRECTED at the end of the same stone: this note first said the scan "still finds the
    // `\":wat::core::use!\" => Ok(Value::Unit)` arm at `runtime.rs:2947` — that arm is unchanged
    // by this stone, STOP-5". Both halves went stale within the hour. The brief's STOP-5 said not
    // to touch the eval arms; registering a `role = eval` handler then made
    // `registry_first_door_owns_every_handler_row_no_literal_arm_survives` demand the arm's
    // DELETION by name — the registry-first door answers `use!` now, so the arm could never fire.
    // **The gate outranked the STOP, and the gate was right.** The arm is gone; the no-op lives in
    // `intrinsic/special/use_form.rs`.
    // `:wat::core::show` DELETED from this ledger arc 255 Stone the-seven-that-need-no-
    // extraction: homed into a `#[wat_intrinsic]` handler (`src/runtime.rs`'s `eval_show`)
    // with its full directive block, so `intrinsic_meta` now classifies it from the registry
    // — it is no longer unreviewed. The DESIGN predicted this exact deletion by name (the
    // stone's own "★★ Expect THREE ledgers to move").
    // `:wat::form::matches?` DELETED from this ledger 2026-08-28 (arc 255 Stone P6-c-1):
    // homing it into the intrinsic registry gave it `intrinsic_meta` purity, so it is no longer
    // unreviewed. This gate went RED demanding the deletion — the ratchet shrinking as the debt
    // is paid, exactly as designed. Every verb the P6-c campaign homes will do the same.
    // Arc 255 Stone E-iv — `keyword` gets its home. `to-string`/`from-string` carry forward
    // the SAME open ruling under their OLD spelling (this ledger never classified them
    // either); `to-symbol`/`to-type-form`/`to-type-form-colon` are newly VISIBLE to this scan
    // for the first time — registering all five via `#[wat_intrinsic]` makes `dispatch_verbs`'
    // intrinsic-homes scan see them, where before the three producers lived only in
    // `dispatch_keyword_head`'s producer match (a region this scan never reads). Parked here
    // rather than classified in `intrinsic_meta`: the F5 `is_pure_total` allow-list
    // (`macros/eval.rs`) already treats all five as pure/deterministic, but ruling on THIS
    // axis (RETE-fireability) for a verb nothing forces into a `where` is out of this stone's
    // scope — same restraint as E-iii's refused `RETE_MODULES` entry.
    // Arc 255 Stone HOME-9 — `:wat::std::math::*` moved to `:wat::math::*`. Carries forward
    // the SAME open ruling under the new spelling (this ledger never classified them either);
    // `log` (the seventh old verb) is DELETED, not moved, so it drops out of this ledger
    // rather than being renamed.
    // Arc 255 Stone P6-c-W5a — the nine read-only rete predicates/matchers (the six `?`
    // predicates + the three alpha-matchers) are HOMED (`src/intrinsic/rete.rs`) and
    // CLASSIFIED (`intrinsic_meta`, below) — deleted from this ledger, not carried forward.
    // Arc 255 Stone P6-c-W5b — six more: `arm-session`/`release-session` (`src/rete/kernel/
    // arm.rs`), `export`/`import` (`src/rete/export.rs`), and `eval-insert`/`eval-test`
    // (`src/rete/eval_insert.rs`/`src/rete/eval_test.rs`) are HOMED (`#[wat_intrinsic]`, in
    // place — not relocated to `src/intrinsic/`) and CLASSIFIED `@Purity Effectful` — deleted
    // from this ledger, not carried forward.
    // Arc 255 Stone P6-c-W5c — the four remaining readers are HOMED (`#[wat_intrinsic]`, in
    // place) and CLASSIFIED — deleted from this ledger, not carried forward: `lower`
    // (`src/rete/expr_ir.rs`, `Pure`/`Deterministic` — a static compile pass, no `eval_inner` on
    // user code), `step-payload` (`src/rete/step_payload.rs`, `Pure`/`Deterministic` — reads an
    // already-compiled network structurally), `axis-violation` (`src/rete/purity.rs`, same file,
    // `Pure`/`Deterministic` — the same walk `pure?`/`deterministic?`/`total?`/`primitive?` run),
    // and `collect-rules` (`src/rete/collect.rs`, `Effectful`/`Nondeterministic` — its reflection
    // filter is shape-only (zero-arg + ret-type `Rule`) and does not verify the discovered fn's
    // body came from `defrule`'s always-quoted expansion, so it invokes arbitrary already-defined
    // code via `eval_inner`, unbounded, the same shape `eval-test`/`eval-insert` were ruled
    // `Effectful` for in W5b). The remaining `:wat::rete::` verbs (the firing family — fire-*,
    // insert-*, the $native twins) are unaffected and remain on this ledger under `RULES`'s
    // `:wat::rete::` Unreviewed disposition.
    ":wat::rete::fire-once",
    ":wat::rete::fire-once$native",
    ":wat::rete::fire-rules",
    ":wat::rete::fire-rules$native",
    ":wat::rete::fire-rules-explain",
    ":wat::rete::fire-rules-explain$native",
    ":wat::rete::insert",  // DESIGN-STONE-insert-prime-split — 2-ary native; 3+ is insert-all
    ":wat::rete::insert$native",
    ":wat::rete::insert-all",
    ":wat::rete::insert-all$native",
    // Arc 255 Stone HOME-9 — `:wat::std::list::{zip,window,remove-at}` moved to
    // `:wat::seq::*` (and became Seqable-generic in the same motion — a runtime/check
    // concern, not a purity ruling; they carry forward the SAME open ruling under the new
    // spelling). `map-with-index` (the fourth old verb) is DELETED, not moved, so it drops
    // out of this ledger; its replacement `:wat::core::map-indexed` was already classified
    // (or already unreviewed under its own name) before this stone and is unaffected.
    // Arc 255 Stone HOME-9 — `:wat::std::stat::*` moved to `:wat::stat::*`. Carries forward
    // the SAME open ruling under the new spelling.
    // `:wat::stdlib::sources` STAYS — arc 255 Stone P6-c-W2 (STOP-A) dropped it from that
    // wave's homing: its handler (`crate::io::eval_stdlib_sources`) returns `Result<Value,
    // RuntimeError>`, and `#[wat_intrinsic]` rejects any return type other than
    // `Result<Value, EvalBreak>` — homing it as-written would not compile. Untouched, still
    // dispatched from the giant match, still genuinely unreviewed.
    ":wat::stdlib::sources",
    // `:wat::stream::cons`/`empty`/`next` RULED and CLASSIFIED — arc 255 Stone P6-c-W2
    // (`intrinsic_meta`, above: `cons`/`empty` pure∧det∧total; `next` false on all three —
    // forcing a thunk runs arbitrary user code).
    //
    // Arc 255 Stone 1a-zeta — `:wat::stream::lazy` LEAVES too now. `intrinsic_meta`'s
    // registry-first consult answers `Some` for it (the `@Purity Pure` / `@Determinism
    // Deterministic` / `@Totality Total` this stone registered — `intrinsic/special/
    // stream_lazy.rs`), so `dispatch_verbs`'s scan (which still finds the literal arm RETIRED
    // in `runtime.rs`'s `dispatch_keyword_head_value`, STOP-1 unchanged past the arm deletion
    // the registry-first door itself demanded) classifies it instead of leaving it unreviewed.
    // Leaving the name here after registering would fail this ledger's own STALE check.

        // ── ADDED 2026-08-27 (HOME-12, the AST registry home). These ten were ALWAYS
        // dispatched and ALWAYS unreviewed; the gate simply could not SEE them. Its scan
        // anchors on `dispatch_keyword_head_value` and `dispatch_substrate_impl`
        // (`dispatch_verbs`, below) and has never covered `dispatch_keyword_head` — the
        // `Result<TrackedValue, _>` path where PRODUCERS live. All ten are producers, so all
        // ten sat in the blind spot. HOME-12 registered them as `#[wat_intrinsic]`, the scan's
        // other half, and the gate saw them for the first time and went red.
        //
        // ⚠ THE RED WAS THE GATE WORKING, and it is the mirror of this ledger's own 2026-08-20
        // lesson: carve stones used to REMOVE verbs from the scan's sight and call the smaller
        // number a review. This carve ADDED verbs to its sight. Parking them here is the honest
        // disposition — nobody has ruled on their purity — not a ratchet being loosened.
        //
        // ⛔→✅ CLOSED by STONE meter-2 (the whole-file, shape-based scan): at HOME-12 time, four
        // more arms were invisible for exactly this reason — `:wat::core::write-forms`,
        // `with-children`, `macro-error`, `let`, all living in `dispatch_keyword_head`, one word
        // off the anchored `dispatch_keyword_head_value` — and widening `dispatch_verbs` to a
        // third named anchor was flagged as a separate stone rather than done here. meter-2 did
        // NOT add a third anchor (that would only reload the same defect against a fourth
        // function someday); it replaced the anchor-and-span scan with a whole-file, shape-based
        // one, so all four are visible now regardless of which function they sit in. `let` was
        // already classified (`rete_op_for` admits it as a `RETE_OPS` Form-class row); the other
        // three are RULED in `intrinsic_meta` above (STONE meter-2's block) with citations to
        // `src/edn/render.rs`/`src/runtime.rs`, not parked here.
        //
        // Measured while parking, so the eventual ruling starts from evidence rather than a guess:
        //   read-string   is TOTAL — malformed input returns `ReadOutcome/Malformed`, it does not raise.
        //   fresh-symbol  is NONDETERMINISTIC — `fresh_scope()` is a process-global AtomicU64;
        //                 two calls in ONE process returned `scopes [1069]` then `[1070]`.
        //                 (Two calls in two PROCESSES both return [1069] — an instrument that
        //                 cannot see the defect it is pointed at.)

        // ── ADDED (STONE meter-1, the recursive-walk scan). `dispatch_verbs`'s
        // `#[wat_intrinsic]`-home scan used to `read_dir` only `src/intrinsic` (files plus one
        // subdirectory level); these eleven are homed elsewhere in `src/` (`src/rete/`,
        // `src/rete/kernel/`, and `src/runtime.rs` itself) and were invisible to it — and since
        // homing a verb also deletes its literal dispatch arm, they had left the population
        // entirely rather than becoming ruled. Walking the whole tree recursively makes them
        // visible again, unchanged in every other respect.
        //
        // All eleven already declare `@Purity` and `@Determinism` — each was ruled with a
        // disk-cited reason in arc 255 Stone P6-c (W5a/W5b/W5c) or Stone P6-c-1. What none of
        // them declare is `@Totality`: every one carries `@Totality Unreviewed`, because totality is
        // a separate axis the registry could not hold before stone total-T1. The gate's
        // question is the FENCE's three axes (pure ∧ det ∧ total), not the doc contract's two
        // — so a verb ruled `@Purity Pure` here is still honestly unreviewed for this gate. No
        // `@Purity` value is copied into `intrinsic_meta`; only the ledger records the gap.
        //
        // `:wat::form::matches?`      (`src/runtime.rs`)             @Purity Pure       @Determinism Deterministic — a structural Clara-semantics
        //                             walk over an already-evaluated subject and a never-evaluated pattern; total is unreviewed.
        // `:wat::rete::arm-session`   (`src/rete/kernel/arm.rs`)     @Purity Effectful  @Determinism Deterministic — takes an intern
        //                             lease on the thread-local `ARM_TABLE`; the return value is unchanged, only the table's state moves.
        // `:wat::rete::release-session` (`src/rete/kernel/arm.rs`)   @Purity Effectful  @Determinism Deterministic — drops one intern
        //                             lease on `ARM_TABLE`; same shape as `arm-session`, the opposite direction.
        // `:wat::rete::collect-rules` (`src/rete/collect.rs`)        @Purity Effectful  @Determinism Nondeterministic — the doc's own
        //                             "purity ground" flags that its reflection filter is shape-only (zero-arg, ret-type `Rule`) and
        //                             does not verify the discovered fn's body came from `defrule`'s always-quoted expansion, so it
        //                             invokes arbitrary already-defined code via `eval_inner`, unbounded.
        // `:wat::rete::eval-insert`   (`src/rete/eval_insert.rs`)    @Purity Effectful  @Determinism Nondeterministic — falls through
        //                             to `eval_rhs_expr`, which runs `eval_inner` then `apply_function` on a caller-supplied `List`
        //                             this verb has no way to bound.
        // `:wat::rete::eval-test`     (`src/rete/eval_test.rs`)      @Purity Effectful  @Determinism Nondeterministic — evaluates a
        //                             caller-supplied expression in a fresh child `Environment` via `eval_inner`; the compile-time
        //                             fence that bounds `where`/`:test` clauses does not apply when this verb is called directly.
        // `:wat::rete::export`        (`src/rete/export.rs`)         @Purity Effectful  @Determinism Deterministic — packs the
        //                             session via `rete_arm_get_or_build`, which on a MISS builds AND interns into the same
        //                             thread-local `ARM_TABLE` as `arm-session`.
        // `:wat::rete::import`        (`src/rete/export.rs`)         @Purity Effectful  @Determinism Deterministic — interns the
        //                             reconstructed network into `ARM_TABLE` unconditionally; a session dropped without
        //                             `release-session` leaks until thread end.
        // `:wat::rete::lower`         (`src/rete/expr_ir.rs`)        @Purity Pure       @Determinism Deterministic — a pure static
        //                             compile pass (`lower()`) that reads the symbol table and never calls `eval_inner`/
        //                             `apply_function`; the built `Program` is discarded, nothing outlives the call.
        // `:wat::rete::step-payload`  (`src/rete/step_payload.rs`)   @Purity Pure       @Determinism Deterministic — a read-only
        //                             structural walk over an already-compiled network, the same shape as the W5a axis predicates;
        //                             its two `OnceLock`s cache fixed, compile-time-constant tables, not per-call state.
        // `:wat::rete::axis-violation` (`src/rete/purity.rs`)        @Purity Pure       @Determinism Deterministic — the same
        //                             read-only `classify_expr` walk `is_pure_expr`/`is_deterministic_expr`/`is_total_expr` run,
        //                             never evaluating `expr`.

        // ── ADDED (STONE meter-2, the dispatch scan reads the whole file, both arm shapes).
        // `dispatch_verbs`'s literal-scan used to run only between two named anchors
        // (`dispatch_keyword_head_value`/`dispatch_substrate_impl`) and knew only one arm shape
        // (`":wat::…" =>`). Widening it to the whole file, and to also recognize the
        // keyword-guard shape (`WatAST::Keyword(k, _) if k == "…" =>`), made FIVE more verbs
        // visible that were dispatched all along: `:wat::core::Some`/`Ok`/`Err` (the
        // Option/Result constructor producers, keyword-guard shape in `eval_list`) plus the
        // two below. THREE of those five — `Some`/`Ok`/`Err` — RETIRED from this list arc 255
        // Stone A-2-ii-b-1: homed as `#[wat_intrinsic]`s (`src/intrinsic/option.rs`,
        // `src/intrinsic/result.rs`), ruled `Pure ∧ Deterministic ∧ Total`, and their
        // keyword-guard arms retired from `eval_list` (the line numbers this comment used to
        // cite no longer name that code). The remaining two are unrelated declaration-door
        // siblings, not touched by that stone:
        //
        // `:wat::core::defalias` (`parse_defalias_form`, `src/runtime.rs:2894`) and
        // `:wat::core::extend-type` (`register_stdlib_runtime_defs`/`register_runtime_defs_form`,
        // `src/runtime.rs:1302`/`2852`) — declaration-door siblings of `:wat::core::def`/`fn`/
        // `derive`, already parked above under the identical open question (registration-time
        // forms that mutate the symbol table rather than compute a value). Parked for the same
        // reason as those three, not ruled, for consistency with how they are already treated.
            ];

    /// Pull every verb the runtime dispatches, from BOTH doors: every literal or keyword-guard
    /// match arm anywhere in `runtime.rs` keyed on a wat FQDN, and every `#[wat_intrinsic]`-
    /// registered name anywhere under `src/` (below).
    ///
    /// STONE meter-2: this used to scan only the text BETWEEN two named anchors
    /// (`dispatch_keyword_head_value`/`dispatch_substrate_impl`), so an arm living in any other
    /// function was invisible — including `dispatch_keyword_head`, one word off the anchored
    /// name, and `resolve_verify_payload`, anchored nowhere
    /// (`DESIGN-STONE-meter-2-the-dispatch-half-sees-the-whole-file.md`) — and it knew only one
    /// arm SHAPE (`":wat::…" =>`), missing the keyword-guard shape
    /// (`WatAST::Keyword(k, _) if k == "…" =>`) `eval_list`'s Option/Result producers use,
    /// wherever it might live. THE CONTRACT: the population is defined by SHAPE — any match arm
    /// anywhere in the file whose pattern is (or ends in) a `:`-prefixed string literal — never
    /// by the name of the function it happens to sit in. Anchoring on names is the defect
    /// meter-1 and meter-2 both fix; adding the missing names as more anchors would only reload
    /// it (`DESIGN-STONE-meter-2`'s disqualified alternative).
    ///
    /// A plain substring search over 33k lines also catches a FQDN inside a comment, a doc
    /// `@example`, an error message, or a `matches!`/`if` guard that never reaches a `=>` — so a
    /// candidate counts only if walking FORWARD from its closing quote, through nothing but
    /// whitespace, `|` (an or-pattern separator), `(`/`)` (an or-pattern's grouping), or another
    /// quoted string (another alternative in the same or-pattern), reaches `=>` before anything
    /// else. That single rule is exactly the shape of a real match arm's pattern —
    /// `"a" | "b" => …`, `x @ ("a" | "b") => …`, a bare `"a" => …`, or the keyword-guard
    /// `Keyword(k, _) if k == "a" => …` — and exactly what a `matches!(...)` call, an `if` guard,
    /// or a function-call argument never do (each hits a `)` then `;`/`{`/`.` instead, never
    /// `=>`, so they fail the walk and are silently excluded — no `matches!`-detection needed).
    ///
    /// Two shapes it still cannot tell apart from a genuine dispatch arm this way, named and
    /// EXCLUDED below rather than parked as if they were open rulings (the brief's own
    /// instruction: "not a verb" is disposed by excluding it from the scan's shape, never by a
    /// row pretending it is one):
    ///
    ///   - `:undefined` (`src/runtime.rs:9480`) — not a call head at all; a REQUIRED POSITIONAL
    ///     MARKER argument a rete fallback op checks on its own already-received arg list, never
    ///     a head anyone dispatches `(:undefined …)` on.
    ///   - `:rust::` (`src/runtime.rs:6289`) — not a per-verb arm; `other if
    ///     other.starts_with(":rust::") =>` is a NAMESPACE-PREFIX routing guard into the separate
    ///     `rust_deps` registry (dispatches whichever `:rust::*` symbol is actually present at
    ///     runtime), the same shape as the `RETE_PREFIX`/`is_effectful_op` prefix checks
    ///     elsewhere in this file — never itself a dispatchable verb name.
    ///   - `:wat::core::None` (`src/runtime.rs:16015`) — this occurrence is `try_match_pattern`
    ///     recognizing a PATTERN-CLAUSE head inside `:wat::core::match`'s own implementation (a
    ///     pattern literal `:None` denotes "match `Option::None`"), not dispatching a call. Its
    ///     genuine expression-position evaluation is `src/runtime.rs:5045`'s
    ///     `if k == ":None" || k == ":wat::core::None"` — an `if`, not a match arm, so it sits
    ///     outside this stone's two shapes and was never a scream either way.
    ///
    /// Test code is excluded wholesale: everything from the top-level `mod tests {` to EOF is one
    /// `#[cfg(test)]` block (verified: no production code follows it in this file), so the scan
    /// stops there — a `Value::Enum(ev) if ev.type_path == ":wat::holon::CosineOutcome" => …`
    /// inside a test helper reaches `=>` exactly like a real arm (it matches an ALREADY-PRODUCED
    /// value's type tag, not a call being dispatched) and would otherwise need the same per-name
    /// exclusion as the three above. Smaller `#[cfg(test)] mod { … }` blocks earlier in the file
    /// are NOT separately excluded (no brace-tracking here) — verified empty of any FQDN-shaped
    /// literal as of this stone, so today's measurement is unaffected, but a future one added
    /// inside such a block would need a fresh look, not a name appended to the list above.
    fn dispatch_verbs(src: &str) -> Vec<String> {
        // Not a dispatch arm — see the doc comment above for why each is excluded by name rather
        // than by a `KNOWN_UNREVIEWED` row pretending it is a verb.
        const NOT_A_DISPATCH_ARM: &[&str] = &[":undefined", ":rust::", ":wat::core::None"];

        // Strip whole-line `//`/`///`/`//!` comments and everything from the top-level test
        // module onward (see doc comment), replacing excluded text with blanks so every
        // remaining character keeps its original position.
        let mut clean = String::with_capacity(src.len());
        let mut in_tests = false;
        for line in src.lines() {
            let trimmed = line.trim_start();
            if trimmed == "mod tests {" {
                in_tests = true;
            }
            if !in_tests && !trimmed.starts_with("//") {
                clean.push_str(line);
            }
            clean.push('\n');
        }
        let chars: Vec<char> = clean.chars().collect();
        let n = chars.len();

        // Does walking forward from `i` reach `=>` through nothing but whitespace / `|` / `(` /
        // `)` / another quoted string? See the doc comment above — this IS "a real match arm's
        // pattern", by shape, wherever in the file it lives.
        fn reaches_fat_arrow(chars: &[char], mut i: usize, n: usize) -> bool {
            let mut steps = 0usize;
            loop {
                steps += 1;
                if steps > 4000 || i >= n {
                    return false;
                }
                let c = chars[i];
                if c.is_whitespace() || c == '|' || c == '(' || c == ')' {
                    i += 1;
                    continue;
                }
                if c == '"' {
                    let mut j = i + 1;
                    loop {
                        if j >= n {
                            return false;
                        }
                        if chars[j] == '"' {
                            j += 1;
                            break;
                        }
                        if chars[j] == '\\' {
                            j += 2;
                            continue;
                        }
                        j += 1;
                    }
                    i = j;
                    continue;
                }
                return c == '=' && i + 1 < n && chars[i + 1] == '>';
            }
        }

        let mut out = Vec::new();
        let mut i = 0;
        while i < n {
            if chars[i] == '"' {
                let mut j = i + 1;
                let mut content = String::new();
                let mut closed = false;
                while j < n {
                    if chars[j] == '"' {
                        closed = true;
                        break;
                    }
                    if chars[j] == '\\' {
                        j += 2;
                        continue;
                    }
                    content.push(chars[j]);
                    j += 1;
                }
                if closed {
                    if content.starts_with(':')
                        && !NOT_A_DISPATCH_ARM.contains(&content.as_str())
                        && reaches_fat_arrow(&chars, j + 1, n)
                    {
                        out.push(content);
                    }
                    i = j + 1;
                } else {
                    i = j;
                }
            } else {
                i += 1;
            }
        }
        // ⚠ ARC 255'S CARVE DRAINS THIS SCAN. Every home carved out of `runtime.rs`'s literal
        // dispatch removes verbs from the only population this scan could see — and a shrinking
        // population makes a COMPLETENESS gate report better every stone while measuring less.
        // `:wat::io::` alone moved 23 verbs and took the count 423 → 400, one below the
        // non-vacuity floor below, which is what surfaced this. A verb dispatched through the
        // registry is still DISPATCHED; it just no longer appears as a literal arm. So the
        // population is the UNION — literal arms plus every `#[wat_intrinsic]`-registered name.
        // Read as text, like the arms above, so this stays one mechanism and not two.
        //
        // STONE meter-1: this used to be a `read_dir` over `src/intrinsic` alone — files plus
        // exactly one subdirectory level — so a `#[wat_intrinsic]` homed anywhere else (e.g.
        // `src/rete/`) was invisible to the union, and since homing also deletes the verb's
        // literal dispatch arm, such a verb left the population entirely rather than becoming
        // ruled. Walk the WHOLE `src/` tree, recursively, so a home cannot hide from the meter
        // regardless of depth or directory.
        fn walk_intrinsic_homes(dir: std::path::PathBuf, out: &mut Vec<String>) {
            let Ok(rd) = std::fs::read_dir(&dir) else {
                return;
            };
            for entry in rd.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    walk_intrinsic_homes(p, out);
                    continue;
                }
                if p.extension().is_some_and(|e| e == "rs") {
                    let Ok(text) = std::fs::read_to_string(&p) else {
                        continue;
                    };
                    for line in text.lines() {
                        let t = line.trim_start();
                        if let Some(rest) = t.strip_prefix("#[wat_intrinsic(\"") {
                            if let Some(j) = rest.find('"') {
                                out.push(rest[..j].to_string());
                            }
                        }
                    }
                }
            }
        }
        walk_intrinsic_homes(
            std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")),
            &mut out,
        );
        out.sort();
        out.dedup();
        out
    }

    #[test]
    fn every_dispatched_verb_is_classified_or_disposed() {
        // ⚠ SCOPE, and arc 109's decomposition tested it: this reads ONLY `src/runtime.rs`,
        // so the population is "verbs the EVALUATOR dispatches", not "every match arm mentioning a
        // verb keyword". The declare stone moved `:wat::core::defalias` / `extend-type`'s arms into
        // `src/declare/register.rs` and this gate correctly reported them as leaving the population
        // — they are DECLARATION heads handled at load time, never evaluated as expressions
        // (`check.rs` returns early for both: "declaration forms, not value-producing expressions").
        //
        // ⛔ Widening the scan to all of `src/` was TRIED and is WRONG: it takes the dispatched
        // count 543 -> 693 and the unreviewed worklist 32 -> 170 by sweeping in `check.rs`'s
        // inference arms and `freeze/env.rs`'s declaration matches — consumers of verb names, not
        // dispatch. Narrow blinds the gate; wide captures the wrong population.
        // `[[feedback_a_predicate_can_be_wrong_in_both_directions]]`
        //
        // ⬜ OPEN, recorded not ruled: this anchor is a FILE, and arc 109 will keep moving dispatch
        // out of it. See `NOTE-the-completeness-gate-is-anchored-to-a-file-the-campaign-is-emptying`.
        let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/runtime.rs"))
            .expect(
                "runtime.rs must be readable — it holds the verbs still dispatched \
                 literally; the `#[wat_intrinsic]` homes hold the rest (arc 255)",
            );
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
        // rune:perspicere(read-once) — completeness print grouping; alias would be a mumble.
        let mut by_ns: std::collections::BTreeMap<String, Vec<&String>> = Default::default();
        for v in &unreviewed {
            let ns = v.rsplit_once("::").map(|(a, _)| a.to_string()).unwrap_or_else(|| v.clone());
            by_ns.entry(ns).or_default().push(v);
        }
        // rune:perspicere(read-once) — completeness print rows; alias would be a mumble.
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
