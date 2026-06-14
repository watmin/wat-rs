# BRIEF — Stone C0b.3b-b: the allow-set gate, LIVE (the service refuses the stranger)

**Executor:** Shadowdancer (sonnet). **Anchor:** `/home/watmin/work/holon/wat-rs/` (verify `pwd`;
operate ONLY here; `git -C /home/watmin/work/holon/wat-rs`). Design (read it fully):
`DESIGN-STONE-C0b.3b-b-allow-set-gate.md`. The two RED probes are on disk + verified RED at HEAD:
`tests/probe_arc209_c0b3bb_bounced.rs` (`stranger_is_bounced` RED — the stranger is served 107
instead of bounced; `owner_served_via_birth_seed` already GREEN) and
`tests/probe_arc209_c0b3bb_verbs.rs` (both RED — `UnknownFunction(":wat::kernel::allow'")`).
Do NOT commit — the Inquisitor weighs every gate against its own re-run.

## The work in one paragraph

Wire the SO_PEERCRED gate LIVE into the process service loop. The kernel `SocketListener`
(`src/kernel/listener.rs`) gains a birth-seeded allow-set; `poll'`'s Fd-branch
(`src/runtime.rs`) reads `peer_cred` on each accepted socket and serves only an authorized
connector, dropping (bouncing) the stranger; two new verbs `allow'`/`deny'` let the owner
mutate the set; the checker (`src/check.rs`) types them. The birth-seed is the key: the allow-set
starts `{getppid()}` = the owner (spawner), trusted by construction — so the gate is LIVE here,
not deferred, and the owner is served by construction (the `peer_cred` primitive shipped in
C0b.3b-a is the mechanism this consumes).

## Read in order (the rooms)

1. `src/kernel/listener.rs:254–341` — `SocketListener { listener: UnixListener }` + its
   `CommListener` impl. ADD the allow-set field + `authorizes`/`allow`/`deny` methods here.
2. `src/kernel/listener.rs:364–367` — `Listener::from_socket(listener)`. This is the BIRTH-SEED
   point: construct the `SocketListener` with `allowed_pids` seeded `{ getppid() }`. (It is called
   once, from `eval_listener_prime`'s process arm at `src/runtime.rs:18169`, which runs IN the
   spawned service child — so `getppid()` there is the owner. Seeding inside `from_socket` makes
   an unseeded `SocketListener` unrepresentable.)
3. `src/comms/process.rs:134–172` — `PeerCred { pid: i32, uid: u32, gid: u32 }` +
   `pub fn peer_cred(fd: RawFd) -> io::Result<PeerCred>` (C0b.3b-a, the mechanism `authorizes`
   checks against).
4. `src/runtime.rs:24201–24244` — `eval_poll_prime` Fd-branch, the `SelectOutcome::Listener` arm.
   The `Ok((stream, _addr))` at `:24208` is the ONE gated accept site; `socket_listener` is bound
   at `:24108` (`&SocketListener`). Insert the gate after the `Ok` match, before
   `sender_receiver_from_fd` at `:24211`.
5. `src/runtime.rs:4551–4585` — the kernel head-match dispatch (`poll'`, `listener'`, `connect'`,
   `accept'`). ADD `allow'`/`deny'` arms here → `eval_allow_prime`/`eval_deny_prime`.
6. `src/runtime.rs:18398–18437` — `eval_accept_prime` (the shape to mirror for `eval_allow_prime`:
   eval arg, downcast the `LISTENER_TYPE_PATH` opaque to `&Listener`).
7. `src/check.rs:4863–4939` — the check head-match dispatch (`poll'`…`accept'`). ADD `allow'`/`deny'`
   arms → `infer_allow_prime`/`infer_deny_prime`.
8. `src/check.rs:10207–10257` — `infer_accept_prime` (the shape to mirror: reduce arg[0] to
   `Listener'<S,R>`).

## Implementation sketch (fill the shape)

### (1) `src/kernel/listener.rs` — the allow-set on `SocketListener`

