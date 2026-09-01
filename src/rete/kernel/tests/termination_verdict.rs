//! The termination verdict — one probe per state the verifier can be in.
//!
//! ⛔ WHY THESE LIVE IN RUST AND NOT IN `tests/rete/*.wat`. `TerminationVerdict::NotAnalysable` is
//! deliberately NOT wire-visible: surfacing it would need a new `(:wat::rete::CompileOutcome)`
//! variant behind the outcome wall, which is affirmatively out of scope for the strike that split
//! this type. From wat, `NotAnalysable` and `Proven` both answer `Compiled` — which is precisely
//! the behaviour these probes must NOT disturb — so the only place the distinction is observable
//! is at the `pub(crate)` boundary, here.
//!
//! ⚠ AND WHY A MIXED RULE SET IS THE INTERESTING ONE. If EVERY rule in the set lacks an AST,
//! `edges` is empty, so `edges.iter().all(|e| e.computed.is_none())` is vacuously true and the
//! nothing-computes early exit fires BEFORE the AST-less `continue`'s count can ever reach the
//! graph walk. A probe built only from AST-less rules therefore measures the early exit and proves
//! nothing about the other return. Each pair below is written so exactly one of the two returns is
//! reachable: the `:a5p` sets carry no computed head at all (early exit), the `:a5v` sets do
//! (graph walk). Collapsing either return alone turns exactly one pair red.

use super::*;

use crate::freeze::FrozenWorld;
use crate::rete::kernel::stratify::{refuse_non_terminating, TerminationVerdict};

/// Three rule shapes in three namespaces, so `collect-rules` can hand back one at a time.
///
/// - `:a5p` — a rule that DERIVES but never COMPUTES (no list argument in the `:then` head), so
///   its edge carries `computed: None` and the nothing-computes early exit is the one that fires.
/// - `:a5v` — a rule that computes (`?k + 1`) but is NOT cyclic (`In` in, `Out` out), so the
///   derivation graph is built, closes, and the graph-closed return is the one that fires.
/// - `:a5d` — the soundness twin: computes, cyclic, and its fence points WITH the step, so it is
///   refused. Its terminating sibling is one character away (`<` instead of `>`).
const WORLD: &str = "\
(:wat::core::defrecord :a5p::A [k <- :wat::core::i64])\n\
(:wat::core::defrecord :a5p::B [k <- :wat::core::i64])\n\
(:wat::rete::defrule :a5p::plain\n\
  :when [(:a5p::A (?k <- :k))]\n\
  :then [(:a5p::B :k ?k)])\n\
\n\
(:wat::core::defrecord :a5v::In  [k <- :wat::core::i64])\n\
(:wat::core::defrecord :a5v::Out [k <- :wat::core::i64])\n\
(:wat::rete::defrule :a5v::computes\n\
  :when [(:a5v::In (?k <- :k))]\n\
  :then [(:a5v::Out :k (:wat::rete::core::i64::+ ?k 1 :undefined 0))])\n\
\n\
(:wat::core::defrecord :a5d::N [k <- :wat::core::i64])\n\
(:wat::rete::defrule :a5d::diverges\n\
  :when [(:a5d::N (?k <- :k))\n\
         (:wat::rete::where (:wat::rete::core::i64::> ?k 500))]\n\
  :then [(:a5d::N :k (:wat::rete::core::i64::+ ?k 1 :undefined 0))])\n\
";

/// A `Rule` with empty `:lhs`/`:rhs` — exactly the shape an imported Export's rules have, and the
/// shape `wat-scripts/scratch-pad/a5-termination-silence.wat` drives through `compile-all`.
const AST_LESS: &str = "(:wat::rete::Rule :name \"ast-less\" \
    :lhs (:wat::core::PersistentVector) :rhs (:wat::core::PersistentVector))";

fn a5_world() -> FrozenWorld {
    startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new()))
        .expect("the A5 verdict world should freeze")
}

/// Evaluate a rule-set expression against the world and hand back the `PersistentVector` value.
fn rule_set(world: &FrozenWorld, src: &str) -> Value {
    let ast = crate::parse_one!(src).expect("parse the rule-set expression");
    eval_in_frozen(&ast, world, &Environment::new())
        .unwrap_or_else(|e| panic!("rule-set expression raised: {e:?}"))
        .value_owned()
}

/// The verdict as a byte-exact name, so every assertion below is an `assert_eq!` on a
/// deterministic string rather than a loose shape check. The count is part of the name: a verdict
/// that says "not analysable" without saying HOW MANY is the same silence this type replaced.
fn verdict_name(v: &TerminationVerdict) -> String {
    match v {
        TerminationVerdict::Proven => String::from("Proven"),
        TerminationVerdict::NotAnalysable { rules } => format!("NotAnalysable({rules})"),
        TerminationVerdict::Refused(_) => String::from("Refused"),
    }
}

/// The not-a-vector return. Nothing was analysed and there is no rule to count — and that is NOT
/// the same answer as "analysed everything, found nothing to skip".
#[test]
fn a_non_vector_rules_argument_is_not_analysable() {
    let world = a5_world();
    let not_a_vector = Value::i64(7);
    let got = verdict_name(&refuse_non_terminating(&not_a_vector, world.symbols()));
    assert_eq!(
        got, "NotAnalysable(0)",
        "a rules argument that is not a `PersistentVector` was never looked at; answering `Proven` \
         would be the conflation this type exists to make unrepresentable"
    );
}

