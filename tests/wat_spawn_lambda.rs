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
//! lambda, inline lambda literal, lambda-valued param, closure-
//! captured lambda) — but the contract under test is the mini-TCP
//! shape: input flows in via the pipe, output flows out via the pipe,
//! never via "return value." `Thread/join-result` confirms the body
//! finished without panic.
//!
//! See `docs/arc/2026/04/114-spawn-as-thread/DESIGN.md` for the
//! contract; `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels"
//! for the principle.

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

// ─── Named-define body — the keyword path ─────────────────────────────

#[test]
fn spawn_thread_named_define_body() {
    // The body is a named define matching the channel-shaped contract.
    // Parent sends 41; worker recvs, increments, writes; parent recvs.
    let src = r#"

        (:wat::core::define
          (:app::increment
            (in  :rust::crossbeam_channel::Receiver<wat::core::i64>)
            (out :rust::crossbeam_channel::Sender<wat::core::i64>)
            -> :wat::core::unit)
          (:wat::core::let*
            (((value :wat::core::i64)
              (:wat::core::match (:wat::kernel::recv in)
                -> :wat::core::i64
                ((:wat::core::Ok (:wat::core::Some n)) n)
                ((:wat::core::Ok :wat::core::None)
                 (:wat::kernel::raise! (:wat::holon::leaf "input closed")))
                ((:wat::core::Err _)
                 (:wat::kernel::raise! (:wat::holon::leaf "parent died")))))
             ((sum :wat::core::i64) (:wat::core::i64::+ value 1)))
            (:wat::core::match (:wat::kernel::send out sum)
              -> :wat::core::unit
              ((:wat::core::Ok _) ())
              ((:wat::core::Err _)
               (:wat::kernel::raise! (:wat::holon::leaf "output closed"))))))

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let*
            (((thr :wat::kernel::Thread<wat::core::i64,wat::core::i64>)
              (:wat::kernel::spawn-thread :app::increment))
             ((tx :rust::crossbeam_channel::Sender<wat::core::i64>)
              (:wat::kernel::Thread/input thr))
             ((rx :rust::crossbeam_channel::Receiver<wat::core::i64>)
              (:wat::kernel::Thread/output thr))
             ((_ack :wat::core::unit)
              (:wat::core::match (:wat::kernel::send tx 41)
                -> :wat::core::unit
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _) (:wat::kernel::raise! (:wat::holon::leaf "send died")))))
             ((result :wat::core::i64)
              (:wat::core::match (:wat::kernel::recv rx)
                -> :wat::core::i64
                ((:wat::core::Ok (:wat::core::Some n)) n)
                ((:wat::core::Ok :wat::core::None)    (:wat::kernel::raise! (:wat::holon::leaf "early close")))
                ((:wat::core::Err _)       (:wat::kernel::raise! (:wat::holon::leaf "thread died")))))
             ((_join :wat::core::unit)
              (:wat::core::match (:wat::kernel::Thread/join-result thr)
                -> :wat::core::unit
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _) (:wat::kernel::raise! (:wat::holon::leaf "join failed"))))))
            result))
    "#;
    assert!(matches!(run(src), Value::i64(42)));
}

// ─── Inline lambda literal body — the anonymous path ─────────────────

#[test]
fn spawn_thread_inline_lambda_body() {
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let*
            (((thr :wat::kernel::Thread<wat::core::i64,wat::core::i64>)
              (:wat::kernel::spawn-thread
                (:wat::core::lambda
                  ((in  :rust::crossbeam_channel::Receiver<wat::core::i64>)
                   (out :rust::crossbeam_channel::Sender<wat::core::i64>)
                   -> :wat::core::unit)
                  (:wat::core::let*
                    (((value :wat::core::i64)
                      (:wat::core::match (:wat::kernel::recv in)
                        -> :wat::core::i64
                        ((:wat::core::Ok (:wat::core::Some n)) n)
                        ((:wat::core::Ok :wat::core::None)
                         (:wat::kernel::raise! (:wat::holon::leaf "input closed")))
                        ((:wat::core::Err _)
                         (:wat::kernel::raise! (:wat::holon::leaf "parent died")))))
                     ((doubled :wat::core::i64) (:wat::core::i64::* value 2)))
                    (:wat::core::match (:wat::kernel::send out doubled)
                      -> :wat::core::unit
                      ((:wat::core::Ok _) ())
                      ((:wat::core::Err _)
                       (:wat::kernel::raise! (:wat::holon::leaf "output closed"))))))))
             ((tx :rust::crossbeam_channel::Sender<wat::core::i64>)
              (:wat::kernel::Thread/input thr))
             ((rx :rust::crossbeam_channel::Receiver<wat::core::i64>)
              (:wat::kernel::Thread/output thr))
             ((_ack :wat::core::unit)
              (:wat::core::match (:wat::kernel::send tx 21)
                -> :wat::core::unit
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _) (:wat::kernel::raise! (:wat::holon::leaf "send died")))))
             ((result :wat::core::i64)
              (:wat::core::match (:wat::kernel::recv rx)
                -> :wat::core::i64
                ((:wat::core::Ok (:wat::core::Some n)) n)
                ((:wat::core::Ok :wat::core::None)    (:wat::kernel::raise! (:wat::holon::leaf "early close")))
                ((:wat::core::Err _)       (:wat::kernel::raise! (:wat::holon::leaf "thread died")))))
             ((_join :wat::core::unit)
              (:wat::core::match (:wat::kernel::Thread/join-result thr)
                -> :wat::core::unit
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _) (:wat::kernel::raise! (:wat::holon::leaf "join failed"))))))
            result))
    "#;
    assert!(matches!(run(src), Value::i64(42)));
}

