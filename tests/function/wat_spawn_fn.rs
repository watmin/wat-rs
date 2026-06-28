//! End-to-end tests for `:wat::kernel::spawn-thread` accepting a body
//! whose signature is the mini-TCP contract:
//!   `:Fn(:Receiver<I>, :Sender<O>) -> :()`
//!
//! Arc 114 retired the bare-spawn R-via-join contract; the replacement
//! is `spawn-thread`, which allocates a typed input pipe + output
//! pipe per thread and hands the inside ends to the body. The body
//! reads from `in`, computes, writes to `out`, and returns unit. The
//! parent sends via `Thread/input thr` and recvs via
//! `Thread/output thr`.
//!
//! These tests verify spawn-thread accepts the various function-shape
//! forms that bare-spawn used to accept (named keyword, let-bound
//! fn, inline fn literal, fn-valued param, closure-
//! captured fn) — but the contract under test is the mini-TCP
//! shape: input flows in via the pipe, output flows out via the pipe,
//! never via "return value." `Thread/join-result` confirms the body
//! finished without panic.
//!
//! See `docs/arc/2026/04/114-spawn-as-thread/DESIGN.md` for the
//! contract; `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels"
//! for the principle.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute_tN; world loaded via startup_beside.
//!
//! Wat source: tests/function/wat_spawn_fn.wat (positive, shared world) and
//! tests/function/wat_spawn_fn_not_callable.wat (negative).

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

fn run(compute_fn: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(&format!("({compute_fn})")).expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run").value_owned()
}

// ─── Named-define body — the keyword path ─────────────────────────────

#[test]
fn spawn_thread_named_define_body() {
    // The body is a named define matching the channel-shaped contract.
    // Parent sends 41; worker recvs, increments, writes; parent recvs.
    assert!(matches!(run(":my::compute_t1"), Value::i64(42)));
}

// ─── Inline fn literal body — the anonymous path ─────────────────────

#[test]
fn spawn_thread_inline_fn_body() {
    assert!(matches!(run(":my::compute_t2"), Value::i64(42)));
}

// ─── Closure capture survives spawn-thread ────────────────────────────

#[test]
fn spawn_thread_closure_capture() {
    // delta=100, input=23 → 23+100=123.
    assert!(matches!(run(":my::compute_t3"), Value::i64(123)));
}

// ─── Non-callable body errors at type-check ───────────────────────────

#[test]
fn spawn_thread_rejects_non_callable_body() {
    // 42 is neither a keyword path nor a fn value. The checker's
    // TypeMismatch arm fires because spawn-thread's body parameter
    // expects :Fn(Receiver<I>,Sender<O>) -> :() and i64 doesn't unify.
    match startup_from_file("tests/function/wat_spawn_fn_not_callable.wat") {
        Err(wat::freeze::StartupError::Check(errs)) => {
            let hit = errs.0.iter().any(|e| {
                matches!(
                    e,
                    wat::check::CheckError { kind: wat::check::CheckErrorKind::TypeMismatch { callee, .. }, .. }
                        if callee.contains(":wat::kernel::spawn-thread")
                )
            });
            assert!(hit, "expected spawn-thread TypeMismatch; got {:?}", errs.0);
        }
        Err(other) => panic!("expected Check error; got {:?}", other),
        Ok(_) => panic!("expected check-time failure"),
    }
}
