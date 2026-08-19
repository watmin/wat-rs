//! Acceptance gate for arc 278's vacuous-gate wall
//! (`docs/arc/2026/06/278-rules-engine/BRIEF-vacuous-deftest-gate-wall.md`).
//!
//! The heresy this pins shut: `deftest` fixtures were gated from Rust by
//! `call_beside(file!(), ":user::x")` followed by `assert!(result.is_ok())`. `call_beside`
//! returned `Result<Value, RuntimeError>`; a fired `assert-eq` landed a `Failure` in the
//! returned `:wat::kernel::RunResult` and the *evaluation* still succeeded, so `Ok` came
//! back. `.is_ok()` answers "did it evaluate?", never "did it pass?" — so every assertion
//! in such a fixture was decoration. Proven by mutating a live gate's `(assert-eq n 1)` to
//! `n 4242` and watching the test still PASS.
//!
//! Written RED-first: at HEAD (`call_beside -> Result`) the first test below failed,
//! because a deliberately-false assertion came back `Ok`.
//!
//! The claims, one per channel the wall must keep honest:
//!  1. a FAILING deftest is reported as `DeftestOutcome::Failed`, carrying the structured
//!     `:wat::kernel::Failure` (message / location / actual / expected);
//!  2. a PASSING deftest is reported as `DeftestOutcome::Passed`;
//!  3. the two verbs refuse each other's targets — `call_beside_value` will not run a
//!     deftest (that is the ignore-the-verdict path), and `call_beside` will not invent a
//!     verdict for a plain fn;
//!  4. the value verb still returns a plain fn's Value.

use wat::freeze::{call_beside, call_beside_value, DeftestOutcome};
use wat::runtime::Value;

/// Positional fields of a `Value::Aggregate` of the named colon-free class.
/// The verdict's payload is DATA (a `:wat::kernel::Failure` record), so it is asserted
/// STRUCTURALLY — field by field — never by string-matching a rendered blob.
fn aggregate_fields<'a>(v: &'a Value, class: &str) -> &'a [Value] {
    match v {
        Value::Aggregate(a) if a.class.as_ref() == class => &a.fields,
        other => panic!("expected a {class} aggregate; got {other:?}"),
    }
}

fn option_string(v: &Value) -> Option<String> {
    match v {
        Value::Option(o) => match &**o {
            Some(Value::String(s)) => Some((**s).clone()),
            _ => None,
        },
        other => panic!("expected an Option<String>; got {other:?}"),
    }
}

/// The load-bearing one. A deftest whose assertion cannot hold MUST come back `Failed`,
/// and the failure must be the real structured diagnostic, not a boolean.
#[test]
fn failing_deftest_is_reported_as_failed() {
    let failure = match call_beside(file!(), ":user::verdict-wall-fails") {
        DeftestOutcome::Failed { failure } => failure,
        other => panic!(
            "a deftest with a deliberately-false assertion (2+2 = 4242) MUST be reported as \
             Failed — a fired assertion is not a pass; got: {other:?}"
        ),
    };
    // :wat::kernel::Failure — fields [error, frames, actual, expected] (arc 278).
    let f = aggregate_fields(&failure, "wat::kernel::Failure");
    // :wat::core::Fault — fields [message, location, causes].
    let fault = aggregate_fields(&f[0], "wat::core::Fault");
    assert_eq!(
        match &fault[0] {
            Value::String(s) => (**s).clone(),
            other => panic!("Fault.message must be a String; got {other:?}"),
        },
        "assert-eq failed"
    );
    // :wat::kernel::Location — fields [file, line, col]. The fixture's false assertion
    // is the SECOND deftest in the co-located .wat.
    let loc = aggregate_fields(&fault[1], "wat::kernel::Location");
    assert_eq!(
        match &loc[0] {
            Value::String(s) => std::path::Path::new(&**s)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned()),
            other => panic!("Location.file must be a String; got {other:?}"),
        },
        Some("probe_deftest_verdict_wall.wat".to_string())
    );
    assert_eq!(option_string(&f[2]), Some("4".to_string()), "Failure.actual");
    assert_eq!(option_string(&f[3]), Some("4242".to_string()), "Failure.expected");
}

/// The other half of the claim: the wall does not simply call everything a failure.
#[test]
fn passing_deftest_is_reported_as_passed() {
    call_beside(file!(), ":user::verdict-wall-passes")
        .expect_passed("a deftest whose assertion holds must be reported as Passed");
}

/// Wall #2 — the ignore-the-verdict path has no form. Driving a deftest through the
/// value verb (where its `RunResult` would be an ignorable `Ok(Value)`) is refused.
#[test]
#[should_panic(expected = "IS a deftest")]
fn value_verb_refuses_a_deftest_target() {
    let _ = call_beside_value(file!(), ":user::verdict-wall-fails");
}

/// The mirror wall — the verdict verb refuses a plain fn rather than inventing a verdict
/// it has no basis for.
#[test]
#[should_panic(expected = "is NOT a deftest")]
fn verdict_verb_refuses_a_plain_fn_target() {
    let _ = call_beside(file!(), ":user::plain-value");
}

/// And the value verb still does its own job: a plain fn's Value comes back.
#[test]
fn value_verb_returns_a_plain_fns_value() {
    match call_beside_value(file!(), ":user::plain-value").expect("plain-value must evaluate") {
        Value::i64(n) => assert_eq!(n, 7),
        other => panic!("plain-value must return i64 7; got {other:?}"),
    }
}
