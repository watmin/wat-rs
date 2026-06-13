# DESIGN-STONE C0b.2d — connect-by-name (well-known names)

> The cross-process rendezvous. C0b.2c's `listener'` (process) MINTS a unique name and returns a
> `SocketAddress'` **opaque** — which cannot cross a process boundary (opaques aren't EDN). So a
> separately-spawned client has no way to learn where to `connect'`. This blocked the C0b.3a-ii
> service-loop probe. The fix: both parties rendezvous by a shared **name** (a `String` literal) —
> the model already says abstract UDS names are PUBLIC. After this, the clean cross-process
> service-loop probe (C0b.3a-ii) becomes possible.

## Where we are (grounded, read this session)

- C0b.2c (`runtime.rs:17986` eval / `check.rs:9941` infer): `(listener' (process) :S :R)` →
  `Tuple[SocketListener'<S,R>, SocketAddress'<S,R>]` — mints a unique abstract name
  (`LISTENER_SEQ` counter), returns the listener + a `SocketAddress'` opaque wrapping the name.
  `(connect' addr)` accepts a `SocketAddress'` opaque (`runtime.rs:18099`) → `connect_addr(name)` →
  `SocketPeer'`. `accept'` unchanged. `SOCKET_ADDRESS_TYPE_PATH` (`spawn.rs:150`).
- The opaque is an in-process value; the NAME inside it is a `String` (serializable, shareable as a
  literal). Two processes that name the SAME string rendezvous on the same abstract socket.
- The self-peer (C0b.3a-0) lets a spawned service signal "ready" to its owner over fd1 — the
  race-free way for an owner/client to wait until the service has bound before connecting.

## The contract decision (LOCKED via four-questions)

**Add `(:wat::kernel::socket-address' name :S :R) -> SocketAddress'<S,R>` — construct a typed address
from a shared name. `listener'` (process) BINDS a given address; `connect'`/`accept'` are
UNCHANGED.** The same-process mint-and-return is RETIRED (it was the C0b.2c stepping-stone; the
process tier exists FOR cross-process, which is named).

| verb | shape |
|---|---|
| `socket-address'` (NEW) | `(socket-address' name :S :R) -> SocketAddress'<S,R>` — `name` a `String` value; `:S :R` checker-only |
| `listener'` (process) | `(listener' (process) addr) -> SocketListener'<S,R>` — bind `addr`'s name (was: `(… :S :R)` mint) |
| `connect'` | `(connect' addr) -> SocketPeer'<S,R>` — **UNCHANGED** (addr now from `socket-address'`) |
| `accept'` | `(accept' listener) -> SocketPeer'<R,S>` — **UNCHANGED** |
| `listener'` (thread) | `(listener' (thread) :S :R) -> Tuple[Listener'<S,R>, Address'<S,R>]` — **UNCHANGED** (mints in-memory) |

**Four questions — candidate (2) `socket-address'` constructor vs (1) named verbs
(`connect'`/`listener'` take a name + types directly):**
- **Obvious — both YES.** "construct the address for a name; bind it; dial it" reads cleanly.
- **Simple — (2) wins.** (2) keeps `connect'`/`accept'` uniform (1-arg `addr`, types carried by the
  addr); only `listener'` diverges (binds an addr). (1) makes BOTH `connect'` and `listener'`
  arity-polymorphic per tier (process `(connect' (process) name :S :R)` 4-arg vs thread `(connect'
  addr)` 1-arg). One arity-divergence (2) beats two (1).
