# EXPECTATIONS — Stone C0b.2e-i-a (written before the strike)

Independent scorecard. The Inquisitor verifies each row by its own re-run before any
commit; the strike cannot move these goalposts.

| # | What | Command | Expected |
|---|------|---------|----------|
| 1 | The gate probe goes GREEN | `cargo test --release -p wat --test comms probe_arc209_c0b2eia_boxed_comm_both_tiers -- --test-threads=1` | `1 passed; 0 failed` |
| 2 | No comms regression | `cargo test --release -p wat --test comms -- --test-threads=1` | all pass (i-0 + slice3 + socket probes intact) |
| 3 | Nursery baseline holds | `cargo test --release -p wat --test nursery -- --test-threads=1` | `895 passed / 4 failed` (the 4 known: arc-255 reflection ×2 + undefined-builtin ×2 — ZERO new) |
| 4 | Full surface compiles | `cargo test --release --workspace --no-run` | clean (additive trait change is a recompile cascade) |
| 5 | `ReactorClass` is a NAMED enum, not `Option<RawFd>` | read `src/comms/mod.rs` | `pub enum ReactorClass { InMemory, Fd }`, payloadless `Fd` |
| 6 | `close` UNCHANGED | `git -C . diff -- src/comms/mod.rs` | no edit to either `close` signature (no `where Self: Sized` added) |
| 7 | `CommSender` UNCHANGED | `git diff` | no `CommSender` method/supertrait change |

## Runtime prediction

5–12 min. Three small additive edits + a recompile cascade (the cascade is the bulk of
wall-clock; comms is depended-on widely, so the workspace rebuild dominates).

## Trap-doors named

- **`Any` + `'static`:** if any `Receiver<T>` instantiation in a test uses a non-`'static`
  `T`, the `Any` supertrait would reject it. Grounded expectation: the wire `T`s are all
  `'static` (Send/EdnRepresentable). If it fires, it is a real STOP-2, not a fixup.
- **`reactor_class` naming drift:** the strike must not "improve" it into `poll_fd ->
  Option` or carry the fd in `Fd` — that re-introduces the smell the four-questions
  rejected. Row 5 guards this.
- **Scope creep into i-b:** any `Peer` struct change, runtime/checker edit, or `select'`
  touch is out of scope — would show in `git diff --stat` beyond the three comms files +
  the probe. The Inquisitor checks the diff stat.

## Honest-delta slots (filled at SCORE time)

- Did object-safety actually hold with the `Any` supertrait added (as the probe
  predicted), or did `Any` surface anything? —
- Any baseline drift in rows 2–4? —
