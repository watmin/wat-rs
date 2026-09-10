//! **THE DISCONFIRMING PROBE — an `accumulate` condition's `:from` inner is not type-checked. At
//! all.**
//!
//! One predicate, two positions. `string::=` over an i64-bound field:
//!
//! ```text
//! inside an `accumulate`'s `:from` inner   ->  ACCEPTED SILENTLY. Starts up clean, rc=0
//! the same predicate written inline        ->  ConstraintTypeMismatch, rc=3
//! ```
//!
//! The finding this re-derives is
//! `docs/arc/2026/06/278-rules-engine/the-position-axis-was-chosen-not-derived/FINDING-the-position-axis-was-chosen-not-derived.md`
//! (its coverage table, which survives the amendment) and
//! `docs/arc/2026/06/278-rules-engine/the-position-axis-was-chosen-not-derived/DESIGN.md` (the
//! contract: `:from`'s inner gets the SAME full validation `not`/`exists` already get —
//! `validate_when_entry` recursion, not a bespoke clause-type-check).
//!
//! ## ★ The invariant this file pins
//!
//! > A predicate inside an `accumulate` condition's `:from` inner is type-checked exactly as the
//! > same predicate written inline in a plain condition.
//!
//! ## The site
//!
//! `src/rete/validate/mod.rs:296` — its within-condition twin is one match arm above it (`Not`/
//! `Exists`, which already recurse via `validate_when_entry`), and the top-level twin is
//! `validate_typed_clauses`'s own call site:
//!
//! ```text
//! // Design call 3 — accumulate's `:from` inner gets fact-type-HEAD validation only;
//! // its own clauses and the acc-form's reducer body are out of scope.
//! ReteClauseShape::Accumulate { from, .. } => {
//!     validate_fact_type_head_only(from, rule_name, types, errors);
//! }
//! ```
//!
//! ## ⛔⛔ WHY THE POSITIVE CORPUS IS THE LOAD-BEARING HALF, NOT THE CHECK
//!
//! `:wat::rete::accumulate` has ZERO uses in the entire `.wat` corpus (see DESIGN.md's grep). A
//! cure that over-rejects legal `accumulate` rules REDS NOTHING on the standing floor, because
//! nothing else uses the form — the surface would narrow silently. So
//! [`legal_accumulate_still_compiles`] (the compile-only guard, beside
//! `probe_arc278_accumulate_from_types.wat`) and the native-fire tests below are not decoration:
//! together they are the only thing standing between this cure and a form that quietly stops
//! working for every future caller. D10's header names this exact failure on the `:then` side —
//! *"a cure that refuses every operand it cannot type passes every refusal probe and still stops
//! a corpus of legal rules from compiling."*
//!
//! ## ⭐ STOP-2's positive control — accumulate compiles and fires TODAY, before this strike
//!
//! [`plain_bind_compiles_and_fires`], [`typed_constraint_compiles_and_fires`], and
//! [`earlier_bound_join_var_compiles_and_fires`] exercise the NATIVE `fire-rules` engine (not the
//! wat oracle) end to end: `defrule` with an `accumulate` condition, real fact inserts, a real
//! fire, a real query, a real derived count. None of the three touches the widened validation
//! scope (each is legal both before and after this strike's source change) — they exist to answer
//! the brief's STOP-2 question in the affirmative: `accumulate` is not merely declared, it is
//! live, and a legal rule using it compiles and fires today.
//!
//! [`not_knowable_operand_still_compiles`] is compile-only, deliberately, matching its ancestor:
//! `probe_arc278_fence_interior_types.rs`'s `legal_fences_still_compile` also never fires the
//! identical `i64::+ ?v 1 :undefined 0` construction — its claim is about the TYPE CHECKER
//! (does not refuse a not-knowable operand), not about the runtime behavior of a deliberately
//! malformed nested call, which neither ancestor nor this file asserts anything about.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// THE GAP, closed by this strike. `check_constraint_types` (mod.rs:296, via the recursed
/// `validate_when_entry`) now walks an `accumulate`'s `:from` inner and refuses this shape exactly
/// as the inline twin below is refused.
#[test]
fn from_inner_type_error_is_refused() {
    let result = startup_from_file("tests/rete/probe_arc278_accumulate_from_types_from.wat.bad");
    assert!(
        result.is_err(),
        "`string::=` over an i64-bound field inside an `accumulate`'s `:from` inner must be \
         refused at rule-compile time — the identical predicate written inline is a \
         ConstraintTypeMismatch."
    );
}

/// THE CONTROL that gives the gap its meaning: the same predicate, inline, refused today.
#[test]
fn inline_twin_is_refused() {
    let result = startup_from_file("tests/rete/probe_arc278_accumulate_from_types_inline.wat.bad");
    assert!(
        result.is_err(),
        "`string::=` over an i64-bound field written INLINE in a condition is a \
         ConstraintTypeMismatch at HEAD. If this ever passes, the predicate is not ill-typed and \
         the `:from`-inner gap is measuring nothing."
    );
}