- **Honest — (2) YES.** The NAME (a literal) is the shareable rendezvous; the `SocketAddress'` is the
  local typed handle — no pretense the opaque crosses. (2) reuses C0b.2c's `connect'(SocketAddress')`
  arm rather than churning it.
- **Good UX — (2) YES.** defservice: service `(listener' (process) (socket-address' NAME :S :R))`,
  client `(connect' (socket-address' NAME :S :R))` — both name the same string. Symmetric.

**Sub-decision — retire the mint-and-return: YES.** Two `listener'` (process) forms (mint vs
bind-addr) = optionality smell; the process tier is for cross-process = named; anonymous same-process
process-sockets have no real use (the thread tier covers same-memory). The mint was a stepping-stone;
`socket-address'(name)` + `listener'(addr)` supersedes it. (c0b2c probe updates in-strike.)

## The mechanism

- **`socket-address'`** (`runtime.rs`, beside `eval_socket_pair_prime`): eval `args[0]` → `String`
  name; validate `:S :R` keywords; `make_rust_opaque(SOCKET_ADDRESS_TYPE_PATH, name)`. (Same opaque
  C0b.2c's `connect'` already consumes.) Check (`check.rs`, beside `infer_socket_pair_prime`):
  `(name: String, :S, :R)` → `SocketAddress'<S,R>`. Dispatch entries beside `socket-pair'`.
- **`listener'` (process) binds a given addr** (`eval_listener_prime`): per-tier arity — ThreadOpts →
  3 args `(host :S :R)` (mint, unchanged); ProcessOpts → 2 args `(host addr)`: downcast `addr` →
  `&String` name, `bind_addr` + `set_nonblocking(true)` (C0b.3a-i invariant), return
  `make_rust_opaque(SOCKET_LISTENER_TYPE_PATH, listener)` (just the listener, no tuple). Retire the
  `LISTENER_SEQ` mint + the `SocketAddress'` return. `infer_listener_prime`: ProcessOpts arm → expect
  `(host, addr: SocketAddress'<S,R>)` → `SocketListener'<S,R>` (retire `socket_listener_tuple`'s
  Address' element / the tuple result for process).
- **`connect'` / `accept'`** — UNCHANGED (C0b.2c arms reused; `connect'` consumes the `SocketAddress'`
  from `socket-address'`).

## The gate (RED at HEAD → GREEN on ship) — the first TRUE cross-process connection

A spawned process service binds a known name; the PARENT (a separate process) connects by the same
name and round-trips. Race-free via the self-peer ready-signal (C0b.3a-0):
```
(defn :user::compute [] -> :i64
  (let [svc (spawn-program' (process)
              (forms (defn :user::main [] -> nil
                (let [l    (listener' (process) (socket-address' "wat.arc209.c0b2d.svc" :i64 :i64))
                      _    (send' (:wat::program::self-peer :i64 :i64) 1)   ;; signal READY to owner (fd1)
                      cli  (accept' l)                                       ;; accept the parent
                      x    (recv' cli)
                      _    (send' cli (+ x 100))]
                  nil))))
        _   (recv' svc)                                                     ;; wait for READY (race-free)
        c   (connect' (socket-address' "wat.arc209.c0b2d.svc" :i64 :i64))  ;; parent dials by NAME
        _   (send' c 5)
        got (recv' c)]
    got))   ;; expect 105
```
RED at HEAD: `socket-address'` doesn't exist + `listener' (process)` mints (2-arg form unknown) →
check error. GREEN once C0b.2d ships. Proves: two processes rendezvous on a shared name, cross the
process boundary, round-trip. (No `select'` — that's C0b.3a-ii.) The ready-signal uses the shipped
self-peer; if 2d is the only new piece, a failure points at 2d.

## Files touched

- `src/runtime.rs` — `eval_socket_address_prime` + dispatch; `eval_listener_prime` process arm
  (bind given addr, retire mint); `eval_connect_prime`/`eval_accept_prime` UNCHANGED.
- `src/check.rs` — `infer_socket_address_prime` + dispatch; `infer_listener_prime` process arm
  (bind addr → `SocketListener'`, retire `socket_listener_tuple`).
- `tests/nursery/probe_arc209_c0b2c_process_connection.rs` — update to `socket-address'` +
  `listener'(process addr)` (same-process named; the supersession of the mint form).
- `tests/probe_arc209_c0b2d_named_cross_process.rs` — the new cross-process gate (top-level, forks).

## Out of scope = rejected (named, not deferred)

- **The `select'`-3arg process branch + service loop** — C0b.3a-ii (now unblocked by this).
- **`SO_PEERCRED`** — C0b.3b.
- **`(remote)` AF_INET** — `socket-address'` generalizes (name → host:port) when `:remote` arrives.
- **A `connect'` retry/poll on ECONNREFUSED** — REJECTED; the self-peer ready-signal is the race-free
  rendezvous (no sleep/poll guess — `mora`).

## The deadlock contract carries

The listen fd stays non-blocking (C0b.3a-i). The ready-signal is a real wire event (self-peer
send/recv), not a timed wait. [[feedback_vended_primitives_never_deadlock]]
