//! Arc 214 Stone 4.6b — `select'` (FM-2-bis disconfirming probe).
//!
//! `select' : Vector<peer<I,O>> -> Tuple<i64, O>` — blocking first-ready
//! multiplex over same-tier peers, returning (index, value). Intrinsic —
//! projective (O flows from the element peer type; docs/DISPATCH.md).
//! Mixed-tier selection needs no bespoke rejection: Vector homogeneity
//! already makes it unrepresentable at check.
//!
//! Vacuity-aware (the 4.6a lesson): probe 1 RUNS the program (no eval
//! dispatch at HEAD → RED); probe 2 is a check NEGATIVE (a wrong return
//! annotation must fail once the projective inference exists — fresh vars
//! pass it today → RED).
//!
//! ## Arc 259 S2c-ii-a — apply-loop PURGE
//!
//! Both probes' spawn progs are SWAPPED to self-peer form
//! `[self <- Peer'<i64,i64>] -> nil (send' self (recv' self))`.
//! The `Thread'<i64,i64>` peer type is preserved — `Peer'<O,I>=Peer'<i64,i64>`
//! → `Thread'<R,S>=Thread'<I,O>=Thread'<i64,i64>`. The `select'` multiplex
//! is unchanged; only the spawned progs swap.
//!
//! Run: `cargo test --release --test nursery probe_arc214_stone46b_select_prime`

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

fn startup_err(src: &str) -> String {
    let src = with_nil_main(src);
    match startup_from_source(&src, None, Arc::new(InMemoryLoader::new())) {
        Ok(_) => panic!("expected startup failure; got Ok"),
        Err(e) => format!("{}\n---\n{:?}", e, e),
    }
}

// ─── Probe 1 (LOAD-BEARING, RUNTIME): select' picks the ready peer ────────────

/// Two thread echo peers; send 7 to peer B ONLY (deterministic — only B will
/// ever have data); select' [a b] must return the tuple (1, 7): index 1
/// (peer B's position) and the echoed value. Both peers closed after.
/// At HEAD: no eval dispatch for select' → eval errors → RED.
///
/// Arc 259 S2c-ii-a: spawn prog swapped to self-peer form
/// `[self <- Peer'<i64,i64>] -> nil (send' self (recv' self))` —
/// same `Thread'<i64,i64>` peer type; select' multiplex unchanged.
#[test]
fn probe_1_select_returns_ready_index_and_value() {
    let src = r#"
        (:wat::core::defn :user::mk [] -> :wat::kernel::Thread'<wat::core::i64,wat::core::i64>
          (:wat::kernel::spawn-program' :thread (:wat::program::Env (:wat::time::at-millis 0) (:wat::time::at-millis 0))
            (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
              (:wat::kernel::send' self (:wat::kernel::recv' self)))))
        (:wat::core::defn :user::compute [] -> :(wat::core::i64,wat::core::i64)
          (:wat::core::let [a (:user::mk)
                            b (:user::mk)
                            _ (:wat::kernel::send' b 7)
                            picked (:wat::kernel::select' [a b])
                            _ (:wat::kernel::close' a)
                            _ (:wat::kernel::close' b)]
            picked))
    "#;
    let src = with_nil_main(src);
    let world = startup_from_source(&src, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    let env = Environment::new();
    let got = eval_in_frozen(&ast, &world, &env)
        .expect("compute must evaluate (select' dispatch exists)")
        .value_owned();
    match got {
        Value::Tuple(xs) => {
            assert_eq!(xs.len(), 2, "select' returns (index, value); got {:?}", xs);
            assert_eq!(xs[0], Value::i64(1), "ready peer is index 1 (b); got {:?}", xs[0]);
            assert_eq!(xs[1], Value::i64(7), "the echoed value; got {:?}", xs[1]);
        }
        other => panic!("expected Tuple(index, value); got {:?}", other),
    }
}

// ─── Probe 2 (CHECK NEGATIVE): select' return type is Tuple<i64,O> ────────────

/// Declaring the select' result as `:wat::core::String` over i64-peers MUST
/// fail at check — the projective return is `:(i64,i64)`.
/// RED at HEAD (fresh var unifies with String).
///
/// Arc 259 S2c-ii-a: spawn prog swapped to self-peer form
/// `[self <- Peer'<i64,i64>] -> nil (send' self (recv' self))` —
/// same `Thread'<i64,i64>` peer type; select' return-type rejection unchanged.
#[test]
fn probe_2_select_wrong_return_annotation_rejected() {
    let src = r#"
        (:wat::core::defn :user::bad [] -> :wat::core::String
          (:wat::core::let [p (:wat::kernel::spawn-program' :thread (:wat::program::Env (:wat::time::at-millis 0) (:wat::time::at-millis 0))
                                (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                                  (:wat::kernel::send' self (:wat::kernel::recv' self))))]
            (:wat::kernel::select' [p])))
    "#;
    let _err = startup_err(src);
}
