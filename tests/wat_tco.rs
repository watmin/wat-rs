//! End-to-end tests for the Stage 1 TCO arc — tail-call optimization
//! for `define`-registered functions.
//!
//! Mechanism: when `eval_tail` recognizes a user-defined function call
//! in tail position (the `:wat::core::if` then/else branches, the
//! `:wat::core::match` arm bodies, the `:wat::core::let` / `let*`
//! body) it emits `RuntimeError::TailCall` instead of recursing
//! through `apply_function`. `apply_function`'s trampoline loop
//! reassigns `cur_func`/`cur_args` and re-iterates. Rust stack stays
//! constant across arbitrary tail-recursion depth.
//!
//! Stage 1 scope: named defines (`sym.functions`). Lambda-valued
//! tail calls land in Stage 2; this file includes a negative-space
//! note on that boundary.
//!
//! Coverage:
//!
//! - Self-recursion through `if` at high depth (would overflow without
//!   TCO) returns the correct value.
//! - Self-recursion through `match` (Console/loop-shape — Option arms)
//!   at high depth succeeds.
//! - Mutual recursion between two named defines at high depth.
//! - Tail call nested inside a `let*` body (let* is tail-carrying).
//! - Non-tail recursion still produces the correct result at modest
//!   depth (confirms the TCO doesn't accidentally optimize non-tail
//!   calls).
//! - `try` and `TailCall` coexist: a function that tail-recurses in
//!   its happy path and short-circuits with `try` on the error path
//!   behaves correctly on both.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source, StartupError};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn startup(src: &str) -> Result<wat::freeze::FrozenWorld, StartupError> {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
}

fn run(src: &str) -> Value {
    let world = startup(src).expect("startup should succeed");
    invoke_user_main(&world, Vec::new()).expect("main should run")
}

// ─── Self-recursion via if ────────────────────────────────────────────

#[test]
fn self_recursion_via_if_at_million_depth() {
    // The canonical TCO benchmark from the arc 003 design doc. Without
    // TCO this overflows the default 8MB thread stack well before 1M
    // frames (a fresh apply_function + eval frame per iteration). With
    // TCO the loop in apply_function reuses one frame the entire way.
    let src = r#"

        (:wat::core::define (:app::countdown (n :wat::core::i64) (acc :wat::core::i64) -> :wat::core::i64)
          (:wat::core::if (:wat::core::= n 0) -> :wat::core::i64
            acc
            (:app::countdown (:wat::core::i64::- n 1) (:wat::core::i64::+ acc 1))))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:app::countdown 1000000 0))
    "#;
    assert!(matches!(run(src), Value::i64(1_000_000)));
}

// ─── Self-recursion via match (the Console/loop shape) ────────────────

#[test]
fn self_recursion_via_match_at_high_depth() {
    // Models `:wat::std::service::Console/loop`: match an Option, in
    // the Some arm do work and recurse tail; in the None arm exit. The
    // forcing-function case the user named. Uses :wat::std::list::take
    // to hand back Option<i64> values from a Vec.
    //
    // 100k iterations — well past any default stack without TCO.
    let src = r#"

        (:wat::core::define (:app::drain (remaining :wat::core::i64) (acc :wat::core::i64) -> :wat::core::i64)
          (:wat::core::match
            (:wat::core::if (:wat::core::> remaining 0) -> :wat::core::Option<wat::core::i64>
              (:wat::core::Some remaining)
              :wat::core::None)
            -> :wat::core::i64
            ((:wat::core::Some v)
              (:app::drain (:wat::core::i64::- v 1) (:wat::core::i64::+ acc 1)))
            (:wat::core::None acc)))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:app::drain 100000 0))
    "#;
    assert!(matches!(run(src), Value::i64(100_000)));
}

// ─── Mutual recursion ─────────────────────────────────────────────────

#[test]
fn mutual_recursion_between_two_defines() {
    // A tail-calls B, B tail-calls A, both named defines. Should
    // alternate through apply_function's trampoline; Rust stack
    // constant. 100k each way = 200k tail calls total.
    let src = r#"

        (:wat::core::define (:app::is-even (n :wat::core::i64) -> :wat::core::bool)
          (:wat::core::if (:wat::core::= n 0) -> :wat::core::bool
            true
            (:app::is-odd (:wat::core::i64::- n 1))))

        (:wat::core::define (:app::is-odd (n :wat::core::i64) -> :wat::core::bool)
          (:wat::core::if (:wat::core::= n 0) -> :wat::core::bool
            false
            (:app::is-even (:wat::core::i64::- n 1))))

        (:wat::core::define (:user::main -> :wat::core::bool)
          (:app::is-even 100000))
    "#;
    assert!(matches!(run(src), Value::bool(true)));
}

