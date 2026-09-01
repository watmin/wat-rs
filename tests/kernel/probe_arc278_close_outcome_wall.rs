//! Arc 278 peer-lifecycle Strike 2 — the `close'` OUTCOME WALL.
//!
//! `:wat::kernel::close` used to RAISE on its *handleable* teardown failures
//! (thread-join-panic, process-signaled, process-wait-fail, process-stopped)
//! and return a bare `nil`/`i64` on success. Per the peer-lifecycle LAW
//! (2026-07-23) — *"we deliver an enum for code to handle exceptions with; raise
//! is uncatchable on purpose, a thing that must never happen"* — every handleable
//! outcome is now a matchable `:wat::kernel::CloseOutcome` variant:
//!   Closed  [exit <- Option<i64>]   — clean close (None = thread, Some = process exit)
//!   Signaled[signal <- i64]         — process terminated by a signal
//!   Failed  [cause <- Failure]      — join panic / wait failure / stopped-not-terminated
//! Only the must-never-happen raises stay raises (double-close, close'-on-a-timer,
//! arity/type mismatch).
//!
//! # Why a RUST probe with ONE irreducible inline form
//!
//! `close'` is `#[restricted_to(":wat::kernel::")]` — a wat caller must live in the
//! `:wat::kernel::` namespace, which the ReservedPrefix wall forbids a user fixture
//! from defining. So the `close'` CALL cannot live in a loadable `.wat` fixture: the
//! freeze/check pass would reject a `:user::` caller (that rejection IS a banked
//! restriction, `probe_arc259_s2d_internal_only_close.wat.bad`). The only way to
//! drive it is `eval_in_frozen` — READ→EXPAND→EVAL, no check pass — on an inline
//! `(:wat::kernel::close peer)` form, exactly as a future kernel-namespace teardown
//! caller would run it. Everything that CAN live in a fixture does (the peer spawns
//! are the co-located `.wat`); the sole inline wat is the irreducible restricted call.
//!
//! The returned Value is asserted STRUCTURALLY (`Value::Enum` field extraction),
//! never a loose `format!("{:?}").contains(...)`.

// rune:lint(no-inlined-wat) — the ONLY inline wat here is `(:wat::kernel::close peer)`,
// which is :wat::kernel::-restricted: a loadable .wat fixture calling it from a :user::
// entry is rejected at freeze (the banked restriction probe_arc259_s2d_internal_only_close.
// wat.bad), so eval_in_frozen on an inline form (skipping the check pass) is the ONLY
// mechanism to drive it. Every spawn that CAN live in a fixture does (the co-located .wat).

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_beside, FrozenWorld};
use wat::runtime::{apply_function, Environment, TrackedValue, Value};

/// Spawn a peer via the co-located fixture's `spawn_fn`, then drive the
/// kernel-restricted `close'` on it via `eval_in_frozen` (no check pass), returning
/// the `CloseOutcome` Value. Binding the peer to `peer` keeps the ONLY inline wat the
/// irreducible `(:wat::kernel::close peer)` — the call a fixture cannot legally hold.
fn spawn_then_close(spawn_fn: &str) -> Value {
    let world: FrozenWorld = startup_beside(file!()).expect("startup_beside");
    let func = world
        .symbols()
        .get(spawn_fn)
        .unwrap_or_else(|| panic!("fixture defines {spawn_fn}"))
        .clone();
    let peer = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("{spawn_fn} should spawn a peer: {e:?}"));
    let env = Environment::new()
        .child()
        .bind_unknown_span("peer", TrackedValue::from(peer))
        .build();
    let ast = wat::parse_one!("(:wat::kernel::close peer)").expect("parse close' form");
    eval_in_frozen(&ast, &world, &env)
        .unwrap_or_else(|e| panic!("close' should eval to a CloseOutcome, not raise: {e:?}"))
        .value_owned()
}

