//! Arc 233 Stone 233.2.a — `Value::Tracked` transparency contracts.
//!
//! Eight tests verifying that `Value::Tracked` is transparent for all
//! behavioral contracts: Eq, Hash, Display (via render_value), Clone,
//! `inner()` recursion, `provenance()` outermost, `ValueSnapshot::of`,
//! and bare ValueSnapshot provenance.
//!
//! No WAT evaluation — Rust-level probes only.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use wat::runtime::{Provenance, Value, ValueSnapshot};
use wat::span::Span;

// ─── hash helper ─────────────────────────────────────────────────────────────

fn hash_of(v: &Value) -> u64 {
    let mut h = DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
}

// ─── Contract 1 — Display (render_value) unwraps Tracked ─────────────────────
//
// `render_value` is private; we test via ValueSnapshot::of which calls it.
// Both bare and Tracked should produce identical rendered output.

#[test]
fn contract_1_display_unwraps_tracked() {
    let bare = Value::i64(42);
    let tracked = Value::Tracked {
        inner: Box::new(Value::i64(42)),
        provenance: Provenance::Unknown,
    };
    let snap_bare = ValueSnapshot::of(&bare);
    let snap_tracked = ValueSnapshot::of(&tracked);
    assert_eq!(
        snap_bare.rendered, snap_tracked.rendered,
        "Tracked and bare i64(42) must render identically"
    );
    assert_eq!(
        snap_bare.type_name, snap_tracked.type_name,
        "Tracked and bare i64(42) must have identical type_name"
    );
}

// ─── Contract 2 — Eq compares inner ──────────────────────────────────────────

#[test]
fn contract_2_eq_compares_inner() {
    let bare = Value::i64(42);
    let tracked = Value::Tracked {
        inner: Box::new(Value::i64(42)),
        provenance: Provenance::Unknown,
    };
    // bare == tracked
    assert_eq!(bare, tracked, "bare i64(42) must equal Tracked(i64(42))");
    // tracked == bare
    assert_eq!(tracked, bare, "Tracked(i64(42)) must equal bare i64(42)");

    // Tracked-wrapping-Tracked equals bare too
    let double = Value::Tracked {
        inner: Box::new(tracked.clone()),
        provenance: Provenance::RuntimeBuilt {
            producer: "test",
            call_span: Span::unknown(),
        },
    };
    assert_eq!(
        double, bare,
        "Tracked-of-Tracked must equal bare when inners match"
    );
}

// ─── Contract 3 — Hash unwraps (HashMap correctness) ─────────────────────────

#[test]
fn contract_3_hash_unwraps_tracked_hashmap_correctness() {
    let bare = Value::i64(42);
    let tracked = Value::Tracked {
        inner: Box::new(Value::i64(42)),
        provenance: Provenance::Unknown,
    };

    // Hash values must be identical
    assert_eq!(
        hash_of(&bare),
        hash_of(&tracked),
        "bare and Tracked must hash identically"
    );

    // HashMap behavior: insert with bare key, lookup with tracked key
    let mut map: std::collections::HashMap<Value, &str> = std::collections::HashMap::new();
    map.insert(bare.clone(), "hello");
    assert_eq!(
        map.get(&tracked),
        Some(&"hello"),
        "lookup via Tracked key must find bare-key entry"
    );

    // Also verify tracked-of-keyword hashes same as bare keyword
    let bare_kw = Value::wat__core__keyword(Arc::new(":test::key".to_string()));
    let tracked_kw = Value::Tracked {
        inner: Box::new(Value::wat__core__keyword(Arc::new(":test::key".to_string()))),
        provenance: Provenance::Literal {
            span: Span::unknown(),
        },
    };
    assert_eq!(
        hash_of(&bare_kw),
        hash_of(&tracked_kw),
        "bare and Tracked keyword must hash identically"
    );
}

// ─── Contract 4 — Clone preserves Tracked-ness ───────────────────────────────

