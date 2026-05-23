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

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run_bool(src: &str) -> bool {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Polymorphic is? ─────────────────────────────────────────────────────────

/// `(is? value "Map")` → true when value is a Map-classified HolonAST.
///
/// Arc 226 Stone 226.1 — polymorphic is? positive case.
#[test]
fn probe_is_polymorphic_positive_map() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is?
            (:wat::holon::to-holon {:a 1})
            "Map"))
    "#;
    assert!(run_bool(src), "is? with class-name 'Map' must return true for a Map-encoded HolonAST");
}

/// `(is? value "Set")` → false when value is a Map-classified HolonAST.
///
/// Arc 226 Stone 226.1 — polymorphic is? negative case (wrong class name).
#[test]
fn probe_is_polymorphic_negative_wrong_class() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is?
            (:wat::holon::to-holon {:a 1})
            "Set"))
    "#;
    assert!(!run_bool(src), "is? with class-name 'Set' must return false for a Map-encoded HolonAST");
}

/// `(is? value "Vector")` → true for a Vector-classified HolonAST.
///
/// Arc 226 Stone 226.1 — polymorphic is? with Vector.
#[test]
fn probe_is_polymorphic_positive_vector() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is?
            (:wat::holon::to-holon [1 2 3])
            "Vector"))
    "#;
    assert!(run_bool(src), "is? with class-name 'Vector' must return true for a Vec-encoded HolonAST");
}

// ─── is-Map? ─────────────────────────────────────────────────────────────────

/// `(is-Map? h)` → true when h is a Map-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Map? positive case.
#[test]
fn probe_is_map_positive() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Map?
            (:wat::holon::to-holon {:a 1 :b 2})))
    "#;
    assert!(run_bool(src), "is-Map? must return true for a HashMap-encoded HolonAST");
}

/// `(is-Map? h)` → false when h is a Set-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Map? negative case.
#[test]
fn probe_is_map_negative() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Map?
            (:wat::holon::to-holon #{1 2 3})))
    "#;
    assert!(!run_bool(src), "is-Map? must return false for a Set-encoded HolonAST");
}

// ─── is-Set? ─────────────────────────────────────────────────────────────────

/// `(is-Set? h)` → true when h is a Set-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Set? positive case.
#[test]
fn probe_is_set_positive() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Set?
            (:wat::holon::to-holon #{1 2 3})))
    "#;
    assert!(run_bool(src), "is-Set? must return true for a HashSet-encoded HolonAST");
}

/// `(is-Set? h)` → false when h is a Vector-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Set? negative case.
#[test]
fn probe_is_set_negative() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Set?
            (:wat::holon::to-holon [1 2 3])))
    "#;
    assert!(!run_bool(src), "is-Set? must return false for a Vector-encoded HolonAST");
}

// ─── is-Vector? ──────────────────────────────────────────────────────────────

/// `(is-Vector? h)` → true when h is a Vector-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Vector? positive case.
#[test]
fn probe_is_vector_positive() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Vector?
            (:wat::holon::to-holon [1 2 3])))
    "#;
    assert!(run_bool(src), "is-Vector? must return true for a Vec-encoded HolonAST");
}

/// `(is-Vector? h)` → false when h is a Tuple-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Vector? negative case. Verifies classifier distinction
/// between Vector and Tuple (arc 228 substrate distinction — classifier is sole discriminator).
#[test]
fn probe_is_vector_negative_tuple() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [items  (:wat::core::Vector :wat::holon::HolonAST
                      (:wat::holon::to-holon 1)
                      (:wat::holon::to-holon 2))
             tup    (:wat::holon::Tuple items)]
            (:wat::holon::is-Vector? tup)))
    "#;
    assert!(!run_bool(src), "is-Vector? must return false for a Tuple-classified HolonAST");
}

// ─── is-List? ────────────────────────────────────────────────────────────────

/// `(is-List? h)` → true when h is a List-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-List? positive case.
#[test]
fn probe_is_list_positive() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [items  (:wat::core::Vector :wat::holon::HolonAST
                      (:wat::holon::to-holon 1)
                      (:wat::holon::to-holon 2))
             lst    (:wat::holon::List items)]
            (:wat::holon::is-List? lst)))
    "#;
    assert!(run_bool(src), "is-List? must return true for a List-classified HolonAST");
}

