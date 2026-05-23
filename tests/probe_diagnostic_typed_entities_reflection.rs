//! Diagnostic probe — typed-entities reflection layer (arc 232 Stone 232.0a).
//!
//! The typed-entities doctrine (2026-05-23 evening) made
//! `(Bind (Atom <ClassName>) <Inner>)` the canonical shape for every
//! typed value in wat. defrecord (arc 227) implements the shape;
//! defprotocol (arc 232.1+) needs to DISPATCH on the classifier and
//! method bodies need to ACCESS fields. Both require reflection
//! primitives that the substrate doesn't yet expose.
//!
//! Two new wat-callable verbs proposed for Stone 232.0a:
//!
//!   1. `:wat::holon::extract-classifier <h>` -> :Option<String>
//!      Lifts the existing Rust fn `extract_classifier`
//!      (src/runtime.rs:13986). Returns `Some(class-name)` for any
//!      canonical-wrap shape `(Bind (Atom <s>) <inner>)`; `None`
//!      otherwise. The DISPATCH primitive defprotocol's polymorphic
//!      verb needs to route to per-type implementations.
//!
//!   2. `:wat::holon::Bind/inner <h>` -> :Option<HolonAST>
//!      NEW Rust fn + wat verb. Returns `Some(inner)` for literal
//!      `(Bind _ inner)`; `None` otherwise. Mirrors the existing
//!      `Bundle/children` pattern (variant-narrow decomposer).
//!      Composes with `Bundle/children` + name-match to walk a
//!      defrecord instance to a named field.
//!
//! These probes currently FAIL (verbs don't exist). After Stone 232.0a
//! ships, they PASS. They become the regression guard against the
//! reflection-layer gap reopening.
//!
//! Outcomes:
//!   - ALL PASS: Stone 232.0a complete; defprotocol unblocked.
//!   - ANY FAIL: SPECIFIC failure surfaced (verb missing? wrong
//!     signature? Option-wrapping wrong? composition broken?).
//!     Stone's not done.

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

// ─── Probe 1 ────────────────────────────────────────────────────────────────
//
// `extract-classifier` on a defrecord instance returns Some(class-name).
// Defines a single-field defrecord, builds an instance, asks for its
// classifier. Should return :wat::core::Option::Some "myapp::Voltage".
#[test]
fn probe_1_extract_classifier_on_defrecord_instance() {
    let src = r#"
(:wat::holon::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::define (:user::compute -> :wat::core::Option<wat::core::String>)
  (:wat::core::let
    [v (:myapp::Voltage 5.0)]
    (:wat::holon::extract-classifier v)))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 1 result: {}", s);
            assert!(
                s.contains("myapp::Voltage") || s.contains("Voltage"),
                "Probe 1: extract-classifier on Voltage instance produced unexpected: {}",
                s
            );
            assert!(
                s.contains("Some") || s.contains("Option"),
                "Probe 1: expected Some(...)-wrapped result: {}",
                s
            );
        }
        Err(e) => panic!("Probe 1 FAILED: {}", e),
    }
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// `extract-classifier` on a non-canonical-wrap HolonAST returns None.
// A bare Atom (not a Bind) has no classifier; should return None.
#[test]
fn probe_2_extract_classifier_on_bare_atom() {
    let src = r#"
(:wat::core::define (:user::compute -> :wat::core::Option<wat::core::String>)
  (:wat::core::let
    [bare (:wat::holon::Atom (:wat::holon::to-holon 42))]
    (:wat::holon::extract-classifier bare)))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 2 result: {}", s);
            assert!(
                s.contains("None"),
                "Probe 2: expected None for bare Atom; got: {}",
                s
            );
        }
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// `Bind/inner` on a defrecord instance returns Some(inner Bundle).
// The instance is `Bind(Atom("myapp::Voltage"), Bundle(field-binds))`;
// `Bind/inner` should return the Bundle half.
#[test]
fn probe_3_bind_inner_on_defrecord_instance() {
    let src = r#"
(:wat::holon::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::define (:user::compute -> :wat::core::Option<wat::holon::HolonAST>)
  (:wat::core::let
    [v (:myapp::Voltage 5.0)]
    (:wat::holon::Bind/inner v)))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 3 result: {}", s);
            assert!(
                s.contains("Some") || s.contains("Bundle"),
                "Probe 3: expected Some(Bundle...); got: {}",
                s
            );
        }
        Err(e) => panic!("Probe 3 FAILED: {}", e),
    }
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
//
// `Bind/inner` on a non-Bind HolonAST returns None.
// A bare Atom is not a Bind; should return None.
#[test]
fn probe_4_bind_inner_on_non_bind() {
    let src = r#"
(:wat::core::define (:user::compute -> :wat::core::Option<wat::holon::HolonAST>)
  (:wat::core::let
    [bare (:wat::holon::Atom (:wat::holon::to-holon 42))]
    (:wat::holon::Bind/inner bare)))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 4 result: {}", s);
            assert!(
                s.contains("None"),
                "Probe 4: expected None for non-Bind; got: {}",
                s
            );
        }
        Err(e) => panic!("Probe 4 FAILED: {}", e),
    }
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
//
// Composed walk: extract-classifier + Bind/inner + Bundle/children to get
// the field-Bind list from a defrecord instance. This is the exact
// composition defrecord accessor synthesis (separate stone) will use to
// generate `:ns::Type/field-name` accessors.
//
// Verifies the three primitives compose correctly without intermediate
// substrate gaps. The defrecord instance:
//   Bind(Atom("myapp::Point"), Bundle(Bind(Atom("x"), Atom(3)), Bind(Atom("y"), Atom(4))))
//
// Walk:
//   1. extract-classifier → "myapp::Point"   (dispatch routing)
//   2. Bind/inner → Bundle(field-binds)
//   3. Bundle/children → Vector of 2 Binds (the field-Binds)
//
// Probe asserts the children Vector has length 2.
#[test]
fn probe_5_composed_walk_to_field_binds() {
    let src = r#"
(:wat::holon::defrecord :myapp::Point
  [x <- :wat::core::i64
   y <- :wat::core::i64])

(:wat::core::define (:user::compute -> :wat::core::i64)
  (:wat::core::let
    [p          (:myapp::Point 3 4)
     inner-opt  (:wat::holon::Bind/inner p)
     inner      (:wat::core::Option/expect -> :wat::holon::HolonAST inner-opt "inner missing")
     children   (:wat::holon::Bundle/children inner)]
    (:wat::core::Vector/length children)))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 5 result: {}", s);
            assert!(
                s.contains("2"),
                "Probe 5: expected 2 field-Binds in inner Bundle; got: {}",
                s
            );
        }
        Err(e) => panic!("Probe 5 FAILED: {}", e),
    }
}
