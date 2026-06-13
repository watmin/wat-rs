# BRIEF — Stone C0b.3a-0: the process self-peer verb

**Executor:** Shadowdancer (sonnet). **Anchor cwd:** `/home/watmin/work/holon/wat-rs/` (verify with
`pwd` first; any path containing `.claude/worktrees/` is illegal — re-cd to the anchor; use
`git -C /home/watmin/work/holon/wat-rs` for git). Design:
`DESIGN-STONE-C0b.3a-0-process-self-peer.md` (read it). RED gate committed:
`tests/probe_arc209_c0b3a0_self_peer.rs`.

## The work in one paragraph

Add `(:wat::program::self-peer :S :R) -> SocketPeer'<S,R>` — a verb that returns the spawned process
child's **owner-link** as a `SocketPeer'`: rx over **fd 0** (owner→child), tx over **fd 1**
(child→owner). The runtime constructs it once at the child-only startup seam and installs it in a
`SELF_PEER` thread-local (mirroring `PROGRAM_ENV`); the verb reads that thread-local. In root (no
spawned-child seam) the thread-local is empty → the verb is a clean error. After this, the committed
probe's echo service (parent `send' 5` → child `recv' self` → `send' self 105` → parent `recv'`)
turns GREEN.

## Read in order (the rooms)

1. `src/comms/process.rs:1042` `sender_receiver_from_fd` (C0b.2c) — the template for the new
   split-fd helper; the `Sender`/`Receiver` field shapes + `IoUring::new(4)` are right here.
2. `src/services/client.rs:363`–`405` — `PROGRAM_ENV` thread-local + `install_program_env` (RAII
   `EnvGuard`) + `current_program_env`. **Mirror this exactly** for `SELF_PEER`.
3. `src/kernel/spawn.rs:619` (the child dup2 wiring: fd0=input read, fd1=output write) +
   `src/process/verbs.rs:362` `run_forms_as_server_child` — the **child-only** seam where you install
   the self-peer (root never calls this → the root-vs-child guard is the install site itself).
4. `src/runtime.rs:17535` `eval_program_env` + `:3804` the dispatch arm (`":wat::program::env" =>
   …`) + `:3807` (`cpu-count`) — the verb impl + dispatch pattern to mirror.
5. `src/check.rs:9852` `infer_socket_pair_prime` + `parse_peer_pair_type_arg` + `socket_pair_tuple`
   (`:9879`) — the type-keyword-arg parsing + `SocketPeer'` result shape to mirror.
6. `src/kernel/spawn.rs:142` `SOCKET_PEER_TYPE_PATH` + `:149` `SocketPeerCell` — the opaque the
   self-peer is wrapped as (so `send'`/`recv'`/`select'` drive it unchanged).

## Implementation sketch (fill the shape; do not invent a different one)

**(A) `src/comms/process.rs` — split-fd helper (beside `sender_receiver_from_fd`).**
```rust
/// Wrap a SEPARATE read fd + write fd as a (Sender<T>, Receiver<T>) pair — for a peer over a
/// pipe PAIR (e.g. a process child's fd0 read / fd1 write owner-link), not one bidirectional
/// socket fd. No try_clone (the two fds are already distinct). Per-Receiver IoUring::new(4).
pub fn sender_receiver_from_split_fds<T: HolonRepresentable>(
    read_fd: OwnedFd,
    write_fd: OwnedFd,
) -> std::io::Result<(Sender<T>, Receiver<T>)> {
    let receiver = Receiver {
        read_fd,
        accumulator: RefCell::new(Vec::new()),
        ring: RefCell::new(IoUring::new(4).map_err(|e| std::io::Error::other(
            format!("IoUring::new(4) failed at sender_receiver_from_split_fds: {}", e)))?),
        _phantom: PhantomData,
    };
    Ok((Sender { write_fd, _phantom: PhantomData }, receiver))
}
```

**(B) `src/services/client.rs` — `SELF_PEER` thread-local (mirror `PROGRAM_ENV` verbatim).**
```rust
thread_local! {
    static SELF_PEER: RefCell<Option<crate::runtime::Value>> = const { RefCell::new(None) };
}
pub struct SelfPeerGuard { _private: () }    // RAII: clears SELF_PEER on drop (mirror EnvGuard)
impl Drop for SelfPeerGuard { fn drop(&mut self) { SELF_PEER.with(|c| *c.borrow_mut() = None); } }
pub fn install_self_peer(peer: crate::runtime::Value) -> SelfPeerGuard {
    SELF_PEER.with(|c| *c.borrow_mut() = Some(peer));
    SelfPeerGuard { _private: () }
}
pub fn current_self_peer() -> Option<crate::runtime::Value> {
    SELF_PEER.with(|c| c.borrow().clone())
}
```
Re-export from `src/services.rs`/`mod.rs` alongside `install_program_env`/`current_program_env`
(match how those are exposed).

