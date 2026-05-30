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
//!
//! Arc 170 slice 1f-ζ: tests 1+2 use eval_in_frozen with :my::compute;
//! test 3 unchanged (catch_unwind, startup-error path).

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Arc 170 slice 1f-ζ: append canonical nil-returning `:user::main`.
fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

fn run(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup_from_source(
        &src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    )
    .expect("startup");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run").value_owned()
}

// ─── Test 1: alias :wat::core::foldl — native registration resolves builtin ──

#[test]
fn define_alias_foldl_to_user_fold_delegates_correctly() {
    // Alias :wat::core::foldl as :user::my-fold via native :wat::core::defalias.
    // The builtin is resolved at registration time via CheckEnv::with_builtins().
    // Call (:user::my-fold (Vector :wat::core::i64 1 2 3 4) 0 +fn) → 10.
    // Arc 170 slice 1f-ζ: result returned as i64 via :my::compute.
    let src = r##"

        (:wat::core::defalias :user::my-fold :wat::core::foldl)

        (:wat::core::defn :my::compute [] -> :wat::core::i64
          (:user::my-fold
                      (:wat::core::Vector :wat::core::i64 1 2 3 4)
                      0
                      (:wat::core::fn
                        [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
                        (:wat::core::+ acc x))))
    "##;
    match run(src) {
        Value::i64(n) => assert_eq!(n, 10, "expected alias of foldl to sum [1,2,3,4] from 0 → 10; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Test 2: alias :wat::core::length ────────────────────────────────────────

#[test]
fn define_alias_length_to_user_size_delegates_correctly() {
    // Alias :wat::core::length as :user::my-size via native :wat::core::defalias.
    // Call (:user::my-size (Vector :wat::core::i64 10 20 30)) → 3.
    // Arc 170 slice 1f-ζ: result returned as i64 via :my::compute.
    let src = r##"

        (:wat::core::defalias :user::my-size :wat::core::length)

        (:wat::core::defn :my::compute [] -> :wat::core::i64
          (:user::my-size
                      (:wat::core::Vector :wat::core::i64 10 20 30)))
    "##;
    match run(src) {
        Value::i64(n) => assert_eq!(n, 3, "expected alias of length to return 3 for Vec of 3 elements; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Test 3: retired form :wat::runtime::define-alias is HARD-CUT-rejected ───

#[test]
fn define_alias_retired_form_rejected_at_startup() {
    // Stone 241.12 — :wat::runtime::define-alias is HARD-CUT-rejected.
    // The HARD CUT arm in check.rs fires; startup returns Err with a retirement remedy.
    // Prior behavior: macro panicked at expand-time for unknown targets.
    // New behavior: HARD CUT rejects the retired form regardless of target.
    let src = r##"

        (:wat::runtime::define-alias :user::alias :user::name-that-does-not-exist)

        (:wat::core::defn :user::main [] -> :wat::core::nil nil)
    "##;
    let result = startup_from_source(
        src,
        Some(concat!(file!(), ":", line!())),
        Arc::new(InMemoryLoader::new()),
    );
    assert!(
        result.is_err(),
        "expected startup to fail for retired :wat::runtime::define-alias form; got Ok"
    );
    // Verify the error message names the retired form and the remedy.
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains(":wat::runtime::define-alias") || err_msg.contains(":wat::core::defalias"),
        "error message should reference the retired form or the replacement; got:\n{}",
        err_msg
    );
}
