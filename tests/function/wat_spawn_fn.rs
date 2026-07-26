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

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

// just-eval (rubric): each `compute_fn` names a zero-arg fn defined in the co-located
// fixture; fetch it from the frozen world and `apply_function` it — no inline wat driver.
fn run(compute_fn: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let func = world
        .symbols()
        .get(compute_fn)
        .unwrap_or_else(|| panic!("no {compute_fn} in fixture"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("compute should run")
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

