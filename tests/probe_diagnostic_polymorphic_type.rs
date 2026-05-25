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
//!   - HolonAST classifier-wrap → extract_classifier(holon)
//!     .unwrap_or_else(|| "wat::holon::HolonAST".to_string())
//!   - Value::Struct(sv) → sv.type_name.trim_start_matches(':').to_string()
//!   - Any other Value → Value::type_name().to_string()
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
    eval_in_frozen(&ast, &world, &env)
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {:?}", e))
}

fn assert_type_returns(probe_label: &str, src: &str, expected: &str) {
    match run_compute(src) {
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
    let src = r#"
(:wat::core::define (:user::compute -> :wat::core::String)
  (:wat::core::type 5))
"#;
    assert_type_returns("Probe 1 (i64)", src, "wat::core::i64");
}

// ─── Probe 2 ────────────────────────────────────────────────────────────────
//
// `(:wat::core::type "hello")` on a literal String returns "wat::core::String".
#[test]
fn probe_2_type_on_string() {
    let src = r#"
(:wat::core::define (:user::compute -> :wat::core::String)
  (:wat::core::type "hello"))
"#;
    assert_type_returns("Probe 2 (String)", src, "wat::core::String");
}

// ─── Probe 3 ────────────────────────────────────────────────────────────────
//
// `(:wat::core::type true)` on a literal bool returns "wat::core::bool".
#[test]
fn probe_3_type_on_bool() {
    let src = r#"
(:wat::core::define (:user::compute -> :wat::core::String)
  (:wat::core::type true))
"#;
    assert_type_returns("Probe 3 (bool)", src, "wat::core::bool");
}

// ─── Probe 4 ────────────────────────────────────────────────────────────────
//
// `(:wat::core::type :foo)` on a literal keyword returns "wat::core::keyword".
#[test]
fn probe_4_type_on_keyword() {
    let src = r#"
(:wat::core::define (:user::compute -> :wat::core::String)
  (:wat::core::type :foo))
"#;
    assert_type_returns("Probe 4 (keyword)", src, "wat::core::keyword");
}

// ─── Probe 5 ────────────────────────────────────────────────────────────────
//
// `(:wat::core::type [1 2 3])` on a Vector literal returns "wat::core::Vector".
#[test]
fn probe_5_type_on_vector() {
    let src = r#"
(:wat::core::define (:user::compute -> :wat::core::String)
  (:wat::core::type [1 2 3]))
"#;
    assert_type_returns("Probe 5 (Vector)", src, "wat::core::Vector");
}

// ─── Probe 6 ────────────────────────────────────────────────────────────────
//
// `(:wat::core::type {:a 1})` on a HashMap literal returns "wat::core::HashMap".
#[test]
fn probe_6_type_on_hashmap() {
    let src = r#"
(:wat::core::define (:user::compute -> :wat::core::String)
  (:wat::core::type {:a 1}))
"#;
    assert_type_returns("Probe 6 (HashMap)", src, "wat::core::HashMap");
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
    let src = r#"
(:wat::Record::def :myapp::Voltage [magnitude <- :wat::core::f64])

(:wat::core::define (:user::compute -> :wat::core::String)
  (:wat::core::type (:myapp::Voltage 5.0)))
"#;
    assert_type_returns("Probe 7 (defrecord instance)", src, "myapp::Voltage");
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
    let src = r#"
(:wat::core::struct :myapp::Point (x :wat::core::i64) (y :wat::core::i64))

(:wat::core::define (:user::compute -> :wat::core::String)
  (:wat::core::type (:wat::core::struct-new :myapp::Point 3 4)))
"#;
    match run_compute(src) {
        Ok(v) => {
            let s = format!("{:?}", v);
            println!("Probe 8 (struct instance) result: {}", s);
            assert!(
                s.contains("myapp::Point"),
                "Probe 8: expected return value to contain \"myapp::Point\"; got: {}",
                s
            );
            assert!(
                !s.contains(":myapp::Point"),
                "Probe 8: expected NO leading colon on struct type name (strip ':'); got: {}",
                s
            );
        }
        Err(e) => panic!("Probe 8 (struct instance) FAILED: {}", e),
    }
}
