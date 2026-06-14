# DESIGN-STONE C0b.3b-a — the `SO_PEERCRED` read primitive (the kernel-vouched credential)

> First strike of the C0b.3b identity gate (security model LOCKED in
> `DESIGN-STONE-C0b-SECURITY.md`). The pure mechanism: read the connecting peer's
> kernel-vouched `{pid, uid, gid}` from a connected UDS fd via `getsockopt(SO_PEERCRED)`.
> Zero policy, zero wat surface — just the credential the enforcement (C0b.3b-b) checks
> against the allow-set. Foundation, isolatable, unit-tested. ("`SO_PEERCRED` is local mTLS.")

## Why split this off

C0b.3b is "a service refuses unauthorized connectors." That has THREE parts: (1) read the
connector's credential (this stone — pure OS mechanism, no decisions), (2) the allow-set +
accept enforcement, (3) the wat allow-set API (authorize/revoke verbs + the admin-channel
grant flow + exposing a spawned child's pid). Parts (2)/(3) carry real policy/API design
(builder's security domain). Part (1) is unambiguous and de-risks the one OS-uncertain piece
(the `getsockopt` mechanism) in isolation — ship it first; the enforcement consumes it next.

## Grounded this session (HEAD `30b2b3b5`)

- No `SO_PEERCRED`/`peer_cred`/`ucred`/`getsockopt` anywhere in `src/` — net-new.
- The security model (LOCKED): enforcement = ONE `getsockopt(SO_PEERCRED)` at accept →
  kernel hands the service the connector's real `{pid, uid, gid}`, unforgeable, no `/proc`,
  no handshake. Two layers: `uid == mine` (coarse) + `pid ∈ allow-set` (precise). The
  `SO_PEERCRED` mechanism was proven viable in a spike (security doc); this stone makes it a
  substrate primitive.
- The accepted connection's fd: `SocketListener::accept` (kernel/listener.rs) does
  `UnixListener::accept()` → a `UnixStream` (then `sender_receiver_from_fd` wraps it). The
  credential is read from that accepted stream's fd — that is where C0b.3b-b will call this.
- `comms::process` owns the socket-fd machinery (`Sender`/`Receiver`/`sender_receiver_from_fd`).
  Layering: kernel → comms; this primitive lives in `comms` (kernel's accept consumes it).

## The contract decision (pinned)

A pure-mechanism fn + a plain value, in `src/comms/process.rs` (the socket-fd home):
```rust
/// Kernel-vouched identity of the peer connected to a UDS socket fd.
/// Captured by the kernel at connect time — unforgeable, no /proc, no handshake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerCred { pub pid: i32, pub uid: u32, pub gid: u32 }

/// Read SO_PEERCRED off a connected AF_UNIX SOCK_STREAM fd.
/// Errors if the fd is not a connected UDS socket (ENOTCONN / EINVAL).
pub fn peer_cred(fd: RawFd) -> std::io::Result<PeerCred> {
    // getsockopt(fd, SOL_SOCKET, SO_PEERCRED, &mut libc::ucred, &mut len)
    // → PeerCred { pid: ucred.pid, uid: ucred.uid, gid: ucred.gid }
    // standard libc::ucred { pid: pid_t, uid: uid_t, gid: gid_t }; check the return,
    // last_os_error on -1.
}
```
No wat surface, no allow-set, no accept change — those are C0b.3b-b. `PeerCred` is a plain
copyable value (it crosses no wire; it's read locally at accept). `pid` as `i32` (pid_t).

## The gate (new mechanism — empirical RED probe)

A unit/integration probe `probe_arc209_c0b3ba_peercred` (in `tests/comms/`): make a
`socketpair(AF_UNIX, SOCK_STREAM)` (both ends in THIS process), call `peer_cred(fd)` on one
end → assert `pid == std::process::id()`, `uid == geteuid()`, `gid == getegid()` (the peer
is us — same process). RED at HEAD (`peer_cred` doesn't exist → won't compile); GREEN after.
(Optionally: `peer_cred` on a non-socket fd → `Err`.)

Regression: full comms suite green; nursery serial 895/4 (baseline); full workspace compiles.

## Files touched

`src/comms/process.rs` (the `PeerCred` struct + `peer_cred` fn; re-export from `comms` if the
existing `Sender`/`Receiver` are re-exported there) + `tests/comms/<probe>`. NO kernel/runtime/
check change. NO wat surface.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** `SO_PEERCRED` / `libc::ucred` is not available in the pinned `libc` for this
   target — STOP, report (the security model assumes Linux SO_PEERCRED; the spike proved it,
   so this is not expected).
2. **STOP-2:** `socketpair`-based testing can't observe a same-process peer cred (e.g. both
   ends report different creds) — STOP, report (both ends are this process → same pid/uid/gid).

## Out of scope (rejected — NOT deferred)

- The allow-set on `SocketListener` + the `accept` enforcement (`uid==mine` + `pid∈allow-set`
  → serve/refuse) = **C0b.3b-b**.
- The wat allow-set API (authorize/revoke verbs, the admin-channel grant flow) + exposing a
  spawned child's pid to the parent = **C0b.3b-b** (design with the builder — security-domain API).
- Remote cert identity (mTLS) = the `:remote` north star.

## The deadlock contract carries

A pure local `getsockopt` read; no transport, no blocking, no lifecycle. [[feedback_vended_primitives_never_deadlock]]
