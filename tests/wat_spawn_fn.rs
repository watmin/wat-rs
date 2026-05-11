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
//! Computation moved to :my::compute; canonical nil main appended.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source, StartupError};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn startup(src: &str) -> Result<wat::freeze::FrozenWorld, StartupError> {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
}

/// Arc 170 slice 1f-ζ: append canonical nil-returning `:user::main`.
fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup(&src).expect("startup should succeed");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run")
}

// ─── Named-define body — the keyword path ─────────────────────────────

#[test]
fn spawn_thread_named_define_body() {
    // The body is a named define matching the channel-shaped contract.
    // Parent sends 41; worker recvs, increments, writes; parent recvs.
    // Arc 170 slice 1f-ζ: computation in :my::compute.
    let src = r#"

        (:wat::core::define
          (:app::increment
            (in  :rust::crossbeam_channel::Receiver<wat::core::i64>)
            (out :rust::crossbeam_channel::Sender<wat::core::i64>)
            -> :wat::core::nil)
          (:wat::core::let
            [value
              (:wat::core::match (:wat::kernel::recv in)
                -> :wat::core::i64
                ((:wat::core::Ok (:wat::core::Some n)) n)
                ((:wat::core::Ok :wat::core::None)
                 (:wat::kernel::raise! (:wat::holon::leaf "input closed")))
                ((:wat::core::Err _)
                 (:wat::kernel::raise! (:wat::holon::leaf "parent died"))))
             sum (:wat::core::i64::+'2 value 1)]
            (:wat::core::match (:wat::kernel::send out sum)
              -> :wat::core::nil
              ((:wat::core::Ok _) ())
              ((:wat::core::Err _)
               (:wat::kernel::raise! (:wat::holon::leaf "output closed"))))))

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::let
            [thr
              (:wat::kernel::spawn-thread :app::increment)
             tx
              (:wat::kernel::Thread/input thr)
             rx
              (:wat::kernel::Thread/output thr)
             _ack
              (:wat::core::match (:wat::kernel::send tx 41)
                -> :wat::core::nil
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _) (:wat::kernel::raise! (:wat::holon::leaf "send died"))))
             result
              (:wat::core::match (:wat::kernel::recv rx)
                -> :wat::core::i64
                ((:wat::core::Ok (:wat::core::Some n)) n)
                ((:wat::core::Ok :wat::core::None)    (:wat::kernel::raise! (:wat::holon::leaf "early close")))
                ((:wat::core::Err _)       (:wat::kernel::raise! (:wat::holon::leaf "thread died"))))
             _join
              (:wat::core::match (:wat::kernel::Thread/join-result thr)
                -> :wat::core::nil
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _) (:wat::kernel::raise! (:wat::holon::leaf "join failed"))))]
            result))
    "#;
    assert!(matches!(run(src), Value::i64(42)));
}

// ─── Inline fn literal body — the anonymous path ─────────────────────

#[test]
fn spawn_thread_inline_fn_body() {
    // Arc 170 slice 1f-ζ: computation in :my::compute.
    let src = r#"

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::let
            [thr
              (:wat::kernel::spawn-thread
                (:wat::core::fn
                  [in  <- :rust::crossbeam_channel::Receiver<wat::core::i64>
                   out <- :rust::crossbeam_channel::Sender<wat::core::i64>]
                   -> :wat::core::nil
                  (:wat::core::let
                    [value
                      (:wat::core::match (:wat::kernel::recv in)
                        -> :wat::core::i64
                        ((:wat::core::Ok (:wat::core::Some n)) n)
                        ((:wat::core::Ok :wat::core::None)
                         (:wat::kernel::raise! (:wat::holon::leaf "input closed")))
                        ((:wat::core::Err _)
                         (:wat::kernel::raise! (:wat::holon::leaf "parent died"))))
                     doubled (:wat::core::i64::*'2 value 2)]
                    (:wat::core::match (:wat::kernel::send out doubled)
                      -> :wat::core::nil
                      ((:wat::core::Ok _) ())
                      ((:wat::core::Err _)
                       (:wat::kernel::raise! (:wat::holon::leaf "output closed")))))))
             tx
              (:wat::kernel::Thread/input thr)
             rx
              (:wat::kernel::Thread/output thr)
             _ack
              (:wat::core::match (:wat::kernel::send tx 21)
                -> :wat::core::nil
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _) (:wat::kernel::raise! (:wat::holon::leaf "send died"))))
             result
              (:wat::core::match (:wat::kernel::recv rx)
                -> :wat::core::i64
                ((:wat::core::Ok (:wat::core::Some n)) n)
                ((:wat::core::Ok :wat::core::None)    (:wat::kernel::raise! (:wat::holon::leaf "early close")))
                ((:wat::core::Err _)       (:wat::kernel::raise! (:wat::holon::leaf "thread died"))))
             _join
              (:wat::core::match (:wat::kernel::Thread/join-result thr)
                -> :wat::core::nil
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _) (:wat::kernel::raise! (:wat::holon::leaf "join failed"))))]
            result))
    "#;
    assert!(matches!(run(src), Value::i64(42)));
}

