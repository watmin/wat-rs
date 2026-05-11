//! E3 — :Result<T,E> marshaling + (Ok v)/(Err e) construction + match.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};
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
fn result_ok_matched() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::Fallible)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::match (:rust::test::Fallible::non_negative 42) -> :wat::core::i64
            ((:wat::core::Ok v) v)
            ((:wat::core::Err _) -1)))
    "#;
    assert!(matches!(run(src), Value::i64(42)), "got {:?}", run(src));
}

#[test]
fn result_err_matched() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::Fallible)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::match (:rust::test::Fallible::non_negative -1) -> :wat::core::i64
            ((:wat::core::Ok _) 0)
            ((:wat::core::Err _) 99)))
    "#;
    assert!(matches!(run(src), Value::i64(99)), "got {:?}", run(src));
}

#[test]
fn user_built_ok_value() {
    // (Ok expr) should work at the wat source level too, independent
    // of any Rust shim.
    let src = r#"

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::match (:wat::core::Ok 7) -> :wat::core::i64
            ((:wat::core::Ok v) v)
            ((:wat::core::Err _) -1)))
    "#;
    assert!(matches!(run(src), Value::i64(7)), "got {:?}", run(src));
}

#[test]
fn user_built_err_value() {
    let src = r#"

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::match (:wat::core::Err "x") -> :wat::core::i64
            ((:wat::core::Ok _) 0)
            ((:wat::core::Err _) 11)))
    "#;
    assert!(matches!(run(src), Value::i64(11)), "got {:?}", run(src));
}
