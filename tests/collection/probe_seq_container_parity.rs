//! Seq/collection checker↔runtime CONTAINER PARITY — the drift tripwire.
//!
//! Three collection ops accept a container at RUNTIME that the TYPE-CHECKER rejects (false-reject drift,
//! one-sided changes where a new container repr was added to runtime but not check.rs):
//!   - `first`/`second`/`third` (infer_positional_accessor, check.rs) MISSING PersistentVector + WatAST::List
//!   - `rest`                   (check.rs:5301)                       MISSING PersistentVector + WatAST::List
//!   - `conj`                   (infer_conj, collection/infer.rs)     MISSING List
//!
//! The runtime (runtime.rs / collection/*.rs) handles all of these correctly. RED at HEAD: each probe defn's
//! body type-errors → startup returns Err. GREEN when the checker arms are extended to equal the
//! runtime's accepted container set. This pins checker≡runtime so any FUTURE one-sided arm goes red.
//! Contract: DESIGN-STONE-seq-container-drift.md.
//!
//! Run: cargo test --release -p wat --test probe_seq_container_parity
//!
//! Wat source lives in the co-located fixture: probe_seq_container_parity.wat
//! (slurped via startup_beside(file!())).

use wat::freeze::{call_beside_value, StartupError};
use wat::runtime::Value;

fn eval_probe(fn_name: &str) -> Result<Value, StartupError> {
    call_beside_value(file!(), fn_name).map_err(|e| StartupError::Runtime(Box::new(e)))
}

fn expect_i64(call: &str, want: i64) {
    match eval_probe(call) {
        Ok(Value::i64(n)) => assert_eq!(n, want, "value: got {n} want {want}"),
        Ok(other) => panic!("expected i64({want}); got {other:?}"),
        Err(e) => panic!("checker≡runtime drift (should type-check + run): {e}"),
    }
}

fn expect_true(call: &str) {
    match eval_probe(call) {
        Ok(Value::bool(true)) => {}
        Ok(other) => panic!("expected bool(true); got {other:?}"),
        Err(e) => panic!("checker≡runtime drift (should type-check + run): {e}"),
    }
}

// ── first/second/third on PersistentVector → bare T (arc-278 flip; raising on out-of-range) ──

#[test]
fn first_on_persistent_vector() {
    expect_i64(":p::first-pv", 10);
}

#[test]
fn second_on_persistent_vector() {
    expect_i64(":p::second-pv", 20);
}

#[test]
fn third_on_persistent_vector() {
    expect_i64(":p::third-pv", 30);
}

// ── rest on PersistentVector → PersistentVector<T> (identity preserved; length-of-tail = 2) ──

#[test]
fn rest_on_persistent_vector() {
    expect_i64(":p::rest-pv", 2);
}

// ── conj on List → List<T> (the arc-220 repr the checker forgot; length-after-conj = 3) ──

#[test]
fn conj_on_list() {
    // `:wat::core::List` is the List constructor (variadic, no type keyword; check.rs:4073).
    expect_i64(":p::conj-list", 3);
}

// ── WatAST::List (arc-249 form-values): first/rest must type-check + run (compiles-and-runs asserts) ──
// arc-278: first on WatAST is now bare-raising (returns :wat::WatAST directly).

#[test]
fn first_on_watast_list() {
    // Verify that (first <WatAST List>) type-checks (returns bare :wat::WatAST) and runs.
    // The result is a WatAST node; we assert the eval succeeds and produces a WatAST value.
    match eval_probe(":p::first-watast") {
        Ok(Value::wat__WatAST(_)) => {}
        Ok(other) => panic!("expected WatAST; got {other:?}"),
        Err(e) => panic!("checker≡runtime drift (should type-check + run): {e}"),
    }
}

#[test]
fn rest_on_watast_list() {
    expect_true(":p::rest-watast");
}
