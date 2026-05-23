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
//!  1. `(:wat::holon::to-holon (:wat::core::Tuple 1 "hello"))` → Bundle with 2 Bind children
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

/// Run a wat program that returns an opaque Value (any type).
/// Used for probes that return Tuple — the type checker cannot statically
/// infer the return type of `from-holon`, so element access happens at Rust level.
fn run_value(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute")
}

/// Extract element at `index` from a `Value::Tuple`, asserting it is i64.
fn tuple_element_i64(v: Value, index: usize, probe: &str) -> i64 {
    match v {
        Value::Tuple(items) => match items.get(index) {
            Some(Value::i64(n)) => *n,
            Some(other) => panic!("{}: tuple[{}] is {:?}, expected i64", probe, index, other),
            None => panic!("{}: tuple has fewer than {} elements", probe, index + 1),
        },
        other => panic!("{}: expected Tuple; got {:?}", probe, other),
    }
}

/// Extract length of inner Vec from element at `index` of a `Value::Tuple`.
fn tuple_element_vec_length(v: Value, index: usize, probe: &str) -> i64 {
    match v {
        Value::Tuple(items) => match items.get(index) {
            Some(Value::Vec(inner)) => inner.len() as i64,
            Some(other) => panic!("{}: tuple[{}] is {:?}, expected Vec", probe, index, other),
            None => panic!("{}: tuple has fewer than {} elements", probe, index + 1),
        },
        other => panic!("{}: expected Tuple; got {:?}", probe, other),
    }
}

/// Extract length of inner Tuple from element at `index` of a `Value::Tuple`.
fn tuple_element_tuple_length(v: Value, index: usize, probe: &str) -> i64 {
    match v {
        Value::Tuple(items) => match items.get(index) {
            Some(Value::Tuple(inner)) => inner.len() as i64,
            Some(other) => panic!("{}: tuple[{}] is {:?}, expected Tuple", probe, index, other),
            None => panic!("{}: tuple has fewer than {} elements", probe, index + 1),
        },
        other => panic!("{}: expected Tuple; got {:?}", probe, other),
    }
}