/// Extract a `:wat::kernel::CloseOutcome` enum value, asserting the type path.
fn as_close_outcome(v: &Value) -> &Arc<wat::runtime::EnumValue> {
    match v {
        Value::Enum(ev) => {
            assert_eq!(
                ev.type_path, ":wat::kernel::CloseOutcome",
                "close' must return CloseOutcome; got type_path {:?} (variant {:?})",
                ev.type_path, ev.variant_name
            );
            ev
        }
        other => panic!(
            "close' must return a CloseOutcome enum value, not a bare value / raise; got {:?}",
            other
        ),
    }
}

// ─── thread clean close → Closed[exit = None] (in-process; runs in the floor) ──

/// A thread self-peer whose worker returns `nil` immediately (a clean exit).
/// `close'` drains + joins the worker (join Ok) → `CloseOutcome::Closed[None]`
/// (None = a thread has no OS exit code — loci-agnostic, R32).
///
/// RED before the wall: close' returned `Value::Unit` for a clean thread close,
/// so `as_close_outcome` panics ("not a CloseOutcome enum value"). GREEN after.
#[test]
fn thread_clean_close_yields_closed_none() {
    let v = spawn_then_close(":user::spawn-noop-thread");
    let ev = as_close_outcome(&v);
    assert_eq!(ev.variant_name, "Closed", "clean thread close is Closed; got {:?}", ev.variant_name);
    assert_eq!(ev.fields.len(), 1, "Closed carries one field (exit); got {:?}", ev.fields);
    match &ev.fields[0] {
        Value::Option(o) if o.as_ref().is_none() => {}
        other => panic!("thread Closed.exit must be None (no OS exit code); got {:?}", other),
    }
}

// ─── process clean exit 0 → Closed[exit = Some(0)] (fork-contained; ignored) ──

/// A `:process` peer whose main returns nil → child exits 0 → `close'` waits →
/// `CloseOutcome::Closed[Some(0)]`. This is the ONLY case exercising the `Some(code)`
/// branch of `crate::kernel::outcome::close_outcome_closed` + the `ExitStatus::Exited` arm.
///
/// `#[ignore]` — process-tier: forks via `spawn-program' :process`; run under
/// setsid + timeout to prevent fd/lock inheritance from the multi-threaded test
/// binary (mirrors `peer_select_prime_process.rs`). Not part of the default floor.
///   setsid timeout 180 cargo test --release --test kernel \
///     probe_arc278_close_outcome_wall -- --ignored --test-threads=1
#[test]
fn process_clean_close_yields_closed_some_zero() {
    let v = spawn_then_close(":user::spawn-noop-process");
    let ev = as_close_outcome(&v);
    assert_eq!(ev.variant_name, "Closed", "clean process close is Closed; got {:?}", ev.variant_name);
    assert_eq!(ev.fields.len(), 1, "Closed carries one field (exit); got {:?}", ev.fields);
    match &ev.fields[0] {
        Value::Option(o) => match o.as_ref() {
            Some(Value::i64(0)) => {}
            other => panic!("process Closed.exit must be Some(0); got {:?}", other),
        },
        other => panic!("process Closed.exit must be an Option; got {:?}", other),
    }
}

// ─── Signaled / Failed — not cheaply reachable; covered by the eval disposition ─
//
// `Signaled[signal]` (a process TERMINATED by a signal) and `Failed[cause]` (a
// thread-join panic, a process wait failure, or a stopped-not-terminated child)
// require a child that signals/panics itself deterministically under the forked,
// fd-inheriting test binary — NOT cheaply reachable, and the brief forbids faking a
// hard-to-reach path. They are constructed by the SAME `close_outcome_*` helpers the
// two Closed cases exercise (identical enum-value construction, differing only in
// variant + payload), and mapped from the `Signaled`/`Stopped`/wait-fail/join-panic
// arms of `eval_peer_close_prime` (`src/kernel/resource.rs`). No live probe asserts them here.
