//! Reach-stumble enrichment — `Duration` is a WRITE-ONLY value; mint the readout family.
//!
//! The time-ops surface is asymmetric. `Instant` round-trips: you build one from
//! an i64 (`at` / `at-millis` / `at-nanos`) and read it back to an i64
//! (`epoch-seconds` / `epoch-millis` / `epoch-nanos`). `Duration` only has the
//! IN half: seven unit constructors (`Nanoseconds` … `Days`) build one from an i64,
//! you can add/subtract and compare Durations — but there is NO way to read the
//! number back OUT in any unit. A program can compute `(now - started-at)` and
//! never learn how long it was.
//!
//! This probe reaches for that missing readout — the inverse of `epoch-*`, the
//! seven-unit symmetric mirror of the constructors — and RED-fails at HEAD
//! because the verbs do not exist (undefined function; either at check via the
//! reserved-prefix path or at eval).
//!
//! The family (intueri-named, 2026-06-11 — bare unit-plural, the accessor IS the
//! unit word; capitalized `Second` constructs, lowercase `seconds` reads out):
//!   nanoseconds · microseconds · milliseconds · seconds ·
//!   minutes · hours · days        ;; each :wat::time::Duration -> :i64
//!
//! Run: `cargo nextest run --release -E 'binary(kernel)' -F probe_time_duration_readout`
//!
//! WAT fixtures: tests/kernel/probe_time_duration_readout_{same_unit,across_units,truncates,instant_delta}.wat

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

/// Eval a fixture that returns `:i64` in a frozen world. Panics (RED) if startup or
/// eval fails — at HEAD the undefined readout verb makes eval fail here.
fn eval_i64(path: &str) -> i64 {
    let world = startup_from_file(path).expect("startup/check should succeed");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!("no :user::compute in {path:?}"))
        .clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("compute eval")
    {
        Value::i64(n) => n,
        other => panic!("expected i64; got {other:?}"),
    }
}

/// Identity round-trip: build in a unit, read back in the SAME unit.
#[test]
fn duration_reads_back_in_same_unit() {
    assert_eq!(
        eval_i64("tests/kernel/probe_time_duration_readout_same_unit.wat"),
        1500,
        "a Duration built as 1500ms reads back as 1500ms",
    );
}

/// Cross-unit conversion: 1ms = 1_000_000ns.
#[test]
fn duration_reads_across_units() {
    assert_eq!(
        eval_i64("tests/kernel/probe_time_duration_readout_across_units.wat"),
        1_000_000,
        "1ms read as nanoseconds is 1_000_000",
    );
}

/// Truncating toward zero, exactly like `epoch-millis`: 1500ms read as whole
/// seconds is 1.
#[test]
fn duration_readout_truncates_like_epoch() {
    assert_eq!(
        eval_i64("tests/kernel/probe_time_duration_readout_truncates.wat"),
        1,
        "1500ms read as whole seconds truncates to 1",
    );
}

/// The reached-for measurement: an `Instant` delta, read out as a number. This is
/// the exact capability arc 259's timing correction needs — measure
/// `(peer-started-at - started-at)` as nanos/seconds.
#[test]
fn instant_delta_reads_as_a_number() {
    // (now - 5s-ago) is ~5s; read as whole seconds it is >= 4 (truncation slack).
    let got = eval_i64("tests/kernel/probe_time_duration_readout_instant_delta.wat");
    assert!(got >= 4, "a ~5s Instant delta reads as >= 4 whole seconds; got {got}");
}
