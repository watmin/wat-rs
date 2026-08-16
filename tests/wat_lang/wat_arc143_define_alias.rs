//! Integration coverage for arc 143 slice 6 — alias binding.
//!
//! Stone 241.12 — migrated from `:wat::runtime::define-alias` defmacro to
//! `:wat::core::defalias` native substrate form.
//!
//! The native form is parsed + registered in Rust at `src/runtime.rs::register_defalias`.
//! No macro expansion required; the alias is available immediately after step 6
//! (register_defines), not deferred to macro expansion at step 4.
//!
//! Tests:
//!   1. Alias a substrate primitive (:wat::core::foldl) — native registration resolves
//!      the builtin via CheckEnv::with_builtins(); alias delegates correctly.
//!   2. Alias another substrate primitive (:wat::core::length) — verifies
//!      the native form works for multiple targets.
//!   3. Alias an unknown target — the native form registers a stub; the HARD CUT
//!      for :wat::runtime::define-alias fires with a retirement remedy.

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;

fn run_expr(name: &str) -> Value {
    call_beside_value(file!(), name).expect("eval should succeed")
}

// ─── Test 1: alias :wat::core::foldl — native registration resolves builtin ──

#[test]
fn define_alias_foldl_to_user_fold_delegates_correctly() {
    match run_expr(":t::test1-foldl-alias") {
        Value::i64(n) => assert_eq!(n, 10, "expected alias of foldl to sum [1,2,3,4] from 0 → 10; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Test 2: alias :wat::core::length ────────────────────────────────────────

#[test]
fn define_alias_length_to_user_size_delegates_correctly() {
    match run_expr(":t::test2-length-alias") {
        Value::i64(n) => assert_eq!(n, 3, "expected alias of length to return 3 for Vec of 3 elements; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Test 3: retired form :wat::runtime::define-alias is HARD-CUT-rejected ───

#[test]
fn define_alias_retired_form_rejected_at_startup() {
    let result = startup_from_file("tests/wat_lang/wat_arc143_define_alias_retired.wat.bad");
    assert!(
        result.is_err(),
        "expected startup to fail for retired :wat::runtime::define-alias form; got Ok"
    );
    // Verify the error message names the retired form and the remedy.
    let err_msg = format!("{}", result.unwrap_err());
    wat::assert_edn_matches_file!(
        err_msg,
        "wat_arc143_define_alias__define_alias_retired_form_rejected_at_startup.edn",
        "error message should name the retired form and the :wat::core::defalias replacement"
    );
}
