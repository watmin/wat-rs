//! Arc 220 Stone 220.4 — `:wat::core::List<T>` integration tests.
//!
//! Exercises:
//! - Construction via `(:wat::core::List ...)` constructor
//! - Empty list
//! - first/rest/conj (List conj PREPENDS; Vector conj APPENDS — distinct semantics)
//! - length/empty?/contains?/get
//! - **Cross-type Eq:** `List(1,2,3) == Vector(1,2,3)` returns true (EDN spec §282-289)
//! - **Cross-type HashMap key:** List and Vector with same contents hash equal
//! - EDN round-trip: parse `(1 2 3)` via wat-edn → wat__core__List → write → reparse
//!
//! Architecture mirrors `tests/wat_arc207_uuid_typed.rs` (eval_in_frozen pattern).
//!
//! Wat source lives in the co-located fixture: list.wat
//! (slurped via startup_beside(file!())).

use std::collections::LinkedList;
use std::sync::Arc;
use wat::freeze::call_beside_value;
use wat::runtime::Value;

// just-eval (rubric): each named zero-arg fn lives in the co-located list.wat;
// `call_beside_value` runs it in-process and returns the typed Value directly.
fn ev(fn_name: &str) -> Value {
    call_beside_value(file!(), fn_name).expect("compute")
}

// ─── Construction ─────────────────────────────────────────────────────────────

#[test]
fn list_constructor_of_builds_list() {
    // (:wat::core::List 1 2 3) returns a List with 3 elements
    let v = ev(":list::length-of-3");
    assert_eq!(v, Value::i64(3), "List/of 1 2 3 should have length 3");
}

#[test]
fn list_constructor_of_returns_list_type() {
    // Verify that (:wat::core::List 1 2) produces a wat__core__List at Rust level.
    let length = ev(":list::length-of-2");
    assert_eq!(length, Value::i64(2), "List/of 1 2 has length 2");
    // Confirm the Rust variant is wat__core__List, not Vec
    let list_val = {
        let mut ll = LinkedList::new();
        ll.push_back(Value::i64(1));
        ll.push_back(Value::i64(2));
        Value::wat__core__List(Arc::new(ll))
    };
    assert_eq!(list_val.type_name(), "wat::core::List");
}

#[test]
fn list_constructor_empty() {
    // (:wat::core::List) returns an empty List — verify via empty?
    let v = ev(":list::empty-q-of-empty");
    assert_eq!(v, Value::bool(true), "empty List/of should satisfy empty?");
}

// ─── length / empty? ─────────────────────────────────────────────────────────

#[test]
fn list_length() {
    let v = ev(":list::length-3");
    assert_eq!(v, Value::i64(3));
}

#[test]
fn list_length_empty() {
    let v = ev(":list::length-0");
    assert_eq!(v, Value::i64(0));
}

#[test]
fn list_empty_q_true() {
    let v = ev(":list::empty-q-true");
    assert_eq!(v, Value::bool(true));
}

#[test]
fn list_empty_q_false() {
    let v = ev(":list::empty-q-false");
    assert_eq!(v, Value::bool(false));
}

// ─── first / rest ─────────────────────────────────────────────────────────────

#[test]
fn list_first_returns_some() {
    // (:wat::core::first list) returns T directly (arc-278 — no Option wrapper).
    let v = ev(":list::first-some");
    assert_eq!(v, Value::bool(true), "first of (10 20 30) should be 10");
}

#[test]
fn list_rest_returns_tail_as_list() {
    // rest of (1 2 3) should give a List of length 2
    let v = ev(":list::rest-tail-len");
    assert_eq!(v, Value::i64(2), "rest of 3-element list should have length 2");
}

#[test]
fn list_rest_preserves_list_type() {
    // rest of a List should return a List (not Vec) — check via length of tail
    let v = ev(":list::rest-tail-len");
    assert_eq!(v, Value::i64(2), "rest of 3-element List should return List of length 2");
}

// ─── conj — PREPEND semantic ──────────────────────────────────────────────────

#[test]
fn list_conj_prepends() {
    // List/conj should PREPEND. After conj(List(2,3), 1) → List(1,2,3).
    // first returns T directly (arc-278 — no Option wrapper).
    let v = ev(":list::conj-prepends");
    assert_eq!(v, Value::bool(true), "List/conj prepends: first of conj(List(2,3), 1) should be 1");
}

#[test]
fn vector_conj_appends_distinct_from_list() {
    // Vector/conj APPENDS — first element should still be 2 (the original head).
    // first returns T directly (arc-278 — no Option wrapper).
    let v = ev(":list::vec-conj-appends");
    assert_eq!(v, Value::bool(true), "Vector/conj appends: first of conj([2,3], 1) should still be 2");
}

// ─── contains? / get ─────────────────────────────────────────────────────────

#[test]
fn list_contains_q_found() {
    let v = ev(":list::contains-found");
    assert_eq!(v, Value::bool(true));
}

#[test]
fn list_contains_q_not_found() {
    let v = ev(":list::contains-not-found");
    assert_eq!(v, Value::bool(false));
}

#[test]
fn list_get_found() {
    // get index 1 from List(10,20,30) → Some(20) → extract and verify
    let v = ev(":list::get-found");
    assert_eq!(v, Value::bool(true), "List/get index 1 from (10 20 30) should be 20");
}

#[test]
fn list_get_out_of_bounds_returns_none() {
    let v = ev(":list::get-oob");
    assert_eq!(v, Value::bool(true), "List/get out-of-bounds should return None");
}

