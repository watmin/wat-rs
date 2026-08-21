//! Arc 278 — `insert` joins the dual-impl: the wat form becomes the ORACLE, the native the user path.
//!
//! The builder's ruling (2026-07-31): *"we build correct but slow first, then we build the correct
//! and fast against it — the oracles must be a beacon of correctness then optimize against them.
//! Real wat-rete users should only use the rust native flavor; the wat interpreted path is just a
//! demonstration of correctness."* `insert` is the last hot verb where the interpreted form IS the
//! user path — `probe-insert-cost-split.wat` measured it at 87% of the per-fact cost (11.79 µs of
//! 13.54), and seeding is 74% of a real `accum` workload.
//!
//! Dual-impl law: unprimed public names are native; `$oracle` is the spec mouth;
//! `$native` is the kernel (`wat/rete/oracle/insert.wat`).
//!   `insert$oracle` / `insert$native` / `insert`
//!
//! What would turn this red once it is green — the R59 question, answered before the assertions
//! were written:
//!   (a) the native writing the wrong `Session` slot (a positional-index assumption instead of
//!       resolving `facts` by name) — `staged` would drift, or `fired` would collapse to 0;
//!   (b) the native dropping or reordering facts — `sum` catches content where `count` cannot;
//!   (c) the public `insert` quietly becoming a second implementation rather than a delegate —
//!       test 3 exists solely for that, because a drifting delegate is invisible to tests 1 and 2.
//!
//! Run: cargo test --release -p wat --test probe_arc278_native_insert_differential

use wat::freeze::call_beside_value;
use wat::runtime::Value;

fn count(entry: &str) -> Result<i64, String> {
    match call_beside_value(file!(), entry).map_err(|e| format!("eval: {e:?}"))? {
        Value::i64(n) => Ok(n),
        other => Err(format!("expected i64; got {other:?}")),
    }
}

/// 1 — DIFFERENTIAL: the native stages the same facts as the oracle.
#[test]
fn differential_native_stages_like_the_oracle() {
    let native = count(":user::native-staged").expect("native");
    let oracle = count(":user::spec-staged").expect("oracle");
    assert_eq!(native, oracle, "native==oracle (staged); native={native} oracle={oracle}");
    assert_eq!(native, 5, "five inserts must accumulate to five staged facts; got {native}");
}

/// 2 — DIFFERENTIAL: what landed is USABLE and is the RIGHT CONTENT, not merely the right count.
///
/// `fired` proves the Session each path built is structurally sound enough for the native kernel to
/// fire; `sum` proves the facts themselves carried through (0+1+2+3+4 = 10). A native that wrote a
/// plausible-but-wrong slot could keep the count and lose the content.
#[test]
fn differential_native_content_matches_the_oracle() {
    let native_fired = count(":user::native-fired").expect("native");
    let oracle_fired = count(":user::spec-fired").expect("oracle");
    assert_eq!(native_fired, oracle_fired, "native==oracle (fired); {native_fired} vs {oracle_fired}");
    assert_eq!(native_fired, 5, "five staged Readings must derive five Outs; got {native_fired}");

    let native_sum = count(":user::native-sum").expect("native");
    let oracle_sum = count(":user::spec-sum").expect("oracle");
    assert_eq!(native_sum, oracle_sum, "native==oracle (sum); {native_sum} vs {oracle_sum}");
    assert_eq!(native_sum, 10, "g of 0..4 sums to 10 — the CONTENT landed; got {native_sum}");
}

/// 3 — the public verb is a DELEGATE, not a third implementation.
///
/// `insert` keeps its name and signature so no call site churns; the risk that creates is that it
/// silently becomes its own implementation and drifts from the prime it is supposed to forward to.
/// Nothing in tests 1 or 2 would notice.
#[test]
fn public_insert_delegates_to_the_prime() {
    assert_eq!(
        count(":user::public-staged").expect("public"),
        count(":user::native-staged").expect("native"),
        "the public `insert` must forward to `insert$native` (staged)"
    );
    assert_eq!(
        count(":user::public-fired").expect("public"),
        count(":user::native-fired").expect("native"),
        "the public `insert` must forward to `insert$native` (fired)"
    );
    assert_eq!(
        count(":user::public-sum").expect("public"),
        count(":user::native-sum").expect("native"),
        "the public `insert` must forward to `insert$native` (content)"
    );
}
