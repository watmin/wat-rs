//! Probe — custom panic hook suppresses Rust's default panic output (arc 170 slice 1i).
//!
//! Path exercised: any panic path in `spawn_process_child_branch` where the
//! custom panic hook (installed by `install_silent_panic_hook`) prevents
//! Rust's default handler from writing "thread '...' panicked at ..." and
//! "note: run with RUST_BACKTRACE=1" lines to fd 2.
//!
//! This probe uses the AssertionPayload path (assert-eq with mismatched values)
//! because:
//! 1. It reliably triggers the panic hook.
//! 2. The structured emit (`emit_panics_to_stderr`) is already correct for
//!    AssertionPayload — we are only asserting the ABSENCE of Rust's default
//!    handler output, not the content of the structured EDN.
//!
//! Before arc 170 slice 1i: `probe_runtime_err_stderr_visibility` showed
//! stderr lines [0]-[3] were Rust default handler output:
//!   [1] thread 'probe...' panicked at src/assertion.rs:...
//!   [2] Box<dyn Any>
//!   [3] note: run with `RUST_BACKTRACE=1` to get a backtrace
//!
//! After installing the silent panic hook: fd 2 contains ONLY the structured
//! `#wat.kernel/ProcessPanics` line. The three Rust-default lines are gone.
//!
//! Row G (path-honesty): the body exercises the panic path (not runtime-error
//! or startup-error paths); the assertion verifies ABSENCE of the hook's
//! suppressed output on the SAME panic path.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;
use wat::runtime::{apply_function, Value};

fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

/// Extract the `RunResult.stderr` lines (field index 1).
fn stderr_lines(result: &Value) -> Vec<String> {
    let sv = match result {
        Value::Struct(s) if s.type_name == ":wat::kernel::RunResult" => s,
        other => panic!("expected RunResult; got {:?}", other),
    };
    match &sv.fields[1] {
        Value::Vec(v) => v
            .iter()
            .filter_map(|item| match item {
                Value::String(s) => Some((**s).clone()),
                _ => None,
            })
            .collect(),
        other => panic!("expected Vec for stderr field; got {:?}", other),
    }
}

#[test]
fn probe_no_default_rust_panic_noise_on_stderr() {
    // Body triggers an AssertionPayload panic via assert-eq mismatch.
    // The child's panic hook is installed BEFORE catch_unwind — Rust's
    // default handler (which would write "thread '...' panicked" etc.)
    // is suppressed. Only the structured #wat.kernel/ProcessPanics line
    // should appear on stderr.
    let src = r#"
        (:wat::core::define (:probe::hook-test -> :wat::kernel::RunResult)
          (:wat::test::run-hermetic
            (:wat::test::assert-eq "expected-value" "actual-value")))

        (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
    "#;
    let world = freeze_ok(src);
    let func = world.symbols().get(":probe::hook-test").expect("defined");
    let result = apply_function(
        func.clone(),
        Vec::new(),
        world.symbols(),
        wat::span::Span::unknown(),
    )
    .expect("driver should not panic — RunResult carries the failure");

    let lines = stderr_lines(&result);

    eprintln!("===== probe_no_default_rust_panic_noise_on_stderr =====");
    eprintln!("stderr_lines ({}):", lines.len());
    for (i, line) in lines.iter().enumerate() {
        eprintln!("  [{}] {:?}", i, line);
    }
    eprintln!("=======================================================");

    // Row F — assert NONE of the stderr lines contain Rust default handler text.
    for line in &lines {
        assert!(
            !line.contains("thread '"),
            "Rust default panic handler output found in stderr: {:?}\n\
             Expected: ONLY the structured #wat.kernel/ProcessPanics line.\n\
             Found: {:?}",
            line,
            lines
        );
        assert!(
            !line.contains("note: run with RUST_BACKTRACE"),
            "Rust default panic handler 'RUST_BACKTRACE' hint found in stderr: {:?}\n\
             Expected: ONLY the structured #wat.kernel/ProcessPanics line.\n\
             Found: {:?}",
            line,
            lines
        );
        assert!(
            !line.contains("note: run with `RUST_BACKTRACE"),
            "Rust default panic handler backtrace hint found in stderr: {:?}",
            line
        );
    }

    // Positive assertion: the structured ProcessPanics line IS present.
    let has_structured = lines
        .iter()
        .any(|l| l.trim_start().starts_with("#wat.kernel/ProcessPanics"));
    assert!(
        has_structured,
        "expected a #wat.kernel/ProcessPanics structured line in stderr; \
         got: {:?}",
        lines
    );
}