```rust
use std::collections::HashSet;
use std::sync::Mutex;

pub struct SocketListener {
    pub(crate) listener: UnixListener,
    /// Arc 209 C0b.3b-b — the allow-set: a pid is in it or it isn't. Birth-seeded with
    /// the owner's pid (getppid() = the spawner, trusted by construction). A connector
    /// whose SO_PEERCRED pid ∉ this set (or whose uid ≠ ours) is bounced at accept.
    pub(crate) allowed_pids: Mutex<HashSet<i32>>,
}

impl SocketListener {
    /// The gate decision (SO_PEERCRED is local mTLS): serve only our own euid AND a pid in
    /// the allow-set. Pure + Rust-testable.
    pub(crate) fn authorizes(&self, cred: &crate::comms::process::PeerCred) -> bool {
        cred.uid == unsafe { libc::geteuid() }
            && self.allowed_pids.lock().unwrap().contains(&cred.pid)
    }
    /// Owner provisions another pid (beyond the birth-seeded self).
    pub(crate) fn allow(&self, pid: i32) {
        self.allowed_pids.lock().unwrap().insert(pid);
    }
    /// Owner de-provisions a pid (future accepts of it bounce).
    pub(crate) fn deny(&self, pid: i32) {
        self.allowed_pids.lock().unwrap().remove(&pid);
    }
}
```

Birth-seed in `Listener::from_socket`:

```rust
pub fn from_socket(listener: UnixListener) -> Self {
    // Arc 209 C0b.3b-b — BIRTH-SEED: {getppid()} = the owner. getppid() in the service
    // child IS the spawner — spawn is clone3-direct (no CLONE_PARENT; clone.rs:388) and
    // run_forms_as_server_child runs the body in that child (spawn.rs:632). Dissolves the
    // bootstrap circularity → the gate is LIVE.
    let owner_pid = unsafe { libc::getppid() };
    let mut seed = HashSet::new();
    seed.insert(owner_pid);
    Listener { inner: Box::new(SocketListener { listener, allowed_pids: Mutex::new(seed) }) }
}
```

Add an in-module unit test (the gate decision — the one place the wrong-`uid` branch is
testable, which the e2e probes cannot reach):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::comms::process::PeerCred;

    fn listener_with(pids: &[i32]) -> SocketListener {
        // Bind a throwaway abstract UDS just to own a UnixListener; seed the set directly.
        use std::os::linux::net::SocketAddrExt;
        use std::os::unix::net::{SocketAddr, UnixListener};
        let sa = SocketAddr::from_abstract_name(b"wat.arc209.c0b3bb.unit").unwrap();
        let listener = UnixListener::bind_addr(&sa).unwrap();
        SocketListener { listener, allowed_pids: Mutex::new(pids.iter().copied().collect()) }
    }

    #[test]
    fn authorizes_only_my_uid_and_an_allowed_pid() {
        let me = unsafe { libc::geteuid() };
        let mine = std::process::id() as i32;
        let sl = listener_with(&[]);
        assert!(!sl.authorizes(&PeerCred { pid: mine, uid: me, gid: 0 })); // empty set → no
        sl.allow(mine);
        assert!(sl.authorizes(&PeerCred { pid: mine, uid: me, gid: 0 })); // allowed → yes
        assert!(!sl.authorizes(&PeerCred { pid: mine + 999_999, uid: me, gid: 0 })); // wrong pid
        assert!(!sl.authorizes(&PeerCred { pid: mine, uid: me + 1, gid: 0 })); // wrong uid → no
    }
}
```

### (2) `src/runtime.rs` — the gate in `eval_poll_prime` Fd-branch (the `Ok((stream, _addr))` arm at `:24208`)

After the `Ok((stream, _addr)) =>` opens, BEFORE the existing `let peer_value = { ... }`:

```rust
Ok((stream, _addr)) => {
    // Arc 209 C0b.3b-b — THE GATE: the kernel vouches for the connector's {pid,uid,gid};
    // serve only an authorized one, else bounce the stranger (drop + re-accept).
    use std::os::fd::AsRawFd;
    let cred = crate::comms::process::peer_cred(stream.as_raw_fd()).map_err(|e| RuntimeError {
        span: list_span.clone(),
        kind: RuntimeErrorKind::MalformedForm {
            head: OP.into(),
            reason: format!("poll' (process tier): peer_cred on accepted socket: {}", e),
        },
    })?;
    if !socket_listener.authorizes(&cred) {
        drop(stream);   // bounce the stranger — close the accepted fd, re-accept next
        continue;       // back to socket_listener.listener.accept() (the enclosing loop)
    }
    let peer_value = { /* ...existing wrap unchanged... */ };
    break Value::Enum(/* ...Connection unchanged... */);
}
```

(`AsRawFd` is already `use`d in this branch at `:24107`; the `continue` re-enters the `loop` at
`:24207`, exactly the existing `WouldBlock` re-accept shape. The thread/InMemory branch and raw
`accept'` are UNTOUCHED — the privilege is the service's, not the socket's.)

