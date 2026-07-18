//! Diagnostic probe — disconfirm "macro system can't splice from computed unquote let".
//!
//! Stone 227.2 v2's defrecord macro deferred N>=2 multi-field constructor
//! synthesis with STOP-5b citing "substrate iteration at macro expand time
//! does not yet exist." Task #477 was filed as a substrate-flaw on that basis.
//!
//! This probe tests that claim empirically. The substrate provides:
//!   - `:wat::core::map` (stdlib iteration)
//!   - Runtime quasiquote producing WatAST (arc 091 slice 8)
//!   - `~@(keyword-headed-form)` splice path accepting `Value::Vec` at
//!     expand time (`src/macros.rs:1141-1188`; arc 143 slice 2)
//!
//! Outcomes:
//!   - BOTH PASS: Task #477 DISCONFIRMED. Stone 227.2 v2's STOP-5b was discovery failure.
//!   - EITHER FAILS: SPECIFIC failure surfaced; Task #477 stays open.

use wat::freeze::startup_from_file;
use wat::runtime::{apply_function, Value};

// just-eval (rubric): each `*.wat` fixture defines a zero-arg `:user::compute`; fetch it from
// the frozen world and `apply_function` it — no inline wat driver. (Path-based rather than
// `call_beside` because this probe drives two distinct co-located fixtures from one `.rs`.)
fn compute_from_file(fixture: &str) -> Value {
    let world = startup_from_file(fixture).expect("startup should succeed");
    let func = world
        .symbols()
        .get(":user::compute")
        .unwrap_or_else(|| panic!("no :user::compute in {fixture:?}"))
        .clone();
    apply_function(func, vec![], world.symbols(), wat::rust_caller_span!()).expect("compute should run")
}

/// Disconfirms: "splice can't consume Vec from computed unquote let."
#[test]
fn probe_1_splice_from_let_vec_of_i64() {
    match compute_from_file("tests/macros/probe_diagnostic_macro_splice_from_let_splice_i64.wat") {
        Value::Vec(v) => {
            let nums: Vec<i64> = v
                .iter()
                .map(|e| match e {
                    Value::i64(n) => *n,
                    other => panic!("expected i64 element; got {:?}", other),
                })
                .collect();
            assert_eq!(
                nums,
                vec![2_i64, 4, 6],
                "splice-from-let-Vec<i64> should produce [2 4 6]; got {:?}",
                nums
            );
        }
        other => panic!("expected Vec; got {:?}", other),
    }
}

/// Disconfirms: "splice can't consume Vec<WatAST> from computed unquote let with inner runtime quasiquote."
///
/// THIS IS THE COMPOSITION STONE 227.2 V2 NEEDED for multi-field defrecord.
#[test]
fn probe_2_splice_from_let_vec_of_watast_via_runtime_quasiquote() {
    match compute_from_file("tests/macros/probe_diagnostic_macro_splice_from_let_splice_watast.wat") {
        Value::Vec(v) => {
            let nums: Vec<i64> = v
                .iter()
                .map(|e| match e {
                    Value::i64(n) => *n,
                    other => panic!("expected i64 element; got {:?}", other),
                })
                .collect();
            assert_eq!(
                nums,
                vec![10_i64, 20, 30],
                "splice-from-let-Vec<WatAST-via-runtime-quasiquote> should produce [10 20 30]; got {:?}",
                nums
            );
        }
        other => panic!("expected Vec; got {:?}", other),
    }
}
