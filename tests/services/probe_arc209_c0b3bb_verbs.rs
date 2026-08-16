//! Arc 209 C0b.3b-b — the provisioning verbs: `allow'` / `deny'` (the owner mutates the
//! service's allow-set beyond the birth-seeded self).
//!
//! `(:wat::kernel::allow listener pid) -> :wat::core::nil` inserts `pid`; `deny'` removes it.
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
//! The listener is now autobibound (arc 272 — no fixed name); `(Bound/listener b)` extracts the
//! `Listener'` value for use with `allow'`/`deny'`. The verb proof is identical.
//!
//! Run: cargo test --release -p wat --test probe_arc209_c0b3bb_verbs -- --test-threads=1

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::{apply_function, Value};

#[test]
fn process_listener_allow_deny_succeed() {
    // allow'/deny' on a PROCESS listener' succeed (return nil). Wat source: probe_arc209_c0b3bb_verbs.wat
    let got = call_beside_value(file!(), ":user::compute")
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(42)),
        "expected 42: allow' then deny' on a process listener' each return nil; got {got:?}"
    );
}

#[test]
fn thread_listener_allow_errors_with_tier_message() {
    // RED at HEAD: `allow'` is an unknown head → the error names an unknown verb, NOT the tier.
    // GREEN after 3b-b: `allow'` on a thread listener is rejected with a process-tier message.
    // Wat source: probe_arc209_c0b3bb_verbs_thread.wat
    let outcome = (|| -> Result<Value, String> {
        let world = startup_from_file("tests/services/probe_arc209_c0b3bb_verbs_thread.wat")
            .map_err(|e| format!("{e:?}"))?;
        let func = world
            .symbols()
            .get(":user::compute")
            .ok_or_else(|| "no :user::compute in probe_arc209_c0b3bb_verbs_thread.wat".to_string())?
            .clone();
        apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
            .map_err(|e| format!("{e:?}"))
    })();
    match outcome {
        Err(msg) => {
            wat::assert_edn_matches_file!(msg, "probe_arc209_c0b3bb_verbs__thread_listener_allow_errors_with_tier_message.edn", "allow' on a thread listener must match process-tier rejection golden");
        }
        Ok(v) => panic!(
            "expected allow' on a thread listener to error (the crossbeam handle is the grant); \
             got {v:?}"
        ),
    }
}
