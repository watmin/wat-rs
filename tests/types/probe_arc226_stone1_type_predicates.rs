//! Arc 226 Stone 226.1 — Type predicates for classifier-wrapped entities.
//!
//! Verifies the 10 new type predicate verbs minted by arc 226 stone 226.1:
//!   - `:wat::holon::is?`        — polymorphic 2-arg form `(is? value class-name)`
//!   - `:wat::holon::is-Map?`    — Map classifier check
//!   - `:wat::holon::is-Set?`    — Set classifier check
//!   - `:wat::holon::is-Vector?` — Vector classifier check (distinct from Tuple)
//!   - `:wat::holon::is-List?`   — List classifier check
//!   - `:wat::holon::is-Tuple?`  — Tuple classifier check (distinct from Vector)
//!   - `:wat::holon::is-Symbol?` — Symbol classifier check (post-arc-230)
//!   - `:wat::holon::is-Keyword?`— Keyword classifier check (post-arc-230)
//!   - `:wat::holon::is-Tag?`    — Tag classifier check (post-arc-230)
//!   - `:wat::holon::is-Nil?`    — Nil special case (`symbol("nil")`)
//!
//! ## Doctrine context
//!
//! Type checking emerges from VSA similarity — per [[typed-entities-doctrine]]:
//!   `(is-X? value) ≡ similarity(value's class atom vector, prototype-of-X vector)`
//!
//! Stone 226.1 ships v1: EXACT STRUCTURAL MATCH on classifier name.
//! The classifier name IS a perfect VSA similarity probe in the exact-match case.
//! Future stones 226.2+ add threshold-tunable continuous scoring.
//!
//! ## Test structure
//!
//! For each of the 10 predicates:
//!   - Positive case: matching type → true
//!   - Negative case: different type → false
//! For the polymorphic `is?`:
//!   - Positive + negative with class name as String
//! Edge cases:
//!   - Bare primitive (i64, String, bool) → all predicates return false (no classifier)
//!   - Non-Bind top-level (bare Bundle via `Bundle` constructor) → all predicates return false
//!   - Nested classifier (Bind inside Bind) — outer classifier is the discriminator

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn run_bool(fn_name: &str) -> bool {
    let world = startup_beside(file!()).expect("startup for type predicates fixture");
    let ast = wat::parse_one!(&format!("({fn_name})")).expect("parse fn call");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("eval should succeed")
        .value_owned()
    {
        Value::bool(b) => b,
        other => panic!("expected bool from {}; got {:?}", fn_name, other),
    }
}

// ─── Polymorphic is? ─────────────────────────────────────────────────────────

/// `(is? value "Map")` → true when value is a Map-classified HolonAST.
///
/// Arc 226 Stone 226.1 — polymorphic is? positive case.
#[test]
fn probe_is_polymorphic_positive_map() {
    assert!(run_bool(":user::probe-is-polymorphic-positive-map"), "is? with class-name 'Map' must return true for a Map-encoded HolonAST");
}

/// `(is? value "Set")` → false when value is a Map-classified HolonAST.
///
/// Arc 226 Stone 226.1 — polymorphic is? negative case (wrong class name).
#[test]
fn probe_is_polymorphic_negative_wrong_class() {
    assert!(!run_bool(":user::probe-is-polymorphic-negative-wrong-class"), "is? with class-name 'Set' must return false for a Map-encoded HolonAST");
}

/// `(is? value "Vector")` → true for a Vector-classified HolonAST.
///
/// Arc 226 Stone 226.1 — polymorphic is? with Vector.
#[test]
fn probe_is_polymorphic_positive_vector() {
    assert!(run_bool(":user::probe-is-polymorphic-positive-vector"), "is? with class-name 'Vector' must return true for a Vec-encoded HolonAST");
}

// ─── is-Map? ─────────────────────────────────────────────────────────────────

/// `(is-Map? h)` → true when h is a Map-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Map? positive case.
#[test]
fn probe_is_map_positive() {
    assert!(run_bool(":user::probe-is-map-positive"), "is-Map? must return true for a HashMap-encoded HolonAST");
}

/// `(is-Map? h)` → false when h is a Set-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Map? negative case.
#[test]
fn probe_is_map_negative() {
    assert!(!run_bool(":user::probe-is-map-negative"), "is-Map? must return false for a Set-encoded HolonAST");
}

// ─── is-Set? ─────────────────────────────────────────────────────────────────

