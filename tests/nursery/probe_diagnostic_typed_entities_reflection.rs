//! Diagnostic probe — typed-entities reflection layer (arc 232 Stone 232.0a).
//!
//! The typed-entities doctrine (2026-05-23 evening) made
//! `(Bind (Atom <ClassName>) <Inner>)` the canonical shape for every
//! typed value in wat. defrecord (arc 227) implements the shape;
//! defprotocol (arc 232.1+) needs to DISPATCH on the classifier and
//! method bodies need to ACCESS fields. Both require reflection
//! primitives that the substrate doesn't yet expose.
//!
//! Three new wat-callable verbs proposed for Stone 232.0a:
//!
//!   1. `:wat::holon::extract-classifier <h>` -> :Option<String>
//!      Lifts the existing Rust fn `extract_classifier`
//!      (src/runtime.rs:13986). Returns `Some(class-name)` for any
//!      canonical-wrap shape `(Bind (Atom <s>) <right>)`; `None`
//!      otherwise. The DISPATCH primitive defprotocol's polymorphic
//!      verb needs to route to per-type implementations.
//!
//!   2. `:wat::holon::Bind/left <h>` -> :Option<HolonAST>
//!      NEW Rust fn + wat verb. Returns `Some(left)` for literal
//!      `(Bind left _)`; `None` otherwise. The LEFT position of a
//!      Bind primitive. In classifier-wrap shape, holds the
//!      `(Atom <ClassName>)`. In field-Bind shape, holds the
//!      `(Atom <field-name>)`. Symmetric peer of Bind/right.
//!
//!   3. `:wat::holon::Bind/right <h>` -> :Option<HolonAST>
//!      NEW Rust fn + wat verb. Returns `Some(right)` for literal
//!      `(Bind _ right)`; `None` otherwise. The RIGHT position of a
//!      Bind primitive. In classifier-wrap shape, holds the data
//!      (typically a Bundle of field-Binds). In field-Bind shape,
//!      holds the field's value. Mirrors the existing
//!      `Bundle/children` pattern (variant-narrow decomposer naming
//!      the STRUCTURAL fact, not the doctrine-conventional reading).
//!      Composes with `Bundle/children` + name-match to walk a
//!      defrecord instance to a named field.
//!
//! NAMING NOTE (per intueri cast 2026-05-23 night late): the original
//! proposal was `Bind/inner` (asymmetric, borrowed from typed-entity
//! doctrine convention). Intueri verdict: Level 2 (mumbles). Bind is a
//! GENERAL two-position primitive; "inner" only makes sense in the
//! classifier-wrap use case. `Bind/left` + `Bind/right` are positional,
//! symmetric, honest about Bind's structural shape. Convention-based
//! semantic verbs (extract-classifier) compose on top.
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
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    );
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup: {:?}", e))?;
    let ast = wat::parse_one!("(:user::compute)").map_err(|e| format!("parse: {:?}", e))?;
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
//
// `extract-classifier` on a :wat::Record::def instance returns the class name.
// Defines a single-field record, builds an instance, asks for its classifier.
// Stone 234.5: for :wat::Record, extract-classifier returns String directly
// (not Option<String>) — the class_fqdn is always present at construction.
// Stone 234.6 migration: :wat::Record::def instances are Value::wat__Record;
// extract-classifier returns :String (not Option<String>) for record args.
#[test]
fn probe_1_extract_classifier_on_defrecord_instance() {
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::String
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
(:wat::core::defn :user::compute [] -> :wat::core::Option<wat::core::String>
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
// `Bind/right` on the holon-form of a :wat::Record::def instance returns Some(right Bundle).
// The instance's holon-form is `Bind(Atom("myapp::Voltage"), Bundle(field-binds))`;
// `Bind/right` on the HolonAST form should return the Bundle half.
// Stone 234.6 migration: :wat::Record::def instances are Value::wat__Record;
// Bind/right expects HolonAST — coerce via :wat::holon::to-holon first.
#[test]
fn probe_3_bind_right_on_defrecord_instance() {
    let src = r#"
(:wat::holon::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::Option<wat::holon::HolonAST>
  (:wat::core::let
      [v (:myapp::Voltage 5.0)
       h (:wat::holon::to-holon v)]
      (:wat::holon::Bind/right h)))
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
// `Bind/right` on a non-Bind HolonAST returns None.
// A bare Atom is not a Bind; should return None.
#[test]
fn probe_4_bind_right_on_non_bind() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::Option<wat::holon::HolonAST>
  (:wat::core::let
      [bare (:wat::holon::Atom (:wat::holon::to-holon 42))]
      (:wat::holon::Bind/right bare)))
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
// Composed walk: extract-classifier + Bind/right + Bundle/children to get
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
//   2. Bind/right → Bundle(field-binds)
//   3. Bundle/children → Vector of 2 Binds (the field-Binds)
//
// Probe asserts the children Vector has length 2.
#[test]
fn probe_5_composed_walk_to_field_binds() {
    // Stone 234.6 migration: :wat::Record::def instances are Value::wat__Record;
    // coerce to holon-form via :wat::holon::to-holon before applying HolonAST reflection.
    let src = r#"
(:wat::holon::Record::def :myapp::Point
  [x <- :wat::core::i64
   y <- :wat::core::i64])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
      [p          (:myapp::Point 3 4)
       h          (:wat::holon::to-holon p)
       right-opt  (:wat::holon::Bind/right h)
       right      (:wat::core::Option/expect right-opt "right missing")
       children   (:wat::holon::Bundle/children right)]
      (:wat::core::Vector/length children)))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 5 result: {}", s);
            assert!(
                s.contains("2"),
                "Probe 5: expected 2 field-Binds in right Bundle; got: {}",
                s
            );
        }
        Err(e) => panic!("Probe 5 FAILED: {}", e),
    }
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
//
// `Bind/left` on the holon-form of a :wat::Record::def instance returns Some(left Atom).
// The holon-form is `Bind(Atom("myapp::Voltage"), Bundle(field-binds))`;
// `Bind/left` should return the Atom("myapp::Voltage") half.
// Stone 234.6 migration: coerce to holon-form via :wat::holon::to-holon before applying Bind/left.
#[test]
fn probe_6_bind_left_on_defrecord_instance() {
    let src = r#"
(:wat::holon::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::defn :user::compute [] -> :wat::core::Option<wat::holon::HolonAST>
  (:wat::core::let
      [v (:myapp::Voltage 5.0)
       h (:wat::holon::to-holon v)]
      (:wat::holon::Bind/left h)))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 6 result: {}", s);
            // The left of a classifier-wrap is Atom(String("myapp::Voltage"));
            // expect Some(...) wrapping an Atom whose Display surfaces the
            // class name string.
            assert!(
                s.contains("Some") || s.contains("Atom"),
                "Probe 6: expected Some(Atom...) for left of defrecord instance; got: {}",
                s
            );
            assert!(
                s.contains("Voltage") || s.contains("myapp"),
                "Probe 6: expected classifier string ('Voltage' or 'myapp') in left; got: {}",
                s
            );
        }
        Err(e) => panic!("Probe 6 FAILED: {}", e),
    }
}

// ─── Probe 7 ────────────────────────────────────────────────────────────────
//
// `Bind/left` on a non-Bind HolonAST returns None.
// Symmetric peer of probe 4 (Bind/right on non-Bind).
#[test]
fn probe_7_bind_left_on_non_bind() {
    let src = r#"
(:wat::core::defn :user::compute [] -> :wat::core::Option<wat::holon::HolonAST>
  (:wat::core::let
      [bare (:wat::holon::Atom (:wat::holon::to-holon 42))]
      (:wat::holon::Bind/left bare)))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 7 result: {}", s);
            assert!(
                s.contains("None"),
                "Probe 7: expected None for non-Bind; got: {}",
                s
            );
        }
        Err(e) => panic!("Probe 7 FAILED: {}", e),
    }
}
