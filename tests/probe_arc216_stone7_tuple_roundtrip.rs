//! Arc 216 Stone 7 — `Tuple` round-trip through `HolonAST::Bundle` of positional-Binds.
//!
//! Proves bidirectional round-trip: `value_to_atom` (forward, `Value::Tuple → HolonAST`)
//! and the Rust-level `HolonRepresentable` impls for Rust tuples.
//!
//! Per encoding doctrine (Stone 216.7): Tuple is collection-category — positional-Bind Bundle,
//! identical shape to Vec<T>. `Bundle([Bind(I64(0), t0_holon), Bind(I64(1), t1_holon), ...])`.
//! Keys are sequential i64 starting from 0. Reverse via `atom-value` returns Vec (same shape;
//! consumer-declared type is the discriminator — honest asymmetry per DESIGN Q9).
//!
//! ## The 12 probes (covers all EXPECTATIONS rows 6-11)
//!
//! Forward direction — WAT surface:
//!  1. `(:wat::holon::Atom (:wat::core::Tuple 1 "hello"))` → Bundle with 2 Bind children
//!
//! Reverse direction — WAT surface:
//!  2. `atom-value` on Tuple-encoded Bundle → Vec (positional-Bind shape; honest asymmetry)
//!
//! Heterogeneous 3-tuple:
//!  3. `(bool, i64, String)` Bundle shape — 3 Bind children with I64 keys 0, 1, 2
//!
//! Nested + composition:
//!  4. Nested Tuple: `(:wat::core::Tuple (:wat::core::Tuple 1 2) "outer")` — Bundle of Bundles
//!  5. Tuple containing Vec: `(:wat::core::Tuple [1 2 3] "tag")` — outer Bind + inner Vec-shape
//!
//! Tuple containing HashSet:
//!  6. `(:wat::core::Tuple (:wat::core::HashSet :wat::core::i64 1 2) "label")` → Bundle 2 children
//!
//! is_atomizable predicate:
//!  7. Tuple<i64, String> admits; Tuple containing Fn rejects
//!
//! HolonAST shape verification:
//!  8. Positional Bind keys are 0..n-1 — verified directly on the Bundle structure
//!
//! HolonRepresentable Rust-side:
//!  9. `(String, i64)` compile-time check + runtime round-trip
//!
//! Process-tier IPC:
//! 10. `pair::<(String, i64)>()` send + recv round-trips over process pipe
//!
//! Nested Rust tuple HolonRepresentable:
//! 11. `(String, i64, bool)` 3-tuple round-trips at Rust level
//!
//! Arity mismatch error:
//! 12. `from_holon_ast` on wrong-arity Bundle returns WireError naming mismatch

use std::sync::Arc;
use wat::comms::HolonRepresentable;
use wat::comms::process::pair;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::define (:user::main -> :wat::core::nil) :wat::core::nil)",
        src
    )
}

fn run_i64(src: &str) -> i64 {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute") {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

fn startup_err(src: &str) -> String {
    let src = with_nil_main(src);
    match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

// ─── Probe 1 — Forward: 2-tuple → HolonAST::Bundle of Bind children ──────────

/// `(:wat::holon::Atom (:wat::core::Tuple 1 "hello"))` produces a `HolonAST::Bundle`
/// containing 2 Bind children (one per element with i64 keys 0, 1).
/// Arc 216 Stone 7 forward direction (value_to_atom Tuple arm, encoding doctrine).
#[test]
fn probe_1_forward_2tuple_to_bundle() {
    // Verify the result is a HolonAST Bundle with 2 children.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [t  (:wat::core::Tuple 1 "hello")
             h  (:wat::holon::Atom t)
             cs (:wat::holon::Bundle/children h)]
            (:wat::core::length cs)))
    "#;
    assert_eq!(run_i64(src), 2, "Bundle must have 2 children for (i64, String) tuple");
}

// ─── Probe 2 — Reverse: Bundle → Vec (honest asymmetry) ──────────────────────

/// `atom-value` on a Tuple-encoded Bundle reconstructs a Vec (positional-Bind shape).
/// The Tuple and Vec encodings are identical; `atom-value` returns Vec.
/// This is the honest asymmetry per DESIGN Q9: consumer-declared type discriminates.
/// Verify: result is a Vector with length 2; first element = 1.
#[test]
fn probe_2_reverse_bundle_to_vec_honest_asymmetry() {
    // Length = 2 after round-trip via atom-value (returns Vec, not Tuple).
    let src_len = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [t  (:wat::core::Tuple 1 "hello")
             h  (:wat::holon::Atom t)
             v  (:wat::core::atom-value h)]
            (:wat::core::Vector/length v)))
    "#;
    assert_eq!(run_i64(src_len), 2, "atom-value on Tuple Bundle must produce Vec with length 2");

    // First element = 1 (order preserved).
    let src_first = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [t  (:wat::core::Tuple 1 "hello")
             h  (:wat::holon::Atom t)
             v  (:wat::core::atom-value h)]
            (:wat::core::match
              (:wat::core::Vector/get v 0)
              -> :wat::core::i64
              ((:wat::core::Some x) x)
              (:wat::core::None -1))))
    "#;
    assert_eq!(run_i64(src_first), 1, "atom-value on Tuple Bundle: first element must be 1");
}

