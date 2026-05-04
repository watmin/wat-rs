//! Arc 148 slice 3 — `values_compare` ord buildout.
//!
//! Verifies that `eval_compare` (now backed by the `values_compare`
//! helper) accepts the same set of types `values_equal` accepts, minus
//! the unordered ones. The slice extends ord coverage to:
//!
//! - `:wat::time::Instant` — chronological
//! - `:wat::time::Duration` — chronological (i64 ns)
//! - `:wat::core::Bytes` (`:wat::core::Vector<wat::core::u8>`) — byte-wise lex
//! - `:wat::core::Vector<T>` (the parametric Vec) — element-wise lex
//! - `:wat::core::Tuple<T...>` — element-wise lex
//! - `:wat::core::Option<T>` — variant-ordered (None < Some(_))
//! - `:wat::core::Result<T,E>` — variant-ordered (Err < Ok)
//! - `:wat::holon::Vector` (the algebra Vector) — bit-exact i8 lex
//!
//! For each newly-covered type: `<`, `>`, `<=`, `>=` exercised. For
//! each rejected type (HashMap, HashSet, Enum, Struct, unit, HolonAST):
//! one runtime TypeMismatch test confirming the existing fall-through
//! arm still triggers. For each recursive type (Vec, Tuple, Option,
//! Result): one shallow-fail-fast and one deep-recursion test.
//!
//! Pattern mirrors `tests/wat_polymorphic_arithmetic.rs` and
//! `tests/wat_u8.rs`: `(:user::main -> :bool)` bodies with no IO args
//! so the boolean falls out of `invoke_user_main` directly.

use std::sync::Arc;
use wat::freeze::{invoke_user_main, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::Value;

fn run_bool(src: &str) -> bool {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup");
    match invoke_user_main(&world, Vec::new()).expect("main") {
        Value::bool(b) => b,
        other => panic!("expected :bool; got {:?}", other),
    }
}

fn run_expecting_runtime_error(src: &str) -> String {
    let world = startup_from_source(src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should pass type-check");
    let err = invoke_user_main(&world, Vec::new()).expect_err("main should error");
    format!("{:?}", err)
}

// ─── Instant — chronological ord ─────────────────────────────────────

#[test]
fn instant_lt_works() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::< (:wat::time::at 1) (:wat::time::at 2)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn instant_gt_works() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::> (:wat::time::at 5) (:wat::time::at 2)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn instant_le_works() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::<= (:wat::time::at 3) (:wat::time::at 3)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn instant_ge_works() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::>= (:wat::time::at 3) (:wat::time::at 4)))
    "#;
    assert!(!run_bool(src));
}

// ─── Duration — chronological ord (i64 ns) ───────────────────────────

#[test]
fn duration_lt_works() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::< (:wat::time::Second 1) (:wat::time::Minute 1)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn duration_gt_works() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::> (:wat::time::Hour 1) (:wat::time::Minute 1)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn duration_le_works() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::<= (:wat::time::Hour 1) (:wat::time::Minute 60)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn duration_ge_works() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::>= (:wat::time::Day 1) (:wat::time::Hour 24)))
    "#;
    assert!(run_bool(src));
}

// ─── Bytes — byte-wise lex (Bytes is :wat::core::Vector<wat::core::u8>) ─

#[test]
fn bytes_lt_works() {
    // [1,2,3] < [1,2,4] — byte-wise lex
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::<
            (:wat::core::Vector :wat::core::u8 (:wat::core::u8 1) (:wat::core::u8 2) (:wat::core::u8 3))
            (:wat::core::Vector :wat::core::u8 (:wat::core::u8 1) (:wat::core::u8 2) (:wat::core::u8 4))))
    "#;
    assert!(run_bool(src));
}

#[test]
fn bytes_gt_works() {
    // [9] > [1] — first element decides
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::>
            (:wat::core::Vector :wat::core::u8 (:wat::core::u8 9))
            (:wat::core::Vector :wat::core::u8 (:wat::core::u8 1))))
    "#;
    assert!(run_bool(src));
}

#[test]
fn bytes_le_works() {
    // [1,2] <= [1,2] — equal lex
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::<=
            (:wat::core::Vector :wat::core::u8 (:wat::core::u8 1) (:wat::core::u8 2))
            (:wat::core::Vector :wat::core::u8 (:wat::core::u8 1) (:wat::core::u8 2))))
    "#;
    assert!(run_bool(src));
}

