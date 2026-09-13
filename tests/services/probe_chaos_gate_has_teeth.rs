//! the-chaos-gate-has-teeth — a floor harness that drives `circuit.wat`
//! with chaos argv and fails on an inert injector.
//!
//! Nothing automated had ever set a chaos knob. `:user::chaos` prints and
//! returns `nil`. These tests drive the returning siblings:
//!   `:user::chaos-fires`        — shipped 200 bp, n=50, seed 42
//!                                (n=2000 timed out at 40 s on the floor;
//!                                `:user::chaos` itself stays n=2000)
//!   `:user::chaos-fires-always` — 10000 bp (every draw fires), n=50, seed 42
//!
//! Assertions are relations, not thresholds on a die:
//!   disrupt-draws > 0                 both rates (the alarm is time-based)
//!   disrupts == disrupt-fires         both rates (every fire tears)
//!   disrupt-fires == disrupt-draws    10000 bp only
//! ⛔ Do not assert `fires > 0` at 200 bp (≈1-in-170 flake).
//! ⛔ Do not assert a draw count (4 vs 5 on two runs of one command).
//! ⛔ Do not assert `dup=0` — this gate proves the injector fires and tears,
//!    not that the system survives chaos.
//!
//! `startup_from_file` uses `InMemoryLoader` and cannot resolve this file's
//! relative `load-file!`. Drive it the way `probe_ex001_fanout.rs` does.

use std::sync::Arc;
use wat::freeze::{startup_from_source, FrozenWorld};
use wat::load::loader::FsLoader;
use wat::runtime::{apply_function, Value};

fn load_circuit() -> FrozenWorld {
    let rel = "wat-scripts/fanout/circuit.wat";
    let src = std::fs::read_to_string(rel).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    startup_from_source(&src, Some(rel), Arc::new(FsLoader))
        .expect("circuit should freeze")
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

/// The phase line is `third` of `run-chaos*` (`phases ;; traces ;; inbox…`).
/// Split on `;` then `=`, same helper as `probe_ex001_fanout.rs`.
fn field<'a>(summary: &'a str, key: &str) -> &'a str {
    let phase = summary.split(" ;; ").next().unwrap_or(summary);
    for part in phase.split(';') {
        if let Some((k, v)) = part.split_once('=') {
            if k == key {
                return v;
            }
        }
    }
    panic!("summary missing field {key:?}: {summary}");
}

fn i64_field(summary: &str, key: &str) -> i64 {
    field(summary, key)
        .parse()
        .unwrap_or_else(|_| panic!("{key} not an i64 in {summary}"))
}

#[test]
fn chaos_gate_shipped_rate_draws_and_hits_eq_fires() {
    let world = load_circuit();
    let stored = call_string(&world, ":user::chaos-fires");
    let draws = i64_field(&stored, "disrupt-draws");
    let fires = i64_field(&stored, "disrupt-fires");
    let hits = i64_field(&stored, "disrupts");
    eprintln!("chaos-fires draws={draws} fires={fires} hits={hits} bp-disrupt={}", field(&stored, "bp-disrupt"));
    assert!(
        draws > 0,
        "inert injector: disrupt-draws must be > 0 at the shipped 200 bp (alarm is time-based); got {stored}"
    );
    assert_eq!(
        hits, fires,
        "every fire tears: disrupts (hits) must equal disrupt-fires; got {stored}"
    );
}

#[test]
fn chaos_gate_full_rate_fires_eq_draws() {
    let world = load_circuit();
    let stored = call_string(&world, ":user::chaos-fires-always");
    let draws = i64_field(&stored, "disrupt-draws");
    let fires = i64_field(&stored, "disrupt-fires");
    let hits = i64_field(&stored, "disrupts");
    eprintln!("chaos-fires-always draws={draws} fires={fires} hits={hits} bp-disrupt={}", field(&stored, "bp-disrupt"));
    assert!(
        draws > 0,
        "inert injector: disrupt-draws must be > 0 at 10000 bp; got {stored}"
    );
    assert_eq!(
        hits, fires,
        "every fire tears: disrupts (hits) must equal disrupt-fires; got {stored}"
    );
    assert_eq!(
        fires, draws,
        "10000 bp fires on every draw: disrupt-fires must equal disrupt-draws; got {stored}"
    );
}
