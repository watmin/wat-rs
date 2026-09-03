//! Arc 278 D7 — the seed pass's TYPE-ERASURE SEAM, held to the oracle.
//!
//! ## What went wrong, and why nothing on the floor saw it
//!
//! `alpha_seed` has two writers of `wm.alpha[aid]`: `alpha_activate_fact` pushes, and the
//! occupancy batch does a whole-entry `insert`. `build_alpha_index` files every alpha node under
//! exactly ONE erased `pat.type_head`, so both writers reach the same `aid` the moment one
//! runtime class sends some of its facts down each path. `pack_i64_row` decides that path from
//! RUNTIME values, so a PARAMETRIC record — `(:d7g::Box :- [T] [k <- i64  v <- :T])`, one class
//! `d7g::Box` for every `T` — is exactly such a class. The replace then discarded the push, and
//! `d_alpha[aid]` was left holding slot indices that named DIFFERENT elements. Measured
//! 2026-09-02: `native=2 oracle=3`, a derived fact lost with no diagnostic.
//!
//! ⛔ THE INSTRUMENT THAT WAS SUPPOSED TO CATCH THIS WAS BLIND BY CONSTRUCTION. The `leaf_occ`
//! differential (`record_seed_leaf_vs_alpha`) built its `predicted` set by skipping any fact
//! whose `i64_by_fact[i]` was `None` — the very predicate that decides batch membership — so it
//! compared the batch's output against the batch's output. It read `extra=[] missing=[]` while
//! the fact was dropping. Two engines are the only referee here, which is what this file is.
//!
//! ## What this gates — the property, not the fixture
//!
//! > One runtime class whose instances differ in packability must derive exactly what
//! > `fire-rules$oracle` derives, in every interleaving.
//!
//! Six workloads over one generic class (both interleavings, an alternation, a second and
//! unrelated erasure via a RECORD filler, and both uniform controls), plus a mixed class beside a
//! uniformly-packable one so the fast path is exercised in the same session it is denied to its
//! neighbour. Every arm compares the derived KEY SET, never a count: the aliasing half of this
//! defect produces a wrong answer of the right size.
//!
//! The class-uniform decision's other direction — that a uniform class still BATCHES rather than
//! quietly falling back to activate-everything — cannot be seen from wat, because both paths
//! derive the same facts. It is gated on the census counters in
//! `src/rete/kernel/tests/pass_semantics.rs::seed_batches_uniform_classes_and_defers_mixed_ones`.
//!
//! Run: cargo nextest run --release -E 'test(d7_parametric_erasure)'

use wat::freeze::call_beside_value;
use wat::runtime::Value;

/// `hitN=… hitO=… plainN=… plainO=… pairN=… pairO=…` → the six fields, in order.
fn report(entry: &str) -> [String; 6] {
    let raw = match call_beside_value(file!(), entry) {
        Ok(Value::String(s)) => s.to_string(),
        Ok(other) => panic!("{entry}: expected a String report; got {other:?}"),
        Err(e) => panic!("{entry}: eval failed: {e:?}"),
    };
    let mut out: Vec<String> = Vec::new();
    for field in ["hitN=", "hitO=", "plainN=", "plainO=", "pairN=", "pairO="] {
        let at = raw
            .find(field)
            .unwrap_or_else(|| panic!("{entry}: no {field:?} in report {raw:?}"));
        let rest = &raw[at + field.len()..];
        let end = rest.find(' ').unwrap_or(rest.len());
        out.push(rest[..end].to_string());
    }
    out.try_into().expect("six fields")
}

/// The whole assertion, one place: native == oracle, and both == what the workload must derive.
///
/// `native == oracle` alone is not enough — two engines that agreed on nothing would pass it —
/// so each arm also names the key set it expects. And `expected` is written as keys rather than
/// a count because this defect's own signature is a right-sized wrong answer.
fn assert_agrees(entry: &str, hits: &str, plains: &str, pairs: &str) {
    let [hit_n, hit_o, plain_n, plain_o, pair_n, pair_o] = report(entry);
    assert_eq!(
        hit_n, hit_o,
        "{entry}: NATIVE AND ORACLE DISAGREE on the derived Hit keys — native={hit_n:?} \
         oracle={hit_o:?}. A key present in one and absent in the other is a fact the seed pass \
         dropped or duplicated; a key that differs in VALUE is `d_alpha` indexing an element that \
         moved under it."
    );
    assert_eq!(
        hit_n, hits,
        "{entry}: derived Hit keys {hit_n:?}, expected {hits:?}"
    );
    assert_eq!(
        plain_n, plain_o,
        "{entry}: native/oracle disagree on PlainHit — native={plain_n:?} oracle={plain_o:?}"
    );
    assert_eq!(
        plain_n, plains,
        "{entry}: derived PlainHit keys {plain_n:?}, expected {plains:?}"
    );
    assert_eq!(
        pair_n, pair_o,
        "{entry}: native/oracle disagree on Pair (the JOIN over the erased class) — \
         native={pair_n:?} oracle={pair_o:?}"
    );
    assert_eq!(
        pair_n, pairs,
        "{entry}: derived Pair keys {pair_n:?}, expected {pairs:?}"
    );
}

/// 1 — the D7 shape as first driven: packable instance first, erased one second.
#[test]
fn mixed_packability_i64_first() {
    assert_agrees(":user::mixed-i64-first", "0,1,2,", "", "");
}

/// 2 — the same class, erased instance FIRST. The batch runs after the fact loop, so order must
/// not matter; a cure that only handled the observed order would pass test 1 and fail here.
#[test]
fn mixed_packability_erased_first() {
    assert_agrees(":user::mixed-erased-first", "0,1,2,", "", "");
}

/// 3 — four facts, alternating. Two on each path in one class.
#[test]
fn mixed_packability_alternating() {
    assert_agrees(":user::mixed-alternating", "0,1,2,3,", "", "");
}

/// 4 — A DIFFERENT ERASURE: the unpackable filler is a RECORD, not a String.
///
/// This is what keeps the gate on the property rather than on `Box[i64]`/`Box[String]`. The
/// defect is "one class, mixed packability"; `pack_i64_row` rejects an Aggregate for a different
/// reason than it rejects a String, and both must route the same way.
#[test]
fn mixed_packability_record_filler() {
    assert_agrees(":user::mixed-record-filler", "0,1,2,", "", "");
}

/// 5 — CONTROL, uniformly packable. The class keeps the occupancy batch; nothing changes.
#[test]
fn uniform_packable_control() {
    assert_agrees(":user::uniform-packable", "0,1,2,", "", "");
}

/// 6 — CONTROL, uniformly unpackable. The class was never in the batch: only writer 1 ran, so
/// this arm was correct before the cure and is here to name a regression on the activate path.
#[test]
fn uniform_unpackable_control() {
    assert_agrees(":user::uniform-unpackable", "0,1,2,", "", "");
}

/// 7 — ★ the mixed class BESIDE a uniform one, in a single session, with a join across them.
///
/// `d7g::Box` forfeits the batch while `d7g::Plain` keeps it, so the two seed paths run in the
/// same pass over different classes — the configuration the invariant is actually about. The
/// `Pair` join then consumes `Box`'s alpha delta as SLOT INDICES, which is where the aliasing
/// stops being a missing fact and becomes a wrong binding.
#[test]
fn mixed_class_beside_uniform_class_with_join() {
    assert_agrees(":user::mixed-beside-uniform", "0,1,2,", "0,1,2,", "0,1,2,");
}
