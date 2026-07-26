//! E4 — scope = "shared" support through `#[wat_dispatch]`.
//!
//! Shared-scope handles have no thread-id guard; `&self` methods call
//! through directly. `&mut self` methods are rejected at macro-expand
//! time. Useful for immutable-after-construction Rust values (query
//! rows, cryptographic keys, configuration snapshots).
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use wat::freeze::call_beside_value;
use wat::runtime::Value;
use wat_macros::wat_dispatch;

/// Immutable greeting card — construct once, read many times.
pub struct Greeting {
    message: String,
    year: i64,
}

#[wat_dispatch(path = ":rust::test::Greeting", scope = "shared")]
impl Greeting {
    pub fn new(message: String, year: i64) -> Self {
        Greeting { message, year }
    }

    /// &self — reads the shared payload.
    pub fn message(&self) -> String {
        self.message.clone()
    }

    /// &self — reads the year.
    pub fn year(&self) -> i64 {
        self.year
    }
}

fn install() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut deps = wat::rust_deps::RustDepsBuilder::with_wat_rs_defaults();
        __wat_dispatch_Greeting::register(&mut deps);
        let _ = wat::rust_deps::install(deps.build());
    });
}

fn run_fn(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).expect("eval should succeed")
}

#[test]
fn shared_handle_reads_message() {
    install();
    match run_fn(":my::compute-message") {
        Value::String(s) => assert_eq!(&*s, "hello"),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn shared_handle_reads_year() {
    install();
    let val = run_fn(":my::compute-year");
    assert!(matches!(val, Value::i64(2026)), "got {:?}", val);
}

#[test]
fn shared_handle_survives_thread_crossing() {
    // Shared scope has NO thread-id guard. Construct on one thread,
    // call from another — no guard fires, no error.
    //
    // We construct a Greeting via the shim using eval_in_frozen,
    // manually marshal the opaque Value through a Rust channel into
    // a spawned thread, and invoke the shim's dispatch_year there.
    // If scope=shared installs a thread guard (shouldn't), this
    // would fail.
    install();

    // Build a Greeting through the macro-generated dispatch path.
    let greeting_value =
        call_beside_value(file!(), ":my::compute-crossing").expect("compute should run");

    // Ship the Value into a spawned thread. scope=shared → no guard,
    // so downcast + method call should succeed on the child thread.
    let handle = std::thread::spawn(move || {
        match &greeting_value {
            Value::RustOpaque(inner) => {
                let g: &Greeting = wat::rust_deps::downcast_ref_opaque(
                    inner,
                    ":rust::test::Greeting",
                    ":test::year",
                    wat::rust_caller_span!(),
                )
                .expect("downcast");
                g.year()
            }
            other => panic!("expected RustOpaque, got {:?}", other),
        }
    });
    assert_eq!(handle.join().unwrap(), 1999);
}
