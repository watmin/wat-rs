//! Arc 278 — `(:wat::runtime::return-type-of <fn>) -> :wat::core::String`: the static sibling of
//! `(:wat::core::type <value>)`. Returns a fn's DECLARED return-type FQDN, colon-free (same convention as
//! `type`), so the two are directly comparable. Built to let rete `query` resolve a type-constructor's type
//! in one step (the type's PRIME `:T'` evaluates to its constructor fn whose ret_type IS the record type
//! — arc 294 item 9a moved the positional ctor off the bare type name onto the prime).
//!
//! Arc 278 query (a) — de-masking follow-up: `eval_return_type_of`'s keyword branch (src/runtime.rs)
//! used to ECHO an unresolved keyword's colon-stripped text back as if it were the answer, silently
//! masking unknown-type typos. `return_type_of_unknown_type_raises_not_echoes` pins the fix: reaching
//! that branch is now a raise, never an echo.
//!
//! Run: cargo test --release -p wat --test probe_arc278_return_type_of -- --include-ignored

use std::sync::Arc;
use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeErrorKind, Value};

fn call(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).unwrap_or_else(|e| panic!("eval raised: {e:?}"))
}

#[test]
fn return_type_of_a_record_constructor_is_the_record_fqdn() {
    // The type's PRIME (`:weather::ColdAndWindy'`) evaluates to its constructor fn; its
    // declared return type is the record type.
    assert_eq!(call(":user::return-type-of-ctor"),
        Value::String(Arc::new("weather::ColdAndWindy".to_string())),
        "constructor's return type = the record FQDN, colon-free");
}

#[test]
fn matches_type_of_a_constructed_value() {
    // The whole point: return-type-of(constructor) == type(an instance of that record).
    let from_ctor = call(":user::return-type-of-ctor");
    let from_value = call(":user::type-of-instance");
    assert_eq!(from_ctor, from_value,
        "return-type-of (static) agrees with type (dynamic): {from_ctor:?} vs {from_value:?}");
}

#[test]
fn return_type_of_an_inline_fn_is_its_declared_ret() {
    assert_eq!(call(":user::return-type-of-inline-fn"),
        Value::String(Arc::new("wat::core::bool".to_string())),
        "inline fn → its declared return type FQDN");
}

#[test]
fn return_type_of_unknown_type_raises_not_echoes() {
    // Arc 278 query (a) — the masking site this strike kills: `eval_return_type_of`'s
    // keyword branch used to echo the colon-stripped keyword text back as the "answer" for
    // ANY unresolved keyword, including a typo'd/unregistered type. `:s::Nope'` is never
    // registered in this fixture, reached via a dynamically-built keyword (not a literal AST
    // node, so check.rs's compile-time prime-type validation does not intercept it) — it must
    // raise a RuntimeError naming the unknown type, never return `Ok("s::Nope'")`.
    match call_beside_value(file!(), ":user::return-type-of-unknown-raises") {
        Err(e) => {
            assert!(
                matches!(e.kind(), RuntimeErrorKind::MalformedForm { .. }),
                "expected RuntimeErrorKind::MalformedForm naming the unknown type; got {:?}", e.kind()
            );
            let msg = format!("{}", e.kind());
            assert_eq!(
                msg,
                "malformed :wat::runtime::return-type-of form: unknown type: `:s::Nope'` (return-type-of: no such registered type)",
                "the raised error must name the unknown type keyword; got: {msg}"
            );
        }
        Ok(v) => panic!(
            "return-type-of on an unregistered type must RAISE, not echo; got Ok({v:?})"
        ),
    }
}
