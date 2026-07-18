//! RED probe — arc 294.c.2a: `aggregate-new` is the ONE nature-dispatched constructor.
//!
//! 294 DESIGN:128 — construction collapses to a single varargs primitive
//! `(:wat::core::aggregate-new :T field…)` that reads `:T`'s nature from the TypeEnv and
//! builds the right `AggregateValue`. For a HolonRecord it derives the hologram INTERNALLY
//! (the `build_holon_hologram` Rust helper, routed through the one extracted
//! `bundle_with_capacity` guard) — no precomputed-form arg. `struct-new` / `Record::of` /
//! `holon::Record::of` die into it (the deaths land in 294.c.2b).
//!
//! All three macros (`defstruct` / `defrecord` / `holon::defrecord`) + the struct ctor
//! codegen emit `(aggregate-new :T …)`; the `defholon` macro's hologram quasiquote dies.
//!
//! RED at HEAD: `:wat::core::aggregate-new` is an unknown function → the fixture's defn
//! bodies don't resolve → `startup_beside` errors (or the eval errors). GREEN after c.2a:
//! all three natures construct via `aggregate-new`, field accessors read back, and the
//! holon record's DERIVED hologram measures (`cosine h h == 1.0`).

use wat::freeze::call_beside;
use wat::runtime::Value;

/// Struct constructed via `aggregate-new`; field `a` reads back 7.
#[test]
fn aggregate_new_constructs_struct() {
    let got = call_beside(file!(), ":user::an-struct-a").expect("eval an-struct-a");
    match got {
        Value::i64(7) => {}
        other => panic!("aggregate-new struct: expected i64(7), got {:?}", other),
    }
}

/// Base record constructed via `aggregate-new`; field `b` reads back 8.
#[test]
fn aggregate_new_constructs_base_record() {
    let got = call_beside(file!(), ":user::an-record-b").expect("eval an-record-b");
    match got {
        Value::i64(8) => {}
        other => panic!("aggregate-new base record: expected i64(8), got {:?}", other),
    }
}

/// Holon record constructed via `aggregate-new`; field `a` reads back 7 (hologram derived).
#[test]
fn aggregate_new_constructs_holon_record() {
    let got = call_beside(file!(), ":user::an-holon-a").expect("eval an-holon-a");
    match got {
        Value::i64(7) => {}
        other => panic!("aggregate-new holon record: expected i64(7), got {:?}", other),
    }
}

/// The holon record's DERIVED hologram is correct: `cosine h h == 1.0` (exact coincidence).
/// This is the load-bearing assertion — it proves the hologram was derived right by
/// `aggregate-new`, not merely that a value was constructed.
#[test]
fn aggregate_new_holon_hologram_is_derived_correctly() {
    let got = call_beside(file!(), ":user::an-holon-self-cos").expect("eval an-holon-self-cos");
    match got {
        Value::f64(c) => assert!(
            (c - 1.0).abs() < 1e-6,
            "aggregate-new holon hologram: cosine(h, h) must be 1.0 (derived correctly); got {}",
            c
        ),
        other => panic!("aggregate-new holon self-cosine: expected f64, got {:?}", other),
    }
}

/// The derived hologram is DATA-DEPENDENT: two holon records differing only in field
/// `b` measure strictly < 1.0 (a non-trivial, valid cosine). Self-cosine alone is
/// always 1.0; this proves `build_holon_hologram` actually encodes the fields, not a
/// constant or empty bundle.
#[test]
fn aggregate_new_holon_hologram_is_data_dependent() {
    let got = call_beside(file!(), ":user::an-holon-diff-cos").expect("eval an-holon-diff-cos");
    match got {
        Value::f64(c) => assert!(
            c < 1.0 - 1e-6 && c > -1.0 - 1e-6,
            "aggregate-new holon hologram: two different-data records must measure a valid cosine < 1.0 \
             (data-dependent, not constant); got {}",
            c
        ),
        other => panic!("aggregate-new holon diff-cosine: expected f64, got {:?}", other),
    }
}
