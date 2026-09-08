# EXPECTATIONS — a zero-length write is not progress

Written **before** the strike. Small stone; `src/comms/process.rs` only.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ★ **`Wrote(0)` cannot be constructed** | compile | `WriteWait::Wrote` carries a `NonZeroUsize`; a literal `Wrote(0)` fails to compile |
| 2 | ★ **`n == 0` is reported, not looped** | read the diff | it maps to `SendError::Failed` with a reason naming the zero-length write |
| 3 | **`send` cannot add zero to `written`** | read the diff | `written += n.get()`, with no path that adds `0` |
| 4 | **the contract is unchanged** | read the diff | no new `SendError` variant; `Disconnected` / `Shutdown` / existing `Failed` strings untouched |
| 5 | **`try_send` is untouched** | `git diff` | it was already correct |
| 6 | **the transport still works** | `probe_arc278_partial_frame_residue`, `send_poll_arm`, `the_senders_tie_break_is_a_property`, `a_signal_is_an_fd` | PASS; residue in **~3 s** — read the duration |
| 7 | **blast radius** | `git diff --stat` | `src/comms/process.rs` only |
| 8 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5235 + any tests you add (state the number), 22 skipped, **0 FAIL, 0 TIMEOUT**, quiet box |

## The rows that carry it

★ **Row 1 is the stone.** A `debug_assert!` would catch this; a type makes it unrepresentable. The
distinction matters because the previous two stones both taught the same lesson — the drain classifier
survived review by being *unreachable from a test*, and the reader's STOP-1 only became real when it
turned into a `debug_assert!`. Here the material allows the top rung, so take it.

⚠ **Row 2 is why row 1 is not enough on its own.** Making `Wrote(0)` unrepresentable forces
`write_once` to decide *something* about `n == 0`; the decision must be a **report**, not a silent
`continue` or an `unreachable!()` that panics the transport.

## Runtime prediction

**20–35 minutes.** One enum field, one branch, one match arm, and the compiler names every site.

## Trap-doors

- **`NonZeroUsize::new(n)` returns an `Option`** — the `None` case is exactly the `n == 0` decision;
  do not `unwrap` it.
- **`unreachable!()` is not the answer.** POSIX not defining an outcome is not a guarantee the kernel
  never produces one; a panic on the transport's hot path is worse than a typed error.
- **`n` is `i32` from the CQE** — the `> 0` / `< 0` / `== 0` split already exists; keep it explicit.
- **Do not fold the `Shutdown` arm into this change.**

## What this stone does NOT claim

⚠ It does **not** claim the kernel ever returns 0 here. It removes a state whose only honest outcome is
a report, and makes the wrong one unrepresentable.
⚠ It does **not** touch `try_send`, the ring, the mask, or the demux.
