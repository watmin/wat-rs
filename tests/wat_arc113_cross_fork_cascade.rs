//! Arc 113 slice 3 — cross-fork cascade. Proves the cascade chain
//! preserves AssertionPayload structure (location, actual, expected,
//! frames) across a real fork boundary, via stderr-EDN as the
//! transport.
//!
//! Pattern: outer wat program calls `run-sandboxed-hermetic-ast` on
//! an inner program that triggers `assert-eq`. The inner program
//! runs in a forked OS process; the substrate's catch_unwind
//! captures the panic; `emit_cascade_chain_to_stderr` renders the
//! ProcessDiedError chain to stderr as `#wat.died/chain {...}`. The
//! parent's `drive-hermetic` (in `wat/kernel/hermetic.wat`) calls
//! `extract-died-chain` on stderr-lines; recovery yields the typed
//! Vec<ProcessDiedError>; `failure-from-process-died` walks the
//! head and produces a Failure carrying the original assertion's
//! `actual`/`expected`/`location` — exactly as if the assertion had
//! fired in-process.
//!
//! Symmetry: the slice-2 thread cascade proves the same chain shape
//! reaches the caller through crossbeam channels (zero-copy).
//! Slice 3 proves the same shape reaches the caller through kernel
//! pipes (EDN-serialized). The user-visible Result<R,
//! Vec<*DiedError>> is identical regardless.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Outer uses :my::compute; inner uses canonical nil main.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Arc 170 slice 1f-ζ: append canonical nil-returning `:user::main`.
fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run")
}

#[test]
fn hermetic_assertion_failure_preserves_actual_and_expected() {
    // Inner program: `(assert-eq 1 2)` triggers the structured
    // assertion-failed! panic. Outer reads RunResult.failure and
    // emits the rendered (message, actual, expected) tuple via
    // edn::write so the Rust caller can parse it.
    //
    // Pre-arc-113-slice-3: failure.actual + failure.expected were
    // both :None (the singleton "exited 2" path; the structured
    // AssertionPayload was lost when the child _exit'd).
    //
    // Post-arc-113-slice-3: child writes the chain as EDN to
    // stderr; parent's drive-hermetic reads it back via
    // extract-died-chain; failure-from-process-died walks the
    // head's structured payload; actual = "1", expected = "2".
    //
    // Arc 170 slice 1f-ζ: outer uses :my::compute; inner uses canonical nil main.
    let src = r##"
        (:wat::core::define
          (:my::compute -> :wat::core::Vector<wat::core::String>)
          (:wat::core::let
            [forms
              (:wat::test::program
                (:wat::core::define (:user::main -> :wat::core::nil)
                  (:wat::test::assert-eq 1 2)))
             r
              (:wat::kernel::run-sandboxed-hermetic-ast
                forms
                (:wat::core::Vector :wat::core::String)
                :wat::core::None)
             fail
              (:wat::kernel::RunResult/failure r)
             rendered
              (:wat::core::match fail -> :wat::core::Vector<wat::core::String>
                ((:wat::core::Some f)
                 (:wat::core::Vector :wat::core::String
                   (:wat::kernel::Failure/message f)
                   (:wat::core::match (:wat::kernel::Failure/actual f) -> :wat::core::String
                     ((:wat::core::Some a) a)
                     (:wat::core::None ":None"))
                   (:wat::core::match (:wat::kernel::Failure/expected f) -> :wat::core::String
                     ((:wat::core::Some e) e)
                     (:wat::core::None ":None"))))
                (:wat::core::None
                 (:wat::core::Vector :wat::core::String "NO-FAILURE")))]
            rendered))
    "##;
    let result = run(src);
    let lines: Vec<String> = match result {
        Value::Vec(items) => items
            .iter()
            .map(|v| match v {
                Value::String(s) => (**s).clone(),
                other => panic!("expected String, got {:?}", other),
            })
            .collect(),
        other => panic!("expected Vec<String>, got {:?}", other),
    };
    assert_eq!(
        lines.len(),
        3,
        "expected (message, actual, expected) triple; got {:?}",
        lines
    );
    assert_eq!(lines[0], "assert-eq failed", "message field");
    assert_eq!(lines[1], "1", "actual field — round-trip across fork");
    assert_eq!(lines[2], "2", "expected field — round-trip across fork");
}
