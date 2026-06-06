//! Perimeter-closure probes for the collection dispatch home.
//!
//! ## Coverage
//!
//! **Item 1 (regression)** — `eval_length` and `eval_empty` polymorphic dispatch now
//! includes `Value::wat__core__List`. Probes witness a `List<T>` flowing through
//! `(:wat::core::length ...)` and `(:wat::core::empty? ...)` end-to-end (check + runtime).
//!
//! **Items 4 + 5** — active witnesses for the five previously-unwitnessed
//! `transform.rs` ops and Vector conj immutability.
//!
//! ### Five transform ops (Item 4)
//!
//! - `eval_vec_zip` (`:wat::std::list::zip`)
//! - `eval_vec_window` (`:wat::std::list::window`)
//! - `eval_vec_remove_at` (`:wat::std::list::remove-at`)
//! - `eval_vec_map_with_index` (`:wat::std::list::map-with-index`)
//! - `eval_vec_find_last_index` (`:wat::core::find-last-index`)
//!
//! ### Vector conj immutability (Item 5)
//!
//! Witnesses that `(:wat::core::Vector/conj v0 x)` does not mutate `v0`
//! (analogous to the HashSet witness in `probe_arc216_stone5b_hashset_native_storage.rs`).

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// ─── helpers ─────────────────────────────────────────────────────────────────

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

fn run_i64(src: &str) -> i64 {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    match eval_in_frozen(&ast, &world, &env)
        .expect("compute")
        .value_owned()
    {
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
    match eval_in_frozen(&ast, &world, &env)
        .expect("compute")
        .value_owned()
    {
        Value::bool(b) => b,
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Item 1 — List<T> through polymorphic length + empty? ────────────────────

/// `(:wat::core::length list)` accepts a `List<T>` value — check + runtime.
/// Regresses the gap: before the fix, `eval_length` lacked the
/// `Value::wat__core__List` arm and would TypeMismatch at runtime.
#[test]
fn item1_list_length_polymorphic() {
    // Non-empty list: length = 3.
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::length
            (:wat::core::List/of 10 20 30)))
    "#);
    assert_eq!(n, 3, "length of List<i64>(10,20,30) must be 3");

    // Empty list: length = 0.
    let n2 = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::length (:wat::core::List/of)))
    "#);
    assert_eq!(n2, 0, "length of empty List must be 0");
}

/// `(:wat::core::empty? list)` accepts a `List<T>` value — check + runtime.
/// Regresses the gap: before the fix, `eval_empty` lacked the
/// `Value::wat__core__List` arm and would TypeMismatch at runtime.
#[test]
fn item1_list_empty_q_polymorphic() {
    // Non-empty list → false.
    assert!(
        !run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::empty? (:wat::core::List/of 1 2 3)))
    "#),
        "empty? on non-empty List must be false"
    );

    // Empty list → true.
    assert!(
        run_bool(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::bool
          (:wat::core::empty? (:wat::core::List/of)))
    "#),
        "empty? on empty List must be true"
    );
}

// ─── Item 4 — zip ────────────────────────────────────────────────────────────

/// `(:wat::std::list::zip xs ys)` happy path: paired elements as Tuples.
/// zip([1,2,3], [4,5,6]) → [(1,4),(2,5),(3,6)]; length = 3.
#[test]
fn item4_zip_happy_path() {
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
            [zipped (:wat::std::list::zip
                       (:wat::core::Vector :wat::core::i64 1 2 3)
                       (:wat::core::Vector :wat::core::i64 4 5 6))]
            (:wat::core::Vector/length zipped)))
    "#);
    assert_eq!(n, 3, "zip of two 3-element vectors must have length 3");
}

/// `(:wat::std::list::zip xs ys)` boundary: empty input → empty output.
#[test]
fn item4_zip_empty_input() {
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
            [zipped (:wat::std::list::zip
                       (:wat::core::Vector :wat::core::i64)
                       (:wat::core::Vector :wat::core::i64 1 2 3))]
            (:wat::core::Vector/length zipped)))
    "#);
    assert_eq!(n, 0, "zip with empty first vector must produce empty output");
}

// ─── Item 4 — window ─────────────────────────────────────────────────────────

/// `(:wat::std::list::window xs n)` happy path: sliding windows of size 2.
/// window([1,2,3,4], 2) → [[1,2],[2,3],[3,4]]; outer length = 3.
#[test]
fn item4_window_happy_path() {
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::Vector/length
            (:wat::std::list::window
               (:wat::core::Vector :wat::core::i64 1 2 3 4)
               2)))
    "#);
    assert_eq!(n, 3, "window size 2 on 4-element vector must produce 3 windows");
}

/// `(:wat::std::list::window xs n)` boundary: n > len → empty output.
#[test]
fn item4_window_n_greater_than_len() {
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::Vector/length
            (:wat::std::list::window
               (:wat::core::Vector :wat::core::i64 1 2)
               5)))
    "#);
    assert_eq!(n, 0, "window size > len must produce empty output (no full window fits)");
}

// ─── Item 4 — remove-at ──────────────────────────────────────────────────────

