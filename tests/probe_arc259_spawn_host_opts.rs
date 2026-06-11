//! Arc 259 (The Forced Hand) — spawn host opts (the Keymaker). Stone 1.
//!
//! `wat/spawn.wat` mints the host opts that `spawn-program` will dispatch on:
//! `:wat::spawn::ThreadOpts` / `ProcessOpts` / `RemoteOpts` + the ergonomic
//! constructors `(thread)` / `(process)` / `(remote url key)`. This stone is
//! purely additive (it does not yet touch the live `spawn-program'`); it proves
//! the keys cut, the remote key carries its config, and — the forced hand — that
//! the remote key is UNCUTTABLE without both url and signing-key.
//!
//! Run: `cargo test --release --test probe_arc259_spawn_host_opts`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn eval_i64(decls: &str, body: &str) -> Result<i64, String> {
    let src = format!(
        "{decls}\n\
         (:wat::core::defn :user::compute [] -> :wat::core::i64 {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup/check: {e:?}"))?;
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::i64(n) => Ok(n),
        other => Err(format!("non-i64: {other:?}")),
    }
}

#[test]
fn c01_all_three_keys_cut() {
    // (thread) / (process) / (remote url key) all type-check + construct.
    let src = "(:wat::core::defn :user::main [] -> :wat::core::nil \
                 (:wat::core::do \
                   (:wat::spawn::thread) \
                   (:wat::spawn::process) \
                   (:wat::spawn::remote \"https://host\" \"sig-key\") \
                   nil))";
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_ok(),
        "the Keymaker cuts all three keys (thread/process/remote); got {:?}",
        result.err()
    );
}

#[test]
fn c02_remote_key_carries_its_config() {
    // The RemoteOpts record actually carries the url; read it back at runtime.
    // A known-length url ("abcde" → 5) lets the i64 harness prove the field
    // round-trips its value through the constructor + accessor.
    let got = eval_i64(
        "",
        "(:wat::core::string::length \
           (:wat::spawn::RemoteOpts/remote-url \
             (:wat::spawn::remote \"abcde\" \"sig-key\")))",
    );
    assert_eq!(got, Ok(5), "RemoteOpts carries remote-url; (remote \"abcde\" …) reads back, length 5");
}

#[test]
fn c03_remote_key_uncuttable_without_signing_key() {
    // THE FORCED HAND: (remote url) — missing the signing-key — is an arity error
    // at check. "Remote without a signing-key" is not a runtime check; it is an
    // uncuttable key. The constructor's arity is the lock.
    let src = "(:wat::core::defn :user::main [] -> :wat::core::nil \
                 (:wat::core::do (:wat::spawn::remote \"https://host\") nil))";
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_err(),
        "(remote url) without a signing-key must NOT cut — the remote door stays shut"
    );
}
