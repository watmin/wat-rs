//! Arc 272 rs-1 — a service's `:state` MUST be a record (base or holon-derived).
//!
//! The no-magic / typed-record law applied to defservice: a service's state is the wire-conformant,
//! named-typed contract for both `state0` (in) and the final state (out, via the rs-2 `:Stop` op). A bare
//! scalar like `:wat::core::i64` is EDN-serializable but STRUCTURELESS — no named fields, no conformance —
//! so it must NOT be accepted as a service state. A record (base `:wat::Record` derived, or
//! `:wat::holon::Record` derived) must be.
//!
//! Mechanism (Path A — the build, not asserted here): defservice emits a CHECK-TIME assertion form that
//! resolves `:state-ty` against the TypeEnv and asserts record-or-derived via the existing
//! `is_subtype(ty, ":wat::Record") || is_subtype(ty, ":wat::holon::Record")` (`collection/infer.rs:378-381`).
//! Check-time, NOT macro-time: macros expand at freeze step 4, types register at step 5 — at expand time the
//! registry is empty and records are themselves minted by macros (chicken-and-egg). The check lives where
//! types live: step 8 (check).
//!
//! RED at HEAD: a scalar `:state` is ACCEPTED today (the shipped rs-2 probes use `:state :wat::core::i64`).
//! GREEN once rs-1's check rejects a non-record `:state`. `#[ignore]` on the negative case until then.
//!
//! Run: cargo test --release -p wat --test probe_arc272_rs1_state_must_be_record -- --include-ignored

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

// A service whose `:state` is a bare scalar — the structureless form rs-1 must forbid.
const SCALAR_STATE: &str = r#"
(:wat::service::defservice :my::counter
  :state :wat::core::i64
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::counter::GetResponse s)))])

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

// A service whose `:state` is a record — the valid form. Must compile before AND after rs-1.
const RECORD_STATE: &str = r#"
(:wat::Record::def :my::counter2::CounterState [count <- :wat::core::i64])

(:wat::service::defservice :my::counter2
  :state :my::counter2::CounterState
  :ops
  [(:Get [s <- :State]
         -> [value <- :my::counter2::CounterState]
     (:wat::service::Outcome::Reply s (:my::counter2::GetResponse s)))])

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#;

#[test]
#[ignore = "rs-1 RED until defservice rejects a non-record :state. A scalar :state compiles today \
            (the rs-2 probes use :state :wat::core::i64). UN-IGNORE when rs-1's check lands."]
fn scalar_state_is_rejected() {
    let result = startup_from_source(SCALAR_STATE, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_err(),
        "expected a non-record :state (:wat::core::i64) to be REJECTED at compile time \
         (a service state must be a record — base or holon-derived); instead startup succeeded"
    );
}

#[test]
fn record_state_is_accepted() {
    let result = startup_from_source(RECORD_STATE, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_ok(),
        "a record :state must compile (before and after rs-1 — records are the valid state form); \
         got {result:?}"
    );
}
