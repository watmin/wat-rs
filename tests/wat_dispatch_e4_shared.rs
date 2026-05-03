//! E4 — scope = "shared" support through `#[wat_dispatch]`.
//!
//! Shared-scope handles have no thread-id guard; `&self` methods call
//! through directly. `&mut self` methods are rejected at macro-expand
//! time. Useful for immutable-after-construction Rust values (query
//! rows, cryptographic keys, configuration snapshots).

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
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

#[test]
fn shared_handle_reads_message() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::Greeting)

        (:wat::core::define (:user::main -> :String)
          (:wat::core::let*
            (((g :rust::test::Greeting)
              (:rust::test::Greeting::new "hello" 2026)))
            (:rust::test::Greeting::message g)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, Arc::new(loader)).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    match result {
        Value::String(s) => assert_eq!(&*s, "hello"),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn shared_handle_reads_year() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::Greeting)

        (:wat::core::define (:user::main -> :i64)
          (:wat::core::let*
            (((g :rust::test::Greeting)
              (:rust::test::Greeting::new "any" 2026)))
            (:rust::test::Greeting::year g)))
    "#;
    let loader = InMemoryLoader::new();
    let world = startup_from_source(src, None, Arc::new(loader)).expect("startup");
    let result = invoke_user_main(&world, Vec::new()).expect("main");
    assert!(matches!(result, Value::i64(2026)), "got {:?}", result);
}

#[test]
fn shared_handle_survives_thread_crossing() {
    // Shared scope has NO thread-id guard. Construct on one thread,
    // call from another — no guard fires, no error.
    //
    // We construct a Greeting via the shim in the parent thread,
    // manually marshal the opaque Value through a Rust channel into
    // a spawned thread, and invoke the shim's dispatch_year there.
    // If scope=shared installs a thread guard (shouldn't), this
    // would fail.
    install();

    // Build a Greeting through the macro-generated dispatch path
    // (i.e., by running a tiny wat program on the parent thread).
    let src_make = r#"
        (:wat::core::use! :rust::test::Greeting)

        (:wat::core::define (:user::main -> :rust::test::Greeting)
          (:rust::test::Greeting::new "crossed" 1999))
    "#;
    let loader = InMemoryLoader::new();
    let world = wat::freeze::startup_from_source(src_make, None, Arc::new(loader)).expect("startup");
    let greeting_value =
        wat::freeze::invoke_user_main(&world, Vec::new()).expect("main");

    // Ship the Value into a spawned thread. scope=shared → no guard,
    // so downcast + method call should succeed on the child thread.
    let handle = std::thread::spawn(move || {
        match &greeting_value {
            Value::RustOpaque(inner) => {
                let g: &Greeting = wat::rust_deps::downcast_ref_opaque(
                    inner,
                    ":rust::test::Greeting",
                    ":test::year",
                    wat::span::Span::unknown(),
                )
                .expect("downcast");
                g.year()
            }
            other => panic!("expected RustOpaque, got {:?}", other),
        }
    });
    assert_eq!(handle.join().unwrap(), 1999);
}
