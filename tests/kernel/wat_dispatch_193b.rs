//! End-to-end validation of `#[wat_dispatch]` 193b sub-slice.
//!
//! Covers: `self` receivers (`&self`, `&mut self`) under
//! `scope = "thread_owned"`. Self-returns are wrapped in
//! `ThreadOwnedCell<Self>` by the macro-generated code. Thread-boundary
//! crossings panic with a clean `MalformedForm` error.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use wat::freeze::call_beside_value;
use wat::runtime::Value;
use wat_macros::wat_dispatch;

/// A stateful counter that starts at 0. `increment()` bumps by 1;
/// `read()` returns the current count. Exercises `&mut self` (for
/// increment) and `&self` (for read).
pub struct Counter {
    count: i64,
}

#[wat_dispatch(path = ":rust::test::Counter", scope = "thread_owned")]
impl Counter {
    pub fn new(initial: i64) -> Self {
        Counter { count: initial }
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }

    pub fn read(&self) -> i64 {
        self.count
    }
}

fn install_fixture_shim() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut deps = wat::rust_deps::RustDepsBuilder::with_wat_rs_defaults();
        __wat_dispatch_Counter::register(&mut deps);
        let _ = wat::rust_deps::install(deps.build());
    });
}

fn run_fn(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).expect("eval should succeed")
}

#[test]
fn counter_increments_and_reads_via_macro_generated_shim() {
    install_fixture_shim();
    let val = run_fn(":my::compute-increment");
    assert!(matches!(val, Value::i64(13)), "got {:?}", val);
}

#[test]
fn counter_ref_read_preserves_state() {
    install_fixture_shim();
    let val = run_fn(":my::compute-read");
    assert!(matches!(val, Value::i64(42)), "got {:?}", val);
}