// ─── Tail call through let* body ──────────────────────────────────────

#[test]
fn tail_call_inside_let_star_body_propagates() {
    // The `let*` body is the form's tail position — a call there
    // should trigger TCO. Structured to also validate that the let*
    // bindings are themselves NOT in tail position (their RHS runs
    // through plain eval).
    let src = r#"

        (:wat::core::define (:app::loop (n :wat::core::i64) -> :wat::core::i64)
          (:wat::core::let*
            (((next :wat::core::i64) (:wat::core::i64::- n 1)))
            (:wat::core::if (:wat::core::<= n 0) -> :wat::core::i64
              0
              (:app::loop next))))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:app::loop 100000))
    "#;
    assert!(matches!(run(src), Value::i64(0)));
}

// ─── Non-tail recursion still produces correct result ─────────────────

#[test]
fn non_tail_recursion_modest_depth_correct() {
    // `(* n (recurse ...))` — the recursive call is NOT tail because
    // the multiplication has to wait for the result. This still runs
    // through eval (not eval_tail at that sub-position) and uses Rust
    // stack. Modest depth confirms the value is computed correctly
    // AND that we didn't accidentally optimize the non-tail case.
    //
    // 20 iterations = 2^20 = 1048576. Well within default stack and
    // i64 range.
    let src = r#"

        (:wat::core::define (:app::pow2 (n :wat::core::i64) -> :wat::core::i64)
          (:wat::core::if (:wat::core::= n 0) -> :wat::core::i64
            1
            (:wat::core::i64::* 2 (:app::pow2 (:wat::core::i64::- n 1)))))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:app::pow2 20))
    "#;
    assert!(matches!(run(src), Value::i64(1_048_576)));
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
    let src = r#"

        (:wat::core::define (:app::check (n :wat::core::i64) -> :wat::core::Result<wat::core::i64,wat::core::String>)
          (:wat::core::if (:wat::core::< n 0) -> :wat::core::Result<wat::core::i64,wat::core::String>
            (:wat::core::Err "negative")
            (:wat::core::Ok n)))

        (:wat::core::define (:app::loop (n :wat::core::i64) -> :wat::core::Result<wat::core::i64,wat::core::String>)
          (:wat::core::let*
            (((valid :wat::core::i64) (:wat::core::try (:app::check n))))
            (:wat::core::if (:wat::core::= valid 0) -> :wat::core::Result<wat::core::i64,wat::core::String>
              (:wat::core::Ok 0)
              (:app::loop (:wat::core::i64::- valid 1)))))

        (:wat::core::define (:user::main -> :wat::core::Result<wat::core::i64,wat::core::String>)
          (:app::loop 50000))
    "#;
    match run(src) {
        Value::Result(r) => match &*r {
            Ok(Value::i64(0)) => {}
            other => panic!("expected Ok(0); got {:?}", other),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

#[test]
fn try_inside_tail_recursive_function_propagates_err() {
    let src = r#"

        (:wat::core::define (:app::check (n :wat::core::i64) -> :wat::core::Result<wat::core::i64,wat::core::String>)
          (:wat::core::if (:wat::core::< n 0) -> :wat::core::Result<wat::core::i64,wat::core::String>
            (:wat::core::Err "negative")
            (:wat::core::Ok n)))

        (:wat::core::define (:app::loop (n :wat::core::i64) -> :wat::core::Result<wat::core::i64,wat::core::String>)
          (:wat::core::let*
            (((valid :wat::core::i64) (:wat::core::try (:app::check n))))
            (:wat::core::if (:wat::core::<= valid (:wat::core::i64::- 0 1)) -> :wat::core::Result<wat::core::i64,wat::core::String>
              (:wat::core::Ok 0)
              (:app::loop (:wat::core::i64::- valid 1)))))

        ;; Start at -1 so `check` immediately returns Err and `try`
        ;; propagates.
        (:wat::core::define (:user::main -> :wat::core::Result<wat::core::i64,wat::core::String>)
          (:app::loop -1))
    "#;
    match run(src) {
        Value::Result(r) => match &*r {
            Err(Value::String(s)) => assert_eq!(&**s, "negative"),
            other => panic!("expected Err(\"negative\"); got {:?}", other),
        },
        other => panic!("expected Result; got {:?}", other),
    }
}

// ─── Stage 2: lambda-valued tail calls ────────────────────────────────

#[test]
fn lambda_tail_call_via_let_bound_symbol() {
    // Stage 2 detection path 1: bare-symbol head in tail position
    // resolves to a lambda value in env. `f` is let-bound; calling
    // `(f 42)` at main's tail fires eval_tail's env.lookup lambda
    // check, emits TailCall, trampoline runs the lambda body.
    //
    // Single depth — proves the detection path, not the depth. The
    // million-depth case comes via mutual alternation below.
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let*
            (((f :fn(wat::core::i64)->wat::core::i64)
              (:wat::core::lambda ((n :wat::core::i64) -> :wat::core::i64)
                (:wat::core::if (:wat::core::= n 0) -> :wat::core::i64 0 n))))
            (f 42)))
    "#;
    assert!(matches!(run(src), Value::i64(42)));
}

