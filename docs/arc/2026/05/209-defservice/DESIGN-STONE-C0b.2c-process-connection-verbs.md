# DESIGN-STONE C0b.2c — process `listener'`/`connect'`/`accept'` over abstract UDS

> The fourth rung of the process connection tier (`DESIGN-STONE-C0b.2-process-connection-tier.md`).
> C0b.2a made the `listener'` host load-bearing (a `(process)` host is a clean check error);
> C0b.2b built the socket-backed `SocketPeer'` + `socket-pair'` and PROVED io_uring drives an
> AF_UNIX socket exactly like a pipe. C0b.2c fills the `(process)` arm of the three connection
> verbs: a service binds an abstract-namespace UDS, a client connects, the service accepts a
> per-connection `SocketPeer'`, and a request/response round-trips. Inquisitor draws; Shadowdancer
> builds; Inquisitor weighs.

## Where we are (grounded, read this session)

- **Thread tier (C0b.1/C0b.1b), the template:** `listener'` (host, :S, :R) mints a crossbeam
  rendezvous and returns `Tuple[Listener'<S,R> (= raw Receiver), Address'<S,R> (= raw Sender)]`
  (`runtime.rs:17916`, `eval_listener_prime` — host arg currently **ignored**). `connect'` (addr =
  Sender) mints two channel pairs, wraps the client `Peer'` on this thread, ships the server's raw
  halves one-way over the rendezvous Sender, returns `Peer'<S,R>` (`runtime.rs:17958`). `accept'`
  (listener = Receiver) blocks on the rendezvous, unpacks the connect-request, wraps the server
  `Peer'<R,S>` on this thread (`runtime.rs:18165` + `wrap_connect_request:18036`).
- **C0b.2a:** `infer_listener_prime` (`check.rs:9897`) constrains the host to
  `:wat::spawn::ThreadOpts`; a non-thread host is a `TypeMismatch` check error naming the gap.
- **C0b.2b:** `SocketPeer<I,O>` (`peer.rs:335`) = a `comms::process` `Sender<String>`/`Receiver<String>`
  pair over a socket fd (EDN + newline framing + io_uring), no pidfd, RAII lifecycle, custody via
  `ThreadOwnedCell`. `SOCKET_PEER_TYPE_PATH = ":wat::kernel::SocketPeer'"` (`spawn.rs:142`).
  `comms::process::socket_pair()` (`process.rs:1051`) mints a connected pair from
  `socketpair(2)`, dup'ing each fd for independent Sender/Receiver `OwnedFd` lifetimes.
  `send'`/`recv'` already have socket arms dispatching on `SOCKET_PEER_TYPE_PATH`
  (`runtime.rs:22723`, `:23018`); the checker's `project_peer_io` (`check.rs:10332`) already
  recognizes `wat::kernel::SocketPeer'`.
- **Proven OS mechanism:** `probe_arc209_c0b_uds_abstract_spike` — `SocketAddr::from_abstract_name` /
  `UnixListener::bind_addr` / `UnixStream::connect_addr` / `listener.accept()` round-trip a scalar
  over an abstract-namespace AF_UNIX socket (in-memory, no fs entry), same syscall sequence as
  AF_INET. std-stable since 1.70.

## The contract decision (pinned)

**The process tier gets DISTINCT checker-only types — `SocketListener'<S,R>` and
`SocketAddress'<S,R>` — NOT a reuse of the thread `Listener'`/`Address'`.** The verbs dispatch on
the input's runtime kind and the checker derives the result type from the input's static type:

| verb | thread input → output | process input → output |
|---|---|---|
| `listener'` | `(thread)` → `Tuple[Listener'<S,R>, Address'<S,R>]` | `(process)` → `Tuple[SocketListener'<S,R>, SocketAddress'<S,R>]` |
| `connect'` | `Address'<S,R>` → `Peer'<S,R>` | `SocketAddress'<S,R>` → `SocketPeer'<S,R>` |
| `accept'`  | `Listener'<S,R>` → `Peer'<R,S>` | `SocketListener'<S,R>` → `SocketPeer'<R,S>` |

Four questions (all YES): **Obvious** — the type names the transport. **Simple** — one match arm per
verb; the result type derives from the input type, the pattern the thread tier already uses.
**Honest** — the thread `Address'` is an in-memory `Sender`; the process address is a *name*; they
are different things and ADT-wat has no anonymous union to merge them honestly
([[feedback_optional_is_a_smell]]). **Good UX** — the distinct types carry the tier through to
`send'`/`recv'`/`select'`, all already `SocketPeer'`-aware, so a process service is typed end-to-end.

## The mechanism (the `(process)` arms)

