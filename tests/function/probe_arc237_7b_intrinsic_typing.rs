//! FM-2-bis probe for Stone 237.7b — settle the ∀-scheme-vs-custom-inference
//! fork for the collection ops BEFORE briefing the intrinsic migration.
//!
//! These exercise the ∀T intrinsic behavior of empty? / contains? /
//! get / conj (define-dispatch retired at Stone 241.13), AND reveal the
//! typing precision per op required of the intrinsic impls:
//!
//!   - TIER A (concrete return): empty? (-> bool), contains? (-> bool).
//!     If a plain typed use compiles, a plain ∀ scheme will suffice.
//!   - TIER B (element-typed return): get (-> Option<element>), conj (-> coll).
//!     If the result is usable AT the element/collection type precisely, the
//!     intrinsic MUST reproduce that (custom inference arm), not a loose ∀.
//!
//! Run: cargo test --release --test probe_arc237_7b_intrinsic_typing

//! Wat source: tests/function/probe_arc237_7b_intrinsic_typing.wat
//! Negative fixtures: probe_arc237_7b_contains_wrong_elem.wat.bad, probe_arc237_7b_conj_wrong_elem.wat.bad

use wat::check::error::CheckErrorKind;
use wat::freeze::{startup_beside, startup_from_file};
use wat::runtime::{apply_function, Value};

// just-eval (rubric): each `fn_name` names a zero-arg fn defined in the co-located
// fixture; fetch it from the frozen world and `apply_function` it — no inline wat driver.
fn run(fn_name: &str) -> Value {
    let world = startup_beside(file!()).expect("startup for 7b intrinsic-typing fixture");
    let func = world
        .symbols()
        .get(fn_name)
        .unwrap_or_else(|| panic!("no {fn_name} in fixture"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .expect("eval should succeed")
}

// ─── TIER A — empty? (∀T -> bool) ───────────────────────────────────────────

#[test]
fn empty_q_vector() {
    assert_eq!(run(":user::empty-q-vector"), Value::bool(true));
}

#[test]
fn empty_q_hashset_false() {
    assert_eq!(run(":user::empty-q-hashset-false"), Value::bool(false));
}

// ─── TIER A — contains? ((coll, elem) -> bool) ──────────────────────────────

#[test]
fn contains_q_vector_hit() {
    assert_eq!(run(":user::contains-q-vector-hit"), Value::bool(true));
}

// ─── TIER B — get ((coll, key) -> Option<element>) : PRECISION ──────────────
// get index 1 -> Some(20); 20 + 5 = 25 — proves element x is typed i64.

#[test]
fn get_vector_precise_element_typing() {
    assert_eq!(
        run(":user::get-vector-precise"),
        Value::i64(25),
        "get index 1 -> Some(20); 20 + 5 = 25 — proves element x is typed i64",
    );
}

// ─── TIER B — conj ((coll, elem) -> coll) : TYPE PRESERVATION ───────────────
// conj appends -> Vector of length 3; result is still a collection.

#[test]
fn conj_vector_preserves_collection_type() {
    assert_eq!(
        run(":user::conj-vector-preserves"),
        Value::i64(3),
        "conj appends -> Vector of length 3; result is still a collection",
    );
}

// ─── TIER B — ELEMENT-TYPING ENFORCEMENT (wrong-elem rejection) ─────────────
// The ∀T intrinsics reject wrong-elem calls at check time via custom inference arms.
// Each negative case lives in its own *.wat.bad fixture.

#[test]
fn contains_q_wrong_element_rejected_at_check() {
    let result = startup_from_file("tests/function/probe_arc237_7b_contains_wrong_elem.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":wat::core::contains?"
            && param == "#2"
            && expected == ":wat::core::i64"
            && got == ":wat::core::String"
    );
}

#[test]
fn conj_wrong_element_rejected_at_check() {
    let result = startup_from_file("tests/function/probe_arc237_7b_conj_wrong_elem.wat.bad");
    wat::assert_startup_error!(result, check
        CheckErrorKind::TypeMismatch { callee, param, expected, got, .. }
            if callee == ":wat::core::conj"
            && param == "#2"
            && expected == ":wat::core::i64"
            && got == ":wat::core::String"
    );
}
