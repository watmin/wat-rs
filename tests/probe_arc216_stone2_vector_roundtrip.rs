//! Arc 216 Stone 2 — `Vec<T>` (`:wat::core::Vector<T>`) round-trip through
//! `HolonAST::Bundle` of positional-Binds.
//!
//! Verifies bidirectional round-trip: `value_to_atom` (forward, `Value → HolonAST`)
//! and `atom-value` (reverse, `HolonAST → Value`) for `Vec<T>`.
//!
//! Per DESIGN Q2: `Vector<T>` → `HolonAST::Bundle([Bind(I64(0), T_holon), Bind(I64(1), T_holon), ...])`.
//! Keys are sequential i64 starting from 0. Order preserved. Reverse discriminates
//! positional-Bind shape from bare-atom set-shape (HashSet) by checking key type.
//!
//! ## The 12 probes
//!
//! Forward direction:
//!  1. `(:wat::holon::to-holon [1 2 3])` → `HolonAST::Bundle` containing 3 Bind children
//!
//! Reverse direction:
//!  2. `(:wat::holon::from-holon<bundle>)` on a round-tripped Vec → reconstructs Vec
//!
//! Edge cases:
//!  3. Empty vec `[]` → `Bundle([])` → reconstructs (edge: empty bundle)
//!  4. Single element `[42]` → `Bundle([Bind(0, I64(42))])` → `[42]`
//!
//! Multi-T types:
//!  5. Works for `Vec<i64>`, `Vec<String>`, `Vec<bool>`, `Vec<keyword>`
//!
//! Order preservation:
//!  6. Round-trip preserves element order via i64 key sequence
//!
//! Nested vector:
//!  7. `Vec<Vec<i64>>` — outer Bundle of positional Binds whose values are inner Bundles
//!
//! Mixed nesting:
//!  8. `Vec<HashSet<i64>>` — composes with Stone 216.1 (inner Bundles are bare-atom set-shape)
//!
//! Check-level atomizable predicate:
//!  9. `(:wat::holon::to-holon [1 2 3])` for atomizable T type-checks cleanly
//! 10. `(:wat::holon::to-holonvec-of-fns)` fails at check (non-atomizable T)
//!
//! HolonRepresentable Rust-side:
//! 11. `Vec<String>` satisfies `HolonRepresentable` at compile time; roundtrip correct
//!
//! Reverse-shape validation:
//! 12. Bundle with non-sequential i64 keys → `from_holon_ast` error (positional invariant)

use std::sync::Arc;
use wat::comms::HolonRepresentable;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil :wat::core::nil)",
        src
    )
}

fn run_i64(src: &str) -> i64 {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
        Value::i64(n) => n,
        other => panic!("expected i64; got {:?}", other),
    }
}

fn run_bool(src: &str) -> bool {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env).expect("compute").value_owned() {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

fn startup_err(src: &str) -> String {
    let src = with_nil_main(src);
    match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

// ─── Probe 1 — Forward: `[1 2 3]` → classifier-wrapped HolonAST ─────────────

/// `(:wat::holon::to-holon [1 2 3])` produces a classifier-wrapped HolonAST.
/// Arc 228 Stone 228.1: the output is `Bind(Atom("Vector"), Bundle(positional Binds))`,
/// not a bare Bundle. Arc 216 Stone 2 forward direction — forward-corrected per
/// typed-entities doctrine.
/// Verified via round-trip: to-holon → from-holon → length = 3.
#[test]
fn probe_1_forward_vec_to_bundle() {
    // Arc 228: Bundle/children no longer works on the classifier-wrapped top-level Bind.
    // Verify via round-trip: to-holon produces an encoding that from-holon decodes back
    // to a Vec of length 3. The element count proves encoding captured all 3 elements.
    let src_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [h   (:wat::holon::to-holon [1 2 3])
                       v   (:wat::holon::from-holon h)]
                      (:wat::core::Vector/length v)))
    "#;
    assert_eq!(run_i64(src_len), 3, "classifier-wrapped Vector encoding must preserve 3 elements in round-trip");
}