/// `(:wat::std::list::remove-at xs i)` happy path: removes element at index 1.
/// remove-at([10,20,30], 1) → [10,30]; length = 2.
#[test]
fn item4_remove_at_happy_path() {
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::Vector/length
            (:wat::std::list::remove-at
               (:wat::core::Vector :wat::core::i64 10 20 30)
               1)))
    "#);
    assert_eq!(n, 2, "remove-at index 1 of 3-element vector must yield length 2");
}

/// `(:wat::std::list::remove-at xs i)` boundary: out-of-range index returns Vec unchanged.
/// Documented contract: negative or i >= len is a no-op.
#[test]
fn item4_remove_at_out_of_range_unchanged() {
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::Vector/length
            (:wat::std::list::remove-at
               (:wat::core::Vector :wat::core::i64 10 20 30)
               99)))
    "#);
    assert_eq!(n, 3, "remove-at with out-of-range index must leave vector unchanged");
}

// ─── Item 4 — map-with-index ─────────────────────────────────────────────────

/// `(:wat::std::list::map-with-index xs f)` happy path: f receives (item, index).
/// map-with-index([10,20,30], fn (v,i) -> i) → [0,1,2]; sum = 3.
#[test]
fn item4_map_with_index_happy_path() {
    // Sum the indices to prove f receives the correct index values.
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::foldl
            (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
              (:wat::core::i64::+ acc x))
            0
            (:wat::std::list::map-with-index
              (:wat::core::Vector :wat::core::i64 10 20 30)
              (:wat::core::fn [_v <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64 i))))
    "#);
    assert_eq!(n, 3, "map-with-index indices must be 0,1,2 (sum = 3)");
}

/// `(:wat::std::list::map-with-index xs f)` boundary: empty input → empty output.
#[test]
fn item4_map_with_index_empty_input() {
    let n = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::Vector/length
            (:wat::std::list::map-with-index
              (:wat::core::Vector :wat::core::i64)
              (:wat::core::fn [v <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64 i))))
    "#);
    assert_eq!(n, 0, "map-with-index on empty vector must produce empty output");
}

// ─── Item 4 — find-last-index ────────────────────────────────────────────────

/// `(:wat::core::find-last-index xs pred)` happy path: finds rightmost match.
/// find-last-index([5,12,3,18,7], >10) → Some(3) (index of 18, the last x>10).
/// Unwrap via match to i64 to avoid Option<T> annotation gymnastics in defn.
#[test]
fn item4_find_last_index_happy_path() {
    // Unwrap the Option<i64> result to i64 (-1 sentinel for None) via match.
    let idx = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::match
            (:wat::core::find-last-index
              (:wat::core::Vector :wat::core::i64 5 12 3 18 7)
              (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool
                (:wat::core::i64::> x 10)))
            -> :wat::core::i64
            ((:wat::core::Some i) i)
            (:wat::core::None -1)))
    "#);
    assert_eq!(idx, 3, "rightmost match index must be 3 (the index of 18)");
}

/// `(:wat::core::find-last-index xs pred)` boundary: no match → None (sentinel -1).
#[test]
fn item4_find_last_index_no_match() {
    let idx = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::match
            (:wat::core::find-last-index
              (:wat::core::Vector :wat::core::i64 1 2 3)
              (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool
                (:wat::core::i64::> x 99)))
            -> :wat::core::i64
            ((:wat::core::Some i) i)
            (:wat::core::None -1)))
    "#);
    assert_eq!(idx, -1, "no match must return None (sentinel -1)");
}

// ─── Item 5 — Vector conj immutability ───────────────────────────────────────

/// `(:wat::core::Vector/conj v0 x)` does not mutate `v0`.
///
/// Mirrors the HashSet witness at probe_arc216_stone5b_hashset_native_storage.rs:215-222.
/// The persistent-collection contract: clone-then-new-Arc (functional, not mutating).
/// Witnesses: v0 retains its original length; v1 has the new element.
#[test]
fn item5_vector_conj_does_not_mutate_input() {
    // After conjing a new element onto v0, v0's length must be unchanged (2 → still 2).
    let original_len = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
            [v0 (:wat::core::Vector :wat::core::i64 1 2)
             _  (:wat::core::Vector/conj v0 3)]
            (:wat::core::Vector/length v0)))
    "#);
    assert_eq!(
        original_len,
        2,
        "conj must not mutate the input vector: v0 must still have length 2"
    );

    // v1 must contain the new element (length 3).
    let new_len = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
            [v0 (:wat::core::Vector :wat::core::i64 1 2)
             v1 (:wat::core::Vector/conj v0 3)]
            (:wat::core::Vector/length v1)))
    "#);
    assert_eq!(
        new_len,
        3,
        "conj must return a new vector of length 3 with the element appended"
    );

    // The new element must be accessible at the correct position.
    let new_elem = run_i64(r#"
        (:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::let
            [v0 (:wat::core::Vector :wat::core::i64 1 2)
             v1 (:wat::core::Vector/conj v0 99)]
            (:wat::core::match
              (:wat::core::Vector/get v1 2)
              -> :wat::core::i64
              ((:wat::core::Some x) x)
              (:wat::core::None -1))))
    "#);
    assert_eq!(
        new_elem,
        99,
        "conj must append the new element at the last position"
    );
}
