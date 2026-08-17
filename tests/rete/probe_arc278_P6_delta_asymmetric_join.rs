//! Arc 278 P6 — delta hash-join asymmetric arrival DIFFERENTIAL gate.
//!
//! Root cause under test: the `fire_fixpoint_delta` hash-join step cached `join_keys[J]` lazily
//! from samples. If the RIGHT side of a join (the alpha memory) arrived in an EARLIER round than
//! the LEFT side (the beta/token memory), the join node J was skipped for every prior round and
//! `right_idx[J]` was never populated. When the left side finally arrived, the right index was
//! empty → zero matches → derived fact dropped (C=0 instead of C=2 in the chain case).
//!
//! Fix (P6 catch-up): on first keying of J, rebuild `right_idx[J]` and `left_idx[J]` from ALL
//! cumulative wm.alpha/wm.beta and emit a full cross-join. Zero double-count risk because J
//! produced nothing before first keying.
//!
//! Cases:
//!   1. Chain (the minimal repro): R1: A→B, R2: B⋈A→C.  Insert A(1),A(2).  C=2.
//!   2. 3-level cascade:           R1: A→B, R2: B⋈A→C, R3: C⋈B→D.  Insert A(1),A(2).  D=2.
//!   3. Left arrives before right: X(?k)⋈Y(?k)→Z.  Insert X(1),X(2) THEN Y(1),Y(2) (all before fire).
//!   4. Right arrives before left (classic bug case): same R2 join but with N=5 inputs.
//!
//! Run: cargo nextest run -p wat -E 'test(/P6_delta_asymmetric/)'

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::{Environment, Value};

/// Run a WAT expression against the co-located `.wat` world (all record types for the three scenarios).
/// The rules are constructed at runtime inside `expr` (parameterized by N), so only records live in the
/// fixture. Returns the `Value` produced.
// rune:lint(no-inlined-wat) — expr parameterized by runtime N (loop-generated fact inserts, N up to 5)
// and the FIRE_VERB placeholder swapped per differential side — cannot be pre-extracted to a static .wat file
fn run_expr(expr: &str) -> Value {
    let world = startup_beside(file!()).expect("startup_beside");
    let ast = wat::parse_one!(expr).expect("parse_one");
    eval_in_frozen(&ast, &world, &Environment::new())
        .unwrap_or_else(|e| panic!("eval raised: {e:?}"))
        .value_owned()
}

/// Run the same expression with BOTH `fire-rules'` (native delta) and `fire-rules-spec` (oracle),
/// assert they agree, and return the common count.
fn assert_native_eq_oracle(expr_template: &str, type_str: &str) -> i64 {
    let native_expr = expr_template.replace("FIRE_VERB", ":wat::rete::fire-rules'");
    let oracle_expr = expr_template.replace("FIRE_VERB", ":wat::rete::fire-rules-spec");
    let native = run_expr(&native_expr);
    let oracle = run_expr(&oracle_expr);
    assert_eq!(
        native, oracle,
        "native fire-rules' must match oracle fire-rules-spec for type {type_str}; native={native:?} oracle={oracle:?}"
    );
    match native {
        Value::i64(n) => n,
        other => panic!("expected i64 count, got {other:?}"),
    }
}

// ─── Case 1: chain (the minimal bug repro) ───────────────────────────────────
//
// R1: A(?k) → B(?k)
// R2: B(?k) ⋈ A(?k) → C(?k)
// Insert A(1), A(2). Expected: B=2, C=2.
//
// Bug: A (right of R2's hash join) arrives in round 1 while B (left) is not yet derived.
// J is skipped; right_idx[J] never populated. Round 2: B arrives but right_idx is empty → C=0.

fn q_call(ty: &str) -> String {
    let (ns, name) = ty.rsplit_once("::").expect("namespaced type");
    format!("(:{ns}::q-{name})")
}

