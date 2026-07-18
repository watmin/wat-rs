//! Forward-proof probe — the instinct-faithful ordering surface (intueri-ratified).
//!
//! RED at HEAD: `sort` is UnknownFunction; `sort-by` rejects a key-fn (it wants a comparator).
//!
//! Run: `cargo test --release --test probe_arc251_ordering_surface`

use wat::freeze::call_beside;
use wat::runtime::Value;

// just-eval (rubric): each `:user::cNN` zero-arg fn lives in the co-located fixture;
// drive it via `call_beside` and inspect the returned typed Vec<i64>.
fn eval_vec(fn_name: &str) -> Result<Vec<i64>, String> {
    match call_beside(file!(), fn_name).map_err(|e| format!("eval: {e:?}"))? {
        Value::Vec(v) => v.iter().map(|x| match x {
            Value::i64(n) => Ok(*n),
            other => Err(format!("non-i64 elem: {other:?}")),
        }).collect(),
        other => Err(format!("non-vec: {other:?}")),
    }
}

#[test]
fn c01_sort_natural() {
    assert_eq!(eval_vec(":user::c01"), Ok(vec![1, 2, 3]), "(sort xs) — natural ascending");
}

#[test]
fn c02_sort_comparator() {
    assert_eq!(eval_vec(":user::c02"), Ok(vec![3, 2, 1]), "(sort cmp xs) — comparator descending");
}

#[test]
fn c03_sort_by_key() {
    assert_eq!(eval_vec(":user::c03"), Ok(vec![3, 2, 1]), "(sort-by keyfn xs) — by key, natural");
}

#[test]
fn c04_sort_by_key_and_comparator() {
    assert_eq!(eval_vec(":user::c04"), Ok(vec![3, 2, 1]), "sort-by keyfn cmp xs — by key + comparator");
}

#[test]
fn c05_reverse_preserved() {
    assert_eq!(eval_vec(":user::c05"), Ok(vec![3, 2, 1]), "reverse unchanged");
}
