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
//! TWO proofs, one gate (the DESIGN's "same code, different pid, opposite outcome"):
//!
//! 1. `owner_served_via_birth_seed` — the OWNER (this test process) spawns a `(process)`
//!    service; the service birth-seeds its allow-set with `getppid()` = THIS process. The owner
//!    `connect'`s by name and round-trips 5→105. GREEN at HEAD (no gate yet) AND after 3b-b
//!    (the birth-seed admits the owner) — the regression guard that the gate does NOT break the
//!    owner. (c0b3aii is the broader service-loop guard; this is the explicit birth-seed proof.)
//!
//! 2. `stranger_is_bounced` — the owner spawns the service, then spawns a SEPARATE `(process)`
//!    STRANGER child. The stranger's pid ≠ the owner's pid → it is NOT in the service's
//!    birth-seeded allow-set. The stranger `connect'`s to the service by name, `send'`s, then
//!    `recv'`s, and (if it gets a reply) forwards it to its own self-peer for the owner to read.
//!    - RED at HEAD: no gate → the stranger is SERVED → it gets 107 back, forwards it → the
//!      owner's `recv' stranger` returns 107 (Ok). The test expects a bounce (Err) → FAILS.
//!    - GREEN after 3b-b: the service accepts the stranger's socket, reads `peer_cred`, finds
//!      the stranger's pid ∉ `{owner}` → drops the stream. The stranger's `recv'` sees EOF →
//!      RAISES → the stranger process DIES before forwarding → the owner's `recv' stranger`
//!      surfaces the death → RAISES. The test asserts the raise (a genuine unauthorized process
//!      refused — no `deny'`/owner-pid contrivance).
//!
//! Together: a connector IN the allow-set (the owner, via birth-seed) is served; a connector
//! NOT in it (a real stranger child) is refused — same service code, opposite outcome by pid.
//!
//! These tests FORK (spawn-program' (process)) → their own top-level [[test]] binary.
//! Run: cargo test --release -p wat --test probe_arc209_c0b3bb_bounced -- --test-threads=1

use std::sync::Arc;
use wat::freeze::{eval_in_frozen, startup_from_source};
use wat::load::InMemoryLoader;
use wat::runtime::{Environment, Value};

