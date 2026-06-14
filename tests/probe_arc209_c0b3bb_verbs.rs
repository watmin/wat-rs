//! Arc 209 C0b.3b-b — the provisioning verbs: `allow'` / `deny'` (the owner mutates the
//! service's allow-set beyond the birth-seeded self).
//!
//! `(:wat::kernel::allow' listener pid) -> :wat::core::nil` inserts `pid`; `deny'` removes it.
//! Both are PROCESS-TIER ONLY: a thread/crossbeam listener has no allow-set (the handle IS the
//! grant), so `allow'`/`deny'` on one is a clean error ("process-tier service gate").
//!
//! - `process_listener_allow_deny_succeed` — RED at HEAD (the heads `allow'`/`deny'` are
//!   unknown → check error). GREEN after 3b-b: `allow'` then `deny'` on a process `listener'`
//!   each return nil; `compute` returns 42.
//! - `thread_listener_allow_errors_with_tier_message` — RED at HEAD (the head is unknown → the
//!   error is "unknown verb", which does NOT mention the tier). GREEN after 3b-b: `allow'` on a
//!   thread `listener'` is a clean runtime error whose message names the process-tier gate.
//!
//! Run: cargo test --release -p wat --test probe_arc209_c0b3bb_verbs -- --test-threads=1

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// allow'/deny' on a PROCESS listener' succeed (return nil). The allow-set is the SocketListener's.
const PROCESS_VERBS_PROGRAM: &str = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [l (:wat::kernel::listener' (:wat::spawn::process)
         (:wat::kernel::socket-address' "wat.arc209.c0b3bb.verbs" :wat::core::i64 :wat::core::i64))
     _ (:wat::kernel::allow' l 12345)
     _ (:wat::kernel::deny' l 12345)]
    42))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn process_listener_allow_deny_succeed() {
    let world = startup_from_source(PROCESS_VERBS_PROGRAM, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (C0b.3b-b: allow'/deny' provisioning verbs)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(42)),
        "expected 42: allow' then deny' on a process listener' each return nil; got {got:?}"
    );
}

// allow' on a THREAD listener' is a clean error — the crossbeam handle IS the grant.
const THREAD_VERB_PROGRAM: &str = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [pair (:wat::kernel::listener' (:wat::spawn::thread) :wat::core::i64 :wat::core::i64)
     l    (:wat::core::first pair)
     _    (:wat::kernel::allow' l 123)]
    42))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
fn thread_listener_allow_errors_with_tier_message() {
    // RED at HEAD: `allow'` is an unknown head → the error names an unknown verb, NOT the tier.
    // GREEN after 3b-b: `allow'` on a thread listener is rejected with a process-tier message.
    let outcome = (|| -> Result<Value, String> {
        let world = startup_from_source(THREAD_VERB_PROGRAM, None, Arc::new(InMemoryLoader::new()))
            .map_err(|e| format!("{e:?}"))?;
        let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("{e:?}"))?;
        eval_in_frozen(&ast, &world, &Environment::new())
            .map(|tv| tv.value_owned())
            .map_err(|e| format!("{e:?}"))
    })();
    match outcome {
        Err(msg) => assert!(
            msg.contains("process-tier"),
            "expected allow' on a thread listener to be rejected with a process-tier message; \
             got error: {msg}"
        ),
        Ok(v) => panic!(
            "expected allow' on a thread listener to error (the crossbeam handle is the grant); \
             got {v:?}"
        ),
    }
}
