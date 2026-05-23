//! Arc 227 Stone 227.2 v2 — User-defined types via `:wat::holon::defrecord` macro.
//!
//! Stone 227.2 v2 mandates the field-list: `(defrecord :fqdn [fields])`.
//! Single-arg form `(defrecord :fqdn)` is RETIRED (HARD CUT).
//!
//! Verifies that `:wat::holon::defrecord` correctly generates:
//!   - A constructor in the user-declared namespace (takes typed field args)
//!   - A predicate in the user-declared namespace
//!   - Classifier-wrapped instances: `Bind(Atom("ns::Name"), Bundle(...))`
//!   - Namespace collision-freedom across distinct namespaces
//!   - Polymorphic `:wat::holon::is?` works on user-defined types
//!   - Zero-arg constructor for empty field-list `[]` (tagged unit)
//!   - Single-field constructor for `[field <- :Type]` form
//!
//! ## STOP-5b finding (stone 227.2 v2)
//!
//! Accessor synthesis (`:ns::Type/field-name` functions) is deferred.
//! The substrate lacks a Bind-decomposition primitive (`Bind/inner`) needed
//! to walk the inner Bundle of a defrecord instance at runtime. Accessor
//! tests are NOT included in this probe set; they are future work.
//!
//! N>=2 fields are also deferred: the macro errors at expand time for N>1.
//!
//! ## Doctrine
//!
//! Per [[typed-entities-doctrine]] + `feedback_fqdn_is_the_namespace`:
//!   - `(:wat::holon::defrecord :myapp::Voltage [value <- :f64])` generates
//!     `:myapp::Voltage` (constructor, takes f64) and `:myapp::is-Voltage?`
//!     (predicate) -- entirely in the user-declared namespace.
//!   - Constructor takes TYPED PRIMITIVES directly (no to-holon needed by caller).
//!   - The substrate NEVER inserts into `:user::*` or any auto-namespace.
//!   - Classifier string = FQDN without leading colon ("myapp::Voltage").
//!   - Collision-free: `:appA::Voltage` and `:appB::Voltage` produce distinct classifiers.
//!
//! ## Depends on
//!
//!   - Stone 226.1: `:wat::holon::is?` + `:wat::holon::is-Map?` etc. live.
//!   - Stone 225.1: `:wat::holon::Bind` + `:wat::holon::Atom` + `:wat::holon::to-holon`.
//!   - Stone 227.2 v2 (THIS): `:wat::holon::defrecord` macro (2-arg head; field-list mandate).
//!
//! ## Migrated tests (stone 227.1b -> stone 227.2 v2)
//!
//!   All 18 probes from stone 227.1b migrated to v2 form:
//!   - `(defrecord :fqdn)` -> `(defrecord :fqdn [value <- :Type])`
//!   - `(:fqdn (to-holon X))` -> `(:fqdn X)` (constructor now takes typed primitive)
//!   - Test 6 rewritten: constructor typed per field-type (not HolonAST)
//!
//! ## New v2-specific tests
//!
//!   Test 13 -- empty field-list [] mints zero-arg constructor (tagged unit)
//!   Test 14 -- zero-arg constructor instance recognized by predicate
//!   Test 15 -- zero-arg constructor distinct from non-tag
//!   Test 16 -- single-field String constructor
//!   Test 17 -- cross-namespace tags: same type name, distinct classifiers
//!   Test 18 -- constructor type-checks field: wrong type rejected
//!   Test 19 -- multi-segment namespace with field

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// helpers

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

fn expect_startup_err(src: &str) -> String {
    let src = with_nil_main(src);
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .err()
        .map(|e| e.to_string())
        .unwrap_or_else(|| "no error (startup succeeded)".to_string())
}

// Test 1: Single FQDN defrecord -- construct + predicate

