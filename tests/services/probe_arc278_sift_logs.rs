//! Arc 278 Stone 2 — the `sift-logs` op integration RED gate, proven on BOTH loci (R31/R32 —
//! loci-agnostic is non-negotiable; thread-only would be a failure). journal' + mem-store' on the
//! THREAD locus, then the SAME scenario across a PROCESS fork (grant-before-dial; the Sieve's
//! ::-source String + the opaque Log messages cross the wire; the predicate evals in the child).
//! Write a mixed page of Logs; `sift-logs` with a PURE predicate (level = :error) returns ONLY the
//! survivor (count 1); `sift-logs` with an IMPURE predicate is REJECTED (`::Fatal`, never a silent
//! pass). Thread and process return the SAME result — that identity IS the loci-agnostic proof.
//!
//! Run: cargo test --release -p wat sift_logs

use wat::freeze::startup_beside;
use wat::runtime::{apply_function, Value};

#[test]
fn sift_logs_pure_predicate_returns_only_survivors() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let func = world.symbols().get(":user::sift-pure-survivors").expect(":user::sift-pure-survivors").clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("sift-logs (pure predicate) raised: {e:?}"));
    assert!(matches!(got, Value::i64(1)),
        "expected sift-logs with a pure `level = :error` predicate to return exactly 1 survivor; got {got:?}");
}

#[test]
fn sift_logs_impure_predicate_is_rejected() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let func = world.symbols().get(":user::sift-impure-rejected").expect(":user::sift-impure-rejected").clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("sift-logs (impure predicate) raised: {e:?}"));
    assert!(matches!(got, Value::bool(true)),
        "expected sift-logs with an impure predicate to return ::Fatal (rejected, not a silent pass); got {got:?}");
}

// ── PROCESS locus — the loci-agnostic proof (R31/R32). The SAME two scenarios across a FORK:
// mem-store' + journal' both on processes, journal' dialing mem-store' via grant-before-dial. Same
// ops → same result as thread; thread-only would be a failure.

#[test]
fn sift_logs_pure_predicate_returns_only_survivors_on_process() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let func = world.symbols().get(":user::sift-pure-survivors-process").expect(":user::sift-pure-survivors-process").clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("sift-logs (pure predicate, PROCESS) raised: {e:?}"));
    assert!(matches!(got, Value::i64(1)),
        "loci-agnostic: sift-logs on a PROCESS fork with a pure `level = :error` predicate must return exactly 1 survivor (same as thread); got {got:?}");
}

#[test]
fn sift_logs_impure_predicate_is_rejected_on_process() {
    let world = startup_beside(file!()).expect("startup should succeed");
    let func = world.symbols().get(":user::sift-impure-rejected-process").expect(":user::sift-impure-rejected-process").clone();
    let got = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("sift-logs (impure predicate, PROCESS) raised: {e:?}"));
    assert!(matches!(got, Value::bool(true)),
        "loci-agnostic: sift-logs on a PROCESS fork with an impure predicate must be rejected ::Fatal (same as thread); got {got:?}");
}