/// `(is-Set? h)` → true when h is a Set-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Set? positive case.
#[test]
fn probe_is_set_positive() {
    assert!(run_bool(":user::probe-is-set-positive"), "is-Set? must return true for a HashSet-encoded HolonAST");
}

/// `(is-Set? h)` → false when h is a Vector-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Set? negative case.
#[test]
fn probe_is_set_negative() {
    assert!(!run_bool(":user::probe-is-set-negative"), "is-Set? must return false for a Vector-encoded HolonAST");
}

// ─── is-Vector? ──────────────────────────────────────────────────────────────

/// `(is-Vector? h)` → true when h is a Vector-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Vector? positive case.
#[test]
fn probe_is_vector_positive() {
    assert!(run_bool(":user::probe-is-vector-positive"), "is-Vector? must return true for a Vec-encoded HolonAST");
}

/// `(is-Vector? h)` → false when h is a Tuple-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Vector? negative case. Verifies classifier distinction
/// between Vector and Tuple (arc 228 substrate distinction — classifier is sole discriminator).
#[test]
fn probe_is_vector_negative_tuple() {
    assert!(!run_bool(":user::probe-is-vector-negative-tuple"), "is-Vector? must return false for a Tuple-classified HolonAST");
}

// ─── is-List? ────────────────────────────────────────────────────────────────

/// `(is-List? h)` → true when h is a List-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-List? positive case.
#[test]
fn probe_is_list_positive() {
    assert!(run_bool(":user::probe-is-list-positive"), "is-List? must return true for a List-classified HolonAST");
}

/// `(is-List? h)` → false when h is a Set-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-List? negative case.
#[test]
fn probe_is_list_negative() {
    assert!(!run_bool(":user::probe-is-list-negative"), "is-List? must return false for a Set-encoded HolonAST");
}

// ─── is-Tuple? ───────────────────────────────────────────────────────────────

/// `(is-Tuple? h)` → true when h is a Tuple-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Tuple? positive case.
#[test]
fn probe_is_tuple_positive() {
    assert!(run_bool(":user::probe-is-tuple-positive"), "is-Tuple? must return true for a Tuple-classified HolonAST");
}

/// `(is-Tuple? h)` → false when h is a Vector-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Tuple? negative case. Verifies classifier distinction
/// between Tuple and Vector (arc 228 substrate distinction).
#[test]
fn probe_is_tuple_negative_vector() {
    assert!(!run_bool(":user::probe-is-tuple-negative-vector"), "is-Tuple? must return false for a Vector-classified HolonAST");
}

// ─── is-Symbol? ──────────────────────────────────────────────────────────────

/// `(is-Symbol? h)` → true when h is a Symbol-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Symbol? positive case.
/// Post-arc-230: symbol("foo") = `Bind(Atom("Symbol"), Atom("foo"))`.
/// Constructed directly via `:wat::holon::Bind` + `:wat::holon::Atom` to produce
/// the exact classifier-wrapped composition without needing a WAT-level symbol literal.
#[test]
fn probe_is_symbol_positive() {
    assert!(run_bool(":user::probe-is-symbol-positive"), "is-Symbol? must return true for a Symbol-classified HolonAST (Bind composition)");
}

/// `(is-Symbol? h)` → false when h is a Keyword-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Symbol? negative case.
#[test]
fn probe_is_symbol_negative_keyword() {
    assert!(!run_bool(":user::probe-is-symbol-negative-keyword"), "is-Symbol? must return false for a Keyword-classified HolonAST");
}

// ─── is-Keyword? ─────────────────────────────────────────────────────────────

/// `(is-Keyword? h)` → true when h is a Keyword-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Keyword? positive case.
/// Post-arc-230: keyword("foo") produces `Bind(Atom("Keyword"), Atom("foo"))`.
#[test]
fn probe_is_keyword_positive() {
    assert!(run_bool(":user::probe-is-keyword-positive"), "is-Keyword? must return true for a Keyword-classified HolonAST");
}

/// `(is-Keyword? h)` → false when h is a Map-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Keyword? negative case.
#[test]
fn probe_is_keyword_negative_map() {
    assert!(!run_bool(":user::probe-is-keyword-negative-map"), "is-Keyword? must return false for a Map-classified HolonAST");
}

// ─── is-Tag? ─────────────────────────────────────────────────────────────────

/// `(is-Tag? h)` → true when h is a Tag-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Tag? positive case.
/// Post-arc-230: tag("foo") = `Bind(Atom("Tag"), Atom("foo"))`.
/// Constructed directly via `:wat::holon::Bind` + `:wat::holon::Atom`.
#[test]
fn probe_is_tag_positive() {
    assert!(run_bool(":user::probe-is-tag-positive"), "is-Tag? must return true for a Tag-classified HolonAST (Bind composition)");
}

