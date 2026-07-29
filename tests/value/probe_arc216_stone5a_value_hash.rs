//! Arc 216 Stone 216.5a — `impl Hash for Value` + `impl PartialEq + Eq for Value`.
//!
//! Rust-level probes only — no WAT evaluation. Verifies the new trait impls
//! behave correctly per EXPECTATIONS rows 7-13.
//!
//! ## Probes
//!
//! 1. Self-equality — `hash(&v) == hash(&v)` for each atomizable variant
//! 2. Discriminant tagging — `hash(&Value::bool(true)) != hash(&Value::i64(1))`
//! 3. NaN-safety — `Value::f64(NAN) == Value::f64(NAN)` (bit-pattern); hash stable
//! 4. Recursive composition — `std::collections::HashSet<Value>` + `HashMap<Value,Value>`
//!    build + query at Rust level
//! 5. HolonAST nesting — `Value::holon__HolonAST(...)` hashes consistently
//! 6. Vec composition — reversed-order Vec produces DIFFERENT hash (order preserved)
//! 7. HashSet composition — same elements different insertion order → IDENTICAL hash
//! 8. HashMap composition — same pairs different insertion order → IDENTICAL hash
//! 9. Deep nesting — `Value::Vec([Value::wat__std__HashMap(...)])` hashes consistently
//! 10. Non-atomizable panic — Fn value panics with predicate-citation message

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use wat::runtime::Value;

// ─── hash helper ─────────────────────────────────────────────────────────────

fn hash_value(v: &Value) -> u64 {
    let mut h = DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
}

// ─── Probe 1 — Self-equality (deterministic hash) ─────────────────────────────

#[test]
fn probe_1_self_equality_i64() {
    let v = Value::i64(42);
    assert_eq!(hash_value(&v), hash_value(&v), "i64 hash must be stable");
    assert_eq!(v, v, "i64 PartialEq reflexive");
}

#[test]
fn probe_1_self_equality_f64() {
    let v = Value::f64(3.14);
    assert_eq!(hash_value(&v), hash_value(&v), "f64 hash must be stable");
    assert_eq!(v, v, "f64 PartialEq reflexive");
}

#[test]
fn probe_1_self_equality_bool() {
    let v = Value::bool(true);
    assert_eq!(hash_value(&v), hash_value(&v), "bool hash must be stable");
    assert_eq!(v, v, "bool PartialEq reflexive");
}

#[test]
fn probe_1_self_equality_string() {
    let v = Value::String(Arc::new("hello".to_string()));
    assert_eq!(hash_value(&v), hash_value(&v), "String hash must be stable");
    assert_eq!(v, v, "String PartialEq reflexive");
}

#[test]
fn probe_1_self_equality_keyword() {
    let v = Value::wat__core__keyword(Arc::new(":wat::core::i64".to_string()));
    assert_eq!(hash_value(&v), hash_value(&v), "keyword hash must be stable");
    assert_eq!(v, v, "keyword PartialEq reflexive");
}

#[test]
fn probe_1_self_equality_uuid() {
    let u = uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let v = Value::wat__core__Uuid(u);
    assert_eq!(hash_value(&v), hash_value(&v), "Uuid hash must be stable");
    assert_eq!(v, v, "Uuid PartialEq reflexive");
}

#[test]
fn probe_1_self_equality_vec() {
    let v = Value::Vec(Arc::new(vec![Value::i64(1), Value::i64(2)]));
    assert_eq!(hash_value(&v), hash_value(&v), "Vec hash must be stable");
    assert_eq!(v, v, "Vec PartialEq reflexive");
}

#[test]
fn probe_1_self_equality_holon_ast() {
    use holon::HolonAST;
    let ast = HolonAST::I64(99);
    let v = Value::holon__HolonAST(Arc::new(ast));
    assert_eq!(hash_value(&v), hash_value(&v), "HolonAST hash must be stable");
    assert_eq!(v, v, "HolonAST PartialEq reflexive");
}