// ─── Closure capture survives spawn-thread ────────────────────────────

#[test]
fn spawn_thread_closure_capture() {
    // The body's fn captures `delta` from the enclosing let. The
    // body still does mini-TCP — recv from `in`, send to `out` — but
    // the value sent uses the captured constant. Tests that closed_env
    // crosses the spawn boundary AND that the body uses its substrate
    // input/output pipes (not the captured `delta` as a substitute for
    // input).
    // Arc 170 slice 1f-ζ: computation in :my::compute.
    let src = r#"

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::let
            [delta 100
             body
              (:wat::core::fn
                [in  <- :rust::crossbeam_channel::Receiver<wat::core::i64>
                 out <- :rust::crossbeam_channel::Sender<wat::core::i64>]
                 -> :wat::core::nil
                (:wat::core::let
                  [n
                    (:wat::core::match (:wat::kernel::recv in)
                      -> :wat::core::i64
                      ((:wat::core::Ok (:wat::core::Some v)) v)
                      ((:wat::core::Ok :wat::core::None)
                       (:wat::kernel::raise! (:wat::holon::leaf "input closed")))
                      ((:wat::core::Err _)
                       (:wat::kernel::raise! (:wat::holon::leaf "parent died"))))
                   sum (:wat::core::i64::+'2 n delta)]
                  (:wat::core::match (:wat::kernel::send out sum)
                    -> :wat::core::nil
                    ((:wat::core::Ok _) ())
                    ((:wat::core::Err _)
                     (:wat::kernel::raise! (:wat::holon::leaf "output closed"))))))
             thr
              (:wat::kernel::spawn-thread body)
             tx
              (:wat::kernel::Thread/input thr)
             rx
              (:wat::kernel::Thread/output thr)
             _ack
              (:wat::core::match (:wat::kernel::send tx 23)
                -> :wat::core::nil
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _) (:wat::kernel::raise! (:wat::holon::leaf "send died"))))
             result
              (:wat::core::match (:wat::kernel::recv rx)
                -> :wat::core::i64
                ((:wat::core::Ok (:wat::core::Some n)) n)
                ((:wat::core::Ok :wat::core::None)    (:wat::kernel::raise! (:wat::holon::leaf "early close")))
                ((:wat::core::Err _)       (:wat::kernel::raise! (:wat::holon::leaf "thread died"))))
             _join
              (:wat::core::match (:wat::kernel::Thread/join-result thr)
                -> :wat::core::nil
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _) (:wat::kernel::raise! (:wat::holon::leaf "join failed"))))]
            result))
    "#;
    assert!(matches!(run(src), Value::i64(123)));
}

// ─── Non-callable body errors at type-check ───────────────────────────

#[test]
fn spawn_thread_rejects_non_callable_body() {
    // 42 is neither a keyword path nor a fn value. The checker's
    // TypeMismatch arm fires because spawn-thread's body parameter
    // expects :Fn(Receiver<I>,Sender<O>) -> :() and i64 doesn't unify.
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::nil)
          (:wat::core::let
            [not-fn 42
             thr
              (:wat::kernel::spawn-thread not-fn)]
            ()))
    "#;
    match startup(src) {
        Err(StartupError::Check(errs)) => {
            let hit = errs.0.iter().any(|e| {
                matches!(
                    e,
                    wat::check::CheckError::TypeMismatch { callee, .. }
                        if callee.contains(":wat::kernel::spawn-thread")
                )
            });
            assert!(hit, "expected spawn-thread TypeMismatch; got {:?}", errs.0);
        }
        Err(other) => panic!("expected Check error; got {:?}", other),
        Ok(_) => panic!("expected check-time failure"),
    }
}
