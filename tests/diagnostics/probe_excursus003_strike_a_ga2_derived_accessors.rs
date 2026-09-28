//! GA2 (excursus 003 strike A, BRIEF-shape-strike-A-one-death-shape.md) — the
//! DERIVED accessors: `(:wat::kernel::Failure/actual f)` returns `Some` when the
//! death is an assertion, `None` for every other death (a runtime error, here).
//! `Failure/expected` mirrors it. Driven against the real accessors
//! (`src/kernel/error.rs::eval_failure_actual` / `eval_failure_expected`), through
//! a REAL forked peer's death (never a hand-built Failure value) — the same
//! `:wat::test::spawn-peer` + `recv'` pattern the g2/g3 gates use.
//!
//! MUTATION-PROVEN: making `eval_failure_actual`/`eval_failure_expected` read a
//! stored field that no longer exists (`record_field_by_name(&error, "actual", ..)`
//! → a field name that isn't on ANY registered record) or hard-code `Ok(Value::Option(
//! Arc::new(None)))` unconditionally turns `failure_actual_on_assertion_death_returns_some`
//! RED (see the floor log this excursus's Report cites).

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn option_string(v: &Value) -> Option<String> {
    match v {
        Value::Option(opt) => match &**opt {
            Some(Value::String(s)) => Some((**s).clone()),
            _ => None,
        },
        _ => None,
    }
}

/// Call one of this file's co-located `:ga2::*-report` fns and return its
/// `(actual, expected)` fields.
fn ga2_result(fn_name: &str) -> (Option<String>, Option<String>) {
    let v = call_beside_value(file!(), fn_name)
        .unwrap_or_else(|e| panic!("{fn_name} should run and return its Result value, never raise: {e:?}"));
    let rec = match &v {
        Value::Aggregate(a) => a,
        other => panic!("{fn_name}: expected a :ga2::Result record; got {other:?}"),
    };
    (option_string(&rec.fields[0]), option_string(&rec.fields[1]))
}

#[test]
fn failure_actual_on_assertion_death_returns_some() {
    let (actual, expected) = ga2_result(":ga2::assertion-report");
    assert_eq!(actual.as_deref(), Some("1"), "Failure/actual on an assertion death");
    assert_eq!(expected.as_deref(), Some("2"), "Failure/expected on an assertion death");
}

#[test]
fn failure_actual_on_runtime_error_death_returns_none() {
    let (actual, expected) = ga2_result(":ga2::runtime-error-report");
    assert_eq!(actual, None, "Failure/actual on a runtime-error death");
    assert_eq!(expected, None, "Failure/expected on a runtime-error death");
}
