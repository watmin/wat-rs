//! E3 — :Result<T,E> marshaling + (Ok v)/(Err e) construction + match.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use wat::freeze::{eval_in_frozen, startup_beside};
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

fn run_fn(fn_name: &str) -> Value {
    install();
    let world = startup_beside(file!()).expect("startup");
    let call = format!("({fn_name})");
    let ast = wat::parse_one!(&call).expect("parse compute call");
    eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
}

#[test]
fn result_ok_matched() {
    install();
    let val = run_fn(":my::compute-ok-matched");
    assert!(matches!(val, Value::i64(42)), "got {:?}", val);
}

#[test]
fn result_err_matched() {
    install();
    let val = run_fn(":my::compute-err-matched");
    assert!(matches!(val, Value::i64(99)), "got {:?}", val);
}

#[test]
fn user_built_ok_value() {
    // (Ok expr) should work at the wat source level too, independent
    // of any Rust shim.
    let val = run_fn(":my::compute-user-ok");
    assert!(matches!(val, Value::i64(7)), "got {:?}", val);
}

#[test]
fn user_built_err_value() {
    let val = run_fn(":my::compute-user-err");
    assert!(matches!(val, Value::i64(11)), "got {:?}", val);
}
