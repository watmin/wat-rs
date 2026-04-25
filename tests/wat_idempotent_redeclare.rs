//! Arc 054 — Idempotent re-declaration for typealias / define / defmacro.
//!
//! Three registries gain "byte-equivalent re-registration is a no-op."
//! Divergent re-registration remains an error.
//!
//! Coverage:
//! - typealias: byte-equivalent → ok; divergent → error
//! - define: byte-equivalent → ok; divergent → error
//! - defmacro: byte-equivalent → ok (divergent path covered by lib test)

use std::sync::Arc;
use wat::freeze::startup_from_source;
use wat::load::InMemoryLoader;

fn freeze_ok(src: &str) {
    let result = startup_from_source(src, None, Arc::new(InMemoryLoader::new()));
    assert!(
        result.is_ok(),
        "expected freeze to succeed; got error: {:?}",
        result.err()
    );
}

fn freeze_err(src: &str) -> String {
    let err = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect_err("expected freeze to fail");
    format!("{:?}", err)
}

// ─── Typealias ───────────────────────────────────────────────────────

#[test]
fn typealias_byte_equivalent_is_noop() {
    let src = r##"
        (:wat::core::typealias :my::Amount :f64)
        (:wat::core::typealias :my::Amount :f64)

        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::io::IOWriter/println stdout "ok"))
    "##;
    freeze_ok(src);
}

#[test]
fn typealias_divergent_errors() {
    let src = r##"
        (:wat::core::typealias :my::Amount :f64)
        (:wat::core::typealias :my::Amount :i64)

        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::io::IOWriter/println stdout "ok"))
    "##;
    let err = freeze_err(src);
    assert!(
        err.contains("duplicate") || err.contains("Duplicate") || err.contains("Amount"),
        "expected duplicate-type error mentioning Amount; got: {}",
        err
    );
}

// ─── Define ──────────────────────────────────────────────────────────

#[test]
fn define_byte_equivalent_is_noop() {
    let src = r##"
        (:wat::core::define (:my::add-one (a :i64) -> :i64) (:wat::core::+ a 1))
        (:wat::core::define (:my::add-one (a :i64) -> :i64) (:wat::core::+ a 1))

        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::io::IOWriter/println stdout "ok"))
    "##;
    freeze_ok(src);
}

#[test]
fn define_divergent_body_errors() {
    let src = r##"
        (:wat::core::define (:my::add-one (a :i64) -> :i64) (:wat::core::+ a 1))
        (:wat::core::define (:my::add-one (a :i64) -> :i64) (:wat::core::+ a 2))

        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::io::IOWriter/println stdout "ok"))
    "##;
    let err = freeze_err(src);
    assert!(
        err.contains("Duplicate") || err.contains("duplicate") || err.contains("add-one"),
        "expected duplicate-define error; got: {}",
        err
    );
}

// ─── Defmacro ────────────────────────────────────────────────────────

#[test]
fn defmacro_byte_equivalent_is_noop() {
    let src = r##"
        (:wat::core::defmacro (:my::ident (x :AST) -> :AST) `,x)
        (:wat::core::defmacro (:my::ident (x :AST) -> :AST) `,x)

        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::io::IOWriter/println stdout "ok"))
    "##;
    freeze_ok(src);
}

// ─── In-crate-shim shape — the motivating case ──────────────────────
//
// The lab's CandleStream shim ships its wat surface BOTH via
// `wat_sources()` and as an on-disk file loaded by main.wat / test
// preludes. Both paths register the same typealias. Pre-arc-054, that
// was a duplicate-type error. Post-arc-054, it's a no-op for the
// second registration. This test simulates the shape:
// the same `(:wat::core::typealias ...)` reaches the registry twice.

#[test]
fn shim_double_register_pattern_works() {
    let src = r##"
        ;; First registration — as if delivered by wat_sources()
        (:wat::core::typealias :lab::candles::Stream :i64)

        ;; Second registration — as if delivered by (:wat::load-file! ...)
        ;; resolving to the same file content
        (:wat::core::typealias :lab::candles::Stream :i64)

        (:wat::core::define
          (:user::main
            (stdin :wat::io::IOReader)
            (stdout :wat::io::IOWriter)
            (stderr :wat::io::IOWriter)
            -> :())
          (:wat::io::IOWriter/println stdout "ok"))
    "##;
    freeze_ok(src);
}
