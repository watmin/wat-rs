//! A2 RED probe — program::Env as a typed extensible recordtype base.
//!
//! At HEAD `:wat::program::Env` is a `typealias = HashMap<keyword, HolonAST>` (the dynamic store
//! whose cast-accessors A1 deleted). A2 replaces it with a recordtype base carrying the first
//! system field `wat.started-at : Instant`, defined in blessed stdlib `wat/program.wat` via
//! `Record::def`, and swaps the spawn arg[1] check `unify`→`assignable` so an *extended* env
//! (a child recordtype) satisfies the base.
//!
//! RED at HEAD on both counts:
//!   C01 — program::Env has no record constructor/accessor (it's a HashMap) → no `wat.started-at`.
//!   C02 — program::Env can't be a recordtype PARENT (it's a typealias, not a record).
//!
//! Wat source lives in the co-located sibling fixture `probe_arc258_program_env_record.wat`,
//! slurped via `startup_beside(file!())`. Named probe functions `:probe::c01-compute` and
//! `:probe::c02-compute` replace the inline `eval_i64` helper.
//!
//! Run: `cargo test --release --test probe_arc258_program_env_record`

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};
use wat::span::Span;

fn call_i64(world: &wat::freeze::FrozenWorld, fn_name: &str) -> Result<i64, String> {
    let func = world
        .symbols()
        .get(fn_name)
        .ok_or_else(|| format!("{fn_name} not found in world"))?;
    match apply_function(func.clone(), Vec::new(), world.symbols(), Span::unknown())
        .map_err(|e| format!("startup/check: {e:?}"))?
    {
        Value::i64(n) => Ok(n),
        other => Err(format!("non-i64: {other:?}")),
    }
}

#[test]
fn c01_base_record_started_at() {
    // Construct the base program::Env with started-at = at-millis(5000), read it back.
    let world = startup_beside(file!()).expect("startup");
    let got = call_i64(&world, ":probe::c01-compute");
    assert_eq!(got, Ok(5000),
        "program::Env is a record with a wat.started-at : Instant field, constructed + read");
}

#[test]
fn c02_user_extends_program_env() {
    // A program EXTENDS program::Env with its own typed field; the extension is a subtype.
    // Construct it, read the inherited wat.started-at AND the user field.
    let world = startup_beside(file!()).expect("startup");
    let got = call_i64(&world, ":probe::c02-compute");
    assert_eq!(got, Ok(8080),
        "a user recordtype can extend :wat::program::Env (it is a record base, not a HashMap typealias)");
}
