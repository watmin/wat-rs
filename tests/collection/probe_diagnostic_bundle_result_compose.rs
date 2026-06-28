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
//! Outcomes:
//!   - PASS: Task #478 DISCONFIRMED. Bundle composes via Result/expect.
//!     Stone 227.2 v2 should ship canonical instance shape, not flat Bind.
//!   - FAIL: SPECIFIC failure surfaced; Task #478 stays open with the actual
//!     blocker named.

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

// ─── Probe 1: Bind composes with Bundle via Result/expect ────────────────────

/// Disconfirms: "Bundle's Result return blocks Bind composition."
///
/// Build the canonical defrecord-style instance shape:
///   Bind(Atom("test::Foo"), Bundle(Bind(Atom("a"), Atom(1)),
///                                  Bind(Atom("b"), Atom(2))))
///
/// Verify via the type predicate (is? instance "test::Foo") → true.
#[test]
fn probe_1_bind_composes_with_bundle_via_result_expect() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::probe1-bind-composes)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
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
/// Bind(Atom, Bundle(...)) shape; verify the inner Bundle's child count
/// via statement-length on the extracted inner-bundle handle = 3.
#[test]
fn probe_2_canonical_instance_shape_preserves_inner_bundle() {
    let world = startup_beside(file!()).expect("startup");
    let ast = wat::parse_one!("(:t::probe2-inner-bundle-preserved)").expect("parse");
    match eval_in_frozen(&ast, &world, &Environment::new()).expect("eval").value_owned() {
        Value::i64(n) => assert_eq!(
            n, 3,
            "expected inner Bundle to preserve 3 field-Bind children; got {}",
            n
        ),
        other => panic!("expected i64; got {:?}", other),
    }
}
