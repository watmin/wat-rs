//! End-to-end validation of `#[wat_dispatch]` 193a sub-slice.
//!
//! Annotates a fixture type with the macro and exercises the generated
//! dispatch + scheme + register fns through wat-rs's full startup
//! pipeline. If this test suite stays green, 193a ships — the macro
//! produces working shim code for associated fns with primitive
//! arg/return types.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};
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
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut deps = wat::rust_deps::RustDepsBuilder::with_wat_rs_defaults();
        __wat_dispatch_MathUtils::register(&mut deps);
        let _ = wat::rust_deps::install(deps.build());
    });
}

/// Arc 170 slice 1f-ζ: append canonical nil-returning `:user::main`.
fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run")
}

#[test]
fn add_two_i64s_via_macro_generated_shim() {
    install_fixture_shim();

    let src = r#"
        (:wat::core::use! :rust::test::MathUtils)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:rust::test::MathUtils::add 40 2))
    "#;
    assert!(matches!(run(src), Value::i64(42)), "got {:?}", run(src));
}

#[test]
fn option_some_via_macro_generated_shim() {
    install_fixture_shim();

    let src = r#"
        (:wat::core::use! :rust::test::MathUtils)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::match (:rust::test::MathUtils::maybe_double 21) -> :wat::core::i64
            ((:wat::core::Some v) v)
            (:wat::core::None -1)))
    "#;
    assert!(matches!(run(src), Value::i64(42)), "got {:?}", run(src));
}

#[test]
fn option_none_via_macro_generated_shim() {
    install_fixture_shim();

    let src = r#"
        (:wat::core::use! :rust::test::MathUtils)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::match (:rust::test::MathUtils::maybe_double 0) -> :wat::core::i64
            ((:wat::core::Some v) v)
            (:wat::core::None -1)))
    "#;
    assert!(matches!(run(src), Value::i64(-1)), "got {:?}", run(src));
}

#[test]
fn type_check_rejects_wrong_arg_types() {
    install_fixture_shim();

    let src = r#"
        (:wat::core::use! :rust::test::MathUtils)

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)

        (:wat::core::define (:my::probe -> :wat::core::i64)
          (:rust::test::MathUtils::add "not-an-int" 2))
    "#;
    let loader = InMemoryLoader::new();
    let result = startup_from_source(src, None, Arc::new(loader));
    assert!(result.is_err(), "expected type error; got {:?}", result.ok());
}