/// Build the chain let-expression for `N` input A records. `FIRE_VERB` is a placeholder.
fn chain_expr(n: usize, query_type: &str) -> String {
    let r1c = "(:wat::core::quote (:chain::A (?k <- :k)))";
    let r1t = "(:wat::core::quote (:chain::B ?k))";
    let r2c1 = "(:wat::core::quote (:chain::B (?k <- :k)))";
    let r2c2 = "(:wat::core::quote (:chain::A (?k <- :k)))";
    let r2t = "(:wat::core::quote (:chain::C ?k))";
    let q = q_call(query_type);
    let mut binds = format!(
        "  r1 (:wat::rete::Rule :name \"r1\" \
             :lhs (:wat::core::PersistentVector {r1c}) \
             :rhs (:wat::core::PersistentVector {r1t}))\n\
         r2 (:wat::rete::Rule :name \"r2\" \
             :lhs (:wat::core::PersistentVector {r2c1} {r2c2}) \
             :rhs (:wat::core::PersistentVector {r2t}))\n\
         s0 (:wat::rete::compile-all (:wat::core::PersistentVector r1 r2) (:wat::core::PersistentVector {q}))\n"
    );
    let mut prev = 0usize;
    for i in 1..=n {
        let cur = i;
        binds.push_str(&format!(
            "  s{cur} (:wat::rete::insert s{prev} (:chain::A :k {i}))\n"
        ));
        prev = cur;
    }
    format!(
        "(:wat::core::let [{binds}\n\
           fired (FIRE_VERB s{prev})]\n\
           (:wat::core::length (:wat::rete::query fired {q})))"
    )
}

#[test]
fn chain_b_derived_equals_oracle() {
    let count = assert_native_eq_oracle(&chain_expr(2, "chain::B"), "chain::B");
    assert_eq!(count, 2, "R1 derives B for each A; expected B=2, got {count}");
}

#[test]
fn chain_c_join_equals_oracle() {
    // THE bug case: C was 0 before the fix; oracle gives 2.
    let count = assert_native_eq_oracle(&chain_expr(2, "chain::C"), "chain::C");
    assert_eq!(count, 2, "R2 joins each B with matching A → C=2; got {count}");
}

#[test]
fn chain_c_five_inputs_equals_oracle() {
    // Stress: N=5 inputs.
    let count = assert_native_eq_oracle(&chain_expr(5, "chain::C"), "chain::C");
    assert_eq!(count, 5, "5 A inputs → 5 C outputs; got {count}");
}

// ─── Case 2: 3-level cascade with derived⋈input joins ──────────────────────
//
// R1: A(?k) → B(?k)
// R2: B(?k) ⋈ A(?k) → C(?k)    [derived⋈input]
// R3: C(?k) ⋈ B(?k) → D(?k)    [derived⋈derived]
// Insert A(1), A(2). Expected: B=2, C=2, D=2.

fn triple_expr(n: usize, query_type: &str) -> String {
    let q = q_call(query_type);
    let mut binds = format!(
        "\
        r1 (:wat::rete::Rule :name \"r1\" \
             :lhs (:wat::core::PersistentVector (:wat::core::quote (:tri::A (?k <- :k)))) \
             :rhs (:wat::core::PersistentVector (:wat::core::quote (:tri::B ?k))))\n\
        r2 (:wat::rete::Rule :name \"r2\" \
             :lhs (:wat::core::PersistentVector \
               (:wat::core::quote (:tri::B (?k <- :k))) \
               (:wat::core::quote (:tri::A (?k <- :k)))) \
             :rhs (:wat::core::PersistentVector (:wat::core::quote (:tri::C ?k))))\n\
        r3 (:wat::rete::Rule :name \"r3\" \
             :lhs (:wat::core::PersistentVector \
               (:wat::core::quote (:tri::C (?k <- :k))) \
               (:wat::core::quote (:tri::B (?k <- :k)))) \
             :rhs (:wat::core::PersistentVector (:wat::core::quote (:tri::D ?k))))\n\
        s0 (:wat::rete::compile-all (:wat::core::PersistentVector r1 r2 r3) (:wat::core::PersistentVector {q}))\n"
    );
    let mut prev = 0usize;
    for i in 1..=n {
        binds.push_str(&format!("  s{i} (:wat::rete::insert s{prev} (:tri::A :k {i}))\n"));
        prev = i;
    }
    format!(
        "(:wat::core::let [{binds}\n\
           fired (FIRE_VERB s{prev})]\n\
           (:wat::core::length (:wat::rete::query fired {q})))"
    )
}

