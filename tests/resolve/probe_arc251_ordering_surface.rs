//! Forward-proof probe — the instinct-faithful ordering surface (intueri-ratified).
//!
//! RED at HEAD: `sort` is UnknownFunction; `sort-by` rejects a key-fn (it wants a comparator).
//!
//! Run: `cargo test --release --test probe_arc251_ordering_surface`

use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside_value` and inspect the returned typed Vec<i64>.
//
// arc 296 Stone M: `call_beside_value` already returns `Result<Value, RuntimeError>` — not a
// `StartupError` chain — so the real (never-flattened) error type here is `RuntimeError`
// itself; both "wrong Value shape" arms (outer + per-element) are minted as the same
// `RuntimeErrorKind::TypeMismatch` the runtime itself raises for this shape (see
// `src/assertion.rs::eval_opt_string`).
fn eval_vec(fn_name: &str) -> Result<Vec<i64>, RuntimeError> {
    match call_beside_value(file!(), fn_name)? {
        Value::Vec(v) => v.iter().map(|x| match x {
            Value::i64(n) => Ok(*n),
            other => Err(RuntimeError::new(
                wat::rust_caller_span!(),
                RuntimeErrorKind::TypeMismatch {
                    op: fn_name.into(),
                    expected: "i64",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )),
        }).collect(),
        other => Err(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: fn_name.into(),
                expected: "Vector",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )),
    }
}

#[test]
fn c01_sort_natural() {
    assert_eq!(eval_vec(":user::c01").expect("eval_vec"), vec![1, 2, 3], "(sort xs) — natural ascending");
}

#[test]
fn c02_sort_comparator() {
    assert_eq!(eval_vec(":user::c02").expect("eval_vec"), vec![3, 2, 1], "(sort cmp xs) — comparator descending");
}

#[test]
fn c03_sort_by_key() {
    assert_eq!(eval_vec(":user::c03").expect("eval_vec"), vec![3, 2, 1], "(sort-by keyfn xs) — by key, natural");
}

#[test]
fn c04_sort_by_key_and_comparator() {
    assert_eq!(eval_vec(":user::c04").expect("eval_vec"), vec![3, 2, 1], "sort-by keyfn cmp xs — by key + comparator");
}

#[test]
fn c05_reverse_preserved() {
    assert_eq!(eval_vec(":user::c05").expect("eval_vec"), vec![3, 2, 1], "reverse unchanged");
}
