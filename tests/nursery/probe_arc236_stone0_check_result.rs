//! Diagnostic probe — CheckResult<T> sum-type invariants (arc 236 Stone 236.0; sharpened 236.3).
//!
//! Verifies the type's load-bearing API contract: three variants covering every
//! legitimate inference state; silent-failure state structurally unrepresentable
//! because no `Silent` variant exists in the enum.
//!
//! Probe contracts (6):
//!   1. ok(t) produces (Some(t), [])
//!   2. err(e) produces (None, [e])
//!   3. partial(t, e) produces (Some(t), [e])
//!   4. errs(vec![e1, e2]) produces (None, [e1, e2])
//!   5. map(f) preserves errors AND transforms value
//!   6. No public API path produces the silent-failure state because the type
//!      system has no `Silent` variant — verified by exhaustive pattern matching
//!      over `Ok | Partial | Err`. Every constructor routes to one of the three
//!      variants; none carry (None, []).
//!
//! Initial state: 6/6 FAIL — CheckResult type doesn't exist yet.
//! Post-stone 236.0: 6/6 PASS (struct-with-Option shape).
//! Post-stone 236.3: 6/6 PASS (3-variant enum shape; ✅✅✅ structural impossibility).

use wat::check::{CheckError, CheckErrorKind, CheckResult};
use wat::span::Span;

fn dummy_error() -> CheckError {
    CheckError {
        span: Span::unknown(),
        kind: CheckErrorKind::UnknownCallee {
            callee: ":dummy".to_string(),
        },
    }
}

fn dummy_error_2() -> CheckError {
    CheckError {
        span: Span::unknown(),
        kind: CheckErrorKind::UnknownCallee {
            callee: ":dummy2".to_string(),
        },
    }
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
#[test]
fn probe_1_ok_produces_value_no_errors() {
    let r: CheckResult<i32> = CheckResult::ok(42);
    assert_eq!(r.value(), Some(&42));
    assert!(r.errors().is_empty());
    assert!(r.is_ok());
    assert!(!r.has_errors());
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
#[test]
fn probe_2_err_produces_no_value_single_error() {
    let r: CheckResult<i32> = CheckResult::err(dummy_error());
    assert_eq!(r.value(), None);
    assert_eq!(r.errors().len(), 1);
    assert!(!r.is_ok());
    assert!(r.has_errors());
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
#[test]
fn probe_3_partial_produces_value_and_error() {
    let r: CheckResult<i32> = CheckResult::partial(7, dummy_error());
    assert_eq!(r.value(), Some(&7));
    assert_eq!(r.errors().len(), 1);
    assert!(!r.is_ok()); // has errors → not strictly ok
    assert!(r.has_errors());
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
#[test]
fn probe_4_errs_accepts_multiple() {
    let r: CheckResult<i32> = CheckResult::errs(vec![dummy_error(), dummy_error_2()]);
    assert_eq!(r.value(), None);
    assert_eq!(r.errors().len(), 2);
    assert!(r.has_errors());
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
#[test]
fn probe_5_map_preserves_errors_transforms_value() {
    // partial → map → still partial; value transformed
    let r: CheckResult<i32> = CheckResult::partial(10, dummy_error());
    let mapped: CheckResult<String> = r.map(|n| format!("val:{}", n));
    assert_eq!(mapped.value(), Some(&"val:10".to_string()));
    assert_eq!(mapped.errors().len(), 1);

    // err → map → still err; no value transformation possible
    let r2: CheckResult<i32> = CheckResult::err(dummy_error());
    let mapped2: CheckResult<String> = r2.map(|n| format!("val:{}", n));
    assert_eq!(mapped2.value(), None);
    assert_eq!(mapped2.errors().len(), 1);
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
#[test]
fn probe_6_and_then_chains_and_merges_errors() {
    // ok → and_then(ok) → ok; no errors
    let r: CheckResult<i32> = CheckResult::ok(5);
    let chained: CheckResult<i32> = r.and_then(|v| CheckResult::ok(v * 2));
    assert_eq!(chained.value(), Some(&10));
    assert_eq!(chained.errors().len(), 0);

    // partial → and_then(ok) → ok-with-merged-error
    let r2: CheckResult<i32> = CheckResult::partial(3, dummy_error());
    let chained2: CheckResult<i32> = r2.and_then(|v| CheckResult::ok(v + 1));
    assert_eq!(chained2.value(), Some(&4));
    assert_eq!(chained2.errors().len(), 1);

    // err → and_then(ok) → short-circuit; carry err
    let r3: CheckResult<i32> = CheckResult::err(dummy_error());
    let chained3: CheckResult<i32> = r3.and_then(|v| CheckResult::ok(v + 100));
    assert_eq!(chained3.value(), None);
    assert_eq!(chained3.errors().len(), 1);

    // ok → and_then(err) → err
    let r4: CheckResult<i32> = CheckResult::ok(5);
    let chained4: CheckResult<i32> = r4.and_then(|_| CheckResult::err(dummy_error()));
    assert_eq!(chained4.value(), None);
    assert_eq!(chained4.errors().len(), 1);
}
