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

use wat::freeze::{startup_from_file, StartupError};
use wat::runtime::{apply_function, RuntimeError, RuntimeErrorKind, Value};

// just-eval (rubric): each `*_pN.wat` fixture defines a zero-arg `:user::compute`;
// fetch it from the frozen world and `apply_function` it — no inline wat driver.
// (Path-based rather than `call_beside_value` because this probe shares one `.rs` across
// seven co-located fixtures, so the fixture is not the single sibling `.wat`.)
fn run_compute_from_file(fixture: &str) -> Result<Value, StartupError> {
    let world = startup_from_file(fixture)?;
    let func = world
        .symbols()
        .get(":user::compute")
        .ok_or_else(|| {
            StartupError::Runtime(Box::new(RuntimeError::new(
                wat::rust_caller_span!(),
                RuntimeErrorKind::UnboundSymbol(":user::compute".to_string()),
            )))
        })?
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .map_err(|e| StartupError::Runtime(Box::new(e)))
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
//
// `extract-classifier` on a :wat::core::defrecord instance returns the class name.
// Stone 234.5: for :wat::core::Record, extract-classifier returns String directly
// (not Option<String>) — the class_fqdn is always present at construction.
// Stone 234.6 migration: :wat::core::defrecord instances are Value::wat__core__Record;
// extract-classifier returns :String (not Option<String>) for record args.
#[test]
fn probe_1_extract_classifier_on_defrecord_instance() {
    match run_compute_from_file(
        "tests/reflection/probe_diagnostic_typed_entities_reflection_p1.wat",
    ) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 1 result: {}", s);
            assert_eq!(s, "String(\"myapp::Voltage\")", "Probe 1: extract-classifier on Voltage instance produced unexpected");
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
    match run_compute_from_file(
        "tests/reflection/probe_diagnostic_typed_entities_reflection_p2.wat",
    ) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 2 result: {}", s);
            assert_eq!(s, "Option(None)", "Probe 2: expected None for bare Atom");
        }
        Err(e) => panic!("Probe 2 FAILED: {}", e),
    }
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// `Bind/right` on the holon-form of a :wat::core::defrecord instance returns Some(right Bundle).
// Stone 234.6 migration: :wat::core::defrecord instances are Value::wat__core__Record;
// Bind/right expects HolonAST — coerce via :wat::holon::to-holon first.
#[test]
fn probe_3_bind_right_on_defrecord_instance() {
    match run_compute_from_file(
        "tests/reflection/probe_diagnostic_typed_entities_reflection_p3.wat",
    ) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 3 result: {}", s);
            assert_eq!(s, "Option(Some(holon__HolonAST(Bundle([Bind(Atom(String(\"magnitude\")), Atom(F64(5.0)))]))))", "Probe 3: unexpected Bind/right result");
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
    match run_compute_from_file(
        "tests/reflection/probe_diagnostic_typed_entities_reflection_p4.wat",
    ) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 4 result: {}", s);
            assert_eq!(s, "Option(None)", "Probe 4: expected None for non-Bind");
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
// Stone 234.6 migration: coerce to holon-form via :wat::holon::to-holon
// before applying HolonAST reflection.
#[test]
fn probe_5_composed_walk_to_field_binds() {
    match run_compute_from_file(
        "tests/reflection/probe_diagnostic_typed_entities_reflection_p5.wat",
    ) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 5 result: {}", s);
            assert_eq!(s, "i64(2)", "Probe 5: expected 2 field-Binds in right Bundle");
        }
        Err(e) => panic!("Probe 5 FAILED: {}", e),
    }
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
//
// `Bind/left` on the holon-form of a :wat::core::defrecord instance returns Some(left Atom).
// Stone 234.6 migration: coerce via :wat::holon::to-holon before applying Bind/left.
#[test]
fn probe_6_bind_left_on_defrecord_instance() {
    match run_compute_from_file(
        "tests/reflection/probe_diagnostic_typed_entities_reflection_p6.wat",
    ) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 6 result: {}", s);
            assert_eq!(s, "Option(Some(holon__HolonAST(Atom(String(\"myapp::Voltage\")))))", "Probe 6: unexpected Bind/left result");
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
    match run_compute_from_file(
        "tests/reflection/probe_diagnostic_typed_entities_reflection_p7.wat",
    ) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 7 result: {}", s);
            assert_eq!(s, "Option(None)", "Probe 7: expected None for non-Bind");
        }
        Err(e) => panic!("Probe 7 FAILED: {}", e),
    }
}