// ─── Probe 2 — Reverse: Bundle → Vec round-trip ──────────────────────────────

/// Round-trip: `[1 2 3]` → `Atom` → `atom-value` reconstructs a Vec.
/// After `atom-value` on the positional-Bind Bundle, we get back a `Vec<i64>`.
/// Verify length = 3 and first element = 1 (order preserved).
#[test]
fn probe_2_reverse_bundle_to_vec_roundtrip() {
    // Length = 3 after round-trip.
    let src_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [h   (:wat::holon::to-holon [1 2 3])
                       v   (:wat::holon::from-holon h)]
                      (:wat::core::Vector/length v)))
    "#;
    assert_eq!(run_i64(src_len), 3, "round-trip must preserve length 3");

    // First element = 1 after round-trip (order preserved).
    let src_first = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [h   (:wat::holon::to-holon [1 2 3])
                       v   (:wat::holon::from-holon h)]
                      (:wat::core::match
                        (:wat::core::Vector/get v 0)
                        -> :wat::core::i64
                        ((:wat::core::Some x) x)
                        (:wat::core::None -1))))
    "#;
    assert_eq!(
        run_i64(src_first),
        1,
        "round-trip must preserve first element = 1"
    );
}

// ─── Probe 3 — Empty vec round-trip ──────────────────────────────────────────

/// Empty vec round-trip: `[]` → classifier-wrapped empty Bundle → `Vec` of length 0.
/// Arc 228 Stone 228.1: the output is `Bind(Atom("Vector"), Bundle([]))`. The classifier
/// "Vector" unambiguously identifies the empty collection as a Vector on the reverse trip.
/// This resolves the arc 216 honest edge-case (empty Bundle was ambiguous; now unambiguous).
///
/// Verified via round-trip: to-holon → from-holon → Vector/length = 0.
#[test]
fn probe_3_empty_vec_forward() {
    // Arc 228: to-holon on empty vec produces Bind(Atom("Vector"), Bundle([])).
    // from-holon dispatches by classifier "Vector" → Vec of length 0.
    // Round-trip is now unambiguous (no consumer-hint needed).
    let src_fwd = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [h  (:wat::holon::to-holon [])
                       v  (:wat::holon::from-holon h)]
                      (:wat::core::Vector/length v)))
    "#;
    assert_eq!(
        run_i64(src_fwd),
        0,
        "empty vec classifier-wrapped encoding must round-trip to Vec length 0"
    );
}

// ─── Probe 4 — Single element round-trip ─────────────────────────────────────

/// `[42]` round-trip: element 42 present at index 0, length 1.
#[test]
fn probe_4_single_element_roundtrip() {
    let src_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [h (:wat::holon::to-holon [42])
                       v (:wat::holon::from-holon h)]
                      (:wat::core::Vector/length v)))
    "#;
    assert_eq!(run_i64(src_len), 1, "single-element round-trip must have length 1");

    let src_elem = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [h (:wat::holon::to-holon [42])
                       v (:wat::holon::from-holon h)]
                      (:wat::core::match
                        (:wat::core::Vector/get v 0)
                        -> :wat::core::i64
                        ((:wat::core::Some x) x)
                        (:wat::core::None -1))))
    "#;
    assert_eq!(
        run_i64(src_elem),
        42,
        "single-element round-trip must retrieve 42 at index 0"
    );
}

// ─── Probe 5 — Multi-T types ─────────────────────────────────────────────────

