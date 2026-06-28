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
//!
//! Wat source lives in the co-located fixture: probe_map_container.wat
//! (slurped via startup_beside(file!())).
//! Negative fixture: tests/collection/probe_map_container_bad_assoc.wat

use wat::freeze::{eval_in_frozen, startup_beside, startup_from_file};
use wat::runtime::{Environment, Value};

fn eval_probe(call: &str) -> Result<Value, String> {
    let world = startup_beside(file!()).map_err(|e| format!("startup (type-check): {e:?}"))?;
    let ast = wat::parse_one!(call).map_err(|e| format!("parse: {e:?}"))?;
    eval_in_frozen(&ast, &world, &Environment::new())
        .map_err(|e| format!("eval: {e:?}"))
        .map(|tv| tv.value_owned())
}

// ── assoc round-trip — HashMap ────────────────────────────────────────────────

#[test]
fn hashmap_assoc_key_present_after() {
    match eval_probe("(:p::hashmap-assoc-key-present)") {
        Ok(Value::bool(true)) => {}
        Ok(other) => panic!("HashMap assoc round-trip: expected bool(true), got {other:?}"),
        Err(e) => panic!("HashMap assoc should classify + run: {e}"),
    }
}

#[test]
fn hashmap_assoc_type_preserving() {
    match eval_probe("(:p::hashmap-assoc-type-preserving)") {
        Ok(Value::bool(true)) => {}
        Ok(other) => panic!("HashMap assoc type-preserving: expected bool(true), got {other:?}"),
        Err(e) => panic!("HashMap assoc type-preserving test failed: {e}"),
    }
}

// ── assoc round-trip — PersistentMap ─────────────────────────────────────────

#[test]
fn persistentmap_assoc_length_grows() {
    match eval_probe("(:p::persistentmap-assoc-length-grows)") {
        Ok(Value::i64(2)) => {}
        Ok(other) => panic!("PersistentMap assoc round-trip: expected i64(2), got {other:?}"),
        Err(e) => panic!("PersistentMap assoc should classify + run: {e}"),
    }
}

#[test]
fn persistentmap_assoc_immutable() {
    match eval_probe("(:p::persistentmap-assoc-immutable)") {
        Ok(Value::i64(1)) => {}
        Ok(other) => panic!("PersistentMap assoc must not mutate original: expected i64(1), got {other:?}"),
        Err(e) => panic!("PersistentMap assoc immutability test failed: {e}"),
    }
}

// ── assoc round-trip — base Record (:wat::core::defrecord) ───────────────────────

#[test]
fn base_record_assoc_field_updated() {
    match eval_probe("(:p::base-record-assoc-field-updated)") {
        Ok(Value::i64(99)) => {}
        Ok(other) => panic!("base Record assoc round-trip: expected i64(99), got {other:?}"),
        Err(e) => panic!("base Record (wat__Record) assoc should classify + run: {e}"),
    }
}

#[test]
fn base_record_assoc_preserves_other_fields() {
    match eval_probe("(:p::base-record-assoc-preserves-other-fields)") {
        Ok(Value::i64(10)) => {} // x was untouched
        Ok(other) => panic!("Record assoc must preserve x field: expected i64(10), got {other:?}"),
        Err(e) => panic!("Record assoc field-preservation test failed: {e}"),
    }
}

// ── assoc round-trip — holonic Record (:wat::holon::defrecord) ─────────────

#[test]
fn holonic_record_assoc_field_updated() {
    match eval_probe("(:p::holonic-record-assoc-field-updated)") {
        Ok(Value::i64(77)) => {}
        Ok(other) => panic!("holonic Record assoc round-trip: expected i64(77), got {other:?}"),
        Err(e) => panic!("holonic Record (wat__holon__Record) assoc should classify + run: {e}"),
    }
}

// ── Record get ────────────────────────────────────────────────────────────────

#[test]
fn record_get_existing_field_returns_some() {
    match eval_probe("(:p::record-get-existing-field)") {
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
    match eval_probe("(:p::record-get-missing-field)") {
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
    match eval_probe("(:p::record-contains-existing)") {
        Ok(Value::bool(true)) => {}
        Ok(other) => panic!("record contains? :x expected true, got {other:?}"),
        Err(e) => panic!("record contains? existing field failed: {e}"),
    }
}

#[test]
fn record_contains_missing_field_false() {
    match eval_probe("(:p::record-contains-missing)") {
        Ok(Value::bool(false)) => {}
        Ok(other) => panic!("record contains? :z (missing) expected false, got {other:?}"),
        Err(e) => panic!("record contains? missing field failed: {e}"),
    }
}

// ── Record length ─────────────────────────────────────────────────────────────

#[test]
fn record_length_field_count() {
    match eval_probe("(:p::record-length)") {
        Ok(Value::i64(3)) => {}
        Ok(other) => panic!("record length expected 3, got {other:?}"),
        Err(e) => panic!("record length failed: {e}"),
    }
}

// ── Record empty? ─────────────────────────────────────────────────────────────

#[test]
fn record_empty_q_nonempty_false() {
    match eval_probe("(:p::record-empty-nonempty)") {
        Ok(Value::bool(false)) => {}
        Ok(other) => panic!("record empty? on Pair expected false, got {other:?}"),
        Err(e) => panic!("record empty? failed: {e}"),
    }
}

// ── non-keyed value → TypeMismatch ────────────────────────────────────────────

#[test]
fn non_keyed_vector_assoc_rejected() {
    // The type-checker should reject assoc on a Vector type.
    match startup_from_file("tests/collection/probe_map_container_bad_assoc.wat") {
        Err(_) => {} // TypeMismatch at check — acceptable
        Ok(_) => panic!("assoc on a Vector must be rejected at type-check"),
    }
}
