//! Arc 272 v4 — the comms policy (the object-capability **powerbox**).
//!
//! *"Only my peers can comms with me."* A [`CommsPolicy`] decides, from a peer's KERNEL-VERIFIED
//! credentials ([`PeerCred`] = `SO_PEERCRED` `{pid,uid,gid}`, unforgeable), whether to admit it. It is
//! the single mediator that unifies the formerly-scattered accept/connect credential checks — Mark
//! Miller's *powerbox*: the one place deciding which peers a process may obtain authority from.
//!
//! The trust boundary is verified at the gate (end-to-end); the capability waist (`registry`) then
//! rides an already-authorized channel. Shaped to grow into a **policy language**: today the single
//! `OnlyMyPeers` variant (euid + lineage pid); future variants (any-of-my-user, these-gids, and
//! ultimately a wat `fn(PeerCred) -> bool` predicate) are added to the enum — the rigid `admits`
//! contract never changes, the expressible policies do (the narrow-waist law, applied to authority).

use crate::comms::process::PeerCred;
use std::collections::HashSet;

/// A comms authorization policy over a peer's verified credentials.
pub enum CommsPolicy<'a> {
    /// Admit iff the peer runs as me (euid match) AND its pid is one of mine — a member of the
    /// **lineage set** (the pids I spawned; a listener's allow-set). The 272 trust model — *"only my
    /// peers"* — named. This is the object-capability transfer-only rule made a predicate: authority
    /// flows only along the spawn lineage, verified by the kernel, never to a stranger.
    OnlyMyPeers { lineage: &'a HashSet<i32> },
}

impl CommsPolicy<'_> {
    /// The decision. `my_euid` is the caller's `geteuid()`, taken as a parameter so the policy is a
    /// pure, side-effect-free, unit-testable function of (policy, peer, my-identity).
    pub fn admits(&self, peer: &PeerCred, my_euid: u32) -> bool {
        match self {
            CommsPolicy::OnlyMyPeers { lineage } => {
                peer.uid == my_euid && lineage.contains(&peer.pid)
            }
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
}