**(C) `src/process/verbs.rs` `run_forms_as_server_child` — install before running the forms.**
At the TOP of the child's run (after the dup2 wiring is already in place — fd0=read, fd1=write),
construct the self-peer and install it for the child's lifetime:
```rust
// C0b.3a-0 — hand the forms-child its owner-link as a self-peer (rx=fd0, tx=fd1). dup so the
// self-peer owns independent OwnedFds (EOF on the dup'd read still fires when the owner drops fd0).
use std::os::fd::{BorrowedFd, OwnedFd, AsRawFd};
let read_fd: OwnedFd = unsafe { BorrowedFd::borrow_raw(0) }.try_clone_to_owned()
    .expect("dup fd0 for self-peer");
let write_fd: OwnedFd = unsafe { BorrowedFd::borrow_raw(1) }.try_clone_to_owned()
    .expect("dup fd1 for self-peer");
let (tx, rx) = crate::comms::process::sender_receiver_from_split_fds::<String>(read_fd, write_fd)
    .expect("build self-peer from fd0/fd1");
let self_peer_value = crate::rust_deps::marshal::make_rust_opaque(
    crate::kernel::spawn::SOCKET_PEER_TYPE_PATH,
    std::sync::Arc::new(crate::rust_deps::custodia::ThreadOwnedCell::new(Some(
        crate::kernel::peer::SocketPeer { tx, rx }))),
);
let _self_peer_guard = crate::services::install_self_peer(self_peer_value);
// ... existing: run the forms / invoke main (guard held until the child _exits) ...
```
Confirm the exact handles in scope (the dup2 is at `spawn.rs:619`; in `run_forms_as_server_child` fd
0/1 are already the wired stdio — borrow them by number). Keep `_self_peer_guard` alive across the
forms run.

**(D) `src/runtime.rs` — the verb + dispatch.**
`eval_program_self_peer` beside `eval_program_env` (`:17535`):
```rust
fn eval_program_self_peer(args: &[WatAST], list_span: &Span) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::program::self-peer";
    // 2 type-keyword args (:S :R) — checker-only; validate they're keywords (mirror socket-pair').
    if args.len() != 2 { /* ArityMismatch expected 2 */ }
    for (i, a) in args.iter().enumerate() {
        if !matches!(a, WatAST::Keyword(_, _)) { /* MalformedForm: arg i must be a type keyword */ }
    }
    crate::services::current_self_peer().ok_or_else(|| RuntimeError {
        span: list_span.clone(),
        kind: RuntimeErrorKind::MalformedForm { head: OP.into(),
            reason: "no self-peer — (:wat::program::self-peer) is only valid inside a spawned \
                     process service; root has no owner-link".into() },
    }.into())
}
```
Dispatch arm beside `:3804`: `":wat::program::self-peer" => eval_program_self_peer(args, list_span),`.

**(E) `src/check.rs` — infer `:wat::program::self-peer` → `SocketPeer'<S,R>`.**
Mirror `infer_socket_pair_prime` (`:9852`): 2 args via `parse_peer_pair_type_arg` → return
`TypeExpr::Parametric { head: "wat::kernel::SocketPeer'".into(), args: vec![s, r] }`. Wire it into
the same infer-dispatch that routes `:wat::program::env`/`:wat::program::cpu-count`/`socket-pair'`
(grep how those infer entries are registered and add the `:wat::program::self-peer` entry beside
them).

## Blast radius (bounded)

`src/comms/process.rs`, `src/services/client.rs` (+ the `services` re-export), `src/process/verbs.rs`,
`src/runtime.rs`, `src/check.rs`. NO new files (the probe exists). NO change to root's
`invoke_user_main` (the install is child-only). NO thread-tier change (the thread service gets its
self-peer as a fn arg — untouched). NO `select'`-3arg / reactor / accept work (that's C0b.3a-i/ii).

## STOP triggers (rejection criteria — ship nothing, report the gap)

1. **STOP-1:** if `Sender`/`Receiver` cannot be constructed in `comms::process` the way
   `sender_receiver_from_fd` does (fields not reachable) — STOP, report.
2. **STOP-2:** if `run_forms_as_server_child` does NOT have fd 0/1 already wired as the input-read /
   output-write pipe ends at the point you install (verify against `spawn.rs:619`) — STOP, report
   the actual wiring; do not install over the wrong fds.
3. **STOP-3:** if the infer-dispatch for `:wat::program::*` verbs cannot accept a new entry without a
   structural change beyond mirroring `socket-pair'`/`env` — STOP, report.

Rejection criteria, not permission to ship a workaround.

## The gate

`cargo build --release` clean, then the RED probe must go GREEN:
```
cargo test --release -p wat --test probe_arc209_c0b3a0_self_peer
```
must PASS (echo 5→105). Then no regression on the connection-tier + process surface:
```
cargo test --release -p wat --test nursery probe_arc209_c0b2c probe_arc209_c0b2b_socket_peer probe_arc209_c0b1b_select_listener -- --test-threads=1
cargo test --release -p wat --test wat_hermetic_round_trip
cargo test --release -p wat --test probe_arc211_program_env_ambient   # program-env verbs intact
```
Report the exact `test result:` line for each. Do NOT commit — the Inquisitor weighs and commits.
