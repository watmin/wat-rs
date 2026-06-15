# DESIGN — Arc 272 step 6a: listener-fd inheritance (the child accepts on a parent-minted listener)

> Opened 2026-06-16. Grounded against HEAD `916dc0e1`. The process tier of the host-agnostic
> `start` (272 step 6, the old host-parity "4b") needs the spawned child to accept on a listener the
> **parent** minted by autobind — the inherited-capability doctrine, vs the c0b3aii model where the
> child binds its own listener **by name** (`socket-address' "wat.arc209.c0b3aii.svc"`, the very name
> arc 272 annihilates). This stone builds the substrate mechanism; 6b wires it into the
> `extend-type ProcessOpts Host` launch; 6c installs the pid-trust half.

## The model we mirror (grounded)

The self-peer is the exact precedent — a substrate-handed capability the child reads by accessor,
never names:

| step | self-peer (built, C0b.3a-0) | listener (this stone) |
|---|---|---|
| parent sets up the fd | owner-link pipes dup2'd → child fd 0/1 (`spawn_process_peer` child branch) | parent autobinds `(listener' (process) :S :R)` → `Bound` (step 2b, `5354c582`); the listener fd is threaded in |
| child seam reconstructs | `run_forms_as_server_child` dups fd0/fd1 → `Peer::from_socket` → `install_self_peer` (verbs.rs:391-411) | reconstruct `UnixListener` from the inherited fd → `Listener'` value → `install_listener` (NEW, same seam) |
| child reads it | `(:wat::program::self-peer :S :R)` → `current_self_peer()` (runtime.rs:18211-18247) | `(:wat::program::listener :S :R)` → `current_listener()` (NEW, same shape) |

The difference that forces a new surface: the self-peer is **substrate-created** (the owner-link exists
because of the spawn), so the child needs no parent cooperation. The listener is **parent-minted
before the spawn** (autobind happens in `start`, where the type is concrete), so it must be **passed
in**. This is the shared-memory partition ([[project_shared_memory_partition_hosting]]): the thread
tier captures `l` in its closure; the process tier cannot (forms can't close over a value) → it is
**handed** `l` explicitly.

## The one contract decision

**The parent-minted listener is an explicit input to the PROCESS clause of `spawn-program'`:**

```wat
(:wat::kernel::spawn-program' (:wat::spawn::process) <listener> <forms> [post-spawn-fn] [env-fn])
```

- `<listener>` is a `Listener'<S,R>` (the `Bound/listener` from the parent's autobind).
- The thread clause is UNCHANGED — `(spawn-program' (thread) <prog>)` — it captures `l`. The
  per-tier signature divergence is honest: it IS the partition, not an inconsistency to paper over.
  (`spawn-program'` is already a per-tier wat defclause in `wat/spawn.wat`, arc 259 — adding a
  process-clause arity is a clause edit, not a central-dispatch edit.)

**Transport:** `spawn_process_peer` (kernel/spawn.rs:612) extracts the listener's raw fd, dup2's it
onto a fixed child fd (mirroring the fd0/1/2 convention — listener lands on **fd 3**), and adds fd 3
to `extra_preserved` so `child_post_fork_init_preserving` (child.rs:299) keeps it across the
close-sweep. `run_forms_as_server_child` reconstructs the `UnixListener` from fd 3, wraps it as a
`Listener'` value (transport-blind `Listener`, C0b.2e-ii), and `install_listener`s it.

## Why no separate Rust-level de-risk (the breadcrumb's recommendation, reconsidered)

The breadcrumb proposed "a Rust-level de-risk probe FIRST like the 2a round-trip." Grounding it shows
it is **redundant**: its claim — *an fd survives clone3 + the `extra_preserved` close-sweep and stays
usable in the child* — is already proven by (a) the autobind round-trip
(`comms::process::autobind_tests::autobind_address_round_trips_in_process`) and (b) the comms-endpoint
fds the process path already inherits via `extra_preserved`. A listening socket is just an fd to
`close_range`; nothing about `accept` vs `read` changes the inheritance. So 6a goes straight to the
**gate probe** (end-to-end wat-level), not a raw-libc de-risk on retired fork helpers
(`run_in_fork` is exactly the legacy fork tooling being killed — do NOT anchor on it).

## The gate probe (RED at HEAD)

`tests/probe_arc272_6a_inherited_listener.rs` — a c0b3aii variant with the name removed:
- parent autobinds `(listener' (process) :i64 :i64)` → `Bound`; takes `l`/`addr`;
- `(spawn-program' (process) l <forms>)` hands the child the listener;
- the child's `main` gets the listener via `(:wat::program::listener :i64 :i64)` — **no
  `socket-address'`, no name** — signals READY over its self-peer, then `poll'`-serves;
- parent `(connect' addr)` dials the minted **capability** (not a name), round-trips 5→105, drops
  the handle to terminate.

RED at HEAD on the first unbuilt rung (`spawn-program'(process)` rejects the listener arg /
`:wat::program::listener` is unknown); GREEN when 6a's wiring + 6b land. Proves serve AND
clean owner-drop termination, exactly as c0b3aii does.

## Decomposition

- **6a-i** — `install_listener` + `current_listener` (services/, mirror `install_self_peer`) +
  `(:wat::program::listener :S :R)` accessor (runtime.rs, mirror `eval_program_self_peer`) + checker arm.
- **6a-ii** — `spawn-program'(process)` accepts the listener arg; `spawn_process_peer` dup2→fd3 +
  `extra_preserved`; `run_forms_as_server_child` reconstructs + installs.
- (6b) `extend-type ProcessOpts Host` launch passes `l` into `spawn-program'(process)`; (6c) pid-trust.

Pairs [[project_rendezvous_inherited_capability]] + [[project_shared_memory_partition_hosting]]
+ [[feedback_reach_stumble_is_the_signal]].
