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
//!
//! Wat source lives in the co-located fixture: probe_seq_container_registry.wat
//! (slurped via startup_beside(file!())).

use wat::freeze::{call_beside_value, startup_from_file, StartupError};
use wat::runtime::Value;

fn eval_probe(fn_name: &str) -> Result<Value, StartupError> {
    call_beside_value(file!(), fn_name).map_err(|e| StartupError::Runtime(Box::new(e)))
}

fn expect_i64(call: &str, want: i64) {
    match eval_probe(call) {
        Ok(Value::i64(n)) => assert_eq!(n, want, "value: got {n} want {want}"),
        Ok(other) => panic!("expected i64({want}); got {other:?}"),
        Err(e) => panic!("Indexable container should type-check + run: {e}"),
    }
}


fn expect_bool(call: &str, want: bool) {
    match eval_probe(call) {
        Ok(Value::bool(b)) => assert_eq!(b, want, "expected bool({want}); got bool({b})"),
        Ok(other) => panic!("expected bool({want}); got {other:?}"),
        Err(e) => panic!("should type-check + run: {e}"),
    }
}

fn expect_option_i64(call: &str, want: Option<i64>) {
    match eval_probe(call) {
        Ok(Value::Option(inner)) => match (inner.as_ref(), want) {
            (Some(Value::i64(n)), Some(w)) => assert_eq!(*n, w, "Option<i64>: got {n} want {w}"),
            (None, None) => {}
            (got, _) => panic!("expected Option<i64>({want:?}); got {got:?}"),
        },
        Ok(other) => panic!("expected Option; got {other:?}"),
        Err(e) => panic!("should type-check + run: {e}"),
    }
}

// ── Indexable ✓ : first → element 0, across every ordered container ──

#[test]
fn first_vector() {
    expect_i64(":p::first-vector", 10);
}

#[test]
fn first_persistent_vector() {
    expect_i64(":p::first-persistent-vector", 10);
}

#[test]
fn first_list() {
    expect_i64(":p::first-list", 10);
}

#[test]
fn first_tuple() {
    // Tuple is TOTAL: arity is statically known, so `first` was always bare T (never Option<T>).
    // Vec/List/PV are also now bare-raising after arc-278. All containers: first → bare T.
    expect_i64(":p::first-tuple", 10);
}

#[test]
fn first_watast_list() {
    // arc-278: first on WatAST is now bare-raising — returns :wat::WatAST directly.
    // Verify it type-checks (return type :wat::WatAST) and produces a WatAST value at runtime.
    match eval_probe(":p::first-watast") {
        Ok(Value::wat__WatAST(_)) => {}
        other => panic!("first on WatAstList should return bare WatAST; got {other:?}"),
    }
}

// ── index variants on a Vector (second/third) ──

#[test]
fn second_vector() {
    expect_i64(":p::second-vector", 20);
}

#[test]
fn third_vector() {
    expect_i64(":p::third-vector", 30);
}

// ── ∅ N/A : HashSet is unordered → first is meaningless → rejected on both sides ──

#[test]
fn first_hashset_rejected() {
    // The bad fixture has a defn that calls (first <HashSet>) — must fail at type-check.
    match startup_from_file("tests/collection/probe_seq_container_registry_hashset_first.wat.bad") {
        Err(_) => {}
        Ok(v) => panic!("expected rejection (∅ N/A: container is not Indexable); got {v:?}"),
    }
}

// ── seq-1b additions: measurable (length/empty?) ──────────────────────────────

#[test]
fn tuple_length() {
    expect_i64(":p::tuple-length", 3);
}

#[test]
fn tuple_empty_q_false() {
    expect_bool(":p::tuple-empty-false", false);
}

#[test]
fn tuple_empty_q_single() {
    // Tuples must have at least one element (empty Tuple is rejected by the checker).
    // Verify a single-element Tuple is also non-empty.
    expect_bool(":p::tuple-empty-single", false);
}

#[test]
fn watastlist_length() {
    // `(:wat::core::quote (a b c))` produces a WatAST::List with 3 children.
    expect_i64(":p::watastlist-length", 3);
}

#[test]
fn watastlist_empty_q_false() {
    match eval_probe(":p::watastlist-empty-false") {
        Ok(Value::bool(false)) => {}
        other => panic!("expected bool(false); got {other:?}"),
    }
}

// ── seq-1b additions: searchable (contains?) ──────────────────────────────────

#[test]
fn list_contains_q_found() {
    expect_bool(":p::list-contains-found", true);
}

#[test]
fn list_contains_q_not_found() {
    expect_bool(":p::list-contains-not-found", false);
}

#[test]
fn tuple_contains_q_found() {
    expect_bool(":p::tuple-contains-found", true);
}

#[test]
fn tuple_contains_q_not_found() {
    expect_bool(":p::tuple-contains-not-found", false);
}

#[test]
fn watastlist_contains_q_found() {
    expect_bool(":p::watastlist-contains-found", true);
}

#[test]
fn watastlist_contains_q_not_found() {
    // `a` and `x` are different symbols so not-found.
    expect_bool(":p::watastlist-contains-not-found", false);
}

// ── seq-1b additions: gettable (get → Option) ─────────────────────────────────

#[test]
fn list_get_found() {
    expect_option_i64(":p::list-get-found", Some(20));
}

#[test]
fn list_get_out_of_bounds() {
    expect_option_i64(":p::list-get-oob", None);
}

#[test]
fn watastlist_get_found() {
    match eval_probe(":p::watastlist-get-found") {
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
    match eval_probe(":p::watastlist-get-oob") {
        Ok(Value::Option(inner)) => assert!(inner.is_none(), "expected None; got {inner:?}"),
        Ok(other) => panic!("expected Option<None>; got {other:?}"),
        Err(e) => panic!("watastlist get oob should run: {e}"),
    }
}

#[test]
fn hashset_get_found() {
    expect_option_i64(":p::hashset-get-found", Some(20));
}

#[test]
fn hashset_get_not_found() {
    expect_option_i64(":p::hashset-get-not-found", None);
}
