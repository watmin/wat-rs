# Stone C0b — host-parametric connection: `listener'` / `accept'` / `connect'`

> Supersedes `peer-pair'` (Stone C0) as the *cross-host* connection mechanism; C0's
> `peer-pair'` survives as the same-thread degenerate case. Prereq for Stone C
> (defservice). Re-grounded plan: [`DESIGN-REGROUNDED-2026-06-12.md`](./DESIGN-REGROUNDED-2026-06-12.md).

## The hard requirement (builder, 2026-06-12)

> *"a service can run in a thread, a process and a future not-yet-designed remote — each
> has their own unit comm: thread ⇒ crossbeam, process ⇒ pipe, remote ⇒ socket."*
> *"we do thread and process together — them being concurrently correct guarantees
> remote's success."*

The connection primitive is **host-parametric**, and thread **+** process are built and
proven **together** — because they span the whole memory-model space (shared vs. separate),
and remote is separate-memory on another host: the *same* interface process already proves.

## How sockets became emergent (the design event, 2026-06-12)

The builder pushed *"don't solve remote yet."* That constraint — held honestly together with
*"process correct guarantees remote"* — **forced sockets into existence.** The reasoning is
forced, not chosen:

- Dynamically connecting a **new** client to an **already-running** service across separate
  memory has exactly two mechanisms:
  - **fd-passing** (`socketpair` + `SCM_RIGHTS` over the admin channel) — in-memory, no accept
    loop, matches the pipe intuition. **But `SCM_RIGHTS` is same-host only.** It cannot cross a
    network. So it makes *process* correct and leaves *remote* a **different, unvalidated**
    mechanism — it **refutes** the guarantee.
  - **listen / accept / connect** — `AF_UNIX` for process (abstract namespace, in-memory, no
    filesystem), `AF_INET` for remote. **One syscall sequence**, `s/AF_UNIX/AF_INET/`. Process
    correct ⟹ remote correct, **literally**.

The forcing function (`:remote`, perpetually unbuilt-but-binding) is a **generator**: the only
mechanism identical across same-host and other-host separate memory is the listening socket.
**fd-passing is not deferred — it is refuted**, and the refutation is the proof. (The builder
did not reach for sockets; the constraint reached back and handed them over.)

## The trap C0 hit — and why it dissolves here

`ThreadOwnedCell` (`src/rust_deps/custodia.rs`) enforces single-thread custody: a `Peer'` cell
is born owning the thread that minted it; every `send'`/`recv'`/`select'` rejects another
thread. `peer-pair'` mints both cells on the caller → sound *same-thread* (the C0 probe), wrong
for the cross-thread service split. The substrate's own rule (`spawn.rs`): *"raw endpoints are
Send — they move; the Peer cell is constructed on its owning thread only."*

In the listen/accept/connect model each side wraps its end **on its own side** — so custody
holds **by construction** (the wrong-side wrap is unrepresentable). It is the thread-tier slice
of a universal invariant: each side owns its own end (automatic for process/remote — separate
memory; explicit for thread — wrap on the owning thread).

## The model — listen / accept / connect (the socket model, generalized)

A service **listens**; a client **connects**; the service **accepts** each connection, getting
a per-client endpoint wrapped on its own side. The accept-loop *is* the select-loop.

- **`(:wat::kernel::listener' (host)) -> Listener'`** — a listening endpoint of the host's
  unit-comm. `:wat::kernel::`-restricted (defservice-generated; not user-facing).
- **`(:wat::kernel::connect' (host) addr) -> Peer'`** — connect; client-side `Peer'` wrapped on
  the calling (client) side.
- **`(:wat::kernel::accept' Listener') -> Peer'`** — accept the next pending connection;
  server-side `Peer'` wrapped on the calling (service) side.
- **`select'` accepts a `Listener'`** alongside `Peer'`s — a ready listener means "a connection
  is pending; `accept'` it." Provision = TCO-grow; deprovision = drop the peer = TCO-shrink.

