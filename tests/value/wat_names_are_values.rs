//! Integration coverage for arc 009 — names are values.
//!
//! A registered user/stdlib define's keyword-path evaluates to a
//! `Value::wat__core__fn` in expression position; the type
//! checker infers a `:wat::core::Fn(params)->ret` scheme for the same position.
//! Callers pass named defines to `:wat::core::Fn(...)`-typed parameters without
//! a pass-through fn wrapper — the asymmetry with
//! `:wat::kernel::spawn-thread`'s long-standing accept-by-name
//! convention dissolves.
//!
//! Arc 170 slice 1f-ζ: migrate from invoke_user_main/stdout capture to
//! eval_in_frozen with compute functions returning values directly.
//!
//! Wat source lives in the co-located fixture: wat_names_are_values.wat
//! (slurped via startup_beside(file!())).

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn run(world: &wat::freeze::FrozenWorld, expr: &str) -> Value {
    let ast = wat::parse_one!(expr).expect("parse expr");
    eval_in_frozen(&ast, world, &Environment::new())
        .expect("compute should run")
        .value_owned()
}

// ─── named define lifts to a callable value ────────────────────────────

#[test]
fn named_define_is_a_function_value() {
    // `:t::test1-double` is registered as a define. Referencing it in
    // expression position (not call-head) produces a fn value that can
    // be called by the user via a symbol binding.
    // Arc 170 slice 1f-ζ: returns i64 (42) via :t::test1.
    let world = startup_beside(file!()).expect("startup");
    match run(&world, "(:t::test1)") {
        Value::i64(n) => assert_eq!(n, 42, "expected 42; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── named define as higher-order argument ─────────────────────────────

#[test]
fn named_define_passes_to_higher_order_fn() {
    // A user-defined higher-order function `:t::test2-apply-twice` takes
    // `:wat::core::Fn(wat::core::i64)->wat::core::i64` and an `:wat::core::i64`; calling it with
    // `:t::test2-inc` and `5` via the bare keyword path — no fn wrapper — yields 7.
    // Arc 170 slice 1f-ζ: returns i64 (7) via :t::test2.
    let world = startup_beside(file!()).expect("startup");
    match run(&world, "(:t::test2)") {
        Value::i64(n) => assert_eq!(n, 7, "expected 7; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── polymorphic named define — instantiation at call site ─────────────

#[test]
fn polymorphic_named_define_instantiates_at_use_site() {
    // Polymorphic `:t::test3-identity<T>`. Passed to a monomorphic
    // `:wat::core::Fn(wat::core::i64)->wat::core::i64` slot; the scheme's `T` instantiates to `i64`.
    // Arc 170 slice 1f-ζ: returns i64 (99) via :t::test3.
    let world = startup_beside(file!()).expect("startup");
    match run(&world, "(:t::test3)") {
        Value::i64(n) => assert_eq!(n, 99, "expected 99; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── unregistered keyword stays a literal ──────────────────────────────

#[test]
fn unregistered_keyword_still_a_literal() {
    // A keyword that is NOT a registered define remains a
    // `:wat::core::keyword` value. The lift is only when a define
    // exists at that path.
    // Arc 170 slice 1f-ζ: returns i64 (1=pass, 0=fail) via :t::test4.
    let world = startup_beside(file!()).expect("startup");
    match run(&world, "(:t::test4)") {
        Value::i64(n) => assert_eq!(n, 1, "expected 1 (pass); got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── named define as map argument ───────────────────────────────

#[test]
fn named_define_as_map_fn() {
    // The canonical target: pass `:t::test5-double` to `:wat::core::map`
    // without wrapping in a pass-through fn.
    // Arc 170 slice 1f-ζ: returns i64 via :t::test5 (1=pass, 0=fail).
    // (Migrated off the annihilated `:wat::stream::*` — arc 118, 2026-06-27;
    //  the intent is named-defn-as-HOF-arg, the collection vehicle is incidental.)
    let world = startup_beside(file!()).expect("startup");
    match run(&world, "(:t::test5)") {
        Value::i64(n) => assert_eq!(n, 1, "expected 1 (pass); got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}