- **`listener'` (process):** generate a unique abstract name (`format!("wat.arc209.{pid}.{n}")`,
  `pid` = `std::process::id()`, `n` = a process-local `AtomicU64`); `UnixListener::bind_addr` it.
  Return `Tuple[ opaque(SOCKET_LISTENER) = UnixListener, opaque(SOCKET_ADDRESS) = name String ]`.
  Both are freely `Send`/`Sync` (a UnixListener and a String) — **no `ThreadOwnedCell`**, exactly
  like the thread tier's raw Receiver/Sender carry no custody.
- **`connect'` (process):** downcast the addr opaque → `&String` name; `UnixStream::connect_addr`;
  wrap the stream's fd as a `SocketPeer'<S,R>` (the connection is queued in the listener backlog at
  the kernel level — connect returns before accept runs). Return `SocketPeer'<S,R>`.
- **`accept'` (process):** downcast the listener opaque → `&UnixListener`; `.accept()` (blocks until
  a connection — the honest wire-wait, cf. `mora`); wrap the conn fd as a `SocketPeer'<R,S>`.
  Return `SocketPeer'<R,S>`.
- **fd → SocketPeer:** factor `comms::process::sender_receiver_from_fd<T>(fd: OwnedFd) ->
  io::Result<(Sender<T>, Receiver<T>)>` out of `socket_pair`'s per-end logic (write_fd = fd,
  read_fd = `fd.try_clone()`, per-Receiver `IoUring::new(4)`); `socket_pair` calls it twice (DRY,
  `solvere`); `connect'`/`accept'` call it once each on their `UnixStream`'s `OwnedFd`. A
  `UnixStream` → `OwnedFd` via `OwnedFd::from(stream)`.

The host (`(thread)`/`(process)`) is distinguished at runtime by the record's `class_fqdn` (the
arc-259 dispatch fact, `runtime.rs` `value_matches_type_by_name:5632`): `eval_listener_prime` grows
`env, sym`, evaluates `args[0]`, and matches `Value::wat__Record { class_fqdn, .. }` against
`"wat::spawn::ProcessOpts"` / `"wat::spawn::ThreadOpts"`. `connect'`/`accept'` need no host — they
dispatch on the addr/listener opaque `type_path`.

## Files touched

- `src/comms/process.rs` — `sender_receiver_from_fd` helper; `socket_pair` refactored onto it.
- `src/kernel/spawn.rs` — `SOCKET_LISTENER_TYPE_PATH` + `SOCKET_ADDRESS_TYPE_PATH` consts.
- `src/runtime.rs` — `eval_listener_prime` grows `env,sym` + the `(process)` arm; call site
  `:4562`; `eval_connect_prime`/`eval_accept_prime` gain a `(process)` opaque arm.
- `src/check.rs` — `infer_listener_prime`/`infer_connect_prime`/`infer_accept_prime` gain the
  process branch; `socket_listener_tuple` helper.
- `tests/nursery/probe_arc209_c0b2c_process_connection.rs` — the gate (committed RED first).

## The gate (RED at HEAD → GREEN on ship)

A single-process, single-thread round-trip mirroring the UDS spike through the wat verbs:
`(listener' (process) :i64 :i64)` → `(connect' addr)` (client `SocketPeer'`, connection queued) →
`(accept' listener)` (server `SocketPeer'`) → `(send' client 5)` → `(recv' server)` = 5 →
`(send' server 15)` → `(recv' client)` = 15. RED at HEAD: `(listener' (process) …)` is a C0b.2a
check error, so `startup_from_source` fails. GREEN once C0b.2c ships. Deadlock-free: connect queues
before accept dequeues; sends fit the socket buffer; no thread join.

## Out of scope = rejected (named, not deferred)

- **`select'`-3arg process arm** (the autoscaling `comms::process::Select` over a `SocketListener'`
  + client `SocketPeer'`s) — **C0b.3a**. C0b.2c proves the verbs; the service-loop multiplexer is
  the next rung.
- **`SO_PEERCRED` allow-set** — **C0b.3b** (security model LOCKED in `DESIGN-STONE-C0b-SECURITY.md`).
- **Well-known / caller-provided names** (a client process discovering a service by a stable name) —
  a **Stone C / defservice** concern. C0b.2c mints-and-returns the name to mirror thread-tier
  surface parity; cross-process name discovery is genuinely unresolved and is NOT built now
  ([[feedback_dont_build_the_forcing_function]]).
- **`(remote)` AF_INET / mTLS** — guaranteed by the AF_UNIX clause (`s/AF_UNIX/AF_INET/`); built
  when `:remote` arrives.

## The deadlock contract carries

Per C0b.1b ([[feedback_vended_primitives_never_deadlock]]): `accept'` blocking until a connection is
the honest wire-wait, not a deadlock. The deadlock-free-**on-drop** service loop is C0b.3a's concern
(the `select'` multiplexer terminating on owner-drop); C0b.2c ships the verbs the loop will call.
