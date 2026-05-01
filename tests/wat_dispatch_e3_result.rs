//! E3 — :Result<T,E> marshaling + (Ok v)/(Err e) construction + match.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;
use wat_macros::wat_dispatch;

pub struct Fallible;

#[wat_dispatch(path = ":rust::test::Fallible")]
impl Fallible {
    /// Returns Ok(n) when n >= 0, Err("negative") otherwise.
    pub fn non_negative(n: i64) -> std::result::Result<i64, String> {
        if n >= 0 {
            Ok(n)
        } else {
            Err("negative".into())
        }
    }

    /// Always returns Err — useful for exhaustiveness testing.
    pub fn always_err() -> std::result::Result<i64, String> {
        Err("computed failure".into())
    }
}

fn install() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut deps = wat::rust_deps::RustDepsBuilder::with_wat_rs_defaults();
        __wat_dispatch_Fallible::register(&mut deps);
        let _ = wat::rust_deps::install(deps.build());
    });
}

#[test]
fn result_ok_matched() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::Fallible)

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::match (:rust::test::Fallible::non_negative 42) -> :wat::core::i64
            ((Ok v) v)
            ((Err _) -1)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, Arc::new(loader)).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    assert!(matches!(result, Value::i64(42)), "got {:?}", result);
}

#[test]
fn result_err_matched() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::Fallible)

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::match (:rust::test::Fallible::non_negative -1) -> :wat::core::i64
            ((Ok _) 0)
            ((Err _) 99)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, Arc::new(loader)).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    assert!(matches!(result, Value::i64(99)), "got {:?}", result);
}

#[test]
fn user_built_ok_value() {
    // (Ok expr) should work at the wat source level too, independent
    // of any Rust shim.
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::match (Ok 7) -> :wat::core::i64
            ((Ok v) v)
            ((Err _) -1)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, Arc::new(loader)).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    assert!(matches!(result, Value::i64(7)), "got {:?}", result);
}

#[test]
fn user_built_err_value() {
    let src = r#"

        (:wat::core::define (:user::main -> :wat::core::i64)
          (:wat::core::match (Err "x") -> :wat::core::i64
            ((Ok _) 0)
            ((Err _) 11)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, Arc::new(loader)).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    assert!(matches!(result, Value::i64(11)), "got {:?}", result);
}