The reactor is **reused, not rebuilt**: io_uring already drives `comms::process` fds, and
`IORING_OP_ACCEPT` / `IORING_OP_CONNECT` are native — the listening socket is the only new fd;
the accept loop rides the existing `Select`.

## The four questions — on the concrete listen/accept/connect shape (flat YES/NO)

- **Obvious? YES** — listen/accept/connect is the universally-known connection model; the host
  clause names the unit-comm.
- **Simple? YES** — three verbs, one concept (*a listening service accepting connections*); one
  impl per tier; no `Sender`/fd/socket leaks to the caller; the accept-loop reuses `select'`.
- **Honest? YES** — thread + process **both** ship; remote is `AF_INET` away, **guaranteed by
  construction** (not pretended, not deferred). fd-passing is refuted with its reason on the
  record. Custody can't be violated — the wrong-side wrap is unrepresentable.
- **Good UX? YES** — flipping a service thread→process→remote is one keyword; defservice
  generates the accept-loop; the user writes handlers + state only.

**YES / YES / YES / YES.** The hard requirement eliminated the alternatives at Simple/Honest:
`peer-pair'` (host-agnostic, same-thread) and fd-passing (local-only, breaks the guarantee).

## Per-tier implementation (thread + process built together; remote guaranteed)

| Host | Unit comm | listener' | connect' / accept' | Memory model |
|---|---|---|---|---|
| `(thread)` | crossbeam | in-memory rendezvous channel | handshake ships raw `Send` halves; each side wraps its `Peer'` on its own thread | shared |
| `(process)` | **AF_UNIX socket, abstract namespace** (in-memory, no fs) | `socket`/`bind`/`listen` (abstract `\0`-name) | io_uring `accept`/`connect`; each side wraps its fd as a process `Peer'` | separate (same host) |
| `(remote)` | **AF_INET socket** | the **same** `socket`/`bind`/`listen`, `AF_INET` | the **same** `accept`/`connect` | separate (other host) |

**Thread** uses a crossbeam rendezvous (shared memory *is* the rendezvous — no socket). The
handshake ships only raw `Send` halves through the rendezvous; no cell crosses a thread.
**Process** uses an abstract-namespace `AF_UNIX` listener (purely in-memory, no filesystem
footprint, auto-reaped on close). **Remote** is the `AF_INET` swap — same code, built when
`:remote` arrives, its shape already proven by the process clause.

## Out of scope = affirmatively rejected

- **fd-passing (`SCM_RIGHTS`) for the process tier** — REFUTED, not deferred: same-host only,
  so it cannot be the remote mechanism; using it would make process correct while leaving
  remote unvalidated (it breaks "process ⇒ remote"). The reason is the design.
- **`(remote)` AF_INET clause** — named, built when `:remote` arrives; its success is
  *guaranteed by construction* via the process AF_UNIX clause (same syscall sequence).
- **defservice's admin grant / server-id witness / dispatch / state threading** — Stone C
  (consumes this layer).
- **`peer-pair'` (C0)** — the same-thread degenerate case; not on the cross-host service path.

## Scope + gate (C0b)

- **Build:** `listener'` / `connect'` / `accept'` for **`(thread)` (crossbeam) AND `(process)`
  (abstract-namespace AF_UNIX, io_uring `accept`/`connect`)**, plus `select'`-over-`Listener'`
  + check.rs schemes. `(remote)` named, AF_INET away.
- **Gate — the hand-rolled service proof, written against these:** native-wat `deftest'` (thread
  tier) **and** `deftest-hermetic'` (process tier) that spawn a service select-loop, `connect'`
  a client (TCO-grow), increment a protected scalar through the client peer, assert, deprovision
  (TCO-shrink) — the entire control+data-plane loop running deadlock-free on **both** memory
  models (the thing the legacy counter proofs were `:ignore`d for). Plus nursery + wat-tests clean.

## Estimate

The bulk is the process AF_UNIX socket clause (the thread crossbeam clause is the lighter half).
A candidate for a sonnet strike behind a committed RED proof, or inline; orchestrator's call at
fire time.