#[test]
fn bytes_ge_shorter_lt_longer_on_prefix_tie() {
    // Per Rust's slice cmp: [1,2] < [1,2,3] (shorter is less when prefix ties).
    // So [1,2] >= [1,2,3] is false.
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::>=
            (:wat::core::Vector :wat::core::u8 (:wat::core::u8 1) (:wat::core::u8 2))
            (:wat::core::Vector :wat::core::u8 (:wat::core::u8 1) (:wat::core::u8 2) (:wat::core::u8 3))))
    "#;
    assert!(!run_bool(src));
}

// ─── Vec (parametric :wat::core::Vector<T>) — element-wise lex ───────

#[test]
fn vec_i64_lt_works() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::<
            (:wat::core::Vector :wat::core::i64 1 2 3)
            (:wat::core::Vector :wat::core::i64 1 2 4)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn vec_i64_gt_works() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::>
            (:wat::core::Vector :wat::core::i64 5)
            (:wat::core::Vector :wat::core::i64 1)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn vec_string_le_works() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::<=
            (:wat::core::Vector :wat::core::String "a" "b")
            (:wat::core::Vector :wat::core::String "a" "c")))
    "#;
    assert!(run_bool(src));
}

#[test]
fn vec_string_ge_equal_lex() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::>=
            (:wat::core::Vector :wat::core::String "a" "b")
            (:wat::core::Vector :wat::core::String "a" "b")))
    "#;
    assert!(run_bool(src));
}

// ─── Vec recursion — shallow fail-fast + deep recursion ──────────────

#[test]
fn vec_recursion_shallow_first_element_decides() {
    // [9, 1, 1] > [1, 99, 99] — first element wins; rest never inspected.
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::>
            (:wat::core::Vector :wat::core::i64 9 1 1)
            (:wat::core::Vector :wat::core::i64 1 99 99)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn vec_recursion_deep_via_nested_vec() {
    // Vec<Vec<i64>>: [[1,2],[3,4]] < [[1,2],[3,5]] — recursion descends
    // through outer Vec into inner Vec arm, then to i64 leaf.
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::<
            (:wat::core::Vector :wat::core::Vector<wat::core::i64>
              (:wat::core::Vector :wat::core::i64 1 2)
              (:wat::core::Vector :wat::core::i64 3 4))
            (:wat::core::Vector :wat::core::Vector<wat::core::i64>
              (:wat::core::Vector :wat::core::i64 1 2)
              (:wat::core::Vector :wat::core::i64 3 5))))
    "#;
    assert!(run_bool(src));
}

// ─── Tuple — element-wise lex ────────────────────────────────────────

#[test]
fn tuple_lt_works() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::<
            (:wat::core::Tuple 1 "alpha")
            (:wat::core::Tuple 2 "alpha")))
    "#;
    assert!(run_bool(src));
}

#[test]
fn tuple_gt_works() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::>
            (:wat::core::Tuple 5 "z")
            (:wat::core::Tuple 5 "a")))
    "#;
    assert!(run_bool(src));
}

#[test]
fn tuple_le_equal() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::<=
            (:wat::core::Tuple 1 2 3)
            (:wat::core::Tuple 1 2 3)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn tuple_ge_works() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::>=
            (:wat::core::Tuple 10 "x")
            (:wat::core::Tuple 9 "x")))
    "#;
    assert!(run_bool(src));
}

// ─── Tuple recursion — shallow fail-fast + deep recursion ────────────

#[test]
fn tuple_recursion_shallow_first_element_decides() {
    // (1, X) < (2, Y) — second element of either side never inspected.
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::<
            (:wat::core::Tuple 1 "anything-here")
            (:wat::core::Tuple 2 "anything-there")))
    "#;
    assert!(run_bool(src));
}

#[test]
fn tuple_recursion_deep_via_nested_tuple() {
    // Tuple containing Tuple — recursion descends into the inner Tuple
    // arm, then to leaves.
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::<
            (:wat::core::Tuple 1 (:wat::core::Tuple 2 3))
            (:wat::core::Tuple 1 (:wat::core::Tuple 2 4))))
    "#;
    assert!(run_bool(src));
}

// ─── Option — variant-order (None < Some) ────────────────────────────

