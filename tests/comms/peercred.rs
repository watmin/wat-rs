//! Arc 209 C0b.3b-a — the `SO_PEERCRED` read primitive.
//!
//! The kernel-vouched credential of the peer connected to a UDS socket fd, read via
//! `getsockopt(SOL_SOCKET, SO_PEERCRED)` — unforgeable, no `/proc`, no handshake. This is the
//! mechanism the C0b.3b-b accept enforcement checks against the allow-set ("SO_PEERCRED is
//! local mTLS"). Pure mechanism: no allow-set, no wat surface — those are C0b.3b-b.
//!
//! THE GATE: a `socketpair(AF_UNIX, SOCK_STREAM)` has BOTH ends in THIS process, so each
//! end's peer credential is THIS process. `peer_cred` on each end must report
//! `pid == std::process::id()` and a consistent uid/gid. RED at HEAD (`peer_cred` does not
//! exist — won't compile); GREEN once C0b.3b-a ships the primitive.

use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use wat::comms::process::peer_cred;

#[test]
fn probe_arc209_c0b3ba_peercred_reads_self() {
    let (a, b) = UnixStream::pair().expect("socketpair should succeed");
    let ca = peer_cred(a.as_raw_fd()).expect("peer_cred(a) on a connected UDS");
    let cb = peer_cred(b.as_raw_fd()).expect("peer_cred(b) on a connected UDS");
    let me = std::process::id();
    assert_eq!(ca.pid as u32, me, "the peer connected to end a is this process");
    assert_eq!(cb.pid as u32, me, "the peer connected to end b is this process");
    assert_eq!(ca.uid, cb.uid, "same-process peers report the same uid");
    assert_eq!(ca.gid, cb.gid, "same-process peers report the same gid");
}
