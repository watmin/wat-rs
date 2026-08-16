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
//! `tests/wat_u8.rs`: `(:user::compute -> :wat::core::bool)` bodies with no IO args
//! so the boolean falls out of `invoke_user_main` directly.

use wat::freeze::{startup_from_file, StartupError};
use wat::runtime::{apply_function, Value};

fn run_bool(path: &str) -> bool {
    let world = startup_from_file(path).expect("startup");
    let func = world.symbols().get(":user::compute").expect(":user::compute").clone();
    match apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()).expect("compute") {
        Value::bool(b) => b,
        other => panic!("expected :wat::core::bool; got {:?}", other),
    }
}

/// Stone 245.8 — ordering is a relational intrinsic: non-orderable types are
/// rejected at check time (not at runtime). Assert that startup_from_file
/// returns a Check error containing "TypeMismatch".
fn run_expecting_check_error(path: &str) -> String {
    match startup_from_file(path) {
        Err(StartupError::Check(errs)) => format!("{:?}", errs),
        Ok(_) => panic!("expected check-time rejection but startup succeeded"),
        Err(other) => panic!("expected StartupError::Check but got: {:?}", other),
    }
}

// ─── Instant — chronological ord ─────────────────────────────────────

#[test]
fn instant_lt_works() {
    assert!(run_bool("tests/types/ord_instant_lt.wat"));
}

#[test]
fn instant_gt_works() {
    assert!(run_bool("tests/types/ord_instant_gt.wat"));
}

#[test]
fn instant_le_works() {
    assert!(run_bool("tests/types/ord_instant_le.wat"));
}

#[test]
fn instant_ge_works() {
    assert!(!run_bool("tests/types/ord_instant_ge.wat"));
}

// ─── Duration — chronological ord (i64 ns) ───────────────────────────

#[test]
fn duration_lt_works() {
    assert!(run_bool("tests/types/ord_duration_lt.wat"));
}

#[test]
fn duration_gt_works() {
    assert!(run_bool("tests/types/ord_duration_gt.wat"));
}

#[test]
fn duration_le_works() {
    assert!(run_bool("tests/types/ord_duration_le.wat"));
}

#[test]
fn duration_ge_works() {
    assert!(run_bool("tests/types/ord_duration_ge.wat"));
}

// ─── Bytes — byte-wise lex (Bytes is :wat::core::Vector<wat::core::u8>) ─

#[test]
fn bytes_lt_works() {
    assert!(run_bool("tests/types/ord_bytes_lt.wat"));
}

#[test]
fn bytes_gt_works() {
    assert!(run_bool("tests/types/ord_bytes_gt.wat"));
}

#[test]
fn bytes_le_works() {
    assert!(run_bool("tests/types/ord_bytes_le.wat"));
}

#[test]
fn bytes_ge_shorter_lt_longer_on_prefix_tie() {
    // [1,2] >= [1,2,3] is false — shorter is less when prefix ties
    assert!(!run_bool("tests/types/ord_bytes_ge_prefix_tie.wat"));
}

// ─── Vec (parametric :wat::core::Vector<T>) — element-wise lex ───────

#[test]
fn vec_i64_lt_works() {
    assert!(run_bool("tests/types/ord_vec_i64_lt.wat"));
}

#[test]
fn vec_i64_gt_works() {
    assert!(run_bool("tests/types/ord_vec_i64_gt.wat"));
}

#[test]
fn vec_string_le_works() {
    assert!(run_bool("tests/types/ord_vec_string_le.wat"));
}

#[test]
fn vec_string_ge_equal_lex() {
    assert!(run_bool("tests/types/ord_vec_string_ge_equal.wat"));
}

// ─── Vec recursion — shallow fail-fast + deep recursion ──────────────

#[test]
fn vec_recursion_shallow_first_element_decides() {
    assert!(run_bool("tests/types/ord_vec_recursion_shallow.wat"));
}

#[test]
fn vec_recursion_deep_via_nested_vec() {
    assert!(run_bool("tests/types/ord_vec_recursion_deep.wat"));
}

// ─── Tuple — element-wise lex ────────────────────────────────────────

#[test]
fn tuple_lt_works() {
    assert!(run_bool("tests/types/ord_tuple_lt.wat"));
}

#[test]
fn tuple_gt_works() {
    assert!(run_bool("tests/types/ord_tuple_gt.wat"));
}

#[test]
fn tuple_le_equal() {
    assert!(run_bool("tests/types/ord_tuple_le_equal.wat"));
}

#[test]
fn tuple_ge_works() {
    assert!(run_bool("tests/types/ord_tuple_ge.wat"));
}

// ─── Tuple recursion — shallow fail-fast + deep recursion ────────────

#[test]
fn tuple_recursion_shallow_first_element_decides() {
    assert!(run_bool("tests/types/ord_tuple_recursion_shallow.wat"));
}

#[test]
fn tuple_recursion_deep_via_nested_tuple() {
    assert!(run_bool("tests/types/ord_tuple_recursion_deep.wat"));
}

// ─── Option — variant-order (None < Some) ────────────────────────────

#[test]
fn option_none_lt_some() {
    assert!(run_bool("tests/types/ord_option_none_lt_some.wat"));
}

#[test]
fn option_some_gt_none() {
    assert!(run_bool("tests/types/ord_option_some_gt_none.wat"));
}

#[test]
fn option_some_le_same_payload() {
    assert!(run_bool("tests/types/ord_option_some_le_same.wat"));
}

#[test]
fn option_some_ge_compares_payload() {
    assert!(run_bool("tests/types/ord_option_some_ge_payload.wat"));
}

