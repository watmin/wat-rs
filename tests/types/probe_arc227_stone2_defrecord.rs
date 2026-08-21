//! Arc 227 Stone 227.2 v3 + Stone 234.6 migration — User-defined types via `:wat::core::defrecord` (formerly `:wat::holon::defrecord`).
//!
//! v3 supersedes v2 (commit b4509cb). v2 shipped with STOP-5b deferred framing for
//! N>=2; v3 ships canonical defrecord for ALL N including N>=2 using the composition
//! proven by the diagnostic probes (commits c18fa6b + 72367f1).
//!
//! Wat source: tests/types/probe_arc227_stone2_defrecord.wat (loaded via startup_beside).
//! Negative startup tests use sibling .wat.bad fixtures.

use wat::freeze::{call_beside_value, startup_beside, startup_from_file, FrozenWorld};
use wat::runtime::{apply_function, Value};

fn run_bool(fn_name: &str) -> bool {
    match call_beside_value(file!(), fn_name).expect("eval should succeed") {
        Value::bool(b) => b,
        other => panic!("expected bool from {}; got {:?}", fn_name, other),
    }
}

fn expect_startup_err(bad_file: &str) -> String {
    startup_from_file(bad_file)
        .err()
        .map(|e| e.to_string())
        .unwrap_or_else(|| "no error (startup succeeded)".to_string())
}

/// Fetch + apply a zero-arg entry fn against an ALREADY-FROZEN `world` — for the tests below that
/// share ONE world across two calls (the two-part EXPECTATIONS-row probes), unlike `run_bool`'s
/// fresh-freeze-per-name shape.
fn eval_on(world: &FrozenWorld, fn_name: &str) -> Value {
    let func = world.symbols().get(fn_name).unwrap_or_else(|| panic!("no entry fn {fn_name:?}")).clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("eval {fn_name} raised: {e:?}"))
}

// Test 1: Single FQDN defrecord -- construct + predicate

#[test]
fn probe_defrecord_single_fqdn_positive() {
    assert!(
        run_bool(":user::t01"),
        "is-Voltage? must return true for a Voltage instance constructed by defrecord v2"
    );
}

#[test]
fn probe_defrecord_single_fqdn_negative() {
    assert!(
        !run_bool(":user::t02"),
        "is-Voltage? must return false for a Current instance (different class)"
    );
}

// Test 2: Cross-namespace independence

#[test]
fn probe_defrecord_cross_namespace_app_a_positive() {
    assert!(
        run_bool(":user::t03"),
        "appA::is-Voltage? must return true for appA::Voltage instance"
    );
}

#[test]
fn probe_defrecord_cross_namespace_discrimination() {
    assert!(
        !run_bool(":user::t04"),
        "appA::is-Voltage? must return false for an appB::Voltage instance"
    );
}

// Test 3: Multiple types in same namespace

#[test]
fn probe_defrecord_same_namespace_celsius_positive() {
    assert!(
        run_bool(":user::t05"),
        "is-Celsius? must return true for a Celsius instance"
    );
}

#[test]
fn probe_defrecord_same_namespace_cross_discrimination() {
    assert!(
        !run_bool(":user::t06"),
        "is-Kelvin? must return false for a Celsius instance"
    );
}

// Test 4: User type vs built-in type

#[test]
fn probe_defrecord_user_type_vs_builtin_user_positive() {
    assert!(
        run_bool(":user::t07"),
        "is-MyMap? must return true for a MyMap instance"
    );
}

#[test]
fn probe_defrecord_user_type_vs_builtin_not_map() {
    assert!(
        !run_bool(":user::t08"),
        "is-Other? must return false for a MyMap instance (user types discriminate by class)"
    );
}

// Test 5: Polymorphic is? with FQDN string

#[test]
fn probe_defrecord_polymorphic_is_fqdn_positive() {
    assert!(
        run_bool(":user::t09"),
        "is-Voltage? must return true for a Voltage instance (class membership via generated predicate)"
    );
}

#[test]
fn probe_defrecord_polymorphic_is_bare_basename_negative() {
    assert!(
        !run_bool(":user::t10"),
        "is-Voltage? must return false for a Current instance (class names are FQDN-qualified, not bare)"
    );
}

// Test 6: Constructor type-checked

#[test]
fn probe_defrecord_constructor_typed_rejects_wrong_type() {
    let err = expect_startup_err("tests/types/probe_arc227_stone2_defrecord_typed.wat.bad");
    wat::assert_edn_matches_file!(err, "probe_arc227_stone2_defrecord__probe_defrecord_constructor_typed_rejects_wrong_type.edn", "constructor field type mismatch + return-type fallout");
}

// Test 7: Multi-segment namespace

#[test]
fn probe_defrecord_multi_segment_namespace_positive() {
    assert!(
        run_bool(":user::t11"),
        "is-Sensor? must return true for a Sensor instance from a 3-level namespace"
    );
}

#[test]
fn probe_defrecord_multi_segment_polymorphic_is() {
    assert!(
        run_bool(":user::t12"),
        "is-Sensor? must return true for a Sensor instance in 3-level namespace"
    );
}

