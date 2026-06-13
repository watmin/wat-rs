# DESIGN-STONE C0b.3a-0 — the process self-peer verb

> Prerequisite for the process service loop (C0b.3a-ii). A process service must hold a **self-peer**
> — its owner-link — to pass as `select'` arg0 and watch for `:Shutdown` (owner-drop). The thread
> tier hands the self-peer as a fn arg (`spawn_thread_peer:471`); the **process tier cannot** —
> separate memory carries serialized forms, not live `Peer'` handles (arc 213 program-over-the-wire).
> So the process child must **obtain its own self-peer** from its inherited fds via a verb.
> (Decision A vs B settled by the four questions, 2026-06-13: the verb, not a passed value.)

## Where we are (grounded, read this session)

- **Process child wiring** (`spawn_process_peer`, `spawn.rs:619` — "THE ONE WIRE"): the child
  `dup2`'s **fd 0** = input-pipe read end (owner→child, what the parent's `send'` writes), **fd 1** =
  output-pipe write end (child→owner, what the parent's `recv'` reads), **fd 2** = the err channel.
  The child runs `run_forms_as_server_child(forms, …)` (`process/verbs.rs:362`) → `:user::main`,
  reading fd0 / writing fd1 on the comms::process EDN wire (same wire as `send'`/`recv'`).
- **The thread self-peer handoff** (`spawn_thread_peer:471`): `self_peer = Peer'{tx: output_tx,
  rx: input_rx}`, handed to `prog(self_peer)`. The process tier has no analog — `run_forms_as_server_child`
  hands the child nothing; the child talks to its owner through raw fd0/fd1.
- **The mirror pattern** (`services/client.rs:368`): `PROGRAM_ENV` thread-local +
  `install_program_env` (RAII `EnvGuard`) + `current_program_env`. Installed at the post-bootstrap
  seam `invoke_user_main_orchestrated:1101` (covers root AND fork-child).
- **The peer machinery** (C0b.2b/2c): `SocketPeer<I,O> { tx: Sender, rx: Receiver }` (comms::process
  EDN+io_uring), `SOCKET_PEER_TYPE_PATH`; `send'`/`recv'`/`select'` already dispatch on it;
  `project_peer_io` (`check.rs:10332`) knows `SocketPeer'`. `sender_receiver_from_fd(fd)` (C0b.2c)
  dups one fd for tx+rx (the socket model).

## The contract decision (pinned)

