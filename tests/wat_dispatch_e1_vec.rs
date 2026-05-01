//! End-to-end validation of Vec<T> marshaling through `#[wat_dispatch]`.
//! Fixture exposes associated fns that accept and return Vec<i64>.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;
use wat_macros::wat_dispatch;

pub struct VecUtils;

#[wat_dispatch(path = ":rust::test::VecUtils")]
impl VecUtils {
    /// Sum a vec of i64s.
    pub fn sum(xs: Vec<i64>) -> i64 {
        xs.iter().sum()
    }

    /// Reverse a vec of i64s.
    pub fn reverse(xs: Vec<i64>) -> Vec<i64> {
        xs.into_iter().rev().collect()
    }

    /// Build a sorted copy of a vec.
    pub fn sort(mut xs: Vec<i64>) -> Vec<i64> {
        xs.sort();
        xs
    }
}

fn install() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut deps = wat::rust_deps::RustDepsBuilder::with_wat_rs_defaults();
        __wat_dispatch_VecUtils::register(&mut deps);
        let _ = wat::rust_deps::install(deps.build());
    });
}

#[test]
fn sum_vec_via_macro() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::VecUtils)

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:rust::test::VecUtils::sum (:wat::core::Vector :wat::core::i64 10 20 30)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, Arc::new(loader)).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    assert!(matches!(result, Value::i64(60)), "got {:?}", result);
}

#[test]
fn reverse_vec_via_macro() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::VecUtils)

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::match
            (:wat::core::first
              (:rust::test::VecUtils::reverse (:wat::core::Vector :wat::core::i64 1 2 3)))
            -> :wat::core::i64
            ((Some n) n)
            (:None -1)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, Arc::new(loader)).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    assert!(matches!(result, Value::i64(3)), "got {:?}", result);
}

#[test]
fn sort_vec_via_macro() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::VecUtils)

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::match
            (:wat::core::first
              (:rust::test::VecUtils::sort (:wat::core::Vector :wat::core::i64 5 2 8 1)))
            -> :wat::core::i64
            ((Some n) n)
            (:None -1)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, Arc::new(loader)).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    assert!(matches!(result, Value::i64(1)), "got {:?}", result);
}

#[test]
fn empty_vec_via_macro() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::VecUtils)

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:rust::test::VecUtils::sum (:wat::core::Vector :wat::core::i64)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, Arc::new(loader)).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    assert!(matches!(result, Value::i64(0)), "got {:?}", result);
}