#[test]
fn triple_cascade_d_equals_oracle_n2() {
    // 3-level cascade: A→B (R1), B⋈A→C (R2, derived⋈input), C⋈B→D (R3, derived⋈derived).
    // Both R2 and R3 have asymmetric arrival: left arrives later than right.
    let count = assert_native_eq_oracle(&triple_expr(2, "tri::D"), "tri::D");
    assert_eq!(count, 2, "3-level cascade: 2 A inputs → D=2; got {count}");
}

#[test]
fn triple_cascade_d_equals_oracle_n5() {
    let count = assert_native_eq_oracle(&triple_expr(5, "tri::D"), "tri::D");
    assert_eq!(count, 5, "3-level cascade: 5 A inputs → D=5; got {count}");
}

#[test]
fn triple_cascade_all_types_equal_oracle() {
    // Verify every intermediate type too: B=2, C=2, D=2.
    for ty in ["tri::B", "tri::C", "tri::D"] {
        let count = assert_native_eq_oracle(&triple_expr(2, ty), ty);
        assert_eq!(count, 2, "expected 2 for {ty}, got {count}");
    }
}

// ─── Case 3: left arrives before right ──────────────────────────────────────
//
// R1: X(?k) ⋈ Y(?k) → Z(?k).  Insert ALL X first, then ALL Y (all before fire).
// No cascade here — both sides are input. But the hash-join sees X tokens first
// (from d_alpha[AlphaX]→root-join→d_beta) and Y elements only in the right delta.
// This is the "left before right" case.

fn xyz_expr(n: usize, query_type: &str) -> String {
    // Rule: X(?k) ⋈ Y(?k) → Z(?k).  X is the first (left) condition, Y is the second (right).
    let rule = "\
        r1 (:wat::rete::Rule :name \"r1\" \
             :lhs (:wat::core::PersistentVector \
               (:wat::core::quote (:xyz::X (?k <- :k))) \
               (:wat::core::quote (:xyz::Y (?k <- :k)))) \
             :rhs (:wat::core::PersistentVector (:wat::core::quote (:xyz::Z ?k))))\n\
        s0 (:wat::rete::compile-all (:wat::core::PersistentVector r1) (:wat::core::PersistentVector {q}))\n";
    let q = q_call(query_type);
    let mut binds = rule.replace("{q}", &q);
    let mut prev = 0usize;
    // Insert ALL X first (i=1..n), then ALL Y (i=1..n).
    // X seeds the left memory BEFORE Y arrives on the right.
    for i in 1..=n {
        let idx = prev + 1;
        binds.push_str(&format!("  s{idx} (:wat::rete::insert s{prev} (:xyz::X :k {i}))\n"));
        prev = idx;
    }
    for i in 1..=n {
        let idx = prev + 1;
        binds.push_str(&format!("  s{idx} (:wat::rete::insert s{prev} (:xyz::Y :k {i}))\n"));
        prev = idx;
    }
    format!(
        "(:wat::core::let [{binds}\n\
           fired (FIRE_VERB s{prev})]\n\
           (:wat::core::length (:wat::rete::query fired {q})))"
    )
}

#[test]
fn xyz_z_left_before_right_equals_oracle() {
    // X arrives before Y in the fact stream. Both arrive in the SAME initial fire round
    // (they are input facts, not derived). The join processes them in alpha-delta order.
    let count = assert_native_eq_oracle(&xyz_expr(3, "xyz::Z"), "xyz::Z");
    assert_eq!(count, 3, "X⋈Y→Z with matching k=1..3: expected Z=3, got {count}");
}

// ─── Case 4: right arrives before left (classic bug) — 5-input variant ──────
//
// Same chain network as case 1 but confirms the classic "right before left" scenario
// with N=5 at higher scale to stress-test the catch-up join.

#[test]
fn chain_classic_right_before_left_n5() {
    let count_b = assert_native_eq_oracle(&chain_expr(5, "chain::B"), "chain::B");
    let count_c = assert_native_eq_oracle(&chain_expr(5, "chain::C"), "chain::C");
    assert_eq!(count_b, 5, "B=5 for 5 A inputs; got {count_b}");
    assert_eq!(count_c, 5, "C=5: each B joins its matching A; got {count_c}");
}
