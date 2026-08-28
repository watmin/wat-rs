//! A2 RED probe — program::Env as a typed extensible recordtype base.
//!
//! At HEAD `:wat::program::Env` is a `typealias = HashMap<keyword, HolonAST>` (the dynamic store
//! whose cast-accessors A1 deleted). A2 replaces it with a recordtype base carrying the first
//! system field `started-at : Instant`, defined in blessed stdlib `wat/program.wat` via
//! `Record::def`, and swaps the spawn arg[1] check `unify`→`assignable` so an *extended* env
//! (a child recordtype) satisfies the base.
//!
//! RED at HEAD on both counts:
//!   C01 — program::Env has no record constructor/accessor (it's a HashMap) → no `started-at`.
//!   C02 — program::Env can't be a recordtype PARENT (it's a typealias, not a record).
//!
//! Wat source lives in the co-located sibling fixture `probe_arc258_program_env_record.wat`,
//! slurped via `startup_beside(file!())`. Named probe functions `:probe::c01-compute` and
//! `:probe::c02-compute` replace the inline `eval_i64` helper.
//!
//! Run: `cargo test --release --test probe_arc258_program_env_record`

use wat::freeze::{startup_beside, StartupError};
use wat::runtime::{apply_function, RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

fn call_i64(world: &wat::freeze::FrozenWorld, fn_name: &str) -> Result<i64, StartupError> {
    let func = world.symbols().get(fn_name).ok_or_else(|| {
        StartupError::Runtime(Box::new(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::UnboundSymbol(fn_name.to_string()),
        )))
    })?;
    match apply_function(func.clone(), Vec::new(), world.symbols(), wat::rust_caller_span!())
        .map_err(|e| StartupError::Runtime(Box::new(e)))?
    {
        Value::i64(n) => Ok(n),
        other => Err(StartupError::Runtime(Box::new(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: fn_name.to_string(),
                expected: "i64",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )))),
    }
}

#[test]
fn c01_base_record_started_at() {
    // Construct the base program::Env with started-at = at-millis(5000), read it back.
    let world = startup_beside(file!()).expect("startup");
    match call_i64(&world, ":probe::c01-compute") {
        Ok(n) => assert_eq!(
            n, 5000,
            "program::Env is a record with a started-at : Instant field, constructed + read"
        ),
        Err(e) => panic!(
            "program::Env is a record with a started-at : Instant field, constructed + read; \
             call failed: {e:?}"
        ),
    }
}

// c02_user_extends_program_env DELETED — arc 293 inheritance annihilation:
// (:wat::core::recordtype :user::MyEnv :wat::program::Env [...]) is rejected at registration
// (non-nature-root parent). program::Env is a flat record; user types root at :wat::core::Record.