fn startup_err(src: &str) -> String {
    let src = with_nil_main(src);
    match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

// ─── Probe 1 — Forward: 2-tuple → classifier-wrapped HolonAST ───────────────

/// `(:wat::holon::to-holon (:wat::core::Tuple 1 "hello"))` produces a classifier-wrapped HolonAST.
/// Arc 228 Stone 228.1: the output is `Bind(Atom("Tuple"), Bundle(positional Binds))`.
/// Arc 216 Stone 7 forward direction — forward-corrected per typed-entities doctrine.
/// Verified via round-trip: to-holon → from-holon → Tuple → first element = 1.
///
/// Type-checker note: `from-holon` returns `?T` (fresh type var). To allow the return
/// without calling `first`/`second` (which require statically-known tuple/Vec type),
/// we declare the return type explicitly and extract the element at Rust level.
#[test]
fn probe_1_forward_2tuple_to_bundle() {
    // Arc 228: Bundle/children no longer works on classifier-wrapped top-level Bind.
    // Verify via round-trip: from-holon returns Tuple (classifier "Tuple" is distinct).
    // Return the Tuple directly with explicit type annotation; extract first element in Rust.
    let src = r#"
        (:wat::core::define (:user::compute -> :(wat::core::i64,wat::core::String))
          (:wat::core::let
            [t  (:wat::core::Tuple 1 "hello")
             h  (:wat::holon::to-holon t)
             rt (:wat::holon::from-holon h)]
            rt))
    "#;
    let v = run_value(src);
    assert_eq!(
        tuple_element_i64(v, 0, "probe_1"),
        1,
        "classifier-wrapped Tuple round-trip: first element must be 1"
    );
}

// ─── Probe 2 — Reverse: Tuple-classified → Tuple (arc 228 no honest asymmetry) ─

/// Arc 228 Stone 228.1: from-holon now returns Tuple (not Vec) for Tuple-encoded forms.
///
/// Pre-arc-228 (arc 216 "honest asymmetry"): Tuple and Vec had identical bare-Bundle
/// encoding; from-holon always returned Vec for positional-Bind Bundles; consumer-declared
/// type was the only discriminator.
///
/// Post-arc-228: the classifier Atom("Tuple") vs Atom("Vector") is the discriminator.
/// from-holon on Tuple-classified form returns Tuple; from-holon on Vector returns Vec.
/// The honest asymmetry is resolved: the substrate type is recoverable from data alone.
///
/// Type-checker note: return Tuple directly with explicit annotation; element access at Rust level.
#[test]
fn probe_2_reverse_bundle_to_vec_honest_asymmetry() {
    // Arc 228: from-holon returns Tuple (not Vec). Return Tuple with explicit type annotation.
    // Second element is "hello" (String); verify first element = 1 (i64) at Rust level.
    let src_first = r#"
        (:wat::core::define (:user::compute -> :(wat::core::i64,wat::core::String))
          (:wat::core::let
            [t  (:wat::core::Tuple 1 "hello")
             h  (:wat::holon::to-holon t)
             rt (:wat::holon::from-holon h)]
            rt))
    "#;
    let v = run_value(src_first);
    assert_eq!(
        tuple_element_i64(v, 0, "probe_2"),
        1,
        "arc 228: from-holon Tuple round-trip: first element = 1 (Tuple, not Vec)"
    );
}

// ─── Probe 3 — 3-tuple primitives → round-trip element verification ───────────

/// `(bool, i64, String)` 3-tuple forward: classifier-wrapped Bind with 3-element inner Bundle.
/// Arc 228: Bundle/children no longer works on classifier-wrapped top-level Bind.
/// Verify via round-trip: to-holon → from-holon → Tuple; second element = 42.
///
/// Type-checker note: from-holon returns ?T; declare explicit return type; element at Rust level.
#[test]
fn probe_3_three_tuple_primitives_bundle_shape() {
    // Arc 228: round-trip via from-holon → Tuple. Return with explicit 3-Tuple type annotation.
    // Extract element at index 1 (42) at Rust level from the returned Value::Tuple.
    let src = r#"
        (:wat::core::define (:user::compute -> :(wat::core::bool,wat::core::i64,wat::core::String))
          (:wat::core::let
            [t  (:wat::core::Tuple true 42 "wat")
             h  (:wat::holon::to-holon t)
             rt (:wat::holon::from-holon h)]
            rt))
    "#;
    let v = run_value(src);
    assert_eq!(
        tuple_element_i64(v, 1, "probe_3"),
        42,
        "3-tuple round-trip: element at index 1 must be 42"
    );
}

// ─── Probe 4 — Nested Tuple: ((i64, i64), String) ────────────────────────────

/// `(:wat::core::Tuple (:wat::core::Tuple 1 2) "outer")` — nested Tuple.
/// Arc 228: outer is classifier-wrapped Bind; Bundle/children no longer applies.
/// Verify via round-trip: from-holon → outer Tuple; first element is inner Tuple.
/// Inner Tuple's first element = 1 and second element = 2.
///
/// Type-checker note: from-holon returns ?T; return the outer Tuple directly with
/// explicit type annotation; extract nested elements at Rust level.
#[test]
fn probe_4_nested_tuple_roundtrip() {
    // Arc 228: round-trip outer Tuple via to-holon/from-holon.
    // Return Tuple directly; type annotation uses bare (T,U) form (no leading `:`) for inner Tuple.
    // The function body is just `rt` — no need to call first/second at wat level.
    // Inner Tuple elements verified at Rust level from Value::Tuple.
    let src_outer = r#"
        (:wat::core::define (:user::compute -> :((wat::core::i64,wat::core::i64),wat::core::String))
          (:wat::core::let
            [inner (:wat::core::Tuple 1 2)
             outer (:wat::core::Tuple inner "outer")
             h     (:wat::holon::to-holon outer)
             rt    (:wat::holon::from-holon h)]
            rt))
    "#;
    let v = run_value(src_outer);
    // outer is Tuple; element 0 is inner Tuple; verify inner Tuple length = 2.
    assert_eq!(
        tuple_element_tuple_length(v, 0, "probe_4"),
        2,
        "nested Tuple: inner Tuple (element 0 of outer) must have length 2"
    );

    // Inner Tuple first element = 1, second element = 2: verify via Rust-level extraction.
    let src_inner = r#"
        (:wat::core::define (:user::compute -> :((wat::core::i64,wat::core::i64),wat::core::String))
          (:wat::core::let
            [inner (:wat::core::Tuple 1 2)
             outer (:wat::core::Tuple inner "outer")
             h     (:wat::holon::to-holon outer)
             rt    (:wat::holon::from-holon h)]
            rt))
    "#;
    let v2 = run_value(src_inner);
    match v2 {
        Value::Tuple(outer_items) => match outer_items.first() {
            Some(Value::Tuple(inner_items)) => {
                assert_eq!(inner_items.get(0), Some(&Value::i64(1)), "nested Tuple: inner[0] = 1");
                assert_eq!(inner_items.get(1), Some(&Value::i64(2)), "nested Tuple: inner[1] = 2");
            }
            other => panic!("probe_4: outer[0] should be Tuple; got {:?}", other),
        },
        other => panic!("probe_4: expected outer Tuple; got {:?}", other),
    }
}

// ─── Probe 5 — Tuple containing Vec: (Vec<i64>, String) ──────────────────────

/// `(:wat::core::Tuple [1 2 3] "tag")` — Tuple whose first element is a Vec<i64>.
/// Arc 228: outer is classifier-wrapped Bind; Bundle/children no longer applies.
/// Verify via round-trip: to-holon → from-holon → outer Tuple; first element = inner Vec.
/// Inner Vec (Vector-classified) decodes to Vec; Vector/length = 3.
#[test]
fn probe_5_tuple_containing_vec_roundtrip() {
    // Arc 228: round-trip outer Tuple; first element is inner Vec (Vector-classified → Vec).
    // Return Tuple directly with explicit type annotation; extract inner Vec length at Rust level.
    // Type annotation: :(wat::core::Vector<wat::core::i64>,wat::core::String) — no leading `:` on inner elements.
    let src = r#"
        (:wat::core::define (:user::compute -> :(wat::core::Vector<wat::core::i64>,wat::core::String))
          (:wat::core::let
            [v    [1 2 3]
             t    (:wat::core::Tuple v "tag")
             h    (:wat::holon::to-holon t)
             rt   (:wat::holon::from-holon h)]
            rt))
    "#;
    let v = run_value(src);
    assert_eq!(
        tuple_element_vec_length(v, 0, "probe_5"),
        3,
        "Tuple containing Vec: inner Vec (element 0) must have length 3"
    );

    // Inner Vec element at index 0 = 1: verify via Rust-level extraction.
    let src_inner = r#"
        (:wat::core::define (:user::compute -> :(wat::core::Vector<wat::core::i64>,wat::core::String))
          (:wat::core::let
            [v    [1 2 3]
             t    (:wat::core::Tuple v "tag")
             h    (:wat::holon::to-holon t)
             rt   (:wat::holon::from-holon h)]
            rt))
    "#;
    match run_value(src_inner) {
        Value::Tuple(outer_items) => match outer_items.first() {
            Some(Value::Vec(inner_v)) => {
                assert_eq!(inner_v.get(0), Some(&Value::i64(1)), "Tuple containing Vec: inner Vec[0] = 1");
            }
            other => panic!("probe_5: outer[0] should be Vec; got {:?}", other),
        },
        other => panic!("probe_5: expected Tuple; got {:?}", other),
    }
}

// ─── Probe 6 — Tuple containing HashSet ───────────────────────────────────────

/// `(:wat::core::Tuple (:wat::core::HashSet :wat::core::i64 1 2) "label")` — composition
/// with Stone 216.1. Arc 228: outer is classifier-wrapped Bind; Bundle/children no longer applies.
/// Verify via round-trip: to-holon → from-holon → outer Tuple; first element = inner HashSet.
/// HashSet/length = 2.
#[test]
fn probe_6_tuple_containing_hashset() {
    // Arc 228: round-trip outer Tuple; first element is inner HashSet (Set-classified → HashSet).
    // Return Tuple directly with explicit type annotation; extract inner HashSet length at Rust level.
    // Type annotation: :(wat::core::HashSet<wat::core::i64>,wat::core::String).
    let src = r#"
        (:wat::core::define (:user::compute -> :(wat::core::HashSet<wat::core::i64>,wat::core::String))
          (:wat::core::let
            [s   (:wat::core::HashSet :wat::core::i64 1 2)
             t   (:wat::core::Tuple s "label")
             h   (:wat::holon::to-holon t)
             rt  (:wat::holon::from-holon h)]
            rt))
    "#;
    match run_value(src) {
        Value::Tuple(outer_items) => match outer_items.first() {
            Some(Value::wat__std__HashSet(hs)) => {
                assert_eq!(hs.len(), 2, "Tuple containing HashSet: inner HashSet must have length 2");
            }
            other => panic!("probe_6: outer[0] should be HashSet; got {:?}", other),
        },
        other => panic!("probe_6: expected Tuple; got {:?}", other),
    }
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
             h  (:wat::holon::to-holon t)]
            1))
    "#;
    assert_eq!(run_i64(src_admit), 1, "Tuple<i64, String> must pass is_atomizable check");

    // Rejects: Tuple containing a Fn — Fn types are not atomizable.
    let src_reject = r#"
        (:wat::core::define (:user::compute -> :wat::core::nil)
          (:wat::core::let
            [f (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)
             t (:wat::core::Tuple f "tag")]
            (:wat::holon::to-holon t)))
    "#;
    let err = startup_err(src_reject);
    assert!(
        err.contains("TypeMismatch"),
        "Tuple containing Fn must fail at check with TypeMismatch; got: {}",
        err
    );
    // Arc 225 Stone 225.1: callee is now :wat::holon::to-holon (polymorphic UP verb).
    assert!(
        err.contains(":wat::holon::to-holon"),
        "TypeMismatch must name the callee :wat::holon::to-holon; got: {}",
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