#[test]
fn option_none_lt_some() {
    // :None < (Some 0) regardless of payload
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::let*
            (((a :wat::core::Option<wat::core::i64>) :wat::core::None)
             ((b :wat::core::Option<wat::core::i64>) (:wat::core::Some 0)))
            (:wat::core::< a b)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn option_some_gt_none() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::let*
            (((a :wat::core::Option<wat::core::i64>) (:wat::core::Some 99)))
            (:wat::core::> a :wat::core::None)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn option_some_le_same_payload() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::<= (:wat::core::Some 5) (:wat::core::Some 5)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn option_some_ge_compares_payload() {
    // Some(7) >= Some(3)
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::>= (:wat::core::Some 7) (:wat::core::Some 3)))
    "#;
    assert!(run_bool(src));
}

// ─── Option recursion — shallow + deep ───────────────────────────────

#[test]
fn option_recursion_shallow_payload_decides() {
    // Some(10) < Some(20) — payload comparison; both Some so variant
    // tag matches, then i64 leaf wins.
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::< (:wat::core::Some 10) (:wat::core::Some 20)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn option_recursion_deep_via_nested_option() {
    // Some(Some(1)) < Some(Some(2))
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::let*
            (((a :wat::core::Option<wat::core::Option<wat::core::i64>>)
              (:wat::core::Some (:wat::core::Some 1)))
             ((b :wat::core::Option<wat::core::Option<wat::core::i64>>)
              (:wat::core::Some (:wat::core::Some 2))))
            (:wat::core::< a b)))
    "#;
    assert!(run_bool(src));
}

// ─── Result — variant-order (Err < Ok) ───────────────────────────────

#[test]
fn result_err_lt_ok() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::let*
            (((a :wat::core::Result<wat::core::i64,wat::core::String>)
              (:wat::core::Err "boom"))
             ((b :wat::core::Result<wat::core::i64,wat::core::String>)
              (:wat::core::Ok 1)))
            (:wat::core::< a b)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn result_ok_gt_err() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::let*
            (((a :wat::core::Result<wat::core::i64,wat::core::String>)
              (:wat::core::Ok 100))
             ((b :wat::core::Result<wat::core::i64,wat::core::String>)
              (:wat::core::Err "anything")))
            (:wat::core::> a b)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn result_ok_le_same_payload() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::let*
            (((a :wat::core::Result<wat::core::i64,wat::core::String>)
              (:wat::core::Ok 5))
             ((b :wat::core::Result<wat::core::i64,wat::core::String>)
              (:wat::core::Ok 5)))
            (:wat::core::<= a b)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn result_err_ge_smaller_err_payload() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::let*
            (((a :wat::core::Result<wat::core::i64,wat::core::String>)
              (:wat::core::Err "z"))
             ((b :wat::core::Result<wat::core::i64,wat::core::String>)
              (:wat::core::Err "a")))
            (:wat::core::>= a b)))
    "#;
    assert!(run_bool(src));
}

// ─── Result recursion — shallow + deep ───────────────────────────────

#[test]
fn result_recursion_shallow_same_variant_payload_decides() {
    // Both Err — payload (String) lex decides.
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::let*
            (((a :wat::core::Result<wat::core::i64,wat::core::String>)
              (:wat::core::Err "alpha"))
             ((b :wat::core::Result<wat::core::i64,wat::core::String>)
              (:wat::core::Err "beta")))
            (:wat::core::< a b)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn result_recursion_deep_via_ok_payload_tuple() {
    // Both Ok with Tuple payload — recursion: Result arm → Tuple arm →
    // i64 leaf.
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::let*
            (((a :wat::core::Result<(wat::core::i64,wat::core::i64),wat::core::String>)
              (:wat::core::Ok (:wat::core::Tuple 1 5)))
             ((b :wat::core::Result<(wat::core::i64,wat::core::i64),wat::core::String>)
              (:wat::core::Ok (:wat::core::Tuple 1 9))))
            (:wat::core::< a b)))
    "#;
    assert!(run_bool(src));
}

// ─── Vector (algebra :wat::holon::Vector) — bit-exact i8 lex ─────────

