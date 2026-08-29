//! Arc 209 C0b.3b-b — the allow-set gate, LIVE: the service refuses the stranger.
//!
//! C0b.3b-a shipped `comms::process::peer_cred(fd)` (the kernel-vouched `{pid,uid,gid}`).
//! This stone wires it LIVE into the process service loop: the `SocketListener` carries a
//! birth-seeded allow-set (`{getppid()}` = the owner/spawner, trusted by construction), and
//! `poll'`'s Fd-branch reads `peer_cred` at accept → serves only `uid==mine && pid∈allow-set`,
//! else **bounces the stranger** (drop the accepted stream, re-poll). The privilege is the
//! SERVICE's, not the socket's: raw `accept'` stays ungated; the thread tier stays ungated
//! (the crossbeam handle IS the grant). ("SO_PEERCRED is local mTLS.")
//!
//! Arc 272 step 5: the service NOW autobinds (no fixed name — unguessable capability) and sends
//! its `Address'` to the owner over the self-peer (capability handoff). The owner holds the
//! address and, for the bounce proof, leaks it to a stranger child via the stranger's lineage
//! channel. The allow-set check is by PID (not name), so the mechanism is unchanged.
//!
//! TWO proofs, one gate (the DESIGN's "same code, different pid, opposite outcome"):
//!
//! 1. `owner_served_via_birth_seed` — the OWNER (this test process) spawns a `(process)`
//!    service; the service birth-seeds its allow-set with `getppid()` = THIS process. The owner
//!    `recv'`s the minted `Address'` from `svc` (capability handoff), `connect'`s, and
//!    round-trips 5→105. GREEN at HEAD (no gate yet) AND after 3b-b (the birth-seed admits
//!    the owner) — the regression guard that the gate does NOT break the owner. (c0b3aii is the
//!    broader service-loop guard; this is the explicit birth-seed proof.)
//!
//! 2. `stranger_is_bounced` — the owner spawns the service, `recv'`s the `Address'`, then
//!    spawns a SEPARATE `(process)` STRANGER child and HANDS the (leaked) service address DOWN
//!    to the stranger over the stranger's lineage channel (`send' stranger addr`; the stranger
//!    `recv'`s it via its own self-peer). The stranger's pid ≠ the owner's pid → it is NOT in
//!    the service's birth-seeded allow-set. The stranger `connect'`s to the service, `send'`s,
//!    then `recv'`s.
//!    - RED at HEAD: no gate → the stranger is SERVED → the stranger's `recv'` returns a value;
//!      it doesn't die; the owner's `recv' stranger` returns Ok. The test expects Err → FAILS.
//!    - GREEN after 3b-b: the service accepts the stranger's socket, reads `peer_cred`, finds
//!      the stranger's pid ∉ `{owner}` → drops the stream. The stranger's `recv'` sees EOF →
//!      RAISES → the stranger process DIES → the owner's `recv' stranger` surfaces the death →
//!      RAISES. The test asserts the raise (a genuine unauthorized process refused).
//!
//! Together: a connector IN the allow-set (the owner, via birth-seed) is served; a connector
//! NOT in it (a real stranger child) is refused — same service code, opposite outcome by pid.
//!
//! These tests FORK (spawn-program' (process)) → their own top-level [[test]] binary.
//! Run: cargo test --release -p wat --test probe_arc209_c0b3bb_bounced -- --test-threads=1

use wat::freeze::{call_beside_value, startup_from_file};
use wat::runtime::{apply_function, Value};

#[test]
fn owner_served_via_birth_seed() {
    // Proof 1: the owner is served via the birth-seed (regression guard).
    // Wat source lives in the co-located fixture: probe_arc209_c0b3bb_bounced.wat
    let got = call_beside_value(file!(), ":user::compute")
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(105)),
        "expected 105: the owner (this process) is in the service's birth-seeded allow-set \
         (getppid() = the owner), so its connection is served (5 → n+100). The gate must NOT \
         break the owner. got {got:?}"
    );
}

#[test]
fn stranger_is_bounced() {
    // Proof 2: a real stranger child (pid ≠ owner) is bounced.
    // Wat source: probe_arc209_c0b3bb_bounced_bounced.wat
    let world = startup_from_file("tests/services/probe_arc209_c0b3bb_bounced_bounced.wat")
        .expect("startup should succeed (C0b.3b-b: birth-seeded allow-set gate)");
    let func = world
        .symbols()
        .get(":user::compute")
        .expect("no :user::compute in probe_arc209_c0b3bb_bounced_bounced.wat")
        .clone();
    // arc 278 VALUE-CONTRACT (R53/R55): the owner FACES the stranger's death as a matchable
    // RecvOutcome VALUE and RETURNS a :probe::Outcome — never re-raises past apply_function.
    // The golden #probe.Outcome/Bounced [] is captured (UPDATE_EDN=1), never hand-authored.
    let v = apply_function(func, vec![], world.symbols(), wat::rust_caller_span!())
        .unwrap_or_else(|e| panic!("the stranger's bounce must surface as a VALUE compute FACES (never a raise past apply_function); got Err: {e:?}"));
    let edn = ::wat_edn::write(&wat::edn_shim::value_to_edn_with(&v, None).expect("the probe's value must encode"));
    wat::assert_edn_matches_file!(edn, "c0b3bb_bounced__stranger_is_bounced.edn",
      "the stranger (pid ∉ allow-set) must be BOUNCED — its recv' EOFs → it dies → the owner FACES the death as a matchable Outcome::Bounced, never Served");
}