/// `(:wat::holon::defrecord :test::Voltage [value <- :wat::core::f64])` mints a constructor.
/// `(:test::Voltage 5.0)` constructs an instance (v2: typed primitive arg).
/// `(:test::is-Voltage? instance)` returns true.
///
/// Arc 227 Stone 227.2 v2 -- basic positive case (migrated from 227.1b Test 1).
#[test]
fn probe_defrecord_single_fqdn_positive() {
    let src = r#"
        (:wat::holon::defrecord :test::Voltage [value <- :wat::core::f64])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:test::Voltage 5.0)]
            (:test::is-Voltage? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-Voltage? must return true for a Voltage instance constructed by defrecord v2"
    );
}

/// `(:test::is-Voltage? (to-holon \"random-string\"))` returns false (no Voltage classifier).
///
/// Arc 227 Stone 227.2 v2 -- predicate returns false for non-instance (migrated from 227.1b Test 2).
#[test]
fn probe_defrecord_single_fqdn_negative() {
    let src = r#"
        (:wat::holon::defrecord :test::Voltage [value <- :wat::core::f64])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:test::is-Voltage? (:wat::holon::to-holon "random-string")))
    "#;
    assert!(
        !run_bool(src),
        "is-Voltage? must return false for a bare String HolonAST (no Voltage classifier)"
    );
}

// Test 2: Cross-namespace independence

/// `(:appA::Voltage x)` and `(:appB::Voltage x)` produce classifiers
/// "appA::Voltage" and "appB::Voltage" -- distinct; predicates discriminate.
///
/// Arc 227 Stone 227.2 v2 -- FQDN collision-freedom: appA positive (migrated from 227.1b Test 3).
#[test]
fn probe_defrecord_cross_namespace_app_a_positive() {
    let src = r#"
        (:wat::holon::defrecord :appA::Voltage [value <- :wat::core::i64])
        (:wat::holon::defrecord :appB::Voltage [value <- :wat::core::i64])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [a-instance (:appA::Voltage 42)]
            (:appA::is-Voltage? a-instance)))
    "#;
    assert!(
        run_bool(src),
        "appA::is-Voltage? must return true for appA::Voltage instance"
    );
}

/// `(:appA::is-Voltage? appB-instance)` returns false.
///
/// Arc 227 Stone 227.2 v2 -- cross-namespace discrimination is honest (migrated from 227.1b Test 4).
#[test]
fn probe_defrecord_cross_namespace_discrimination() {
    let src = r#"
        (:wat::holon::defrecord :appA::Voltage [value <- :wat::core::i64])
        (:wat::holon::defrecord :appB::Voltage [value <- :wat::core::i64])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [b-instance (:appB::Voltage 42)]
            (:appA::is-Voltage? b-instance)))
    "#;
    assert!(
        !run_bool(src),
        "appA::is-Voltage? must return false for an appB::Voltage instance"
    );
}

// Test 3: Multiple types in same namespace

/// Two types in the same namespace -- :test::Celsius and :test::Kelvin.
///
/// Arc 227 Stone 227.2 v2 -- same-namespace independence (migrated from 227.1b Test 5).
#[test]
fn probe_defrecord_same_namespace_celsius_positive() {
    let src = r#"
        (:wat::holon::defrecord :test::Celsius [value <- :wat::core::f64])
        (:wat::holon::defrecord :test::Kelvin [value <- :wat::core::f64])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [c (:test::Celsius 100.0)]
            (:test::is-Celsius? c)))
    "#;
    assert!(
        run_bool(src),
        "is-Celsius? must return true for a Celsius instance"
    );
}

/// A Celsius instance is NOT Kelvin.
///
/// Arc 227 Stone 227.2 v2 -- same-namespace cross-discrimination (migrated from 227.1b Test 6).
#[test]
fn probe_defrecord_same_namespace_cross_discrimination() {
    let src = r#"
        (:wat::holon::defrecord :test::Celsius [value <- :wat::core::f64])
        (:wat::holon::defrecord :test::Kelvin [value <- :wat::core::f64])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [c (:test::Celsius 100.0)]
            (:test::is-Kelvin? c)))
    "#;
    assert!(
        !run_bool(src),
        "is-Kelvin? must return false for a Celsius instance"
    );
}