#[test]
fn contract_4_clone_preserves_tracked() {
    let original = Value::Tracked {
        inner: Box::new(Value::i64(42)),
        provenance: Provenance::RuntimeBuilt {
            producer: "test",
            call_span: Span::unknown(),
        },
    };
    let cloned = original.clone();
    assert_eq!(original, cloned, "cloned Tracked must equal original");
    match cloned {
        Value::Tracked { ref provenance, .. } => match provenance {
            Provenance::RuntimeBuilt { producer, .. } => {
                assert_eq!(*producer, "test", "cloned Tracked must preserve producer");
            }
            _ => panic!("provenance lost on clone"),
        },
        _ => panic!("Tracked variant lost on clone"),
    }
}

// ─── Contract 5 — inner() recurses through Tracked-of-Tracked ────────────────

#[test]
fn contract_5_inner_recurses() {
    let bare = Value::i64(42);
    let single = Value::Tracked {
        inner: Box::new(bare.clone()),
        provenance: Provenance::Unknown,
    };
    let double = Value::Tracked {
        inner: Box::new(single.clone()),
        provenance: Provenance::Unknown,
    };

    assert_eq!(
        single.inner(),
        &bare,
        "single-Tracked.inner() must return the bare value"
    );
    assert_eq!(
        double.inner(),
        &bare,
        "double-Tracked.inner() must recurse to the bare value"
    );
    // bare.inner() returns self
    assert_eq!(bare.inner(), &bare, "bare.inner() must return self");
}

// ─── Contract 6 — provenance() returns outermost Tracked's provenance ─────────

#[test]
fn contract_6_provenance_returns_outermost() {
    let inner_tracked = Value::Tracked {
        inner: Box::new(Value::i64(42)),
        provenance: Provenance::Literal {
            span: Span::unknown(),
        },
    };
    let outer_tracked = Value::Tracked {
        inner: Box::new(inner_tracked.clone()),
        provenance: Provenance::RuntimeBuilt {
            producer: "outer",
            call_span: Span::unknown(),
        },
    };
    match outer_tracked.provenance() {
        Provenance::RuntimeBuilt { producer, .. } => {
            assert_eq!(producer, "outer", "provenance() must return outermost");
        }
        _ => panic!("expected outermost RuntimeBuilt provenance"),
    }
    // inner_tracked has Literal at outermost
    match inner_tracked.provenance() {
        Provenance::Literal { .. } => {}
        _ => panic!("inner_tracked.provenance() should be Literal"),
    }
    // bare value returns Unknown
    let bare = Value::i64(42);
    match bare.provenance() {
        Provenance::Unknown => {}
        _ => panic!("bare value must have Unknown provenance"),
    }
}

// ─── Contract 7 — ValueSnapshot::of extracts Provenance from Tracked ──────────

#[test]
fn contract_7_value_snapshot_extracts_provenance() {
    let tracked = Value::Tracked {
        inner: Box::new(Value::wat__core__keyword(Arc::new(":foo".to_string()))),
        provenance: Provenance::RuntimeBuilt {
            producer: "test-producer",
            call_span: Span::unknown(),
        },
    };
    let snap = ValueSnapshot::of(&tracked);
    assert_eq!(
        snap.type_name, "wat::core::keyword",
        "ValueSnapshot must unwrap Tracked for type_name"
    );
    assert!(
        snap.rendered.contains(":foo"),
        "ValueSnapshot rendered must contain the inner keyword"
    );
    match snap.provenance {
        Provenance::RuntimeBuilt { producer, .. } => {
            assert_eq!(
                producer, "test-producer",
                "ValueSnapshot must extract RuntimeBuilt provenance"
            );
        }
        _ => panic!("ValueSnapshot didn't extract provenance from Tracked"),
    }
}

// ─── Contract 8 — Bare Value's ValueSnapshot has Unknown provenance ───────────

#[test]
fn contract_8_bare_value_snapshot_has_unknown_provenance() {
    let bare = Value::wat__core__keyword(Arc::new(":foo".to_string()));
    let snap = ValueSnapshot::of(&bare);
    assert_eq!(
        snap.type_name, "wat::core::keyword",
        "bare ValueSnapshot must have correct type_name"
    );
    match snap.provenance {
        Provenance::Unknown => {} // expected
        other => panic!(
            "bare Value should have Unknown provenance; got {:?}",
            other
        ),
    }
}
