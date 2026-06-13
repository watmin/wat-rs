# DESIGN-STONE C0b.2 / C0b.3 — the PROCESS connection tier (surface parity for defservice)

> The blocker for `defservice`: it must run on **both** thread and process tiers, so the
> connection layer (`listener'`/`connect'`/`accept'` + the 3-arg `select'` multiplexer) must reach
> the process tier. Today it is **thread-only** (crossbeam). This campaign brings it to parity.
> Model: [`DESIGN-STONE-C0b-host-parametric-connection.md`](./DESIGN-STONE-C0b-host-parametric-connection.md);
> security: [`DESIGN-STONE-C0b-SECURITY.md`](./DESIGN-STONE-C0b-SECURITY.md). Inquisitor draws;
> Shadowdancer builds; Inquisitor weighs.

## Where we are (grounded)

- **At parity already:** `spawn-program' (thread)/(process)`; `send'`/`recv'`/`select'`(1-arg) over
  `Thread'|Process'`; structured-peer-death (`recv'` surfaces structured death both tiers).
- **Thread-ONLY (just shipped, C0b.1/C0b.1b):** `listener'`/`connect'`/`accept'` (always
  `comms::thread::pair`) + the 3-arg `select'` service multiplexer (all `comms::thread::Receiver`,
  `PEER_TYPE_PATH`). The process tier of the connection layer does not exist.
- **Proven raw mechanism:** `probe_arc209_c0b_uds_abstract_spike` — abstract-namespace AF_UNIX
  listen/accept/connect/round-trip via std (`SocketAddr::from_abstract_name`/`bind_addr`/
  `connect_addr`, `UnixListener`/`UnixStream`), no fs entry. The socket plumbing is real; io_uring
  non-blocking accept + `SO_PEERCRED` are the unbuilt layers.

## The live honesty gap (close FIRST)

`infer_listener_prime` (`check.rs:9864`) infers the host arg "for error coverage; type **not
constrained**", and `eval_listener_prime` "**ignores** the host" and always builds a crossbeam
rendezvous. So `(listener' (process) :S :R)` type-checks and silently degrades to a thread
rendezvous — a surface claiming a parity it lacks. **C0b.2a closes this** by making the host
load-bearing.

## The load-bearing finding — a NEW socket-backed `Peer'` kind

A connection peer is **not** the existing `Process<I,O>` peer. `Process` is a *spawned child*:
separate input/output **pipes** + a **`pidfd`** (`peer.rs:236`). A connection peer (from
`connect'`/`accept'` over a UDS) is **one bidirectional `UnixStream`**, no child, no `pidfd`. So the
process connection tier needs a **socket-backed `Peer'`** representation:
- one fd, read+write (split the `UnixStream` into reader/writer halves on the same fd);
- `send'`/`recv'` = EDN write/read over the socket (the `comms::process` EDN wire already exists for
  pipes — reuse the codec, swap the transport from pipe to socket);
- no `pidfd` (a connection is not a supervised child — its death is the transport closing, i.e.
  `:Closed`/`:Lost`, NOT a `Crashed(Failure)`; consistent with the isolation-boundary analysis).

This is the real weight of the campaign: a third peer transport (crossbeam thread / pipe+pidfd
spawned-process / **socket connection**), unified under the `Peer'` send'/recv'/select' surface.

## Decomposition (ordered; each a strike behind a RED probe)

- **C0b.2a — host-dispatch the connection verbs (closes the honesty gap).** `listener'`/`connect'`/
  `accept'` dispatch on the host: `(thread)` → the existing crossbeam impl; `(process)` → (for now)
  a clean `CheckError` "process connection tier not yet built — C0b.2b". Constrain
  `infer_listener_prime`'s host to `ThreadOpts|ProcessOpts` and reject anything else; the eval
  stops ignoring the host. Small; makes the host real so subsequent strikes fill the `(process)`
  arm. **Gate:** `(listener' (process) …)` is a clean check error (not a silent thread); the thread
  probes stay green.
- **C0b.2b — the socket-backed `Peer'`.** A `UnixStream` wrapped as a `Peer'<I,O>`: reader/writer
  split, EDN-over-socket `send'`/`recv'` (reuse the `comms::process` EDN codec). A new
  `comms::socket` (or `comms::process` socket variant). **Gate:** a probe that round-trips a scalar
  over a hand-built `UnixStream` pair wrapped as two `Peer'`s.
- **C0b.2c — process `listener'`/`connect'`/`accept'`.** `(process)` arm: `listener'` =
  `bind_addr` an abstract name + `listen`; `connect'` = `connect_addr`; `accept'` = `accept` → a
  socket-backed `Peer'`. The address is the abstract name (a serializable locator — unlike the
  thread `Address'` in-memory value). **Gate:** the UDS spike, but through the wat verbs.
- **C0b.3a — `select'`-3arg process arm.** `select'` over a process `Listener'` (the listen fd) +
  socket-backed client peers — io_uring (`IORING_OP_ACCEPT` + `POLL_ADD` on the client fds), reusing
  the `comms::process` reactor. The self-peer/`:Shutdown` for a process service is the spawn pidfd
  death OR the parent link — settle at draw time. **Gate:** the C0b.1b service-loop probe, process
  tier (`deftest-hermetic'`), grow/serve/shrink/shutdown — deadlock-free on drop.
- **C0b.3b — the `SO_PEERCRED` allow-set.** `getsockopt(SO_PEERCRED)` at `accept`; the admin's pid
  allow-set; refuse strangers (the security model, already LOCKED). **Gate:** a connecting pid not
  in the allow-set is refused; a provisioned pid is served.

Then thread + process are at **surface parity** → `defservice` (Stone C) generates the loop for
both tiers. `(remote)` = `s/AF_UNIX/AF_INET/` + mTLS, guaranteed by the process clause.

## The deadlock contract carries

Every process strike inherits C0b.1b's rule: the service loop terminates on owner-drop,
structurally. For a process service the wake is the equivalent drain (the parent link / pidfd
closing) surfacing as `:Shutdown` in the 3-arg `select'`. **No process strike may ship a loop that
can deadlock on drop** ([[feedback_vended_primitives_never_deadlock]]).

## Out of scope = rejected

- **`(remote)` AF_INET / mTLS** — named, guaranteed by the process AF_UNIX clause; built when
  `:remote` arrives.
- **`defservice` (Stone C)** — consumes this; blocked until parity.
- **Reusing `Process<I,O>` for connection peers** — REJECTED: a connection has no `pidfd`; forcing
  it through the spawned-child shape is the wrong model. New socket-backed peer.

## Estimate

A multi-strike campaign (5 sub-stones). C0b.2a is small (the honesty gap). C0b.2b/2c (the
socket-backed peer + the abstract-UDS verbs) are the bulk. C0b.3a (io_uring select') is the
delicate reactor work. C0b.3b (SO_PEERCRED) is contained. Each behind a committed RED probe.
