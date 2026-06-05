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
//! `startup_from_source` returns `Err`. At HEAD the MACRO-generated form is accepted
//! (param = fresh var unifies with `:bool`); after the fix it is rejected like the
//! hand-written control.
//!
//! Run: cargo test --release --test probe_check_scoped_param_resolution -- --nocapture

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

/// `true` iff the program type-checks clean (freeze succeeds with no CheckError).
fn checks_clean(src: &str) -> bool {
    startup_from_source(src, None, Arc::new(InMemoryLoader::new())).is_ok()
}

// A user::main so the ONLY possible check error is the defclause's ret-mismatch
// (not a missing-main error).
const MAIN: &str = "(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

// A macro expanding to a defclause whose body (the param `x`, declared :i64) does
// NOT match the declared return type (:bool). `walk_template` scope-tags `x` in
// both the binder and the body reference.
const MAKE_BAD_RET: &str = "\
(:wat::core::defmacro :test::make-bad-ret \
  [] -> :AST<wat::holon::HolonAST> \
  `(:wat::core::defclause :test::bad-ret \
     ([x <- :wat::core::i64] -> :wat::core::bool x)))";

const CALL_MAKE_BAD_RET: &str = "(:test::make-bad-ret)";

/// CONTROL — a HAND-WRITTEN defclause with the same ret-mismatch must ALWAYS be
/// rejected (its param is bare; bind-key bare == lookup-key bare → resolves to
/// :i64 → mismatch caught). Proves the checker DOES catch this error for user code,
/// isolating macro-template scoping as the sole variable between this and the bug
/// test below. GREEN at HEAD and after the fix.
#[test]
fn handwritten_defclause_ret_mismatch_is_caught() {
    let src = format!(
        "(:wat::core::defclause :test::bad-ret-direct \
           ([x <- :wat::core::i64] -> :wat::core::bool x))\n{MAIN}"
    );
    assert!(
        !checks_clean(&src),
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
#[ignore = "RED until Stone 249.5e lands (check pass keys locals by env_key); the strike removes this #[ignore] and the test must then pass (macro-gen ret-mismatch rejected)"]
fn macro_generated_defclause_ret_mismatch_is_caught() {
    let src = format!("{MAKE_BAD_RET}\n{CALL_MAKE_BAD_RET}\n{MAIN}");
    assert!(
        !checks_clean(&src),
        "CHECK HYGIENE: a macro-generated defclause returning its :i64 param as \
         :bool must be REJECTED (ReturnTypeMismatch), exactly like the hand-written \
         control. At HEAD it is silently accepted — the scope-tagged param missed \
         the scoped bind at the bare lookup (check.rs:3397) and resolved to a fresh \
         var (Stone 249.5e: key the check-pass locals by env_key)."
    );
}
