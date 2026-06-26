//! Seq-container registry — strike 1 net: the positional-accessor family across every container, pinning the
//! 3-state capability matrix as OBSERVABLE behavior (independent of the registry impl). Green at HEAD (behavior
//! is already correct after the drift fix); must STAY green as `first`/`second`/`third` are migrated to dispatch
//! through `src/collection/seq_container.rs` (the registry home the megafiles will dep on). This is the
//! behavior-preserving net for the refactor + proves the home's classify→capability(Indexable)→element path
//! end to end. Contract: DESIGN-STONE-seq-container-registry.md.
//!
//! Capability matrix row exercised here — Indexable (first/second/third):
//!   Vector ✓ · PersistentVector ✓ · List ✓ · Tuple ✓ · WatAstList ✓ · HashSet ∅ N/A (unordered → rejected).
//!
//! Run: cargo test --release -p wat --test probe_seq_container_registry

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

/// Build a world from one probe `defn`, start it (TYPE-CHECK fires here), then eval `call`.
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
        Err(e) => panic!("Indexable container should type-check + run: {e}"),
    }
}

/// `∅ N/A`: a non-indexable container MUST be rejected (here, at type-check → startup Err).
fn expect_rejected(defn: &str, call: &str) {
    match eval_probe(defn, call) {
        Err(_) => {}
        Ok(v) => panic!("expected rejection (∅ N/A: container is not Indexable); got {v:?}"),
    }
}

// A probe defn whose body is `(:wat::core::first <ctor>)` used BARE as i64 (arc-278: first is raising).
fn first_i64(ctor: &str) -> String {
    format!(
        "(:wat::core::defn :p::f [] -> :wat::core::i64 \
          (:wat::core::first {ctor}))"
    )
}

// ── Indexable ✓ : first → element 0, across every ordered container ──

#[test]
fn first_vector() {
    expect_i64(&first_i64("(:wat::core::Vector :wat::core::i64 10 20 30)"), "(:p::f)", 10);
}

#[test]
fn first_persistent_vector() {
    expect_i64(&first_i64("(:wat::core::PersistentVector 10 20 30)"), "(:p::f)", 10);
}

#[test]
fn first_list() {
    expect_i64(&first_i64("(:wat::core::List/of 10 20 30)"), "(:p::f)", 10);
}

#[test]
fn first_tuple() {
    // Tuple is TOTAL: arity is statically known, so `first` was always bare T (never Option<T>).
    // Vec/List/PV are also now bare-raising after arc-278. All containers: first → bare T.
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::i64 (:wat::core::first (:wat::core::Tuple 10 20)))";
    expect_i64(defn, "(:p::f)", 10);
}

#[test]
fn first_watast_list() {
    // arc-278: first on WatAST is now bare-raising — returns :wat::WatAST directly.
    // Verify it type-checks (return type :wat::WatAST) and produces a WatAST value at runtime.
    let defn = "(:wat::core::defn :p::f [] -> :wat::WatAST \
                (:wat::core::first (:wat::core::quote (a b c))))";
    match eval_probe(defn, "(:p::f)") {
        Ok(Value::wat__WatAST(_)) => {}
        other => panic!("first on WatAstList should return bare WatAST; got {other:?}"),
    }
}

// ── index variants on a Vector (second/third) ──

#[test]
fn second_vector() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::i64 \
                (:wat::core::second (:wat::core::Vector :wat::core::i64 10 20 30)))";
    expect_i64(defn, "(:p::f)", 20);
}

#[test]
fn third_vector() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::i64 \
                (:wat::core::third (:wat::core::Vector :wat::core::i64 10 20 30)))";
    expect_i64(defn, "(:p::f)", 30);
}

// ── ∅ N/A : HashSet is unordered → first is meaningless → rejected on both sides ──

#[test]
fn first_hashset_rejected() {
    expect_rejected(&first_i64("(:wat::core::HashSet :wat::core::i64 10 20 30)"), "(:p::f)");
}

// ── seq-1b additions: measurable (length/empty?) ──────────────────────────────

// Helper: build a defn whose body is `(op ctor)` returning i64.
fn len_defn(op: &str, ctor: &str) -> String {
    format!(
        "(:wat::core::defn :p::f [] -> :wat::core::i64 ({op} {ctor}))"
    )
}

// Helper: build a defn whose body is `(op ctor)` returning bool.
fn empty_defn(op: &str, ctor: &str) -> String {
    format!(
        "(:wat::core::defn :p::f [] -> :wat::core::bool ({op} {ctor}))"
    )
}

fn expect_bool(defn: &str, call: &str, want: bool) {
    match eval_probe(defn, call) {
        Ok(Value::bool(b)) => assert_eq!(b, want, "expected bool({want}); got bool({b})"),
        Ok(other) => panic!("expected bool({want}); got {other:?}"),
        Err(e) => panic!("should type-check + run: {e}"),
    }
}

#[test]
fn tuple_length() {
    expect_i64(&len_defn(":wat::core::length", "(:wat::core::Tuple 10 20 30)"), "(:p::f)", 3);
}

