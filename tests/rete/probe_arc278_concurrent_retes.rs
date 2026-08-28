//! Arc 278 — N CONCURRENT RETES, the correctness fuzz for the concurrency contract.
//!
//! `DESIGN-STONE-intern-lane-per-thread` removed the last shared mutable cell on
//! the fire path (a process-global `AtomicU64` bumped by every one-entry `PMap`).
//! That stone's evidence was a scaling probe over a counter in ISOLATION — it
//! never ran two real retes at once, and said so. Nothing else in the suite fired
//! more than one engine at a time.
//!
//! THE CONTRACT UNDER TEST: N concurrent rete instances share NOTHING. Each
//! worker owns its rules, network, session, memories and queries end to end;
//! nothing is passed between workers and no session is shared. The only thing
//! they have in common is the process — which is precisely where a global
//! would hide, invisible to every single-threaded test.
//!
//! This closes that hole with wat's own first-party thread pool
//! (`:wat::bracket::map (:wat::spawn::thread)`): 48 workers, each compiling,
//! seeding, firing and querying a complete engine of its own.
//!
//! CORRECTNESS ONLY — no timing is asserted anywhere here. Multithreaded
//! performance is out of scope for this gate: a duration assertion on a shared
//! box is a flake generator, and a red here has to mean cross-thread damage and
//! nothing else.
//!
//! WHAT A RED MEANS. Even workers run the `:cc` 3-stratum chain, odd workers the
//! `:dd` 2-stratum one, so two distinct compiled networks are live on the pool at
//! the same time. Each witness carries a rule-set TAG as well as its derived
//! counts, because the counts alone are identical between the two sets — without
//! the tag, a worker reading another thread's arm (the failure mode stone 27's
//! thread-local arm table exists to prevent) would be invisible. A mismatch here
//! is a mint-id collision, a cross-thread arm read, or a torn session.
//!
//! WHAT THIS GATE DOES NOT PROVE. It does not itself establish that the pool is
//! multi-threaded — a pool that silently ran every task on one thread would make
//! every assertion below pass vacuously. That property belongs to the bracket
//! layer and is covered by its own tests (`tests/kernel/probe_arc259_brackets_*`,
//! which drive `:wat::spawn::thread` with more items than cores). It is named
//! here rather than quietly assumed, because a vacuous green is worse than a red.
//! Timing was deliberately NOT used to prove it: perf is out of scope, and a
//! duration assertion would put a flake in a gate whose reds must all mean one
//! thing.
//!
//! Run: cargo test --release -p wat --test rete probe_arc278_concurrent_retes

use wat::freeze::call_beside_value;
use wat::runtime::{RuntimeError, RuntimeErrorKind, Value, ValueSnapshot};

/// Call a zero-arg entry point and read its `Vector<i64>` of witnesses.
fn witnesses(entry: &str) -> Result<Vec<i64>, RuntimeError> {
    let out = call_beside_value(file!(), entry)?;
    let items: Vec<&Value> = match &out {
        Value::wat__core__PersistentVector(v) => v.iter().collect(),
        Value::Vec(v) => v.iter().collect(),
        other => {
            return Err(RuntimeError::new(
                wat::rust_caller_span!(),
                RuntimeErrorKind::TypeMismatch {
                    op: format!("witnesses({entry})"),
                    expected: "vector",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            ))
        }
    };
    items
        .into_iter()
        .enumerate()
        .map(|(i, v)| match v {
            Value::i64(x) => Ok(*x),
            other => Err(RuntimeError::new(
                wat::rust_caller_span!(),
                RuntimeErrorKind::TypeMismatch {
                    op: format!("witnesses({entry}) slot {i}"),
                    expected: "i64",
                    got: Box::new(ValueSnapshot::of(other)),
                },
            )),
        })
        .collect()
}

/// Worker `i` seeds `100 + i` items. Even -> `:cc` (tag 1), odd -> `:dd` (tag 2);
/// both derive exactly one Bad and `n - 1` of the second type.
fn expected(i: i64) -> i64 {
    let n = 100 + i;
    let tag = if i % 2 == 0 { 1 } else { 2 };
    1_000_000 + (n - 1) * 1_000 + tag
}

/// The serial reference must be right before it can referee anything. A
/// reference that is only self-consistent would let a systematic error pass on
/// both sides of the comparison.
#[test]
fn serial_witnesses_are_the_known_closure() {
    let serial = witnesses(":user::cc-serial").expect("serial run");
    assert!(!serial.is_empty(), "no workers");
    let want: Vec<i64> = (0..serial.len() as i64).map(expected).collect();
    assert_eq!(
        serial, want,
        "each worker's closure is Bad=1 plus n-1 of its second type, tagged by rule set"
    );
}

/// THE FUZZ — 48 whole retes at once must agree, element for element, with the
/// same 48 run one at a time.
#[test]
fn concurrent_retes_match_serial() {
    let serial = witnesses(":user::cc-serial").expect("serial run");
    let concurrent = witnesses(":user::cc-concurrent").expect("concurrent run");

    assert_eq!(
        concurrent.len(),
        serial.len(),
        "the pool must return one witness per worker, in input order"
    );

    let bad: Vec<(usize, i64, i64)> = concurrent
        .iter()
        .zip(serial.iter())
        .enumerate()
        .filter(|(_, (c, s))| c != s)
        .map(|(i, (c, s))| (i, *c, *s))
        .collect();

    assert!(
        bad.is_empty(),
        "{} of {} concurrent retes disagreed with the serial run — cross-thread \
         damage. (worker, concurrent, serial): {:?}",
        bad.len(),
        serial.len(),
        bad
    );
}

/// The concurrent run must also match the ANALYTIC closure, not merely the
/// serial run. If both paths were wrong the same way, the comparison above would
/// still pass.
#[test]
fn concurrent_witnesses_are_the_known_closure() {
    let concurrent = witnesses(":user::cc-concurrent").expect("concurrent run");
    let want: Vec<i64> = (0..concurrent.len() as i64).map(expected).collect();
    assert_eq!(concurrent, want, "concurrent closure must be the known one");
}

/// Both rule sets must actually have run. If the dispatch collapsed to one set,
/// every assertion above would still pass while testing half of what it claims.
#[test]
fn both_rule_sets_are_exercised_concurrently() {
    let concurrent = witnesses(":user::cc-concurrent").expect("concurrent run");
    let cc = concurrent.iter().filter(|w| *w % 1_000 == 1).count();
    let dd = concurrent.iter().filter(|w| *w % 1_000 == 2).count();
    assert_eq!(
        cc + dd,
        concurrent.len(),
        "every witness must carry a known rule-set tag; got {concurrent:?}"
    );
    assert!(
        cc > 0 && dd > 0,
        "both networks must be live on the pool: cc={cc}, dd={dd}"
    );
}

/// Run the pool repeatedly. A race that needs a particular interleaving will not
/// show on one pass; the repeat is what makes this a fuzz rather than a demo.
#[test]
fn concurrent_retes_are_stable_across_repeats() {
    const REPEATS: usize = 8;
    let want: Vec<i64> = (0..48_i64).map(expected).collect();
    for pass in 0..REPEATS {
        let concurrent = witnesses(":user::cc-concurrent")
            .unwrap_or_else(|e| panic!("pass {pass} raised: {e}"));
        assert_eq!(
            concurrent, want,
            "pass {pass} of {REPEATS} diverged from the known closure"
        );
    }
}
