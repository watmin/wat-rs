//! Arc 209 C0b PREREQUISITE — structured peer death, PROCESS tier (Sub-stone B).
//!
//! The thread tier (Sub-stone A, shipped) now carries the `#wat.kernel/AssertionFailure`
//! envelope over its crash channel. This probe asks the empirical question for the process
//! tier: does a process peer's `assertion-failed!` — carrying a known `actual`/`expected` —
//! surface those STRUCTURED fields through `recv'`, or only the bare message?
//!
//! The process tier already emits a structured `#wat.kernel/ProcessPanics` / DiedError chain
//! envelope over its Err channel (`emit_structured_exit` → `conj_died_chain_value`), and the
//! arc 214 1b-ii-α probe proved `recv'` auto-raises a process crash reason (DivisionByZero).
//! The open question is whether the ASSERTION structure (actual/expected) rides that envelope.
//! GREEN here → Sub-stone B is already satisfied (document it). RED → B needs its own strike.
//!
//! Forks a `:process` child — run SERIALLY:
//!   `cargo test --release -p wat --test nursery probe_arc209_structured_peer_death_process -- --test-threads=1`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Environment;

/// A `:process` peer dies via `assertion-failed!` carrying a structured `actual` + `expected`.
/// `recv'` raises — the raised reason MUST carry BOTH structured fields, not just the message.
#[test]
fn process_peer_recv_surfaces_structured_actual_and_expected() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let [peer (:wat::kernel::spawn-program' (:wat::spawn::process)
                                   (:wat::core::forms
                                     (:wat::core::defn :user::main [] -> :wat::core::nil
                                       (:wat::core::let [n (:wat::kernel::readln -> :wat::core::i64)
                                                         _ (:wat::kernel::assertion-failed! "proc-structured-marker"
                                                             (:wat::core::Some "PROC-ACTUAL-5521")
                                                             (:wat::core::Some "PROC-EXPECTED-8841"))]
                                         nil))))
                            _ (:wat::kernel::send' peer 0)
                            got (:wat::kernel::recv' peer)]
            got))
        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "#;
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let err = match eval_in_frozen(&ast, &world, &Environment::new()) {
        Ok(v) => panic!("expected recv' to RAISE (the process peer crashed); got Ok({v:?})"),
        Err(e) => format!("{e:?}"),
    };
    assert!(
        err.contains("proc-structured-marker"),
        "the crash MESSAGE must travel on the process tier. got: {err}"
    );
    assert!(
        err.contains("PROC-ACTUAL-5521"),
        "process tier: the structured `actual` must survive recv'. got: {err}"
    );
    assert!(
        err.contains("PROC-EXPECTED-8841"),
        "process tier: the structured `expected` must survive recv'. got: {err}"
    );
}
