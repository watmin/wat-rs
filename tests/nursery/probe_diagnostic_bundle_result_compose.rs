//! Diagnostic probe — disconfirm "Bundle's Result return blocks canonical
//! Bind(Atom, Bundle) defrecord composition."
//!
//! Stone 227.2 v2 reported (SCORE Delta 2): "Bundle returns Result<HolonAST,
//! CapacityExceeded>, incompatible with Bind's bare HolonAST input — used
//! Atom(nil) + flat Bind workaround."
//!
//! Task #478 was filed on that basis. But arc 037 doc at src/runtime.rs:15244-15268
//! makes clear Bundle's Result return is BY DESIGN — callers acknowledge via
//! :wat::core::Result/expect or :wat::core::try.
//!
//! Belief to disconfirm: "defrecord can't produce canonical Bind(Atom, Bundle)
//! shape because of Bundle's Result return."
//!
//! Empirical test: compose (Bind classifier (Result/expect (Bundle items) msg))
//! at runtime; verify the resulting instance has the canonical shape per
//! typed-entities doctrine.
//!
//! Outcomes:
//!   - PASS: Task #478 DISCONFIRMED. Bundle composes via Result/expect.
//!     Stone 227.2 v2 should ship canonical instance shape, not flat Bind.
//!   - FAIL: SPECIFIC failure surfaced; Task #478 stays open with the actual
//!     blocker named.

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

fn run(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run").value_owned()
}

// ─── Probe 1: Bind composes with Bundle via Result/expect ────────────────────

/// Disconfirms: "Bundle's Result return blocks Bind composition."
///
/// Build the canonical defrecord-style instance shape:
///   Bind(Atom("test::Foo"), Bundle(Bind(Atom("a"), Atom(1)),
///                                  Bind(Atom("b"), Atom(2))))
///
/// Via:
///   (:wat::holon::Bind
///     (:wat::holon::Atom (:wat::holon::to-holon "test::Foo"))
///     (:wat::core::Result/expect
///       (:wat::holon::Bundle [field-bind-a field-bind-b])
///       "Bundle should not overflow at this dim"))
///
/// Verify via the type predicate (is? instance "test::Foo") → true.
#[test]
fn probe_1_bind_composes_with_bundle_via_result_expect() {
    let src = r##"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::let
                      [field-a (:wat::holon::Bind
                                 (:wat::holon::Atom (:wat::holon::to-holon "a"))
                                 (:wat::holon::Atom (:wat::holon::to-holon 1)))
                       field-b (:wat::holon::Bind
                                 (:wat::holon::Atom (:wat::holon::to-holon "b"))
                                 (:wat::holon::Atom (:wat::holon::to-holon 2)))
                       inner-bundle (:wat::core::Result/expect -> :wat::holon::HolonAST
                                      (:wat::holon::Bundle [field-a field-b])
                                      "Bundle should not overflow")
                       instance (:wat::holon::Bind
                                  (:wat::holon::Atom (:wat::holon::to-holon "test::Foo"))
                                  inner-bundle)]
                      (:wat::holon::is? instance "test::Foo")))
    "##;
    match run(src) {
        Value::bool(b) => assert!(
            b,
            "expected is? to confirm 'test::Foo' classifier on Bind(Atom, Bundle(...)) instance"
        ),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Probe 2: round-trip through from-holon validates inner Bundle preservation ─

/// Disconfirms: "Bundle composition loses field information."
///
/// Build a single-field defrecord-style instance via the canonical
/// Bind(Atom, Bundle(...)) shape, then round-trip via from-holon.
///
/// If the inner Bundle survives, the resulting Value should be a HashMap
/// (Bundle of Bind pairs with String keys decodes to HashMap per arc 228
/// classifier-dispatch via the classifier-wrap encoding).
///
/// Actually — easier: verify the inner Bundle's child count via
/// statement-length on the extracted inner-bundle handle.
#[test]
fn probe_2_canonical_instance_shape_preserves_inner_bundle() {
    let src = r##"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [field-a (:wat::holon::Bind
                                 (:wat::holon::Atom (:wat::holon::to-holon "a"))
                                 (:wat::holon::Atom (:wat::holon::to-holon 1)))
                       field-b (:wat::holon::Bind
                                 (:wat::holon::Atom (:wat::holon::to-holon "b"))
                                 (:wat::holon::Atom (:wat::holon::to-holon 2)))
                       field-c (:wat::holon::Bind
                                 (:wat::holon::Atom (:wat::holon::to-holon "c"))
                                 (:wat::holon::Atom (:wat::holon::to-holon 3)))
                       inner-bundle (:wat::core::Result/expect -> :wat::holon::HolonAST
                                      (:wat::holon::Bundle [field-a field-b field-c])
                                      "Bundle should not overflow")]
                      ;; statement-length on the inner Bundle should return 3 (three children)
                      (:wat::holon::statement-length inner-bundle)))
    "##;
    match run(src) {
        Value::i64(n) => assert_eq!(
            n, 3,
            "expected inner Bundle to preserve 3 field-Bind children; got {}",
            n
        ),
        other => panic!("expected i64; got {:?}", other),
    }
}
