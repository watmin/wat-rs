//! Arc 170 M1-teeth — the capability circuit's revoke BITES, deterministically.
//!
//! Proves the two halves of the teeth against a defservice's PID allow-set, driven purely by
//! OUR grant/revoke calls (not the birth-seed — the prober is a SEPARATE process whose pid ∉ A's
//! birth-seed). The allow-set predicate itself is already unit-tested in
//! `src/capability/policy.rs::only_my_peers_admits_lineage_and_refuses_everyone_else`; M1 proves
//! only that OUR grant admits and OUR revoke refuses the SAME live pid on a real dial.
//!
//! 1. `granted_prober_is_admitted` — the owner spawns a `(process)` prober whose pid it GRANTS
//!    into service A's allow-set, then hands A's addr down. The prober `connect'`s + `echo`s and
//!    is SERVED (only because granted). `compute` returns the reply → `Ok "echo:hi"`.
//!
//! 2. `revoked_prober_is_bounced` — a TWO-PHASE prober. Dial #1 (post-grant) is admitted and its
//!    reply reported up. The owner then REVOKES the pid (blocks on the PeersDenied ack — the pid
//!    is provably gone) and ONLY THEN sends the re-dial signal. Dial #2 is refused → the prober's
//!    echo recv' EOFs → the prober RAISES → dies → the owner's recv' surfaces the death →
//!    `compute` RAISES → `Err`. DETERMINISTIC: revoke happens-before re-dial happens-before
//!    dial #2 (the ack ordering forbids the race).
//!
//! Together: OUR grant admits (test 1 + test 2's dial #1) / OUR revoke refuses (test 2's raise) —
//! same service code, same live pid, opposite outcome across the revoke.
//!
//! These tests FORK (spawn-program' (process)) → their own top-level [[test]] binary.
//! Run: cargo nextest run --release -p wat --test probe_arc170_m1_teeth --test-threads=1

use wat::freeze::{eval_in_frozen, startup_from_file};
use wat::runtime::{Environment, Value};

#[test]
fn granted_prober_is_admitted() {
    // The admit-via-grant control: a granted prober's pid admits its raw dial.
    let world = startup_from_file("tests/services/probe_arc170_m1_teeth_admitted.wat")
        .expect("startup should succeed (arc 170 M1-teeth: admit-via-grant control)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    match got {
        Value::String(ref s) if s.as_str() == "echo:hi" => { /* granted prober was served */ }
        other => panic!(
            "expected Ok \"echo:hi\": the prober's pid was GRANTED into A's allow-set, so its \
             raw connect' + echo is served. got {other:?}"
        ),
    }
}

#[test]
fn revoked_prober_is_bounced() {
    // The teeth: after revoke (ack'd), the SAME live pid's re-dial is bounced → the prober dies.
    let world = startup_from_file("tests/services/probe_arc170_m1_teeth_revoked.wat")
        .expect("startup should succeed (arc 170 M1-teeth: deterministic revoke-refusal)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let outcome = eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned());
    match outcome {
        Err(_e) => { /* the revoked prober's dial #2 was bounced → it died → compute raised */ }
        Ok(v) => panic!(
            "expected Err — after echo'/revoke ack'd the pid gone, the prober's dial #2 should be \
             BOUNCED (its echo recv' EOFs → the prober RAISES → dies → the owner's recv' surfaces \
             the death). Instead dial #2 was SERVED and a reply was observed: got {v:?}. The \
             revoke did not gate the new accept (STOP-1)."
        ),
    }
}
