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
//! Tests that were order-fragile under the old `dep_sources`-as-
//! parameter shape (per-test different dep sets in one process)
//! retired here — equivalent coverage now lives in
//! `crates/wat-lru/tests/wat_suite.rs` where a real external wat
//! crate owns its own test binary.

use wat::harness::{Harness, Outcome};
use wat::WatSource;

/// Two in-memory dep "files" — stand-ins for what an external wat
/// crate's `wat_sources()` would return. Both under `:user::*`
/// per arc 013's namespace convention.
const DEP_A: &[WatSource] = &[WatSource {
    path: "test-harness-deps/a.wat",
    source: r#"
        (:wat::core::define
          (:user::test::dep-a::label -> :String)
          "A")
    "#,
}];
const DEP_B: &[WatSource] = &[WatSource {
    path: "test-harness-deps/b.wat",
    source: r#"
        (:wat::core::define
          (:user::test::dep-b::label -> :String)
          "B")
    "#,
}];

#[test]
fn harness_composes_multiple_deps_into_user_source() {
    let user = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)

        (:wat::core::define (:user::main
                             (stdin  :wat::io::IOReader)
                             (stdout :wat::io::IOWriter)
                             (stderr :wat::io::IOWriter)
                             -> :())
          (:wat::core::let*
            (((_ :i64) (:wat::io::IOWriter/writeln stdout (:user::test::dep-a::label)))
             ((_ :i64) (:wat::io::IOWriter/writeln stdout (:user::test::dep-b::label))))
            ()))
    "#;
    let h = Harness::from_source_with_deps(user, &[DEP_A, DEP_B], &[]).expect("freeze");
    let out = h.run(&[]).expect("run");
    assert_eq!(out.stdout, vec!["A".to_string(), "B".to_string()]);
}

#[test]
fn harness_same_deps_usable_from_different_entry_source() {
    // Same two deps (the process-global install from whichever test
    // won first), different entry program — proves deps survive
    // across multiple Harness construction calls in one process.
    let user = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define (:user::main
                             (stdin  :wat::io::IOReader)
                             (stdout :wat::io::IOWriter)
                             (stderr :wat::io::IOWriter)
                             -> :())
          (:wat::io::IOWriter/println stdout (:user::test::dep-a::label)))
    "#;
    let h = Harness::from_source_with_deps(user, &[DEP_A, DEP_B], &[]).expect("freeze");
    let Outcome { stdout, stderr } = h.run(&[]).expect("run");
    assert_eq!(stdout, vec!["A".to_string()]);
    assert!(stderr.is_empty(), "expected empty stderr; got {:?}", stderr);
}

#[test]
fn harness_with_zero_deps_matches_from_source() {
    // Passing &[] uses no deps. Regardless of the process-global
    // install state from other tests, this program only uses baked
    // stdlib, so both entry paths produce identical output.
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::define (:user::main
                             (stdin  :wat::io::IOReader)
                             (stdout :wat::io::IOWriter)
                             (stderr :wat::io::IOWriter)
                             -> :())
          (:wat::io::IOWriter/println stdout "no deps"))
    "#;
    let h_no_deps = Harness::from_source_with_deps(src, &[], &[]).expect("freeze-empty-deps");
    let h_ref = Harness::from_source(src).expect("freeze-from-source");
    let out_a = h_no_deps.run(&[]).expect("run-no-deps");
    let out_b = h_ref.run(&[]).expect("run-from-source");
    assert_eq!(out_a, out_b);
    assert_eq!(out_a.stdout, vec!["no deps".to_string()]);
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