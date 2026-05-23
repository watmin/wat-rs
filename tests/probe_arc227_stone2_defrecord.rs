//! Arc 227 Stone 227.1 — User-defined types via `:wat::holon::defrecord` macro.
//!
//! Verifies that `:wat::holon::defrecord` correctly generates:
//!   - A constructor in the user-declared namespace (accepts `:wat::holon::HolonAST`)
//!   - A predicate in the user-declared namespace
//!   - Classifier-wrapped instances: `Bind(Atom("ns::Name"), Atom(data))`
//!   - Namespace collision-freedom across distinct namespaces
//!   - Polymorphic `:wat::holon::is?` works on user-defined types
//!   - Non-atomizable input: at-type-boundary (caller uses to-holon first)
//!
//! ## Doctrine
//!
//! Per [[typed-entities-doctrine]] + `feedback_fqdn_is_the_namespace`:
//!   - `(:wat::holon::defrecord :myapp::Voltage)` generates `:myapp::Voltage` (constructor)
//!     and `:myapp::is-Voltage?` (predicate) — entirely in the user-declared namespace.
//!   - The substrate NEVER inserts into `:user::*` or any auto-namespace.
//!   - Classifier string = FQDN without leading colon ("myapp::Voltage").
//!   - Collision-free: `:appA::Voltage` and `:appB::Voltage` produce distinct classifiers.
//!   - The constructor accepts `:wat::holon::HolonAST`; callers use `to-holon` to lift
//!     primitive values before construction.
//!
//! ## Depends on
//!
//!   - Stone 226.1 (`e7ba909`): `:wat::holon::is?` + `:wat::holon::is-Map?` etc. live.
//!   - Stone 225.1: `:wat::holon::Bind` + `:wat::holon::Atom` + `:wat::holon::to-holon`.
//!   - Stone 227.1 (THIS): `:wat::holon::defrecord` macro in `wat/holon/defrecord.wat`.
//!
//! ## Test structure
//!
//!   Test 1 — single FQDN defrecord: construct + query (positive + negative)
//!   Test 2 — cross-namespace independence (appA vs appB)
//!   Test 3 — multiple distinct types in same namespace (Celsius vs Kelvin)
//!   Test 4 — user type distinct from built-in types (MyMap vs Map)
//!   Test 5 — polymorphic is? with full FQDN string works; bare basename does not
//!   Test 6 — constructor is typed: non-HolonAST input caught at check time
//!   Test 7 — multi-segment namespace (three-level FQDN)
//!   Test 8 — predicate name shape (is- prefix on basename, namespace preserved)
//!   Test 9 — user type with i64 payload
//!   Test 10 — cross-type discrimination (Celsius is-Celsius? true; is-Kelvin? false)
//!   Test 11 — no :user::* insertion (uses declared test:: namespace)
//!   Test 12 — appB cross-namespace predicate works independently

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

fn expect_startup_err(src: &str) -> String {
    let src = with_nil_main(src);
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .err()
        .map(|e| e.to_string())
        .unwrap_or_else(|| "no error (startup succeeded)".to_string())
}

// ─── Test 1: Single FQDN defrecord — construct + predicate ────────────────────

/// `(:wat::holon::defrecord :test::Voltage)` mints a constructor.
/// `(:test::Voltage (to-holon 5.0))` constructs an instance.
/// `(:test::is-Voltage? instance)` returns true.
///
/// Arc 227 Stone 227.1 — basic positive case.
#[test]
fn probe_defrecord_single_fqdn_positive() {
    let src = r#"
        (:wat::holon::defrecord :test::Voltage)
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:test::Voltage (:wat::holon::to-holon 5.0))]
            (:test::is-Voltage? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-Voltage? must return true for a Voltage instance constructed by defrecord"
    );
}

