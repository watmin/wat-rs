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
//! If those three compose, the composition Stone 227.2 v2 needed exists.
//! Sonnet's failure was DISCOVERY, not substrate deficiency.
//!
//! Two probes here:
//!   1. `~@(let [...] vec-of-i64)` — does splice consume primitive Vec?
//!   2. `~@(let [...] vec-of-WatAST-via-runtime-quasiquote)` — does splice
//!      consume Vec<WatAST> built via Vector/map + inner quasiquote?
//!
//! Probe 2 IS the exact shape Stone 227.2 v2 needed for multi-field defrecord.
//!
//! Outcomes:
//!   - BOTH PASS: Task #477 DISCONFIRMED. Macro system is NOT deficient.
//!     Stone 227.2 v2's STOP-5b was discovery failure. Stone 227.2 v3 should
//!     ship multi-field via this composition pattern.
//!   - EITHER FAILS: SPECIFIC failure surfaced; Task #477 stays open with
//!     the actual blocker named (parser? evaluator? value_to_watast bridge?
//!     runtime quasiquote bug?).

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

fn with_nil_main(src: &str) -> String {
    format!(
        "{}\n(:wat::core::defn :user::main [] -> :wat::core::nil nil)",
        src
    )
}

fn run(src: &str) -> Value {
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    eval_in_frozen(&ast, &world, &env).expect("compute should run").value_owned()
}

// ─── Probe 1: splice from let returning Vec<i64> ─────────────────────────────

/// Disconfirms: "splice can't consume Vec from computed unquote let."
///
/// Macro template:
///   `(:wat::core::Vector :wat::core::i64
///      ~@(:wat::core::let [doubled (Vector/map double xs)] doubled))`
///
/// If the splice path (arc 143 slice 2) consumes the let's returned Vec
/// and converts each element via value_to_watast, the expansion produces
/// (Vector i64 2 4 6) for input [1 2 3].
#[test]
fn probe_1_splice_from_let_vec_of_i64() {
    let src = r##"
        (:wat::core::defmacro :probe::splice-i64
          [xs <- :wat::WatAST]
          -> :wat::WatAST
          (:wat::core::quasiquote
            (:wat::core::Vector :wat::core::i64
              (:wat::core::unquote-splicing
                (:wat::core::let
                  [doubled (:wat::core::map
                             (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
                               (:wat::core::i64::* x 2))
                             xs)]
                  doubled)))))

        (:wat::core::defn :user::compute [] -> :wat::core::Vector<wat::core::i64> (:probe::splice-i64 [1 2 3]))
    "##;
    match run(src) {
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

// ─── Probe 2: splice from let returning Vec<WatAST> via runtime quasiquote ──

/// Disconfirms: "splice can't consume Vec<WatAST> from computed unquote
/// let with inner runtime quasiquote."
///
/// THIS IS THE COMPOSITION STONE 227.2 V2 NEEDED for multi-field defrecord.
///
/// Macro template:
///   `(:wat::core::Vector :wat::core::i64
///      ~@(:wat::core::let
///          [forms (:wat::core::map
///                   (:wat::core::fn [x <- :wat::core::i64] -> :wat::WatAST
///                     `~(:wat::core::i64::* x 10))
///                   xs)]
///          forms))`
///
/// Inner runtime quasiquote `` `~(* x 10) `` produces a WatAST::IntLit at
/// each iteration. Vector/map collects Vec<WatAST>. ~@ splices.
///
/// Expected: input [1 2 3] -> [10 20 30] (each x mapped via runtime
/// quasiquote-of-integer-literal).
#[test]
fn probe_2_splice_from_let_vec_of_watast_via_runtime_quasiquote() {
    let src = r##"
        (:wat::core::defmacro :probe::splice-watast
          [xs <- :wat::WatAST]
          -> :wat::WatAST
          (:wat::core::quasiquote
            (:wat::core::Vector :wat::core::i64
              (:wat::core::unquote-splicing
                (:wat::core::let
                  [forms (:wat::core::map
                           (:wat::core::fn [x <- :wat::core::i64] -> :wat::WatAST
                             (:wat::core::quasiquote
                               (:wat::core::unquote (:wat::core::i64::* x 10))))
                           xs)]
                  forms)))))

        (:wat::core::defn :user::compute [] -> :wat::core::Vector<wat::core::i64> (:probe::splice-watast [1 2 3]))
    "##;
    match run(src) {
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
