//! Arc 278 Stone 6a — the rete condition fence: four orthogonal classifiers,
//! `pure?` + `deterministic?` + `total?` + `primitive?`.
//!
//! A rete condition (a `where`/`:test` predicate, an accumulator fn) must be a **deterministic,
//! effect-free function of the facts**. Those are two INDEPENDENT properties:
//!
//! - **pure** — effect-free: no IO/mutation/spawn (seed: the negation of `is_effectful_op`).
//! - **deterministic** — referentially transparent: same inputs → same output (no randomness/clock).
//!
//! They are genuinely orthogonal. `:wat::core::Uuid/v4` does no IO and mutates nothing → it is PURE,
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
//! time a poisoned token reaches it. `is_total_expr`/`eval_total_predicate` mirror the two
//! siblings exactly (same walk, same `OpMeta` shape, same default-deny). **`compile-condition`
//! consults `total?`** as the third conjunct of the four-axis fence (pure ∧ det ∧ total ∧ rete).
//! Partial core ops enter rete only as `OpClass::Fallback` + a mandatory `:undefined`.

use crate::ast::WatAST;
use crate::runtime::{
    EvalBreak, Environment, FunctionBody, RuntimeError, RuntimeErrorKind, SymbolTable, Value,
    ValueSnapshot,
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
/// `ast->*`, `with-children`, `write-forms`); `Option/expect` / `Result/expect` (total but they
/// raise); and the generic sequence verbs (`range`, `take`, `drop`, `rest`, `last`, `assoc`,
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
    if head.starts_with(":wat::string::") || head.starts_with(":wat::core::regex::") {
        let total = matches!(
            head,
            ":wat::string::length" | ":wat::string::trim" | ":wat::string::to-lowercase"
        );
        return Some(OpMeta { pure: true, deterministic: true, total });
    }
    // The whole `:wat::edn::` namespace is pure data transforms — parse/serialize/navigate
    // (read, read-foreign, write, write-pretty, write-json, write-json-natural,
    // ForeignRecord/get, ForeignRecord/class, ForeignVariant/variant, ForeignVariant/enum-class,
    // ForeignVariant/fields), no IO, no entropy. Root-level by namespace, not a per-verb
    // hand-list — the next foreign verb slips past a hand-list.
    // `total`: DEFAULT-DENY, then carve the verbs measured total by reading their
    // implementation (the string:: prefix pattern). `read` still raises on
    // malformed input (domain-fault). `read-foreign` returns ReadForeignOutcome
    // (Malformed on junk, never a raise — read-json's contract). `ForeignRecord/get`
    // returns Option (miss is None — HashMap/get's contract). `ForeignRecord/class`
    // always returns a String for a well-typed ForeignRecord (type mismatch is
    // the checker's concern, not this axis).
    if head.starts_with(":wat::edn::") {
        let total = matches!(
            head,
            ":wat::edn::read-foreign"
                | ":wat::edn::ForeignRecord/get"
                | ":wat::edn::ForeignRecord/class"
        );
        return Some(OpMeta { pure: true, deterministic: true, total });
    }
    // `:wat::core::aggregate-new` / `:wat::core::kwargs-construct` (BRIEF-construction-inside-a-
    // fn.md) — the two verbs a record/struct's macro-expanded kwargs/positional construction
    // lowers to (`wat/Record.wat:183-190`, `wat/core.wat:1780-1788` — defstruct's OWN bare-name
    // companion macro emits the identical `kwargs-construct` call a record's does).
    //
    // PURE: construction is assignment — it evaluates already-supplied field VALUES and binds
    // them into a new container; it opens nothing, reads no ambient state, mutates nothing
    // (`construct_aggregate`, runtime.rs:15519-15592, and `build_holon_hologram`'s structural
    // `to_holon_inner` fold, runtime.rs:15426-15461 — no IO, no RNG on either path). This holds
    // regardless of the target's `Nature`: a Struct MAY hold a live resource, but resource
    // ACQUISITION is a property of how a value was obtained, not of the assignment that later
    // carries it, and any acquisition (e.g. `:wat::io::IOReader/open-file`) is caught independently
    // at THAT op's own head by this same walk (`classify_expr` recurses into every field-value
    // argument on the same axis). A pure aggregate can never smuggle one in either way:
    // `validate_aggregate_containment` (check.rs:12511-12573) is a post-registration freeze-time
    // pass that REJECTS STARTUP for any `Nature::Record`/`HolonRecord` type declaring an impure
    // field (`TypeErrorKind::ImpureFieldInPureAggregate`) — a pure-nature aggregate cannot even be
    // registered with a resource-shaped field, let alone constructed holding one. Matches this
    // file's own existing precedent for container ops (`PersistentVector/conj`, `HashMap/assoc`
    // above): the ACT of binding a value into a structure is pure independent of what runtime type
    // that value happens to be.
    //
    // DETERMINISTIC: true, unconditionally — no ambient/random dependency on either path.
    //
    // TOTAL: true — BOTH named gaps this classification originally found are now CLOSED, not
    // merely unarmed-and-therefore-moot. Recorded so a future reader does not have to re-derive
    // why this holds:
    //
    //   (a) was `aggregate-new`-only — a CHECKER hole. `infer_kwargs_construct_check`
    //       (check.rs:11597-11727) closed unknown-field/bad-arity/retired-positional-shape for
    //       `kwargs-construct` by delegating to the prime ctor's own checked call, but
    //       `infer_aggregate_new_check` never unified the supplied positional values against the
    //       declared field count, so a wrong arity passed `--check` and only raised at runtime
    //       (`construct_aggregate`, runtime.rs:15560-15568). CLOSED (check.rs:11516-11607,
    //       `infer_aggregate_new_check`) — but NOT by mirroring kwargs-construct's synthetic-call
    //       approach verbatim: `aggregate-new` is invoked in exactly one place architecturally,
    //       `:T'`'s own generated body (`register_aggregate_methods`, runtime.rs:1510-1559 mints
    //       EVERY nature's ctor as `(:wat::core::aggregate-new :T field-syms…)`), so checking it
    //       is always checking a definition against its OWN signature, not a call site. Routing it
    //       through a synthetic re-call (as first attempted) minted a FRESH instantiation of the
    //       type's own generics and broke every generic self-constructing type in the stdlib
    //       (`Bound<S,R>`, `Launched<S,R,Sh,Lu>`, `Cache::Entry<K,V>`, …); the fix instead unifies
    //       each field value against the scheme's RAW (un-instantiated) `params` and returns its
    //       raw `ret` unchanged — see that function's own doc for the full account.
    //   (b) BOTH verbs, for `Nature::HolonRecord` — a FREEZE-TIME-closeable hole, not a checker
    //       one: `bundle_capacity_verdict` (runtime.rs, `pub(crate)`) checks a type's declared
    //       field count against `ctx.capacity`, and BOTH are freeze-time constants per TYPE (not
    //       per call or per instance), so a program that clears it can never reach the runtime
    //       raise for that type again. CLOSED by `freeze::validate_holon_record_capacity`
    //       (freeze.rs), a new post-registration pass mirroring `validate_aggregate_containment`'s
    //       own timing, called from `FrozenWorld::freeze` right after `EncodingCtx` resolves
    //       `dim_count` — rejecting STARTUP (`TypeErrorKind::HolonRecordCapacityExceeded`) for any
    //       over-budget HolonRecord before any rule ever compiles. The runtime check in
    //       `build_holon_hologram` is now an unreachable backstop, kept as defense in depth rather
    //       than removed (a program that reaches `fire-rules` at all has, by definition, already
    //       cleared freeze).
    //
    // `total?` is ARMED at both `compile-condition` and `then-item-fence`. Recorded as
    // `true` because it genuinely is, with both closures on the board.
    if head == ":wat::core::aggregate-new" || head == ":wat::core::kwargs-construct" {
        return Some(OpMeta { pure: true, deterministic: true, total: true });
    }
    // Arc 109 β-ii-c — `type-params-used-in` is a structural SEARCH over an AST: it reads the
    // node, allocates nothing observable, touches no world state, and returns a subset of its own
    // first argument in the order given. Pure ∧ deterministic ∧ total, and RULED here rather than
    // parked in `KNOWN_UNREVIEWED` — the gate's own remedy says parking "is the LAST resort and is
    // only honest for a verb whose ruling is genuinely open", and this one's is not. It is the
    // first `#[wat_intrinsic]` verb to be classified rather than left to 255.3.
    if head == ":wat::core::type-params-used-in" {
        return Some(OpMeta { pure: true, deterministic: true, total: true });
    }
    // Arc 109 stone (`BRIEF-STONE-type-equal-the-missing-door.md`) — `type-equal?` reads two AST
    // nodes, parses each once via `parse_type_node` (the one door that reads all four type
    // surfaces), and compares the resulting `TypeExpr`s (`PartialEq, Eq`-derived). It allocates
    // nothing observable, touches no world state, and returns a bool — same category as
    // `type-params-used-in` immediately above, and RULED here rather than parked in
    // `KNOWN_UNREVIEWED`: the gate's own remedy says parking "is the LAST resort and is only
    // honest for a verb whose ruling is genuinely open", and this one's is not.
    if head == ":wat::core::type-equal?" {
        return Some(OpMeta { pure: true, deterministic: true, total: true });
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
            // Stone 118.B4-0 — `nth` promoted from a wat `defclause` (which carried no purity
            // ruling of its own; `intrinsic_meta` only judges DISPATCHED verbs) to a Rust
            // intrinsic, which does. Same ruling as its siblings immediately above: it reads an
            // already-evaluated collection + an already-evaluated i64 index, performs no IO, no
            // entropy, no mutation — pure ∧ deterministic. NOT added to the `total` list below:
            // like `first`/`second`/`third`, it raises on out-of-range (verified `eval_nth`,
            // runtime.rs) — a genuinely partial function, exactly the reason this axis exists.
            | ":wat::core::nth"
            // Stone 118.B5 — `stream->vec`/`stream->pvec` promoted from wat `defn` (also
            // unruled here; same reason `nth`'s comment gives — `intrinsic_meta` only judges
            // DISPATCHED verbs, and a wat `defn` is not one) to Rust intrinsics, which are.
            // Discovered by going red on `rete::purity::completeness_gate::
            // every_dispatched_verb_is_classified_or_disposed` — a SEPARATE gate from
            // `is_pure_total` (`macros/eval.rs`), no link between them. Same ruling as `nth`
            // immediately above: each reads an already-evaluated receiver + an
            // already-evaluated Stream and realizes it one cell at a time
            // (`eval_stream_to_vec`/`eval_stream_to_pvec`, `collection/transform.rs`) — no IO,
            // no entropy, no mutation of anything the caller can observe — pure ∧
            // deterministic. NOT added to the `total` list below: the walk forces the
            // producer, and the producer can raise (a `lazy-seq` body is arbitrary user code) —
            // genuinely partial, same axis this exists to catch.
            | ":wat::core::stream->vec"
            | ":wat::core::stream->pvec"
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
            | ":wat::core::map"
            | ":wat::core::mapv"
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
            // (`wat/rete/compile.wat`), so `(:wat::rete::where (:wat::core::i64::> ?bytes 10000))` and
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
            // per-type-equality-restored (2026-08-05) — `i64::=`/`i64::not=`/`f64::=`/
            // `f64::not=`, restored beside their ordering twins above (237.8d's cut
            // reversed). Same shape: a value operation over already-evaluated
            // arguments, no IO, no entropy, no mutation, same inputs -> same output.
            | ":wat::core::i64::=" | ":wat::core::i64::not="
            | ":wat::core::f64::=" | ":wat::core::f64::not="
            // Per-Type integer division family (`i64::/` was already here; its siblings were not).
            | ":wat::core::i64::mod" | ":wat::core::i64::quot" | ":wat::core::i64::rem"
            // f64 numeric readers/roundings — total functions of their argument.
            | ":wat::core::f64::round" | ":wat::core::f64::clamp"
            | ":wat::core::f64::max-of" | ":wat::core::f64::min-of"
            | ":wat::core::f64::to-i64" | ":wat::core::f64::to-string"
            // The `String/` family — ENTIRELY absent. Note `:wat::string::` (lowercase, a
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
            // can use directly; `cosine`/`dot` are measurements, so at the RETE surface they carry
            // a mandatory `:undefined` fallback (DESIGN-STONE-the-vsa-seam-opens.md, 2026-08-05,
            // ruled by the builder) rather than handing back their outcome enum unwrapped —
            // `(:wat::rete::core::f64::> (:wat::rete::holon::cosine ?a ?b :undefined 0.0) 0.9)` now
            // composes, where before BOTH halves were unclassified. (This comment previously named
            // a stale motivating expression that could not type-check, since `cosine` returns
            // `CosineOutcome`, not a bare f64 — the exact guarded-`0.0`-as-confident-no-match
            // fabrication the cosine outcome wall exists to prevent.)
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
    //   `i64::to-string` `f64::to-string` `bool::to-string` — scalar→String conversions with no
    //     domain restriction (verified against each `eval_*_to_string` implementation).
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
            // BRIEF-the-f64-surface-is-a-stub.md Part B (2026-08-05) — generic `:wat::core::<`
            // REMOVED. It is a false-true: `eval_compare` (`runtime.rs:5191`) returns
            // `RuntimeErrorKind::TypeMismatch` when `values_compare` yields `None` (the
            // incomparable-operands domain hole `DESIGN-STONE-where-admits-only-rete-ops.md`
            // names as the whole reason per-type comparison exists), and its three siblings
            // `>`/`>=`/`<=` were never marked true — this was the odd one out.
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
            // BRIEF-one-naming-rule-then-first-nth-to-string.md (2026-08-05) — `i64::to-string` /
            // `f64::to-string` / `bool::to-string`: verified by reading `eval_i64_to_string`
            // (`n.to_string()`) / `eval_f64_to_string` (`format!("{}", f)`, defined for NaN/±Inf/
            // -0.0 too) / `eval_bool_to_string` (`if b {"true"} else {"false"}`) — each converts a
            // well-typed scalar to a String with no domain restriction whatsoever, same reasoning
            // as `i64::to-f64` immediately above. `bool::to-string` was previously listed only in
            // the pure∧det block above, NOT here — the brief that asked for these rete rows named
            // it "already in the total list", which this file's own text did not support; grounded
            // and promoted here rather than trusted.
            | ":wat::core::i64::to-string"
            | ":wat::core::f64::to-string"
            | ":wat::core::bool::to-string"
            // per-type-equality-restored (2026-08-05) — `i64::=`/`i64::not=`: an
            // equality compare over i64 never raises, same class as the ordering
            // family immediately above.
            | ":wat::core::i64::="
            | ":wat::core::i64::not="
            | ":wat::core::f64::>"
            // BRIEF-the-f64-surface-is-a-stub.md Part A (2026-08-05) — `f64::<`/`f64::<=`/
            // `f64::>=` ADDED beside `f64::>`. Same warrant: each is a comparison whose OUTPUT
            // is a bool, never itself the undefined value, and `eval_f64_compare` is
            // NaN-correct (`NaN > 1.0` is `false`, not a raise) — there is no input on which
            // any of the four fails to produce an ordinary bool. #52's own STOP-3 ("do not
            // widen the audit past entries already `true`") swept false-trues and never
            // revisited entries already `false`; these three were the mirror image it missed.
            | ":wat::core::f64::<"
            | ":wat::core::f64::<="
            | ":wat::core::f64::>="
            // per-type-equality-restored (2026-08-05) — `f64::=`/`f64::not=`: a
            // comparison, not arithmetic — `eval_f64_compare` returns a `bool` for any
            // two f64 inputs including NaN/±Inf (never raises), the same reasoning
            // `f64::>` (kept total) already uses immediately above.
            | ":wat::core::f64::="
            | ":wat::core::f64::not="
            | ":wat::core::PersistentVector/length"
            | ":wat::core::PersistentVector/contains?"
            | ":wat::core::PersistentVector/get"
            | ":wat::core::String/concat"
            | ":wat::core::String/starts-with?"
            | ":wat::core::String/ends-with?"
            | ":wat::core::String/contains?"
            | ":wat::core::String/empty?"
            | ":wat::core::foldl"
            // ★ THE FOUR HOF SIBLINGS, added 2026-08-05 (task #80) — `foldl` stood here ALONE for
            // three days and its four siblings did not, which was an inconsistency inside ONE
            // family, not a judgement. The old reason is on the record and it was the corpus
            // fallacy: *"extremely likely total... but are NOT included — no `where` row in the
            // corpus uses them... Flagged, not classified."* Absence of a caller is not evidence of
            // partiality — `[[feedback_optimize_for_the_expressivity_surface_not_the_corpus]]`.
            //
            // GROUNDED, not assumed. A combinator's totality is CONDITIONAL on its fn-argument, and
            // the walk resolves that conditionality itself — proven by run on `foldl`, which
            // already carried `total: true`:
            //
            //   (total? '(foldl (fn [a b] (rete i64::+ a b :undefined 0)) 0 xs))  -> TRUE
            //   (total? '(foldl (fn [a b] (core i64::/ a b))              0 xs))  -> FALSE
            //
            // `classify_expr` enters the typed fn body and finds the partial op. So `total: true`
            // on the HEAD means exactly what `pure: true`/`deterministic: true` already mean for
            // these four — "the combinator itself adds no partiality" — and those two columns took
            // the conditional-TRUE reading from the start. One row, one convention.
            //
            // Arc 118.B6b: `foldr` retired from this arm (and from `pure_det` above) — it was
            // `reverse`+`foldl` wearing a name borrowed from Haskell, where the verb is distinct
            // only because it is LAZY, a property strict wat cannot have. Its right-fold
            // replacement, `(reduce f init (reverse coll))`, is still covered here via `reduce`.
            | ":wat::core::map" | ":wat::core::mapv" | ":wat::core::filter" | ":wat::core::reduce"
            // ── BRIEF-total-column-honest.md Direction 2 (2026-08-02) — the VSA seam ───────────
            //
            // `:wat::holon::presence?` — TRUE. `eval_algebra_presence_q` (`runtime.rs:18623`)
            //     takes both args through `require_holon` (HolonAST only — a raw `Vector` is
            //     rejected as a `TypeMismatch`, the ordinary "type checker's concern" exclusion this
            //     axis already uses elsewhere), then encodes BOTH at the same ambient `d`
            //     (`program_dim` → one `enc`, `runtime.rs:18646-18649`) — so there is no code path by
            //     which its two vectors can disagree in dimension; unlike its three siblings below it
            //     never reaches `pair_values_to_vectors`. Its only float op is `cosine >
            //     enc.presence_floor(sym)` — a comparison, total for the same reason `f64::>` is
            //     (returns `bool`, never raises, never itself NaN/Inf) — so even the guarded `0.0`
            //     `Similarity::cosine` can return does not threaten totality here: whatever `cosine`
            //     returns, `>` against it is defined. `presence?` was ALREADY total before the strike
            //     below and its path is UNCHANGED by it (STOP-3 — no diff in `eval_algebra_presence_q`).
            //
            // ── BRIEF-cosine-outcome-wall.md (2026-08-03) — cosine/dot/coincident? join presence? ──
            //
            // `:wat::holon::coincident?`, `:wat::holon::cosine`, `:wat::holon::dot` were each left
            // `false` above (T1/Direction-2 audit) for the SAME reason: all three route through the
            // shared `pair_values_to_vectors` guard (`runtime.rs`), which used to RAISE
            // `RuntimeErrorKind::TypeMismatch` on a dimension-mismatched Vector pair — a raise is not
            // total by this axis's own definition (an ordinary value on every input), full stop,
            // regardless of what each verb's own arithmetic can or cannot do beyond that shared gate.
            //
            // The cosine outcome wall retired that raise: the guard now returns the mismatch as a
            // `PairedVectors::DimensionMismatch` fact instead of unwinding, and each caller decides
            // what to do with it — per the design stone's ruled law (`DESIGN-STONE-where-admits-only-
            // rete-ops.md`, "RULED 2026-08-02 — THE MEASUREMENT IS FULL; THE PREDICATE IS EXACT"):
            //
            //   `:wat::holon::coincident?` — now TRUE. A dimension mismatch answers `Value::bool(false)`
            //     (a PREDICATE absorbs its own undefined case — a documented total contract, not an
            //     IEEE accident); every other path returns an ordinary `bool` as before. No raise
            //     remains on any input.
            //
            //   `:wat::holon::cosine` — now TRUE. It is a MEASUREMENT, so per the same ruling it may
            //     NOT absorb its own undefined case into a value drawn from its own range — a dimension
            //     mismatch and a zero-magnitude operand (the guarded `0.0` `Similarity::cosine` used to
            //     return, which reads as "orthogonal, unrelated" in cosine's own codomain — a
            //     fabrication, proven reachable and indistinguishable from genuine unrelatedness by
            //     `wat-scripts/scratch-pad/probe-zero-magnitude-reachable.wat`) both become named
            //     `:wat::holon::CosineOutcome` variants (`Similarity`/`Degenerate`/`DimensionMismatch`)
            //     instead. An enum construction never raises and is always a well-typed value — total.
            //
            //   `:wat::holon::dot` — now TRUE. Its arithmetic still cannot overflow (`Vector.data:
            //     Vec<i8>`, bounded by `d × 127²`, unreachable at real dimensions — the family's one
            //     open question, now closed: `d ≈ 10³⁰⁴` to reach ±Inf) and it needs no `Degenerate`
            //     case (a zero-magnitude operand dots to an HONEST `0.0` — no division happens). The
            //     one thing that made it partial was the same shared-guard raise `coincident?` had;
            //     with that retired, `dot` returns `:wat::holon::DotOutcome`
            //     (`Computed`/`DimensionMismatch`) on every input.
            //
            // `:wat::holon::coincident-explain` and `:wat::holon::presence?` are UNCHANGED and
            // deliberately NOT added here: `coincident-explain`'s fixed `CoincidentExplanation` struct
            // return shape has no field able to carry a mismatch honestly, so it re-raises on that one
            // hole exactly as the guard itself used to (STOP-5 — its return shape was not touched, so
            // its totality claim is not touched either); `presence?` never reached the guard in the
            // first place (see its own paragraph above).
            | ":wat::holon::presence?"
            | ":wat::holon::coincident?"
            | ":wat::holon::cosine"
            | ":wat::holon::dot"
    );

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

/// Does `head` satisfy `axis`? Data constructors and field accessors are recognized first
/// (pure-by-declaration); then user fns recurse transitively; intrinsics consult
/// `intrinsic_meta`; unknown heads default-deny.
fn head_ok(head: &str, axis: Axis, sym: &SymbolTable, seen: &mut HashSet<String>, at: &Span) -> Result<(), AxisViolation> {
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
    if sym.has_function(head) {
        return classify_fn(head, axis, sym, seen, at);
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
    match axis {
        // Pure: effectful namespaces are an explicit deny; otherwise the metadata must declare pure.
        Axis::Pure => {
            if crate::runtime::is_effectful_op(head) {
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
fn classify_expr(ast: &WatAST, axes: &[Axis], sym: &SymbolTable, seen: &mut HashSet<String>) -> Result<(), AxisViolation> {
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
                classify_expr(arg, axes, sym, seen)?;
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
                            classify_expr(e, axes, sym, seen)?;
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
            classify_expr(scrut, axes, sym, seen)?;
            let arms = items.get(2..).ok_or_else(|| {
                AxisViolation::at(list_span.clone(), "<malformed match: no arms>", axes[0])
            })?;
            for arm in arms {
                match arm {
                    // skip pattern (element 0); check body forms (1..).
                    WatAST::List(parts, _) => {
                        for e in parts.iter().skip(1) {
                            classify_expr(e, axes, sym, seen)?;
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
                        classify_expr(e, axes, sym, seen)?;
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
                head_ok(head, axis, sym, &mut axis_seen, &at)?;
            }
            for a in &items[1..] {
                classify_expr(a, axes, sym, seen)?;
            }
            Ok(())
        }

        // Vectors / maps / sets → recurse element-wise.
        WatAST::Vector(elems, _) => {
            for e in elems {
                classify_expr(e, axes, sym, seen)?;
            }
            Ok(())
        }
        WatAST::Map(pairs, _) => {
            for (k, v) in pairs {
                classify_expr(k, axes, sym, seen)?;
                classify_expr(v, axes, sym, seen)?;
            }
            Ok(())
        }
        WatAST::Set(elems, _) => {
            for e in elems {
                classify_expr(e, axes, sym, seen)?;
            }
            Ok(())
        }
    }
}

/// Classify a named user fn against `axis` by inspecting its body transitively. `seen` detects cycles;
/// a back-edge (fqdn already in `seen`) returns `true` (fixpoint: the cycle adds no new violation).
fn classify_fn(fqdn: &str, axis: Axis, sym: &SymbolTable, seen: &mut HashSet<String>, at: &Span) -> Result<(), AxisViolation> {
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
                classify_expr(body_ast.as_ref(), std::slice::from_ref(&axis), sym, seen)
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
        assert!(classify_native_fn(":wat::core::Uuid/v4", Axis::Pure).is_ok());
        assert!(classify_native_fn(":wat::core::Uuid/v4", Axis::Deterministic).is_err());
    }
}

// ─── Public axis classifiers (fresh `seen` per call) — also for stone 6b+ ──────

/// Is `ast` effect-free (no IO/mutation/spawn)? `:wat::core::Uuid/v4` is pure (it does no IO).
pub(crate) fn is_pure_expr(ast: &WatAST, sym: &SymbolTable) -> bool {
    classify_expr(ast, &[Axis::Pure], sym, &mut HashSet::new()).is_ok()
}

/// Is `ast` referentially transparent (same inputs → same output)? `:wat::core::Uuid/v4` is NOT.
pub(crate) fn is_deterministic_expr(ast: &WatAST, sym: &SymbolTable) -> bool {
    classify_expr(ast, &[Axis::Deterministic], sym, &mut HashSet::new()).is_ok()
}

/// Is `ast` domain-total (defined on all its inputs)? ARMED: `compile-condition` consults
/// this as the third fence conjunct. `:wat::core::i64::/` is NOT (undefined at a zero
/// divisor, and separately at the one input pair that overflows i64).
pub(crate) fn is_total_expr(ast: &WatAST, sym: &SymbolTable) -> bool {
    classify_expr(ast, &[Axis::Total], sym, &mut HashSet::new()).is_ok()
}

/// LAW A — is every head in `ast`'s transitive walk a rete primitive? Armed on the
/// `where` / accumulate / `:then` fences (`compile-condition`); fact-pattern Law A is
/// the freeze wall plus intern `compile_condition_local` (CoreGeneric → none). Same
/// walk as the three predicates above; only the axis differs — a user fn is admitted
/// iff its BODY is, at any depth.
pub(crate) fn is_rete_primitive_expr(ast: &WatAST, sym: &SymbolTable) -> bool {
    classify_expr(ast, &[Axis::RetePrimitive], sym, &mut HashSet::new()).is_ok()
}

/// Run the SAME walk `is_pure_expr`/`is_deterministic_expr` use, but keep the violation instead of
/// collapsing it to `false`. `None` ⟺ `ast` satisfies `axis` (agrees with the bool predicates above
/// by construction — same function, same recursion, only the return type differs). Backs the
/// wat-visible `:wat::rete::axis-violation` diagnostic surface.
pub(crate) fn find_axis_violation(ast: &WatAST, axis: Axis, sym: &SymbolTable) -> Option<AxisViolation> {
    classify_expr(ast, std::slice::from_ref(&axis), sym, &mut HashSet::new()).err()
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
        if let Some(v) = classify_expr(body_ast.as_ref(), &Axis::ALL, sym, &mut seen).err() {
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
/// ARMED: `compile-condition` consults this as the third fence conjunct.
pub(crate) fn eval_total_predicate(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    eval_axis_predicate(":wat::rete::total?", is_total_expr, args, list_span, env, sym)
}

/// `(:wat::rete::primitive? <quoted-expr>) -> :bool` — is the expression composed ONLY of rete
/// primitives (law A)? The verb is `primitive?` rather than `rete-primitive?` because the
/// namespace already says rete, exactly as `pure?` is not `rete-pure?`.
pub(crate) fn eval_rete_primitive_predicate(
    args: &[WatAST],
    list_span: &Span,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    eval_axis_predicate(":wat::rete::primitive?", is_rete_primitive_expr, args, list_span, env, sym)
}

::wat_source_derive::wat_field_names_from!(AXIS_VIOLATION_FIELDS, "wat/rete/compile.wat", ":wat::rete::AxisViolation");
fn axis_violation_names() -> crate::rete::kernel::FieldNames {
    static N: std::sync::OnceLock<crate::rete::kernel::FieldNames> = std::sync::OnceLock::new();
    N.get_or_init(|| crate::value::value::names_arc_from_static(AXIS_VIOLATION_FIELDS)).clone()
}

/// `(:wat::rete::axis-violation <quoted-expr> <axis: :wat::rete::Axis>) ->
/// (:wat::core::Option :- [wat::rete::AxisViolation])`
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
    // ONE DOOR (`Axis::from_variant_name`) — never a second, hand-spelled variant list here.
    // See `Axis::variant_name`'s doc for the 39-test failure the old duplicate decode caused.
    let axis = match &axis_val {
        Value::Enum(ev) if ev.type_path == AXIS_TYPE => Axis::from_variant_name(&ev.variant_name),
        _ => None,
    };
    let Some(axis) = axis else {
        return Err(RuntimeError::new(args[1].span().clone(), RuntimeErrorKind::TypeMismatch {
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
        ":wat::core::Bytes::from-hex",
        ":wat::core::Bytes::to-hex",
        ":wat::core::render-doc",
        ":wat::core::show-source",
        ":wat::intrinsic::examples",
        ":wat::intrinsic::variadic-args-measurement",
        ":wat::intrinsic::yields-witness",
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
    ":wat::core::Option/expect",
    ":wat::core::Option/try",
    ":wat::core::Record/assoc",
    ":wat::core::Record/field-at",
    ":wat::core::Record/same-data?",
    ":wat::core::Result/expect",
    ":wat::core::Result/try",
    ":wat::core::Tuple",
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
    ":wat::rete::alpha-match-local",
    ":wat::rete::alpha-match-under",
    ":wat::rete::arm-session",
    ":wat::rete::release-session",
    ":wat::rete::cond-has-deferred-constraint?",
    ":wat::rete::axis-violation",
    ":wat::rete::collect-rules",
    ":wat::rete::export",
    ":wat::rete::import",
    ":wat::rete::deterministic?",
    ":wat::rete::eval-insert",
    ":wat::rete::eval-test",
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
    ":wat::rete::lower",
    ":wat::rete::primitive?",
    ":wat::rete::pure?",
    ":wat::rete::step-payload",
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
    // Arc 118.11a — mint next/NextOutcome. Same open question as its three siblings just
    // above (RULES: ":wat::stream::" is Disp::Unreviewed, "laziness — a Stream's purity is
    // its producer's") — `next` forces the SAME cell `first`/`rest` already force, so it
    // inherits exactly their unreviewed status, not a fresh one. Ruling purity is out of
    // scope for this stone (additive: mint the verb, change nothing else).
    ":wat::stream::next",
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
        // ⚠ ARC 255'S CARVE DRAINS THIS SCAN. Every home carved out of `runtime.rs`'s literal
        // dispatch removes verbs from the only population this scan could see — and a shrinking
        // population makes a COMPLETENESS gate report better every stone while measuring less.
        // `:wat::io::` alone moved 23 verbs and took the count 423 → 400, one below the
        // non-vacuity floor below, which is what surfaced this. A verb dispatched through the
        // registry is still DISPATCHED; it just no longer appears as a literal arm. So the
        // population is the UNION — literal arms plus every `#[wat_intrinsic]`-registered name.
        // Read as text, like the arms above, so this stays one mechanism and not two.
        for entry in std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/src/intrinsic"))
            .into_iter()
            .flatten()
            .flatten()
        {
            let mut files = Vec::new();
            if entry.path().is_dir() {
                files.extend(
                    std::fs::read_dir(entry.path())
                        .into_iter()
                        .flatten()
                        .flatten()
                        .map(|e| e.path()),
                );
            } else {
                files.push(entry.path());
            }
            let rs = files
                .into_iter()
                .filter(|f| f.extension().is_some_and(|e| e == "rs"));
            for f in rs {
                let Ok(text) = std::fs::read_to_string(&f) else {
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
        out.sort();
        out.dedup();
        out
    }

    #[test]
    fn every_dispatched_verb_is_classified_or_disposed() {
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