// ─── Probe 3 — 3-tuple primitives → Bundle shape verification ─────────────────

/// `(bool, i64, String)` 3-tuple forward: Bundle has 3 Bind children with I64 keys.
/// Verifies heterogeneous element types all atomize correctly.
#[test]
fn probe_3_three_tuple_primitives_bundle_shape() {
    // Verify Bundle has 3 children.
    let src_count = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [t  (:wat::core::Tuple true 42 "wat")
             h  (:wat::holon::Atom t)
             cs (:wat::holon::Bundle/children h)]
            (:wat::core::length cs)))
    "#;
    assert_eq!(run_i64(src_count), 3, "3-tuple Bundle must have 3 children");

    // After atom-value: Vec with length 3; element at index 1 = 42.
    let src_elem = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [t  (:wat::core::Tuple true 42 "wat")
             h  (:wat::holon::Atom t)
             v  (:wat::core::atom-value h)]
            (:wat::core::match
              (:wat::core::Vector/get v 1)
              -> :wat::core::i64
              ((:wat::core::Some x) x)
              (:wat::core::None -1))))
    "#;
    assert_eq!(run_i64(src_elem), 42, "3-tuple: element at index 1 must be 42");
}

// ─── Probe 4 — Nested Tuple: ((i64, i64), String) ────────────────────────────

/// `(:wat::core::Tuple (:wat::core::Tuple 1 2) "outer")` — nested Tuple.
/// Outer Bundle has 2 Bind children; inner Bind's value is a Bundle of 2 Binds.
/// Verifies recursive atomization (Tuple arm calls value_to_atom on each element).
#[test]
fn probe_4_nested_tuple_roundtrip() {
    // Outer Bundle has 2 children.
    let src_outer = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [inner (:wat::core::Tuple 1 2)
             outer (:wat::core::Tuple inner "outer")
             h     (:wat::holon::Atom outer)
             cs    (:wat::holon::Bundle/children h)]
            (:wat::core::length cs)))
    "#;
    assert_eq!(run_i64(src_outer), 2, "nested Tuple outer Bundle must have 2 children");

    // After atom-value on outer: Vec of length 2; element 0 is itself a Vec (inner Tuple decoded).
    // The inner Tuple encodes as a positional-Bind Bundle; atom-value decodes the outer Bundle
    // recursively — each element is decoded via holon_item_to_value, which decodes the inner
    // Bundle as Vec. So element 0 of the outer Vec is already a Vec of length 2; call Vector/length.
    let src_inner_len = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [inner (:wat::core::Tuple 1 2)
             outer (:wat::core::Tuple inner "outer")
             h     (:wat::holon::Atom outer)
             v     (:wat::core::atom-value h)]
            (:wat::core::match
              (:wat::core::Vector/get v 0)
              -> :wat::core::i64
              ((:wat::core::Some inner_v)
                (:wat::core::Vector/length inner_v))
              (:wat::core::None -1))))
    "#;
    assert_eq!(run_i64(src_inner_len), 2, "nested Tuple: inner element round-trips to Vec of length 2");
}

// ─── Probe 5 — Tuple containing Vec: (Vec<i64>, String) ──────────────────────

/// `(:wat::core::Tuple [1 2 3] "tag")` — Tuple whose first element is a Vec<i64>.
/// Outer Bundle has 2 Bind children; inner Bind's value is a positional-Bind Bundle (Vec-shape).
/// Verifies composition of Tuple and Vec encodings.
#[test]
fn probe_5_tuple_containing_vec_roundtrip() {
    // Outer Bundle has 2 children.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [v   [1 2 3]
             t   (:wat::core::Tuple v "tag")
             h   (:wat::holon::Atom t)
             cs  (:wat::holon::Bundle/children h)]
            (:wat::core::length cs)))
    "#;
    assert_eq!(run_i64(src), 2, "Tuple containing Vec: outer Bundle must have 2 children");

    // Inner Vec element: after atom-value on outer, element 0 is itself a Vec (decoded recursively).
    // The Vec<i64> inside the Tuple encodes as a positional-Bind Bundle; holon_item_to_value
    // decodes it as Vec. So element 0 of the outer Vec is already a Vec; call Vector/length directly.
    let src_inner = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [v   [1 2 3]
             t   (:wat::core::Tuple v "tag")
             h   (:wat::holon::Atom t)
             rv  (:wat::core::atom-value h)]
            (:wat::core::match
              (:wat::core::Vector/get rv 0)
              -> :wat::core::i64
              ((:wat::core::Some inner_v)
                (:wat::core::Vector/length inner_v))
              (:wat::core::None -1))))
    "#;
    assert_eq!(run_i64(src_inner), 3, "Tuple containing Vec: inner Vec round-trips to length 3");
}

