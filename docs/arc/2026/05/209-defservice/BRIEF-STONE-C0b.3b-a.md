# BRIEF — Stone C0b.3b-a: the `SO_PEERCRED` read primitive

**Executor:** Shadowdancer (sonnet). **Anchor:** `/home/watmin/work/holon/wat-rs/` (verify
`pwd`; operate only here; `git -C /home/watmin/work/holon/wat-rs`). Design:
`DESIGN-STONE-C0b.3b-a-peercred-read-primitive.md` (read it fully). The RED probe is on disk
+ verified RED (E0432: `no peer_cred in comms::process`). Do NOT commit — the Inquisitor weighs.

## The work in one paragraph

Add a pure-mechanism socket-credential primitive to `src/comms/process.rs`: a `PeerCred {
pid: i32, uid: u32, gid: u32 }` value + `pub fn peer_cred(fd: RawFd) -> std::io::Result<PeerCred>`
that reads the kernel-vouched credential of the peer connected to a UDS fd via
`getsockopt(fd, SOL_SOCKET, SO_PEERCRED, &mut libc::ucred, &mut len)`. No allow-set, no accept
change, no wat surface — those are C0b.3b-b. This makes the existing RED probe
`probe_arc209_c0b3ba_peercred` compile and pass.

## Read in order (the rooms)

1. `src/comms/process.rs` — the socket-fd home (`Sender`/`Receiver`/`sender_receiver_from_fd`,
   the `libc`/fd patterns already in use here). Add `PeerCred` + `peer_cred` near them.
2. `src/comms/mod.rs` — if `comms::process::Sender` etc. are re-exported, mirror for `peer_cred`/
   `PeerCred` (so the probe's `wat::comms::process::peer_cred` resolves; the probe imports that path).
3. `tests/comms/peercred.rs` — the gate probe (already on disk); it is the exact contract.

## Implementation sketch (fill the shape)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerCred { pub pid: i32, pub uid: u32, pub gid: u32 }

pub fn peer_cred(fd: std::os::fd::RawFd) -> std::io::Result<PeerCred> {
    let mut cred = libc::ucred { pid: 0, uid: 0, gid: 0 };
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: getsockopt writes into &mut cred / &mut len; fd is borrowed for the call.
    let rc = unsafe {
        libc::getsockopt(fd, libc::SOL_SOCKET, libc::SO_PEERCRED,
                         &mut cred as *mut _ as *mut libc::c_void, &mut len)
    };
    if rc != 0 { return Err(std::io::Error::last_os_error()); }
    Ok(PeerCred { pid: cred.pid, uid: cred.uid, gid: cred.gid })
}
```
Confirm `libc::ucred` field types (`pid: pid_t` (i32), `uid/gid: uid_t/gid_t` (u32)) on this
target; cast if needed. Use the same `libc` import style already present in `comms/process.rs`.

## Blast radius

`src/comms/process.rs` (+ a re-export in `comms/mod.rs` if needed for the probe's path). The
probe exists. NO kernel/runtime/check change. NO wat surface.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** `libc::SO_PEERCRED` / `libc::ucred` not available in the pinned `libc` — STOP,
   report (not expected; the security spike proved SO_PEERCRED on this target).
2. **STOP-2:** the probe's `wat::comms::process::peer_cred` path can't be made to resolve
   without a layering oddity — STOP, report (place `peer_cred` so that path works, mirroring
   how `pair`/`Sender` are exposed).

## The gate

```
cargo test --release -p wat --test comms probe_arc209_c0b3ba_peercred -- --test-threads=1   # GREEN (self pid)
cargo test --release -p wat --test comms -- --test-threads=1                                # all comms green
cargo test --release -p wat --test nursery -- --test-threads=1                              # 895 passed / 4 failed (baseline)
cargo test --release --workspace --no-run                                                   # full surface compiles
```
Report each exact `test result:` line + any STOP/honest delta. Do NOT commit.

## Prior comparable (copy the shape)

`BRIEF-STONE-C0b.2e-i-a.md` (the comms-trait foundation — same `src/comms/` + `tests/comms/`
shape, a small additive primitive with a focused probe).