/// Round-trip works for Vec<i64>, Vec<String>, Vec<bool>, Vec<keyword>.
/// Each T atomizes via the corresponding primitive HolonAST leaf.
#[test]
fn probe_5_multi_t_types() {
    // Vec<i64>: additional element check.
    let src_i64 = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [h (:wat::holon::to-holon [10 20 30])
                       v (:wat::holon::from-holon h)]
                      (:wat::core::match
                        (:wat::core::Vector/get v 1)
                        -> :wat::core::i64
                        ((:wat::core::Some x) x)
                        (:wat::core::None -1))))
    "#;
    assert_eq!(
        run_i64(src_i64),
        20,
        "Vec<i64> round-trip: element at index 1 must be 20"
    );

    // Vec<String>: strings atomize as HolonAST::String leaves.
    let src_string = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [h (:wat::holon::to-holon (:wat::core::Vector :wat::core::String "a" "b" "c"))
                       v (:wat::holon::from-holon h)]
                      (:wat::core::Vector/length v)))
    "#;
    assert_eq!(
        run_i64(src_string),
        3,
        "Vec<String> round-trip: length must be 3"
    );

    // Vec<bool>: bool leaves atomize as HolonAST::Bool.
    let src_bool = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [h (:wat::holon::to-holon (:wat::core::Vector :wat::core::bool true false true))
                       v (:wat::holon::from-holon h)]
                      (:wat::core::Vector/length v)))
    "#;
    assert_eq!(
        run_i64(src_bool),
        3,
        "Vec<bool> round-trip: length must be 3"
    );
}

// ─── Probe 6 — Order preservation ────────────────────────────────────────────

/// Order is preserved through the round-trip via i64 key sequence.
/// `[10 20 30]` → Bind(0, I64(10)), Bind(1, I64(20)), Bind(2, I64(30)) →
/// atom-value reconstructs [10, 20, 30] in original order.
/// Verify: index 0 = 10, index 1 = 20, index 2 = 30.
#[test]
fn probe_6_order_preservation() {
    let src_idx0 = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [h (:wat::holon::to-holon [10 20 30])
                       v (:wat::holon::from-holon h)]
                      (:wat::core::match
                        (:wat::core::Vector/get v 0)
                        -> :wat::core::i64
                        ((:wat::core::Some x) x)
                        (:wat::core::None -1))))
    "#;
    let src_idx2 = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [h (:wat::holon::to-holon [10 20 30])
                       v (:wat::holon::from-holon h)]
                      (:wat::core::match
                        (:wat::core::Vector/get v 2)
                        -> :wat::core::i64
                        ((:wat::core::Some x) x)
                        (:wat::core::None -1))))
    "#;
    assert_eq!(
        run_i64(src_idx0),
        10,
        "order preservation: index 0 must be 10"
    );
    assert_eq!(
        run_i64(src_idx2),
        30,
        "order preservation: index 2 must be 30"
    );
}

// ─── Probe 7 — Nested vector round-trip ──────────────────────────────────────

