//! Probe — plain Rust panic path emits structured EDN (arc 170 slice 1i).
//!
//! Path exercised: `child_branch_from_source` `Err(panic_payload)` arm
//! where `panic_payload` is NOT an `AssertionPayload`.
//!
//! The only way to trigger a raw Rust `panic!()` from a wat body is via
//! the `:wat::holon::Bundle` capacity-exceeded path with
//! `capacity_mode = :panic`. With `dim_count = 1` the budget is
//! `floor(sqrt(1)) = 1`, so a Bundle with 2 atoms exceeds capacity and
//! calls Rust's `panic!("...: capacity exceeded ...")` — a bare String
//! payload, NOT an AssertionPayload.
//!
//! The probe uses `run-sandboxed` (which goes through fork-program →
//! `child_branch_from_source`) so the inner program can declare its OWN
//! config. The outer program does not need holon config.
//!
//! Before arc 170 slice 1i: the `else` branch of `Err(panic_payload)`
//! did NOT exist — only AssertionPayload was handled; plain panics fell
//! through to `write_direct_to_stderr("panic: …")` which the harness
//! discarded, yielding "forked program exited 1".
//!
//! After the fix: the `else` branch extracts the String payload and calls
//! `emit_structured_exit` with `ProcessDiedError::Panic(msg, None)`.
//!
//! Row G (path-honesty): the inner program exercises ONLY the
//! non-AssertionPayload panic exit path. No assert-eq, no raise!, no
//! RuntimeError.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn freeze_ok(src: &str) -> wat::freeze::FrozenWorld {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(w) => w,
        Err(e) => panic!("freeze should succeed; got: {}", e),
    }
}

/// Extract `RunResult.failure.message` from a `Value::Struct` RunResult.
fn failure_message(v: &Value) -> String {
    let sv = match v {
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
fn probe_plain_panic_produces_structured_edn() {
    // Inner program: dim_count=1 → budget=floor(sqrt(1))=1;
    // a Bundle with 2 atoms exceeds capacity and triggers
    // panic!("...: capacity exceeded ...") — a bare Rust String panic,
    // NOT an AssertionPayload. This is the only reliably reachable
    // non-AssertionPayload panic path from a wat body.
    let inner_src = r#"
(:wat::config::set-dim-count! 1)
(:wat::config::set-capacity-mode! :panic)
(:wat::core::define (:user::main -> :wat::core::nil)
  ;; Two Atom children exceed floor(sqrt(1))=1 budget
  ;; → panic!("capacity exceeded under :panic") fires inside eval_algebra_bundle.
  (:wat::core::let
    [_bundle
      (:wat::holon::Bundle
        (:wat::holon::Atom "key1")
        (:wat::holon::Atom "key2"))]
    :wat::core::nil))
"#;

    // Outer program wraps inner source string in run-sandboxed.
    // Outer does NOT need holon config (it doesn't call Bundle).
    let outer_src = format!(
        r#"
(:wat::core::define (:probe::plain-panic -> :wat::kernel::RunResult)
  (:wat::kernel::run-sandboxed
    {inner_quoted}
    (:wat::core::Vector :wat::core::String)
    :wat::core::None))

(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
"#,
        inner_quoted = {
            // Escape the inner source as a wat String literal.
            let escaped = inner_src.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{}\"", escaped)
        }
    );

    let world = freeze_ok(&outer_src);
    let ast = wat::parse_one!("(:probe::plain-panic)").expect("parse");
    let env = Environment::new();
    let result = eval_in_frozen(&ast, &world, &env).expect("outer should not panic");

    let msg = failure_message(&result);

    eprintln!("===== probe_plain_panic_produces_structured_edn =====");
    eprintln!("Failure.message: {:?}", msg);
    eprintln!("=====================================================");

    // Row E — failure MUST be Some (child panicked).
    assert_ne!(msg, "<no failure>", "expected Some failure; child should have panicked");

    // failure.message MUST NOT be "forked program exited N" (old plain-text fallback).
    assert!(
        !msg.contains("forked program exited"),
        "expected actual panic text in Failure.message; \
         got the old exit-code-only fallback: {:?}",
        msg
    );

    // The structured EDN round-trip should give us the capacity message or a
    // non-empty string.
    assert!(
        !msg.is_empty(),
        "expected non-empty error message; got empty string"
    );
}
