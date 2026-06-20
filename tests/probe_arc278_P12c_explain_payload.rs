//! Arc 278 — P12c: the EXPLAIN payload (`:constraints` / `:bindings` / `:pattern` / `rule`).
//! RED at HEAD (`DerivationStep` + `DerivationNode/rule` + the payload accessors don't exist; via is still
//! `PV<DerivationNode>`); GREEN when P12c lands. Contract: DESIGN-STONE-P12c-explain-payload.md.
//!
//! The operator-legibility stone: each support edge (`DerivationStep`) carries the satisfied constraint
//! predicates with concrete values substituted (`(:wat::core::< -5 0)`), the per-step bound vars, the matched
//! type, and the node's rule. These assertions are on the cold-and-windy explain; `via[0]` is the Temperature
//! step (first condition).
//!
//! Run: cargo test --release -p wat --test probe_arc278_P12c_explain_payload -- --include-ignored

use std::sync::Arc;
use wat::ast::WatAST;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const WORLD: &str = "\
(:wat::Record::def :weather::Temperature  [celsius <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::Record::def :weather::WindSpeed    [kph     <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::Record::def :weather::ColdAndWindy [celsius <- :wat::core::i64  kph      <- :wat::core::i64])\n\
(:wat::Record::def :weather::WeatherAlert [celsius <- :wat::core::i64  kph      <- :wat::core::i64])\n\
\n\
(:wat::rete::defrule :weather::cold-and-windy\n\
  :when\n\
  [(:weather::Temperature (?loc <- :location) (?c <- :celsius) (:wat::core::< ?c 0))\n\
   (:weather::WindSpeed   (?loc <- :location) (?k <- :kph)     (:wat::core::> ?k 30))]\n\
  :then\n\
  (:wat::rete::insert (:weather::ColdAndWindy ?c ?k)))\n\
\n\
(:wat::rete::defrule :weather::alert\n\
  :when\n\
  [(:weather::ColdAndWindy (?c <- :celsius) (?k <- :kph))]\n\
  :then\n\
  (:wat::rete::insert (:weather::WeatherAlert ?c ?k)))\n\
\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

/// Lifecycle prefix binding `root` (explain of ColdAndWindy) and `step0` (its first via edge), then `body`.
fn nav(body: &str) -> Value {
    let compute = format!(
        "(:wat::core::let\n\
          [rules   (:wat::rete::collect-rules :weather)\n\
           session (:wat::rete::compile rules)\n\
           session (:wat::rete::insert session (:weather::Temperature -5 \"Oslo\"))\n\
           session (:wat::rete::insert session (:weather::WindSpeed    40 \"Oslo\"))\n\
           ex      (:wat::rete::fire-rules-explain session)\n\
           root    (:wat::rete::explain ex (:weather::ColdAndWindy -5 40))\n\
           step0   (:wat::core::first (:wat::rete::DerivationNode/via root))]\n\
          {body})"
    );
    let world = startup_from_source(WORLD, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let ast = wat::parse_one!(&compute).expect("parse compute");
    eval_in_frozen(&ast, &world, &Environment::new()).expect("compute should run").value_owned()
}

/// PATTERN — the first step matched a Temperature condition.
#[test]
#[ignore = "RED until P12c lands — un-ignore on completion"]
fn step_pattern_is_the_matched_type() {
    let v = nav("(:wat::rete::DerivationStep/pattern step0)");
    assert!(matches!(&v, Value::String(s) if s.as_str() == "weather::Temperature"), "pattern = matched type; got {v:?}");
}

/// BINDINGS — per-step: the Temperature step bound ?c = -5 (projected to THIS condition's vars).
#[test]
#[ignore = "RED until P12c lands — un-ignore on completion"]
fn step_bindings_are_per_step() {
    let v = nav("(:wat::core::PersistentMap/get (:wat::rete::DerivationStep/bindings step0) \"?c\")");
    assert!(matches!(&v, Value::Option(o) if matches!(&**o, Some(Value::i64(-5)))), "bindings[?c] = -5; got {v:?}");
}

/// RULE (Some) — the root (a derived fact) carries its rule name.
#[test]
#[ignore = "RED until P12c lands — un-ignore on completion"]
fn derived_node_rule_is_some() {
    let v = nav("(:wat::rete::DerivationNode/rule root)");
    assert!(
        matches!(&v, Value::Option(o) if matches!(&**o, Some(Value::String(s)) if s.as_str() == "weather::cold-and-windy")),
        "root rule = Some(\"weather::cold-and-windy\"); got {v:?}"
    );
}

/// RULE (None) — a base/asserted supporting fact has no rule (renders nil).
#[test]
#[ignore = "RED until P12c lands — un-ignore on completion"]
fn base_node_rule_is_none() {
    let v = nav("(:wat::rete::DerivationNode/rule (:wat::rete::DerivationStep/supporting step0))");
    assert!(matches!(&v, Value::Option(o) if o.is_none()), "base fact rule = None; got {v:?}");
}

/// CONSTRAINTS count — one satisfied predicate on the Temperature step ((< ?c 0)).
#[test]
#[ignore = "RED until P12c lands — un-ignore on completion"]
fn step_has_one_constraint() {
    let v = nav("(:wat::core::length (:wat::rete::DerivationStep/constraints step0))");
    assert!(matches!(v, Value::i64(1)), "one constraint on the Temperature step; got {v:?}");
}

/// CONSTRAINTS substitution (THE load-bearing assertion) — the satisfied predicate is the form with the bound
/// value substituted: `(:wat::core::< -5 0)` (?c → -5), NOT `(:wat::core::< ?c 0)`. Span-agnostic structural match.
#[test]
#[ignore = "RED until P12c lands — un-ignore on completion"]
fn constraint_is_the_substituted_form() {
    let v = nav("(:wat::core::first (:wat::rete::DerivationStep/constraints step0))");
    let Value::wat__WatAST(a) = &v else { panic!("constraint must be a WatAST form; got {v:?}") };
    let WatAST::List(items, _) = a.as_ref() else { panic!("constraint must be a list form; got {a:?}") };
    assert_eq!(items.len(), 3, "(op a b); got {items:?}");
    assert!(matches!(items[1], WatAST::IntLit(-5, _)), "operand a must be the substituted -5 (not ?c); got {:?}", items[1]);
    assert!(matches!(items[2], WatAST::IntLit(0, _)), "operand b must be 0; got {:?}", items[2]);
}
