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

use wat::check::error::CheckErrorKind;
use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::Value;
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

fn run_fn(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).expect("eval should succeed")
}

#[test]
fn add_two_i64s_via_macro_generated_shim() {
    install_fixture_shim();
    let val = run_fn(":my::compute-add");
    assert!(matches!(val, Value::i64(42)), "got {:?}", val);
}

#[test]
fn option_some_via_macro_generated_shim() {
    install_fixture_shim();
    let val = run_fn(":my::compute-some");
    assert!(matches!(val, Value::i64(42)), "got {:?}", val);
}

#[test]
fn option_none_via_macro_generated_shim() {
    install_fixture_shim();
    let val = run_fn(":my::compute-none");
    assert!(matches!(val, Value::i64(-1)), "got {:?}", val);
}

#[test]
fn type_check_rejects_wrong_arg_types() {
    install_fixture_shim();
    let result = startup_from_file("tests/kernel/wat_dispatch_193a.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":rust::test::MathUtils::add"
            && param == "#1"
            && expected == "Path(\":wat::core::i64\")"
            && got == "Path(\":wat::core::String\")"
    );
}
