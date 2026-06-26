//! Arc 278 — `(:wat::runtime::return-type-of <fn>) -> :wat::core::String`: the static sibling of
//! `(:wat::core::type <value>)`. Returns a fn's DECLARED return-type FQDN, colon-free (same convention as
//! `type`), so the two are directly comparable. Built to let rete `query` resolve a type-constructor's type
//! in one step (a bare type name evaluates to its constructor fn whose ret_type IS the record type).
//!
//! Run: cargo test --release -p wat --test probe_arc278_return_type_of -- --include-ignored

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

const WORLD: &str = "\
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])\n\
(:wat::core::defn :user::main [] -> :wat::core::nil nil)";

fn ev(expr: &str) -> Value {
    let world = startup_from_source(WORLD, None, Arc::new(InMemoryLoader::new())).expect("startup");
    let ast = wat::parse_one!(expr).expect("parse");
    eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
        .value_owned()
}

#[test]
fn return_type_of_a_record_constructor_is_the_record_fqdn() {
    // A bare type name evaluates to its constructor fn; its declared return type is the record type.
    assert_eq!(ev("(:wat::runtime::return-type-of :weather::ColdAndWindy)"),
        Value::String(Arc::new("weather::ColdAndWindy".to_string())),
        "constructor's return type = the record FQDN, colon-free");
}

#[test]
fn matches_type_of_a_constructed_value() {
    // The whole point: return-type-of(constructor) == type(an instance of that record).
    let from_ctor = ev("(:wat::runtime::return-type-of :weather::ColdAndWindy)");
    let from_value = ev("(:wat::core::type (:weather::ColdAndWindy \"Oslo\"))");
    assert_eq!(from_ctor, from_value,
        "return-type-of (static) agrees with type (dynamic): {from_ctor:?} vs {from_value:?}");
}

#[test]
fn return_type_of_an_inline_fn_is_its_declared_ret() {
    assert_eq!(ev("(:wat::runtime::return-type-of (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool true))"),
        Value::String(Arc::new("wat::core::bool".to_string())),
        "inline fn → its declared return type FQDN");
}