#[test]
fn algebra_vector_le_self() {
    // Same vector compared to itself: <= true (equal).
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::let*
            (((v :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x"))))
            (:wat::core::<= v v)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn algebra_vector_ge_self() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::let*
            (((v :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x"))))
            (:wat::core::>= v v)))
    "#;
    assert!(run_bool(src));
}

#[test]
fn algebra_vector_lt_self_is_false() {
    // v < v should be false (equality holds).
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::let*
            (((v :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "x"))))
            (:wat::core::< v v)))
    "#;
    assert!(!run_bool(src));
}

#[test]
fn algebra_vector_distinct_atoms_have_some_order() {
    // Two encoded atoms produce distinct Vectors. They are NOT equal,
    // so exactly one of (a < b) and (b < a) is true. We assert that
    // the call returns a bool without raising — establishing the arm
    // is reachable without panicking — by OR'ing the two.
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::let*
            (((va :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "alpha")))
             ((vb :wat::holon::Vector) (:wat::holon::encode (:wat::holon::Atom "omega"))))
            (:wat::core::or (:wat::core::< va vb) (:wat::core::> va vb))))
    "#;
    assert!(run_bool(src));
}

// ─── Rejected types — fall-through arm raises TypeMismatch at runtime ─

#[test]
fn hashmap_ord_raises_type_mismatch() {
    // Two HashMaps; type-checker accepts (same-type unify); runtime
    // values_compare returns None → eval_compare raises TypeMismatch.
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::let*
            (((m1 :wat::core::HashMap<wat::core::String,wat::core::i64>)
              (:wat::core::HashMap :(wat::core::String,wat::core::i64) "a" 1))
             ((m2 :wat::core::HashMap<wat::core::String,wat::core::i64>)
              (:wat::core::HashMap :(wat::core::String,wat::core::i64) "b" 2)))
            (:wat::core::< m1 m2)))
    "#;
    let err = run_expecting_runtime_error(src);
    assert!(
        err.contains("TypeMismatch") || err.to_lowercase().contains("comparable"),
        "expected TypeMismatch on HashMap ord; got {}",
        err
    );
}

#[test]
fn hashset_ord_raises_type_mismatch() {
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::let*
            (((s1 :wat::core::HashSet<wat::core::i64>)
              (:wat::core::HashSet :wat::core::i64 1 2))
             ((s2 :wat::core::HashSet<wat::core::i64>)
              (:wat::core::HashSet :wat::core::i64 3 4)))
            (:wat::core::< s1 s2)))
    "#;
    let err = run_expecting_runtime_error(src);
    assert!(
        err.contains("TypeMismatch") || err.to_lowercase().contains("comparable"),
        "expected TypeMismatch on HashSet ord; got {}",
        err
    );
}

#[test]
fn enum_ord_raises_type_mismatch() {
    // User enum — variants have no inherent order in this slice.
    let src = r#"
        (:wat::core::enum :my::Color :Red :Green :Blue)

        (:wat::core::define (:user::main -> :bool)
          (:wat::core::< :my::Color::Red :my::Color::Blue))
    "#;
    let err = run_expecting_runtime_error(src);
    assert!(
        err.contains("TypeMismatch") || err.to_lowercase().contains("comparable"),
        "expected TypeMismatch on Enum ord; got {}",
        err
    );
}

#[test]
fn struct_ord_raises_type_mismatch() {
    let src = r#"
        (:wat::core::struct :my::Point
          (x :wat::core::i64)
          (y :wat::core::i64))

        (:wat::core::define (:user::main -> :bool)
          (:wat::core::let*
            (((p :my::Point) (:my::Point/new 1 2))
             ((q :my::Point) (:my::Point/new 3 4)))
            (:wat::core::< p q)))
    "#;
    let err = run_expecting_runtime_error(src);
    assert!(
        err.contains("TypeMismatch") || err.to_lowercase().contains("comparable"),
        "expected TypeMismatch on Struct ord; got {}",
        err
    );
}

#[test]
fn unit_ord_raises_type_mismatch() {
    // Two unit values () compared via < — only one inhabitant; no
    // order. Fall-through arm raises.
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::< () ()))
    "#;
    let err = run_expecting_runtime_error(src);
    assert!(
        err.contains("TypeMismatch") || err.to_lowercase().contains("comparable"),
        "expected TypeMismatch on unit ord; got {}",
        err
    );
}

#[test]
fn holon_ast_ord_raises_type_mismatch() {
    // HolonAST is the algebraic surface; no canonical order.
    let src = r#"
        (:wat::core::define (:user::main -> :bool)
          (:wat::core::<
            (:wat::holon::Atom "x")
            (:wat::holon::Atom "y")))
    "#;
    let err = run_expecting_runtime_error(src);
    assert!(
        err.contains("TypeMismatch") || err.to_lowercase().contains("comparable"),
        "expected TypeMismatch on HolonAST ord; got {}",
        err
    );
}
