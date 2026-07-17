//! Arc 278 — wat NEVER HIDES A FAILURE (see DESIGN-no-hidden-failures.md).
//!
//! A `journal'` service forked to a PROCESS receives a client message it cannot decode (a `Log` whose
//! `message` is the user record `:probe::Note`, absent from the forked child's baked type registry).
//! At HEAD the child dies with a rich, located reason —
//!   "poll' (process tier): client message decode failed: ... unknown tag #probe/Note (body shape:
//!    map); no matching struct or enum in the type registry"
//! — that is written to an ALREADY-CLOSED err pipe (EPIPE) and LOST; the caller's `write-logs` `recv'`
//! raises a MUTE "recv failed: peer closed / channel disconnected".
//!
//! THE LAW: the caller's error must CARRY the reason. This differs from
//! `probe_arc272_rs2_crash_surfaces_to_client`, which only asserts the crash *raises* (is_err) — a mute
//! raise passes that. Here we assert the raise carries the REASON. RED at HEAD (mute); GREEN when the
//! masking is pulled out by the root (RecvError carries a reason; the `|_|` discards bind the error; the
//! crash channel survives the child's death; poll' replies-and-survives instead of dying).
//!
//! Run: cargo test --release -p wat --test services dead_child_speaks

use wat::freeze::{eval_in_frozen, startup_beside};
use wat::runtime::Environment;

#[ignore = "RED gate for DESIGN-no-hidden-failures.md — un-ignore when the mask is pulled: the caller \
            must carry the child's decode reason (unknown tag #probe/Note ...), not a mute 'peer closed'"]
#[test]
fn a_forked_service_that_cannot_decode_a_message_speaks_its_reason_to_the_caller() {
    let world =
        startup_beside(file!()).expect("startup should succeed (dead-child-speaks probe)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let result = eval_in_frozen(&ast, &world, &Environment::new());

    // The undecodable message MUST raise (not hang, not fake a value) — and, crucially, the raise MUST
    // carry the child's real reason, not a mute mask.
    let err = result.expect_err(
        "write-logs of an undecodable payload across a process fork must RAISE (the child cannot decode it)",
    );
    let msg = format!("{err:?}");
    assert!(
        msg.contains("unknown tag")
            || msg.contains("decode failed")
            || msg.contains("no matching struct or enum"),
        "THE LAW (wat never hides a failure): the caller's error must carry the child's real reason \
         (e.g. 'unknown tag #probe/Note ... no matching struct or enum in the type registry'). \
         Instead it surfaced a MUTE mask: {msg}"
    );
}
