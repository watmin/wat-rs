# EXPECTATIONS — Stone C0b.2e-ii (written before the strike)

Independent scorecard. The Inquisitor verifies each row by its own re-run before any
commit. A proper-entity refactor — the disconfirm is the grep, the proof is the existing
listener flow via the real `Listener` entity.

| # | What | Command | Expected |
|---|------|---------|----------|
| 1 | `SocketListener'` family GONE | `grep -rn "SOCKET_LISTENER_TYPE_PATH\|wat::kernel::SocketListener'" src/` | **no matches** |
| 2 | `Listener` is now a real entity | `grep -rn "LISTENER_TYPE_PATH" src/kernel/spawn.rs` + read `src/kernel/listener.rs` | `LISTENER_TYPE_PATH = ":wat::kernel::Listener'"`; `CommListener` trait + `Listener{Box<dyn CommListener>}` + 2 impls |
| 3 | thread listener is no longer a raw `Receiver` | read `eval_listener_prime` thread arm | the listener tuple slot is a `Listener` opaque (LISTENER_TYPE_PATH), not a bare `Value::wat__kernel__Receiver` |
| 4 | thread connect flow green | `cargo test --release -p wat --test nursery connection_primitive -- --test-threads=1` | `1 passed` |
| 5 | process listener'/accept'/connect' green | `cargo test --release -p wat --test nursery probe_arc209_c0b2c -- --test-threads=1` then `probe_arc209_c0b2d` | each `1 passed` (via the real `Listener`) |
| 6 | Nursery baseline holds | `cargo test --release -p wat --test nursery -- --test-threads=1` | `895 passed / 4 failed` (4 known: arc-255 reflection ×2 + undefined-builtin ×2 — ZERO new) |
| 7 | Full surface compiles | `cargo test --release --workspace --no-run` | clean |
| 8 | `accept'` is one arm | read `eval_accept_prime` | one arm: downcast `Listener` → `inner.accept(...)`; the two inline accept bodies moved into the impls |
| 9 | `CommListener` is `accept`-only | read `src/kernel/listener.rs` | NO `reactor_class`/`listen_fd` (those are C0b.3a-ii); just `accept` |

## Runtime prediction

12–22 min. New file + a trait + two impls (accept bodies moved verbatim) + the
listener'/accept' rewire + the checker `SocketListener'`→`Listener'` + a recompile cascade.

## Trap-doors named

- **Layering:** `kernel::listener` → `comms` + `kernel::Peer` must compile clean (kernel→comms).
  If it cycles, STOP-3.
- **`accept` context:** the moved bodies need `sym`/`span`; if they need `env` or more, STOP-1.
- **The socket poll-accept loop** must move verbatim into `SocketListener::accept` (it uses
  `process::Select::listener` which exists). If it can't, STOP-2.
- **Scope creep:** any `Address'`/`SocketAddress'` change (= iii), any poll'-loop listener
  integration or `reactor_class`/`listen_fd` (= C0b.3a-ii), any `comms` edit is OUT — would
  show in `git diff --stat` beyond the five files.

## Honest-delta slots (filled at SCORE time)

- Did the two accept bodies move verbatim, or did the trait-method shape force changes? —
- Any baseline drift in rows 4–7? —
- Diff stat (files + line counts)? —
