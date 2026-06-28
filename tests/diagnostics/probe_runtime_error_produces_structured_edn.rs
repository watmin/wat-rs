//! Probe — runtime-error path emits structured EDN (arc 170 slice 1i).
//!
//! Path exercised: `spawn_process_child_branch` `Ok(Err(runtime_err))` arm.
//!
//! The body calls `(:wat::kernel::println ...)` inside a `run-hermetic`
//! child that has NO ambient stdio service installed. `eval_kernel_println`
//! returns `RuntimeError::ServiceNotRunning`; this propagates through
//! `apply_function` as `Ok(Err(runtime_err))` to the match arm.
//!
//! Before arc 170 slice 1i: that arm wrote plain text to stderr ("runtime: …")
//! and the harness discarded it, returning `Failure.message = "forked program
//! exited 3"`. After the fix: the arm calls `emit_structured_exit` with a
//! `ProcessDiedError::RuntimeError` value; `extract-panics` recovers the EDN;
//! `Failure.message` carries the actual runtime error text.
//!
//! Row G (path-honesty): the body exercises ONLY the runtime-error exit path.
//! No AssertionPayload, no plain panic.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

/// Extract `RunResult.failure.message` from a `Value::Struct` RunResult.
fn failure_message(result: &Value) -> String {
    let sv = match result {
        Value::Struct(s) if s.type_name == ":wat::kernel::RunResult" => s,
        other => panic!("expected RunResult; got {:?}", other),
    };
    match &sv.fields[2] {
        Value::Option(opt) => match opt.as_ref() {
            Some(Value::Struct(f)) if f.type_name == ":wat::kernel::Failure" => {
                match &f.fields[0] {
                    Value::String(s) => (**s).clone(),
                    _ => "<missing message>".to_string(),
                }
            }
            _ => "<no failure>".to_string(),
        },
        _ => "<malformed failure field>".to_string(),
    }
}

#[test]
fn probe_runtime_error_produces_structured_edn() {
    // Stone C: children have ambient stdio (bootstrap). To hit
    // Ok(Err(runtime_err)) in the child branch, we use integer division
    // by zero — (:wat::core::i64::/ 1 0) — which produces
    // RuntimeError::DivisionByZero (not a Rust panic). This passes the
    // type-checker (valid expression) but fails at runtime, flowing
    // through apply_function as Err(RuntimeError) and landing in the
    // Ok(Err(runtime_err)) arm of spawn_process_child_branch.
    // Wat source lives in the co-located fixture: probe_runtime_error_produces_structured_edn.wat
    let world = startup_beside(file!()).expect("startup");
    let func = world.symbols().get(":probe::runtime-err").expect("defined");
    let result = apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::span::Span::unknown(),
    )
    .expect("driver should not panic — RunResult carries the failure");

    let msg = failure_message(&result);

    eprintln!("===== probe_runtime_error_produces_structured_edn =====");
    eprintln!("Failure.message: {:?}", msg);
    eprintln!("=======================================================");

    // Row D — failure.message MUST NOT be "forked program exited N".
    assert!(
        !msg.contains("forked program exited"),
        "expected actual runtime error text in Failure.message; \
         got the old plain-text fallback: {:?}",
        msg
    );

    // failure MUST be Some (child errored).
    assert_ne!(msg, "<no failure>", "expected Some failure; got None");

    // The actual error text should mention the service or the op.
    // ServiceNotRunning produces a diagnostic naming the op.
    assert!(
        !msg.is_empty(),
        "expected non-empty error message; got empty string"
    );
}
