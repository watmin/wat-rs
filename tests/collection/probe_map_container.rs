//! MapContainer registry — strike 5 net: every keyed-collection kind classifies
//! correctly and `assoc` round-trips through the registry on both sides (runtime +
//! checker). Mirrors `tests/probe_seq_container_registry.rs` in structure.
//!
//! BLACK-BOX: drives everything through the public wat API via wat programs.
//! No registry import — `MapContainer` is `pub(crate)` and not visible here.
//! The capability-table truth tests live as `#[cfg(test)]` unit tests INSIDE
//! `src/collection/map_container.rs` (same crate → `pub(crate)` visible).
//!
//! Probes:
//!   - `assoc` round-trips on HashMap (key present after assoc)
//!   - `assoc` round-trips on PersistentMap (length grows; original immutable)
//!   - `assoc` round-trips on a base Record (field-update + other field preserved)
//!   - `assoc` round-trips on a holonic Record (field-update)
//!   - A non-keyed value (Vector) → assoc TypeMismatch (type-check rejection)
//!
//! Run: cargo test --release -p wat --test probe_map_container

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Build a world from one probe `defn` + optional preamble, start it (type-check
/// fires here), then eval `call`. Mirrors the helper in probe_seq_container_registry.
fn eval_probe(preamble: &str, defn: &str, call: &str) -> Result<Value, String> {
    let world = format!(
        "{preamble}\n{defn}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)"
    );
    let w = startup_from_source(&world, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup (type-check): {e:?}"))?;
    let ast = wat::parse_one!(call).map_err(|e| format!("parse: {e:?}"))?;
    eval_in_frozen(&ast, &w, &Environment::new())
        .map_err(|e| format!("eval: {e:?}"))
        .map(|tv| tv.value_owned())
}

/// Convenience: no preamble.
fn eval_simple(defn: &str, call: &str) -> Result<Value, String> {
    eval_probe("", defn, call)
}

// ── assoc round-trip — HashMap ────────────────────────────────────────────────

#[test]
fn hashmap_assoc_key_present_after() {
    // Build a HashMap<String,i64>, assoc a key, verify the key is present.
    let defn = r#"
        (:wat::core::defn :p::f [] -> :wat::core::bool
          (:wat::core::let
            [m  (:wat::core::HashMap :wat::core::String :wat::core::i64)
             m2 (:wat::core::assoc m "answer" 42)]
            (:wat::core::HashMap/contains-key? m2 "answer")))
    "#;
    match eval_simple(defn, "(:p::f)") {
        Ok(Value::bool(true)) => {}
        Ok(other) => panic!("HashMap assoc round-trip: expected bool(true), got {other:?}"),
        Err(e) => panic!("HashMap assoc should classify + run: {e}"),
    }
}

#[test]
fn hashmap_assoc_type_preserving() {
    // After assoc, the returned map still supports HashMap/contains-key?.
    let defn = r#"
        (:wat::core::defn :p::f [] -> :wat::core::bool
          (:wat::core::let
            [m  (:wat::core::HashMap :wat::core::String :wat::core::i64)
             m2 (:wat::core::assoc m "k" 1)]
            (:wat::core::HashMap/contains-key? m2 "k")))
    "#;
    match eval_simple(defn, "(:p::f)") {
        Ok(Value::bool(true)) => {}
        Ok(other) => panic!("HashMap assoc type-preserving: expected bool(true), got {other:?}"),
        Err(e) => panic!("HashMap assoc type-preserving test failed: {e}"),
    }
}

// ── assoc round-trip — PersistentMap ─────────────────────────────────────────

#[test]
fn persistentmap_assoc_length_grows() {
    // Build a PersistentMap, assoc a key, check its length grew.
    let defn = r#"
        (:wat::core::defn :p::f [] -> :wat::core::i64
          (:wat::core::let
            [pm  (:wat::core::PersistentMap :a 1)
             pm2 (:wat::core::assoc pm :b 2)]
            (:wat::core::PersistentMap/length pm2)))
    "#;
    match eval_simple(defn, "(:p::f)") {
        Ok(Value::i64(2)) => {}
        Ok(other) => panic!("PersistentMap assoc round-trip: expected i64(2), got {other:?}"),
        Err(e) => panic!("PersistentMap assoc should classify + run: {e}"),
    }
}

#[test]
fn persistentmap_assoc_immutable() {
    // assoc must NOT mutate the original; original stays length 1.
    let defn = r#"
        (:wat::core::defn :p::f [] -> :wat::core::i64
          (:wat::core::let
            [pm  (:wat::core::PersistentMap :a 1)
             _   (:wat::core::assoc pm :b 2)]
            (:wat::core::PersistentMap/length pm)))
    "#;
    match eval_simple(defn, "(:p::f)") {
        Ok(Value::i64(1)) => {}
        Ok(other) => panic!("PersistentMap assoc must not mutate original: expected i64(1), got {other:?}"),
        Err(e) => panic!("PersistentMap assoc immutability test failed: {e}"),
    }
}

// ── assoc round-trip — base Record (:wat::Record::def) ───────────────────────

#[test]
fn base_record_assoc_field_updated() {
    let preamble = r#"(:wat::Record::def :probe::mr::Pt [x <- :wat::core::i64  y <- :wat::core::i64])"#;
    // assoc :y → 99; read :y back from the updated record.
    let defn = r#"
        (:wat::core::defn :p::f [] -> :wat::core::i64
          (:wat::core::let
            [pt  (:probe::mr::Pt 3 4)
             pt2 (:wat::core::assoc pt :y 99)]
            (:probe::mr::Pt/y pt2)))
    "#;
    match eval_probe(preamble, defn, "(:p::f)") {
        Ok(Value::i64(99)) => {}
        Ok(other) => panic!("base Record assoc round-trip: expected i64(99), got {other:?}"),
        Err(e) => panic!("base Record (wat__Record) assoc should classify + run: {e}"),
    }
}

#[test]
fn base_record_assoc_preserves_other_fields() {
    let preamble = r#"(:wat::Record::def :probe::mr::Coord [x <- :wat::core::i64  y <- :wat::core::i64])"#;
    let defn = r#"
        (:wat::core::defn :p::f [] -> :wat::core::i64
          (:wat::core::let
            [c  (:probe::mr::Coord 10 20)
             c2 (:wat::core::assoc c :y 99)]
            (:probe::mr::Coord/x c2)))
    "#;
    match eval_probe(preamble, defn, "(:p::f)") {
        Ok(Value::i64(10)) => {} // x was untouched
        Ok(other) => panic!("Record assoc must preserve x field: expected i64(10), got {other:?}"),
        Err(e) => panic!("Record assoc field-preservation test failed: {e}"),
    }
}

// ── assoc round-trip — holonic Record (:wat::holon::Record::def) ─────────────

#[test]
fn holonic_record_assoc_field_updated() {
    let preamble = r#"(:wat::holon::Record::def :probe::mr::Volt [value <- :wat::core::i64])"#;
    // assoc :value → 77; read it back.
    let defn = r#"
        (:wat::core::defn :p::f [] -> :wat::core::i64
          (:wat::core::let
            [v  (:probe::mr::Volt 10)
             v2 (:wat::core::assoc v :value 77)]
            (:probe::mr::Volt/value v2)))
    "#;
    match eval_probe(preamble, defn, "(:p::f)") {
        Ok(Value::i64(77)) => {}
        Ok(other) => panic!("holonic Record assoc round-trip: expected i64(77), got {other:?}"),
        Err(e) => panic!("holonic Record (wat__holon__Record) assoc should classify + run: {e}"),
    }
}

// ── Record get ────────────────────────────────────────────────────────────────

#[test]
fn record_get_existing_field_returns_some() {
    let preamble = r#"(:wat::Record::def :probe::rgal::Sensor [id <- :wat::core::i64  label <- :wat::core::String])"#;
    let defn = r#"
        (:wat::core::defn :p::f [] -> (:wat::core::Option :wat::core::Value)
          (:wat::core::let
            [s (:probe::rgal::Sensor 42 "temp")]
            (:wat::core::get s :id)))
    "#;
    match eval_probe(preamble, defn, "(:p::f)") {
        Ok(Value::Option(inner)) => match inner.as_ref() {
            Some(Value::i64(42)) => {}
            other => panic!("record get :id expected Some(i64(42)), got {other:?}"),
        },
        Ok(other) => panic!("record get :id expected Option, got {other:?}"),
        Err(e) => panic!("record get :id failed: {e}"),
    }
}

#[test]
fn record_get_missing_field_returns_none() {
    let preamble = r#"(:wat::Record::def :probe::rgal::Sensor2 [id <- :wat::core::i64])"#;
    let defn = r#"
        (:wat::core::defn :p::f [] -> (:wat::core::Option :wat::core::Value)
          (:wat::core::let
            [s (:probe::rgal::Sensor2 7)]
            (:wat::core::get s :no-such-field)))
    "#;
    match eval_probe(preamble, defn, "(:p::f)") {
        Ok(Value::Option(inner)) => match inner.as_ref() {
            None => {}
            other => panic!("record get missing field expected None, got {other:?}"),
        },
        Ok(other) => panic!("record get missing field expected Option, got {other:?}"),
        Err(e) => panic!("record get missing field failed: {e}"),
    }
}

// ── Record contains? ──────────────────────────────────────────────────────────

#[test]
fn record_contains_existing_field_true() {
    let preamble = r#"(:wat::Record::def :probe::rgal::Node [x <- :wat::core::i64  y <- :wat::core::i64])"#;
    let defn = r#"
        (:wat::core::defn :p::f [] -> :wat::core::bool
          (:wat::core::let
            [n (:probe::rgal::Node 1 2)]
            (:wat::core::contains? n :x)))
    "#;
    match eval_probe(preamble, defn, "(:p::f)") {
        Ok(Value::bool(true)) => {}
        Ok(other) => panic!("record contains? :x expected true, got {other:?}"),
        Err(e) => panic!("record contains? existing field failed: {e}"),
    }
}

#[test]
fn record_contains_missing_field_false() {
    let preamble = r#"(:wat::Record::def :probe::rgal::Node2 [x <- :wat::core::i64])"#;
    let defn = r#"
        (:wat::core::defn :p::f [] -> :wat::core::bool
          (:wat::core::let
            [n (:probe::rgal::Node2 5)]
            (:wat::core::contains? n :z)))
    "#;
    match eval_probe(preamble, defn, "(:p::f)") {
        Ok(Value::bool(false)) => {}
        Ok(other) => panic!("record contains? :z (missing) expected false, got {other:?}"),
        Err(e) => panic!("record contains? missing field failed: {e}"),
    }
}

// ── Record length ─────────────────────────────────────────────────────────────

#[test]
fn record_length_field_count() {
    let preamble = r#"(:wat::Record::def :probe::rgal::Triple [a <- :wat::core::i64  b <- :wat::core::i64  c <- :wat::core::i64])"#;
    let defn = r#"
        (:wat::core::defn :p::f [] -> :wat::core::i64
          (:wat::core::let
            [t (:probe::rgal::Triple 1 2 3)]
            (:wat::core::length t)))
    "#;
    match eval_probe(preamble, defn, "(:p::f)") {
        Ok(Value::i64(3)) => {}
        Ok(other) => panic!("record length expected 3, got {other:?}"),
        Err(e) => panic!("record length failed: {e}"),
    }
}

// ── Record empty? ─────────────────────────────────────────────────────────────

#[test]
fn record_empty_q_nonempty_false() {
    let preamble = r#"(:wat::Record::def :probe::rgal::Pair [a <- :wat::core::i64  b <- :wat::core::i64])"#;
    let defn = r#"
        (:wat::core::defn :p::f [] -> :wat::core::bool
          (:wat::core::let
            [p (:probe::rgal::Pair 10 20)]
            (:wat::core::empty? p)))
    "#;
    match eval_probe(preamble, defn, "(:p::f)") {
        Ok(Value::bool(false)) => {}
        Ok(other) => panic!("record empty? on Pair expected false, got {other:?}"),
        Err(e) => panic!("record empty? failed: {e}"),
    }
}

// ── non-keyed value → TypeMismatch ────────────────────────────────────────────

#[test]
fn non_keyed_vector_assoc_rejected() {
    // The type-checker should reject assoc on a Vector type.
    let defn = r#"
        (:wat::core::defn :p::f [] -> :wat::core::i64
          (:wat::core::assoc (:wat::core::Vector :wat::core::i64 1 2 3) 0 99))
    "#;
    match eval_simple(defn, "(:p::f)") {
        Err(_) => {} // TypeMismatch at check or runtime — both acceptable
        Ok(v) => panic!("assoc on a Vector must be rejected; got {v:?}"),
    }
}
