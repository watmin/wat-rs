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

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const PRELUDE: &str = "(:wat::Record::def :my::Pt [x <- :wat::core::f64  y <- :wat::core::f64])\n";

fn eval_f64(compute_expr: &str) -> f64 {
    let full = format!(
        "{prelude}\
         (:wat::core::defn :user::compute [] -> :wat::core::f64 {expr})\n\
         (:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        prelude = PRELUDE,
        expr = compute_expr
    );
    let world = startup_from_source(&full, None, Arc::new(InMemoryLoader::new()))
        .unwrap_or_else(|e| panic!("startup failed for `{}`: {:?}", compute_expr, e));
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).map(|tv| tv.value_owned()) {
        Ok(Value::f64(x)) => x,
        other => panic!("expected f64 for `{}`; got {:?}", compute_expr, other),
    }
}

// keyword-accessor of the FIRST field (sanity).
#[test]
fn first_field_by_keyword() {
    assert!((eval_f64("(:x (:my::Pt 1.0 2.0))") - 1.0).abs() < 1e-9);
}

// keyword-accessor of the SECOND field — the name-ORDER guard. Wrong field_names order
// would return 1.0 here.
#[test]
fn second_field_by_keyword() {
    assert!((eval_f64("(:y (:my::Pt 1.0 2.0))") - 2.0).abs() < 1e-9);
}

// generated positional accessor of the second field (struct path; should agree).
#[test]
fn second_field_by_accessor() {
    assert!((eval_f64("(:my::Pt/y (:my::Pt 1.0 2.0))") - 2.0).abs() < 1e-9);
}

// assoc the SECOND field by NAME, then read it back — proves assoc's name→index order.
#[test]
fn assoc_second_field_by_name() {
    assert!((eval_f64("(:y (:wat::Record/assoc (:my::Pt 1.0 2.0) :y 9.0))") - 9.0).abs() < 1e-9);
}

// assoc the second field must NOT disturb the first (parity / correct index).
#[test]
fn assoc_second_leaves_first() {
    assert!((eval_f64("(:x (:wat::Record/assoc (:my::Pt 1.0 2.0) :y 9.0))") - 1.0).abs() < 1e-9);
}