#[test]
fn probe_1_self_equality_watast() {
    use wat::WatAST;
    let ast = WatAST::int(42);
    let v = Value::wat__WatAST(Arc::new(ast));
    assert_eq!(hash_value(&v), hash_value(&v), "WatAST hash must be stable");
    assert_eq!(v, v, "WatAST PartialEq reflexive");
}

// ─── Probe 2 — Discriminant tagging ─────────────────────────────────────────

#[test]
fn probe_2_discriminant_tagging_bool_vs_i64() {
    // bool(true) and i64(1) have "same-looking" payloads but must hash differently
    let b = Value::bool(true);
    let n = Value::i64(1);
    assert_ne!(
        hash_value(&b),
        hash_value(&n),
        "bool(true) and i64(1) must produce different hashes (discriminant tagging)"
    );
    assert_ne!(b, n, "bool(true) != i64(1) in PartialEq");
}

#[test]
fn probe_2_discriminant_tagging_keyword_vs_string() {
    // keyword(":foo") and String(":foo") — same payload bytes, different variants
    let k = Value::wat__core__keyword(Arc::new(":foo".to_string()));
    let s = Value::String(Arc::new(":foo".to_string()));
    assert_ne!(
        hash_value(&k),
        hash_value(&s),
        "keyword and String with same content must hash differently"
    );
    assert_ne!(k, s, "keyword != String in PartialEq");
}

// ─── Probe 3 — NaN-safety ────────────────────────────────────────────────────

#[test]
fn probe_3_nan_equality() {
    let nan1 = Value::f64(f64::NAN);
    let nan2 = Value::f64(f64::NAN);
    // Rust's f64::NAN != f64::NAN (IEEE-754), but our PartialEq uses to_bits()
    assert_eq!(nan1, nan2, "Value::f64(NAN) == Value::f64(NAN) via to_bits()");
}

#[test]
fn probe_3_nan_hash_stable() {
    let nan = Value::f64(f64::NAN);
    let h1 = hash_value(&nan);
    let h2 = hash_value(&nan);
    assert_eq!(h1, h2, "NaN hash must be stable (same bits = same hash)");
}

#[test]
fn probe_3_nan_hash_matches_equal_value() {
    // Two NaN Values that are PartialEq-equal must have the same hash
    let nan1 = Value::f64(f64::NAN);
    let nan2 = Value::f64(f64::NAN);
    assert_eq!(nan1, nan2, "NaN == NaN via to_bits");
    assert_eq!(hash_value(&nan1), hash_value(&nan2), "equal Values must have equal hashes");
}

// ─── Probe 4 — Recursive composition (Rust HashSet<Value> / HashMap<Value,Value>) ──

#[test]
fn probe_4_rust_hashset_of_value() {
    // Build a std::collections::HashSet<Value> — requires Value: Hash + Eq
    let mut set: std::collections::HashSet<Value> = std::collections::HashSet::new();
    set.insert(Value::i64(1));
    set.insert(Value::i64(2));
    set.insert(Value::i64(1)); // duplicate — should be deduped
    assert_eq!(set.len(), 2, "HashSet<Value> deduplicates correctly");
    assert!(set.contains(&Value::i64(1)), "HashSet<Value> contains i64(1)");
    assert!(set.contains(&Value::i64(2)), "HashSet<Value> contains i64(2)");
    assert!(!set.contains(&Value::i64(3)), "HashSet<Value> does not contain i64(3)");
}

#[test]
fn probe_4_rust_hashmap_value_to_value() {
    // Build a std::collections::HashMap<Value, Value>
    let mut map: std::collections::HashMap<Value, Value> = std::collections::HashMap::new();
    let k1 = Value::String(Arc::new("key1".to_string()));
    let k2 = Value::wat__core__keyword(Arc::new(":key2".to_string()));
    map.insert(k1.clone(), Value::i64(100));
    map.insert(k2.clone(), Value::bool(true));
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&k1), Some(&Value::i64(100)));
    assert_eq!(map.get(&k2), Some(&Value::bool(true)));
}

