//! Arc 278 stone 7-exists — `:exists` (existential, the NegationNode sibling); `fire-rules` == `fire-rules$oracle`.
//! Dual-impl: the unprimed public Fn is native; `$oracle` is the spec mouth.
//!
//! `(:wat::rete::exists <inner>)` passes a token iff ≥1 element matches the inner condition for its bindings,
//! binds NOTHING, and fires the token EXACTLY ONCE regardless of how many match (no multiplicity — the key
//! difference from a join). It is NegationNode's filter predicate flipped: negation passes iff ZERO compatible;
//! exists passes iff ≥1 compatible. Present → 1; absent → 0; three readings still fire once.
//!
//! Run: cargo test --release -p wat --test probe_arc278_7exists_native_differential

use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

fn count(entry: &str) -> Result<i64, RuntimeError> {
    match call_beside_value(file!(), entry)? {
        Value::i64(n) => Ok(n),
        other => Err(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: format!("count({entry})"),
                expected: "i64",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )),
    }
}

#[test]
fn compile_watched_fires_nothing() {
    assert_eq!(count(":user::compile-watched-fires-nothing").expect("eval"), 0,
        "compile+fire with no facts derives no Watched");
}

/// 1 — DIFFERENTIAL, exists passes (≥1 match): native == oracle, both 1.
#[test]
fn differential_exists_present() {
    let native = count(":user::native-one-reading").expect("native");
    let oracle = count(":user::oracle-one-reading").expect("oracle");
    assert_eq!(native, oracle, "native==oracle (present); native={native} oracle={oracle}");
    assert_eq!(native, 1, "≥1 reading → 1; got {native}");
}

/// 2 — DIFFERENTIAL, exists blocks (zero matches): native == oracle, both 0.
#[test]
fn differential_exists_absent() {
    let native = count(":user::native-station-only").expect("native");
    let oracle = count(":user::oracle-station-only").expect("oracle");
    assert_eq!(native, oracle, "native==oracle (absent); native={native} oracle={oracle}");
    assert_eq!(native, 0, "no readings → 0; got {native}");
}

/// 3 — DIFFERENTIAL, 3 readings derive ONE fact. Named for what it can actually observe.
///
/// This was called `differential_exists_no_multiplicity` until 2026-08-25, when `vocare` found
/// it named for a contract its fixture cannot reach. `:w::watched`'s `:then` binds only `?loc`,
/// so three passes of the same token derive the SAME `Watched{location:"Oslo"}` three times and
/// `production_delta`'s value-dedup collapses them. A count of 1 is what a correct engine and a
/// fully-multiplying engine BOTH report — the assertion could not fail for the reason it named.
///
/// That is not a small mislabel: the same mask hid the real leading-filter multiplicity defect
/// for the whole arc. The contract now has a gate that can go red, directly below.
#[test]
fn differential_exists_three_readings_derive_one_fact() {
    let native = count(":user::native-three-readings").expect("native");
    let oracle = count(":user::oracle-three-readings").expect("oracle");
    assert_eq!(native, oracle, "native==oracle (3 readings); native={native} oracle={oracle}");
    assert_eq!(native, 1, "3 readings → ONE derived Watched; got {native}");
}

/// 3b — NO MULTIPLICITY, observed where it is observable: rows out of the rule's own BETA.
///
/// `:w::q-watched-tokens` carries `:w::watched`'s `:when` verbatim, so `query` reads beta and
/// every token that reached the production is one row — below the derived-fact dedup that blinds
/// test 3. Three readings against one distinct `?loc`: correct is ONE row; an engine that
/// multiplied the existential reports three. Native and oracle are held to the same number, so
/// a divergence names itself rather than merely failing.
#[test]
fn differential_exists_no_multiplicity() {
    let native = count(":user::native-three-readings-tokens").expect("native");
    let oracle = count(":user::oracle-three-readings-tokens").expect("oracle");
    assert_eq!(
        native, oracle,
        "native==oracle (3 readings, beta rows); native={native} oracle={oracle}"
    );
    assert_eq!(
        native, 1,
        "3 readings → the token passes ONCE (existential, not a join); got {native} beta row(s)"
    );

    // The control: one reading must give the same single row. If this ever differs from the
    // three-reading count in the OTHER direction, the query is counting readings, not tokens —
    // i.e. the instrument itself has stopped measuring what it claims.
    let one = count(":user::native-one-reading-tokens").expect("native one");
    assert_eq!(one, 1, "1 reading → 1 beta row; got {one}");
}

/// 3c — THE INSTRUMENT'S SENSITIVITY, proven rather than assumed.
///
/// `:w::q-watched-join` is `:w::q-watched-tokens` with the `exists` wrapper removed and nothing
/// else changed. A plain join multiplies, so the SAME three readings must give THREE beta rows
/// here against one there. That difference is the whole content of "no multiplicity", and it is
/// what makes the sibling gate a measurement instead of a restatement: a query that could only
/// ever return 1 would pass 3b while seeing nothing.
///
/// If this ever reads 1, do not trust 3b's green — the instrument has gone blind.
#[test]
fn the_multiplicity_instrument_can_count_above_one() {
    let joined = count(":user::native-three-readings-join-tokens").expect("native join");
    assert_eq!(
        joined, 3,
        "3 readings through a plain JOIN → 3 beta rows (this is the control that proves \
         `differential_exists_no_multiplicity` can go red); got {joined}"
    );

    let existential = count(":user::native-three-readings-tokens").expect("native exists");
    assert!(
        joined > existential,
        "the join must out-count the existential on identical facts — join={joined}, \
         exists={existential}. Equal counts mean the wrapper changed nothing and the \
         existential contract is untested"
    );
}

/// 4 — DIFFERENTIAL, the shared-var filter: a reading elsewhere does not satisfy exists for Oslo.
#[test]
fn differential_exists_shared_var() {
    let native = count(":user::native-reading-elsewhere").expect("native");
    let oracle = count(":user::oracle-reading-elsewhere").expect("oracle");
    assert_eq!(native, oracle, "native==oracle (elsewhere); native={native} oracle={oracle}");
    assert_eq!(native, 0, "reading@Bergen ≠ Oslo → 0; got {native}");
}

/// 5 — the NATIVE engine alone honors exists (headline: native passes-once / blocks, no under/over-derive).
#[test]
fn native_exists_passes_once_and_blocks() {
    assert_eq!(count(":user::native-one-reading").expect("native"), 1, "native ≥1 → 1");
    assert_eq!(count(":user::native-station-only").expect("native"), 0, "native none → 0");
    assert_eq!(count(":user::native-three-readings").expect("native"), 1, "native 3 → 1 (once)");
}