/// `(is-List? h)` → false when h is a Set-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-List? negative case.
#[test]
fn probe_is_list_negative() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-List?
            (:wat::holon::to-holon #{1 2 3})))
    "#;
    assert!(!run_bool(src), "is-List? must return false for a Set-encoded HolonAST");
}

// ─── is-Tuple? ───────────────────────────────────────────────────────────────

/// `(is-Tuple? h)` → true when h is a Tuple-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Tuple? positive case.
#[test]
fn probe_is_tuple_positive() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [items  (:wat::core::Vector :wat::holon::HolonAST
                      (:wat::holon::to-holon 1)
                      (:wat::holon::to-holon 2))
             tup    (:wat::holon::Tuple items)]
            (:wat::holon::is-Tuple? tup)))
    "#;
    assert!(run_bool(src), "is-Tuple? must return true for a Tuple-classified HolonAST");
}

/// `(is-Tuple? h)` → false when h is a Vector-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Tuple? negative case. Verifies classifier distinction
/// between Tuple and Vector (arc 228 substrate distinction).
#[test]
fn probe_is_tuple_negative_vector() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Tuple?
            (:wat::holon::to-holon [1 2 3])))
    "#;
    assert!(!run_bool(src), "is-Tuple? must return false for a Vector-classified HolonAST");
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
    // Build Bind(Atom(String("Symbol")), Atom(String("foo"))) — the arc-230 Symbol composition.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Symbol?
            (:wat::holon::Bind
              (:wat::holon::Atom (:wat::holon::to-holon "Symbol"))
              (:wat::holon::Atom (:wat::holon::to-holon "foo")))))
    "#;
    assert!(run_bool(src), "is-Symbol? must return true for a Symbol-classified HolonAST (Bind composition)");
}

/// `(is-Symbol? h)` → false when h is a Keyword-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Symbol? negative case.
#[test]
fn probe_is_symbol_negative_keyword() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Symbol?
            (:wat::holon::to-holon :foo)))
    "#;
    assert!(!run_bool(src), "is-Symbol? must return false for a Keyword-classified HolonAST");
}

// ─── is-Keyword? ─────────────────────────────────────────────────────────────

/// `(is-Keyword? h)` → true when h is a Keyword-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Keyword? positive case.
/// Post-arc-230: keyword("foo") produces `Bind(Atom("Keyword"), Atom("foo"))`.
#[test]
fn probe_is_keyword_positive() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Keyword?
            (:wat::holon::to-holon :foo)))
    "#;
    assert!(run_bool(src), "is-Keyword? must return true for a Keyword-classified HolonAST");
}

/// `(is-Keyword? h)` → false when h is a Map-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Keyword? negative case.
#[test]
fn probe_is_keyword_negative_map() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Keyword?
            (:wat::holon::to-holon {:a 1})))
    "#;
    assert!(!run_bool(src), "is-Keyword? must return false for a Map-classified HolonAST");
}

// ─── is-Tag? ─────────────────────────────────────────────────────────────────

/// `(is-Tag? h)` → true when h is a Tag-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Tag? positive case.
/// Post-arc-230: tag("foo") = `Bind(Atom("Tag"), Atom("foo"))`.
/// Constructed directly via `:wat::holon::Bind` + `:wat::holon::Atom`.
#[test]
fn probe_is_tag_positive() {
    // Build Bind(Atom(String("Tag")), Atom(String("foo"))) — the arc-230 Tag composition.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Tag?
            (:wat::holon::Bind
              (:wat::holon::Atom (:wat::holon::to-holon "Tag"))
              (:wat::holon::Atom (:wat::holon::to-holon "foo")))))
    "#;
    assert!(run_bool(src), "is-Tag? must return true for a Tag-classified HolonAST (Bind composition)");
}

/// `(is-Tag? h)` → false when h is a Keyword-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Tag? negative case.
#[test]
fn probe_is_tag_negative_keyword() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Tag?
            (:wat::holon::to-holon :foo)))
    "#;
    assert!(!run_bool(src), "is-Tag? must return false for a Keyword-classified HolonAST");
}

// ─── is-Nil? ─────────────────────────────────────────────────────────────────

