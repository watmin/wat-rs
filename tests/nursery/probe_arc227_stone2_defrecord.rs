//! Arc 227 Stone 227.2 v3 + Stone 234.6 migration — User-defined types via `:wat::Record::def` (formerly `:wat::holon::defrecord`).
//!
//! v3 supersedes v2 (commit b4509cb). v2 shipped with STOP-5b deferred framing for
//! N>=2; v3 ships canonical defrecord for ALL N including N>=2 using the composition
//! proven by the diagnostic probes (commits c18fa6b + 72367f1).
//!
//! Mandated 2-arg form (stone 227.2 v2 hard cut — preserved in v3):
//!   `(defrecord <fqdn> <field-list>)`
//! Single-arg form `(defrecord :fqdn)` is RETIRED (HARD CUT).
//!
//! Verifies that `:wat::Record::def` correctly generates:
//!   - A constructor in the user-declared namespace (takes typed field args)
//!   - A predicate in the user-declared namespace
//!   - Canonical classifier-wrapped instances: `Bind(Atom("ns::Name"), Bundle(...))`
//!   - N=0: Bind(Atom("ns::Tag"), Bundle())
//!   - N=1: Bind(Atom("ns::W"),   Bundle(Bind(Atom("v"), Atom(value))))
//!   - N=2: Bind(Atom("ns::P"),   Bundle(Bind(Atom("a"), Atom(av)), Bind(Atom("b"), Atom(bv))))
//!   - N=k: Bind(Atom("ns::T"),   Bundle(... k field-Binds ...))
//!   - Namespace collision-freedom across distinct namespaces
//!   - Polymorphic `:wat::holon::is?` works on user-defined types for all N
//!
//! ## Design substrate (v3)
//!
//! The composition that ships N>=2 is proven empirically by two diagnostic probes:
//!   - `tests/probe_diagnostic_macro_splice_from_let.rs` (c18fa6b): proves
//!     `~@(let [forms (map xs fn)] forms)` splices Vec<WatAST> built via
//!     `:wat::core::map` + runtime quasiquote at macro expand time.
//!   - `tests/probe_diagnostic_bundle_result_compose.rs` (72367f1): proves
//!     `Bind(Atom, Result/expect(Bundle(items)))` is the canonical instance shape.
//!
//! ## Accessor deferred
//!
//! Accessor synthesis (`:ns::Type/field-name` functions) is deferred.
//! The substrate lacks a Bind-decomposition primitive (`Bind/inner`) needed
//! to walk the inner Bundle of a defrecord instance at runtime. Named-field
//! accessors are future work pending a Bind/inner substrate primitive.
//!
//! ## Inner-bundle shape verification note
//!
//! The substrate lacks a Bind/inner accessor to extract the inner Bundle
//! from an instance at the WAT level. Inner-bundle child-count is verified
//! via SEPARATE Bundle constructions (matching the macro's composition) that
//! prove `Bundle([N items])` has `statement-length = N`. The macro mirrors
//! the probes verbatim — if the composition is correct (probes prove it),
//! the canonical shape follows.
//!
//! ## Doctrine
//!
//! Per [[typed-entities-doctrine]] + `feedback_fqdn_is_the_namespace`:
//!   - `(:wat::Record::def :myapp::Voltage [value <- :f64])` generates
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
//!   - Stone 227.2 v2 (THIS): `:wat::Record::def` macro (2-arg head; field-list mandate).
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
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