// Test 8: Predicate name shape

#[test]
fn probe_defrecord_predicate_name_shape() {
    assert!(
        run_bool(":user::t13"),
        "is-BasisPoint? must return true (checks predicate name shape)"
    );
}

// Test 9: i64 field

#[test]
fn probe_defrecord_i64_payload() {
    assert!(
        run_bool(":user::t14"),
        "is-Count? must return true for a Count instance with an i64 field"
    );
}

// Test 10: Cross-type discrimination

#[test]
fn probe_defrecord_cross_type_discrimination_kelvin_positive() {
    assert!(
        run_bool(":user::t15"),
        "is-Kelvin? must return true for a Kelvin instance"
    );
}

// Test 11: No :user::* insertion

#[test]
fn probe_defrecord_no_user_namespace_insertion() {
    assert!(
        run_bool(":user::t16"),
        ":test::is-Celsius? must be in :test:: namespace (not :user::*)"
    );
}

// Test 12: appB cross-namespace predicate

#[test]
fn probe_defrecord_cross_namespace_app_b_positive() {
    assert!(
        run_bool(":user::t17"),
        "appB::is-Voltage? must return true for appB::Voltage instance"
    );
}

// Test 13: Empty field-list [] mints zero-arg constructor (NEW v2)

#[test]
fn probe_defrecord_empty_field_list_zero_arg_constructor() {
    assert!(
        run_bool(":user::t18"),
        "is-Tag? must return true for a Tag instance constructed with zero-arg form"
    );
}

// Test 14: Zero-arg tagged unit: predicate true for instance (NEW v2)

#[test]
fn probe_defrecord_tagged_unit_predicate_true() {
    assert!(
        run_bool(":user::t19"),
        "is-Done? must return true for a Done instance (zero-field tagged unit)"
    );
}

// Test 15: Zero-arg tagged unit: predicate false for non-instance (NEW v2)

#[test]
fn probe_defrecord_tagged_unit_predicate_false_for_non_instance() {
    assert!(
        !run_bool(":user::t20"),
        "is-Done? must return false for a Pending instance (different class)"
    );
}

// Test 16: Single-field String constructor (NEW v2)

#[test]
fn probe_defrecord_single_field_string_constructor() {
    assert!(
        run_bool(":user::t21"),
        "is-Label? must return true for a Label instance with String field"
    );
}

// Test 17: Cross-namespace tags: same type name, distinct classifiers (NEW v2)

#[test]
fn probe_defrecord_cross_namespace_tags_distinct() {
    assert!(
        run_bool(":user::t22"),
        "nsA::is-Tag? must return true for nsA::Tag instance"
    );
}

// Test 18: Field type enforcement -- wrong type rejected (NEW v2)

#[test]
fn probe_defrecord_field_type_check_bool_rejected() {
    let err = expect_startup_err("tests/types/probe_arc227_stone2_defrecord_bool.wat.bad");
    wat::assert_edn_matches_file!(err, "probe_arc227_stone2_defrecord__probe_defrecord_field_type_check_bool_rejected.edn", "field type check bool rejected + return-type fallout");
}

// Test 19: Multi-segment namespace with field (NEW v2)

#[test]
fn probe_defrecord_multi_segment_with_field() {
    assert!(
        run_bool(":user::t23"),
        "is-Reading? must return true for a Reading instance from a 4-level namespace"
    );
}

// ─── v3 tests — Stone 227.2 v3 canonical instance shape + N>=2 fields ────────

// EXPECTATIONS row 1: single-arg form errors at expand time (HARD CUT preserved).
//
// Arc 109 binder strike β-i — the ERROR CLASS changed and the intent did not. `defrecord` became
// variadic (`[& args]`) so an optional `:- [T…]` binder can sit between the name and the field
// vector, which retires the fixed-arity macro signature that used to raise `ArityMismatch`. A
// one-arg form still fails AT EXPAND TIME, now as `ProgramBodyEvalFailed` wrapping the primitive
// that could not proceed. ⚠ The message names `:wat::core::rest`, an internal — a real loss of
// diagnosis quality, filed at `109/NOTE-a-macro-cannot-diagnose-with-option-expect.md` along with
// the reason `Option/expect` cannot fix it (it panics rather than producing an error value).

#[test]
fn probe_two_arg_form_only_one_arg_errors() {
    let err = expect_startup_err("tests/types/probe_arc227_stone2_defrecord_onearg.wat.bad");
    wat::assert_edn_matches_file!(err, "probe_arc227_stone2_defrecord__probe_two_arg_form_only_one_arg_errors.edn", "one-arg defrecord form: expand-time failure (ProgramBodyEvalFailed since arc 109 β-i)");
}

// EXPECTATIONS row 3: N=0 canonical instance shape uses Bundle (not Atom(nil))