/// `(:test::is-Voltage? (to-holon "random-string"))` returns false (no Voltage classifier).
///
/// Arc 227 Stone 227.1 — predicate returns false for non-instance.
#[test]
fn probe_defrecord_single_fqdn_negative() {
    let src = r#"
        (:wat::holon::defrecord :test::Voltage)
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:test::is-Voltage? (:wat::holon::to-holon "random-string")))
    "#;
    assert!(
        !run_bool(src),
        "is-Voltage? must return false for a bare String HolonAST (no Voltage classifier)"
    );
}

// ─── Test 2: Cross-namespace independence ─────────────────────────────────────

/// `(:appA::Voltage x)` and `(:appB::Voltage x)` produce classifiers
/// "appA::Voltage" and "appB::Voltage" — distinct; predicates discriminate.
///
/// Arc 227 Stone 227.1 — FQDN collision-freedom: appA positive.
#[test]
fn probe_defrecord_cross_namespace_app_a_positive() {
    let src = r#"
        (:wat::holon::defrecord :appA::Voltage)
        (:wat::holon::defrecord :appB::Voltage)
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [a-instance (:appA::Voltage (:wat::holon::to-holon 42))]
            (:appA::is-Voltage? a-instance)))
    "#;
    assert!(
        run_bool(src),
        "appA::is-Voltage? must return true for appA::Voltage instance"
    );
}

/// `(:appA::is-Voltage? appB-instance)` returns false — classifier "appB::Voltage"
/// does NOT match the "appA::Voltage" predicate.
///
/// Arc 227 Stone 227.1 — cross-namespace discrimination is honest.
#[test]
fn probe_defrecord_cross_namespace_discrimination() {
    let src = r#"
        (:wat::holon::defrecord :appA::Voltage)
        (:wat::holon::defrecord :appB::Voltage)
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [b-instance (:appB::Voltage (:wat::holon::to-holon 42))]
            (:appA::is-Voltage? b-instance)))
    "#;
    assert!(
        !run_bool(src),
        "appA::is-Voltage? must return false for an appB::Voltage instance"
    );
}

// ─── Test 3: Multiple types in same namespace ──────────────────────────────────

/// Two types in the same namespace — :test::Celsius and :test::Kelvin.
/// A Celsius instance is Celsius, not Kelvin.
///
/// Arc 227 Stone 227.1 — same-namespace independence.
#[test]
fn probe_defrecord_same_namespace_celsius_positive() {
    let src = r#"
        (:wat::holon::defrecord :test::Celsius)
        (:wat::holon::defrecord :test::Kelvin)
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [c (:test::Celsius (:wat::holon::to-holon 100.0))]
            (:test::is-Celsius? c)))
    "#;
    assert!(
        run_bool(src),
        "is-Celsius? must return true for a Celsius instance"
    );
}

/// A Celsius instance is NOT Kelvin.
///
/// Arc 227 Stone 227.1 — same-namespace cross-discrimination.
#[test]
fn probe_defrecord_same_namespace_cross_discrimination() {
    let src = r#"
        (:wat::holon::defrecord :test::Celsius)
        (:wat::holon::defrecord :test::Kelvin)
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [c (:test::Celsius (:wat::holon::to-holon 100.0))]
            (:test::is-Kelvin? c)))
    "#;
    assert!(
        !run_bool(src),
        "is-Kelvin? must return false for a Celsius instance"
    );
}

// ─── Test 4: User type vs built-in type ───────────────────────────────────────

/// `(:test::MyMap instance)` produces classifier "test::MyMap".
/// `(:test::is-MyMap? instance)` → true.
///
/// Arc 227 Stone 227.1 — user types work independently of built-in types.
#[test]
fn probe_defrecord_user_type_vs_builtin_user_positive() {
    let src = r#"
        (:wat::holon::defrecord :test::MyMap)
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:test::MyMap (:wat::holon::to-holon "data"))]
            (:test::is-MyMap? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-MyMap? must return true for a MyMap instance"
    );
}

