//! Arc 049 — newtype value support. End-to-end coverage of:
//! - Constructor `:Type/new(value)` round-trip
//! - Accessor `:Type/0(self)` returns the inner value
//! - Nominal distinction enforced by the type checker
//!   (cannot mix newtype with its inner type)
//! - Newtype as a struct field round-trip
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main/stdout-capture to
//! eval_in_frozen. Check-error tests use :my::probe + canonical nil main.

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

fn run(path: &str) -> Value {
    let world = startup_from_file(path).expect("startup");
    let func = world.symbols().get(":my::compute").expect(":my::compute").clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()).expect("compute should run")
}

fn run_expecting_check_error(path: &str) -> String {
    let err = startup_from_file(path).expect_err("startup should fail with check error");
    format!("{:?}", err)
}

// ─── Construct + access round-trip ────────────────────────────────────

#[test]
fn newtype_construct_and_accessor_roundtrip() {
    match run("tests/types/newtype_construct_and_accessor_roundtrip.wat") {
        Value::String(s) => assert_eq!(&*s, "100", "expected '100'; got {}", s),
        other => panic!("expected String; got {:?}", other),
    }
}

// ─── Nominal distinction in argument position ─────────────────────────

#[test]
fn newtype_rejects_inner_type_at_arg_position() {
    let err = run_expecting_check_error("tests/types/newtype_rejects_inner_type.wat.bad");
    wat::assert_edn_matches_file!(err, "newtype__newtype_rejects_inner_type_at_arg_position.edn", "newtype rejected in arg position: TypeMismatch");
}

// ─── Inverse: newtype rejected where inner expected ───────────────────

#[test]
fn newtype_rejected_where_inner_expected() {
    let err = run_expecting_check_error("tests/types/newtype_rejected_where_inner_expected.wat.bad");
    wat::assert_edn_matches_file!(err, "newtype__newtype_rejected_where_inner_expected.edn", "inverse: newtype rejected where inner type expected");
}

// ─── Newtype as struct field round-trip ────────────────────────────────

#[test]
fn newtype_as_struct_field_roundtrip() {
    match run("tests/types/newtype_as_struct_field_roundtrip.wat") {
        Value::String(s) => assert_eq!(&*s, "99.5", "expected '99.5'; got {}", s),
        other => panic!("expected String; got {:?}", other),
    }
}

// ─── Two distinct newtypes over the same inner stay distinct ──────────

#[test]
fn distinct_newtypes_over_same_inner_are_distinct_types() {
    let err = run_expecting_check_error("tests/types/newtype_distinct_newtypes.wat.bad");
    wat::assert_edn_matches_file!(err, "newtype__distinct_newtypes_over_same_inner_are_distinct_types.edn", "two distinct newtypes over the same inner stay distinct: TypeMismatch");
}
