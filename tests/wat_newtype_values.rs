//! Arc 049 — newtype value support. End-to-end coverage of:
//! - Constructor `:Type/new(value)` round-trip
//! - Accessor `:Type/0(self)` returns the inner value
//! - Nominal distinction enforced by the type checker
//!   (cannot mix newtype with its inner type)
//! - Newtype as a struct field round-trip
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main/stdout-capture to
//! eval_in_frozen. Check-error tests use :my::probe + canonical nil main.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Arc 170 slice 1f-ζ: append canonical nil-returning `:user::main`.
fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    let ast = wat::parse_one!("(:my::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run")
}

fn run_expecting_check_error(src: &str) -> String {
    let err = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect_err("startup should fail with check error");
    format!("{:?}", err)
}

// ─── Construct + access round-trip ────────────────────────────────────

#[test]
fn newtype_construct_and_accessor_roundtrip() {
    // Arc 170 slice 1f-ζ: returns String via :my::compute.
    let src = r##"
        (:wat::core::newtype :my::trading::Price :wat::core::f64)

        (:wat::core::define (:my::compute -> :wat::core::String)
          (:wat::core::let
            [p (:my::trading::Price/new 100.0)
             inner (:my::trading::Price/0 p)]
            (:wat::core::f64::to-string inner)))
    "##;
    match run(src) {
        Value::String(s) => assert_eq!(&*s, "100", "expected '100'; got {}", s),
        other => panic!("expected String; got {:?}", other),
    }
}

// ─── Nominal distinction in argument position ─────────────────────────

#[test]
fn newtype_rejects_inner_type_at_arg_position() {
    // Arc 170 slice 1f-ζ: bad call in :my::probe + canonical nil main.
    let src = r##"
        (:wat::core::newtype :my::trading::Price :wat::core::f64)

        (:wat::core::define (:my::trading::pretty (p :my::trading::Price) -> :wat::core::String)
          (:wat::core::f64::to-string (:my::trading::Price/0 p)))

        (:wat::core::define (:my::probe -> :wat::core::String)
          (:my::trading::pretty 100.0))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "##;
    let err = run_expecting_check_error(src);
    assert!(
        err.contains("Price") || err.to_lowercase().contains("type"),
        "expected type-mismatch diagnostic mentioning Price; got: {}",
        err
    );
}

// ─── Inverse: newtype rejected where inner expected ───────────────────

#[test]
fn newtype_rejected_where_inner_expected() {
    // Arc 170 slice 1f-ζ: bad call in :my::probe + canonical nil main.
    let src = r##"
        (:wat::core::newtype :my::trading::Price :wat::core::f64)

        (:wat::core::define (:my::probe -> :wat::core::String)
          ;; Pass a Price where an f64 is expected — type-checker should refuse.
          (:wat::core::let
            [p (:my::trading::Price/new 100.0)
             bogus (:wat::core::f64::+'2 p 1.0)]
            (:wat::core::f64::to-string bogus)))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "##;
    let err = run_expecting_check_error(src);
    assert!(
        err.contains("Price")
            || err.contains("f64")
            || err.to_lowercase().contains("type"),
        "expected type-mismatch diagnostic for Price/f64 mix; got: {}",
        err
    );
}

// ─── Newtype as struct field round-trip ────────────────────────────────

#[test]
fn newtype_as_struct_field_roundtrip() {
    // Arc 170 slice 1f-ζ: returns String via :my::compute.
    let src = r##"
        (:wat::core::newtype :my::trading::Price :wat::core::f64)

        (:wat::core::struct :my::Order
          (label :wat::core::String)
          (price :my::trading::Price)
          (qty   :wat::core::i64))

        (:wat::core::define (:my::compute -> :wat::core::String)
          (:wat::core::let
            [p (:my::trading::Price/new 99.5)
             o          (:my::Order/new "BTC" p 7)
             retrieved (:my::Order/price o)
             inner (:my::trading::Price/0 retrieved)]
            (:wat::core::f64::to-string inner)))
    "##;
    match run(src) {
        Value::String(s) => assert_eq!(&*s, "99.5", "expected '99.5'; got {}", s),
        other => panic!("expected String; got {:?}", other),
    }
}

// ─── Two distinct newtypes over the same inner stay distinct ──────────

#[test]
fn distinct_newtypes_over_same_inner_are_distinct_types() {
    // Arc 170 slice 1f-ζ: bad call in :my::probe + canonical nil main.
    let src = r##"
        (:wat::core::newtype :my::trading::Price :wat::core::f64)
        (:wat::core::newtype :my::trading::Amount :wat::core::f64)

        (:wat::core::define (:my::trading::price-pretty (p :my::trading::Price) -> :wat::core::String)
          (:wat::core::f64::to-string (:my::trading::Price/0 p)))

        (:wat::core::define (:my::probe -> :wat::core::String)
          ;; Pass an Amount where Price is expected — must fail.
          (:wat::core::let
            [a (:my::trading::Amount/new 50.0)]
            (:my::trading::price-pretty a)))

        (:wat::core::define (:user::main -> :wat::core::nil)
          :wat::core::nil)
    "##;
    let err = run_expecting_check_error(src);
    assert!(
        err.contains("Price")
            || err.contains("Amount")
            || err.to_lowercase().contains("type"),
        "expected type-mismatch diagnostic Price vs Amount; got: {}",
        err
    );
}
