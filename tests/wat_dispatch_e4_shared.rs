//! E4 — scope = "shared" support through `#[wat_dispatch]`.
//!
//! Shared-scope handles have no thread-id guard; `&self` methods call
//! through directly. `&mut self` methods are rejected at macro-expand
//! time. Useful for immutable-after-construction Rust values (query
//! rows, cryptographic keys, configuration snapshots).
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main to eval_in_frozen.
//! Computation moved to :my::compute; canonical nil main appended.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};
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
fn shared_handle_reads_message() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::Greeting)

        (:wat::core::define (:my::compute -> :wat::core::String)
          (:wat::core::let
            [g
              (:rust::test::Greeting::new "hello" 2026)]
            (:rust::test::Greeting::message g)))
    "#;
    match run(src) {
        Value::String(s) => assert_eq!(&*s, "hello"),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn shared_handle_reads_year() {
    install();
    let src = r#"
        (:wat::core::use! :rust::test::Greeting)

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::let
            [g
              (:rust::test::Greeting::new "any" 2026)]
            (:rust::test::Greeting::year g)))
    "#;
    assert!(matches!(run(src), Value::i64(2026)), "got {:?}", run(src));
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

    // Build a Greeting through the macro-generated dispatch path
    // (i.e., by running a tiny wat program via eval_in_frozen on the parent thread).
    let src_make = r#"
        (:wat::core::use! :rust::test::Greeting)

        (:wat::core::define (:my::compute -> :rust::test::Greeting)
          (:rust::test::Greeting::new "crossed" 1999))
    "#;
    let src_make_with_nil = format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src_make
    );
    let world = startup_from_source(&src_make_with_nil, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    let greeting_value = eval_in_frozen(&ast, &world, &env).expect("compute should run");

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
