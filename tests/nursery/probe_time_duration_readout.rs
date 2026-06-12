//! Reach-stumble enrichment — `Duration` is a WRITE-ONLY value; mint the readout family.
//!
//! The time-ops surface is asymmetric. `Instant` round-trips: you build one from
//! an i64 (`at` / `at-millis` / `at-nanos`) and read it back to an i64
//! (`epoch-seconds` / `epoch-millis` / `epoch-nanos`). `Duration` only has the
//! IN half: seven unit constructors (`Nanosecond` … `Day`) build one from an i64,
//! you can add/subtract and compare Durations — but there is NO way to read the
//! number back OUT in any unit. A program can compute `(now - started-at)` and
//! never learn how long it was.
//!
//! This probe reaches for that missing readout — the inverse of `epoch-*`, the
//! seven-unit symmetric mirror of the constructors — and RED-fails at HEAD
//! because the verbs do not exist (undefined function; either at check via the
//! reserved-prefix path or at eval).
//!
//! Proposed family (intueri to ratify the spelling; the FAMILY is locked by
//! symmetry — 7 constructors deserve 7 readouts, truncating like `epoch-millis`):
//!   as-nanoseconds · as-microseconds · as-milliseconds · as-seconds ·
//!   as-minutes · as-hours · as-days        ;; each :wat::time::Duration -> :i64
//!
//! Run: `cargo test --release -p wat --test nursery probe_time_duration_readout`

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Eval a body that returns `:i64` in a frozen world. Panics (RED) if startup or
/// eval fails — at HEAD the undefined readout verb makes eval fail here.
fn eval_i64(body: &str) -> i64 {
    let src = format!(
        "(:wat::core::defn :user::compute [] -> :wat::core::i64 {body})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)"
    );
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup/check should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("compute eval")
        .value_owned()
    {
        Value::i64(n) => n,
        other => panic!("expected i64; got {other:?}"),
    }
}

/// Identity round-trip: build in a unit, read back in the SAME unit.
#[test]
fn duration_reads_back_in_same_unit() {
    assert_eq!(
        eval_i64("(:wat::time::as-milliseconds (:wat::time::Millisecond 1500))"),
        1500,
        "a Duration built as 1500ms reads back as 1500ms",
    );
}

/// Cross-unit conversion: 1ms = 1_000_000ns.
#[test]
fn duration_reads_across_units() {
    assert_eq!(
        eval_i64("(:wat::time::as-nanoseconds (:wat::time::Millisecond 1))"),
        1_000_000,
        "1ms read as nanoseconds is 1_000_000",
    );
}

/// Truncating toward zero, exactly like `epoch-millis`: 1500ms read as whole
/// seconds is 1.
#[test]
fn duration_readout_truncates_like_epoch() {
    assert_eq!(
        eval_i64("(:wat::time::as-seconds (:wat::time::Millisecond 1500))"),
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
    let got = eval_i64(
        "(:wat::time::as-seconds \
           (:wat::time::- (:wat::time::now) (:wat::time::seconds-ago 5)))",
    );
    assert!(got >= 4, "a ~5s Instant delta reads as >= 4 whole seconds; got {got}");
}