#[test]
fn inline_lambda_literal_tail_call() {
    // Stage 2 detection path 2: the head is itself a list
    // `(lambda ...)`. Evaluated non-tail; the resulting lambda value
    // triggers a TailCall emission from the List head arm.
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::i64)
          ((:wat::core::lambda ((n :wat::core::i64) -> :wat::core::i64)
             (:wat::core::i64::* n 2))
           21))
    "#;
    assert!(matches!(run(src), Value::i64(42)));
}

#[test]
fn named_define_tail_calls_lambda_param() {
    // `:app::invoke`'s body is `(f n)` — a bare-symbol tail call
    // where `f` is a parameter whose value is a lambda. Stage 2
    // detects via env.lookup and TailCall fires with the lambda's
    // Arc<Function>.
    let src = r#"

        (:wat::core::define (:app::invoke
                             (f :fn(wat::core::i64)->wat::core::i64)
                             (n :wat::core::i64)
                             -> :wat::core::i64)
          (f n))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let*
            (((double :fn(wat::core::i64)->wat::core::i64)
              (:wat::core::lambda ((x :wat::core::i64) -> :wat::core::i64)
                (:wat::core::i64::* x 2))))
            (:app::invoke double 21)))
    "#;
    assert!(matches!(run(src), Value::i64(42)));
}

#[test]
fn inline_lambda_named_alternation_at_high_depth() {
    // The high-depth test that requires BOTH stages. `:app::go`
    // (named) recursion is Stage 1 TCO; each call creates a FRESH
    // inline lambda literal in tail position and invokes it
    // `((:wat::core::lambda ...) state n)` — Stage 2 TCO on the
    // List-head path. The lambda body, running inside the
    // trampoline's next iteration, tail-calls go again (Stage 1).
    //
    // Without Stage 2, the inline-lambda tail call burns one Rust
    // frame per iteration — overflows well before 100k. Constant
    // stack at 100k proves Stage 2 detection fires on the
    // inline-lambda-literal head.
    //
    // (The lambda is re-constructed each iteration — that's heap
    // allocation, not stack. The test doesn't care about allocation
    // rate; it cares that stack stays flat.)
    let src = r#"

        (:wat::core::define (:app::go (state :wat::core::i64) (n :wat::core::i64) -> :wat::core::i64)
          (:wat::core::if (:wat::core::= n 0) -> :wat::core::i64
            state
            ((:wat::core::lambda ((s :wat::core::i64) (k :wat::core::i64) -> :wat::core::i64)
               (:app::go (:wat::core::i64::+ s 1) (:wat::core::i64::- k 1)))
             state n)))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:app::go 0 100000))
    "#;
    assert!(matches!(run(src), Value::i64(100_000)));
}

// ─── What Stage 2 does NOT do ─────────────────────────────────────────

// Mutual recursion between two let-bound LAMBDAS (lambda A tail-calls
// lambda B, lambda B tail-calls lambda A, both bound in the same
// `let*` block) requires letrec-style binding — each lambda's closure
// must see the other name. wat's `let*` evaluates RHSes sequentially
// in the prefix scope; a lambda bound first can't close over a name
// bound later, and the reverse direction can only reach backward.
// No test here because the language doesn't offer the binding form.
// Mutual recursion across NAMED defines works (see
// `mutual_recursion_between_two_defines` above) because the static
// symbol table serves as the letrec env.
