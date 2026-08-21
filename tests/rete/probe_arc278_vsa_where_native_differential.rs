//! BRIEF-native-where-vsa-ops — native `where` must execute the VSA seam.
//!
//! Oracle (`fire-rules$oracle` → `eval_test` → `dispatch_rete_op`) already
//! unwraps `CosineOutcome` / `DotOutcome` and runs `coincident?` / `presence?`.
//! Native (`fire-rules` → `exec_where` → `apply_op`) was holon-blind:
//! cosine/`dot` silent-missed via CallFallback; the predicates raised
//! `compiled apply cannot dispatch`. This differential dies if that hole
//! reopens. Shape copied from `probe_arc278_7strat_native_differential.rs`.
//!
//! Run: cargo test --release -p wat --test rete probe_arc278_vsa_where_native_differential

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn call_str(entry: &str) -> String {
    match call_beside_value(file!(), entry) {
        Ok(Value::String(s)) => (*s).clone(),
        Ok(other) => panic!("{entry}: expected String, got {other:?}"),
        Err(e) => panic!("{entry}: eval raised: {e:?}"),
    }
}

fn call_i64s(entry: &str, n: usize) -> Vec<i64> {
    match call_beside_value(file!(), entry) {
        Ok(Value::wat__core__PersistentVector(v)) => (0..n)
            .map(|i| match v.get(i) {
                Some(Value::i64(x)) => *x,
                other => panic!("{entry} slot {i}: expected i64; got {other:?}"),
            })
            .collect(),
        Ok(other) => panic!("{entry}: expected PV; got {other:?}"),
        Err(e) => panic!("{entry}: eval raised: {e:?}"),
    }
}

fn assert_named(oracle_entry: &str, native_entry: &str, want: &str) {
    let oracle = call_str(oracle_entry);
    let native = call_str(native_entry);
    assert_eq!(
        native, oracle,
        "native==oracle on VSA where; native={native:?} oracle={oracle:?} want={want}"
    );
    assert_eq!(
        native, want,
        "{native_entry} must Guess {want:?} exactly once (four-row catalog); got {native:?}"
    );
}

#[test]
fn differential_cosine_identity() {
    assert_named(":user::oracle-id", ":user::native-id", "identity");
}

#[test]
fn differential_cosine_not() {
    assert_named(":user::oracle-not", ":user::native-not", "not");
}

#[test]
fn differential_cosine_const_true() {
    assert_named(
        ":user::oracle-const-true",
        ":user::native-const-true",
        "const-true",
    );
}

#[test]
fn differential_cosine_const_false() {
    assert_named(
        ":user::oracle-const-false",
        ":user::native-const-false",
        "const-false",
    );
}

#[test]
fn native_identity_does_not_guess_not() {
    let native = call_str(":user::native-id");
    assert_ne!(native, "not", "identity must not also Guess not");
}

#[test]
fn differential_degenerate_takes_caller_undefined() {
    let oracle = call_i64s(":user::oracle-deg", 2);
    let native = call_i64s(":user::native-deg", 2);
    assert_eq!(
        native, oracle,
        "native==oracle on degenerate cosine where; native={native:?} oracle={oracle:?}"
    );
    assert_eq!(
        native,
        vec![1, 1],
        "degenerate cosine must take caller :undefined (-1.0 and 7.0), not a constant; got {native:?}"
    );
}

#[test]
fn differential_coincident_identity() {
    assert_named(
        ":user::oracle-coincident-id",
        ":user::native-coincident-id",
        "identity",
    );
}

#[test]
fn differential_coincident_not() {
    assert_named(
        ":user::oracle-coincident-not",
        ":user::native-coincident-not",
        "not",
    );
}

#[test]
fn differential_presence_four_row_native_eq_oracle() {
    // presence-floor is looser than cosine>0.9, so four tables can yield
    // more than one hit. The hole was a native RAISE; native==oracle is
    // the gate. Self/orthogonal below pin it is not silent-false.
    let oracle = call_str(":user::oracle-presence-id");
    let native = call_str(":user::native-presence-id");
    assert_eq!(
        native, oracle,
        "native==oracle on presence? where; native={native:?} oracle={oracle:?}"
    );
    assert_ne!(native, "count=0", "presence? must not silent-miss identity");
}

#[test]
fn differential_presence_self() {
    assert_named(
        ":user::oracle-presence-self",
        ":user::native-presence-self",
        "identity",
    );
}

#[test]
fn differential_presence_orthogonal() {
    assert_named(
        ":user::oracle-presence-orthogonal",
        ":user::native-presence-orthogonal",
        "count=0",
    );
}