### (3) `src/runtime.rs` — the verbs (`allow'`/`deny'`) + 2 head arms at `:4585`

```rust
":wat::kernel::allow'" => eval_allow_prime(args, list_span, env, sym),
":wat::kernel::deny'"  => eval_deny_prime(args, list_span, env, sym),
```

`eval_allow_prime` (mirror `eval_accept_prime`'s downcast; `eval_deny_prime` is identical but
calls `.deny`):

```rust
fn eval_allow_prime(args, list_span, env, sym) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::kernel::allow'";
    // arity 2: (allow' listener pid)
    // arg0 → LISTENER_TYPE_PATH opaque → &Listener (downcast_ref_opaque, as in eval_accept_prime)
    // listener.inner.as_any_ref().downcast_ref::<SocketListener>():
    //   Some(sl) => { let pid = <eval arg1 → i64>; sl.allow(pid as i32); Ok(Value::Unit) }
    //   None     => Err(MalformedForm { head: OP, reason:
    //       "allow' is a process-tier service gate; a thread listener's handle IS the grant" })
}
```

The `None` branch's reason MUST contain the text `process-tier` (the verbs probe asserts it).
Return `Value::Unit` (wat `nil`) on success.

### (4) `src/check.rs` — `infer_allow_prime`/`infer_deny_prime` + 2 head arms at `:4939`

Mirror `infer_accept_prime`: arity 2; reduce `arg[0]` to `Listener'<S,R>` (same
`TypeExpr::Parametric { head: "wat::kernel::Listener'", args: [S, R] }` check); unify `arg[1]`
with `i64`; the result type is `nil` (`TypeExpr::Path(":wat::core::nil")` — confirm how nil is
spelled in this checker, e.g. grep an existing `-> :wat::core::nil` intrinsic's infer fn). The
tier (thread vs process) is NOT known at check time (both are `Listener'<S,R>`), so the
tier-rejection is a RUNTIME error only — the checker types both tiers identically.

## Blast radius

`src/kernel/listener.rs` (allow-set + methods + birth-seed + unit test), `src/runtime.rs` (gate
+ 2 eval fns + 2 head arms), `src/check.rs` (2 infer fns + 2 head arms). NO `comms` change. NO
change to raw `accept'`, the thread/InMemory branch, or any existing test.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** the birth-seed serves the WRONG party — i.e. `owner_served_via_birth_seed` or
   `probe_arc209_c0b3aii_process_service_loop` goes RED (the owner is bounced). That means
   `getppid()` in the service child is NOT the owner (a helper/double-fork between them); STOP
   and report — the birth-seed must then carry a conveyed owner-pid instead.
2. **STOP-2:** the gate breaks the thread/InMemory branch or raw `accept'` (any pre-existing
   green test goes red); STOP — the gate belongs ONLY in the Fd-branch's accept arm.
3. **STOP-3:** `nil`/`Listener'<S,R>` cannot be expressed in `infer_allow_prime` the way
   `infer_accept_prime` expresses its types; STOP and report the exact type-construction gap.

## The gate (report each exact `test result:` line; do NOT commit)

```
cargo test --release -p wat --test probe_arc209_c0b3bb_bounced -- --test-threads=1   # 2 passed
cargo test --release -p wat --test probe_arc209_c0b3bb_verbs   -- --test-threads=1   # 2 passed
cargo test --release -p wat --lib kernel::listener             -- --test-threads=1   # unit GREEN
cargo test --release -p wat --test probe_arc209_c0b3aii_process_service_loop -- --test-threads=1  # still GREEN (birth-seed serves its owner)
cargo test --release -p wat --test comms -- --test-threads=1                         # all green (peer_cred/3b-a intact)
cargo test --release -p wat --test nursery -- --test-threads=1                       # 895 passed / 4 failed (baseline; ZERO new)
cargo test --release --workspace --no-run                                            # full surface compiles
```

## Prior comparable (copy the shape)

`BRIEF-STONE-C0b.3b-a.md` (the same `src/comms/` + `src/kernel/` neighborhood; the `peer_cred`
mechanism this consumes) and the c0b3aii service-loop probe (the serve-loop the gate sits inside).
