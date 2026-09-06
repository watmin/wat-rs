//! Floor gate: a failed sqlite put must close its transaction.
//! Companion to `probe_arc278_txn_must_close.wat`.
//!
//! STOP-6 measured `put2=Fatal: cannot start a transaction within a transaction`.
//! After rollback on the store error path, put2 must still report the original
//! cause (`no such table: main`), never the poisoned-connection string.

use std::sync::Arc;
use wat::freeze::{startup_from_source, FrozenWorld};
use wat::load::loader::FsLoader;
use wat::runtime::{apply_function, Value};

fn load_probe() -> FrozenWorld {
    let rel = "tests/services/probe_arc278_txn_must_close.wat";
    let src = std::fs::read_to_string(rel).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    startup_from_source(&src, Some(rel), Arc::new(FsLoader))
        .expect("txn-must-close probe should freeze")
}

fn call_string(world: &FrozenWorld, name: &str) -> String {
    let func = world
        .symbols()
        .get(name)
        .unwrap_or_else(|| panic!("{name} not registered"))
        .clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!("{name} returned non-String: {other:?}"),
        Err(e) => panic!("{name} raised: {e:?}"),
    }
}

fn field<'a>(summary: &'a str, key: &str) -> &'a str {
    for part in summary.split(';') {
        if let Some((k, v)) = part.split_once('=') {
            if k == key {
                return v;
            }
        }
    }
    panic!("summary missing field {key:?}: {summary}");
}

#[test]
fn a_failed_put_does_not_poison_the_next_put() {
    let world = load_probe();
    let stored = call_string(&world, ":user::compute");
    assert_eq!(
        field(&stored, "put1"),
        "Fatal:no such table: main",
        "put1 must keep the original cause; got {stored}"
    );
    assert_eq!(
        field(&stored, "put2"),
        "Fatal:no such table: main",
        "put2 must still report the original cause, not a poisoned connection; got {stored}"
    );
    assert_eq!(
        field(&stored, "begin2"),
        "Ok",
        "CONN begin2 after rollback must succeed; got {stored}"
    );
    assert_eq!(
        field(&stored, "rollback"),
        "Ok",
        "CONN rollback after the failed statement must succeed; got {stored}"
    );
}
