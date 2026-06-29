//! Arc 259 — the `user.program` slot: the env's user-extension half (the floor).
//!
//! The env's `wat.*` fields are platform-owned. User data lives in a nested slot
//! `user.program`, typed `:wat::core::Record` (the root — every record is a subtype, so
//! ANY user record fits). It is **always a record, never nil/optional** — the
//! default is `:wat::program::EmptyEnv`, a 0-field NOMINAL record (not nil, not an
//! anonymous map). That is what dodges optional-is-a-smell: there is no nil branch.
//!
//! This stone is the SLOT + the EmptyEnv default (stamped at the seam + per-peer
//! install). The `(thread/init f)` machinery that POPULATES it with a custom record
//! is the next stone.
//!
//! RED at HEAD: `Env` is a 5-arg record → the 6-arg constructor is an arity error,
//! and the seam env carries no `user.program`.
//!
//! Wat source lives in the co-located sibling fixture `probe_arc259_user_program_slot.wat`,
//! slurped via `startup_beside(file!())`.
//!
//! Run: `cargo test --release --test program probe_arc259_user_program_slot`

use wat::freeze::{eval_in_frozen, invoke_user_main, startup_beside};
use wat::runtime::{Environment, Value};

/// The record carries `user.program` holding a record (RED via arity at HEAD: a
/// 6-arg `Env` constructor is an arity error). `conforms?` to `:wat::core::Record` proves
/// the slot holds a genuine record value.
#[test]
fn env_record_carries_user_program_slot() {
    let world = startup_beside(file!()).expect("startup/check should succeed");
    let ast = wat::parse_one!("(:probe::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval")
        .value_owned();
    assert_eq!(
        got,
        Value::bool(true),
        "Env carries user.program (6th field) holding a record"
    );
}

/// The SEAM defaults `user.program` to `:wat::program::EmptyEnv` — a real record,
/// never nil. RED at HEAD: the field does not exist → accessor fails → main errors.
#[test]
fn seam_defaults_user_program_to_empty_env() {
    let world = startup_beside(file!()).expect("startup");
    assert!(
        invoke_user_main(&world, vec![]).is_ok(),
        "the seam must default user.program to a :wat::program::EmptyEnv record"
    );
}
