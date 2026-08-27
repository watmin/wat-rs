//! Arc 278 — `insert-all` joins the dual-impl: batch insert becomes the real primitive, and the
//! existing 2-ary `insert` gains a variadic clause as sugar over it.
//!
//! Clara's primitive is the *batch* form and the single-fact call is sugar over it
//! (`rules.cljc:11,17` — both delegate to `(eng/insert session facts)`). We shipped only the
//! degenerate case (`insert$native`, ~1.03 µs of pure 8-field Session rebuild per fact
//! above a bare `conj` — `DESIGN-STONE-insert-all.md`). Dual-impl mouths are live:
//! `insert-all$oracle` / `insert-all$native` / `insert-all`.
//!
//! The trio this extends (unprimed fire-rules / insert-all / fire-once are native;
//! `$oracle` is the reference):
//!   `insert$oracle` / `insert$native` / `insert`  ->  `insert-all$oracle` / `insert-all$native` / `insert-all`
//!
//! What would turn this red once it is green — the R59 question, answered before the assertions
//! were written:
//!   (a) `insert-all` writing the wrong `Session` slot (a positional-index assumption instead of
//!       resolving `facts` by name through `RecordDef.field_names`) — `staged` would drift, or
//!       `fired`/`sum` would collapse to 0;
//!   (b) `insert-all` silently returning the session UNCHANGED (a no-op) — assertions 1 and 2
//!       would both pass vacuously against an empty fact vector; assertion 3 (N > 1, `facts`
//!       length == N exactly) is the ONLY thing that would catch it;
//!   (c) the public `insert-all` drifting into a second implementation instead of a delegate to
//!       `insert-all$native` — invisible to a test that only ever calls the public verb, which is why
//!       assertion 2 compares `insert-all$oracle` to `insert-all$native` directly;
//!   (d) the 2-ary `insert` being silently RE-ROUTED through `insert-all` (STOP-1, the ONE
//!       contract decision this stone exists to enforce) — every other assertion here would miss
//!       it. Assertion 4 checks it by BEHAVIOUR: a lone 2-ary `insert` call must match `insert$native`
//!       called directly, fact for fact. (The companion form-level proof — that
//!       `wat/rete/oracle/insert.wat`'s 2-ary clause body is `(:wat::rete::insert$native session
//!       fact)` with no reference to `insert-all` — was read by hand against the source; a test
//!       cannot introspect a `defclause`'s per-arm body without reproducing the checker.)
//!
//! Run: cargo test --release -p wat --test probe_arc278_insert_all_differential

use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

fn count(entry: &str) -> Result<i64, RuntimeError> {
    match call_beside_value(file!(), entry)? {
        Value::i64(n) => Ok(n),
        other => Err(RuntimeError::new(
            wat::rust_caller_span!(),
            RuntimeErrorKind::TypeMismatch {
                op: format!("count({entry})"),
                expected: "i64",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )),
    }
}

/// 1 — EQUIVALENCE: `insert-all(s, [f1..f5])` produces a Session structurally identical to 5
/// chained `insert` calls. The load-bearing correctness claim.
#[test]
fn equivalence_batch_matches_chained_insert() {
    let batch_staged = count(":user::batch-staged").expect("batch staged");
    let chained_staged = count(":user::chained-staged").expect("chained staged");
    assert_eq!(batch_staged, chained_staged, "batch==chained (staged); batch={batch_staged} chained={chained_staged}");
    assert_eq!(batch_staged, 5, "five facts via insert-all must stage to five; got {batch_staged}");

    let batch_fired = count(":user::batch-fired").expect("batch fired");
    let chained_fired = count(":user::chained-fired").expect("chained fired");
    assert_eq!(batch_fired, chained_fired, "batch==chained (fired); batch={batch_fired} chained={chained_fired}");
    assert_eq!(batch_fired, 5, "five staged Readings must derive five Outs; got {batch_fired}");

    // Content, not merely count: g of 0..4 sums to 10. A batch that dropped or reordered facts
    // could keep the count and lose the content.
    let batch_sum = count(":user::batch-sum").expect("batch sum");
    let chained_sum = count(":user::chained-sum").expect("chained sum");
    assert_eq!(batch_sum, chained_sum, "batch==chained (content); batch={batch_sum} chained={chained_sum}");
    assert_eq!(batch_sum, 10, "g of 0..4 sums to 10 — the CONTENT landed; got {batch_sum}");
}

/// 2 — THE ORACLE: `insert-all$oracle` (the wat reference engine) == `insert-all$native` (the native
/// prime) on the same input. The dual-impl is never skipped.
#[test]
fn oracle_matches_native_prime() {
    let oracle_staged = count(":user::oracle-staged").expect("oracle staged");
    let native_staged = count(":user::native-staged").expect("native staged");
    assert_eq!(oracle_staged, native_staged, "oracle==native (staged); oracle={oracle_staged} native={native_staged}");

    let oracle_fired = count(":user::oracle-fired").expect("oracle fired");
    let native_fired = count(":user::native-fired").expect("native fired");
    assert_eq!(oracle_fired, native_fired, "oracle==native (fired); oracle={oracle_fired} native={native_fired}");

    let oracle_sum = count(":user::oracle-sum").expect("oracle sum");
    let native_sum = count(":user::native-sum").expect("native sum");
    assert_eq!(oracle_sum, native_sum, "oracle==native (content); oracle={oracle_sum} native={native_sum}");
}

/// 3 — NON-VACUITY: N > 1 and the resulting `facts` length is EXACTLY N. A no-op `insert-all`
/// that returned the session unchanged would pass assertions 1 and 2 against an empty vector —
/// this is the only row that would catch that.
#[test]
fn non_vacuity_n_greater_than_one_and_exact() {
    let n = count(":user::n-under-test").expect("n");
    assert!(n > 1, "the differential is meaningless at N<=1; got N={n}");
    assert_eq!(n, 5, "the fact vector under test has 5 elements; got {n}");

    let batch_len = count(":user::batch-facts-len").expect("batch facts len");
    assert_eq!(batch_len, n, "insert-all must land EXACTLY N facts, not fewer/more/zero; got {batch_len} for N={n}");
}

/// 4 — THE 2-ARY PATH IS UNTOUCHED (STOP-1): a single 2-ary `insert` call must still match
/// `insert$native` called directly, fact for fact — proving the hot path was not silently re-routed
/// through the new batch primitive.
#[test]
fn two_ary_insert_is_not_rerouted_through_insert_all() {
    let public_staged = count(":user::single-public-staged").expect("public staged");
    let native_staged = count(":user::single-native-staged").expect("native staged");
    assert_eq!(public_staged, native_staged, "2-ary insert==insert$native (staged); public={public_staged} native={native_staged}");
    assert_eq!(public_staged, 1, "a single 2-ary insert must stage exactly one fact; got {public_staged}");

    let public_fired = count(":user::single-public-fired").expect("public fired");
    let native_fired = count(":user::single-native-fired").expect("native fired");
    assert_eq!(public_fired, native_fired, "2-ary insert==insert$native (fired); public={public_fired} native={native_fired}");
    assert_eq!(public_fired, 1, "a single 2-ary insert must derive exactly one Out; got {public_fired}");
}
