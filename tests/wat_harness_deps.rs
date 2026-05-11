//! Integration: `wat::Harness::from_source_with_deps` under arc 015
//! slice 3a's global-install-once architecture.
//!
//! Dep sources and Rust shims install process-globally via OnceLock
//! (first-caller-wins). One test binary = one consistent dep set.
//! Tests needing different dep sets live in separate `tests/*.rs`
//! files — Cargo compiles each to its own test binary where the
//! install race doesn't cross.
//!
//! This file installs ONE dep set (two in-memory `.wat` files) and
//! exercises it from multiple entry-source shapes. The pattern
//! mirrors how a consumer crate would use `Harness::from_source_with_deps`
//! at test time: one superset, many callers.
//!
//! Arc 170 slice 1f-ζ: migrated from 3-arg main + stdout-capture to
//! canonical nil main + eval_in_frozen via h.world(). Dep presence
//! verified through symbol lookup + eval.

use wat::freeze::eval_in_frozen;
use wat::harness::Harness;
use wat::runtime::{Environment, Value};
use wat::WatSource;

/// Two in-memory dep "files" — stand-ins for what an external wat
/// crate's `wat_sources()` would return. Both under `:user::*`
/// per arc 013's namespace convention.
const DEP_A: &[WatSource] = &[WatSource {
    path: "test-harness-deps/a.wat",
    source: r#"
        (:wat::core::define
          (:user::test::dep-a::label -> :wat::core::String)
          "A")
    "#,
}];
const DEP_B: &[WatSource] = &[WatSource {
    path: "test-harness-deps/b.wat",
    source: r#"
        (:wat::core::define
          (:user::test::dep-b::label -> :wat::core::String)
          "B")
    "#,
}];

// Each test body runs in a forked child — fresh OnceLock state per
// test, no race between the three Harness::from_source_with_deps
// callers even though cargo runs them in parallel within one binary.
// `wat::fork::run_in_fork` is the promoted helper (was private in
// runtime.rs::tests as in_signal_subprocess; exposed publicly in arc
// 024 slice 0 for general test isolation).

#[test]
fn harness_composes_multiple_deps_into_user_source() {
    wat::fork::run_in_fork(|| {
        // Arc 170 slice 1f-ζ: canonical nil main; dep functions verified
        // via eval_in_frozen on the frozen world.
        let user = r#"
            (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
        "#;
        let h = Harness::from_source_with_deps(user, &[DEP_A, DEP_B], &[]).expect("freeze");
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
        let env = Environment::new();
        let ast_a = wat::parse_one!("(:user::test::dep-a::label)").expect("parse a");
        let ast_b = wat::parse_one!("(:user::test::dep-b::label)").expect("parse b");
        let val_a = eval_in_frozen(&ast_a, &world, &env).expect("eval a");
        let val_b = eval_in_frozen(&ast_b, &world, &env).expect("eval b");
        assert!(matches!(val_a, Value::String(ref s) if &**s == "A"), "expected dep-a to return 'A'; got {:?}", val_a);
        assert!(matches!(val_b, Value::String(ref s) if &**s == "B"), "expected dep-b to return 'B'; got {:?}", val_b);
    });
}

#[test]
fn harness_same_deps_usable_from_different_entry_source() {
    wat::fork::run_in_fork(|| {
        // Arc 170 slice 1f-ζ: canonical nil main; dep-a verified via eval.
        let user = r#"
            (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
        "#;
        let h = Harness::from_source_with_deps(user, &[DEP_A, DEP_B], &[]).expect("freeze");
        let out = h.run(&[]).expect("run");
        // Arc 170: stdio capture retired.
        assert!(out.stdout.is_empty());
        assert!(out.stderr.is_empty(), "expected empty stderr; got {:?}", out.stderr);
        // Verify dep-a returns "A" via eval_in_frozen.
        let world = h.world();
        let env = Environment::new();
        let ast = wat::parse_one!("(:user::test::dep-a::label)").expect("parse dep-a");
        let val = eval_in_frozen(&ast, &world, &env).expect("eval dep-a");
        assert!(matches!(val, Value::String(ref s) if &**s == "A"),
                "expected dep-a to return 'A'; got {:?}", val);
    });
}

#[test]
fn harness_with_zero_deps_matches_from_source() {
    wat::fork::run_in_fork(|| {
        // Arc 170 slice 1f-ζ: canonical nil main. Passing &[] uses no deps.
        // Verify both harness constructions succeed and run returns Ok.
        let src = r#"
            (:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)
        "#;
        let h_no_deps = Harness::from_source_with_deps(src, &[], &[]).expect("freeze-empty-deps");
        let h_ref = Harness::from_source(src).expect("freeze-from-source");
        let out_a = h_no_deps.run(&[]).expect("run-no-deps");
        let out_b = h_ref.run(&[]).expect("run-from-source");
        // Arc 170: stdio capture retired — both return empty stdout/stderr.
        assert_eq!(out_a.stdout, out_b.stdout);
        assert!(out_a.stdout.is_empty());
    });
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
