# DESIGN-STONE C0b.3b-b — the allow-set gate, LIVE (the service refuses the stranger)

> The SO_PEERCRED enforcement, wired live into the process service loop. Consumes C0b.3b-a's
> `peer_cred`. **The privilege is the service's, not the socket's** (builder): the raw comms
> socket layer stays unprivileged; the kernel `SocketListener` (the service's door) carries the
> allow-set; `poll'` enforces it at accept. **Birth-seed** (builder's key insight): the
> allow-set starts populated with the owner's pid — the spawner is trusted by construction and
> the pid is free (`getppid()` in the child IS the owner, since spawn is clone3-direct). That
> dissolves the bootstrap circularity, so the gate goes LIVE here (not deferred). Names:
> intueri-blessed (`allow'`/`deny'`/`allowed_pids`/`stranger`).

## The model (pinned — builder, 2026-06-13)

- The allow-set is **just a `HashSet<i32>`** — no `Option`, no open/closed flag. A pid is in it
  or isn't. It lives on the kernel `SocketListener` as `allowed_pids: Mutex<HashSet<i32>>`
  (`CommListener` is `Send+Sync`).
- **Birth-seed:** `listener'` (process) seeds `allowed_pids = {getppid() as i32}` — the owner
  (spawner) is trusted from birth. `getppid()` = the owner because spawn is clone3-direct
  (clone.rs:388; execve preserves pid+ppid) → the child's OS parent IS the spawner.
- `allow'`/`deny'` mutate the set: the owner provisions/de-provisions OTHER pids (beyond itself).
- The gate is in **`poll'`** (the service multiplexer), Fd-branch, via a Rust-testable
  `SocketListener::authorizes(cred) -> bool` (`uid==geteuid()` AND `pid∈allowed_pids`). On
  accept: authorized → serve (`:Connection`); else **bounce the `stranger`** (drop + re-poll,
  never surfaced).
- **Raw `accept'` is NOT gated** (sockets aren't privileged; plumbing; `c0b2c`/`c0b2d` untouched).
  **Thread `poll'` is NOT gated** (the crossbeam handle IS the grant; no pids in shared memory).

## Grounded this session (HEAD `277439c9`)

- `comms::process::peer_cred(fd) -> io::Result<PeerCred{pid,uid,gid}>` (C0b.3b-a, `e1227004`).
- spawn is clone3-direct (clone.rs:388) → `getppid()` in the child = the owner. `getppid()` is
  new (no existing use); `libc::getppid()`/`libc::geteuid()` available.
- `SocketListener { listener: UnixListener }` (kernel/listener.rs); `CommListener: Send+Sync`.
- `listener'` (process) constructs the `SocketListener` in `eval_listener_prime` (runtime.rs);
  the service accept is `eval_poll_prime`'s Fd-branch Listener arm (`SelectOutcome::Listener` →
  `self.listener.accept()` → `Peer::from_socket`) — the ONE gated accept site.

## The contract decision (pinned)

**(1) The set + birth-seed.** `SocketListener` gains `allowed_pids: Mutex<HashSet<i32>>`.
`eval_listener_prime` (process arm) seeds it `{ getppid() as i32 }` at construction.

**(2) `authorizes` (Rust-testable gate decision).**
```rust
impl SocketListener {
    fn authorizes(&self, cred: &crate::comms::process::PeerCred) -> bool {
        cred.uid == unsafe { libc::geteuid() }
            && self.allowed_pids.lock().unwrap().contains(&cred.pid)
    }
}
```

**(3) The verbs (intueri-blessed).**
- `(:wat::kernel::allow' listener pid) -> :wat::core::nil` — insert `pid`. Process-tier only;
  on a thread/crossbeam listener it is a clean error ("`allow'` is a process-tier service gate;
  the thread handle is the grant").
- `(:wat::kernel::deny' listener pid) -> :wat::core::nil` — remove `pid` (future accepts of it
  bounce; the live-connection drop is the connection's RAII, out of scope).

**(4) The gate** — `eval_poll_prime` Fd-branch, after `self.listener.accept()` → `Ok((stream,_))`,
BEFORE `Peer::from_socket`:
```rust
let cred = comms::process::peer_cred(stream.as_raw_fd())?;
if !socket_listener.authorizes(&cred) { drop(stream); continue; }  // bounce the stranger
```
Thread (InMemory) branch untouched. Raw `SocketListener::accept` untouched.

## The gate (probes — the birth-seed makes the e2e non-circular)

1. **`probe_arc209_c0b3bb_gate_decision`** (Rust unit, `tests/comms/`): `socketpair` → `peer_cred`
   → cred.pid is the test's own pid. Assert: empty `allowed_pids` → `authorizes` false; insert
   `std::process::id()` → true; insert a bogus pid only → false; a synthesized cred with a wrong
   uid → false. RED at HEAD (`allowed_pids`/`authorizes` absent).
2. **`probe_arc209_c0b3bb_served`** (wat e2e): a spawned `(process)` service `listener'`s by
   name (birth-seeds `{owner}`), `poll'`-serves (echo n+100); the OWNER (test) connects → its
   pid is the birth-seed → **served** (5→105). Proves serve + the gate doesn't break the owner.
3. **`probe_arc209_c0b3bb_bounced`** (wat e2e — proves the gate is WIRED, the HONEST stranger
   test): the test (owner) spawns the service (birth-seed `{owner}`). It ALSO spawns a **stranger
   child** — a separate `(process)`, `pid ≠ owner`, NOT in the service's allow-set. The stranger
   `send'`s `:ready` to its owner (proves it started), then `connect'`s to the service by name +
   `recv'`s → the service's `poll'` accepts its socket, `peer_cred` → stranger-pid ∉ `{owner}` →
   **bounced** (dropped) → the stranger's `recv'` raises → the stranger dies. The test observes
   `:ready` then the stranger handle's `recv'` raising (died = bounced), while the OWNER's own
   connect was served (probe 2). Same code, different pid, opposite outcome ⇒ the gate is live.
   No `deny'`/owner-pid-reader contrivance — a genuine unauthorized process refused (the model).
4. **`probe_arc209_c0b3bb_verbs`** (wat smoke): `(allow' L pid)`/`(deny' L pid)` on a process
   `listener'` succeed; on a thread `listener'` cleanly ERROR.

Regression: `c0b2c`/`c0b2d` (raw `accept'`, UNGATED) green UNTOUCHED; `c0b1b` (thread `poll'`,
ungated) green; **`c0b3aii`** (the existing process service) — its owner is the test, birth-seeded,
so its client (the test) is served → **still green untouched** (the birth-seed is exactly why
it doesn't break). Nursery 895/4 + full compile.

## Files touched

`src/kernel/listener.rs` (`allowed_pids` + `authorizes`), `src/runtime.rs` (`eval_listener_prime`
birth-seed; `eval_poll_prime` Fd gate; `allow'`/`deny'` arms; maybe `:wat::program::owner-pid`
reader), `src/check.rs` (`allow'`/`deny'` infer → `Listener'<S,R> -> nil`; the owner-pid reader if
added), dispatch registration, the probes. No `comms` change. No raw `accept'` / thread change.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** `getppid()` in the spawned child is NOT the owner (a helper/double-fork between
   them) — STOP, report (grounded clone3-direct → expected owner; if execve or a wrapper breaks
   it, the birth-seed is wrong and must use a conveyed owner-pid instead).
2. **STOP-2:** gating `poll'`'s Fd accept breaks `c0b3aii`/`c0b1b`/thread branch — STOP (the
   birth-seed should make c0b3aii's owner served; if it bounces, getppid≠owner → STOP-1).
3. **STOP-3:** `allow'` on a thread listener can't cleanly error — STOP, report.

## Out of scope (rejected — NOT deferred)

- The **post-spawn block** (owner provisioning THIRD-party clients via `allow'`) = #237. Live
  connection drop on `deny'` (vs future-bounce) = the connection's RAII. **user.program parity** =
  #238. Remote mTLS = `:remote`.

## The deadlock contract carries

The gate is a synchronous local check (`peer_cred` + `Mutex` lookup) inside the existing accept
arm; bounce = `drop + continue` (same shape as the WouldBlock re-poll). No new blocking, no
lifecycle change. [[feedback_vended_primitives_never_deadlock]] [[feedback_optional_is_a_smell]]
