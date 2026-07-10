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

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

fn run_expr(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse expr");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
}

// ─── Test 1: alias :wat::core::foldl — native registration resolves builtin ──

#[test]
fn define_alias_foldl_to_user_fold_delegates_correctly() {
    match run_expr("(:t::test1-foldl-alias)") {
        Value::i64(n) => assert_eq!(n, 10, "expected alias of foldl to sum [1,2,3,4] from 0 → 10; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Test 2: alias :wat::core::length ────────────────────────────────────────

#[test]
fn define_alias_length_to_user_size_delegates_correctly() {
    match run_expr("(:t::test2-length-alias)") {
        Value::i64(n) => assert_eq!(n, 3, "expected alias of length to return 3 for Vec of 3 elements; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Test 3: retired form :wat::runtime::define-alias is HARD-CUT-rejected ───

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn define_alias_retired_form_rejected_at_startup() {
    let result = startup_from_file("tests/wat_lang/wat_arc143_define_alias_retired.wat.bad");
    assert!(
        result.is_err(),
        "expected startup to fail for retired :wat::runtime::define-alias form; got Ok"
    );
    // Verify the error message names the retired form and the remedy.
    let err_msg = format!("{}", result.unwrap_err());
    assert_eq!(
        err_msg,
        "check:\n1 type-check error(s):\n  - tests/wat_lang/wat_arc143_define_alias_retired.wat.bad:4:2: malformed :wat::runtime::define-alias form: ':wat::runtime::define-alias' is retired (Stone 241.12); use ':wat::core::defalias' instead\n  did you mean: :wat::core::defalias [replaces a retired form]\n",
        "error message should name the retired form and the :wat::core::defalias replacement"
    );
}