#[test]
fn tuple_empty_q_false() {
    expect_bool(&empty_defn(":wat::core::empty?", "(:wat::core::Tuple 10 20 30)"), "(:p::f)", false);
}

#[test]
fn tuple_empty_q_single() {
    // Tuples must have at least one element (empty Tuple is rejected by the checker).
    // Verify a single-element Tuple is also non-empty.
    expect_bool(&empty_defn(":wat::core::empty?", "(:wat::core::Tuple 42)"), "(:p::f)", false);
}

#[test]
fn watastlist_length() {
    // `(:wat::core::quote (a b c))` produces a WatAST::List with 3 children.
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::i64 \
                (:wat::core::length (:wat::core::quote (a b c))))";
    expect_i64(defn, "(:p::f)", 3);
}

#[test]
fn watastlist_empty_q_false() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::bool \
                (:wat::core::empty? (:wat::core::quote (a b c))))";
    match eval_probe(defn, "(:p::f)") {
        Ok(Value::bool(false)) => {}
        other => panic!("expected bool(false); got {other:?}"),
    }
}

// ── seq-1b additions: searchable (contains?) ──────────────────────────────────

#[test]
fn list_contains_q_found() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::bool \
                (:wat::core::contains? (:wat::core::List/of 10 20 30) 20))";
    expect_bool(defn, "(:p::f)", true);
}

#[test]
fn list_contains_q_not_found() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::bool \
                (:wat::core::contains? (:wat::core::List/of 10 20 30) 99))";
    expect_bool(defn, "(:p::f)", false);
}

#[test]
fn tuple_contains_q_found() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::bool \
                (:wat::core::contains? (:wat::core::Tuple 10 20 30) 20))";
    expect_bool(defn, "(:p::f)", true);
}

#[test]
fn tuple_contains_q_not_found() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::bool \
                (:wat::core::contains? (:wat::core::Tuple 10 20 30) 99))";
    expect_bool(defn, "(:p::f)", false);
}

#[test]
fn watastlist_contains_q_found() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::bool \
                (:wat::core::contains? (:wat::core::quote (a b c)) \
                  (:wat::core::first (:wat::core::quote (a b c)))))";
    expect_bool(defn, "(:p::f)", true);
}

#[test]
fn watastlist_contains_q_not_found() {
    // `a` and `x` are different symbols so not-found.
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::bool \
                (:wat::core::contains? (:wat::core::quote (a b c)) \
                  (:wat::core::first (:wat::core::quote (x y z)))))";
    expect_bool(defn, "(:p::f)", false);
}

// ── seq-1b additions: gettable (get → Option) ─────────────────────────────────

fn expect_option_i64(defn: &str, call: &str, want: Option<i64>) {
    match eval_probe(defn, call) {
        Ok(Value::Option(inner)) => match (inner.as_ref(), want) {
            (Some(Value::i64(n)), Some(w)) => assert_eq!(*n, w, "Option<i64>: got {n} want {w}"),
            (None, None) => {}
            (got, _) => panic!("expected Option<i64>({want:?}); got {got:?}"),
        },
        Ok(other) => panic!("expected Option; got {other:?}"),
        Err(e) => panic!("should type-check + run: {e}"),
    }
}

#[test]
fn list_get_found() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::Option<wat::core::i64> \
                (:wat::core::get (:wat::core::List/of 10 20 30) 1))";
    expect_option_i64(defn, "(:p::f)", Some(20));
}

#[test]
fn list_get_out_of_bounds() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::Option<wat::core::i64> \
                (:wat::core::get (:wat::core::List/of 10 20 30) 99))";
    expect_option_i64(defn, "(:p::f)", None);
}

#[test]
fn watastlist_get_found() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::Option<wat::WatAST> \
                (:wat::core::get (:wat::core::quote (a b c)) 1))";
    match eval_probe(defn, "(:p::f)") {
        Ok(Value::Option(inner)) => match inner.as_ref() {
            Some(Value::wat__WatAST(_)) => {}
            other => panic!("expected Option<Some(WatAST)>; got {other:?}"),
        },
        Ok(other) => panic!("expected Option<WatAST>; got {other:?}"),
        Err(e) => panic!("watastlist get should run: {e}"),
    }
}

#[test]
fn watastlist_get_out_of_bounds() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::Option<wat::WatAST> \
                (:wat::core::get (:wat::core::quote (a b c)) 99))";
    match eval_probe(defn, "(:p::f)") {
        Ok(Value::Option(inner)) => assert!(inner.is_none(), "expected None; got {inner:?}"),
        Ok(other) => panic!("expected Option<None>; got {other:?}"),
        Err(e) => panic!("watastlist get oob should run: {e}"),
    }
}

#[test]
fn hashset_get_found() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::Option<wat::core::i64> \
                (:wat::core::get (:wat::core::HashSet :wat::core::i64 10 20 30) 20))";
    expect_option_i64(defn, "(:p::f)", Some(20));
}

#[test]
fn hashset_get_not_found() {
    let defn = "(:wat::core::defn :p::f [] -> :wat::core::Option<wat::core::i64> \
                (:wat::core::get (:wat::core::HashSet :wat::core::i64 10 20 30) 99))";
    expect_option_i64(defn, "(:p::f)", None);
}