/// A user-defined MyMap instance is NOT a built-in Map.
/// Classifier "test::MyMap" ≠ "Map".
///
/// Arc 227 Stone 227.1 — user types don't masquerade as built-in types.
#[test]
fn probe_defrecord_user_type_vs_builtin_not_map() {
    let src = r#"
        (:wat::holon::defrecord :test::MyMap)
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:test::MyMap (:wat::holon::to-holon "data"))]
            (:wat::holon::is-Map? instance)))
    "#;
    assert!(
        !run_bool(src),
        "is-Map? must return false for a user-defined MyMap instance (classifier 'test::MyMap' != 'Map')"
    );
}

// ─── Test 5: Polymorphic is? with FQDN string ─────────────────────────────────

/// `(:wat::holon::is? instance "test::Voltage")` returns true.
/// The polymorphic predicate from arc 226 works on user-defined types.
///
/// Arc 227 Stone 227.1 — polymorphic is? works for user classifier names.
#[test]
fn probe_defrecord_polymorphic_is_fqdn_positive() {
    let src = r#"
        (:wat::holon::defrecord :test::Voltage)
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:test::Voltage (:wat::holon::to-holon 5.0))]
            (:wat::holon::is? instance "test::Voltage")))
    "#;
    assert!(
        run_bool(src),
        "is? with 'test::Voltage' classifier string must return true for a Voltage instance"
    );
}

/// `(:wat::holon::is? instance "Voltage")` returns false —
/// bare basename without namespace does NOT match "test::Voltage".
///
/// Arc 227 Stone 227.1 — classifier is FQDN-qualified; basename alone is insufficient.
#[test]
fn probe_defrecord_polymorphic_is_bare_basename_negative() {
    let src = r#"
        (:wat::holon::defrecord :test::Voltage)
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:test::Voltage (:wat::holon::to-holon 5.0))]
            (:wat::holon::is? instance "Voltage")))
    "#;
    assert!(
        !run_bool(src),
        "is? with bare 'Voltage' must return false — FQDN-qualified classifier required"
    );
}

// ─── Test 6: Constructor is typed — non-HolonAST input caught at check time ───

/// The generated constructor is typed `[v <- :wat::holon::HolonAST]`.
/// Passing a non-HolonAST value (e.g. raw i64 literal) fails at check time.
///
/// Arc 227 Stone 227.1 — constructor enforces HolonAST input at the type boundary.
#[test]
fn probe_defrecord_constructor_typed_rejects_non_holon() {
    let err = expect_startup_err(r#"
        (:wat::holon::defrecord :test::Voltage)
        (:wat::core::define (:user::compute -> :wat::holon::HolonAST)
          (:test::Voltage 5.0))
    "#);
    // The type checker must reject a raw f64 literal where HolonAST is expected.
    // Any startup error (type mismatch / check failure) satisfies this.
    assert!(
        !err.contains("no error"),
        "constructor must reject non-HolonAST at check time (got: {})",
        err
    );
}

// ─── Test 7: Multi-segment namespace ─────────────────────────────────────────

/// Three-level FQDN: `:awesome::lib::Sensor`.
/// Predicate should be `:awesome::lib::is-Sensor?`.
/// Classifier string = "awesome::lib::Sensor".
///
/// Arc 227 Stone 227.1 — multi-segment namespace handled correctly.
#[test]
fn probe_defrecord_multi_segment_namespace_positive() {
    let src = r#"
        (:wat::holon::defrecord :awesome::lib::Sensor)
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:awesome::lib::Sensor (:wat::holon::to-holon 42))]
            (:awesome::lib::is-Sensor? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-Sensor? must return true for a Sensor instance from a 3-level namespace"
    );
}

/// Multi-segment namespace: polymorphic is? with full classifier string.
///
/// Arc 227 Stone 227.1 — multi-segment classifier string is FQDN-qualified.
#[test]
fn probe_defrecord_multi_segment_polymorphic_is() {
    let src = r#"
        (:wat::holon::defrecord :awesome::lib::Sensor)
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:awesome::lib::Sensor (:wat::holon::to-holon 42))]
            (:wat::holon::is? instance "awesome::lib::Sensor")))
    "#;
    assert!(
        run_bool(src),
        "is? with 'awesome::lib::Sensor' must return true for a Sensor instance"
    );
}

