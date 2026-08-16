//! End-to-end tests for the Stage 1 TCO arc — tail-call optimization
//! for `define`-registered functions.
//!
//! Mechanism: when `eval_tail` recognizes a user-defined function call
//! in tail position (the `:wat::core::if` then/else branches, the
//! `:wat::core::match` arm bodies, the `:wat::core::let`
//! body) it emits `RuntimeError::TailCall` instead of recursing
//! through `apply_function`. `apply_function`'s trampoline loop
//! reassigns `cur_func`/`cur_args` and re-iterates. Rust stack stays
//! constant across arbitrary tail-recursion depth.
//!
//! Stage 1 scope: named defines (`sym.functions`). Fn-valued
//! tail calls land in Stage 2; this file includes a negative-space
//! note on that boundary.
//!
//! Coverage:
//!
//! - Self-recursion through `if` at high depth (would overflow without
//!   TCO) returns the correct value.
//! - Self-recursion through `match` (driver-loop-shape — Option arms)
//!   at high depth succeeds.
//! - Mutual recursion between two named defines at high depth.
//! - Tail call nested inside a `let` body (let is tail-carrying).
//! - Non-tail recursion still produces the correct result at modest
//!   depth (confirms the TCO doesn't accidentally optimize non-tail
//!   calls).
//! - `try` and `TailCall` coexist: a function that tail-recurses in
//!   its happy path and short-circuits with `try` on the error path
//!   behaves correctly on both.
//!
//! Wat source: tests/function/tco.wat (shared world via startup_beside).

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

// ─── Self-recursion via if ────────────────────────────────────────────

// 296 Stone K, move 3 — BOUNDARY PROBE: million-depth self-recursion. It asserts,
// it can fail, it belongs in the suite; it is excluded from the default floor on
// RUNTIME (it's genuinely slow, not blocked/broken — it passes). That is a policy
// about WHEN to run it, so it lives in the runner (`.config/nextest.toml`'s
// `default-filter`), not hidden in an attribute on this one test. It runs — and
// passes — under `--profile slow`.
//
//   cargo nextest run --release --profile slow -E 'test(self_recursion_via_if_at_million_depth)'
#[test]
fn self_recursion_via_if_at_million_depth() {
    // The canonical TCO benchmark from the arc 003 design doc. Without
    // TCO this overflows the default 8MB thread stack well before 1M
    // frames (a fresh apply_function + eval frame per iteration). With
    // TCO the loop in apply_function reuses one frame the entire way.
    assert!(matches!(run(":user::compute_t1"), Value::i64(1_000_000)));
}

// ─── Self-recursion via match (driver-loop shape) ─────────────────────

#[test]
fn self_recursion_via_match_at_high_depth() {
    // Models a driver loop: match an Option, in
    // the Some arm do work and recurse tail; in the None arm exit.
    // 100k iterations — well past any default stack without TCO.
    assert!(matches!(run(":user::compute_t2"), Value::i64(100_000)));
}

// ─── Mutual recursion ─────────────────────────────────────────────────

#[test]
fn mutual_recursion_between_two_defines() {
    // A tail-calls B, B tail-calls A, both named defines. Should
    // alternate through apply_function's trampoline; Rust stack
    // constant. 100k each way = 200k tail calls total.
    assert!(matches!(run(":user::compute_t3"), Value::bool(true)));
}

// ─── Tail call through let body ──────────────────────────────────────

#[test]
fn tail_call_inside_let_body_propagates() {
    // The `let` body is the form's tail position — a call there
    // should trigger TCO. Structured to also validate that the let
    // bindings are themselves NOT in tail position (their RHS runs
    // through plain eval).
    assert!(matches!(run(":user::compute_t4"), Value::i64(0)));
}

// ─── Non-tail recursion still produces correct result ─────────────────

