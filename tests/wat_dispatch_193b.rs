//! End-to-end validation of `#[wat_dispatch]` 193b sub-slice.
//!
//! Covers: `self` receivers (`&self`, `&mut self`) under
//! `scope = "thread_owned"`. Self-returns are wrapped in
//! `ThreadOwnedCell<Self>` by the macro-generated code. Thread-boundary
//! crossings panic with a clean `MalformedForm` error.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
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

#[test]
fn counter_increments_and_reads_via_macro_generated_shim() {
    install_fixture_shim();

    let src = r#"
        (:wat::core::use! :rust::test::Counter)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((c :rust::test::Counter) (:rust::test::Counter::new 10))
             ((_ :()) (:rust::test::Counter::increment c))
             ((_ :()) (:rust::test::Counter::increment c))
             ((_ :()) (:rust::test::Counter::increment c)))
            (:rust::test::Counter::read c)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, Arc::new(loader)).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    assert!(matches!(result, Value::i64(13)), "got {:?}", result);
}

#[test]
fn counter_ref_read_preserves_state() {
    install_fixture_shim();

    let src = r#"
        (:wat::core::use! :rust::test::Counter)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((c :rust::test::Counter) (:rust::test::Counter::new 42)))
            (:rust::test::Counter::read c)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, Arc::new(loader)).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    assert!(matches!(result, Value::i64(42)), "got {:?}", result);
}