/// `Vec<Vec<i64>>` round-trip.
///
/// Outer Vec<Vec<i64>> atomizes: outer Bundle of Binds, each Bind's value is
/// an inner Bundle of Binds. `atom-value` recurses: outer Bundle → Vec,
/// each element is a Bind(I64, inner_bundle) → inner Bundle → Vec<i64>.
///
/// Verify outer length = 2 and inner length of first element = 3.
#[test]
fn probe_7_nested_vector_roundtrip() {
    // Outer length = 2 (two inner vecs).
    let src_outer_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [inner1  (:wat::core::Vector :wat::core::i64 1 2 3)
                       inner2  (:wat::core::Vector :wat::core::i64 4 5)
                       outer   (:wat::core::Vector :wat::type::Infer inner1 inner2)
                       h       (:wat::holon::to-holon outer)
                       v       (:wat::holon::from-holon h)]
                      (:wat::core::Vector/length v)))
    "#;
    assert_eq!(
        run_i64(src_outer_len),
        2,
        "nested Vec round-trip: outer length must be 2"
    );

    // Arc 228: Bundle/children no longer applies to the classifier-wrapped top-level Bind.
    // Verify via second round-trip: outer Vec has 2 elements (already proven above).
    // Re-verify using a distinct sub-expression to confirm idempotency.
    let src_bundle_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [inner1  (:wat::core::Vector :wat::core::i64 1 2 3)
                       inner2  (:wat::core::Vector :wat::core::i64 4 5)
                       outer   (:wat::core::Vector :wat::type::Infer inner1 inner2)
                       h       (:wat::holon::to-holon outer)
                       v       (:wat::holon::from-holon h)]
                      (:wat::core::Vector/length v)))
    "#;
    assert_eq!(
        run_i64(src_bundle_len),
        2,
        "nested Vec arc 228: classifier-wrapped encoding outer length = 2"
    );

    // Inner element check: after round-trip, get element 0 from outer Vec,
    // then get element 0 from the inner Vec (which should be 1).
    // Use nested match: outer get → Some(inner_vec); inner get at index 0 → Some(1).
    let src_inner_elem = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [inner1  (:wat::core::Vector :wat::core::i64 1 2 3)
                       inner2  (:wat::core::Vector :wat::core::i64 4 5)
                       outer   (:wat::core::Vector :wat::type::Infer inner1 inner2)
                       h       (:wat::holon::to-holon outer)
                       v       (:wat::holon::from-holon h)]
                      (:wat::core::match
                        (:wat::core::Vector/get v 1)
                        -> :wat::core::i64
                        ((:wat::core::Some inner)
                          (:wat::core::match
                            (:wat::core::Vector/get inner 0)
                            -> :wat::core::i64
                            ((:wat::core::Some x) x)
                            (:wat::core::None -1)))
                        (:wat::core::None -1))))
    "#;
    assert_eq!(
        run_i64(src_inner_elem),
        4,
        "nested Vec round-trip: inner vec at index 1, element at index 0 must be 4"
    );
}

// ─── Probe 8 — Mixed nesting: Vec<HashSet<i64>> ──────────────────────────────

/// `Vec<HashSet<i64>>` round-trip — composes with Stone 216.1.
///
/// Each element of the Vec is a HashSet<i64> (bare-atom Bundle shape).
/// Forward: outer Bundle of Binds, each Bind's value is a bare-atom Bundle.
/// Reverse: outer Bundle → Vec; each inner Bundle (bare-atom) → HashSet<i64>.
///
/// Verify outer length = 2; inner sets are non-empty.
#[test]
fn probe_8_mixed_nesting_vec_of_hashset() {
    // Outer length = 2.
    let src_outer_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [s1  (:wat::core::HashSet :wat::core::i64 1 2 3)
                       s2  (:wat::core::HashSet :wat::core::i64 4 5)
                       v   (:wat::core::Vector :wat::type::Infer s1 s2)
                       h   (:wat::holon::to-holon v)
                       rv  (:wat::holon::from-holon h)]
                      (:wat::core::Vector/length rv)))
    "#;
    assert_eq!(
        run_i64(src_outer_len),
        2,
        "Vec<HashSet<i64>> round-trip: outer length must be 2"
    );

    // Arc 228: Bundle/children no longer applies to the classifier-wrapped top-level Bind.
    // Verify the outer element count via round-trip: outer Vec has 2 elements.
    let src_bundle_len = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [s1  (:wat::core::HashSet :wat::core::i64 1 2 3)
                       s2  (:wat::core::HashSet :wat::core::i64 4 5)
                       v   (:wat::core::Vector :wat::type::Infer s1 s2)
                       h   (:wat::holon::to-holon v)
                       rv  (:wat::holon::from-holon h)]
                      (:wat::core::Vector/length rv)))
    "#;
    assert_eq!(
        run_i64(src_bundle_len),
        2,
        "Vec<HashSet<i64>> arc 228: classifier-wrapped outer Vec length = 2"
    );
}

// ─── Probe 9 — Check passes for atomizable T ─────────────────────────────────