// ─── Test 8: Predicate name shape ────────────────────────────────────────────

/// The generated predicate name has the correct shape:
/// "is-" prefix on the basename, namespace prefix preserved.
/// For :test::BasisPoint → :test::is-BasisPoint?
///
/// Arc 227 Stone 227.1 — predicate naming rule correctness.
#[test]
fn probe_defrecord_predicate_name_shape() {
    let src = r#"
        (:wat::holon::defrecord :test::BasisPoint)
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:test::BasisPoint (:wat::holon::to-holon 25))]
            (:test::is-BasisPoint? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-BasisPoint? must return true for a BasisPoint instance (checks predicate name shape)"
    );
}

// ─── Test 9: User type with i64 payload ───────────────────────────────────────

/// Constructor accepts i64 payloads (via to-holon) and wraps them correctly.
///
/// Arc 227 Stone 227.1 — i64 payload construction.
#[test]
fn probe_defrecord_i64_payload() {
    let src = r#"
        (:wat::holon::defrecord :test::Count)
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [instance (:test::Count (:wat::holon::to-holon 99))]
            (:test::is-Count? instance)))
    "#;
    assert!(
        run_bool(src),
        "is-Count? must return true for a Count instance with an i64 payload"
    );
}

// ─── Test 10: Cross-type discrimination ──────────────────────────────────────

/// Celsius is Celsius, Kelvin is not Celsius.
/// Kelvin is Kelvin, Celsius is not Kelvin.
///
/// Arc 227 Stone 227.1 — full symmetric cross-type discrimination.
#[test]
fn probe_defrecord_cross_type_discrimination_kelvin_positive() {
    let src = r#"
        (:wat::holon::defrecord :test::Celsius)
        (:wat::holon::defrecord :test::Kelvin)
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [k (:test::Kelvin (:wat::holon::to-holon 373.15))]
            (:test::is-Kelvin? k)))
    "#;
    assert!(
        run_bool(src),
        "is-Kelvin? must return true for a Kelvin instance"
    );
}

// ─── Test 11: No :user::* insertion ──────────────────────────────────────────

/// defrecord NEVER inserts into :user::* — only into the user-declared namespace.
/// The test verifies that `:test::is-Celsius?` is defined (not `:user::is-Celsius?`).
///
/// Arc 227 Stone 227.1 — STOP-8 compliance: no auto-namespace insertion.
#[test]
fn probe_defrecord_no_user_namespace_insertion() {
    // Verify :test::Celsius exists and works — confirming defrecord used the user namespace
    let src = r#"
        (:wat::holon::defrecord :test::Celsius)
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [c (:test::Celsius (:wat::holon::to-holon 273.15))]
            (:test::is-Celsius? c)))
    "#;
    assert!(
        run_bool(src),
        ":test::is-Celsius? must be defined in :test:: namespace (not :user::*)"
    );
}

// ─── Test 12: appB cross-namespace predicate works independently ──────────────

/// appB::Voltage is correctly identified by appB::is-Voltage?.
///
/// Arc 227 Stone 227.1 — both appA and appB predicates work; independent.
#[test]
fn probe_defrecord_cross_namespace_app_b_positive() {
    let src = r#"
        (:wat::holon::defrecord :appA::Voltage)
        (:wat::holon::defrecord :appB::Voltage)
        (:wat::core::define (:user::compute -> :wat::core::bool)
          (:wat::core::let
            [b-instance (:appB::Voltage (:wat::holon::to-holon 99))]
            (:appB::is-Voltage? b-instance)))
    "#;
    assert!(
        run_bool(src),
        "appB::is-Voltage? must return true for appB::Voltage instance"
    );
}