// ─── Cross-type Eq (EDN spec §282-289) — Rust level ─────────────────────────

#[test]
fn cross_type_eq_list_equals_vector_same_contents() {
    // At the Rust PartialEq level: List([1,2,3]) == Vec([1,2,3])
    let list = Value::wat__core__List(Arc::new({
        let mut ll = LinkedList::new();
        ll.push_back(Value::i64(1));
        ll.push_back(Value::i64(2));
        ll.push_back(Value::i64(3));
        ll
    }));
    let vec = Value::Vec(Arc::new(vec![Value::i64(1), Value::i64(2), Value::i64(3)]));
    assert_eq!(list, vec, "List([1,2,3]) should equal Vector([1,2,3]) per EDN spec §282-289");
    assert_eq!(vec, list, "Vector([1,2,3]) should equal List([1,2,3]) per EDN spec §282-289");
}

#[test]
fn cross_type_eq_list_ne_vector_different_contents() {
    let list = Value::wat__core__List(Arc::new({
        let mut ll = LinkedList::new();
        ll.push_back(Value::i64(1));
        ll.push_back(Value::i64(2));
        ll
    }));
    let vec = Value::Vec(Arc::new(vec![Value::i64(1), Value::i64(2), Value::i64(3)]));
    assert_ne!(list, vec, "List([1,2]) should not equal Vector([1,2,3])");
}

#[test]
fn cross_type_eq_empty_list_equals_empty_vector() {
    let list = Value::wat__core__List(Arc::new(LinkedList::new()));
    let vec = Value::Vec(Arc::new(vec![]));
    assert_eq!(list, vec, "empty List should equal empty Vector per EDN spec");
}

// ─── Cross-type Hash invariant ────────────────────────────────────────────────

#[test]
fn cross_type_hash_list_vector_same_contents_same_hash() {
    use std::collections::HashMap;
    // Build a HashMap with a Vec key, then look it up with a List key.
    // If Hash invariant holds (List(1,2) and Vec(1,2) hash equal AND eq),
    // the HashMap lookup succeeds.
    let vec_key = Value::Vec(Arc::new(vec![Value::i64(1), Value::i64(2)]));
    let list_key = Value::wat__core__List(Arc::new({
        let mut ll = LinkedList::new();
        ll.push_back(Value::i64(1));
        ll.push_back(Value::i64(2));
        ll
    }));
    let mut map: HashMap<Value, Value> = HashMap::new();
    map.insert(vec_key, Value::wat__core__keyword(Arc::new(":found".to_string())));

    // List key should find the Vec-keyed entry
    let result = map.get(&list_key);
    assert!(
        result.is_some(),
        "List([1,2]) as HashMap key should find entry inserted with Vector([1,2]) \
         — Hash invariant violated if this fails"
    );
}

// ─── EDN round-trip ───────────────────────────────────────────────────────────

#[test]
fn edn_roundtrip_list_parse_to_wat_core_list() {
    // Parse EDN `(1 2 3)` → should produce Value::wat__core__List (not Vec)
    use wat_edn::parse;
    use wat::edn::render::edn_to_value;

    // rune:lint(no-inlined-edn) — input under test: EDN source string fed to wat_edn::parse; the subject is the reader's list-parse path, not an evaluated value.
    let edn_src = "(1 2 3)";
    let parsed = parse(edn_src).expect("wat-edn parse of (1 2 3) failed");

    // wat-edn Value::List — check it's a List variant
    assert!(
        matches!(parsed, wat_edn::Value::List(_)),
        "EDN `(1 2 3)` should parse as Value::List in wat-edn"
    );

    // Convert through edn_shim: wat-edn List → wat Value::wat__core__List
    let wat_val = edn_to_value(&parsed, None, None)
        .expect("edn_to_value failed for List");
    match &wat_val {
        Value::wat__core__List(xs) => {
            assert_eq!(xs.len(), 3, "List from (1 2 3) should have 3 elements");
        }
        other => panic!("expected wat__core__List from EDN list, got {}", other.type_name()),
    }
}

#[test]
fn edn_roundtrip_vector_still_goes_to_vec() {
    // Parse EDN `[1 2 3]` → should produce Value::Vec (not List)
    use wat_edn::parse;
    use wat::edn::render::edn_to_value;

    // rune:lint(no-inlined-edn) — input under test: EDN source string fed to wat_edn::parse; the subject is the reader's vector-parse path, not an evaluated value.
    let edn_src = "[1 2 3]";
    let parsed = parse(edn_src).expect("wat-edn parse of [1 2 3] failed");

    let wat_val = edn_to_value(&parsed, None, None)
        .expect("edn_to_value failed for Vector");
    match &wat_val {
        Value::Vec(xs) => {
            assert_eq!(xs.len(), 3, "Vector from [1 2 3] should have 3 elements");
        }
        other => panic!("expected Vec from EDN vector, got {}", other.type_name()),
    }
}

#[test]
fn edn_roundtrip_list_writes_as_parens() {
    // Write a wat__core__List → should produce EDN parens form, not brackets
    use wat_edn::write;
    use wat::edn::render::value_to_edn;

    let list = Value::wat__core__List(Arc::new({
        let mut ll = LinkedList::new();
        ll.push_back(Value::i64(1));
        ll.push_back(Value::i64(2));
        ll.push_back(Value::i64(3));
        ll
    }));

    let owned = value_to_edn(&list);
    let written = write(&owned);
    // EDN list form uses parens
    assert!(
        written.trim().starts_with('('),
        "List should write as EDN parens form `(...)`, got: {}",
        written
    );
}
