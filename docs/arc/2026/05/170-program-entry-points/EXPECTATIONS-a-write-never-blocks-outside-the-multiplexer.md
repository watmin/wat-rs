# EXPECTATIONS — a write never blocks outside the multiplexer

Written **before** the strike. The probe that found the bug is the gate.

## Rows

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ **the RED goes green** | `cargo nextest run --release probe_arc278_partial_frame_residue` | **PASS**, and in ≪ 20 s |
| 2 | the sibling still passes | `… probe_arc278_send_poll_arm` | PASS — the full-pipe path is not regressed |
| 3 | no blocking write after a poll | read both sites | `O_NONBLOCK` in force, or the write capped to what poll promised |
| 4 | short counts resume | read both loops | `EWOULDBLOCK`/`EAGAIN` loops back to **poll**, not to a retry or an error |
| 5 | flags restored on every path | read the exits | success, `Shutdown`, `Disconnected`, `Failed` — all restore |
| 6 | the false comments are gone | `grep -n 'NOT THE BUG\|deliberately unresolved' src/` | **no hits** |
| 7 | the circuit is unaffected | `… circuit.wat` no-args | every field identical |
| 8 | chaos unchanged | `scripts/floor.sh`, count `drop` | **38 drop tests pass** |
| 9 | the floor | `scripts/floor.sh` | **read the Summary line**: **5221 passed**, 22 skipped |

⚠ **Row 1 is the stone.** A green floor with that probe still failing is not a pass — STOP-5.

⚠ **Row 2 is the regression guard.** The full-pipe path works today; the fix must not trade one
state for the other.

⚠ **Row 5 is the one that fails silently.** A leaked `O_NONBLOCK` on a shared fd turns every later
blocking reader into a spin. It will not show in these rows — read the exits.

## Runtime prediction

**60–90 minutes.** Two files, one pattern, plus a Rust rebuild. The floor is the long pole and the
rebuild adds to it.

## Trap-doors named in advance

- **`io.rs`'s write is used by more than pipes.** Check every caller before assuming the fd is a
  pipe; `O_NONBLOCK` on a regular file behaves differently.
- **`try_send` toggles the same fd.** Two paths toggling `O_NONBLOCK` on one fd must not race; the
  file's own comment claims single-writer, so **verify that still holds**.
- **A `write` returning `0`** on a non-blocking pipe is not `EAGAIN` — `io.rs` already treats it as a
  named error (`"pipe write returned 0 bytes"`). Keep that.
- **Do not "fix" the tie-break.** "Writable wins ties" is correct and deliberate; the bug is what
  happens *after* the break, not the break itself.

## What this stone does NOT claim

⚠ **It does not establish whether today's harness work made the bug reachable.** One RED, not
re-run. That needs the probe in isolation, repeatedly, on a quiet box.

⚠ **It does not remove the pattern** — `src/io.rs` still has zero `io_uring` and the write path is
still `libc`. That is stone 3.

⚠ **It does not touch signal handling** — stone 2.
