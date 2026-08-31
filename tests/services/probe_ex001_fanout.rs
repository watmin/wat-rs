//! Excursus 001 stone 7 — the fan-out circuit. The app lives at
//! `wat-scripts/fanout/circuit.wat` (not stdlib). This harness drives
//! `:user::compute` (`run 12 2 2`), the floor-weight wiring.
//!
//! Completeness fields are byte-identical (`assert_eq!`, no `.contains(` —
//! `no_loose_string_assert`). Worker-id count is scheduling-dependent (two
//! process workers racing a serializing queue actor); it is asserted as a
//! range, not a pinned number.
//!
//! `startup_from_file` uses `InMemoryLoader` and cannot resolve this file's
//! `(:wat::load-file! "../topic/…")` / `"../queue/…"`. Drive it the way
//! `every_wat_scripts_file_loads` does: `startup_from_source` + `FsLoader`,
//! so relative loads resolve against `wat-scripts/fanout/`.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::loader::FsLoader;
use wat::runtime::{apply_function, Value};

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
fn fanout_compute_is_complete_and_lossless() {
    let rel = "wat-scripts/fanout/circuit.wat";
    let src = std::fs::read_to_string(rel).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    let world = startup_from_source(&src, Some(rel), Arc::new(FsLoader))
        .expect("circuit should freeze (Outcome in Worker :messages; Envelope in Queue :messages)");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!(":user::compute not registered"))
        .clone();
    let stored = match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!(":user::compute returned non-String: {other:?}"),
        Err(e) => panic!("fan-out circuit raised: {e:?}"),
    };

    assert_eq!(field(&stored, "n"), "12");
    assert_eq!(field(&stored, "m"), "2");
    assert_eq!(field(&stored, "j"), "2");
    // N×M outcomes, no duplicate (queue,id), leftover receive empty (empty=1 is the
    // all-queues-empty flag, not a count of queues).
    assert_eq!(field(&stored, "total"), "24");
    assert_eq!(field(&stored, "distinct"), "24");
    assert_eq!(field(&stored, "dup"), "0");
    assert_eq!(field(&stored, "empty"), "1");

    let workers: i64 = field(&stored, "workers")
        .parse()
        .unwrap_or_else(|_| panic!("workers not an i64 in {stored}"));
    assert!(
        workers > 0 && workers <= 4,
        "workers must be in 1..=M×J so a zero summary cannot pass as complete; got {workers} in {stored}"
    );
}
