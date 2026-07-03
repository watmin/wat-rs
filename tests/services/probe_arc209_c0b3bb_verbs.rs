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
//! The listener is now autobibound (arc 272 — no fixed name); `(Bound/listener b)` extracts the
//! `Listener'` value for use with `allow'`/`deny'`. The verb proof is identical.
//!
//! Run: cargo test --release -p wat --test probe_arc209_c0b3bb_verbs -- --test-threads=1

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

#[test]
fn process_listener_allow_deny_succeed() {
    // allow'/deny' on a PROCESS listener' succeed (return nil). Wat source: probe_arc209_c0b3bb_verbs.wat
    let world = startup_beside(file!())
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

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn thread_listener_allow_errors_with_tier_message() {
    // RED at HEAD: `allow'` is an unknown head → the error names an unknown verb, NOT the tier.
    // GREEN after 3b-b: `allow'` on a thread listener is rejected with a process-tier message.
    // Wat source: probe_arc209_c0b3bb_verbs_thread.wat
    let outcome = (|| -> Result<Value, String> {
        let world = startup_from_file("tests/services/probe_arc209_c0b3bb_verbs_thread.wat")
            .map_err(|e| format!("{e:?}"))?;
        let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("{e:?}"))?;
        eval_in_frozen(&ast, &world, &Environment::new())
            .map(|tv| tv.value_owned())
            .map_err(|e| format!("{e:?}"))
    })();
    match outcome {
        Err(msg) => {
            assert_eq!(
                msg,
                "RuntimeError { span: Span { file: \"tests/services/probe_arc209_c0b3bb_verbs_thread.wat\", line: 6, col: 33, end_line: 6, end_col: 34 }, kind: MalformedForm { head: \":wat::kernel::allow'\", reason: \"allow' is a process-tier service gate; a thread listener's handle IS the grant\" } }",
                "allow' on a thread listener must match process-tier rejection golden"
            );
        }
        Ok(v) => panic!(
            "expected allow' on a thread listener to error (the crossbeam handle is the grant); \
             got {v:?}"
        ),
    }
}
