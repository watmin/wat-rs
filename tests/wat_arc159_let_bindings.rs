//! Integration tests for arc 159 slice 1 — untyped `:wat::core::let`
//! bindings (V2; user-visible).
//!
//! Arc 159 drops the per-binding type annotation `:T` from
//! `:wat::core::let`. Each binding's type is inferred from its
//! expression — same lesson as arc 145 / arc 157 (`def`), applied to
//! the inner-binding slot.
//!
//! | Before (legacy) | After (canonical) |
//! |---|---|
//! | `(let (((name :T) expr) ...) body)` | `(let ((name expr) ...) body)` |
//!
//! ## Test structure
//!
//! Integration tests cover the WALKER (tests 6-8): `LegacyTypedLetBinding`
//! fires per legacy `((name :T) expr)` binding site. Runtime end-to-end
//! (tests 1-5) and check-level (test 11) tests live in src/ unit tests
//! to avoid the stdlib legacy-binding firings that break `startup_from_source`
//! during the migration window (expected per BRIEF § "The workspace WILL
//! break post-substrate-change").
//!
//! Tests 9-10 (destructure preservation) also live in src/runtime.rs
//! unit tests; arc 158 v1's destructure-mangling bug targeted `eval_let`
//! directly.

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// Startup that MUST fail. Returns the `Debug`-formatted error bundle
/// so tests can assert which variants appear in the output.
fn startup_err(src: &str) -> String {
    match startup_from_source(src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{:?}", e),
    }
}

// ─── Walker on legacy shape — 3 tests ────────────────────────────────────────
//
// These tests use `startup_from_source` which loads the stdlib. During
// the arc 159 migration window the stdlib has ~951 legacy binding sites
// that also fire `LegacyTypedLetBinding`. The assertions below verify
// that the ENTRY-FILE legacy bindings fire (identified by
// `file: "<entry>"` in the error output) — not counting stdlib sites.

/// Test 6 — single legacy binding fires `LegacyTypedLetBinding` walker.
///
/// The legacy form `((x :wat::core::i64) 2)` must cause startup to fail
/// with `LegacyTypedLetBinding` referencing the entry file and binding
/// name `x`. Consumer sweep (slice 2) uses this diagnostic stream as
/// the mechanical work list.
#[test]
fn walker_fires_on_single_legacy_binding() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let
            (((x :wat::core::i64) 2))
            x))
    "#;
    let err = startup_err(src);
    // The entry-file legacy binding `x` must appear.
    assert!(
        err.contains("LegacyTypedLetBinding"),
        "arc 159: expected LegacyTypedLetBinding; got: {}",
        err
    );
    assert!(
        err.contains("binding_name: \"x\""),
        "arc 159: expected binding_name: \"x\" in error; got: {}",
        err
    );
    assert!(
        err.contains("file: \"<entry>\""),
        "arc 159: expected entry-file LegacyTypedLetBinding; got: {}",
        err
    );
}

/// Test 7 — multi-binding all-legacy: walker fires per binding.
///
/// Three legacy bindings in one `let`; walker emits one
/// `LegacyTypedLetBinding` per binding site. Verifies the walker
/// iterates all bindings, not just the first. We check that all three
/// binding names (`a`, `b`, `c`) appear in the entry-file errors.
#[test]
fn walker_fires_per_legacy_binding_in_multi_binding_let() {
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let
            (((a :wat::core::i64) 1)
             ((b :wat::core::i64) 2)
             ((c :wat::core::i64) 3))
            (:wat::core::i64::+,2 a (:wat::core::i64::+,2 b c))))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("LegacyTypedLetBinding"),
        "arc 159: expected LegacyTypedLetBinding on multi-binding all-legacy; got: {}",
        err
    );
    // All three binding names must appear.
    assert!(
        err.contains("binding_name: \"a\""),
        "arc 159: expected binding `a` in error; got: {}",
        err
    );
    assert!(
        err.contains("binding_name: \"b\""),
        "arc 159: expected binding `b` in error; got: {}",
        err
    );
    assert!(
        err.contains("binding_name: \"c\""),
        "arc 159: expected binding `c` in error; got: {}",
        err
    );
}

/// Test 8 — mixed (legacy + new) in one let: walker fires only on
/// legacy binding(s); new binding passes silently.
///
/// `(let ((a 1) ((b :wat::core::i64) 2)) (+ a b))` — `a` is new
/// shape (no error); `b` is legacy (one `LegacyTypedLetBinding` fires
/// with `binding_name: "b"`). `a` (new shape) must NOT appear in entry-
/// file errors.
#[test]
fn walker_fires_only_on_legacy_in_mixed_let() {
    // Name the new-shape binding something distinctive (`uniq_a_159`)
    // so we can verify it's absent from the entry-file error list.
    let src = r#"
        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::let
            ((uniq_a_159 1)
             ((uniq_b_159 :wat::core::i64) 2))
            (:wat::core::i64::+,2 uniq_a_159 uniq_b_159)))
    "#;
    let err = startup_err(src);
    // Legacy `uniq_b_159` fires.
    assert!(
        err.contains("binding_name: \"uniq_b_159\""),
        "arc 159 mixed let: expected LegacyTypedLetBinding for uniq_b_159; got: {}",
        err
    );
    // New-shape `uniq_a_159` must NOT be in the error (its name doesn't
    // appear as a LegacyTypedLetBinding binding_name).
    assert!(
        !err.contains("binding_name: \"uniq_a_159\""),
        "arc 159 mixed let: uniq_a_159 (new shape) must NOT fire LegacyTypedLetBinding; got: {}",
        err
    );
}
