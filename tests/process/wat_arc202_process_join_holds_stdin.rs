//! Arc 202 — `ProcessJoinHoldsStdinSender` walker rule.
//!
//! Verifies the freeze-time refusal that fires when a `let` form calls
//! `:wat::kernel::Process/join-result proc` without any preceding
//! `:wat::kernel::Process/stdin proc` extraction in the let's scope tree.
//!
//! ## Namespace consideration
//!
//! Arc 198 slice 2 Stone 3 applies `#[restricted_to(":wat::")]` to
//! `eval_kernel_process_join_result` — `DefRestrictedCallerNotAllowed` also
//! fires for user-namespace callers. The negative tests assert BOTH errors
//! are present (arc 198 restriction AND arc 202 stdin rule). The positive
//! tests use a legal structural shape that satisfies both constraints by
//! indirection through `(:wat::test::run-hermetic ...)` (a macro that
//! internally calls the substrate driver which IS in `:wat::` namespace).
//!
//! ## Tests
//!
//! 1. `process_join_without_stdin_extraction_fails_check` — user-namespace
//!    function with `Process/join-result proc` and NO `Process/stdin proc`
//!    → `ProcessJoinHoldsStdinSender` fires (plus arc 198 restriction).
//! 2. `process_join_with_stdin_extraction_passes_check` — the stdlib
//!    `run-hermetic-driver` is loaded on every `startup_from_source`; after
//!    the arc 202 wat-side fix, the stdlib compiles cleanly → startup_ok.
//! 3. `process_join_with_stdin_present_does_not_fire_stdin_rule` — a
//!    user-namespace function calling both `Process/stdin` and
//!    `Process/join-result` → `ProcessJoinHoldsStdinSender` does NOT appear
//!    (only `DefRestrictedCallerNotAllowed` from arc 198 fires).

use wat::freeze::{startup_bare, startup_from_file};

/// Asserts the given fixture file fails to freeze and returns the Debug-formatted
/// error string for further inspection.
fn startup_err(fixture_rel: &str) -> String {
    match startup_from_file(fixture_rel) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

// ─── Test 1 — negative: join without any stdin extraction fires the rule ─

#[test]
fn process_join_without_stdin_extraction_fails_check() {
    // A user-namespace function calls `Process/join-result proc` inside a let
    // form that never calls `Process/stdin proc`. The child's structural
    // StdInService is blocked on read(fd 0) with no EOF coming — a true
    // deadlock. The new rule must fire with `ProcessJoinHoldsStdinSender`.
    //
    // Note: arc 198's `DefRestrictedCallerNotAllowed` ALSO fires (user namespace
    // calling a substrate-restricted verb). We assert BOTH are present: the
    // restriction confirms arc 198 enforcement is intact; the stdin rule confirms
    // arc 202 detection is additive and independent.
    // Negative fixture: user-namespace fn calls Process/join-result without Process/stdin.
    let err = startup_err("tests/process/wat_arc202_process_join_holds_stdin_no_stdin.wat");
    assert_eq!(
        err,
        "Check(CheckErrors([CheckError { span: Span { file: \"tests/process/wat_arc202_process_join_holds_stdin_no_stdin.wat\", line: 7, col: 14, end_line: 7, end_col: 47 }, kind: DefRestrictedCallerNotAllowed { callee: \":wat::kernel::Process/join-result\", enclosing_fn: \":my::arc202::negative-no-stdin\", prefixes: [\":wat::\"] } }, CheckError { span: Span { file: \"tests/process/wat_arc202_process_join_holds_stdin_no_stdin.wat\", line: 7, col: 13, end_line: 7, end_col: 53 }, kind: ProcessJoinHoldsStdinSender { process_identifier: \"proc\", stdin_sender_span: Span { file: \"tests/process/wat_arc202_process_join_holds_stdin_no_stdin.wat\", line: 7, col: 13, end_line: 7, end_col: 53 } } }]))",
        "error must match golden (arc202 ProcessJoinHoldsStdinSender + arc198 DefRestrictedCallerNotAllowed)"
    );
}

// ─── Test 2 — positive: stdlib compiles cleanly after the wat-side fix ────

#[test]
fn process_join_with_stdin_extraction_passes_check() {
    // Every startup loads the full substrate stdlib including
    // `wat/test.wat::run-hermetic-driver`. After the arc 202 wat-side fix
    // (adding `stdin-w` to the inner let of `run-hermetic-driver`), that
    // function satisfies the new rule: `Process/stdin proc` appears in the
    // inner let's scope, so the rule does not fire.
    //
    // A bare startup (stdlib only) proves this: if the stdlib's `run-hermetic-driver`
    // still had the old shape (no `Process/stdin` extraction), startup would
    // fail with `ProcessJoinHoldsStdinSender` on that substrate function.
    // Startup succeeding = the canonical legal shape passes cleanly.
    startup_bare().expect("expected startup success; stdlib must satisfy arc202 rule");
}

// ─── Test 3 — negative with stdin present: stdin rule does NOT fire ────────

#[test]
fn process_join_with_stdin_present_does_not_fire_stdin_rule() {
    // A user-namespace function calls BOTH `Process/stdin proc` AND
    // `Process/join-result proc` in the same let scope. The v1 absence-only
    // detection sees `Process/stdin` is present → `ProcessJoinHoldsStdinSender`
    // does NOT fire. Only `DefRestrictedCallerNotAllowed` fires (arc 198
    // restriction on user-namespace callers).
    //
    // This proves the rule correctly distinguishes absent-stdin (deadlock) from
    // present-stdin (either legal or a different shape the rule defers on).
    // Negative fixture: user-namespace fn calls both Process/stdin AND Process/join-result.
    let err = startup_err("tests/process/wat_arc202_process_join_holds_stdin_with_stdin.wat");
    assert_eq!(
        err,
        "Check(CheckErrors([CheckError { span: Span { file: \"tests/process/wat_arc202_process_join_holds_stdin_with_stdin.wat\", line: 9, col: 15, end_line: 9, end_col: 48 }, kind: DefRestrictedCallerNotAllowed { callee: \":wat::kernel::Process/join-result\", enclosing_fn: \":my::arc202::negative-stdin-present\", prefixes: [\":wat::\"] } }]))",
        "error must match golden (arc198 only; arc202 must NOT fire when stdin is present)"
    );
}
