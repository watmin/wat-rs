# EXPECTATIONS — Stone C0b.2e-iii (written before the strike)

Independent scorecard. The Inquisitor verifies each row by its own re-run before any commit.
A proper-entity refactor (mirrors ii) — the disconfirm is the grep, the proof is the existing
connect flow via the real `Address` entity.

| # | What | Command | Expected |
|---|------|---------|----------|
| 1 | `SocketAddress'` family GONE | `grep -rn "SOCKET_ADDRESS_TYPE_PATH\|wat::kernel::SocketAddress'" src/` | **no matches** |
| 2 | `Address` is now a real entity | `grep -rn "ADDRESS_TYPE_PATH" src/kernel/spawn.rs` + read `src/kernel/address.rs` | `ADDRESS_TYPE_PATH = ":wat::kernel::Address'"`; `CommAddress` trait + `Address{Box<dyn CommAddress>}` + `ThreadAddress`/`SocketAddress` impls |
| 3 | thread Address' no longer a raw `Sender` | read `eval_listener_prime` thread arm | the Address' tuple slot is an `Address` opaque (ADDRESS_TYPE_PATH), not a bare `Value::wat__kernel__Sender` |
| 4 | thread connect flow green | `cargo test --release -p wat --test nursery connection_primitive -- --test-threads=1` | `1 passed` |
| 5 | process same-process + cross-process + service loop green | `c0b2c` / `probe_arc209_c0b2d_named_cross_process` / `probe_arc209_c0b3aii_process_service_loop` (each) | each `1 passed` (via the real `Address`) |
| 6 | Nursery baseline holds | `cargo test --release -p wat --test nursery -- --test-threads=1` | `895 passed / 4 failed` (4 known — ZERO new) |
| 7 | Full surface compiles | `cargo test --release --workspace --no-run` | clean |
| 8 | `connect'` is one arm | read `eval_connect_prime` | one arm: downcast `Address` → `inner.connect(...)`; the two inline connect bodies moved into the impls |
| 9 | `CommAddress` is `connect`-only | read `src/kernel/address.rs` | just `connect`; NO `reactor_class`/`as_any`/`Option` (an address is dialed, never poll'd); NOT a closed `:Thread/:Process/:Remote` sum |

## Runtime prediction

10–18 min. New file + a trait + two impls (connect bodies moved verbatim) + the
socket-address'/connect'/listener'-slot rewire + the checker `SocketAddress'`→`Address'` +
a recompile cascade. Mirrors ii (which ran clean).

## Trap-doors named

- **Layering:** `kernel::address` → `comms` + `kernel::Peer` + `channel` (typed_send) must
  compile clean (kernel→comms/channel). If it cycles, STOP-2.
- **`connect` context:** the moved bodies need `sym`/`span`; if they need `env` or more, STOP-1.
- **Closed-sum drift:** must be the OPEN `CommAddress` trait, NOT a `:Thread/:Process/:Remote`
  sum (the stale framing) — row 9 guards this.
- **Scope creep:** any SO_PEERCRED (= C0b.3b), `poll'`/`Listener`/`Peer`/`comms` change is OUT —
  `git diff --stat` confined to address.rs + mod.rs + spawn.rs + runtime.rs + check.rs.

## Honest-delta slots (filled at SCORE time)

- Did the two connect bodies move verbatim, or did the trait-method shape force changes? —
- Any baseline drift in rows 4–7? Diff stat? —
- After iii: is the connection surface (Peer + Listener + Address) fully unified + first-class? —