#[test]
fn probe_zero_field_instance_uses_empty_bundle() {
    let world = startup_beside(file!()).expect("startup");

    // Part a: is? confirms classifier (t25a)
    match eval_on(&world, ":user::t25a") {
        Value::bool(b) => assert!(b, "N=0 instance must be recognized by predicate (classifier Bind correct)"),
        other => panic!("t25a: expected bool; got {:?}", other),
    }

    // Part b: empty Bundle has statement-length 0 (t25b)
    match eval_on(&world, ":user::t25b") {
        Value::i64(n) => assert_eq!(n, 0, "Bundle([]) has statement-length 0 — canonical empty inner for N=0"),
        other => panic!("t25b: expected i64; got {:?}", other),
    }
}

// EXPECTATIONS row 5: N=1 canonical instance shape uses Bundle(Bind(...))

#[test]
fn probe_one_field_instance_uses_bundle_with_one_bind() {
    let world = startup_beside(file!()).expect("startup");

    // Part a: predicate works (t27a)
    match eval_on(&world, ":user::t27a") {
        Value::bool(b) => assert!(b, "N=1 instance must be recognized by predicate"),
        other => panic!("t27a: expected bool; got {:?}", other),
    }

    // Part b: Bundle([one-item]) has statement-length 1 (t27b)
    match eval_on(&world, ":user::t27b") {
        Value::i64(n) => assert_eq!(n, 1, "Bundle([one-field-bind]) has statement-length 1 — canonical Bundle(Bind) inner for N=1"),
        other => panic!("t27b: expected i64; got {:?}", other),
    }
}

// EXPECTATIONS row 6: N=2 multi-field constructor takes 2 typed args

#[test]
fn probe_two_field_construct_with_typed_args() {
    assert!(
        run_bool(":user::t26"),
        "N=2 constructor (:ns::P 5 \"hi\") must succeed and is-P? must return true"
    );
}

// EXPECTATIONS row 7: N=2 canonical instance shape uses Bundle with 2 children

#[test]
fn probe_two_field_instance_bundle_has_two_binds() {
    let world = startup_beside(file!()).expect("startup");

    // Part a: predicate works for N=2 (t28a)
    match eval_on(&world, ":user::t28a") {
        Value::bool(b) => assert!(b, "N=2 instance is-P? must return true"),
        other => panic!("t28a: expected bool; got {:?}", other),
    }

    // Part b: Bundle([fa, fb]) has statement-length 2 (t28b)
    match eval_on(&world, ":user::t28b") {
        Value::i64(n) => assert_eq!(n, 2, "Bundle([field-a, field-b]) has statement-length 2 — canonical 2-child Bundle for N=2"),
        other => panic!("t28b: expected i64; got {:?}", other),
    }
}

// EXPECTATIONS row 8: N=3 multi-field constructor takes 3 typed args

#[test]
fn probe_three_field_construct_with_typed_args() {
    assert!(
        run_bool(":user::t29"),
        "N=3 constructor (:ns::T 7 \"world\" true) must succeed and is-T? must return true"
    );
}

#[test]
fn probe_three_field_instance_bundle_has_three_binds() {
    let world = startup_beside(file!()).expect("startup");

    // Part a: predicate works for N=3 (t30a)
    match eval_on(&world, ":user::t30a") {
        Value::bool(b) => assert!(b, "N=3 instance is-T? must return true"),
        other => panic!("t30a: expected bool; got {:?}", other),
    }

    // Part b: Bundle([fa, fb, fc]) has statement-length 3 (t30b)
    match eval_on(&world, ":user::t30b") {
        Value::i64(n) => assert_eq!(n, 3, "Bundle([fa, fb, fc]) has statement-length 3 — canonical 3-child Bundle for N=3"),
        other => panic!("t30b: expected i64; got {:?}", other),
    }
}

// EXPECTATIONS row 9: predicate works for all N

#[test]
fn probe_predicate_works_for_n0_n1_n2_n3() {
    assert!(run_bool(":user::t31-n0"), "N=0 predicate must work");
    assert!(run_bool(":user::t31-n1"), "N=1 predicate must work");
    assert!(run_bool(":user::t31-n2"), "N=2 predicate must work");
    assert!(run_bool(":user::t31-n3"), "N=3 predicate must work");
    assert!(
        !run_bool(":user::t31-neg"),
        "N=2 predicate must return false for instance of different type"
    );
}

// EXPECTATIONS row 10: cross-namespace independence with N>=2

#[test]
fn probe_cross_namespace_distinct_classifiers_n2() {
    assert!(
        run_bool(":user::t32a"),
        "appA::is-Point? must return true for appA::Point N=2 instance"
    );
    assert!(
        !run_bool(":user::t32neg"),
        "appA::is-Point? must return false for appB::Point instance (distinct classifiers)"
    );
}

// EXPECTATIONS row 11: constructor type-checks each field

#[test]
fn probe_constructor_rejects_wrong_typed_field() {
    let err = expect_startup_err("tests/types/probe_arc227_stone2_defrecord_wrongfield.wat.bad");
    wat::assert_edn_matches_file!(err, "probe_arc227_stone2_defrecord__probe_constructor_rejects_wrong_typed_field.edn", "constructor rejects wrong-typed field a (String for declared i64) + return-type fallout");
}
