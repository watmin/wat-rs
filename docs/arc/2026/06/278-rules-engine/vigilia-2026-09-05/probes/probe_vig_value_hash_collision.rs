//! VIGILIA experiri probe — `impl Hash for Value` (src/value/value.rs:751) early-returns
//! `Vec` and `List` into `hash_sequence` (`:556`), which writes a tag and each element but
//! NO length and NO terminator. Two structurally-UNEQUAL nestings therefore emit the same
//! write stream. The doc above `hash_sequence` claims "collision ~1/2^64".
//!
//! Run: cargo nextest run --release -p wat --test rete probe_vig_value_hash_collision

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use wat::runtime::Value;

fn v(items: Vec<Value>) -> Value {
    Value::Vec(Arc::new(items))
}

fn h(x: &Value) -> u64 {
    let mut s = DefaultHasher::new();
    x.hash(&mut s);
    s.finish()
}

/// Calibration — two drives to FIRE (equal values hash equal) and two to REFUSE
/// (unequal values hash unequal), on shapes the impl plainly separates.
#[test]
fn calibration() {
    let a = v(vec![Value::i64(1), Value::i64(2)]);
    let b = v(vec![Value::i64(1), Value::i64(2)]);
    let c = v(vec![Value::i64(1), Value::i64(3)]);
    assert_eq!(h(&a), h(&b), "FIRE 1: equal vectors must hash equal");
    assert_eq!(a, b, "FIRE 2: equal vectors must compare equal");
    assert_ne!(h(&a), h(&c), "REFUSE 1: differing element must change the hash");
    assert_ne!(a, c, "REFUSE 2: differing element must compare unequal");
}

/// The two `assert_ne!` lines the ward could not run.
#[test]
fn nested_empty_vectors_do_not_collide() {
    // Vec[ Vec[], Vec[1] ]
    let left = v(vec![v(vec![]), v(vec![Value::i64(1)])]);
    // Vec[ Vec[ Vec[], 1 ] ]
    let right = v(vec![v(vec![v(vec![]), Value::i64(1)])]);
    assert_ne!(left, right, "the two shapes must be UNEQUAL under PartialEq");
    assert_ne!(
        h(&left),
        h(&right),
        "STRUCTURAL COLLISION: hash_sequence writes no length and no terminator, so \
         Vec[Vec[],Vec[1]] and Vec[Vec[Vec[],1]] emit identical write streams. \
         left={:#x} right={:#x}",
        h(&left),
        h(&right)
    );
}

/// A second, flatter witness: the boundary between a nested empty and its sibling.
#[test]
fn a_shifted_nesting_boundary_does_not_collide() {
    // Vec[ Vec[1], Vec[2] ]  vs  Vec[ Vec[1, 2] ]   -- hmm, differing depths only
    let left = v(vec![v(vec![Value::i64(1)]), Value::i64(2)]);
    let right = v(vec![v(vec![Value::i64(1), Value::i64(2)])]);
    assert_ne!(left, right, "the two shapes must be UNEQUAL under PartialEq");
    assert_ne!(
        h(&left),
        h(&right),
        "STRUCTURAL COLLISION #2: left={:#x} right={:#x}",
        h(&left),
        h(&right)
    );
}

/// Blast radius: does the collision survive into `wat__std__HashSet`, whose arm hashes
/// element hash VALUES? If two singleton sets over colliding elements hash equal, the
/// collision is amplified one level up rather than absorbed.
#[test]
fn the_collision_reaches_hashset() {
    let left = v(vec![v(vec![]), v(vec![Value::i64(1)])]);
    let right = v(vec![v(vec![v(vec![]), Value::i64(1)])]);
    let sl = Value::wat__std__HashSet(Arc::new(std::iter::once(left).collect()));
    let sr = Value::wat__std__HashSet(Arc::new(std::iter::once(right).collect()));
    assert_ne!(sl, sr, "the two singleton sets must be UNEQUAL");
    assert_ne!(
        h(&sl),
        h(&sr),
        "the element collision propagated into the HashSet arm: {:#x} == {:#x}",
        h(&sl),
        h(&sr)
    );
}
