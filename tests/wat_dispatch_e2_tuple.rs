//! E2 — tuple marshaling through `#[wat_dispatch]`.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};
use wat_macros::wat_dispatch;

pub struct TupleUtils;

#[wat_dispatch(path = ":rust::test::TupleUtils")]
impl TupleUtils {
    /// Sum a pair.
    pub fn sum2(pair: (i64, i64)) -> i64 {
        pair.0 + pair.1
    }

    /// Build a pair from two i64s.
    pub fn pair_of(a: i64, b: i64) -> (i64, i64) {
        (a, b)
    }

    /// Mixed-type triple — bind-params shape (typical SQL).
    pub fn describe(triple: (i64, String, bool)) -> String {
        format!("{}/{}/{}", triple.0, triple.1, triple.2)
    }
}

fn install() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut deps = wat::rust_deps::RustDepsBuilder::with_wat_rs_defaults();
        __wat_dispatch_TupleUtils::register(&mut deps);
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
fn sum2_via_macro() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::TupleUtils)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:rust::test::TupleUtils::sum2 (:wat::core::Tuple 20 22)))
    "#;
    assert!(matches!(run(src), Value::i64(42)), "got {:?}", run(src));
}

#[test]
fn pair_of_returns_tuple() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::TupleUtils)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::first (:rust::test::TupleUtils::pair_of 7 13)))
    "#;
    assert!(matches!(run(src), Value::i64(7)), "got {:?}", run(src));
}

#[test]
fn heterogeneous_triple_via_macro() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::TupleUtils)

        (:wat::core::define (:my::compute -> :wat::core::String)
          (:rust::test::TupleUtils::describe
            (:wat::core::Tuple 1 "row" true)))
    "#;
    match run(src) {
        Value::String(s) => assert_eq!(&*s, "1/row/true"),
        other => panic!("expected string, got {:?}", other),
    }
}
