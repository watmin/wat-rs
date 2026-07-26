//! FM-9 guard — arc 237 Stone S-C.2ab: field-NAME→INDEX order via RecordDef.field_names.
//!
//! S-C.2ab re-routed name-based access (keyword-accessor, assoc, record->map) off
//! holon_form onto `RecordDef.field_names`. The macro emits the names; the access sites
//! look them up by `.position(...)`. With a SINGLE-field record this is trivially correct
//! (one name, order can't be wrong) — so the existing keyword-access/assoc probes do NOT
//! prove the macro emits names in declaration order. This guard uses a MULTI-field record
//! and accesses the SECOND field by NAME: if field_names order is wrong (e.g. reversed),
//! `(:y p)` returns x's value and the test fails.
//!
//! All contracts must PASS post-S-C.2ab.

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn eval_f64(fn_name: &str) -> f64 {
    match call_beside_value(file!(), fn_name).expect("eval") {
        Value::f64(x) => x,
        other => panic!("expected f64 from {}; got {:?}", fn_name, other),
    }
}

// keyword-accessor of the FIRST field (sanity).
#[test]
fn first_field_by_keyword() {
    assert!((eval_f64(":user::first-field-by-keyword") - 1.0).abs() < 1e-9);
}

// keyword-accessor of the SECOND field — the name-ORDER guard. Wrong field_names order
// would return 1.0 here.
#[test]
fn second_field_by_keyword() {
    assert!((eval_f64(":user::second-field-by-keyword") - 2.0).abs() < 1e-9);
}

// generated positional accessor of the second field (struct path; should agree).
#[test]
fn second_field_by_accessor() {
    assert!((eval_f64(":user::second-field-by-accessor") - 2.0).abs() < 1e-9);
}

// assoc the SECOND field by NAME, then read it back — proves assoc's name→index order.
#[test]
fn assoc_second_field_by_name() {
    assert!((eval_f64(":user::assoc-second-field-by-name") - 9.0).abs() < 1e-9);
}

// assoc the second field must NOT disturb the first (parity / correct index).
#[test]
fn assoc_second_leaves_first() {
    assert!((eval_f64(":user::assoc-second-leaves-first") - 1.0).abs() < 1e-9);
}
