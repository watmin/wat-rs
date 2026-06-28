//! Forward-proof probe — the instinct-faithful ordering surface (intueri-ratified).
//!
//! RED at HEAD: `sort` is UnknownFunction; `sort-by` rejects a key-fn (it wants a comparator).
//!
//! Run: `cargo test --release --test probe_arc251_ordering_surface`

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

fn eval_vec(world: &wat::freeze::FrozenWorld, call: &str) -> Result<Vec<i64>, String> {
    let ast = wat::parse_one!(call).expect("parse");
    match eval_in_frozen(&ast, world, &Environment::new())
        .map(|tv| tv.value_owned())
        .map_err(|e| format!("eval: {e:?}"))?
    {
        Value::Vec(v) => v.iter().map(|x| match x {
            Value::i64(n) => Ok(*n),
            other => Err(format!("non-i64 elem: {other:?}")),
        }).collect(),
        other => Err(format!("non-vec: {other:?}")),
    }
}

#[test]
fn c01_sort_natural() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_vec(&world, "(:user::c01)"), Ok(vec![1, 2, 3]), "(sort xs) — natural ascending");
}

#[test]
fn c02_sort_comparator() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_vec(&world, "(:user::c02)"), Ok(vec![3, 2, 1]), "(sort cmp xs) — comparator descending");
}

#[test]
fn c03_sort_by_key() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_vec(&world, "(:user::c03)"), Ok(vec![3, 2, 1]), "(sort-by keyfn xs) — by key, natural");
}

#[test]
fn c04_sort_by_key_and_comparator() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_vec(&world, "(:user::c04)"), Ok(vec![3, 2, 1]), "(sort-by keyfn cmp xs)");
}

#[test]
fn c05_reverse_preserved() {
    let world = startup_beside(file!()).expect("startup");
    assert_eq!(eval_vec(&world, "(:user::c05)"), Ok(vec![3, 2, 1]), "reverse unchanged");
}