// ─── Closure capture survives spawn-thread ────────────────────────────

#[test]
fn spawn_thread_closure_capture() {
    // The body's lambda captures `delta` from the enclosing let*. The
    // body still does mini-TCP — recv from `in`, send to `out` — but
    // the value sent uses the captured constant. Tests that closed_env
    // crosses the spawn boundary AND that the body uses its substrate
    // input/output pipes (not the captured `delta` as a substitute for
    // input).
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let*
            (((delta :wat::core::i64) 100)
             ((body :fn(rust::crossbeam_channel::Receiver<wat::core::i64>,rust::crossbeam_channel::Sender<wat::core::i64>)->wat::core::unit)
              (:wat::core::lambda
                ((in  :rust::crossbeam_channel::Receiver<wat::core::i64>)
                 (out :rust::crossbeam_channel::Sender<wat::core::i64>)
                 -> :wat::core::unit)
                (:wat::core::let*
                  (((n :wat::core::i64)
                    (:wat::core::match (:wat::kernel::recv in)
                      -> :wat::core::i64
                      ((:wat::core::Ok (:wat::core::Some v)) v)
                      ((:wat::core::Ok :wat::core::None)
                       (:wat::kernel::raise! (:wat::holon::leaf "input closed")))
                      ((:wat::core::Err _)
                       (:wat::kernel::raise! (:wat::holon::leaf "parent died")))))
                   ((sum :wat::core::i64) (:wat::core::i64::+ n delta)))
                  (:wat::core::match (:wat::kernel::send out sum)
                    -> :wat::core::unit
                    ((:wat::core::Ok _) ())
                    ((:wat::core::Err _)
                     (:wat::kernel::raise! (:wat::holon::leaf "output closed")))))))
             ((thr :wat::kernel::Thread<wat::core::i64,wat::core::i64>)
              (:wat::kernel::spawn-thread body))
             ((tx :rust::crossbeam_channel::Sender<wat::core::i64>)
              (:wat::kernel::Thread/input thr))
             ((rx :rust::crossbeam_channel::Receiver<wat::core::i64>)
              (:wat::kernel::Thread/output thr))
             ((_ack :wat::core::unit)
              (:wat::core::match (:wat::kernel::send tx 23)
                -> :wat::core::unit
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _) (:wat::kernel::raise! (:wat::holon::leaf "send died")))))
             ((result :wat::core::i64)
              (:wat::core::match (:wat::kernel::recv rx)
                -> :wat::core::i64
                ((:wat::core::Ok (:wat::core::Some n)) n)
                ((:wat::core::Ok :wat::core::None)    (:wat::kernel::raise! (:wat::holon::leaf "early close")))
                ((:wat::core::Err _)       (:wat::kernel::raise! (:wat::holon::leaf "thread died")))))
             ((_join :wat::core::unit)
              (:wat::core::match (:wat::kernel::Thread/join-result thr)
                -> :wat::core::unit
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _) (:wat::kernel::raise! (:wat::holon::leaf "join failed"))))))
            result))
    "#;
    assert!(matches!(run(src), Value::i64(123)));
}

// ─── Non-callable body errors at type-check ───────────────────────────

#[test]
fn spawn_thread_rejects_non_callable_body() {
    // 42 is neither a keyword path nor a lambda value. The checker's
    // TypeMismatch arm fires because spawn-thread's body parameter
    // expects :Fn(Receiver<I>,Sender<O>) -> :() and i64 doesn't unify.
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::unit)
          (:wat::core::let*
            (((not-fn :wat::core::i64) 42)
             ((thr :wat::kernel::Thread<wat::core::i64,wat::core::i64>)
              (:wat::kernel::spawn-thread not-fn)))
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
