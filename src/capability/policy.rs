//! Arc 272 v4 / 6c.2 — the comms policy (the object-capability **powerbox**).
//!
//! *"Only my peers can comms with me."* A [`CommsPolicy`] decides, from a peer's KERNEL-VERIFIED
//! credentials ([`PeerCred`] = `SO_PEERCRED` `{pid,uid,gid}`, unforgeable), whether to admit it. It is
//! the single mediator for the **process-tier** (cross-boundary) accept/connect credential checks —
//! Mark Miller's *powerbox*: the one place deciding which peers a process may obtain authority from.
//! The thread tier needs no powerbox: an in-process peer is reached only by holding its crossbeam
//! handle, so possession of the handle IS the grant — there is no kernel credential to verify.
//!
//! The trust boundary is verified at the gate (end-to-end); the capability waist (`registry`) then
//! rides an already-authorized channel. The two live rungs are `OnlyMyPeers` (accept gate — euid +
//! lineage pid set) and `OnlyThisPeer` (connect gate — euid + minter pid stamped in the address
//! capability). Adding a rung extends the policy language; the `admits` contract never changes
//! (the narrow-waist law).

use crate::comms::process::PeerCred;
use std::collections::HashSet;

/// A comms authorization policy over a peer's verified credentials.
///
/// The gates pick the rung they can honestly stand on — `OnlyMyPeers` where the full lineage set
/// is held (the accept gate), `OnlyThisPeer` where the minter pid is stamped in the capability
/// (the connect gate: the address carries the minter's pid, and the connect gate verifies the
/// kernel-vouched answerer pid against it). The `admits` contract never changes.
pub enum CommsPolicy<'a> {
    /// Admit iff the peer runs as me (euid match) AND its pid is one of mine — a member of the
    /// **lineage set** (the pids I spawned; a listener's allow-set). The 272 trust model — *"only my
    /// peers"* — named. This is the object-capability transfer-only rule made a predicate: authority
    /// flows only along the spawn lineage, verified by the kernel, never to a stranger.
    OnlyMyPeers { lineage: &'a HashSet<i32> },
    /// Admit iff the peer runs as me (euid match) AND its pid equals the **minter pid** stamped in
    /// the address capability. The connect gate's rung: the minter stamps `getpid()` at autobind;
    /// the capability carries it by value to the dialer; the dialer verifies the kernel-vouched
    /// `SO_PEERCRED` answerer pid matches the stamped pid. Symmetric with the accept gate's
    /// `OnlyMyPeers` pid check — both verify kernel-vouched pid, both require euid match.
    OnlyThisPeer { pid: i32 },
}

impl CommsPolicy<'_> {
    /// The decision. `my_euid` is the caller's `geteuid()`, taken as a parameter so the policy is a
    /// pure, side-effect-free, unit-testable function of (policy, peer, my-identity).
    pub fn admits(&self, peer: &PeerCred, my_euid: u32) -> bool {
        match self {
            CommsPolicy::OnlyMyPeers { lineage } => {
                peer.uid == my_euid && lineage.contains(&peer.pid)
            }
            CommsPolicy::OnlyThisPeer { pid } => peer.uid == my_euid && peer.pid == *pid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cred(pid: i32, uid: u32) -> PeerCred {
        PeerCred { pid, uid, gid: 0 }
    }

    #[test]
    fn only_my_peers_admits_lineage_and_refuses_everyone_else() {
        let lineage: HashSet<i32> = [100, 101].into_iter().collect();
        let policy = CommsPolicy::OnlyMyPeers { lineage: &lineage };
        let me: u32 = 1000;

        // MY peer — my euid AND a lineage pid → admitted.
        assert!(policy.admits(&cred(100, me), me), "a lineage peer running as me is my peer");
        // Right pid, WRONG user (euid mismatch) → refused (not running as me).
        assert!(!policy.admits(&cred(100, me + 1), me), "another user's process is not my peer");
        // My user, but a STRANGER pid (∉ lineage) → refused (transfer-only: not in my lineage).
        assert!(!policy.admits(&cred(999, me), me), "a non-lineage pid is not my peer");
    }

    #[test]
    fn only_this_peer_admits_exact_pid_refuses_wrong_pid_and_wrong_uid() {
        let minter_pid: i32 = 4242;
        let policy = CommsPolicy::OnlyThisPeer { pid: minter_pid };
        let me: u32 = 1000;

        // Exact pid AND same uid → admitted.
        assert!(
            policy.admits(&cred(minter_pid, me), me),
            "exact minter pid + same euid must be admitted"
        );
        // Same uid, WRONG pid → REFUSED (the capability was minted by a specific process).
        assert!(
            !policy.admits(&cred(minter_pid + 1, me), me),
            "same uid but wrong pid must be refused"
        );
        // Right pid, WRONG uid → REFUSED (always requires euid match).
        assert!(
            !policy.admits(&cred(minter_pid, me + 1), me),
            "right pid but wrong uid must be refused"
        );
    }
}
