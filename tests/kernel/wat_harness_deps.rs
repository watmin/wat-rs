//! Integration: `wat::Harness::from_source_with_deps` under arc 015
//! slice 3a's global-install-once architecture.
//!
//! Dep sources and Rust shims install process-globally via OnceLock
//! (first-caller-wins). One test binary = one consistent dep set.
//! Tests needing different dep sets live in separate `tests/*.rs`
//! files — Cargo compiles each to its own test binary where the
//! install race doesn't cross.
//!
//! This file installs ONE dep set (two co-located `.wat` fixtures, read via
//! `include_str!` into `WatSource`s) and exercises it from multiple entry-source
//! shapes. The pattern mirrors how a consumer crate would use
//! `Harness::from_source_with_deps` at test time: one superset, many callers.
//!
//! Arc 170 slice 1f-ζ: migrated from 3-arg main + stdout-capture to
//! canonical nil main + eval_in_frozen via h.world(). Dep presence
//! verified through symbol lookup + eval.
//!
//! no_inlined_wat: every wat source here (deps + the shared canonical-nil
//! `:user::main`) lives in a co-located `.wat` fixture beside this file, pulled
//! in via `include_str!`; drivers fetch the named fn off `h.world().symbols()`
//! and `apply_function` it — no inline wat driver strings.

use wat::runtime::{apply_function, Value};
use wat::host::harness::Harness;
use wat::WatSource;

/// Two co-located `.wat` fixtures — stand-ins for what an external wat crate's
/// `wat_sources()` would return. Both under `:user::*` per arc 013's namespace
/// convention.
const DEP_A: &[WatSource] = &[WatSource {
    path: "test-harness-deps/a.wat",
    source: include_str!("wat_harness_deps_dep_a.wat"),
}];
const DEP_B: &[WatSource] = &[WatSource {
    path: "test-harness-deps/b.wat",
    source: include_str!("wat_harness_deps_dep_b.wat"),
}];

/// The canonical nil `:user::main` shared by every test in this file — co-located
/// (tests/kernel/wat_harness_deps_user_main.wat), never inlined.
const USER_MAIN: &str = include_str!("wat_harness_deps_user_main.wat");

/// Fetch a zero-arg fn off the harness's frozen world and apply it.
fn call(h: &Harness, fn_name: &str) -> Value {
    let world = h.world();
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name} registered in harness world"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("eval {fn_name} failed: {e:?}"))
}

// Fresh OnceLock state per test comes from the RUNNER: nextest gives
// every test its own forked process (.config/nextest.toml). The old
// hand-rolled `run_in_fork` wrapper here predated that and duplicated it;
// annihilated once the floor moved to `cargo nextest run --release`.

#[test]
fn harness_composes_multiple_deps_into_user_source() {
    // Arc 170 slice 1f-ζ: canonical nil main; dep functions verified
    // via eval_in_frozen on the frozen world.
    let h = Harness::from_source_with_deps(USER_MAIN, &[DEP_A, DEP_B], &[]).expect("freeze");
    let out = h.run(&[]).expect("run");
    // Arc 170: stdio capture retired — stdout/stderr are always empty.
    assert!(out.stdout.is_empty());
    assert!(out.stderr.is_empty());
    // Verify both dep functions are registered in the frozen world.
    let world = h.world();
    assert!(world.symbols().get(":user::test::dep-a::label").is_some(),
            "expected dep-a to be registered");
    assert!(world.symbols().get(":user::test::dep-b::label").is_some(),
            "expected dep-b to be registered");
    // Verify dep-a returns "A" and dep-b returns "B" via eval.
    let val_a = call(&h, ":user::test::dep-a::label");
    let val_b = call(&h, ":user::test::dep-b::label");
    assert!(matches!(val_a, Value::String(ref s) if &**s == "A"), "expected dep-a to return 'A'; got {:?}", val_a);
    assert!(matches!(val_b, Value::String(ref s) if &**s == "B"), "expected dep-b to return 'B'; got {:?}", val_b);
}

#[test]
fn harness_same_deps_usable_from_different_entry_source() {
    // Arc 170 slice 1f-ζ: canonical nil main; dep-a verified via eval.
    let h = Harness::from_source_with_deps(USER_MAIN, &[DEP_A, DEP_B], &[]).expect("freeze");
    let out = h.run(&[]).expect("run");
    // Arc 170: stdio capture retired.
    assert!(out.stdout.is_empty());
    assert!(out.stderr.is_empty(), "expected empty stderr; got {:?}", out.stderr);
    // Verify dep-a returns "A" via eval_in_frozen.
    let val = call(&h, ":user::test::dep-a::label");
    assert!(matches!(val, Value::String(ref s) if &**s == "A"),
            "expected dep-a to return 'A'; got {:?}", val);
}

#[test]
fn harness_with_zero_deps_matches_from_source() {
    // Arc 170 slice 1f-ζ: canonical nil main. Passing &[] uses no deps.
    // Verify both harness constructions succeed and run returns Ok.
    let h_no_deps = Harness::from_source_with_deps(USER_MAIN, &[], &[]).expect("freeze-empty-deps");
    let h_ref = Harness::from_source(USER_MAIN).expect("freeze-from-source");
    let out_a = h_no_deps.run(&[]).expect("run-no-deps");
    let out_b = h_ref.run(&[]).expect("run-from-source");
    // Arc 170: stdio capture retired — both return empty stdout/stderr.
    assert_eq!(out_a.stdout, out_b.stdout);
    assert!(out_a.stdout.is_empty());
}

// ─── Retired tests ──────────────────────────────────────────────────
//
// - `harness_composes_user_source_with_one_dep` — subsumed by
//   `harness_composes_multiple_deps_into_user_source`. One-dep is
//   a trivial case of multi-dep.
//
// - `harness_accepts_dep_registrar_for_rust_shim` — the slice-4a
//   probe. Retired in slice 4b (see arc 013 BACKLOG); registrar
//   plumbing is end-to-end-proven in `crates/wat-lru/tests/wat_suite.rs`.
//
// - `harness_dep_declaring_under_wat_namespace_is_rejected` —
//   retired in arc 015 slice 3a. Dep sources now flow through the
//   stdlib pipeline (global install) rather than the user-tier
//   reserved-prefix gate. Community discipline via `:user::*`
//   namespace convention + duplicate-define collision detection
//   carry the protection. The reserved-prefix gate still applies
//   to the USER's own source — user code under `:wat::*` fails
//   loud, which is what genuinely needed protecting.
