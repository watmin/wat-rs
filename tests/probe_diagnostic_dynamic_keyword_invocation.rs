//! Diagnostic probe (arc 232 research; not a real stone).
//!
//! Question: can a keyword bound to a value at runtime be used as the
//! HEAD of an invocation? defprotocol's polymorphic dispatcher would
//! construct a mangled FQDN keyword at dispatch time + invoke it.
//!
//! Probe 1: bind a known substrate verb's keyword to a local; call via the binding.
//! Probe 2: build a keyword via keyword/from-string; call via the binding.
//! Probe 3: build a mangled "namespace/method" keyword + invoke a user defn.
//!
//! If any probe FAILS to dispatch, the substrate needs a new "call-by-name"
//! primitive (or the dispatcher accepts only literal-head keywords) and
//! arc 232 has substrate-extension work as a prerequisite.
//!
//! If ALL probes PASS, defprotocol can use the natural pattern
//! `(<built-keyword> args)` and arc 232 is unblocked on the substrate side.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn run_compute(src: &str) -> Result<Value, String> {
    let full = format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    );
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).map_err(|e| format!("eval: {:?}", e))
}

#[test]
fn probe_1_bound_keyword_invokes_substrate_verb() {
    // Bind a known substrate verb keyword to a local; invoke via the local.
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [plus :wat::core::i64::+'2]
    (plus 2 3)))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 1 result: {}", s);
            assert!(
                s.contains("5"),
                "Probe 1: bound-keyword invocation produced unexpected: {}",
                s
            );
        }
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

#[test]
fn probe_2_runtime_built_keyword_invokes_substrate_verb() {
    // Build a keyword string at runtime via keyword/from-string; invoke.
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [plus (:wat::core::keyword/from-string "wat::core::i64::+'2")]
    (plus 2 3)))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 2 result: {}", s);
            assert!(
                s.contains("5"),
                "Probe 2: runtime-built keyword invocation produced unexpected: {}",
                s
            );
        }
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

#[test]
fn probe_3_mangled_namespace_invokes_user_defn() {
    // Mirror defprotocol's dispatch pattern: user defn at a known FQDN;
    // dispatcher builds the FQDN keyword string + invokes.
    let src = r#"
(:wat::core::defn :ns::greeting [name <- :wat::core::String] -> :wat::core::String
  (:wat::core::string::concat "hello " name))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [verb (:wat::core::keyword/from-string "ns::greeting")]
    (verb "world")))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 3 result: {}", s);
            assert!(
                s.contains("hello world"),
                "Probe 3: mangled-namespace invocation produced unexpected: {}",
                s
            );
        }
        Err(e) => panic!("Probe 3 FAILED: {}", e),
    }
}