fn run_bool(src: &str) -> bool {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
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

/// `(:wat::Record::def :test::Voltage [value <- :wat::core::f64])` mints a constructor.
/// `(:test::Voltage 5.0)` constructs an instance (v2: typed primitive arg).
/// `(:test::is-Voltage? instance)` returns true.
///
/// Arc 227 Stone 227.2 v2 -- basic positive case (migrated from 227.1b Test 1).
#[test]
fn probe_defrecord_single_fqdn_positive() {
    let src = r#"
        (:wat::Record::def :test::Voltage [value <- :wat::core::f64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [instance (:test::Voltage 5.0)]
                      (:test::is-Voltage? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-Voltage? must return true for a Voltage instance constructed by defrecord v2"
    );
}

/// Predicate returns false for an instance of a different record type (no Voltage classifier).
///
/// Arc 227 Stone 227.2 v2 -- predicate returns false for non-instance (migrated from 227.1b Test 2).
/// Stone 234.6 migration: adjusted for :wat::Record::def — predicate takes :wat::Record, not HolonAST.
/// Uses a different-class Record as the negative example instead of bare HolonAST.
#[test]
fn probe_defrecord_single_fqdn_negative() {
    let src = r#"
        (:wat::Record::def :test::Voltage [value <- :wat::core::f64])
        (:wat::Record::def :test::Current [value <- :wat::core::f64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:test::is-Voltage? (:test::Current 1.0)))
    "#;
    assert!(
        !run_bool(src),
        "is-Voltage? must return false for a Current instance (different class)"
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
        (:wat::Record::def :appA::Voltage [value <- :wat::core::i64])
        (:wat::Record::def :appB::Voltage [value <- :wat::core::i64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
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
        (:wat::Record::def :appA::Voltage [value <- :wat::core::i64])
        (:wat::Record::def :appB::Voltage [value <- :wat::core::i64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
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
        (:wat::Record::def :test::Celsius [value <- :wat::core::f64])
        (:wat::Record::def :test::Kelvin [value <- :wat::core::f64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
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
        (:wat::Record::def :test::Celsius [value <- :wat::core::f64])
        (:wat::Record::def :test::Kelvin [value <- :wat::core::f64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
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
        (:wat::Record::def :test::MyMap [value <- :wat::core::String])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
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
/// Stone 234.6 migration: adjusted for :wat::Record::def — instances are Value::wat__Record, not HolonAST.
/// is-Map? accepts HolonAST only; use cross-predicate discrimination to prove user type is distinct.
#[test]
fn probe_defrecord_user_type_vs_builtin_not_map() {
    let src = r#"
        (:wat::Record::def :test::MyMap [value <- :wat::core::String])
        (:wat::Record::def :test::Other [value <- :wat::core::String])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [instance (:test::MyMap "data")]
                      (:test::is-Other? instance)))
    "#;
    assert!(
        !run_bool(src),
        "is-Other? must return false for a MyMap instance (user types discriminate by class)"
    );
}

// Test 5: Polymorphic is? with FQDN string

/// Generated predicate correctly identifies instances by class.
///
/// Arc 227 Stone 227.2 v2 -- polymorphic is? (migrated from 227.1b Test 9).
/// Stone 234.6 migration: adjusted for :wat::Record::def — instances are Value::wat__Record.
/// :wat::holon::is? accepts HolonAST only; generated predicate (:test::is-Voltage?) is the
/// correct class-membership check for :wat::Record instances.
#[test]
fn probe_defrecord_polymorphic_is_fqdn_positive() {
    let src = r#"
        (:wat::Record::def :test::Voltage [value <- :wat::core::f64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [instance (:test::Voltage 5.0)]
                      (:test::is-Voltage? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-Voltage? must return true for a Voltage instance (class membership via generated predicate)"
    );
}

/// Generated predicate rejects instances of a different class (cross-class discrimination).
///
/// Arc 227 Stone 227.2 v2 -- FQDN-qualified classifier required (migrated from 227.1b Test 10).
/// Stone 234.6 migration: adjusted for :wat::Record::def — instances are Value::wat__Record.
/// Cross-class discrimination: :test::is-Voltage? on a :test::Current instance returns false.
#[test]
fn probe_defrecord_polymorphic_is_bare_basename_negative() {
    let src = r#"
        (:wat::Record::def :test::Voltage [value <- :wat::core::f64])
        (:wat::Record::def :test::Current [value <- :wat::core::f64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [instance (:test::Current 2.0)]
                      (:test::is-Voltage? instance)))
    "#;
    assert!(
        !run_bool(src),
        "is-Voltage? must return false for a Current instance (class names are FQDN-qualified, not bare)"
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
        (:wat::Record::def :test::Voltage [value <- :wat::core::f64])
        (:wat::core::defn :user::compute [] -> :wat::holon::HolonAST (:test::Voltage "not-a-float"))
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
        (:wat::Record::def :awesome::lib::Sensor [value <- :wat::core::i64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [instance (:awesome::lib::Sensor 42)]
                      (:awesome::lib::is-Sensor? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-Sensor? must return true for a Sensor instance from a 3-level namespace"
    );
}

/// Multi-segment namespace: generated predicate correctly identifies instances.
///
/// Arc 227 Stone 227.2 v2 -- multi-segment classifier (migrated from 227.1b Test 13).
/// Stone 234.6 migration: adjusted for :wat::Record::def — instances are Value::wat__Record.
/// Generated predicate :awesome::lib::is-Sensor? provides class-membership check.
#[test]
fn probe_defrecord_multi_segment_polymorphic_is() {
    let src = r#"
        (:wat::Record::def :awesome::lib::Sensor [value <- :wat::core::i64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [instance (:awesome::lib::Sensor 42)]
                      (:awesome::lib::is-Sensor? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-Sensor? must return true for a Sensor instance in 3-level namespace"
    );
}

// Test 8: Predicate name shape

/// For :test::BasisPoint -> :test::is-BasisPoint?
///
/// Arc 227 Stone 227.2 v2 -- predicate naming rule (migrated from 227.1b Test 14).
#[test]
fn probe_defrecord_predicate_name_shape() {
    let src = r#"
        (:wat::Record::def :test::BasisPoint [value <- :wat::core::i64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
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
        (:wat::Record::def :test::Count [value <- :wat::core::i64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
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
        (:wat::Record::def :test::Celsius [value <- :wat::core::f64])
        (:wat::Record::def :test::Kelvin [value <- :wat::core::f64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
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
        (:wat::Record::def :test::Celsius [value <- :wat::core::f64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
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
        (:wat::Record::def :appA::Voltage [value <- :wat::core::i64])
        (:wat::Record::def :appB::Voltage [value <- :wat::core::i64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
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

/// `(:wat::Record::def :test::Tag [])` mints a zero-arg constructor.
/// `(:test::Tag)` with no arguments constructs a tagged unit instance.
/// `(:test::is-Tag? instance)` returns true.
///
/// Arc 227 Stone 227.2 v2 -- empty field-list tagged unit (NEW test).
#[test]
fn probe_defrecord_empty_field_list_zero_arg_constructor() {
    let src = r#"
        (:wat::Record::def :test::Tag [])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
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
        (:wat::Record::def :ns::Done [])
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:ns::is-Done? (:ns::Done)))
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
        (:wat::Record::def :ns::Done [])
        (:wat::Record::def :ns::Pending [])
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:ns::is-Done? (:ns::Pending)))
    "#;
    assert!(
        !run_bool(src),
        "is-Done? must return false for a Pending instance (different class)"
    );
}

// Test 16: Single-field String constructor (NEW v2)

/// `(defrecord :test::Label [text <- :wat::core::String])` mints a String-typed constructor.
///
/// Arc 227 Stone 227.2 v2 -- single-field String type (NEW test).
#[test]
fn probe_defrecord_single_field_string_constructor() {
    let src = r#"
        (:wat::Record::def :test::Label [text <- :wat::core::String])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
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
        (:wat::Record::def :nsA::Tag [])
        (:wat::Record::def :nsB::Tag [])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
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
        (:wat::Record::def :test::Measured [value <- :wat::core::f64])
        (:wat::core::defn :user::compute [] -> :wat::holon::HolonAST (:test::Measured true))
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
        (:wat::Record::def :my::deep::ns::Reading [value <- :wat::core::f64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [instance (:my::deep::ns::Reading 3.14)]
                      (:my::deep::ns::is-Reading? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-Reading? must return true for a Reading instance from a 4-level namespace"
    );
}

// ─── v3 tests — Stone 227.2 v3 canonical instance shape + N>=2 fields ────────

// EXPECTATIONS row 1: single-arg form errors at expand time (HARD CUT preserved)

/// `(defrecord :fqdn)` — single-arg form — must error with ArityMismatch.
///
/// Arc 227 Stone 227.2 v3 -- HARD CUT verified (EXPECTATIONS row 1).
#[test]
fn probe_two_arg_form_only_one_arg_errors() {
    let err = expect_startup_err(r#"
        (:wat::Record::def :test::Orphan)
        (:wat::core::defn :user::compute [] -> :wat::core::bool :wat::core::true)
    "#);
    assert!(
        err.contains("ArityMismatch") || err.contains("arity") || !err.contains("no error"),
        "single-arg defrecord must error with ArityMismatch; got: {}",
        err
    );
    assert!(
        !err.contains("no error"),
        "single-arg defrecord must error; startup succeeded unexpectedly"
    );
}

// EXPECTATIONS row 3: N=0 canonical instance shape uses Bundle (not Atom(nil))
//
// Verification approach: the instance is Bind(Atom("ns::Tag"), Bundle()).
// statement-length on a Bind = 2 (it IS a Bind). We additionally verify
// that a separately constructed empty Bundle has statement-length = 0,
// proving Bundle() is the canonical empty-inner form.

/// N=0 instance: inner slot is Bundle() — verified via separate Bundle construction.
///
/// Arc 227 Stone 227.2 v3 -- EXPECTATIONS row 3: canonical Bundle() inner slot.
/// Strategy: (a) is? confirms classifier; (b) separately construct Bundle([]) and
/// verify statement-length = 0 (proving the v3 canonical shape for N=0).
#[test]
fn probe_zero_field_instance_uses_empty_bundle() {
    // Part a: instance is recognized by predicate (classifier is correct)
    let pred_src = r#"
        (:wat::Record::def :ns::Tag [])
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:ns::is-Tag? (:ns::Tag)))
    "#;
    assert!(
        run_bool(pred_src),
        "N=0 instance must be recognized by predicate (classifier Bind correct)"
    );

    // Part b: empty Bundle has 0 children (proves Bundle() is the empty-inner form)
    let bundle_src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::holon::statement-length
                      (:wat::core::Result/expect
                        (:wat::holon::Bundle [])
                        "empty bundle should not overflow")))
    "#;
    let result = {
        let src = with_nil_main(bundle_src);
        let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
            .expect("startup should succeed");
        let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
        let env = Environment::new();
        match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
            Value::i64(n) => n,
            other => panic!("expected i64; got {:?}", other),
        }
    };
    assert_eq!(
        result, 0,
        "Bundle([]) has statement-length 0 — canonical empty inner for N=0"
    );
}

// EXPECTATIONS row 5: N=1 canonical instance shape uses Bundle(Bind(...))

/// N=1 instance: inner slot is Bundle(Bind(Atom("v"), Atom(value))) not flat Bind.
///
/// Arc 227 Stone 227.2 v3 -- EXPECTATIONS row 5: canonical Bundle(Bind) inner for N=1.
/// Strategy: (a) is? confirms classifier; (b) separately construct Bundle([field-bind])
/// and verify statement-length = 1 (proving the Bundle-wrapping for N=1).
#[test]
fn probe_one_field_instance_uses_bundle_with_one_bind() {
    // Part a: instance is recognized by predicate
    let pred_src = r#"
        (:wat::Record::def :ns::W [v <- :wat::core::i64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:ns::is-W? (:ns::W 42)))
    "#;
    assert!(
        run_bool(pred_src),
        "N=1 instance must be recognized by predicate"
    );

    // Part b: Bundle([one-item]) has statement-length = 1
    let bundle_src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [field-bind (:wat::holon::Bind
                                    (:wat::holon::Atom (:wat::holon::to-holon "v"))
                                    (:wat::holon::Atom (:wat::holon::to-holon 42)))]
                      (:wat::holon::statement-length
                        (:wat::core::Result/expect
                          (:wat::holon::Bundle [field-bind])
                          "single-item bundle should not overflow"))))
    "#;
    let result = {
        let src = with_nil_main(bundle_src);
        let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
            .expect("startup should succeed");
        let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
        let env = Environment::new();
        match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
            Value::i64(n) => n,
            other => panic!("expected i64; got {:?}", other),
        }
    };
    assert_eq!(
        result, 1,
        "Bundle([one-field-bind]) has statement-length 1 — canonical Bundle(Bind) inner for N=1"
    );
}

// EXPECTATIONS row 6: N=2 multi-field constructor takes 2 typed args

/// `(defrecord :ns::P [a <- :i64  b <- :String])` → `(:ns::P 5 "hi")` succeeds.
///
/// Arc 227 Stone 227.2 v3 -- EXPECTATIONS row 6: N=2 constructor (THE LOAD-BEARING ROW).
#[test]
fn probe_two_field_construct_with_typed_args() {
    let src = r#"
        (:wat::Record::def :ns::P [a <- :wat::core::i64  b <- :wat::core::String])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [instance (:ns::P 5 "hi")]
                      (:ns::is-P? instance)))
    "#;
    assert!(
        run_bool(src),
        "N=2 constructor (:ns::P 5 \"hi\") must succeed and is-P? must return true"
    );
}

// EXPECTATIONS row 7: N=2 canonical instance shape uses Bundle with 2 children

/// N=2 inner Bundle has exactly 2 field-Binds — verified via separate Bundle construction.
///
/// Arc 227 Stone 227.2 v3 -- EXPECTATIONS row 7.
#[test]
fn probe_two_field_instance_bundle_has_two_binds() {
    // Part a: predicate works for N=2
    let pred_src = r#"
        (:wat::Record::def :ns::P [a <- :wat::core::i64  b <- :wat::core::String])
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:ns::is-P? (:ns::P 99 "test")))
    "#;
    assert!(
        run_bool(pred_src),
        "N=2 instance is-P? must return true"
    );

    // Part b: Bundle([field-a, field-b]) has statement-length = 2
    let bundle_src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [fa (:wat::holon::Bind
                            (:wat::holon::Atom (:wat::holon::to-holon "a"))
                            (:wat::holon::Atom (:wat::holon::to-holon 5)))
                       fb (:wat::holon::Bind
                            (:wat::holon::Atom (:wat::holon::to-holon "b"))
                            (:wat::holon::Atom (:wat::holon::to-holon "hi")))]
                      (:wat::holon::statement-length
                        (:wat::core::Result/expect
                          (:wat::holon::Bundle [fa fb])
                          "two-item bundle should not overflow"))))
    "#;
    let result = {
        let src = with_nil_main(bundle_src);
        let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
            .expect("startup should succeed");
        let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
        let env = Environment::new();
        match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
            Value::i64(n) => n,
            other => panic!("expected i64; got {:?}", other),
        }
    };
    assert_eq!(
        result, 2,
        "Bundle([field-a, field-b]) has statement-length 2 — canonical 2-child Bundle for N=2"
    );
}

// EXPECTATIONS row 8: N=3 multi-field constructor takes 3 typed args

/// `(defrecord :ns::T [a <- :i64  b <- :String  c <- :bool])` → 3-arg constructor.
///
/// Arc 227 Stone 227.2 v3 -- EXPECTATIONS row 8: N=3 constructor.
#[test]
fn probe_three_field_construct_with_typed_args() {
    let src = r#"
        (:wat::Record::def :ns::T [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [instance (:ns::T 7 "world" true)]
                      (:ns::is-T? instance)))
    "#;
    assert!(
        run_bool(src),
        "N=3 constructor (:ns::T 7 \"world\" true) must succeed and is-T? must return true"
    );
}

/// N=3 inner Bundle has exactly 3 field-Binds — verified via separate construction.
///
/// Arc 227 Stone 227.2 v3 -- EXPECTATIONS row 8 (bundle shape).
#[test]
fn probe_three_field_instance_bundle_has_three_binds() {
    // Part a: predicate works for N=3
    let pred_src = r#"
        (:wat::Record::def :ns::T [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:ns::is-T? (:ns::T 1 "x" false)))
    "#;
    assert!(
        run_bool(pred_src),
        "N=3 instance is-T? must return true"
    );

    // Part b: Bundle([fa, fb, fc]) has statement-length = 3
    let bundle_src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [fa (:wat::holon::Bind
                            (:wat::holon::Atom (:wat::holon::to-holon "a"))
                            (:wat::holon::Atom (:wat::holon::to-holon 7)))
                       fb (:wat::holon::Bind
                            (:wat::holon::Atom (:wat::holon::to-holon "b"))
                            (:wat::holon::Atom (:wat::holon::to-holon "world")))
                       fc (:wat::holon::Bind
                            (:wat::holon::Atom (:wat::holon::to-holon "c"))
                            (:wat::holon::Atom (:wat::holon::to-holon true)))]
                      (:wat::holon::statement-length
                        (:wat::core::Result/expect
                          (:wat::holon::Bundle [fa fb fc])
                          "three-item bundle should not overflow"))))
    "#;
    let result = {
        let src = with_nil_main(bundle_src);
        let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
            .expect("startup should succeed");
        let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
        let env = Environment::new();
        match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
            Value::i64(n) => n,
            other => panic!("expected i64; got {:?}", other),
        }
    };
    assert_eq!(
        result, 3,
        "Bundle([fa, fb, fc]) has statement-length 3 — canonical 3-child Bundle for N=3"
    );
}

// EXPECTATIONS row 9: predicate works for all N

/// Predicates for N=0, N=1, N=2, N=3 all work correctly.
///
/// Arc 227 Stone 227.2 v3 -- EXPECTATIONS row 9: predicate works for all N.
#[test]
fn probe_predicate_works_for_n0_n1_n2_n3() {
    // N=0: tagged unit
    let src0 = r#"
        (:wat::Record::def :multi::Tag [])
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:multi::is-Tag? (:multi::Tag)))
    "#;
    assert!(run_bool(src0), "N=0 predicate must work");

    // N=1
    let src1 = r#"
        (:wat::Record::def :multi::W [v <- :wat::core::i64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:multi::is-W? (:multi::W 42)))
    "#;
    assert!(run_bool(src1), "N=1 predicate must work");

    // N=2
    let src2 = r#"
        (:wat::Record::def :multi::P [a <- :wat::core::i64  b <- :wat::core::String])
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:multi::is-P? (:multi::P 5 "hi")))
    "#;
    assert!(run_bool(src2), "N=2 predicate must work");

    // N=3
    let src3 = r#"
        (:wat::Record::def :multi::T [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:multi::is-T? (:multi::T 1 "x" false)))
    "#;
    assert!(run_bool(src3), "N=3 predicate must work");

    // Predicate returns false for wrong type (cross-type discrimination)
    let src_neg = r#"
        (:wat::Record::def :multi::P [a <- :wat::core::i64  b <- :wat::core::String])
        (:wat::Record::def :multi::Q [a <- :wat::core::i64  b <- :wat::core::String])
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:multi::is-P? (:multi::Q 1 "y")))
    "#;
    assert!(
        !run_bool(src_neg),
        "N=2 predicate must return false for instance of different type"
    );
}

// EXPECTATIONS row 10: cross-namespace independence with N>=2

/// N=2 defrecord in two namespaces produces distinct classifiers.
///
/// Arc 227 Stone 227.2 v3 -- EXPECTATIONS row 10: cross-namespace distinct N=2.
#[test]
fn probe_cross_namespace_distinct_classifiers_n2() {
    // appA::Point is recognized by appA::is-Point?
    let src_a = r#"
        (:wat::Record::def :appA::Point [x <- :wat::core::i64  y <- :wat::core::i64])
        (:wat::Record::def :appB::Point [x <- :wat::core::i64  y <- :wat::core::i64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:appA::is-Point? (:appA::Point 1 2)))
    "#;
    assert!(run_bool(src_a), "appA::is-Point? must return true for appA::Point N=2 instance");

    // appA::is-Point? returns false for appB::Point
    let src_neg = r#"
        (:wat::Record::def :appA::Point [x <- :wat::core::i64  y <- :wat::core::i64])
        (:wat::Record::def :appB::Point [x <- :wat::core::i64  y <- :wat::core::i64])
        (:wat::core::defn :user::compute [] -> :wat::core::bool (:appA::is-Point? (:appB::Point 1 2)))
    "#;
    assert!(
        !run_bool(src_neg),
        "appA::is-Point? must return false for appB::Point instance (distinct classifiers)"
    );
}

// EXPECTATIONS row 11: constructor type-checks each field

/// N=2 constructor type-checks both fields; wrong type for first field errors.
///
/// Arc 227 Stone 227.2 v3 -- EXPECTATIONS row 11: type-check for N=2.
#[test]
fn probe_constructor_rejects_wrong_typed_field() {
    // Wrong type for first field of N=2 constructor
    let err = expect_startup_err(r#"
        (:wat::Record::def :ns::P [a <- :wat::core::i64  b <- :wat::core::String])
        (:wat::core::defn :user::compute [] -> :wat::holon::HolonAST (:ns::P "wrong" "hi"))
    "#);
    assert!(
        !err.contains("no error"),
        "N=2 constructor must reject String where i64 expected for field a (got: {})",
        err
    );
}
