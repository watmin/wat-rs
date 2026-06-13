# EXPECTATIONS — Stone C0b.2e-i-b (written before the strike)

Independent scorecard. The Inquisitor verifies each row by its own re-run before any
commit; the strike cannot move these goalposts. This is a structural collapse — the
disconfirm is the grep, the proof is the existing arc-209 probes via the unified `Peer`.

| # | What | Command | Expected |
|---|------|---------|----------|
| 1 | `SocketPeer'` family is GONE | `grep -rn "SOCKET_PEER_TYPE_PATH\|wat::kernel::SocketPeer'\|struct SocketPeer\|SocketPeerCell" src/` | **no matches** (empty) |
| 2 | 1-arg `select'` over a crossbeam `Peer'` survives | `cargo test --release -p wat --test nursery connection_primitive -- --test-threads=1` | `1 passed` (the survival proof; via `as_any`) |
| 3 | Thread service 3-arg `select'` survives | `cargo test --release -p wat --test nursery probe_arc209_c0b1b -- --test-threads=1` | pass |
| 4 | Socket round-trip via unified `Peer` (Value wire) | `cargo test --release -p wat --test nursery probe_arc209_c0b2b -- --test-threads=1` | pass |
| 5 | Process connect + by-name via unified `Peer` | `cargo test --release -p wat --test nursery probe_arc209_c0b2c probe_arc209_c0b2d -- --test-threads=1` | pass |
| 6 | Self-peer via unified `Peer` | `cargo test --release -p wat --test nursery probe_arc209_c0b3a0 -- --test-threads=1` | pass |
| 7 | Nursery baseline holds | `cargo test --release -p wat --test nursery -- --test-threads=1` | `895 passed / 4 failed` (the 4 known: arc-255 reflection ×2 + undefined-builtin ×2 — ZERO new) |
| 8 | Full surface compiles | `cargo test --release --workspace --no-run` | clean (the type collapse is a recompile cascade) |
| 9 | One `Peer`, honest doc | read `src/kernel/peer.rs` | one boxed `Peer { Box<dyn CommSender<Value>>, Box<dyn CommReceiver<Value>> }`; doc says general transport-blind connection/self peer (NOT "pipes-only worker self-peer"); NO `SelfPeer` rename |
| 10 | `select'`/`poll'` unchanged | `git diff` | no `select'` rename, no `SelectEvent` rename, no `poll'` (that is the later split stone) |

## Runtime prediction

15–30 min. A wide uniform cascade (~70 sites, `SocketPeer'` → `Peer'`) + the boxed-struct
redefine + the two `as_any` `select'` arms; the workspace recompile dominates wall-clock.

## Trap-doors named

- **`as_any` None in a `select'` arm:** if the downcast to `&thread::Receiver<Value>` fails
  in an exercised test, a *socket* peer reached a connection `select'` arm — STOP-4, not a
  fixup (socket `select'` is C0b.3a-ii).
- **socket `Value` wire:** `socket_pair::<Value>()` / `sender_receiver_from_fd::<Value>`
  must type (i-0 payoff). If not, STOP-2.
- **Scope creep:** any `select'`→`poll'` / `SelectEvent`→`ServiceEvent` rename, any
  `Listener'`/`Address'` `Socket`-prefix drop, or any `comms/` edit is OUT of i-b — would
  show in `git diff --stat` beyond peer.rs/spawn.rs/runtime.rs/check.rs/process/verbs.rs.
- **Stale-doc carry-over:** the `Peer` doc must be *rewritten* honest, not left saying
  "worker self-peer" (row 9).

## Honest-delta slots (filled at SCORE time)

- Did every `SocketPeer'` site collapse cleanly to `Peer'`, or did any consumer resist? —
- Any baseline drift in rows 2–8? —
- Final diff stat (files + line counts)? —
