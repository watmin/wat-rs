//! Arc 272 6c.2 — pure unit probe: `CommsPolicy::OnlyThisPeer` gate logic.
//!
//! Tests the new `OnlyThisPeer { pid }` policy rung with SYNTHESIZED `PeerCred` values.
//! NO socket, NO fork, NO privilege required — this tests OUR gate logic, not the kernel's
//! SO_PEERCRED honesty (we don't test our axioms).
//!
//! ## Contracts
//!
//! C1 — Exact pid + same uid → ADMITTED.
//! C2 — Same uid, wrong pid → REFUSED.
//! C3 — Right pid, wrong uid → REFUSED.
//!
//! Run: `cargo test --release --test probe_arc272_6c2_pid_gate`

use wat::capability::CommsPolicy;
use wat::comms::process::PeerCred;

fn cred(pid: i32, uid: u32) -> PeerCred {
    PeerCred { pid, uid, gid: 0 }
}

#[test]
fn only_this_peer_admits_exact_pid_same_uid() {
    let minter_pid: i32 = 4242;
    let my_euid: u32 = 1000;
    let policy = CommsPolicy::OnlyThisPeer { pid: minter_pid };

    assert!(
        policy.admits(&cred(minter_pid, my_euid), my_euid),
        "C1: exact minter pid + same euid must be admitted"
    );
}

#[test]
fn only_this_peer_refuses_wrong_pid_same_uid() {
    let minter_pid: i32 = 4242;
    let my_euid: u32 = 1000;
    let policy = CommsPolicy::OnlyThisPeer { pid: minter_pid };

    assert!(
        !policy.admits(&cred(minter_pid + 1, my_euid), my_euid),
        "C2: same uid but wrong pid must be refused — the answerer is not the minter"
    );
}

#[test]
fn only_this_peer_refuses_right_pid_wrong_uid() {
    let minter_pid: i32 = 4242;
    let my_euid: u32 = 1000;
    let policy = CommsPolicy::OnlyThisPeer { pid: minter_pid };

    assert!(
        !policy.admits(&cred(minter_pid, my_euid + 1), my_euid),
        "C3: right pid but different uid must be refused — euid mismatch"
    );
}