**`(:wat::program::self-peer :S :R) -> SocketPeer'<S,R>`** — a nullary-of-state verb taking two
checker-only type keywords (like `socket-pair'`/`listener'`), returning the child's owner-link as a
`SocketPeer'`: **rx over fd 0** (recv' reads owner→child; **EOF = owner dropped = `:Shutdown`**),
**tx over fd 1** (send' writes child→owner). Runtime wire is `String`/EDN (checker-only `S`/`R`,
exactly as `socket-pair'`). Reuses `SOCKET_PEER_TYPE_PATH` so `send'`/`recv'`/`select'` drive it
unchanged.

Why a verb (not a passed value): the four-questions verdict — the process child runs shipped forms in
separate memory and cannot receive a live `Peer'`; it must obtain its self-peer from its own
inherited fds. The verb mirrors the arc-259 program-env escape-hatch (the child interrogates the
runtime about itself).

## The root-vs-child distinction (honest, by construction)

The self-peer is installed **only** at the child-only seam `run_forms_as_server_child`
(`process/verbs.rs:362`) — which **only a spawned process child reaches**. Root's
`invoke_user_main_orchestrated` never calls it, so root never installs a self-peer, so
`(self-peer)` in root is a clean error ("no owner-link — not a spawned peer"). No flag, no
peer-kind check — the install site IS the guard. (Mirrors how `install_program_env` scopes the env.)

## The mechanism

1. **`comms::process::sender_receiver_from_split_fds<T>(read_fd: OwnedFd, write_fd: OwnedFd) ->
   io::Result<(Sender<T>, Receiver<T>)>`** — like `sender_receiver_from_fd` but two distinct fds:
   `Sender { write_fd }` + `Receiver { read_fd, ring: IoUring::new(4) }`. (The self-peer is over a
   pipe PAIR, not one socket fd — no `try_clone`.)
2. **A `SELF_PEER` thread-local + `install_self_peer(Value) -> SelfPeerGuard` + `current_self_peer()`**
   in `services/client.rs`, mirroring `PROGRAM_ENV` exactly (RAII guard, `const RefCell::new(None)`).
3. **Install at `run_forms_as_server_child`** (child path, after the dup2 wiring): `dup(0)`→read_fd,
   `dup(1)`→write_fd (dup so the self-peer owns independent OwnedFds without closing the child's real
   fd0/fd1), `sender_receiver_from_split_fds` → `SocketPeer`, `make_rust_opaque(SOCKET_PEER_TYPE_PATH,
   Arc::new(ThreadOwnedCell::new(Some(peer))))`, `install_self_peer(value)` (held for the child's
   lifetime). dup so EOF on the dup'd read_fd still fires when the owner drops fd0's write end.
4. **Verb `eval_program_self_peer`** (`runtime.rs`, beside `eval_program_env:17535`): validates 2
   type-keyword args; reads `current_self_peer()`; errors if `None` ("no self-peer — `(self-peer)` is
   only valid inside a spawned process service; root has no owner-link"); returns the installed value.
   Dispatch arm `":wat::program::self-peer" => eval_program_self_peer(args, list_span)` (beside
   `:3804`).
5. **Check `infer` for `:wat::program::self-peer`**: 2 type-keyword args → `SocketPeer'<S,R>`
   (`Parametric { "wat::kernel::SocketPeer'", [S, R] }`). Mirror `infer_socket_pair_prime:9852`.

## The gate (RED at HEAD → GREEN on ship)

A spawned process **echo service via the self-peer** — proves both directions of the owner-link:
```
;; child: get self-peer, echo owner→child + 100 back to the owner
(spawn-program' (process)
  (forms (defn :user::main [] -> nil
    (let [self (:wat::program::self-peer :i64 :i64)
          x    (recv' self)]          ;; reads fd0 (what the parent send'd)
      (send' self (+ x 100))))))      ;; writes fd1 (parent recv's it)
;; parent: send' 5 to the handle → recv' the handle → 105
```
RED at HEAD: `(:wat::program::self-peer …)` doesn't exist → check error. GREEN once C0b.3a-0 ships.
Proves: the verb returns a working self-peer; rx=fd0 receives the owner's `send'`; tx=fd1 reaches the
owner's `recv'`. (The fd0-EOF→`:Shutdown` path is exercised by C0b.3a-ii's service-loop probe.)

## Out of scope = rejected (named, not deferred)

- **The reactor listener-arm + non-blocking accept** — C0b.3a-i.
- **The 3-arg `select'` process branch + service loop** — C0b.3a-ii (consumes this self-peer).
- **A thread-tier `(self-peer)` verb** (for full tier-agnostic `serve`) — the thread tier already
  hands self as a fn arg; a thread verb is a defservice-layer uniformity nicety, built only if
  Stone C needs it ([[feedback_dont_build_the_forcing_function]]).
- **fd1 sharing with `println`/stdout** — "THE ONE WIRE": a service that holds the self-peer uses
  `send' self`, not `println` (both write fd1; don't mix). Documented, not enforced here.

## The deadlock contract carries

The self-peer's rx (fd0) EOFs when the owner drops the spawn handle (RAII drains the input pipe) —
that EOF is the `:Shutdown` wake C0b.3a-ii's reactor watches. C0b.3a-0 builds the value; C0b.3a-ii
watches it. [[feedback_vended_primitives_never_deadlock]]