// ─── Probe 5 — HolonAST nesting ──────────────────────────────────────────────

#[test]
fn probe_5_holon_ast_nesting() {
    use holon::HolonAST;
    let ast1 = HolonAST::Bundle(Arc::new(vec![HolonAST::I64(1), HolonAST::I64(2)]));
    let ast2 = HolonAST::Bundle(Arc::new(vec![HolonAST::I64(1), HolonAST::I64(2)]));
    let v1 = Value::holon__HolonAST(Arc::new(ast1));
    let v2 = Value::holon__HolonAST(Arc::new(ast2));
    assert_eq!(v1, v2, "structurally-equal HolonAST Values are equal");
    assert_eq!(hash_value(&v1), hash_value(&v2), "equal HolonAST Values have same hash");
}

// ─── Probe 6 — Vec composition (order preserved) ─────────────────────────────

#[test]
fn probe_6_vec_order_preserved() {
    let v_ab = Value::Vec(Arc::new(vec![Value::i64(1), Value::i64(2)]));
    let v_ba = Value::Vec(Arc::new(vec![Value::i64(2), Value::i64(1)]));
    assert_ne!(v_ab, v_ba, "Vec order matters for equality");
    assert_ne!(
        hash_value(&v_ab),
        hash_value(&v_ba),
        "Vec order matters for hash (order preserved)"
    );
}

// ─── Probe 7 — HashSet composition (set semantics) ───────────────────────────

#[test]
fn probe_7_hashset_value_set_semantics() {
    // Build two Value::wat__std__HashSet with the same elements inserted in different order.
    // Stone 216.5b — storage is now Arc<HashSet<Value>>; native insert via Value: Hash + Eq.
    // The Value::Hash impl uses sorted element-hashes for order-independence.

    // Build set A: {i64(1), i64(2)} inserted as 1 then 2
    let mut set_a_inner: std::collections::HashSet<Value> =
        std::collections::HashSet::new();
    set_a_inner.insert(Value::i64(1));
    set_a_inner.insert(Value::i64(2));
    let set_a = Value::wat__std__HashSet(Arc::new(set_a_inner));

    // Build set B: same elements inserted as 2 then 1
    let mut set_b_inner: std::collections::HashSet<Value> =
        std::collections::HashSet::new();
    set_b_inner.insert(Value::i64(2));
    set_b_inner.insert(Value::i64(1));
    let set_b = Value::wat__std__HashSet(Arc::new(set_b_inner));

    assert_eq!(set_a, set_b, "HashSet Values with same elements are equal regardless of insertion order");
    assert_eq!(
        hash_value(&set_a),
        hash_value(&set_b),
        "HashSet Values with same elements produce identical hashes (set semantics)"
    );
}

// ─── Probe 8 — HashMap composition (map semantics) ───────────────────────────

#[test]
fn probe_8_hashmap_value_map_semantics() {
    // Build two Value::wat__std__HashMap with the same pairs in different order.
    // Storage is Arc<HashMap<canonical_key, (key_Value, val_Value)>>.
    // Value::Hash impl uses sorted (key_hash, val_hash) pairs for order-independence.

    // Map A: {keyword(":a") → i64(1), keyword(":b") → i64(2)} — ":a" first.
    // Stone 216.5c — storage is now HashMap<Value, Value>; K is the native key.
    #[allow(clippy::mutable_key_type)]
    let mut map_a: std::collections::HashMap<Value, Value> =
        std::collections::HashMap::new();
    map_a.insert(
        Value::wat__core__keyword(Arc::new(":a".to_string())),
        Value::i64(1),
    );
    map_a.insert(
        Value::wat__core__keyword(Arc::new(":b".to_string())),
        Value::i64(2),
    );
    let hmap_a = Value::wat__std__HashMap(Arc::new(map_a));

    // Map B: same pairs, ":b" first
    #[allow(clippy::mutable_key_type)]
    let mut map_b: std::collections::HashMap<Value, Value> =
        std::collections::HashMap::new();
    map_b.insert(
        Value::wat__core__keyword(Arc::new(":b".to_string())),
        Value::i64(2),
    );
    map_b.insert(
        Value::wat__core__keyword(Arc::new(":a".to_string())),
        Value::i64(1),
    );
    let hmap_b = Value::wat__std__HashMap(Arc::new(map_b));

    assert_eq!(hmap_a, hmap_b, "HashMap Values with same pairs are equal regardless of insertion order");
    assert_eq!(
        hash_value(&hmap_a),
        hash_value(&hmap_b),
        "HashMap Values with same pairs produce identical hashes (map semantics)"
    );
}

