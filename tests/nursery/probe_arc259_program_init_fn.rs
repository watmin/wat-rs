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
//! Run SERIALLY (spawns a thread):
//!   `cargo test --release -p wat --test nursery probe_arc259_program_init_fn -- --test-threads=1`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// A `(thread/init f)` peer's `user.program` is f's custom record. f returns a
/// `MyEnv{port: 8080}`; the peer reads `user.program`'s port back. Parent asserts 8080.
#[test]
fn thread_init_populates_user_program() {
    let src = "(:wat::Record::def :user::MyEnv [port <- :wat::core::i64]) \
               (:wat::core::defn :user::compute [] -> :wat::core::i64 \
                 (:wat::core::let \
                   [peer (:wat::kernel::spawn-program' \
                           (:wat::spawn::thread/init \
                             (:wat::core::fn [] -> :wat::Record (:user::MyEnv 8080))) \
                           (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil \
                             (:wat::kernel::send' self \
                               (:user::MyEnv/port \
                                 (:wat::program::Env/user.program (:wat::program::env)))))) \
                    got (:wat::kernel::recv' peer) \
                    _ (:wat::kernel::close' peer)] \
                   got)) \
               (:wat::core::defn :user::main [] -> :wat::core::nil nil)";
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup (RED at HEAD: (thread/init …) does not exist)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("compute eval")
        .value_owned()
    {
        Value::i64(n) => n,
        other => panic!("expected i64; got {other:?}"),
    };
    assert_eq!(
        got, 8080,
        "(thread/init f) peer's user.program is f's MyEnv{{port:8080}}"
    );
}

/// A plain `(thread)` peer's `user.program` stays the EmptyEnv default — the default
/// constructor's init-fn is the EmptyEnv thunk. The peer reports conformance (1/0).
#[test]
fn thread_default_user_program_is_empty_env() {
    let src = "(:wat::core::defn :user::compute [] -> :wat::core::i64 \
                 (:wat::core::let \
                   [peer (:wat::kernel::spawn-program' (:wat::spawn::thread) \
                           (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil \
                             (:wat::kernel::send' self \
                               (:wat::core::if \
                                 (:wat::core::conforms? \
                                   (:wat::program::Env/user.program (:wat::program::env)) \
                                   :wat::program::EmptyEnv) -> :wat::core::i64 \
                                 1 0)))) \
                    got (:wat::kernel::recv' peer) \
                    _ (:wat::kernel::close' peer)] \
                   got)) \
               (:wat::core::defn :user::main [] -> :wat::core::nil nil)";
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("compute eval")
        .value_owned()
    {
        Value::i64(n) => n,
        other => panic!("expected i64; got {other:?}"),
    };
    assert_eq!(got, 1, "(thread) peer's user.program defaults to EmptyEnv");
}