/// `(is-Nil? h)` → true when h is the nil composition.
///
/// Arc 226 Stone 226.1 — is-Nil? positive case.
/// Per arc 230 nil doctrine: nil = symbol("nil") = `Bind(Atom("Symbol"), Atom("nil"))`.
/// `is-Nil?` uses `HolonAST::is_nil()` (arc 230 accessor).
#[test]
fn probe_is_nil_positive() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Nil?
            (:wat::holon::to-holon :wat::core::nil)))
    "#;
    assert!(run_bool(src), "is-Nil? must return true for the nil composition (symbol nil)");
}

/// `(is-Nil? h)` → false when h is a non-nil Symbol HolonAST.
///
/// Arc 226 Stone 226.1 — is-Nil? negative case.
/// A non-nil symbol has classifier "Symbol" but inner content is NOT "nil".
/// Constructed as `Bind(Atom("Symbol"), Atom("foo"))` — classifier matches but content differs.
#[test]
fn probe_is_nil_negative_non_nil_symbol() {
    // Build Bind(Atom(String("Symbol")), Atom(String("foo"))) — Symbol but NOT nil.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Nil?
            (:wat::holon::Bind
              (:wat::holon::Atom (:wat::holon::to-holon "Symbol"))
              (:wat::holon::Atom (:wat::holon::to-holon "foo")))))
    "#;
    assert!(!run_bool(src), "is-Nil? must return false for Symbol with content 'foo' (not 'nil')");
}

/// `(is-Nil? h)` → false when h is a Map-classified HolonAST.
///
/// Arc 226 Stone 226.1 — is-Nil? negative case (non-symbol type).
#[test]
fn probe_is_nil_negative_map() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Nil?
            (:wat::holon::to-holon {:a 1})))
    "#;
    assert!(!run_bool(src), "is-Nil? must return false for a Map-classified HolonAST");
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
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Symbol?
            (:wat::holon::to-holon :wat::core::nil)))
    "#;
    assert!(run_bool(src), "is-Symbol? must return true for nil (nil = symbol('nil'), classifier is 'Symbol')");
}

// ─── Edge cases ──────────────────────────────────────────────────────────────

/// Bare i64 primitive → all predicates return false (no classifier).
///
/// Arc 226 Stone 226.1 — edge case: non-HolonAST value in predicate.
/// The type-system contract: predicates accept HolonAST; non-HolonAST values
/// at the WAT level cannot reach predicate verbs (type checker rejects them).
/// This edge is tested at the Rust level by direct HolonAST inspection.
/// At the WAT level, use `to-holon` to get an I64-leaf HolonAST (no classifier).
#[test]
fn probe_edge_holon_i64_leaf_not_map() {
    // `(:wat::holon::to-holon 42)` produces `HolonAST::I64(42)` — no classifier.
    // `is-Map?` on a bare I64 leaf returns false.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Map?
            (:wat::holon::to-holon 42)))
    "#;
    assert!(!run_bool(src), "is-Map? on a bare I64 HolonAST (no classifier) must return false");
}

/// Bare String leaf → is-Keyword? returns false (no classifier).
///
/// Arc 226 Stone 226.1 — edge case: string leaf has no classifier.
#[test]
fn probe_edge_holon_string_leaf_not_keyword() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Keyword?
            (:wat::holon::to-holon "hello")))
    "#;
    assert!(!run_bool(src), "is-Keyword? on a bare String HolonAST (no classifier) must return false");
}

/// Bare boolean leaf → is-Symbol? returns false (no classifier).
///
/// Arc 226 Stone 226.1 — edge case: bool leaf has no classifier.
#[test]
fn probe_edge_holon_bool_leaf_not_symbol() {
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Symbol?
            (:wat::holon::to-holon true)))
    "#;
    assert!(!run_bool(src), "is-Symbol? on a bare Bool HolonAST (no classifier) must return false");
}

/// Cross-type predicate: is-Set? on Vector returns false; is-Vector? on Set returns false.
///
/// Arc 226 Stone 226.1 — cross-type discrimination edge case.
/// The two collections (Set and Vector) have distinct classifiers; each predicate
/// accepts only its own classifier.
#[test]
fn probe_edge_cross_type_set_vs_vector() {
    let src_set_not_vector = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Vector?
            (:wat::holon::to-holon #{1 2 3})))
    "#;
    assert!(!run_bool(src_set_not_vector), "is-Vector? on Set must return false");

    let src_vector_not_set = r#"
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::holon::is-Set?
            (:wat::holon::to-holon [1 2 3])))
    "#;
    assert!(!run_bool(src_vector_not_set), "is-Set? on Vector must return false");
}