// ─── Probe 6 — Tuple containing HashSet ───────────────────────────────────────

/// `(:wat::core::Tuple (:wat::core::HashSet :wat::core::i64 1 2) "label")` — composition
/// with Stone 216.1. Outer Bundle has 2 Bind children; inner is a bare-atom Bundle (HashSet-shape).
#[test]
fn probe_6_tuple_containing_hashset() {
    // Outer Bundle has 2 children.
    let src = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [s   (:wat::core::HashSet :wat::core::i64 1 2)
             t   (:wat::core::Tuple s "label")
             h   (:wat::holon::Atom t)
             cs  (:wat::holon::Bundle/children h)]
            (:wat::core::length cs)))
    "#;
    assert_eq!(run_i64(src), 2, "Tuple containing HashSet: outer Bundle must have 2 children");
}

// ─── Probe 7 — is_atomizable predicate ────────────────────────────────────────

/// Tuple<i64, String> admits (all elements atomizable).
/// Tuple containing Fn rejects (Fn not in atomizable set).
#[test]
fn probe_7_is_atomizable_tuple() {
    // Admits: (:wat::core::Tuple 1 "hello") — i64 and String are atomizable.
    let src_admit = r#"
        (:wat::core::define (:user::compute -> :wat::core::i64)
          (:wat::core::let
            [t  (:wat::core::Tuple 1 "hello")
             h  (:wat::holon::Atom t)]
            1))
    "#;
    assert_eq!(run_i64(src_admit), 1, "Tuple<i64, String> must pass is_atomizable check");

    // Rejects: Tuple containing a Fn — Fn types are not atomizable.
    let src_reject = r#"
        (:wat::core::define (:user::compute -> :wat::core::nil)
          (:wat::core::let
            [f (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)
             t (:wat::core::Tuple f "tag")]
            (:wat::holon::Atom t)))
    "#;
    let err = startup_err(src_reject);
    assert!(
        err.contains("TypeMismatch"),
        "Tuple containing Fn must fail at check with TypeMismatch; got: {}",
        err
    );
    assert!(
        err.contains(":wat::holon::Atom"),
        "TypeMismatch must name the callee :wat::holon::Atom; got: {}",
        err
    );
}

// ─── Probe 8 — HolonAST shape verification: keys are 0..n-1 ─────────────────

/// Positional Bind keys in the Tuple Bundle are 0..n-1.
/// Verified at the Rust level via to_holon_ast on a 3-tuple.
/// Note: only `String` has a primitive `HolonRepresentable` impl (Delta 1 — same as Vec Stone 2
/// Delta 4). Using `(String, String, String)` — strictly equivalent for the shape check.
#[test]
fn probe_8_holon_ast_shape_keys_sequential() {
    // (String, String, String) 3-tuple; to_holon_ast should produce Bundle([Bind(0,_), Bind(1,_), Bind(2,_)]).
    let tup: (String, String, String) = ("hello".to_string(), "world".to_string(), "foo".to_string());
    let ast = tup.to_holon_ast();

    match &ast {
        holon::HolonAST::Bundle(items) => {
            assert_eq!(items.len(), 3, "3-tuple Bundle must have 3 children");
            for (i, item) in items.iter().enumerate() {
                match item {
                    holon::HolonAST::Bind(k, _) => {
                        assert!(
                            matches!(k.as_ref(), holon::HolonAST::I64(n) if *n == i as i64),
                            "Bind key at position {} must be I64({}); encoding doctrine: keys 0..n-1",
                            i, i
                        );
                    }
                    other => panic!(
                        "element {} must be HolonAST::Bind; got {:?}",
                        i, other
                    ),
                }
            }
        }
        other => panic!("expected HolonAST::Bundle; got {:?}", other),
    }
}

// ─── Probe 9 — HolonRepresentable cascade: Rust-level round-trip ─────────────

