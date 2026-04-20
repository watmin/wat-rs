//! E2 — tuple marshaling through `#[wat_dispatch]`.

use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;
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

#[test]
fn sum2_via_macro() {
    install();
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::use! :rust::test::TupleUtils)

        (:wat::core::define (:user::main -> :i64)
          (:rust::test::TupleUtils::sum2 (:wat::core::tuple 20 22)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, &loader).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    assert!(matches!(result, Value::i64(42)), "got {:?}", result);
}

#[test]
fn pair_of_returns_tuple() {
    install();
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::use! :rust::test::TupleUtils)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::first (:rust::test::TupleUtils::pair_of 7 13)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, &loader).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    assert!(matches!(result, Value::i64(7)), "got {:?}", result);
}

#[test]
fn heterogeneous_triple_via_macro() {
    install();
    let src = r#"
        (:wat::config::set-dims! 1024)
        (:wat::config::set-capacity-mode! :error)
        (:wat::core::use! :rust::test::TupleUtils)

        (:wat::core::define (:user::main -> :String)
          (:rust::test::TupleUtils::describe
            (:wat::core::tuple 1 "row" true)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, &loader).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    match result {
        Value::String(s) => assert_eq!(&*s, "1/row/true"),
        other => panic!("expected string, got {:?}", other),
    }
}