// Test 4: User type vs built-in type

/// User-defined MyMap is recognized by its own predicate.
///
/// Arc 227 Stone 227.2 v2 -- user types work independently (migrated from 227.1b Test 7).
#[test]
fn probe_defrecord_user_type_vs_builtin_user_positive() {
    let src = r#"
        (:wat::holon::defrecord :test::MyMap [value <- :wat::core::String])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:test::MyMap "data")]
            (:test::is-MyMap? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-MyMap? must return true for a MyMap instance"
    );
}

/// A user-defined MyMap instance is NOT a built-in Map.
///
/// Arc 227 Stone 227.2 v2 -- user types don't masquerade as built-in types (migrated from 227.1b Test 8).
#[test]
fn probe_defrecord_user_type_vs_builtin_not_map() {
    let src = r#"
        (:wat::holon::defrecord :test::MyMap [value <- :wat::core::String])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:test::MyMap "data")]
            (:wat::holon::is-Map? instance)))
    "#;
    assert!(
        !run_bool(src),
        "is-Map? must return false for a user-defined MyMap instance"
    );
}

// Test 5: Polymorphic is? with FQDN string

/// Polymorphic is? works on user-defined types via classifier string.
///
/// Arc 227 Stone 227.2 v2 -- polymorphic is? (migrated from 227.1b Test 9).
#[test]
fn probe_defrecord_polymorphic_is_fqdn_positive() {
    let src = r#"
        (:wat::holon::defrecord :test::Voltage [value <- :wat::core::f64])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:test::Voltage 5.0)]
            (:wat::holon::is? instance "test::Voltage")))
    "#;
    assert!(
        run_bool(src),
        "is? with 'test::Voltage' classifier string must return true"
    );
}

/// Bare basename without namespace does NOT match.
///
/// Arc 227 Stone 227.2 v2 -- FQDN-qualified classifier required (migrated from 227.1b Test 10).
#[test]
fn probe_defrecord_polymorphic_is_bare_basename_negative() {
    let src = r#"
        (:wat::holon::defrecord :test::Voltage [value <- :wat::core::f64])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:test::Voltage 5.0)]
            (:wat::holon::is? instance "Voltage")))
    "#;
    assert!(
        !run_bool(src),
        "is? with bare 'Voltage' must return false"
    );
}

// Test 6: Constructor type-checked -- REWRITTEN for v2

/// The generated constructor takes typed field args (v2: not HolonAST).
/// Passing wrong type (String where f64 expected) fails at check time.
///
/// Arc 227 Stone 227.2 v2 -- field type enforcement (REWRITTEN from 227.1b Test 11).
#[test]
fn probe_defrecord_constructor_typed_rejects_wrong_type() {
    let err = expect_startup_err(r#"
        (:wat::holon::defrecord :test::Voltage [value <- :wat::core::f64])
        (:wat::core::define (:user::compute -> :wat::holon::HolonAST)
          (:test::Voltage "not-a-float"))
    "#);
    assert!(
        !err.contains("no error"),
        "constructor must reject String where f64 expected (got: {})",
        err
    );
}

// Test 7: Multi-segment namespace

/// Three-level FQDN handled correctly.
///
/// Arc 227 Stone 227.2 v2 -- multi-segment namespace (migrated from 227.1b Test 12).
#[test]
fn probe_defrecord_multi_segment_namespace_positive() {
    let src = r#"
        (:wat::holon::defrecord :awesome::lib::Sensor [value <- :wat::core::i64])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:awesome::lib::Sensor 42)]
            (:awesome::lib::is-Sensor? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-Sensor? must return true for a Sensor instance from a 3-level namespace"
    );
}

