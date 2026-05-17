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

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Asserts `startup_from_source` fails and returns the Debug-formatted
/// error string for further inspection.
fn startup_err(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

/// Asserts `startup_from_source` succeeds (no errors).
fn startup_ok(src: &str) {
    if let Err(e) = startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        panic!("expected startup success; got errors: {:?}", e);
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
    let src = r#"
        (:wat::core::define
          (:my::arc202::negative-no-stdin
            (proc :wat::kernel::Process<wat::core::nil,wat::core::nil>)
            -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::ProcessDiedError>>)
          (:wat::core::let
            [joined (:wat::kernel::Process/join-result proc)]
            joined))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("ProcessJoinHoldsStdinSender"),
        "error should name the new arc202 rule variant; got: {}",
        err
    );
    assert!(
        err.contains("DefRestrictedCallerNotAllowed"),
        "arc198 restriction should also fire for user-namespace Process/join-result; got: {}",
        err
    );
}

// ─── Test 2 — positive: stdlib compiles cleanly after the wat-side fix ────

#[test]
fn process_join_with_stdin_extraction_passes_check() {
    // Every `startup_from_source` loads the full substrate stdlib including
    // `wat/test.wat::run-hermetic-driver`. After the arc 202 wat-side fix
    // (adding `stdin-w` to the inner let of `run-hermetic-driver`), that
    // function satisfies the new rule: `Process/stdin proc` appears in the
    // inner let's scope, so the rule does not fire.
    //
    // A trivial user program proves this: if the stdlib's `run-hermetic-driver`
    // still had the old shape (no `Process/stdin` extraction), startup would
    // fail with `ProcessJoinHoldsStdinSender` on that substrate function.
    // Startup succeeding = the canonical legal shape passes cleanly.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    startup_ok(src);
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
    let src = r#"
        (:wat::core::define
          (:my::arc202::negative-stdin-present
            (proc :wat::kernel::Process<wat::core::nil,wat::core::nil>)
            -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::ProcessDiedError>>)
          (:wat::core::let
            [stdin-w (:wat::kernel::Process/stdin proc)
             joined  (:wat::kernel::Process/join-result proc)]
            joined))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let err = startup_err(src);
    // Arc 202 rule must NOT fire — stdin is present.
    assert!(
        !err.contains("ProcessJoinHoldsStdinSender"),
        "ProcessJoinHoldsStdinSender must NOT fire when Process/stdin is present; got: {}",
        err
    );
    // Arc 198 restriction still fires (user namespace calling restricted verb).
    assert!(
        err.contains("DefRestrictedCallerNotAllowed"),
        "arc198 restriction should fire for user-namespace Process/join-result; got: {}",
        err
    );
}
