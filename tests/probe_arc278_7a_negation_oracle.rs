//! Arc 278 — Stone 7-a: negation (`:not` / NegationNode) in the ORACLE (`rete.wat` compile + fire).
//! RED at HEAD (no NegationNode: compile-condition treats `(:wat::rete::not …)` as an unsatisfiable
//! alpha → the rule never fires). GREEN when 7-a lands. Contract: DESIGN-STONE-7-negation.md.
//!
//! Probed through the ORACLE (`fire-rules-spec`) — 7-a builds the oracle NegationNode; the native port +
//! differential are 7-b. A `:not` passes a token iff NO fact matches the negated condition for that
//! token's bindings (the shared `?loc` must agree — the join-filter half).
//!
//! Run: cargo test --release -p wat --test probe_arc278_7a_negation_oracle

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

/// Fire the oracle after the given inserts (each a wat insert form) and count derived Unattended facts.
fn count(inserts: &[&str]) -> Result<i64, String> {
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
             fired   (:wat::rete::fire-rules-spec session)]\n\
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

/// 1 — `:not` PASSES when the negated fact is ABSENT: Temp(Oslo), no Maintenance → 1 Unattended.
#[test]
fn negation_passes_when_absent() {
    let r = count(&["(:weather::Temperature -5 \"Oslo\")"]);
    assert!(matches!(r, Ok(1)), "no Maintenance at Oslo → 1 Unattended; got {r:?}");
}

/// 2 — `:not` BLOCKS when the negated fact is PRESENT and MATCHES: Temp(Oslo) + Maintenance(Oslo) → 0.
#[test]
fn negation_blocks_when_present_matching() {
    let r = count(&["(:weather::Temperature -5 \"Oslo\")", "(:ops::Maintenance \"Oslo\")"]);
    assert!(matches!(r, Ok(0)), "Maintenance at Oslo → 0 Unattended; got {r:?}");
}

/// 3 — `:not` PASSES when a negated fact exists but at a DIFFERENT binding (the shared-var join-filter):
/// Temp(Oslo) + Maintenance(Bergen) → the Bergen maintenance does NOT match ?loc=Oslo → 1 Unattended.
#[test]
fn negation_passes_when_present_different_binding() {
    let r = count(&["(:weather::Temperature -5 \"Oslo\")", "(:ops::Maintenance \"Bergen\")"]);
    assert!(matches!(r, Ok(1)), "Maintenance at Bergen ≠ Oslo → 1 Unattended; got {r:?}");
}
