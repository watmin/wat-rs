//! A consumer declared at the checker's stated type, then run.
//! `compose-variant` is `wat.type/symbol`. `variant-parent-of` is `Option` of
//! `wat.type/symbol`. `TypeInfo.name` is `wat.type/symbol`.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

#[test]
fn compose_variant_result_matches_the_stated_type() {
    let v = call_beside_value(file!(), ":user::consume-compose").expect("compose-variant consumer");
    assert!(matches!(v, Value::Symbol(_)), "compose-variant consumer returned {v:?}");
}

#[test]
fn variant_parent_matches_the_stated_type() {
    let v = call_beside_value(file!(), ":user::consume-parent").expect("variant-parent consumer");
    assert!(matches!(v, Value::Symbol(_)), "variant-parent consumer returned {v:?}");
}

#[test]
fn type_info_name_matches_the_stated_field_type() {
    let v = call_beside_value(file!(), ":user::consume-type-name").expect("TypeInfo.name consumer");
    assert!(matches!(v, Value::Symbol(_)), "TypeInfo.name consumer returned {v:?}");
}
