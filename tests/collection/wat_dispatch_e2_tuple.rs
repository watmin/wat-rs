//! E2 — tuple marshaling through `#[wat_dispatch]`.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen (later
//! superseded by call_beside_value — see docs/CONVENTIONS.md § Test idioms).
//! Computation moved to distinct :my::compute-* defns; driven via call_beside_value(file!(), fn_name).
//!
//! Wat source lives in the co-located fixture: wat_dispatch_e2_tuple.wat

use wat::freeze::call_beside_value;
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

fn run(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).expect("compute should run")
}

#[test]
fn sum2_via_macro() {
    install();
    assert!(matches!(run(":my::compute-sum2"), Value::i64(42)), "got {:?}", run(":my::compute-sum2"));
}

#[test]
fn pair_of_returns_tuple() {
    install();
    assert!(matches!(run(":my::compute-pair-first"), Value::i64(7)), "got {:?}", run(":my::compute-pair-first"));
}

#[test]
fn heterogeneous_triple_via_macro() {
    install();
    match run(":my::compute-describe") {
        Value::String(s) => assert_eq!(&*s, "1/row/true"),
        other => panic!("expected string, got {:?}", other),
    }
}