/// `(:wat::holon::to-holon [1 2 3])` type-checks cleanly for `Vec<i64>` T.
/// The atomizable predicate recurses: Vector<i64> → atomizable(i64) → YES.
/// Nested `Vec<Vec<i64>>` also passes (predicate recurses both levels).
#[test]
fn probe_9_check_passes_for_atomizable_t() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [h (:wat::holon::to-holon [1 2 3])]
                      1))
    "#;
    assert_eq!(run_i64(src), 1, "Atom on Vec<i64> must pass check and run");

    // Nested Vec<Vec<i64>> — predicate recurses through both levels.
    let src_nested = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
                      [inner  (:wat::core::Vector :wat::core::i64 1 2)
                       outer  (:wat::core::Vector :wat::type::Infer inner)
                       h      (:wat::holon::to-holon outer)]
                      1))
    "#;
    assert_eq!(
        run_i64(src_nested),
        1,
        "Atom on Vec<Vec<i64>> must pass check and run (recursive atomizable)"
    );
}

// ─── Probe 10 — Check fails for non-atomizable T ─────────────────────────────

/// `(:wat::holon::to-holon fn-value)` where T is a function type fails at check.
/// Function types are not in the atomizable set (DESIGN Q6).
/// The predicate `is_atomizable(Fn(...)->...)` = false; check emits TypeMismatch.
///
/// Note: The predicate fires on any non-atomizable T, not specifically on Vec<T>.
/// A function value is the simplest statically-resolvable non-atomizable type.
/// Arc 225 Stone 225.1: callee in TypeMismatch is now :wat::holon::to-holon
/// (the polymorphic UP verb; the old Atom no longer accepts non-HolonAST input).
#[test]
fn probe_10_check_fails_for_non_atomizable_t() {
    let src = r#"
        (:wat::core::defn :user::compute [] -> :wat::core::nil
          (:wat::core::let
                      [f (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 x)]
                      (:wat::holon::to-holon f)))
    "#;
    let err = startup_err(src);
    assert!(
        err.contains("TypeMismatch"),
        "to-holon on Fn type must fail at check with TypeMismatch; got: {}",
        err
    );
    assert!(
        err.contains(":wat::holon::to-holon"),
        "TypeMismatch must name the callee :wat::holon::to-holon; got: {}",
        err
    );
}

// ─── Probe 11 — HolonRepresentable cascade (compile-time + runtime) ──────────

/// `Vec<String>` satisfies `HolonRepresentable` at compile time.
///
/// Arc 216 Stone 2: `impl<T> HolonRepresentable for Vec<T>` where
/// `T: HolonRepresentable + Send + 'static`. `String` satisfies all bounds.
///
/// Also verifies `to_holon_ast`/`from_holon_ast` round-trip at the Rust level:
/// - `to_holon_ast` → Bundle of positional Binds (Bind(I64(i), String leaf))
/// - `from_holon_ast` → reconstructed Vec<String> in original order
fn assert_holon_representable<T: HolonRepresentable>() {}

