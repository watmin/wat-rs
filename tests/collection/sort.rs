//! Arc 251 Stone — `:wat::core::sort` (migrated from `sort_by.rs`).
//!
//! The comparator-sort (`sort-by` in Arc 056/247) is now the 2-ary `sort` clause:
//! `(sort cmp xs)` where `cmp : (T,T)->bool`. Identical arg shape — only the
//! function name changed (the old `sort-by` was Clojure's `sort` mis-named).
//!
//! Wat source lives in the co-located fixture: sort.wat, driven via
//! call_beside(file!(), fn_name). Functions return their results as
//! String/i64 so tests inspect the returned Value rather than stdout capture.

use wat::freeze::call_beside;
use wat::runtime::Value;

fn run_str(fn_name: &str) -> String {
    match call_beside(file!(), fn_name).expect("eval") {
        Value::String(s) => (*s).clone(),
        other => panic!("expected String; got {other:?}"),
    }
}

fn run_i64(fn_name: &str) -> i64 {
    match call_beside(file!(), fn_name).expect("eval") {
        Value::i64(n) => n,
        other => panic!("expected i64; got {other:?}"),
    }
}

#[test]
fn sort_ascending_i64() {
    assert_eq!(run_str(":sort::ascending-i64"), "1,1,2,3,4,5,6,9");
}

#[test]
fn sort_descending_f64() {
    assert_eq!(run_str(":sort::descending-f64"), "2.5,1.5,1,0.5");
}

#[test]
fn sort_string() {
    assert_eq!(run_str(":sort::string-asc"), "apple,banana,cherry");
}

#[test]
fn sort_empty_vec() {
    assert_eq!(run_i64(":sort::empty-length"), 0);
}

#[test]
fn sort_tuple_first_field_key() {
    assert_eq!(run_str(":sort::tuple-first-field"), "carol,bob,alice");
}
