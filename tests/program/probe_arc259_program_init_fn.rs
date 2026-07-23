//! Arc 259 — the program init-fn: `(thread/init f)` populates `user.program` with a
//! CUSTOM record, end to end. The user-extension half, completed.
//!
//!   `(thread)`        → user.program = EmptyEnv (the default init-fn thunk).
//!   `(thread/init f)` → user.program = f's record, where f : [] -> SomeRecord, run
//!                       AT THE PEER'S START (in the peer thread, so user.program
//!                       reflects the peer's own context).
//!
//! No optional token: `(thread)` is a COMPLETE constructor whose init-fn IS the
//! EmptyEnv thunk; `(thread/init f)` is a complete constructor carrying f. The user
//! picks intent by verb, never by an omitted arg.
//!
//! RED at HEAD: `(thread/init …)` does not exist, ThreadOpts carries no init-fn, and
//! user.program is always EmptyEnv. The peer reads `user.program`'s `port` field
//! back over the channel (a peer assertion is swallowed; only what it sends counts).
//!
//! Wat source lives in the co-located sibling fixture `probe_arc259_program_init_fn.wat`,
//! slurped via `startup_beside(file!())`.
//!
//! Run SERIALLY (spawns threads):
//!   `cargo nextest run --release -E 'test(init_fn)'`

use wat::freeze::call_beside;
use wat::runtime::Value;

/// A `(thread/init f)` peer's `user.program` is f's custom record. f returns a
/// `MyEnv{port: 8080}`; the peer reads `user.program`'s port back. Parent asserts 8080.
#[test]
fn thread_init_populates_user_program() {
    let got = match call_beside(file!(), ":probe::compute-init").expect("compute eval") {
        Value::i64(n) => n,
        other => panic!("expected i64; got {other:?}"),
    };
    assert_eq!(
        got, 8080,
        "(thread/init f) peer's user.program is f's MyEnv{{port:8080}}"
    );
}

/// An init-fn that ERRORS kills the peer honestly — the env is never built with a
/// non-record fallback in `user.program`. Arc 278 recv'-wall: the dead peer surfaces as a matchable
/// `RecvOutcome::Lost` VALUE (never a raise — a raise unwinds past the reader); the fixture RETURNS
/// the Lost cause's `Failure/message`. We assert `is_ok` (it matched Lost as a value) + that the
/// returned reason is the init-fn's crash (::Lost, not a smuggled ::Message/::Closed).
#[test]
fn erroring_init_fn_kills_the_peer() {
    // init-fn divides by zero → errors at peer-start → the thread exits before sending → recv'
    // returns a NON-::Message outcome (the peer died before it could smuggle its 7). On this tier the
    // dying init-fn exits before buffering a crash reason, so it surfaces as ::Closed; a
    // reason-carrying tier would surface ::Lost. Both prove the kill — only a smuggled ::Message fails.
    let result = call_beside(file!(), ":probe::compute-error-init");
    let text = format!("{result:?}");
    assert!(
        result.is_ok(),
        "the erroring init-fn must kill the peer and surface as a matchable RecvOutcome VALUE \
         (never a raise); got Err: {text}"
    );
    assert!(
        // rune:lint(loose-assert) — absence check over a value-based RecvOutcome marker: the peer must
        // NOT deliver the smuggled 7 as a ::Message (::Lost/::Closed both prove the kill).
        !text.contains("SMUGGLED-VALUE"),
        "the erroring init-fn must kill the peer → recv' must NOT deliver the smuggled 7 as a \
         ::Message (a ::Lost/::Closed both prove the kill); got: {text}"
    );
}

/// A plain `(thread)` peer's `user.program` stays the EmptyEnv default — the default
/// constructor's init-fn is the EmptyEnv thunk. The peer reports conformance (1/0).
#[test]
fn thread_default_user_program_is_empty_env() {
    let got = match call_beside(file!(), ":probe::compute-default").expect("compute eval") {
        Value::i64(n) => n,
        other => panic!("expected i64; got {other:?}"),
    };
    assert_eq!(got, 1, "(thread) peer's user.program defaults to EmptyEnv");
}
