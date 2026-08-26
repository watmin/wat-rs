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
//! Witnesses that `(:wat::vec::conj v0 x)` does not mutate `v0`
//! (analogous to the HashSet witness in `probe_arc216_stone5b_hashset_native_storage.rs`).

use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): each `:t::item…` entry is a zero-arg fn in the co-located
// `.wat` fixture, driven via `call_beside_value` — no inline wat driver.

// ─── Item 1 — List<T> through polymorphic length + empty? ────────────────────

/// `(:wat::core::length list)` accepts a `List<T>` value — check + runtime.
#[test]
fn item1_list_length_polymorphic() {

    match call_beside_value(file!(), ":t::item1a-list-len-nonempty").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "length of List<i64>(10,20,30) must be 3"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::item1b-list-len-empty").expect("eval") {
        Value::i64(n) => assert_eq!(n, 0, "length of empty List must be 0"),
        other => panic!("expected i64; got {:?}", other),
    }
}

/// `(:wat::core::empty? list)` accepts a `List<T>` value — check + runtime.
#[test]
fn item1_list_empty_q_polymorphic() {

    match call_beside_value(file!(), ":t::item1c-list-empty-nonempty").expect("eval") {
        Value::bool(b) => assert!(!b, "empty? on non-empty List must be false"),
        other => panic!("expected bool; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::item1d-list-empty-empty").expect("eval") {
        Value::bool(b) => assert!(b, "empty? on empty List must be true"),
        other => panic!("expected bool; got {:?}", other),
    }
}

// ─── Item 4 — zip ────────────────────────────────────────────────────────────

/// `(:wat::std::list::zip xs ys)` happy path: length = 3.
#[test]
fn item4_zip_happy_path() {
    match call_beside_value(file!(), ":t::item4a-zip-happy-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "zip of two 3-element vectors must have length 3"),
        other => panic!("expected i64; got {:?}", other),
    }
}

/// `(:wat::std::list::zip xs ys)` boundary: empty input → empty output.
#[test]
fn item4_zip_empty_input() {
    match call_beside_value(file!(), ":t::item4b-zip-empty-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 0, "zip with empty first vector must produce empty output"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Item 4 — window ─────────────────────────────────────────────────────────

/// `(:wat::std::list::window xs n)` happy path: 3 windows.
#[test]
fn item4_window_happy_path() {
    match call_beside_value(file!(), ":t::item4c-window-happy-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "window size 2 on 4-element vector must produce 3 windows"),
        other => panic!("expected i64; got {:?}", other),
    }
}

/// `(:wat::std::list::window xs n)` boundary: n > len → empty output.
#[test]
fn item4_window_n_greater_than_len() {
    match call_beside_value(file!(), ":t::item4d-window-n-gt-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 0, "window size > len must produce empty output"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Item 4 — remove-at ──────────────────────────────────────────────────────

/// `(:wat::std::list::remove-at xs i)` happy path: length 2.
#[test]
fn item4_remove_at_happy_path() {
    match call_beside_value(file!(), ":t::item4e-remove-at-happy-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "remove-at index 1 of 3-element vector must yield length 2"),
        other => panic!("expected i64; got {:?}", other),
    }
}

/// `(:wat::std::list::remove-at xs i)` boundary: out-of-range index returns Vec unchanged.
#[test]
fn item4_remove_at_out_of_range_unchanged() {
    match call_beside_value(file!(), ":t::item4f-remove-at-oob-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "remove-at with out-of-range index must leave vector unchanged"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Item 4 — map-with-index ─────────────────────────────────────────────────

/// `(:wat::std::list::map-with-index xs f)` happy path: sum of indices = 3.
#[test]
fn item4_map_with_index_happy_path() {
    match call_beside_value(file!(), ":t::item4g-map-with-index-happy").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "map-with-index indices must be 0,1,2 (sum = 3)"),
        other => panic!("expected i64; got {:?}", other),
    }
}

/// `(:wat::std::list::map-with-index xs f)` boundary: empty input → empty output.
#[test]
fn item4_map_with_index_empty_input() {
    match call_beside_value(file!(), ":t::item4h-map-with-index-empty").expect("eval") {
        Value::i64(n) => assert_eq!(n, 0, "map-with-index on empty vector must produce empty output"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Item 4 — find-last-index ────────────────────────────────────────────────

/// `(:wat::core::find-last-index xs pred)` happy path: index of last x>10 = 3.
#[test]
fn item4_find_last_index_happy_path() {
    match call_beside_value(file!(), ":t::item4i-find-last-idx-happy").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "rightmost match index must be 3 (the index of 18)"),
        other => panic!("expected i64; got {:?}", other),
    }
}

/// `(:wat::core::find-last-index xs pred)` boundary: no match → None (sentinel -1).
#[test]
fn item4_find_last_index_no_match() {
    match call_beside_value(file!(), ":t::item4j-find-last-idx-none").expect("eval") {
        Value::i64(n) => assert_eq!(n, -1, "no match must return None (sentinel -1)"),
        other => panic!("expected i64; got {:?}", other),
    }
}

// ─── Item 5 — Vector conj immutability ───────────────────────────────────────

/// `(:wat::vec::conj v0 x)` does not mutate `v0`.
#[test]
fn item5_vector_conj_does_not_mutate_input() {

    match call_beside_value(file!(), ":t::item5a-conj-immutable-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 2, "conj must not mutate the input vector: v0 must still have length 2"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::item5b-conj-new-len").expect("eval") {
        Value::i64(n) => assert_eq!(n, 3, "conj must return a new vector of length 3 with the element appended"),
        other => panic!("expected i64; got {:?}", other),
    }

    match call_beside_value(file!(), ":t::item5c-conj-new-elem").expect("eval") {
        Value::i64(n) => assert_eq!(n, 99, "conj must append the new element at the last position"),
        other => panic!("expected i64; got {:?}", other),
    }
}
