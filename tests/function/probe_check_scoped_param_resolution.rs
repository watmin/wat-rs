//! FM-2-bis DIAGNOSTIC PROBE — Stone 249.5e (check-pass scoped param resolution).
//!
//! Does the TYPE CHECKER resolve a macro-generated defclause's scope-tagged params
//! to their DECLARED types, or does it silently fall to the permissive fresh-var
//! arm and lose the type?
//!
//! THE BUG (at HEAD): the check pass binds params with env_key'd (scoped) strings
//! (`func.params` / `clause.args`, Stone 249.5b/d) but resolves body symbols BARE
//! (`check.rs:3397` — `locals.get(ident.as_str())`). A macro-generated body's
//! scope-tagged param reference computes the BARE name, misses the SCOPED bind, and
//! falls to `fresh.fresh()` (`check.rs:3402`, silent-by-intent for genuinely-unknown
//! symbols) → the param's declared type is LOST → a return-type mismatch that the
//! checker SHOULD catch (`infer_defclause`, `check.rs:7955`) is silently suppressed.
//!
//! THE FIX (Stone 249.5e): key the check-pass `locals` by `env_key` uniformly — the
//! lookup AND every `Identifier`-keyed bind — mirroring the runtime `Environment`.
//! The scope-tagged param resolves to its declared `:i64`; the mismatch is caught.
//!
//! Observable: a defclause declaring `-> :wat::core::bool` but returning its
//! `:wat::core::i64` param `x`. `unify(:i64, :bool)` fails → `ReturnTypeMismatch` →
//! `startup_from_file` returns `Err`. At HEAD the MACRO-generated form is accepted
//! (param = fresh var unifies with `:bool`); after the fix it is rejected like the
//! hand-written control.
//!
//! Wat source lives in co-located fixture files slurped via `startup_from_file`.
//! Run: cargo test --release --test probe_check_scoped_param_resolution -- --nocapture

use wat::freeze::startup_from_file;

/// `true` iff the program type-checks clean (freeze succeeds with no CheckError).
fn checks_clean(path: &str) -> bool {
    startup_from_file(path).is_ok()
}

/// CONTROL — a HAND-WRITTEN defclause with the same ret-mismatch must ALWAYS be
/// rejected (its param is bare; bind-key bare == lookup-key bare → resolves to
/// :i64 → mismatch caught). Proves the checker DOES catch this error for user code,
/// isolating macro-template scoping as the sole variable between this and the bug
/// test below. GREEN at HEAD and after the fix.
#[test]
fn handwritten_defclause_ret_mismatch_is_caught() {
    assert!(
        !checks_clean("tests/function/probe_check_scoped_param_resolution_handwritten.wat"),
        "CONTROL: a hand-written defclause returning its :i64 param as :bool must be \
         REJECTED (ReturnTypeMismatch). If this checks clean, the probe's observable \
         is invalid and the bug test below proves nothing."
    );
}

/// THE BUG — a MACRO-GENERATED defclause with the same ret-mismatch must ALSO be
/// rejected. At HEAD it is silently ACCEPTED: the scope-tagged param `x` misses the
/// scoped bind at the bare lookup → `fresh.fresh()` → unifies with `:bool`. Stone
/// 249.5e keys the check-pass locals by `env_key`, so `x` resolves to `:i64` and the
/// mismatch is caught. RED at HEAD (checks clean); GREEN after the fix (rejected).
#[test]
fn macro_generated_defclause_ret_mismatch_is_caught() {
    assert!(
        !checks_clean("tests/function/probe_check_scoped_param_resolution_macro.wat"),
        "CHECK HYGIENE: a macro-generated defclause returning its :i64 param as \
         :bool must be REJECTED (ReturnTypeMismatch), exactly like the hand-written \
         control. At HEAD it is silently accepted — the scope-tagged param missed \
         the scoped bind at the bare lookup (check.rs:3397) and resolved to a fresh \
         var (Stone 249.5e: key the check-pass locals by env_key)."
    );
}