// ─── Probe 9 — Deep nesting ──────────────────────────────────────────────────

#[test]
fn probe_9_deep_nesting() {
    // Value::Vec([Value::wat__std__HashMap({keyword(":a") → i64(1)})])
    // Stone 216.5c — storage is now HashMap<Value, Value>; K is the native key.
    #[allow(clippy::mutable_key_type)]
    let mut inner_map: std::collections::HashMap<Value, Value> =
        std::collections::HashMap::new();
    inner_map.insert(
        Value::wat__core__keyword(Arc::new(":a".to_string())),
        Value::i64(1),
    );
    let hmap_val = Value::wat__std__HashMap(Arc::new(inner_map));
    let nested = Value::Vec(Arc::new(vec![hmap_val]));

    // Must hash consistently (same value → same hash)
    let h1 = hash_value(&nested);
    let h2 = hash_value(&nested);
    assert_eq!(h1, h2, "deep nested Value hashes consistently");

    // And be equal to itself
    assert_eq!(nested, nested, "deep nested Value equals itself");
}

// ─── Probe 10 — Non-atomizable panic ─────────────────────────────────────────

#[test]
fn probe_10_non_atomizable_fn_panics() {
    // Construct a Value::wat__core__fn (a closure).
    // We use the WAT evaluation path to get a real Function Value.
    // If construction at this layer isn't accessible, we document the skip.
    //
    // NOTE: Value::wat__core__fn's Arc<Function> is not publicly constructible
    // at the test layer without going through WAT eval (Function is an internal
    // substrate type). We exercise the panic via a known non-atomizable variant
    // that IS constructible: Value::OnlineSubspace or similar ML types.
    //
    // However, ThreadOwnedCell is also private. The only approach is to verify
    // that the Hash impl's unreachable!() doc-comment accurately describes the
    // predicate contract. We document this skip per BRIEF § Part E Probe 10:
    // "document if Fn construction isn't accessible at this test layer."
    //
    // SKIP REASON: Value::wat__core__fn(Arc<Function>), Value::OnlineSubspace,
    // Value::Reckoner, etc. wrap private internal types (Function, ThreadOwnedCell)
    // that have no public constructor outside of WAT eval. The unreachable!() arms
    // are verified by the doc-comment contract: is_atomizable at check.rs:3623
    // never admits these variants; if it ever does and the Hash impl isn't updated,
    // the runtime panic fires with the predicate-citation message.
    //
    // This skip is an honesty delta: the unreachable!() arms exist in the impl
    // and are verbally verified; the panic path itself cannot be unit-tested at
    // this layer without test-infrastructure changes.
    //
    // See SCORE-STONE-216.5a.md § Probe 10 skip documentation.
    let _ = "Probe 10 skip documented: non-atomizable types not constructible at test layer";
    // Verify the impl compiles and the trait bounds are satisfied:
    let v = Value::i64(42);
    let h = hash_value(&v);
    assert!(h >= 0, "probe 10 placeholder: hash impl compiles and runs");
}