// ── The service forms (the c0b3aii poll'-loop), reused by both programs below. ──────────────
// A spawned (process) service: bind a listener by NAME (birth-seeds the allow-set with
// getppid() = the owner), signal READY over the self-peer, then poll'-serve echo n+100.
const SERVICE_FORMS: &str = r#"
             (:wat::core::defn :user::serve
               [self    <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>
                l       <- :wat::kernel::Listener'<wat::core::i64,wat::core::i64>
                clients <- :wat::core::Vector<wat::kernel::Peer'<wat::core::i64,wat::core::i64>>]
               -> :wat::core::nil
               (:wat::core::match (:wat::kernel::poll' self l clients) -> :wat::core::nil
                 (:wat::kernel::ServiceEvent::Shutdown nil)
                 ((:wat::kernel::ServiceEvent::Connection peer)
                   (:user::serve self l (:wat::core::conj clients peer)))
                 ((:wat::kernel::ServiceEvent::Message idx n)
                   (:wat::core::let [_ (:wat::kernel::send' (:wat::core::nth clients idx)
                                          (:wat::core::+ n 100))]
                     (:user::serve self l clients)))
                 ((:wat::kernel::ServiceEvent::Closed idx)
                   (:user::serve self l (:wat::std::list::remove-at clients idx)))
                 ((:wat::kernel::ServiceEvent::Lost idx _cause)
                   (:user::serve self l (:wat::std::list::remove-at clients idx)))))
             (:wat::core::defn :user::main [] -> :wat::core::nil
               (:wat::core::let
                 [l    (:wat::kernel::listener' (:wat::spawn::process)
                         (:wat::kernel::socket-address' "wat.arc209.c0b3bb.svc" :wat::core::i64 :wat::core::i64))
                  self (:wat::program::self-peer :wat::core::i64 :wat::core::i64)
                  _    (:wat::kernel::send' self 1)]
                 (:user::serve self l
                   (:wat::core::Vector :wat::kernel::Peer'<wat::core::i64,wat::core::i64>))))
"#;

// ── Proof 1: the owner is served via the birth-seed (regression guard). ─────────────────────
fn served_program() -> String {
    format!(
        r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc (:wat::kernel::spawn-program' (:wat::spawn::process)
           (:wat::core::forms
{SERVICE_FORMS}))
     _   (:wat::kernel::recv' svc)
     c   (:wat::kernel::connect'
           (:wat::kernel::socket-address' "wat.arc209.c0b3bb.svc" :wat::core::i64 :wat::core::i64))
     _   (:wat::kernel::send' c 5)
     got (:wat::kernel::recv' c)]
    got))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#
    )
}

#[test]
fn owner_served_via_birth_seed() {
    let program = served_program();
    let world = startup_from_source(&program, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (C0b.3b-b: birth-seeded allow-set gate)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let got = eval_in_frozen(&ast, &world, &Environment::new())
        .map(|tv| tv.value_owned())
        .unwrap_or_else(|e| panic!("compute raised: {e:?}"));
    assert!(
        matches!(got, Value::i64(105)),
        "expected 105: the owner (this process) is in the service's birth-seeded allow-set \
         (getppid() = the owner), so its connection is served (5 → n+100). The gate must NOT \
         break the owner. got {got:?}"
    );
}

// ── Proof 2: a real stranger child (pid ≠ owner) is bounced. ────────────────────────────────
// The stranger: connect' to the service by name, send 7, recv the reply, forward it to its own
// self-peer. At HEAD (no gate) it is served → forwards 107. After 3b-b it is bounced → its
// recv' EOFs → it DIES before forwarding → the owner's recv' on the stranger handle raises.
fn bounced_program() -> String {
    format!(
        r#"
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [svc (:wat::kernel::spawn-program' (:wat::spawn::process)
           (:wat::core::forms
{SERVICE_FORMS}))
     _   (:wat::kernel::recv' svc)
     ;; A SEPARATE process child — its pid ≠ the owner's → NOT in the birth-seeded allow-set.
     stranger (:wat::kernel::spawn-program' (:wat::spawn::process)
                (:wat::core::forms
                  (:wat::core::defn :user::main [] -> :wat::core::nil
                    (:wat::core::let
                      [c    (:wat::kernel::connect'
                              (:wat::kernel::socket-address' "wat.arc209.c0b3bb.svc" :wat::core::i64 :wat::core::i64))
                       _    (:wat::kernel::send' c 7)
                       got  (:wat::kernel::recv' c)        ;; 3b-b: EOF on the bounce → RAISES → die
                       self (:wat::program::self-peer :wat::core::i64 :wat::core::i64)
                       _    (:wat::kernel::send' self got)] ;; HEAD only: forward the served reply
                      nil))))
     got (:wat::kernel::recv' stranger)]                   ;; HEAD: 107 ; 3b-b: stranger died → RAISES
    got))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
"#
    )
}

#[test]
fn stranger_is_bounced() {
    let program = bounced_program();
    let world = startup_from_source(&program, None, Arc::new(InMemoryLoader::new()))
        .expect("startup should succeed (C0b.3b-b: birth-seeded allow-set gate)");
    let ast = wat::parse_one!("(:user::compute)").expect("parse");
    let outcome = eval_in_frozen(&ast, &world, &Environment::new()).map(|tv| tv.value_owned());
    // GREEN (3b-b): the stranger (pid ∉ allow-set) is bounced → its recv' EOFs → it dies →
    // the owner's recv' on the stranger surfaces the death → Err.
    // RED (HEAD): no gate → the stranger is served → it forwards 107 → Ok(107).
    match outcome {
        Err(_e) => { /* the stranger was refused and died — the gate is live */ }
        Ok(v) => panic!(
            "expected the stranger (a process whose pid is NOT in the service's birth-seeded \
             allow-set) to be BOUNCED — its recv' should EOF on the dropped stream and the \
             stranger should die, raising on the owner's recv'. Instead the stranger was SERVED \
             and forwarded a reply: got {v:?}. The allow-set gate is not live."
        ),
    }
}