/// ⛔ THE ANTI-INVERSION ROW. The nothing-computes early exit is a REAL PROOF, taken by 371 of 381
/// corpus rules (measured 2026-08-27). If a future change folds it into `NotAnalysable`, the
/// overwhelming majority of the corpus starts reading as unverified — a green suite with the
/// finding backwards. This row is what makes that loud.
#[test]
fn a_rule_set_that_computes_nothing_is_proven() {
    let world = a5_world();
    let rules = rule_set(&world, "(:wat::rete::collect-rules :a5p)");
    let got = verdict_name(&refuse_non_terminating(&rules, world.symbols()));
    assert_eq!(
        got, "Proven",
        "nothing computes, so no cycle can be unbounded — that is a proof about every rule in the \
         set, and none of them was skipped"
    );
}

/// The graph-closed return, likewise a real proof: something computes, the derivation graph was
/// built, and no unbounded cycle closed through it.
#[test]
fn a_computing_acyclic_rule_set_is_proven() {
    let world = a5_world();
    let rules = rule_set(&world, "(:wat::rete::collect-rules :a5v)");
    let got = verdict_name(&refuse_non_terminating(&rules, world.symbols()));
    assert_eq!(
        got, "Proven",
        "`:a5v::computes` derives `:a5v::Out` from `:a5v::In` and nothing feeds back — the graph \
         closes with no unbounded cycle, which is a proof and must not read as a skip"
    );
}

/// The AST-less rule ALONE. `edges` is empty, so this reaches the nothing-computes early exit —
/// and the count carried there is what stops that exit from calling an unlooked-at set proven.
/// This is the repro's own shape.
#[test]
fn an_ast_less_rule_alone_is_not_analysable() {
    let world = a5_world();
    let rules = rule_set(
        &world,
        &format!("(:wat::core::PersistentVector :- [:wat::rete::Rule] {AST_LESS})"),
    );
    let got = verdict_name(&refuse_non_terminating(&rules, world.symbols()));
    assert_eq!(
        got, "NotAnalysable(1)",
        "a `Rule` with empty `:lhs`/`:rhs` carries no AST to analyse; before the verdict was split \
         this returned the same value as a proof, and `compile-all` answered `Compiled`"
    );
}

/// A mixed set that still lands on the EARLY EXIT — deliberately, and it is worth being precise
/// about why, because the obvious reading is wrong. The early exit fires on
/// `edges.iter().all(|e| e.computed.is_none())`, which is about what the edges COMPUTE and not
/// about whether any edge exists. `:a5p::plain` builds a real edge with `computed: None`, so the
/// predicate is still true and control never reaches the graph. This row therefore pins the early
/// exit with a surviving rule beside the skipped one; the graph-walk return is the NEXT test's.
#[test]
fn a_skipped_rule_beside_a_non_computing_one_is_not_analysable() {
    let world = a5_world();
    let rules = rule_set(
        &world,
        &format!("(:wat::core::conj (:wat::rete::collect-rules :a5p) {AST_LESS})"),
    );
    let got = verdict_name(&refuse_non_terminating(&rules, world.symbols()));
    assert_eq!(
        got, "NotAnalysable(1)",
        "one rule was analysed and one was skipped — the proof covers the edge that was built and \
         says nothing about the rule that was not"
    );
}

/// The same mixture, but the analysable rule COMPUTES — so the early exit is bypassed entirely and
/// the skip count has to survive the graph walk to reach the graph-closed return.
#[test]
fn a_skipped_rule_beside_a_computing_one_is_not_analysable() {
    let world = a5_world();
    let rules = rule_set(
        &world,
        &format!("(:wat::core::conj (:wat::rete::collect-rules :a5v) {AST_LESS})"),
    );
    let got = verdict_name(&refuse_non_terminating(&rules, world.symbols()));
    assert_eq!(
        got, "NotAnalysable(1)",
        "the derivation graph closed for the one rule that had an AST; the other was never looked \
         at, and the graph-closed return may not launder that into a proof"
    );
}

/// The refusal still refuses. A cyclic computed head whose fence points WITH the step has neither
/// proof available.
#[test]
fn an_unbounded_cycle_is_still_refused() {
    let world = a5_world();
    let rules = rule_set(&world, "(:wat::rete::collect-rules :a5d)");
    let got = verdict_name(&refuse_non_terminating(&rules, world.symbols()));
    assert_eq!(
        got, "Refused",
        "`k + 1` while `k > 500` satisfies its own guard forever; splitting the verdict by type \
         must not have widened what the verifier admits"
    );
}

/// ⛔ BEHAVIOUR DID NOT CHANGE, and this is the row that says so. `NotAnalysable` is a NAME for a
/// state, not a refusal: making it fatal would break every session whose rules legitimately carry
/// no AST — every imported Export — and that is a policy change, not an honesty fix. So the very
/// set that now answers `NotAnalysable(1)` above must still compile.
#[test]
fn not_analysable_still_compiles_and_is_not_a_refusal() {
    let world = a5_world();
    let src = format!(
        "(:wat::core::match (:wat::rete::compile-all \
           (:wat::core::PersistentVector :- [:wat::rete::Rule] {AST_LESS}) \
           (:wat::core::PersistentVector)) \
           ((:wat::rete::CompileOutcome::Compiled __s) \"Compiled\") \
           ((:wat::rete::CompileOutcome::MayNotTerminate __r __f) \"MayNotTerminate\"))"
    );
    let ast = crate::parse_one!(src.as_str()).expect("parse the compile-all driver");
    let outcome = eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("compile-all raised on an AST-less rule set: {e:?}"))
        .value_owned();
    let got = match &outcome {
        Value::String(s) => (**s).clone(),
        other => panic!("compile-all driver returned a non-String: {other:?}"),
    };
    assert_eq!(
        got, "Compiled",
        "an AST-less rule set must still compile. If this is `MayNotTerminate`, `NotAnalysable` \
         was made fatal — stop, because that is a policy change wearing an honesty fix"
    );
}
