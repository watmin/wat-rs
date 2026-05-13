//! Arc 170 slice 3 Gap K — spawn-process lockstep verification.
//!
//! Verifies that `run-hermetic` (the spawn-process Layer 1 path) does NOT
//! deadlock after the `run-hermetic-driver` drain-then-join restructure.
//!
//! ## Path exercised
//!
//! Both probes use `:wat::test::run-hermetic` exclusively — the Layer 1
//! spawn-process surface. The macro expands the body into a
//! `(:wat::core::fn [_rx <- Receiver<nil> _tx <- Sender<nil>] -> nil <body>)`
//! form, calls `(:wat::kernel::spawn-process fn)`, then passes the resulting
//! `Process<nil,nil>` to `(:wat::test::run-hermetic-driver proc)`. That
//! driver is the site restructured in Gap K: inner let owns
//! `Process/stdout` and `Process/stderr` Receivers and drains them; outer
//! let calls `Process/join-result` only after the inner scope has exited.
//!
//! ## What is NOT tested here
//!
//! Stdout-capture on the spawn-process path is OUT OF SCOPE. The
//! spawn-process child does not install ThreadIO or the ambient stdio
//! services; `(:wat::kernel::println ...)` would error with
//! `ServiceNotRunning` in a spawn-process body. stdout-capture verification
//! lives in `tests/probe_run_hermetic_ast_stdout_capture.rs` which
//! exercises the fork-program-ast path where ambient stdio IS installed.
//!
//! ## Row C1 verification
//!
//! These two probes prove the lockstep rule holds for the spawn-process path:
//!
//! - Probe 1: empty body returning nil → child exits 0 → `RunResult.failure = None`
//!   and the test completes without hanging (drain-before-join allows clean shutdown).
//!
//! - Probe 2: body calling `assertion-failed!` → child panics → `RunResult.failure = Some(...)`
//!   and the test completes without hanging (drain-before-join drains panic stderr
//!   before join even on the failure path).
//!
//! If the deadlock category were present (join-before-drain), both tests would hang.
//! Completing without hang IS the positive verification of the lockstep fix.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

// ─── Probe 1 — empty body: drain-before-join allows clean child shutdown ───

/// `run-hermetic` with an empty body (just `:wat::core::nil`).
///
/// The child exits 0. Under the old join-before-drain shape this would
/// hang if the child's OS pipes had any content to buffer. Under the
/// corrected drain-before-join shape, the Receivers drop before join is
/// called; the child can exit cleanly; join returns immediately.
///
/// Verifies: `RunResult.failure = None` (clean exit) and test completes
/// (no hang). Path: `:wat::test::run-hermetic` (spawn-process Layer 1).
#[test]
fn probe_run_hermetic_clean_exit_no_deadlock() {
    let src = r#"
        (:wat::core::define (:probe::test::clean-exit -> :wat::kernel::RunResult)
          (:wat::test::run-hermetic
            :wat::core::nil))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let func = world
        .symbols()
        .get(":probe::test::clean-exit")
        .expect(":probe::test::clean-exit defined");
    let result = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::span::Span::unknown(),
    )
    .expect("run-hermetic driver should not itself panic");

    // result is :wat::kernel::RunResult { stdout stderr failure }
    let sv = match &result {
        wat::runtime::Value::Struct(s) if s.type_name == ":wat::kernel::RunResult" => s,
        other => panic!("expected RunResult Struct; got {:?}", other),
    };

    // RunResult field 2 is failure :Option<Failure>; must be None (clean exit).
    let failure_field = &sv.fields[2];
    let is_none = match failure_field {
        wat::runtime::Value::Option(opt) => opt.as_ref().is_none(),
        other => panic!("expected Option failure field; got {:?}", other),
    };
    assert!(
        is_none,
        "expected clean-exit body to produce RunResult with failure=None; got {:?}",
        result
    );
}

// ─── Probe 2 — panicking body: drain-before-join drains stderr before join ─

/// `run-hermetic` with a body that calls `assertion-failed!` (intentional panic).
///
/// The child panics before returning. Under the old join-before-drain shape,
/// the drain threads could block if the panic chain on stderr exceeds pipe
/// buffer capacity; join would block waiting for the child to exit while
/// the drain threads are blocked on send. Under the corrected drain-before-join
/// shape, the Receivers drop before join; the drain threads see EOF and
/// complete; the child can finish writing and exit; join returns.
///
/// Verifies: `RunResult.failure = Some(...)` (structured failure captured)
/// and the test completes without hanging. Path: `:wat::test::run-hermetic`
/// (spawn-process Layer 1).
#[test]
fn probe_run_hermetic_panic_body_no_deadlock() {
    let src = r#"
        (:wat::core::define (:probe::test::intentional-panic -> :wat::kernel::RunResult)
          (:wat::test::run-hermetic
            (:wat::kernel::assertion-failed!
              "intentional panic from probe_run_hermetic_no_deadlock"
              :wat::core::None
              :wat::core::None)))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let func = world
        .symbols()
        .get(":probe::test::intentional-panic")
        .expect(":probe::test::intentional-panic defined");
    let result = wat::runtime::apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::span::Span::unknown(),
    )
    .expect("run-hermetic driver should not itself panic (failure captured as RunResult)");

    // result is :wat::kernel::RunResult { stdout stderr failure }
    let sv = match &result {
        wat::runtime::Value::Struct(s) if s.type_name == ":wat::kernel::RunResult" => s,
        other => panic!("expected RunResult Struct; got {:?}", other),
    };

    // RunResult field 2 is failure :Option<Failure>; must be Some (child panicked).
    let failure_field = &sv.fields[2];
    let failure_val = match failure_field {
        wat::runtime::Value::Option(opt) => match opt.as_ref() {
            Some(v) => v,
            None => panic!(
                "expected panicking body to produce Some(Failure); got None — \
                 child panic may not have been captured (drain-before-join may be broken)"
            ),
        },
        other => panic!("expected Option failure field; got {:?}", other),
    };

    // Failure struct must have the correct type_name.
    let failure_struct = match failure_val {
        wat::runtime::Value::Struct(s) if s.type_name == ":wat::kernel::Failure" => s,
        other => panic!("expected :wat::kernel::Failure struct; got {:?}", other),
    };

    // Failure.message (field 0) must carry a non-empty diagnostic.
    let message = match &failure_struct.fields[0] {
        wat::runtime::Value::String(s) => s.to_string(),
        other => panic!("expected Failure.message :String; got {:?}", other),
    };
    assert!(
        !message.is_empty(),
        "expected non-empty Failure message from intentional panic; got empty string"
    );
}
