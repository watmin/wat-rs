//! End-to-end validation of Vec<T> marshaling through `#[wat_dispatch]`.
//! Fixture exposes associated fns that accept and return Vec<i64>.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen (later
//! superseded by call_beside — see docs/CONVENTIONS.md § Test idioms).
//! Computation moved to distinct :my::compute-* defns; driven via call_beside(file!(), fn_name).
//!
//! Wat source lives in the co-located fixture: wat_dispatch_e1_vec.wat

use wat::freeze::call_beside;
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

fn run(fn_name: &str) -> Value {
    call_beside(file!(), fn_name).expect("compute should run")
}

#[test]
fn sum_vec_via_macro() {
    install();
    assert!(matches!(run(":my::compute-sum"), Value::i64(60)), "got {:?}", run(":my::compute-sum"));
}

#[test]
fn reverse_vec_via_macro() {
    install();
    assert!(matches!(run(":my::compute-reverse"), Value::i64(3)), "got {:?}", run(":my::compute-reverse"));
}

#[test]
fn sort_vec_via_macro() {
    install();
    assert!(matches!(run(":my::compute-sort"), Value::i64(1)), "got {:?}", run(":my::compute-sort"));
}

#[test]
fn empty_vec_via_macro() {
    install();
    assert!(matches!(run(":my::compute-empty"), Value::i64(0)), "got {:?}", run(":my::compute-empty"));
}