#[test]
fn non_tail_recursion_modest_depth_correct() {
    // `(* 2 (recurse ...))` — the recursive call is NOT tail because
    // the multiplication has to wait for the result. This still runs
    // through eval (not eval_tail at that sub-position) and uses Rust
    // stack. Modest depth confirms the value is computed correctly
    // AND that we didn't accidentally optimize the non-tail case.
    //
    // 20 iterations = 2^20 = 1048576. Well within default stack and
    // i64 range.
    assert!(matches!(run(":user::compute_t5"), Value::i64(1_048_576)));
}

// ─── try + TailCall coexistence ───────────────────────────────────────

#[test]
fn try_inside_tail_recursive_function_short_circuits() {
    // A Result-returning tail-recursive function: happy path tail-
    // recurses; error path uses `try` to short-circuit. Both signals
    // (TailCall and TryPropagate) are internal variants of
    // RuntimeError caught at apply_function's loop; verify they don't
    // interfere with each other.
    //
    // The function walks a count down; if the argument goes negative,
    // the `check` helper returns Err and `try` propagates.
    match run(":user::compute_t6") {
        Value::Result(r) => match &*r {
            Ok(Value::i64(0)) => {}
            other => panic!("expected Ok(0); got {:?}", other),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

#[test]
fn try_inside_tail_recursive_function_propagates_err() {
    match run(":user::compute_t7") {
        Value::Result(r) => match &*r {
            Err(Value::String(s)) => assert_eq!(&**s, "negative"),
            other => panic!("expected Err(\"negative\"); got {:?}", other),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

// ─── Stage 2: fn-valued tail calls ────────────────────────────────

#[test]
fn fn_tail_call_via_let_bound_symbol() {
    // Stage 2 detection path 1: bare-symbol head in tail position
    // resolves to a fn value in env. `f` is let-bound; calling
    // `(f 42)` at main's tail fires eval_tail's env.lookup fn
    // check, emits TailCall, trampoline runs the fn body.
    //
    // Single depth — proves the detection path, not the depth.
    assert!(matches!(run(":user::compute_t8"), Value::i64(42)));
}

#[test]
fn inline_fn_literal_tail_call() {
    // Stage 2 detection path 2: the head is itself a list
    // `(fn ...)`. Evaluated non-tail; the resulting fn value
    // triggers a TailCall emission from the List head arm.
    assert!(matches!(run(":user::compute_t9"), Value::i64(42)));
}

#[test]
fn named_define_tail_calls_fn_param() {
    // `:app::invoke`'s body is `(f n)` — a bare-symbol tail call
    // where `f` is a parameter whose value is a fn. Stage 2
    // detects via env.lookup and TailCall fires with the fn's
    // Arc<Function>.
    assert!(matches!(run(":user::compute_t10"), Value::i64(42)));
}

#[test]
fn inline_fn_named_alternation_at_high_depth() {
    // The high-depth test that requires BOTH stages. `:app::go`
    // (named) recursion is Stage 1 TCO; each call creates a FRESH
    // inline fn literal in tail position and invokes it
    // `((:wat::core::fn ...) state n)` — Stage 2 TCO on the
    // List-head path. The fn body, running inside the
    // trampoline's next iteration, tail-calls go again (Stage 1).
    //
    // Without Stage 2, the inline-fn tail call burns one Rust
    // frame per iteration — overflows well before 100k. Constant
    // stack at 100k proves Stage 2 detection fires on the
    // inline-fn-literal head.
    assert!(matches!(run(":user::compute_t11"), Value::i64(100_000)));
}

// ─── What Stage 2 does NOT do ─────────────────────────────────────────

// Mutual recursion between two let-bound Fns (fn A tail-calls
// fn B, fn B tail-calls fn A, both bound in the same
// `let` block) requires letrec-style binding — each fn's closure
// must see the other name. wat's `let` evaluates RHSes sequentially
// in the prefix scope; a fn bound first can't close over a name
// bound later, and the reverse direction can only reach backward.
// No test here because the language doesn't offer the binding form.
// Mutual recursion across NAMED defines works (see
// `mutual_recursion_between_two_defines` above) because the static
// symbol table serves as the letrec env.