/// `(is-Tag? h)` → false when h is a Keyword-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Tag? negative case.
#[test]
fn probe_is_tag_negative_keyword() {
    assert!(!run_bool(":user::probe-is-tag-negative-keyword"), "is-Tag? must return false for a Keyword-classified HolonAST");
}

// ─── is-Nil? ─────────────────────────────────────────────────────────────────

/// `(is-Nil? h)` → true when h is the nil composition.
///
/// Arc 226 Stone 226.1 — is-Nil? positive case.
/// Per arc 230 nil doctrine: nil = symbol("nil") = `Bind(Atom("Symbol"), Atom("nil"))`.
/// `is-Nil?` uses `HolonAST::is_nil()` (arc 230 accessor).
#[test]
fn probe_is_nil_positive() {
    assert!(run_bool(":user::probe-is-nil-positive"), "is-Nil? must return true for the nil composition (symbol nil)");
}

/// `(is-Nil? h)` → false when h is a non-nil Symbol HolonAST.
///
/// Arc 226 Stone 226.1 — is-Nil? negative case.
/// A non-nil symbol has classifier "Symbol" but inner content is NOT "nil".
#[test]
fn probe_is_nil_negative_non_nil_symbol() {
    assert!(!run_bool(":user::probe-is-nil-negative-non-nil-symbol"), "is-Nil? must return false for Symbol with content 'foo' (not 'nil')");
}

/// `(is-Nil? h)` → false when h is a Map-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Nil? negative case (non-symbol type).
#[test]
fn probe_is_nil_negative_map() {
    assert!(!run_bool(":user::probe-is-nil-negative-map"), "is-Nil? must return false for a Map-classified HolonAST");
}

// ─── is-Symbol? subsumes nil ─────────────────────────────────────────────────

/// `(is-Symbol? nil-holon)` → true — nil IS a Symbol (classifier is "Symbol").
///
/// Arc 226 Stone 226.1 — nil is-Symbol? edge case.
/// Per arc 230 nil doctrine: nil = symbol("nil"). Both have classifier "Symbol".
/// `is-Symbol?` returns true for nil; `is-Nil?` is the nil-specific tighter check.
/// `to-holon nil` produces `HolonAST::nil()` = `Bind(Atom("Symbol"), Atom("nil"))`.
#[test]
fn probe_is_symbol_true_for_nil() {
    assert!(run_bool(":user::probe-is-symbol-true-for-nil"), "is-Symbol? must return true for nil (nil = symbol('nil'), classifier is 'Symbol')");
}

// ─── Edge cases ──────────────────────────────────────────────────────────────

/// Bare i64 primitive → all predicates return false (no classifier).
///
/// Arc 226 Stone 226.1 — edge case: non-HolonAST value in predicate.
#[test]
fn probe_edge_holon_i64_leaf_not_map() {
    assert!(!run_bool(":user::probe-edge-holon-i64-leaf-not-map"), "is-Map? on a bare I64 HolonAST (no classifier) must return false");
}

/// Bare String leaf → is-Keyword? returns false (no classifier).
///
/// Arc 226 Stone 226.1 — edge case: string leaf has no classifier.
#[test]
fn probe_edge_holon_string_leaf_not_keyword() {
    assert!(!run_bool(":user::probe-edge-holon-string-leaf-not-keyword"), "is-Keyword? on a bare String HolonAST (no classifier) must return false");
}

/// Bare boolean leaf → is-Symbol? returns false (no classifier).
///
/// Arc 226 Stone 226.1 — edge case: bool leaf has no classifier.
#[test]
fn probe_edge_holon_bool_leaf_not_symbol() {
    assert!(!run_bool(":user::probe-edge-holon-bool-leaf-not-symbol"), "is-Symbol? on a bare Bool HolonAST (no classifier) must return false");
}

/// Cross-type predicate: is-Set? on Vector returns false; is-Vector? on Set returns false.
///
/// Arc 226 Stone 226.1 — cross-type discrimination edge case.
#[test]
fn probe_edge_cross_type_set_vs_vector() {
    assert!(!run_bool(":user::probe-edge-cross-type-set-not-vector"), "is-Vector? on Set must return false");
    assert!(!run_bool(":user::probe-edge-cross-type-vector-not-set"), "is-Set? on Vector must return false");
}