/// THE OVER-REJECTION GUARD — four legal `accumulate` rules, including one whose operand is NOT
/// knowable. Compile-only (mirrors `legal_fences_still_compile`); see the module doc for why row 4
/// is not fired.
#[test]
fn legal_accumulate_still_compiles() {
    let result = startup_beside(file!());
    assert!(
        result.is_ok(),
        "well-typed `accumulate` `:from` inners — a plain bind, a well-typed inline constraint, \
         an earlier-bound join var, and a fence whose operand is a computed form the checker \
         cannot type — must keep compiling. A cure that refuses what it cannot type passes both \
         `.wat.bad` refusal rows above and breaks the corpus; got: {:?}",
        result.err()
    );
}

/// A world whose rule counts `Reading`s at a `Station`'s location, optionally gated by a
/// `Threshold` fact, filtered by an inner `accumulate` predicate, and gated on `(where <gate>)`.
// rune:lint(no-inlined-wat) — world parameterized by runtime acc/gate strings — cartesian matrix of combinations cannot be pre-extracted
fn world(with_threshold: bool, acc: &str, gate: &str) -> String {
    let threshold_defrecord = if with_threshold {
        "(:wat::core::defrecord :w3::Threshold [min <- :wat::core::i64])\n"
    } else {
        ""
    };
    let threshold_when = if with_threshold { "            (:w3::Threshold (?min <- :min))\n" } else { "" };
    format!(
        "(:wat::core::defrecord :w3::Station [location <- :wat::core::String])\n\
         (:wat::core::defrecord :w3::Reading [location <- :wat::core::String  value <- :wat::core::i64])\n\
         {threshold_defrecord}\
         (:wat::core::defrecord :w3::Busy    [location <- :wat::core::String  n <- :wat::core::i64])\n\
         \n\
         (:wat::rete::defrule :w3::busy\n\
           :when\n\
           [(:w3::Station (?loc <- :location))\n\
{threshold_when}\
            {acc}\n\
            (:wat::rete::where {gate})]\n\
           :then\n\
           [(:w3::Busy :location ?loc :n ?n)])\n\
         \n\
         (:wat::rete::defquery :w3::q-Busy\n\
           :params []\n\
           :when [(:w3::Busy)])"
    )
}

