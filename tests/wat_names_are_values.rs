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
//! eval_in_frozen with :my::compute returning values directly.

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

// ─── named define lifts to a callable value ────────────────────────────

#[test]
fn named_define_is_a_function_value() {
    // `:my::double` is registered as a define. Referencing it in
    // expression position (not call-head) produces a fn value that can
    // be called by the user via a symbol binding.
    // Arc 170 slice 1f-ζ: returns i64 (42) via :my::compute.
    let src = r##"

        (:wat::core::define (:my::double (x :wat::core::i64) -> :wat::core::i64)
          (:wat::core::i64::*'2 x 2))

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::let
            [f :my::double
             result (f 21)]
            result))
    "##;
    match run(src) {
        Value::i64(n) => assert_eq!(n, 42, "expected 42; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── named define as higher-order argument ─────────────────────────────

#[test]
fn named_define_passes_to_higher_order_fn() {
    // A user-defined higher-order function `:my::apply-twice` takes
    // `:wat::core::Fn(wat::core::i64)->wat::core::i64` and an `:wat::core::i64`; calling it with `:my::inc` and
    // `5` via the bare keyword path — no fn wrapper — yields 7.
    // Arc 170 slice 1f-ζ: returns i64 (7) via :my::compute.
    let src = r##"

        (:wat::core::define (:my::inc (n :wat::core::i64) -> :wat::core::i64)
          (:wat::core::i64::+'2 n 1))

        (:wat::core::define (:my::apply-twice (f :wat::core::Fn(wat::core::i64)->wat::core::i64) (x :wat::core::i64) -> :wat::core::i64)
          (f (f x)))

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:my::apply-twice :my::inc 5))
    "##;
    match run(src) {
        Value::i64(n) => assert_eq!(n, 7, "expected 7; got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── polymorphic named define — instantiation at call site ─────────────

#[test]
fn polymorphic_named_define_instantiates_at_use_site() {
    // Polymorphic `:my::identity<T>`. Passed to a monomorphic
    // `:wat::core::Fn(wat::core::i64)->wat::core::i64` slot; the scheme's `T` instantiates to `i64`.
    // Arc 170 slice 1f-ζ: returns i64 (99) via :my::compute.
    let src = r##"

        (:wat::core::define (:my::identity<T> (x :T) -> :T) x)

        (:wat::core::define (:my::apply (f :wat::core::Fn(wat::core::i64)->wat::core::i64) (x :wat::core::i64) -> :wat::core::i64)
          (f x))

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:my::apply :my::identity 99))
    "##;
    match run(src) {
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
    // Arc 170 slice 1f-ζ: returns i64 (1=pass, 0=fail) via :my::compute.
    let src = r##"

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::let
            [tag :my-app::tag::user-event
             same? (:wat::core::= tag :my-app::tag::user-event)]
            (:wat::core::if same? -> :wat::core::i64
              1
              0)))
    "##;
    match run(src) {
        Value::i64(n) => assert_eq!(n, 1, "expected 1 (pass); got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── named define as stream map argument ───────────────────────────────

#[test]
fn named_define_as_stream_map_fn() {
    // The canonical target: pass `:my::double` to `:wat::stream::map`
    // without wrapping in a pass-through fn.
    // Arc 170 slice 1f-ζ: returns i64 via :my::compute (first doubled value = 2).
    let src = r##"

        (:wat::core::define (:my::double (n :wat::core::i64) -> :wat::core::i64)
          (:wat::core::i64::*'2 n 2))

        (:wat::core::define (:my::compute -> :wat::core::i64)
          (:wat::core::let
            [source
              (:wat::stream::spawn-producer
                (:wat::core::fn [tx <- :wat::kernel::Sender<wat::core::i64>] -> :wat::core::nil
                  (:wat::core::do
                    (:wat::core::Result/expect -> :wat::core::nil
                      (:wat::kernel::send tx 1)
                      "producer: tx disconnected on send 1")
                    (:wat::core::Result/expect -> :wat::core::nil
                      (:wat::kernel::send tx 2)
                      "producer: tx disconnected on send 2")
                    (:wat::core::Result/expect -> :wat::core::nil
                      (:wat::kernel::send tx 3)
                      "producer: tx disconnected on send 3")
                    ())))
             doubled
              (:wat::stream::map source :my::double)
             collected (:wat::stream::collect doubled)
             first
              (:wat::core::match (:wat::core::first collected) -> :wat::core::i64
                ((:wat::core::Some n) n)
                (:wat::core::None -1))
             len (:wat::core::length collected)]
            (:wat::core::if (:wat::core::and (:wat::core::= first 2) (:wat::core::= len 3))
              -> :wat::core::i64
              1
              0)))
    "##;
    match run(src) {
        Value::i64(n) => assert_eq!(n, 1, "expected 1 (pass); got {}", n),
        other => panic!("expected i64; got {:?}", other),
    }
}
