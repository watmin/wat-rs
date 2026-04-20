//! End-to-end validation of `#[wat_dispatch]` 193a sub-slice.
//!
//! Annotates a fixture type with the macro and exercises the generated
//! dispatch + scheme + register fns through wat-rs's full startup
//! pipeline. If this test suite stays green, 193a ships — the macro
//! produces working shim code for associated fns with primitive
//! arg/return types.

use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;
use wat_macros::wat_dispatch;

/// The fixture type. All methods are `fn` with no `self` receiver —
/// 193a supports associated fns only.
pub struct MathUtils;

#[wat_dispatch(path = ":rust::test::MathUtils")]
impl MathUtils {
    /// Add two i64s. Primitive in, primitive out.
    pub fn add(a: i64, b: i64) -> i64 {
        a + b
    }

    /// Return None when n is 0; Some(n*2) otherwise. Primitive in,
    /// Option<primitive> out.
    pub fn maybe_double(n: i64) -> Option<i64> {
        if n == 0 {
            None
        } else {
            Some(n * 2)
        }
    }
}

/// Install the macro-generated shim into the wat-rs registry via the
/// sibling `register()` fn that the macro produces.
fn install_fixture_shim() {
    // The registry is lazily initialized on first access. For this
    // test we need to install BEFORE any wat code runs. A clean
    // consumer pattern would be to build a custom registry and call
    // `wat::rust_deps::install(registry)` — for the test harness we
    // just register into the default-initialized registry.
    //
    // Because `install` is one-shot, we use a `std::sync::Once` so
    // multiple tests in this file (or in parallel test runs) don't
    // double-init. The init order is:
    //   1. Build a RustDepsBuilder pre-loaded with wat-rs defaults.
    //   2. Call the macro-generated `register()` to add fixture shim.
    //   3. Install.
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut deps = wat::rust_deps::RustDepsBuilder::with_wat_rs_defaults();
        __wat_dispatch_MathUtils::register(&mut deps);
        // If install fails, the default registry was already accessed.
        // That's fine for tests — the macro registrations persist via
        // the lazy-init path if we instead mutate the existing registry.
        // Current implementation uses OnceLock; either path works as
        // long as it happens once.
        let _ = wat::rust_deps::install(deps.build());
    });
}

#[test]
fn add_two_i64s_via_macro_generated_shim() {
    install_fixture_shim();

    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::use! :rust::test::MathUtils)

        (:wat::core::define (:user::main -> :i64)
          (:rust::test::MathUtils::add 40 2))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, &loader).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    assert!(matches!(result, Value::i64(42)), "got {:?}", result);
}

#[test]
fn option_some_via_macro_generated_shim() {
    install_fixture_shim();

    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::use! :rust::test::MathUtils)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::match (:rust::test::MathUtils::maybe_double 21)
            ((Some v) v)
            (:None -1)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, &loader).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    assert!(matches!(result, Value::i64(42)), "got {:?}", result);
}

#[test]
fn option_none_via_macro_generated_shim() {
    install_fixture_shim();

    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::use! :rust::test::MathUtils)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::match (:rust::test::MathUtils::maybe_double 0)
            ((Some v) v)
            (:None -1)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, &loader).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    assert!(matches!(result, Value::i64(-1)), "got {:?}", result);
}

#[test]
fn type_check_rejects_wrong_arg_types() {
    install_fixture_shim();

    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::use! :rust::test::MathUtils)

        (:wat::core::define (:user::main -> :i64)
          (:rust::test::MathUtils::add "not-an-int" 2))
    "#;
    let loader = InMemoryLoader::new();
    let result = startup_from_source(src, None, &loader);
    assert!(result.is_err(), "expected type error; got {:?}", result.ok());
}

