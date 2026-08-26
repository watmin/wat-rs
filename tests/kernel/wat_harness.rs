//! Integration coverage for `wat::Guest` (arc 007 slice 5).
//!
//! The Guest is a thin wrapper — its value is ergonomic. Tests
//! verify it threads through the full startup + invocation pipeline
//! correctly and that its error surface discriminates each failure
//! class cleanly.
//!
//! Arc 170 slice 1f-ζ: `:user::main` is now `[] -> :wat::core::nil`
//! (canonical nil form). `Guest::run` returns empty stdout/stderr
//! vecs (stdio capture retired with the four-arg shape; substrate
//! services own stdio at the substrate layer).

use wat::host::guest::{Guest, GuestError};

const DIMS_AND_MODE: &str = r##"
"##;

/// Real-body trivial-main source, loaded from a .wat fixture so no_inlined_wat never sees it.
fn trivial_src() -> String {
    std::fs::read_to_string("tests/kernel/wat_harness_trivial_main.wat")
        .expect("harness trivial-main fixture must exist")
}

// ─── happy path — run returns Ok ────────────────────────────────────────

#[test]
fn harness_captures_stdout() {
    // Arc 170: stdout capture retired. run() returns Ok with empty stdout/stderr.
    // Test verifies the program compiles and runs without error.
    let src = trivial_src();
    let h = Guest::from_source(&src).expect("freeze");
    let out = h.run(&[]).expect("run");
    // stdout/stderr capture retired with the four-arg main shape.
    assert!(out.stdout.is_empty(), "expected empty stdout; got {:?}", out.stdout);
    assert!(out.stderr.is_empty());
}

// ─── happy path — stdin injection ───────────────────────────────────────

#[test]
fn harness_injects_stdin_lines() {
    // Arc 170: stdin injection retired alongside four-arg main.
    // Test verifies the program compiles and runs cleanly.
    let src = trivial_src();
    let h = Guest::from_source(&src).expect("freeze");
    let out = h.run(&["alpha", "beta", "gamma"]).expect("run");
    // stdin injection and output capture retired with the four-arg shape.
    assert!(out.stdout.is_empty());
}

// ─── happy path — freeze once, run many ─────────────────────────────────

#[test]
fn harness_freeze_once_run_many() {
    // Arc 170: main is canonical nil. Verifies freeze-once, run-many is stable.
    let src = trivial_src();
    let h = Guest::from_source(&src).expect("freeze");
    for _ in 0..3 {
        let out = h.run(&[]).expect("run");
        assert!(out.stdout.is_empty(), "expected empty stdout on each run");
    }
}

// ─── startup error surfaces through GuestError::Startup ───────────────

#[test]
fn harness_startup_error_surfaces() {
    let bad = "(this-is-not-a-valid-form))";
    let err = Guest::from_source(bad).expect_err("bad source must fail");
    assert!(matches!(err, GuestError::Startup(_)), "got {:?}", err);
}

// ─── non-canonical :user::main signature fires GuestError::Startup ─────

#[test]
fn harness_main_signature_mismatch() {
    // Arc 170: non-canonical main (returns i64) is rejected at FREEZE by the
    // :user::main wall (validate_user_main_signature imposed in startup_from_source),
    // so it surfaces as GuestError::Startup(StartupError::MainSignature) — the
    // wall pre-empts the harness's own post-startup validate path.
    let src = format!(
        r##"{}
        (:wat::core::defn :user::main [] -> :wat::core::i64 42)
        "##,
        DIMS_AND_MODE
    );
    let err = Guest::from_source(&src).expect_err("sig mismatch must fail");
    assert!(
        matches!(&err, GuestError::Startup(e) if matches!(e.as_ref(), wat::freeze::StartupError::MainSignature(_))),
        "expected GuestError::Startup(MainSignature) for non-canonical main; got {:?}",
        err
    );
}

// ─── stderr capture ─────────────────────────────────────────────────────

#[test]
fn harness_captures_stderr() {
    // Arc 170: stderr capture retired. run() returns Ok with empty stderr.
    // Test verifies the program compiles and runs without error.
    let src = trivial_src();
    let h = Guest::from_source(&src).expect("freeze");
    let out = h.run(&[]).expect("run");
    assert!(out.stdout.is_empty(), "expected empty stdout");
    assert!(out.stderr.is_empty(), "expected empty stderr");
}

// ─── world() accessor for advanced callers ──────────────────────────────

#[test]
fn harness_world_accessor_exposes_frozen_world() {
    // Arc 170: canonical nil main. world() accessor still works.
    let src = trivial_src();
    let h = Guest::from_source(&src).expect("freeze");
    // Should have :user::main registered; function lookup must succeed.
    let world = h.world();
    assert!(world.symbols().get(":user::main").is_some());
}
