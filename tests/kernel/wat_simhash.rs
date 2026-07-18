//! Arc 051 — SimHash direction-space quantization.
//!
//! Coverage:
//! - Determinism: same AST, two calls → same i64
//! - Atom identity: `(simhash (Atom 0))` is stable
//! - Cosine-near-1 → small hamming distance (same/perturbed AST)
//! - Cosine-near-0 → hamming distance ≈ 32 (orthogonal-by-construction
//!   AST pair)
//! - Type system: returns `:wat::core::i64`; arithmetic + cache integration work

use wat::freeze::call_beside;
use wat::runtime::Value;

fn run_fn(fn_name: &str) -> Value {
    call_beside(file!(), fn_name).expect("eval should succeed")
}

fn assert_str(val: Value, expected: &str) {
    match val {
        Value::String(s) => assert_eq!(
            &*s, expected,
            "expected String({expected:?}); got String({s:?})"
        ),
        other => panic!("expected String({expected:?}); got {:?}", other),
    }
}

// ─── Determinism ─────────────────────────────────────────────────────

#[test]
fn simhash_deterministic_same_ast() {
    assert_str(run_fn(":my::compute-deterministic"), "yes");
}

// ─── Atom identity ───────────────────────────────────────────────────

#[test]
fn simhash_atom_zero_stable() {
    assert_str(run_fn(":my::compute-atom-stable"), "yes");
}

// ─── Cosine-near-1 → low hamming distance (same AST) ─────────────────
//
// Two encodings of the same AST shape produce the same vector,
// therefore the same SimHash. Hamming distance = 0.

#[test]
fn simhash_same_shape_zero_hamming() {
    assert_str(run_fn(":my::compute-same-shape"), "same");
}

// ─── Different ASTs → different keys (with high probability) ────────
//
// Two structurally different ASTs (e.g., distinct atoms) almost
// certainly produce different SimHash keys. Hamming distance is
// expected to be near 32 (half the bits differ on orthogonal inputs).
// We can't assert distance reliably; we assert the keys differ.

#[test]
fn simhash_distinct_atoms_distinct_keys() {
    assert_str(run_fn(":my::compute-distinct-atoms"), "diff");
}

// ─── Type system: simhash returns :wat::core::i64; works with arithmetic ───────

#[test]
fn simhash_result_works_in_arithmetic() {
    assert_str(run_fn(":my::compute-arithmetic"), "ok");
}

// (Cache composition with `:rust::lru::LruCache<i64,V>` is documented
// in the arc 051 DESIGN and exercised by `wat-lru`'s own test crate
// where the LRU shim is registered. The five tests above cover the
// primitive's contract: deterministic, identity-stable, distinct-AST-
// distinct-key, and i64-typed for downstream composition.)