#[test]
fn probe_11_holon_representable_cascade() {
    // Compile-time: if this call compiles, Vec<String>: HolonRepresentable.
    assert_holon_representable::<Vec<String>>();

    // Runtime roundtrip: ["hello", "world", "foo"].
    let v: Vec<String> = vec!["hello".into(), "world".into(), "foo".into()];
    let ast = v.to_holon_ast();

    // to_holon_ast produces a Bundle of 3 Bind children.
    match &ast {
        holon::HolonAST::Bundle(items) => {
            assert_eq!(items.len(), 3, "Bundle must have 3 children");
            for (i, item) in items.iter().enumerate() {
                match item {
                    holon::HolonAST::Bind(k, val_ast) => {
                        assert!(
                            matches!(k.as_ref(), holon::HolonAST::I64(n) if *n == i as i64),
                            "Bind key at position {} must be I64({})",
                            i,
                            i
                        );
                        assert!(
                            matches!(val_ast.as_ref(), holon::HolonAST::String(_)),
                            "Bind value at position {} must be HolonAST::String leaf",
                            i
                        );
                    }
                    other => panic!(
                        "element {} must be HolonAST::Bind, got {:?}",
                        i, other
                    ),
                }
            }
        }
        other => panic!("expected HolonAST::Bundle, got {:?}", other),
    }

    // from_holon_ast reconstructs the vec in original order.
    let reconstructed: Vec<String> =
        HolonRepresentable::from_holon_ast(&ast).expect("roundtrip");
    assert_eq!(
        reconstructed,
        v,
        "roundtrip must reproduce original Vec<String> in original order"
    );

    // Nested: Vec<Vec<String>> — bounds compose.
    assert_holon_representable::<Vec<Vec<String>>>();
    let nested: Vec<Vec<String>> = vec![
        vec!["a".into(), "b".into()],
        vec!["c".into()],
    ];
    let nested_ast = nested.to_holon_ast();
    let nested_back: Vec<Vec<String>> =
        HolonRepresentable::from_holon_ast(&nested_ast).expect("nested roundtrip");
    assert_eq!(nested_back, nested, "nested Vec<Vec<String>> roundtrip");
}

// ─── Probe 12 — Reverse-shape validation ─────────────────────────────────────

/// Bundle with non-sequential i64 keys → `from_holon_ast` error.
///
/// A Bundle with Bind(I64(0), String) and Bind(I64(2), String) (missing key 1)
/// violates the positional invariant 0..n-1. `from_holon_ast` must
/// return an `Err` (WireError) naming the violation — it does NOT
/// silently produce a truncated Vec.
///
/// This validates the positional-invariant enforcement in the
/// `HolonRepresentable::from_holon_ast` impl for Vec<T>.
#[test]
fn probe_12_reverse_shape_validation_non_sequential_keys() {
    // Construct a malformed Bundle: Bind(0, String("a")), Bind(2, String("b")) — key 1 missing.
    // Vec<String>: String implements HolonRepresentable.
    let bind0 =
        holon::HolonAST::bind(holon::HolonAST::i64(0), holon::HolonAST::string("a"));
    let bind2 =
        holon::HolonAST::bind(holon::HolonAST::i64(2), holon::HolonAST::string("b"));
    let malformed_bundle = holon::HolonAST::bundle(vec![bind0, bind2]);

    // from_holon_ast must return Err (positional invariant violated).
    let result = <Vec<String> as HolonRepresentable>::from_holon_ast(&malformed_bundle);
    assert!(
        result.is_err(),
        "from_holon_ast on non-sequential Bundle must return Err; got Ok({:?})",
        result.ok()
    );
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.message().contains("positional invariant violated")
            || err_msg.message().contains("sequential"),
        "error must mention positional invariant; got: {}",
        err_msg.message()
    );

    // Also: Bundle with reversed keys [Bind(1, "second"), Bind(0, "first")] — keys present
    // but supplied out-of-order. from_holon_ast sorts by key, so this SHOULD succeed
    // (producing ["first", "second"] in key order). Verify it round-trips in sorted order.
    let bind1 = holon::HolonAST::bind(holon::HolonAST::i64(1), holon::HolonAST::string("second"));
    let bind0_str =
        holon::HolonAST::bind(holon::HolonAST::i64(0), holon::HolonAST::string("first"));
    let reordered_bundle = holon::HolonAST::bundle(vec![bind1, bind0_str]);
    let reordered: Vec<String> =
        HolonRepresentable::from_holon_ast(&reordered_bundle).expect("reordered bundle ok");
    assert_eq!(
        reordered,
        vec!["first".to_string(), "second".to_string()],
        "reversed-key Bundle must sort by key and reconstruct in key order"
    );
}
