//! Probe — does the test harness drop stderr-lines when the child runtime-errors?
//!
//! Read confirmed (wat/test.wat:530-540): when extract-panics returns None,
//! the harness's match fallback uses ONLY join-result's exit-code chain,
//! discarding the drained stderr-lines field. This probe verifies
//! empirically by:
//!
//! 1. Building a run-hermetic with a body that should trigger a runtime
//!    error (not a panic). Calling an undefined function is the cleanest
//!    way: the child boots fine, but the body's resolved-but-unbound call
//!    hits a RuntimeError, which substrate writes as "runtime: {:?}" to
//!    fd 2 before exiting EXIT_RUNTIME_ERROR (3).
//! 2. Reading RunResult.stderr (the drained stderr-lines Vec).
//! 3. Reading RunResult.failure (which should be Some Failure with
//!    "exited 3" message — confirming the lossiness).
//!
//! The probe DOESN'T assert behavior change — it surfaces the current
//! state for the substrate-as-teacher gap analysis. If RunResult.stderr
//! contains useful diagnostic text but RunResult.failure.message is just
//! "exited 3", the test infra is throwing diagnostics away. That's the
//! foundational flaw to fix.

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

/// Run a body that triggers a runtime error in the spawn-process child.
/// Surface the full RunResult — both `stderr` field and `failure` field —
/// so we can see what gets dropped.
#[test]
fn probe_runtime_err_stderr_visibility() {
    // Body that runtime-errors: calls assert-eq with mismatched values.
    // This goes through the assertion-failed! path which IS structured
    // (we should see the cascade). Use this as the CONTROL: structured
    // path should populate stderr-chain properly.
    // Wat source lives in the co-located fixture: probe_runtime_err_stderr_visibility.wat
    let world = startup_beside(file!()).expect("startup");
    let func = world.symbols().get(":probe::structured").expect("defined");
    let result = apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::span::Span::unknown(),
    )
    .expect("driver should not panic");

    let sv = match &result {
        Value::Aggregate(s) if s.holder == wat::Holder::Struct && s.class == "wat::kernel::RunResult" => s,
        other => panic!("expected RunResult; got {:?}", other),
    };

    // stderr field (Vec<String>)
    let stderr_lines = match &sv.fields[1] {
        Value::Vec(v) => v.as_ref().clone(),
        other => panic!("expected stderr Vec; got {:?}", other),
    };

    // failure field (Option<Failure>)
    let failure_message = match &sv.fields[2] {
        Value::Option(opt) => match opt.as_ref() {
            Some(Value::Aggregate(f)) if f.holder == wat::Holder::Struct && f.class == "wat::kernel::Failure" => {
                match &f.fields[0] {
                    Value::String(s) => (**s).clone(),
                    _ => "<missing>".to_string(),
                }
            }
            _ => "<no failure>".to_string(),
        },
        _ => "<malformed>".to_string(),
    };

    eprintln!("===== probe_runtime_err_stderr_visibility =====");
    eprintln!("stderr_lines ({}):", stderr_lines.len());
    for (i, line) in stderr_lines.iter().enumerate() {
        if let Value::String(s) = line {
            eprintln!("  [{}] {}", i, s);
        }
    }
    eprintln!("failure.message: {:?}", failure_message);
    eprintln!("================================================");

    // Probe PASSES (just surfaces data). Don't gate on assertions.
}
