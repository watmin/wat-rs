//! Arc 278 — Stone 7-b: negation (`:not`/NegationNode) in the NATIVE kernel + the DIFFERENTIAL (native==oracle).
//! RED at HEAD (7-a taught the ORACLE + compile the NegationNode, but the native delta engine
//! `fire_fixpoint_delta` has no negation filter → native under-derives → native ≠ oracle).
//! GREEN when 7-b lands. Contract: DESIGN-STONE-7-negation.md (the 7-b entry).
//!
//! `fire-rules` = native (P5a → `fire-rules'`); `fire-rules-spec` = the wat oracle. For a `:not` rule the
//! two MUST agree. Negation passes a token iff NO fact matches the negated condition for its bindings.
//!
//! Run: cargo test --release -p wat --test probe_arc278_7b_negation_native_differential

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const WORLD: &str = "\
(:wat::Record::def :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])\n\
(:wat::Record::def :ops::Maintenance     [location <- :wat::core::String])\n\
(:wat::Record::def :alert::Unattended    [location <- :wat::core::String])\n\
\n\
(:wat::rete::defrule :alert::unattended\n\
  :when\n\
  [(:weather::Temperature (?loc <- :location) (?c <- :celsius))\n\
   (:wat::rete::not (:ops::Maintenance (?loc <- :location)))]\n\
  :then\n\
  (:wat::rete::insert (:alert::Unattended ?loc)))\n\
\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

/// Fire via `fire_fn` after the given inserts; count derived Unattended facts.
fn count(fire_fn: &str, inserts: &[&str]) -> Result<i64, String> {
    let insert_lines: String = inserts
        .iter()
        .map(|f| format!("             session (:wat::rete::insert session {f})\n"))
        .collect();
    let run = format!(
        "(:wat::core::length\n\
          (:wat::core::let\n\
            [rules   (:wat::rete::collect-rules :alert)\n\
             session (:wat::rete::compile rules)\n\
{insert_lines}\
             fired   (:wat::rete::{fire_fn} session)]\n\
            (:wat::rete::query fired :alert::Unattended)))"
    );
    let world = startup_from_source(WORLD, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {e:?}"))?;
    let ast = wat::parse_one!(&run).map_err(|e| format!("parse: {e:?}"))?;
    match eval_in_frozen(&ast, &world, &Environment::new()).map_err(|e| format!("eval: {e:?}"))?.value_owned() {
        Value::i64(n) => Ok(n),
        other => Err(format!("expected i64; got {other:?}")),
    }
}

const ABSENT: &[&str] = &["(:weather::Temperature -5 \"Oslo\")"];
const PRESENT_MATCH: &[&str] = &["(:weather::Temperature -5 \"Oslo\")", "(:ops::Maintenance \"Oslo\")"];
const PRESENT_DIFF: &[&str] = &["(:weather::Temperature -5 \"Oslo\")", "(:ops::Maintenance \"Bergen\")"];

/// 1 — DIFFERENTIAL, negation passes (absent): native == oracle, both 1.
#[test]
fn differential_negation_absent() {
    let native = count("fire-rules", ABSENT).expect("native");
    let oracle = count("fire-rules-spec", ABSENT).expect("oracle");
    assert_eq!(native, oracle, "native==oracle (absent); native={native} oracle={oracle}");
    assert_eq!(native, 1, "absent → 1; got {native}");
}

/// 2 — DIFFERENTIAL, negation blocks (present-matching): native == oracle, both 0.
#[test]
fn differential_negation_present_matching() {
    let native = count("fire-rules", PRESENT_MATCH).expect("native");
    let oracle = count("fire-rules-spec", PRESENT_MATCH).expect("oracle");
    assert_eq!(native, oracle, "native==oracle (present-matching); native={native} oracle={oracle}");
    assert_eq!(native, 0, "present-matching → 0; got {native}");
}

/// 3 — DIFFERENTIAL, the shared-var join-filter (present-different): native == oracle, both 1.
#[test]
fn differential_negation_present_different() {
    let native = count("fire-rules", PRESENT_DIFF).expect("native");
    let oracle = count("fire-rules-spec", PRESENT_DIFF).expect("oracle");
    assert_eq!(native, oracle, "native==oracle (present-different); native={native} oracle={oracle}");
    assert_eq!(native, 1, "Maintenance@Bergen ≠ Oslo → 1; got {native}");
}

/// 4 — the NATIVE engine alone honors negation (the headline: native filters, not under-derives).
#[test]
fn native_negation_passes_and_blocks() {
    assert_eq!(count("fire-rules", ABSENT).expect("native"), 1, "native absent → 1");
    assert_eq!(count("fire-rules", PRESENT_MATCH).expect("native"), 0, "native present-match → 0");
}
