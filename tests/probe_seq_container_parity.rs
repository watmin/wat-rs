//! Seq/collection checker↔runtime CONTAINER PARITY — the drift tripwire.
//!
//! Three collection ops accept a container at RUNTIME that the TYPE-CHECKER rejects (false-reject drift,
//! one-sided changes where a new container repr was added to runtime but not check.rs):
//!   - `first`/`second`/`third` (infer_positional_accessor, check.rs) MISSING PersistentVector + WatAST::List
//!   - `rest`                   (check.rs:5301)                       MISSING PersistentVector + WatAST::List
//!   - `conj`                   (infer_conj, collection/infer.rs)     MISSING List
//! The runtime (runtime.rs / collection/*.rs) handles all of these correctly. RED at HEAD: each probe defn's
//! body type-errors → `startup_from_source` returns Err. GREEN when the checker arms are extended to equal the
//! runtime's accepted container set. This pins checker≡runtime so any FUTURE one-sided arm goes red.
//! Contract: DESIGN-STONE-seq-container-drift.md.
//!
//! Run: cargo test --release -p wat --test probe_seq_container_parity

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Build a world from one probe `defn`, start it (TYPE-CHECK fires here), then eval `call`.
/// At HEAD a drifted op makes `startup_from_source` return Err → the test's assertion fails = RED.
fn eval_probe(defn: &str, call: &str) -> Result<Value, String> {
    let world = format!("{defn}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)");
    let w = startup_from_source(&world, Some(concat!(file!(), ":", line!())), Arc::new(InMemoryLoader::new()))
        .map_err(|e| format!("startup (type-check): {e:?}"))?;
    let ast = wat::parse_one!(call).map_err(|e| format!("parse: {e:?}"))?;
    eval_in_frozen(&ast, &w, &Environment::new())
        .map_err(|e| format!("eval: {e:?}"))
        .map(|tv| tv.value_owned())
}

fn expect_i64(defn: &str, call: &str, want: i64) {
    match eval_probe(defn, call) {
        Ok(Value::i64(n)) => assert_eq!(n, want, "value: got {n} want {want}"),
        Ok(other) => panic!("expected i64({want}); got {other:?}"),
        Err(e) => panic!("checker≡runtime drift (should type-check + run): {e}"),
    }
}

fn expect_true(defn: &str, call: &str) {
    match eval_probe(defn, call) {
        Ok(Value::bool(true)) => {}
        Ok(other) => panic!("expected bool(true); got {other:?}"),
        Err(e) => panic!("checker≡runtime drift (should type-check + run): {e}"),
    }
}

// ── first/second/third on PersistentVector → Option<T> (strong value asserts; the rete-relevant repr) ──

#[test]
fn first_on_persistent_vector() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::i64 \
                (:wat::core::Option/expect -> :wat::core::i64 \
                  (:wat::core::first (:wat::core::PersistentVector 10 20 30)) \"empty\"))";
    expect_i64(defn, "(:p::f)", 10);
}

#[test]
fn second_on_persistent_vector() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::i64 \
                (:wat::core::Option/expect -> :wat::core::i64 \
                  (:wat::core::second (:wat::core::PersistentVector 10 20 30)) \"empty\"))";
    expect_i64(defn, "(:p::f)", 20);
}

#[test]
fn third_on_persistent_vector() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::i64 \
                (:wat::core::Option/expect -> :wat::core::i64 \
                  (:wat::core::third (:wat::core::PersistentVector 10 20 30)) \"empty\"))";
    expect_i64(defn, "(:p::f)", 30);
}

// ── rest on PersistentVector → PersistentVector<T> (identity preserved; length-of-tail = 2) ──

#[test]
fn rest_on_persistent_vector() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::i64 \
                (:wat::core::PersistentVector/length \
                  (:wat::core::rest (:wat::core::PersistentVector 10 20 30))))";
    expect_i64(defn, "(:p::f)", 2);
}

// ── conj on List → List<T> (the arc-220 repr the checker forgot; length-after-conj = 3) ──

#[test]
fn conj_on_list() {
    // `:wat::core::List/of` is the List constructor (variadic, no type keyword; check.rs:4073).
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::i64 \
                (:wat::core::length \
                  (:wat::core::conj (:wat::core::List/of 1 2) 3)))";
    expect_i64(defn, "(:p::f)", 3);
}

// ── WatAST::List (arc-249 form-values): first/rest must type-check + run (compiles-and-runs asserts) ──

#[test]
fn first_on_watast_list() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::bool \
                (:wat::core::match (:wat::core::first (:wat::core::quote (a b c))) -> :wat::core::bool \
                  ((:wat::core::Some _) true) \
                  (:wat::core::None false)))";
    expect_true(defn, "(:p::f)");
}

#[test]
fn rest_on_watast_list() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::bool \
                (:wat::core::let [_r (:wat::core::rest (:wat::core::quote (a b c)))] true))";
    expect_true(defn, "(:p::f)");
}
