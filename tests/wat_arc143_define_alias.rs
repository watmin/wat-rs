//! Integration coverage for arc 143 slice 6 — the
//! `:wat::runtime::define-alias` defmacro.
//!
//! The macro lives in `wat/runtime.wat` (pure wat) and composes
//! the substrate primitives shipped in slices 1+2+3:
//!   - `:wat::runtime::signature-of-defn`     (slice 1)
//!   - `:wat::runtime::rename-callable-name`  (slice 3)
//!   - `:wat::runtime::extract-arg-names`     (slice 3)
//!   - computed unquote `,(expr)`             (slice 2)
//!
//! Sequencing constraint discovered in slice 6:
//!   Computed unquote runs during macro expansion (step 4 in
//!   startup_from_forms_post_config) with &SymbolTable::default()
//!   (empty). Only substrate primitives visible via
//!   CheckEnv::with_builtins() are reachable at expand-time.
//!   User-defined functions in the same source file are NOT visible
//!   until step 6 (register_defines). Therefore define-alias can
//!   only alias substrate primitives at expand-time.
//!
//! Tests:
//!   1. Alias a substrate primitive (:wat::core::foldl) — expand-time
//!      signature-of-defn succeeds; alias delegates to the primitive correctly.
//!   2. Alias another substrate primitive (:wat::core::length) — verifies
//!      the macro works for multiple targets.
//!   3. Error case — alias to a name that doesn't exist (not a substrate
//!      primitive, not a user define) — Option/expect panics at expand-time.
//!      Verifies the macro expands eagerly and the error message propagates.
//!
//! Arc 170 slice 1f-ζ: tests 1+2 use eval_in_frozen with :my::compute;
//! test 3 unchanged (catch_unwind, panic-at-startup path).

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Arc 170 slice 1f-ζ: append canonical nil-returning `:user::main`.
fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
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
    eval_in_frozen(&ast, &world, &env).expect("compute should run")
}

// ─── Test 1: alias :wat::core::foldl — expand-time substrate lookup works ────

#[test]
fn define_alias_foldl_to_user_fold_delegates_correctly() {
    // Alias :wat::core::foldl as :user::my-fold.
    // At expand-time, signature-of-defn :wat::core::foldl resolves via
    // CheckEnv::with_builtins() — the substrate primitive IS visible.
    // Call (:user::my-fold (Vector :wat::core::i64 1 2 3 4) 0 +fn) → 10.
    // Arc 170 slice 1f-ζ: result returned as i64 via :my::compute.
    let src = r##"

        (:wat::runtime::define-alias :user::my-fold :wat::core::foldl)

        (:wat::core::define (:my::compute -> :wat::core::i64)
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
    // Alias :wat::core::length as :user::my-size.
    // At expand-time, signature-of-defn :wat::core::length resolves via substrate.
    // Call (:user::my-size (Vector :wat::core::i64 10 20 30)) → 3.
    // Arc 170 slice 1f-ζ: result returned as i64 via :my::compute.
    let src = r##"

        (:wat::runtime::define-alias :user::my-size :wat::core::length)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:user::my-size
            (:wat::core::Vector :wat::core::i64 10 20 30)))
    "##;
    match run(src) {
        Value::i64(n) => assert_eq!(n, 3, "expected alias of length to return 3 for Vec of 3 elements; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Test 3: unknown target panics at expand-time ────────────────────────────

#[test]
fn define_alias_unknown_target_panics_at_expand_time() {
    // :user::name-that-does-not-exist is not a substrate primitive or user define.
    // The macro calls (Option/expect (signature-of-defn :user::name-that-does-not-exist) ...)
    // at expand-time; expect panics via std::panic::panic_any —
    // this propagates as a Rust panic out of startup_from_source.
    //
    // NOTE: expect_panic uses std::panic::panic_any (not a Result::Err),
    // so startup_from_source propagates the panic rather than returning
    // Err(StartupError). We use catch_unwind to detect it.
    let src = r##"

        (:wat::runtime::define-alias :user::alias :user::name-that-does-not-exist)

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "##;
    let result = std::panic::catch_unwind(|| {
        startup_from_source(
            src,
            Some(concat!(file!(), ":", line!())),
            Arc::new(InMemoryLoader::new()),
        )
    });
    assert!(
        result.is_err(),
        "expected startup to panic for unknown target name, but it returned Ok"
    );
    // The panic payload is an AssertionPayload; its message field carries
    // "define-alias: target name not found in environment".
    // We don't inspect the payload here — the panic-at-startup is the
    // observable signal. The message is in the macro body verbatim.
}