// ─── Option recursion — shallow + deep ───────────────────────────────

#[test]
fn option_recursion_shallow_payload_decides() {
    assert!(run_bool("tests/types/ord_option_recursion_shallow.wat"));
}

#[test]
fn option_recursion_deep_via_nested_option() {
    assert!(run_bool("tests/types/ord_option_recursion_deep.wat"));
}

// ─── Result — variant-order (Err < Ok) ───────────────────────────────

#[test]
fn result_err_lt_ok() {
    assert!(run_bool("tests/types/ord_result_err_lt_ok.wat"));
}

#[test]
fn result_ok_gt_err() {
    assert!(run_bool("tests/types/ord_result_ok_gt_err.wat"));
}

#[test]
fn result_ok_le_same_payload() {
    assert!(run_bool("tests/types/ord_result_ok_le_same.wat"));
}

#[test]
fn result_err_ge_smaller_err_payload() {
    assert!(run_bool("tests/types/ord_result_err_ge_smaller.wat"));
}

// ─── Result recursion — shallow + deep ───────────────────────────────

#[test]
fn result_recursion_shallow_same_variant_payload_decides() {
    assert!(run_bool("tests/types/ord_result_recursion_shallow.wat"));
}

#[test]
fn result_recursion_deep_via_ok_payload_tuple() {
    assert!(run_bool("tests/types/ord_result_recursion_deep.wat"));
}

// ─── Vector (algebra :wat::holon::Vector) — bit-exact i8 lex ─────────

#[test]
fn algebra_vector_le_self() {
    assert!(run_bool("tests/types/ord_algebra_vector_le_self.wat"));
}

#[test]
fn algebra_vector_ge_self() {
    assert!(run_bool("tests/types/ord_algebra_vector_ge_self.wat"));
}

#[test]
fn algebra_vector_lt_self_is_false() {
    assert!(!run_bool("tests/types/ord_algebra_vector_lt_self_false.wat"));
}

#[test]
fn algebra_vector_distinct_atoms_have_some_order() {
    assert!(run_bool("tests/types/ord_algebra_vector_distinct_order.wat"));
}

// ─── Rejected types — Stone 245.8: intrinsic rejects at check time ────────
//
// Before Stone 245.8: the defclauses only covered i64/f64; non-orderable types
// passed the type-checker (same-type unify OK at check), then raised
// TypeMismatch at runtime (values_compare → None).
//
// After Stone 245.8: ordering is a relational intrinsic with an orderable-class
// gate. Non-orderable types (HashMap, HashSet, user enum, Struct, HolonAST)
// are rejected at CHECK TIME with TypeMismatch pointing at param "#1".
// Runtime `values_compare → None` remains the defense-in-depth backstop.

#[test]
fn hashmap_ord_raises_type_mismatch() {
    let err = run_expecting_check_error("tests/types/ord_hashmap.wat.bad");
    wat::assert_edn_matches_file!(err, "wat_arc148_ord_buildout__hashmap_ord_raises_type_mismatch.edn", "HashMap is not orderable: TypeMismatch");
}

#[test]
fn hashset_ord_raises_type_mismatch() {
    let err = run_expecting_check_error("tests/types/ord_hashset.wat.bad");
    wat::assert_edn_matches_file!(err, "wat_arc148_ord_buildout__hashset_ord_raises_type_mismatch.edn", "HashSet is not orderable: TypeMismatch");
}

#[test]
fn enum_ord_raises_type_mismatch() {
    let err = run_expecting_check_error("tests/types/ord_enum.wat.bad");
    wat::assert_edn_matches_file!(err, "wat_arc148_ord_buildout__enum_ord_raises_type_mismatch.edn", "user enum is not orderable: TypeMismatch");
}

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn struct_ord_raises_type_mismatch() {
    let err = run_expecting_check_error("tests/types/ord_struct.wat.bad");
    assert_eq!(err, r##"CheckErrors([CheckError { span: Span { file: "tests/types/ord_struct.wat.bad", line: 9, col: 6, end_line: 9, end_col: 19 }, kind: TypeMismatch { callee: ":wat::core::<", param: "#1", expected: "an orderable type (i64, u8, f64, String, bool, keyword, Instant, Duration, Vector<T>, Tuple<T…>, Option<T>, Result<T,E>)", got: ":my::Point" } }])"##);
}

#[ignore = "296-recapture-pending: golden asserts pre-stone-B rust-debug face; unlock: 296 recapture (.edn data-equality flip)"]
#[test]
fn unit_ord_raises_type_mismatch() {
    // Unit () — not in the orderable class; rejected at check (Stone 245.8:
    // the checker types `()` as unit, which the orderable gate excludes —
    // ordering a one-inhabitant type is meaningless). Same shape as the
    // five sibling rejection witnesses above.
    let err = run_expecting_check_error("tests/types/ord_unit.wat.bad");
    assert_eq!(err, r##"CheckErrors([CheckError { span: Span { file: "tests/types/ord_unit.wat.bad", line: 3, col: 4, end_line: 3, end_col: 17 }, kind: TypeMismatch { callee: ":wat::core::<", param: "#1", expected: "an orderable type (i64, u8, f64, String, bool, keyword, Instant, Duration, Vector<T>, Tuple<T…>, Option<T>, Result<T,E>)", got: ":()" } }])"##);
}

#[test]
fn holon_ast_ord_raises_type_mismatch() {
    let err = run_expecting_check_error("tests/types/ord_holon_ast.wat.bad");
    wat::assert_edn_matches_file!(err, "wat_arc148_ord_buildout__holon_ast_ord_raises_type_mismatch.edn", "HolonAST is not orderable: TypeMismatch");
}
