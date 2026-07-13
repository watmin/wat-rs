//! Arc 278 — Stone 7-exists: `:exists` (existential, the NegationNode sibling) — oracle + native + DIFFERENTIAL.
//! `(:wat::rete::exists <inner>)` passes a token iff ≥1 element matches the inner condition for its bindings,
//! binds NOTHING, and fires the token EXACTLY ONCE regardless of how many match (no multiplicity — the key
//! difference from a join). It is NegationNode's filter predicate flipped: negation passes iff ZERO compatible;
//! exists passes iff ≥1 compatible. RED at HEAD (`:exists` head is unrecognized → compile mis-handles it →
//! native and/or oracle wrong). GREEN when 7-exists lands. Contract: DESIGN-STONE-7-exists.md.
//!
//! `fire-rules` = native (`fire-rules'`); `fire-rules-spec` = the wat oracle. For an `:exists` rule the two
//! MUST agree.
//!
//! Run: cargo test --release -p wat --test probe_arc278_7exists_native_differential

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// Fire via `fire_fn` after the given inserts; count derived Watched facts.
fn count(fire_fn: &str, inserts: &[&str]) -> Result<i64, String> {
    let insert_lines: String = inserts
        .iter()
        .map(|f| format!("             session (:wat::rete::insert session {f})\n"))
        .collect();
    let run = format!(
        "(:wat::core::length\n\
          (:wat::core::let\n\
            [rules   (:wat::rete::collect-rules :w)\n\
             session (:wat::rete::compile rules)\n\
{insert_lines}\
             fired   (:wat::rete::{fire_fn} session)]\n\
            (:wat::rete::query fired :w::Watched)))"
    );
    let world = startup_beside(file!()).map_err(|e| format!("startup: {e:?}"))?;
    let ast = wat::parse_one!(&run).map_err(|e| format!("parse: {e:?}"))?;
    match eval_in_frozen(&ast, &world, &Environment::new()).map_err(|e| format!("eval: {e:?}"))?.value_owned() {
        Value::i64(n) => Ok(n),
        other => Err(format!("expected i64; got {other:?}")),
    }
}

const STATION_ONLY: &[&str] = &["(:w::Station :location \"Oslo\")"];
const STATION_ONE_READING: &[&str] = &["(:w::Station :location \"Oslo\")", "(:w::Reading :location \"Oslo\" :value 1)"];
const STATION_THREE_READINGS: &[&str] = &[
    "(:w::Station :location \"Oslo\")",
    "(:w::Reading :location \"Oslo\" :value 1)",
    "(:w::Reading :location \"Oslo\" :value 2)",
    "(:w::Reading :location \"Oslo\" :value 3)",
];
const STATION_READING_ELSEWHERE: &[&str] = &["(:w::Station :location \"Oslo\")", "(:w::Reading :location \"Bergen\" :value 1)"];

/// 1 — DIFFERENTIAL, exists passes (≥1 match): native == oracle, both 1.
#[test]
fn differential_exists_present() {
    let native = count("fire-rules", STATION_ONE_READING).expect("native");
    let oracle = count("fire-rules-spec", STATION_ONE_READING).expect("oracle");
    assert_eq!(native, oracle, "native==oracle (present); native={native} oracle={oracle}");
    assert_eq!(native, 1, "≥1 reading → 1; got {native}");
}

/// 2 — DIFFERENTIAL, exists blocks (zero matches): native == oracle, both 0.
#[test]
fn differential_exists_absent() {
    let native = count("fire-rules", STATION_ONLY).expect("native");
    let oracle = count("fire-rules-spec", STATION_ONLY).expect("oracle");
    assert_eq!(native, oracle, "native==oracle (absent); native={native} oracle={oracle}");
    assert_eq!(native, 0, "no readings → 0; got {native}");
}

/// 3 — DIFFERENTIAL, NO MULTIPLICITY (the existential property): 3 readings → fires ONCE, not 3.
/// This is what separates `:exists` from a join — the token passes once iff ≥1, never multiplied.
#[test]
fn differential_exists_no_multiplicity() {
    let native = count("fire-rules", STATION_THREE_READINGS).expect("native");
    let oracle = count("fire-rules-spec", STATION_THREE_READINGS).expect("oracle");
    assert_eq!(native, oracle, "native==oracle (3 readings); native={native} oracle={oracle}");
    assert_eq!(native, 1, "3 readings → fires ONCE (existential, not a join); got {native}");
}

/// 4 — DIFFERENTIAL, the shared-var filter: a reading elsewhere does not satisfy exists for Oslo.
#[test]
fn differential_exists_shared_var() {
    let native = count("fire-rules", STATION_READING_ELSEWHERE).expect("native");
    let oracle = count("fire-rules-spec", STATION_READING_ELSEWHERE).expect("oracle");
    assert_eq!(native, oracle, "native==oracle (elsewhere); native={native} oracle={oracle}");
    assert_eq!(native, 0, "reading@Bergen ≠ Oslo → 0; got {native}");
}

/// 5 — the NATIVE engine alone honors exists (headline: native passes-once / blocks, no under/over-derive).
#[test]
fn native_exists_passes_once_and_blocks() {
    assert_eq!(count("fire-rules", STATION_ONE_READING).expect("native"), 1, "native ≥1 → 1");
    assert_eq!(count("fire-rules", STATION_ONLY).expect("native"), 0, "native none → 0");
    assert_eq!(count("fire-rules", STATION_THREE_READINGS).expect("native"), 1, "native 3 → 1 (once)");
}
