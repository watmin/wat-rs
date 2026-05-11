//! End-to-end validation of Vec<T> marshaling through `#[wat_dispatch]`.
//! Fixture exposes associated fns that accept and return Vec<i64>.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};
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

/// Arc 170 slice 1f-ζ: append canonical nil-returning `:user::main`.
fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run")
}

#[test]
fn sum_vec_via_macro() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::VecUtils)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:rust::test::VecUtils::sum (:wat::core::Vector :wat::core::i64 10 20 30)))
    "#;
    assert!(matches!(run(src), Value::i64(60)), "got {:?}", run(src));
}

#[test]
fn reverse_vec_via_macro() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::VecUtils)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::match
            (:wat::core::first
              (:rust::test::VecUtils::reverse (:wat::core::Vector :wat::core::i64 1 2 3)))
            -> :wat::core::i64
            ((:wat::core::Some n) n)
            (:wat::core::None -1)))
    "#;
    assert!(matches!(run(src), Value::i64(3)), "got {:?}", run(src));
}

#[test]
fn sort_vec_via_macro() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::VecUtils)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::match
            (:wat::core::first
              (:rust::test::VecUtils::sort (:wat::core::Vector :wat::core::i64 5 2 8 1)))
            -> :wat::core::i64
            ((:wat::core::Some n) n)
            (:wat::core::None -1)))
    "#;
    assert!(matches!(run(src), Value::i64(1)), "got {:?}", run(src));
}

#[test]
fn empty_vec_via_macro() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::VecUtils)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:rust::test::VecUtils::sum (:wat::core::Vector :wat::core::i64)))
    "#;
    assert!(matches!(run(src), Value::i64(0)), "got {:?}", run(src));
}
