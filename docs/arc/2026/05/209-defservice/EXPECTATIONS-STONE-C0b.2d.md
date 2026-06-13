# EXPECTATIONS — Stone C0b.2d (written before the strike)

The independent scorecard. The Inquisitor re-runs every row against its own build before crediting.

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | named cross-process round-trip | `cargo test --release -p wat --test probe_arc209_c0b2d_named_cross_process` | `1 passed` (5→105 across the boundary, rendezvoused by name) |
| 2 | c0b2c updated to named, still GREEN | `cargo test --release -p wat --test nursery probe_arc209_c0b2c -- --test-threads=1` | `1 passed` (reply == 15, now via `socket-address'`) |
| 3 | c0b2b socket peer intact | `cargo test --release -p wat --test nursery probe_arc209_c0b2b_socket_peer -- --test-threads=1` | `1 passed` |
| 4 | c0b1b select multiplexer intact | `cargo test --release -p wat --test nursery probe_arc209_c0b1b_select_listener -- --test-threads=1` | `1 passed` |
| 5 | self-peer (C0b.3a-0) intact | `cargo test --release -p wat --test probe_arc209_c0b3a0_self_peer` | `1 passed` |
| 6 | listener-arm (C0b.3a-i) intact | `cargo test --release -p wat --test comms select_listener_arm_fires_on_pending_connection -- --test-threads=1` | `1 passed` |
| 7 | full nursery, no NEW reds | `cargo test --release -p wat --test nursery -- --test-threads=1` | `895 passed / 4 failed` (the 4 known baseline reds only) |
| 8 | full workspace test surface compiles | `cargo test --release --workspace --no-run` | clean |

## Runtime prediction

8–16 min. `socket-address'` mirrors `socket-pair'`; the `listener'` process arm goes from mint to
bind-addr (smaller); `connect'`/`accept'` untouched. The retire is a deletion, not a rewrite.

## Trap-doors named

- **Per-tier arity on `listener'`.** Thread = 3 args `(host :S :R)`; process = 2 args `(host addr)`.
  The arity guard must branch AFTER the host dispatch. Confirm both forms parse + a wrong arity per
  tier is a clean error (re-read `eval_listener_prime`/`infer_listener_prime` in the diff).
- **`socket_listener_tuple` retirement.** The process `listener'` now returns a single
  `SocketListener'`, not the `(SocketListener', SocketAddress')` tuple. If `socket_listener_tuple`
  becomes unused, remove it (dead code) — but grep first (it may be referenced elsewhere).
- **The startup race is handled by the self-peer ready-signal, NOT a sleep.** The gate's service
  `send'`s `1` on its `(self-peer)` after binding; the parent `recv' svc` before `connect'`. If the
  parent connected before the bind, abstract-UDS `connect_addr` would `ECONNREFUSED`. Confirm the
  gate passes deterministically (the handshake closes the race).
- **`Value::String` accessor** (STOP-1) — confirm the actual variant/deref for a wat String value.
- **c0b2c is now same-process *named*** (not minted) — verify it still round-trips 15.

## Out of scope (must NOT appear in the diff)

The `select'`-3arg process branch; any `SO_PEERCRED`; any `connect'`/`accept'` signature change; any
thread-tier `listener'` change; a `connect'` retry/poll on ECONNREFUSED (the ready-signal is the
rendezvous).