/// Insert a Station(Oslo) + optional Threshold(min) + the given Readings, fire NATIVELY
/// (`fire-rules`, not the oracle — that differential is already 8-b's job), count derived Busy.
fn busy_count(with_threshold: bool, acc: &str, gate: &str, min: i64, readings: &[(&str, i64)]) -> Result<i64, String> {
    let reading_inserts: String = readings
        .iter()
        .map(|(loc, v)| format!("             session (:wat::core::match (:wat::rete::insert session (:w3::Reading :location \"{loc}\" :value {v})) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))\n"))
        .collect();
    let threshold_insert = if with_threshold {
        format!("             session (:wat::core::match (:wat::rete::insert session (:w3::Threshold :min {min})) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))\n")
    } else {
        String::new()
    };
    let run = format!(
        "(:wat::core::length\n\
          (:wat::core::let\n\
            [rules   (:wat::rete::collect-rules :w3)\n\
             session (:wat::core::match (:wat::rete::compile-all rules (:wat::core::PersistentVector (:w3::q-Busy))) ((:wat::rete::CompileOutcome::Compiled __session) __session) ((:wat::rete::CompileOutcome::MayNotTerminate __rule __ft) (:wat::kernel::assertion-failed! \"compile: the rule set may not terminate\" :wat::core::None :wat::core::None)))\n\
             session (:wat::core::match (:wat::rete::insert session (:w3::Station :location \"Oslo\")) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __ilimit __iused __icount) (:wat::kernel::assertion-failed! \"insert: session memory ceiling exceeded while staging\" :wat::core::None :wat::core::None)))\n\
{threshold_insert}\
{reading_inserts}\
             fired   (:wat::core::match (:wat::rete::fire-rules session) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! \"fire-rules: session memory ceiling exceeded\" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! \"fire-rules: fixpoint round cap exceeded\" :wat::core::None :wat::core::None)))]\n\
            (:wat::rete::query fired (:w3::q-Busy))))"
    );
    let world_src = world(with_threshold, acc, gate);
    let w = startup_from_source(&world_src, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {e:?}"))?;
    let ast = wat::parse_one!(&run).map_err(|e| format!("parse: {e:?}"))?;
    match eval_in_frozen(&ast, &w, &Environment::new()).map_err(|e| format!("eval: {e:?}"))?.value_owned() {
        Value::i64(n) => Ok(n),
        other => Err(format!("expected i64; got {other:?}")),
    }
}

/// row 1 — a PLAIN BIND `:from` inner, native fire, exact count.
#[test]
fn plain_bind_compiles_and_fires() {
    let acc = "(?n <- (:wat::rete::acc::count) :from (:w3::Reading (?loc <- :location)))";
    let n = busy_count(false, acc, "(:wat::rete::core::i64::= ?n 3)", 0, &[("Oslo", 1), ("Oslo", 2), ("Oslo", 3)])
        .expect("plain-bind accumulate must compile and fire");
    assert_eq!(n, 1, "3 Oslo readings, gate = 3 -> fires once");
}

/// row 2 — a WELL-TYPED inline constraint inside `:from`'s inner filters correctly.
///
/// ⚠ Uses `acc::sum ?v`, not `acc::count`. Driven in `wat-scripts/scratch-pad/arc278-accfrom/`:
/// `acc::count` with a SECOND bind (`?v <- :value`) present in `:from`'s inner but unconsumed by
/// the acc-form derives a wrong count (3 real readings -> `n=1`), reproducibly, independent of any
/// constraint. That is a native-engine accumulate defect (`src/rete/kernel/fire/acc.rs` /
/// `pass/accumulate.rs`), NOT a validator question — DESIGN.md excludes the reducer body and the
/// engine outright, so it is reported, not fixed, here. `acc::sum ?v` with the same two binds is
/// unaffected (confirmed: 10+20+30=60 unfiltered, matches `differential_sum`), so it is what this
/// fixture uses to keep the CLAIM about the validator, not about an unrelated engine gap.
#[test]
fn typed_constraint_compiles_and_fires() {
    let acc = "(?n <- (:wat::rete::acc::sum ?v) :from (:w3::Reading (?loc <- :location) (?v <- :value) (:wat::rete::core::i64::> ?v 0)))";
    // Only the two positive readings should be summed; the non-positive one is filtered out.
    let n = busy_count(false, acc, "(:wat::rete::core::i64::= ?n 12)", 0, &[("Oslo", 5), ("Oslo", -1), ("Oslo", 7)])
        .expect("typed-constraint accumulate must compile and fire");
    assert_eq!(n, 1, "5 + 7 = 12 (of 3 readings, `?v > 0` filters out -1), gate = 12 -> fires once");
}

/// row 3 — a JOIN VARIABLE bound in an EARLIER condition (`?min` from `:w3::Threshold`), read
/// freely inside `:from`'s inner constraint, joins and filters correctly. `acc::sum ?v` for the
/// same reason as row 2 above.
#[test]
fn earlier_bound_join_var_compiles_and_fires() {
    let acc = "(?n <- (:wat::rete::acc::sum ?v) :from (:w3::Reading (?loc <- :location) (?v <- :value) (:wat::rete::core::i64::> ?v ?min)))";
    let n = busy_count(true, acc, "(:wat::rete::core::i64::= ?n 12)", 3, &[("Oslo", 1), ("Oslo", 5), ("Oslo", 7)])
        .expect("earlier-bound-join accumulate must compile and fire");
    assert_eq!(n, 1, "threshold min=3; 5 + 7 = 12 (of 3 readings, `?v > ?min` filters out 1), gate = 12 -> fires once");
}

/// row 4 — NOT KNOWABLE operand, compile-only. See the module doc for why this is not fired.
///
/// ⛔ Uses `cond` (a `Form`-class op with no `TypeScheme`, so `resolve_operand_type` returns
/// `ComputedNotDerivableHere`) — NOT the fence's `i64::+ ?v 1 :undefined 0` trick.
/// `i64::+`'s vocabulary row declares `ret: Ret::Is(ParamType::I64)`, a REAL knowable type, so it
/// resolves as `Resolved("i64")` and never reaches `ComputedNotDerivableHere` — driven directly
/// (a mutation that turns that arm into a refusal left this test green), so it would not have
/// caught an over-rejecting cure. `cond` is the same construction
/// `probe_arc278_D10_then_field_types_notknowable.wat`'s `nk1` uses for the identical reason on
/// the `:then` side.
#[test]
fn not_knowable_operand_still_compiles() {
    let acc = "(?n <- (:wat::rete::acc::count) :from (:w3::Reading (?loc <- :location) (?v <- :value) (:wat::rete::core::i64::= ?v (:wat::rete::core::cond ((:wat::rete::core::string::= ?loc \"Oslo\") 10) (:else 999)))))";
    let world_src = world(false, acc, "(:wat::rete::core::i64::= ?n 0)");
    let result = startup_from_source(&world_src, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_ok(),
        "a computed, not-statically-typeable operand inside `:from`'s inner must not be refused; \
         got: {:?}",
        result.err()
    );
}
