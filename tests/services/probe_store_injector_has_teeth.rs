//! the-store-injector-has-teeth — a floor harness that drives
//! `wat-scripts/scratch-pad/probe-accepted-is-a-count.wat` (which loads
//! `wat-scripts/query/faulting-store.wat`) and fails if `Accepted n` is
//! no longer the store's true count after a lost reply.
//!
//! Nothing automated had ever set `drop-reply-bp` / `die-bp`. The probe
//! already exposes three zero-arg String entry points; `:user::main` only
//! prints them. This file does not change the probe or the proxy.
//!
//!   `:sf::run-passthrough`  rates 0
//!   `:sf::run-timedout`     drop-reply-bp 10000  (the D1-c invariant)
//!   `:sf::run-die`          die-bp 10000
//!
//! Assertions are equalities (and one `drops-fired > 0` to prove the fault
//! fired). ⛔ Do not pin `drops-fired` to a count. ⛔ Do not assert elapsed-ms.
//! ⛔ Three tests, not one: the drop-reply path waits the 10 s client deadline.
//!
//! `startup_from_file` uses `InMemoryLoader` and cannot resolve this file's
//! relative `load-file!`. Drive it the way `probe_chaos_gate_has_teeth.rs` does.

use std::sync::Arc;
use wat::freeze::{startup_from_source, FrozenWorld};
use wat::load::loader::FsLoader;
use wat::runtime::{apply_function, Value};

fn load_probe() -> FrozenWorld {
    let rel = "wat-scripts/scratch-pad/probe-accepted-is-a-count.wat";
    let src = std::fs::read_to_string(rel).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    startup_from_source(&src, Some(rel), Arc::new(FsLoader))
        .expect("accepted-is-a-count probe should freeze")
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
            // Probe labels the line (`passthrough send=…`); the key is the
            // last whitespace-separated token before `=`.
            let k = k.rsplit(' ').next().unwrap_or(k);
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

fn accepted_n(send: &str) -> i64 {
    send.strip_prefix("Accepted ")
        .and_then(|rest| rest.parse().ok())
        .unwrap_or_else(|| panic!("send is not Accepted n: {send:?}"))
}

#[test]
fn store_injector_passthrough_n_eq_real_rows() {
    let world = load_probe();
    let stored = call_string(&world, ":sf::run-passthrough");
    let n = accepted_n(field(&stored, "send"));
    let rows = i64_field(&stored, "real-rows");
    eprintln!("passthrough {stored}");
    assert_eq!(
        n, rows,
        "passthrough: Accepted n must equal real-rows; got {stored}"
    );
}

#[test]
fn store_injector_drop_reply_n_eq_real_rows() {
    let world = load_probe();
    let stored = call_string(&world, ":sf::run-timedout");
    let n = accepted_n(field(&stored, "send"));
    let rows = i64_field(&stored, "real-rows");
    let drops = i64_field(&stored, "drops-fired");
    eprintln!("drop-reply {stored}");
    assert!(
        drops > 0,
        "drop-reply: fault must have fired (drops-fired > 0); got {stored}"
    );
    assert_eq!(
        n, rows,
        "drop-reply: Accepted n must equal real-rows after the reply died; got {stored}"
    );
}

#[test]
fn store_injector_die_write_landed_and_no_success() {
    let world = load_probe();
    let stored = call_string(&world, ":sf::run-die");
    let send = field(&stored, "send");
    let rows = i64_field(&stored, "real-rows");
    eprintln!("die {stored}");
    // ⛔ The client-visible OUTCOME on the die path is a LOAD-DEPENDENT RACE, measured:
    //   quiet box   10/10  TimedOut @10000ms   (6 via :user::main, 4 via this test alone)
    //   8 spinners   1/4    Lost @5ms
    //   the floor    Lost @20ms                (.floor/2026-09-13T09-04-47Z/, ARM.txt kept)
    // Both are correct consequences of the proxy exiting after forwarding: the queue
    // redials, fails, raises ("queue: redial failed — peer is dead, not a broken pipe")
    // and dies — so the client either learns the queue is gone (Lost) or gives up first
    // (TimedOut). An earlier revision of this test pinned "TimedOut" and reddened the
    // floor on correct code: that gated an OBSERVATION as an invariant.
    // What IS invariant in all 15 observations: the write landed, and the client never
    // saw a success.
    // Exact comparisons, not `starts_with` — `no_loose_string_assert` caught that and is right to:
    // enumerating the two MEASURED outcomes is tighter than a prefix test, because a NEW third
    // outcome would red here instead of being silently accepted.
    assert!(
        send == "Lost" || send == "TimedOut",
        "die: the client must see Lost (queue died) or TimedOut (client gave up first) — never a          success; got {stored}"
    );
    assert_eq!(
        rows, 1,
        "die: write landed in the real store (real-rows == 1); got {stored}"
    );
}