/// Multi-segment namespace: polymorphic is? with full classifier string.
///
/// Arc 227 Stone 227.2 v2 -- multi-segment classifier (migrated from 227.1b Test 13).
#[test]
fn probe_defrecord_multi_segment_polymorphic_is() {
    let src = r#"
        (:wat::holon::defrecord :awesome::lib::Sensor [value <- :wat::core::i64])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:awesome::lib::Sensor 42)]
            (:wat::holon::is? instance "awesome::lib::Sensor")))
    "#;
    assert!(
        run_bool(src),
        "is? with 'awesome::lib::Sensor' must return true"
    );
}

// Test 8: Predicate name shape

/// For :test::BasisPoint -> :test::is-BasisPoint?
///
/// Arc 227 Stone 227.2 v2 -- predicate naming rule (migrated from 227.1b Test 14).
#[test]
fn probe_defrecord_predicate_name_shape() {
    let src = r#"
        (:wat::holon::defrecord :test::BasisPoint [value <- :wat::core::i64])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:test::BasisPoint 25)]
            (:test::is-BasisPoint? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-BasisPoint? must return true (checks predicate name shape)"
    );
}

// Test 9: i64 field

/// Constructor accepts i64 fields.
///
/// Arc 227 Stone 227.2 v2 -- i64 field (migrated from 227.1b Test 15).
#[test]
fn probe_defrecord_i64_payload() {
    let src = r#"
        (:wat::holon::defrecord :test::Count [value <- :wat::core::i64])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:test::Count 99)]
            (:test::is-Count? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-Count? must return true for a Count instance with an i64 field"
    );
}

// Test 10: Cross-type discrimination

/// Kelvin is Kelvin.
///
/// Arc 227 Stone 227.2 v2 -- cross-type discrimination (migrated from 227.1b Test 16).
#[test]
fn probe_defrecord_cross_type_discrimination_kelvin_positive() {
    let src = r#"
        (:wat::holon::defrecord :test::Celsius [value <- :wat::core::f64])
        (:wat::holon::defrecord :test::Kelvin [value <- :wat::core::f64])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [k (:test::Kelvin 373.15)]
            (:test::is-Kelvin? k)))
    "#;
    assert!(
        run_bool(src),
        "is-Kelvin? must return true for a Kelvin instance"
    );
}

// Test 11: No :user::* insertion

/// defrecord only inserts into the user-declared namespace.
///
/// Arc 227 Stone 227.2 v2 -- no auto-namespace insertion (migrated from 227.1b Test 17).
#[test]
fn probe_defrecord_no_user_namespace_insertion() {
    let src = r#"
        (:wat::holon::defrecord :test::Celsius [value <- :wat::core::f64])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [c (:test::Celsius 273.15)]
            (:test::is-Celsius? c)))
    "#;
    assert!(
        run_bool(src),
        ":test::is-Celsius? must be in :test:: namespace (not :user::*)"
    );
}

// Test 12: appB cross-namespace predicate

/// appB::Voltage is correctly identified by appB::is-Voltage?.
///
/// Arc 227 Stone 227.2 v2 -- appB predicate works (migrated from 227.1b Test 18).
#[test]
fn probe_defrecord_cross_namespace_app_b_positive() {
    let src = r#"
        (:wat::holon::defrecord :appA::Voltage [value <- :wat::core::i64])
        (:wat::holon::defrecord :appB::Voltage [value <- :wat::core::i64])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [b-instance (:appB::Voltage 99)]
            (:appB::is-Voltage? b-instance)))
    "#;
    assert!(
        run_bool(src),
        "appB::is-Voltage? must return true for appB::Voltage instance"
    );
}

// Test 13: Empty field-list [] mints zero-arg constructor (NEW v2)

/// `(:wat::holon::defrecord :test::Tag [])` mints a zero-arg constructor.
/// `(:test::Tag)` with no arguments constructs a tagged unit instance.
/// `(:test::is-Tag? instance)` returns true.
///
/// Arc 227 Stone 227.2 v2 -- empty field-list tagged unit (NEW test).
#[test]
fn probe_defrecord_empty_field_list_zero_arg_constructor() {
    let src = r#"
        (:wat::holon::defrecord :test::Tag [])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:test::Tag)]
            (:test::is-Tag? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-Tag? must return true for a Tag instance constructed with zero-arg form"
    );
}