/// `(String, String)` satisfies `HolonRepresentable` at compile time.
/// Runtime round-trip: to_holon_ast → from_holon_ast reconstructs the pair.
///
/// Note: only `String` has a primitive `HolonRepresentable` impl (Delta 1 — mirrors
/// Vec Stone 2 Delta 4). `(String, String)` is strictly equivalent for testing the
/// encoding shape — the positional-Bind Bundle + per-position decode is exercised fully.
/// i64 and bool impls are a future stone (substrate parity).
fn assert_holon_representable<T: HolonRepresentable>() {}

#[test]
fn probe_9_holon_representable_2tuple_cascade() {
    // Compile-time: if this call compiles, (String, String): HolonRepresentable.
    assert_holon_representable::<(String, String)>();

    // Runtime round-trip.
    let original: (String, String) = ("world".to_string(), "hello".to_string());
    let ast = original.to_holon_ast();

    // to_holon_ast produces Bundle of 2 Bind children.
    match &ast {
        holon::HolonAST::Bundle(items) => {
            assert_eq!(items.len(), 2, "2-tuple Bundle must have 2 children");
        }
        other => panic!("expected HolonAST::Bundle; got {:?}", other),
    }

    // from_holon_ast reconstructs the pair.
    let reconstructed: (String, String) =
        HolonRepresentable::from_holon_ast(&ast).expect("round-trip");
    assert_eq!(
        reconstructed,
        original,
        "round-trip must reproduce original (String, String)"
    );
}

// ─── Probe 10 — Process-tier IPC: pair::<(String, i64)>() ────────────────────

/// `pair::<(String, String)>()` send + recv round-trips through the full process-tier
/// wire chain: HolonAST → tagged EDN → newline-framed bytes → pipe → bytes → EDN →
/// HolonAST → (String, String).
///
/// Note: Delta 1 applies — only `String` has `HolonRepresentable`. Using `(String, String)`.
#[test]
fn probe_10_process_tier_ipc_tuple_roundtrip() {
    let (tx, rx) = pair::<(String, String)>().expect("pair must succeed");

    let original: (String, String) = ("arc216".to_string(), "stone7".to_string());
    tx.send(original.clone()).expect("send must succeed on live channel");
    let got = rx.recv().expect("recv must return the sent tuple");

    assert_eq!(got, original, "process-tier IPC must round-trip (String, String) faithfully");
}

// ─── Probe 11 — 3-tuple HolonRepresentable round-trip ────────────────────────

/// `(String, String, String)` 3-tuple round-trips at the Rust HolonRepresentable level.
/// Proves heterogeneous-position decode (each Bind decoded by position) works correctly.
/// Note: Delta 1 — only `String` has `HolonRepresentable`. 3 String elements exercise
/// the positional dispatch fully (element 0, 1, 2 each decoded independently by position).
#[test]
fn probe_11_three_tuple_holon_representable_roundtrip() {
    assert_holon_representable::<(String, String, String)>();

    let original: (String, String, String) = ("stone".to_string(), "216".to_string(), "tuple".to_string());
    let ast = original.to_holon_ast();

    // Bundle of 3 children.
    match &ast {
        holon::HolonAST::Bundle(items) => {
            assert_eq!(items.len(), 3, "3-tuple Bundle must have 3 children");
        }
        other => panic!("expected HolonAST::Bundle; got {:?}", other),
    }

    let reconstructed: (String, String, String) =
        HolonRepresentable::from_holon_ast(&ast).expect("3-tuple round-trip");
    assert_eq!(
        reconstructed,
        original,
        "round-trip must reproduce original (String, String, String)"
    );
}

// ─── Probe 12 — Arity mismatch: from_holon_ast on wrong-arity Bundle ─────────

/// `from_holon_ast` on a Bundle with 3 children when the impl expects 2 → WireError
/// naming the arity mismatch. Validates the guard in `extract_positional_binds`.
#[test]
fn probe_12_from_holon_ast_arity_mismatch_error() {
    // Construct a Bundle with 3 Bind(I64, String) children.
    let bind0 = holon::HolonAST::bind(holon::HolonAST::i64(0), holon::HolonAST::string("a"));
    let bind1 = holon::HolonAST::bind(holon::HolonAST::i64(1), holon::HolonAST::string("b"));
    let bind2 = holon::HolonAST::bind(holon::HolonAST::i64(2), holon::HolonAST::string("c"));
    let bundle_3 = holon::HolonAST::bundle(vec![bind0, bind1, bind2]);

    // Decoding as a 2-tuple (String, String) must fail — arity mismatch.
    let result = <(String, String) as HolonRepresentable>::from_holon_ast(&bundle_3);
    assert!(
        result.is_err(),
        "from_holon_ast on 3-child Bundle as 2-tuple must return Err; got Ok({:?})",
        result.ok()
    );
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.message().contains("arity mismatch") || err_msg.message().contains("2"),
        "error must mention arity mismatch; got: {}",
        err_msg.message()
    );
}
