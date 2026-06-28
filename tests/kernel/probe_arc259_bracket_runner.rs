//! Arc 259 S3.2a — `:wat::bracket::runner-loop`: the multi-message server, the
//! foundation the brackets pool stands on.
//!
//! A `spawn-program` peer that serves a STREAM of work: `recv' item → work-fn →
//! send' result`, NAMED-RECURSING until its channel drains (recv' raises → the
//! runner exits cleanly). Today's peers are single-shot (recv once, send once,
//! return); the runner serves arbitrarily many messages. wat has TCO for named-defn
//! tail-calls (arc 003: "any gen_server-shaped driver, constant stack"), so the loop
//! is safe at any item count — no stack growth.
//!
//! RED at HEAD: `:wat::bracket::runner-loop` does not exist.
//!
//! Run SERIALLY (spawns a thread):
//!   `cargo nextest run --release -E 'binary(kernel)' -F probe_arc259_bracket_runner`
//!
//! WAT fixtures: tests/kernel/probe_arc259_bracket_runner_{large_stream,stream_of_messages}.wat

use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

fn run_compute_i64(path: &str) -> i64 {
    let world = startup_from_file(path).expect("startup should succeed");
    let ast = wat::parse_one!("(:user::compute)").expect("parse compute call");
    match eval_in_frozen(&ast, &world, &Environment::new())
        .expect("compute eval")
        .value_owned()
    {
        Value::i64(n) => n,
        other => panic!("expected i64; got {other:?}"),
    }
}

/// 300-item large-stream test: proves TCO in practice.
///
/// A named recursive driver `:user::drive [peer n acc] -> i64` sends n, recvs the
/// doubled result, accumulates, and recurses on n-1 until n=0 (tail-call in the
/// branch → TCO'd).  300 recursive frames would overflow any non-TCO stack for
/// this shape; green here means the named-defn TCO fires for both the driver AND
/// the runner-loop.
///
/// work-fn = x*2; driver sends 1..=300; receives 2..=600; sum = 2*(1+…+300) = 90300.
#[test]
fn runner_handles_a_large_stream() {
    let v = run_compute_i64("tests/kernel/probe_arc259_bracket_runner_large_stream.wat");
    assert_eq!(v, 90300, "300 items doubled and summed: 2*(1+2+...+300) = 90300; no stack overflow = TCO confirmed");
}

/// One runner with a doubling work-fn, served a STREAM of 3 items: 1→2, 2→4, 3→6.
/// The parent sums the three results (12) — proving the peer served MULTIPLE
/// messages (a single-shot peer could not). The peer drops at scope-exit → RAII
/// drain → the runner's `recv'` raises → it exits; no `close'`, no hang.
#[test]
fn runner_serves_a_stream_of_messages() {
    let v = run_compute_i64("tests/kernel/probe_arc259_bracket_runner_stream_of_messages.wat");
    assert_eq!(v, 12, "runner served 3 messages: 1->2, 2->4, 3->6; sum 12");
}
