//! Excursus 003 step 3a — the duplicate-builtin guard.
//!
//! `TypeEnv::register_builtin`'s duplicate check used to be a bare
//! `debug_assert!`, which the release floor (`wat-rs/CLAUDE.md`: "the floor is
//! weighed in RELEASE") never runs. A real double registration — 27
//! `:wat::runtime::<Variant>` records, arc 278's hand-typed
//! `register_runtime_error_variants` table silently overwriting this stone's
//! wat-derived registrations — went unnoticed through every green `--release`
//! floor and was caught only by an accidental debug-mode run. This asserts the
//! fixed point in RELEASE: a healthy `TypeEnv::with_builtins()` records zero
//! duplicates, so a reintroduced double registration goes RED here, not only
//! under a debug build nobody runs as the floor.

use wat::types::TypeEnv;

#[test]
fn with_builtins_records_no_duplicates() {
    let env = TypeEnv::with_builtins();
    let dups: &[String] = env.duplicate_builtins();
    assert!(
        dups.is_empty(),
        "TypeEnv::with_builtins() registered at least one builtin type twice: {dups:?}"
    );
}
