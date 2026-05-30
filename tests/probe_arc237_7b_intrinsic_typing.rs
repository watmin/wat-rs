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

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

fn eval_value(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new())).expect("startup");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute").value_owned()
}

// ─── TIER A — empty? (∀T -> bool) ───────────────────────────────────────────

#[test]
fn empty_q_vector() {
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::empty? (:wat::core::Vector :wat::core::i64)))"#),
        Value::bool(true),
    );
}

#[test]
fn empty_q_hashset_false() {
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::empty? (:wat::core::HashSet :wat::core::i64 1 2)))"#),
        Value::bool(false),
    );
}

// ─── TIER A — contains? ((coll, elem) -> bool) ──────────────────────────────

#[test]
fn contains_q_vector_hit() {
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::contains? (:wat::core::Vector :wat::core::i64 1 2 3) 2))"#),
        Value::bool(true),
    );
}

// ─── TIER B — get ((coll, key) -> Option<element>) : PRECISION ──────────────
// The load-bearing case: the result is matched as :wat::core::Option<i64> and
// the Some-arm binds x AT i64 (used in i64 arithmetic). If this type-checks
// today, the intrinsic must reproduce Option<element> precision (custom arm),
// NOT a loose ∀.

#[test]
fn get_vector_precise_element_typing() {
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::i64
          (:wat::core::match (:wat::core::get (:wat::core::Vector :wat::core::i64 10 20 30) 1)
                                    -> :wat::core::i64
                                    ((:wat::core::Some x) (:wat::core::i64::+'2 x 5))
                                    (:wat::core::None -1)))"#),
        Value::i64(25),
        "get index 1 -> Some(20); 20 + 5 = 25 — proves element x is typed i64",
    );
}

// ─── TIER B — conj ((coll, elem) -> coll) : TYPE PRESERVATION ───────────────
// conj returns the SAME collection type; the result is fed back to empty?/length
// (a collection op), proving it stayed a Vector.

#[test]
fn conj_vector_preserves_collection_type() {
    assert_eq!(
        eval_value(r#"(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length (:wat::core::conj (:wat::core::Vector :wat::core::i64 1 2) 3)))"#),
        Value::i64(3),
        "conj appends -> Vector of length 3; result is still a collection",
    );
}

// ─── TIER B — ELEMENT-TYPING ENFORCEMENT (wrong-elem rejection) ─────────────
// Proven: the ∀T intrinsics reject wrong-elem calls (Vector<i64>.contains?("x")
// / .conj("x")) at check time — confirming custom inference arms were needed.
// Uses startup_from_source directly (no eval) — checks CHECK-time rejection.

fn try_startup(src: &str) -> Result<(), String> {
    let src = with_nil_main(src);
    startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

#[test]
fn contains_q_wrong_element_rejected_at_check() {
    let result = try_startup(
        r#"(:wat::core::defn :user::compute [] -> :wat::core::bool (:wat::core::contains? (:wat::core::Vector :wat::core::i64 1 2 3) "x"))"#,
    );
    assert!(
        result.is_err(),
        "contains? on Vector<i64> with String elem MUST reject at check (current behavior; \
         intrinsic must preserve via custom arm); got: {:?}",
        result,
    );
}

#[test]
fn conj_wrong_element_rejected_at_check() {
    let result = try_startup(
        r#"(:wat::core::defn :user::compute [] -> :wat::core::i64 (:wat::core::length (:wat::core::conj (:wat::core::Vector :wat::core::i64 1 2) "x")))"#,
    );
    assert!(
        result.is_err(),
        "conj on Vector<i64> with String elem MUST reject at check (current behavior; \
         intrinsic must preserve via custom arm — defines whether plain ∀T,E suffices); got: {:?}",
        result,
    );
}
