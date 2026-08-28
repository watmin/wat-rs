//! Diagnostic probe — `:wat::core::type` polymorphic primitive (arc 234 Stone 234.0).
//!
//! FM 2-bis empirical probe authored BEFORE the Stone 234.0 BRIEF. Proves
//! the polymorphic type-extraction primitive composes correctly across
//! Value variants. If these probes PASS post-stone, the substrate primitive
//! is sufficient for defprotocol's polymorphic dispatcher (revised Stone
//! 232.1) and the rest of arc 234.x.
//!
//! Semantics: `(:wat::core::type <any-value>) -> :wat::core::String` returns
//! the record-type FQDN as a String, regardless of underlying storage backend.
//!
//! Dispatch table (per DESIGN-STONE-234.0 D2):
//!   - HolonAST classifier-wrap -> extract_classifier(holon)
//!     .unwrap_or_else(|| "wat::holon::HolonAST".to_string())
//!   - Value::Struct(sv) -> sv.type_name.trim_start_matches(':').to_string()
//!   - Any other Value -> Value::type_name().to_string()
//!
//! Probe contracts (8):
//!   1. primitive — i64 returns "wat::core::i64"
//!   2. primitive — String returns "wat::core::String"
//!   3. primitive — bool returns "wat::core::bool"
//!   4. primitive — keyword returns "wat::core::keyword"
//!   5. Vector literal returns "wat::core::Vector"
//!   6. HashMap literal returns "wat::core::HashMap"
//!   7. defrecord instance (HolonAST classifier-wrap) returns the FQDN
//!      class name (e.g., "myapp::Voltage"), NOT "wat::holon::HolonAST"
//!   8. struct instance returns the FQDN type name WITHOUT leading colon
//!      (e.g., "myapp::Point"), NOT ":myapp::Point" or "wat::core::Struct"
//!
//! Initial state: all FAIL (verb doesn't exist; UnknownFunction).
//! Post-stone: all 8 PASS.
//!
//! Outcomes:
//!   - ALL PASS: Stone 234.0 ships clean; substrate ready for revised
//!     Stone 232.1 + arc 234.x downstream stones
//!   - ANY FAIL: SPECIFIC trap surfaced. The dispatch arm or TypeScheme or
//!     extract_classifier integration has an issue; resolve before BRIEF
//!     ships to sonnet OR encode the resolution in the BRIEF

use wat::freeze::{startup_from_file, StartupError};
use wat::runtime::{apply_function, RuntimeError, RuntimeErrorKind, Value};

// just-eval (rubric): each `*_pN.wat` fixture defines a zero-arg `:user::compute`;
// fetch it from the frozen world and `apply_function` it — no inline wat driver.
// (Path-based rather than `call_beside_value` because this probe shares one `.rs` across
// eight co-located fixtures, so the fixture is not the single sibling `.wat`.)
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

fn assert_type_contains(probe_label: &str, fixture: &str, expected: &str) {
    match run_compute_from_file(fixture) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("{} result: {}", probe_label, s);
            assert!(
                s.contains(expected),
                "{}: expected return value to contain {:?}; got: {}",
                probe_label,
                expected,
                s
            );
        }
        Err(e) => panic!("{} FAILED: {}", probe_label, e),
    }
}

// ─── Probe 1 ────────────────────────────────────────────────────────────────
//
// `(:wat::core::type 5)` on a literal i64 returns "wat::core::i64".
#[test]
fn probe_1_type_on_i64() {
    assert_type_contains(
        "Probe 1 (i64)",
        "tests/reflection/probe_diagnostic_polymorphic_type_p1.wat",
        "wat::core::i64",
    );
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// `(:wat::core::type "hello")` on a literal String returns "wat::core::String".
#[test]
fn probe_2_type_on_string() {
    assert_type_contains(
        "Probe 2 (String)",
        "tests/reflection/probe_diagnostic_polymorphic_type_p2.wat",
        "wat::core::String",
    );
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// `(:wat::core::type true)` on a literal bool returns "wat::core::bool".
#[test]
fn probe_3_type_on_bool() {
    assert_type_contains(
        "Probe 3 (bool)",
        "tests/reflection/probe_diagnostic_polymorphic_type_p3.wat",
        "wat::core::bool",
    );
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
//
// `(:wat::core::type :foo)` on a literal keyword returns "wat::core::keyword".
#[test]
fn probe_4_type_on_keyword() {
    assert_type_contains(
        "Probe 4 (keyword)",
        "tests/reflection/probe_diagnostic_polymorphic_type_p4.wat",
        "wat::core::keyword",
    );
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
//
// `(:wat::core::type [1 2 3])` on a Vector literal returns "wat::core::Vector".
#[test]
fn probe_5_type_on_vector() {
    assert_type_contains(
        "Probe 5 (Vector)",
        "tests/reflection/probe_diagnostic_polymorphic_type_p5.wat",
        "wat::core::Vector",
    );
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
//
// `(:wat::core::type {:a 1})` on a HashMap literal returns "wat::core::HashMap".
#[test]
fn probe_6_type_on_hashmap() {
    assert_type_contains(
        "Probe 6 (HashMap)",
        "tests/reflection/probe_diagnostic_polymorphic_type_p6.wat",
        "wat::core::HashMap",
    );
}

// ─── Probe 7 ────────────────────────────────────────────────────────────────
//
// `(:wat::core::type (:myapp::Voltage 5.0))` on a defrecord instance
// (HolonAST classifier-wrap) returns the FQDN class name "myapp::Voltage".
//
// This is the load-bearing case for defprotocol's dispatcher. The HolonAST
// arm must invoke extract_classifier(h) and return the inner classifier
// name, NOT the variant name "wat::holon::HolonAST".
#[test]
fn probe_7_type_on_defrecord_instance() {
    assert_type_contains(
        "Probe 7 (defrecord instance)",
        "tests/reflection/probe_diagnostic_polymorphic_type_p7.wat",
        "myapp::Voltage",
    );
}

// ─── Probe 8 ────────────────────────────────────────────────────────────────
//
// `(:wat::core::type point-instance)` on a struct instance returns the FQDN
// type name WITHOUT leading colon (e.g., "myapp::Point").
//
// StructValue.type_name carries the FQDN WITH leading colon (e.g.,
// ":myapp::Point"). The struct arm must strip the leading ':' for
// consistency with extract_classifier convention.
#[test]
fn probe_8_type_on_struct_instance() {
    match run_compute_from_file("tests/reflection/probe_diagnostic_polymorphic_type_p8.wat") {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 8 (struct instance) result: {}", s);
            assert_eq!(s, "String(\"myapp::Point\")", "Probe 8: unexpected struct type name (expected no leading colon)");
        }
        Err(e) => panic!("Probe 8 (struct instance) FAILED: {}", e),
    }
}
