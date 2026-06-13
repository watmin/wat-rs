# EXPECTATIONS — Stone C0b.3a-i (written before the strike)

Gate-shape note: a **primitive addition** (`Select::listener` + `SelectOutcome::Listener`), so there
is NO pre-committed wat RED probe — the disconfirming fact is the grep-absent API (verified at HEAD).
The reactor unit test ships with the impl; the c0b2c regression is the end-to-end guard. The
Inquisitor re-runs every row against its own build before crediting.

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | reactor listener-arm fires on a pending connection | `cargo test --release -p wat --test comms select_listener_arm_fires_on_pending_connection -- --test-threads=1` | `1 passed` (`select()` → `SelectOutcome::Listener`) |
| 2 | standalone `accept'` still round-trips (now poll-driven) | `cargo test --release -p wat --test nursery probe_arc209_c0b2c -- --test-threads=1` | `1 passed` |
| 3 | c0b1b select multiplexer intact | `cargo test --release -p wat --test nursery probe_arc209_c0b1b_select_listener -- --test-threads=1` | `1 passed` |
| 4 | c0b2b socket peer intact | `cargo test --release -p wat --test nursery probe_arc209_c0b2b_socket_peer -- --test-threads=1` | `1 passed` |
| 5 | self-peer (C0b.3a-0) intact | `cargo test --release -p wat --test probe_arc209_c0b3a0_self_peer` | `1 passed` |
| 6 | comms group intact (the `SelectOutcome::Listener` ripple) | `cargo test --release -p wat --test comms -- --test-threads=1` | all pass |
| 7 | full nursery, no NEW reds | `cargo test --release -p wat --test nursery -- --test-threads=1` | `895 passed / 4 failed` (the 4 known baseline reds only) |
| 8 | FULL workspace test surface compiles (the ripple) | `cargo test --release --workspace --no-run` | clean |

## Runtime prediction

8–16 min. The listener arm is one more `PollAdd` in an established loop; the accept rework mirrors the
existing poll-then-act pattern; the ripple is mechanical (`Listener => unreachable!` arms the compiler
points at).

## Trap-doors named

- **The `SelectOutcome::Listener` ripple is the main risk.** Adding a variant to the shared enum
  breaks every `match` until each gains a `Listener` arm. Verify `cargo build --release` is clean AND
  `cargo test --workspace --no-run` compiles — a sampled build hides a test-binary that didn't get
  its arm.
- **CQE priority.** broadcast > data > listener. Confirm the drain returns `Listener` only when no
  data arm fired (so a busy service serves existing clients before accepting new — no accept-flood
  starvation). For standalone `accept'` (no receivers) this is moot.
- **Non-blocking listener + `EWOULDBLOCK` loop.** The accept' loop must re-`select()` on the SAME
  `Select` (ring reused) on `WouldBlock`, not rebuild per iteration. Confirm no busy-spin.
- **`PollAdd POLLIN` on a listen socket** must fire on a pending connection (STOP-1). The reactor
  unit test is the direct proof.

## Out of scope (must NOT appear in the diff)

The `select'`-3arg process branch; the service-loop probe; any `SO_PEERCRED`; any thread-tier behavior
change beyond the mechanical `Listener` exhaustiveness arms; any change to the C0b.3a-0 self-peer.