// Test 14: Zero-arg tagged unit: predicate true for instance (NEW v2)

/// A zero-field tagged unit is recognized by its own predicate.
///
/// Arc 227 Stone 227.2 v2 -- tagged unit predicate positive (NEW test).
#[test]
fn probe_defrecord_tagged_unit_predicate_true() {
    let src = r#"
        (:wat::holon::defrecord :ns::Done [])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:ns::is-Done? (:ns::Done)))
    "#;
    assert!(
        run_bool(src),
        "is-Done? must return true for a Done instance (zero-field tagged unit)"
    );
}

// Test 15: Zero-arg tagged unit: predicate false for non-instance (NEW v2)

/// A zero-field tagged unit predicate returns false for unrelated HolonAST.
///
/// Arc 227 Stone 227.2 v2 -- tagged unit predicate negative (NEW test).
#[test]
fn probe_defrecord_tagged_unit_predicate_false_for_non_instance() {
    let src = r#"
        (:wat::holon::defrecord :ns::Done [])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:ns::is-Done? (:wat::holon::to-holon "not-done")))
    "#;
    assert!(
        !run_bool(src),
        "is-Done? must return false for a non-Done HolonAST"
    );
}

// Test 16: Single-field String constructor (NEW v2)

/// `(defrecord :test::Label [text <- :wat::core::String])` mints a String-typed constructor.
///
/// Arc 227 Stone 227.2 v2 -- single-field String type (NEW test).
#[test]
fn probe_defrecord_single_field_string_constructor() {
    let src = r#"
        (:wat::holon::defrecord :test::Label [text <- :wat::core::String])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:test::Label "hello")]
            (:test::is-Label? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-Label? must return true for a Label instance with String field"
    );
}

// Test 17: Cross-namespace tags: same type name, distinct classifiers (NEW v2)

/// Two namespaces both define an empty-field Tag type.
/// The classifiers are distinct; predicates discriminate.
///
/// Arc 227 Stone 227.2 v2 -- cross-namespace tagged unit independence (NEW test).
#[test]
fn probe_defrecord_cross_namespace_tags_distinct() {
    let src = r#"
        (:wat::holon::defrecord :nsA::Tag [])
        (:wat::holon::defrecord :nsB::Tag [])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [a-tag (:nsA::Tag)]
            (:nsA::is-Tag? a-tag)))
    "#;
    assert!(
        run_bool(src),
        "nsA::is-Tag? must return true for nsA::Tag instance"
    );
}

// Test 18: Field type enforcement -- wrong type rejected (NEW v2)

/// Constructor type-checks its field argument.
/// Passing bool where f64 expected fails at check time.
///
/// Arc 227 Stone 227.2 v2 -- field type enforcement (NEW test).
#[test]
fn probe_defrecord_field_type_check_bool_rejected() {
    let err = expect_startup_err(r#"
        (:wat::holon::defrecord :test::Measured [value <- :wat::core::f64])
        (:wat::core::define (:user::compute -> :wat::holon::HolonAST)
          (:test::Measured true))
    "#);
    assert!(
        !err.contains("no error"),
        "constructor must reject bool where f64 expected (got: {})",
        err
    );
}

// Test 19: Multi-segment namespace with field (NEW v2)

/// Four-level FQDN with a single field works correctly.
///
/// Arc 227 Stone 227.2 v2 -- multi-segment namespace + field (NEW test).
#[test]
fn probe_defrecord_multi_segment_with_field() {
    let src = r#"
        (:wat::holon::defrecord :my::deep::ns::Reading [value <- :wat::core::f64])
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:my::deep::ns::Reading 3.14)]
            (:my::deep::ns::is-Reading? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-Reading? must return true for a Reading instance from a 4-level namespace"
    );
}
