# SCORE — STONE 255.42: drive the re-poll path from Rust

Struck against draw `d84120354`. The text-count guard is gone. A Rust test
drives the real process `poll` re-poll.

## Why a seam

A pending connection is what makes the listen fd readable, and it is still
in the backlog when `accept` runs. `accept` then returns the connection.
It does not return `EAGAIN`. Closing the connector before `accept` still
yields a socket, not `WouldBlock`. There is no gap in `poll_process_tier`
where the test can empty that backlog after the first select and before
`accept`.

`accept_for_poll` (`src/kernel/message.rs` 1415–1430) is that one event.
Under `cfg(test)`, one armed hook runs and the function returns
`WouldBlock`. Otherwise it is `UnixListener::accept`. The release binary
is not built with `cfg(test)`, so the hook is not in it. Nothing in the
unarmed test build changes an accept.

The hook is also when the owner acts. That is the only instant between the
first select observing the listener and the re-poll's `select_raw`. The
re-poll itself is the production second select: a real lineage pipe, a
real quiet client, a real non-blocking `UnixListener` with one pending
connection.

## The rows

| row | result |
|---|---|
| owner sends `42` during the spurious accept | `ServiceEvent.Admin`, field `42` |
| owner drops the lineage during the spurious accept | `ServiceEvent.Shutdown`, no fields |

With the re-poll arm put back to the pre-255.41 body (index 0 builds
`Shutdown` and ignores `res2`), the admin row failed:

```
assertion `left == right` failed
  left: "Shutdown"
 right: "Admin"
```

That arm was restored. The same two tests then passed 21 invocations in a
row, each invocation repeating each situation 20 times. All identical.

`tests/kernel/probe_arc255_41_both_poll_arms_call_the_lineage_helper.rs`
is deleted. The helper's own `Admin` / `Shutdown` unit test stays.

## For the sns-sqs merge

`src/kernel/message.rs`:

| what | lines |
|---|---|
| `accept_for_poll` | 1415–1430 |
| `poll_process_tier` (the process `poll` loop, moved out of `eval_poll_prime`) | 1443–1817 |
| `eval_poll_prime` calls it | 2185 |
| `drive_the_repoll` | the test module at the end of the file |

## Gates

Clippy `--all-targets --workspace -- -D warnings` exited 0.

Census `.census/2026-09-25T22-40-06Z.txt` against
`.census/2026-09-25T22-34-29Z.txt`: `census-diff: no STOP-8`. 0 rc flips.
2281 files, 215 nonzero.

Delta `.delta/2026-09-25T22-40-54Z`: NEW 2 / RECOVERY 0. The two NEW files
are `wat-scripts/probes/arc-170/probe-c1-clean-surface.wat` and
`wat/holon/Ngram.wat`.

Ledger 198. `the_heresy_ledger_matches_its_frozen_census` passed.

`.floor/2026-09-25T22-42-12Z`: `6136 tests run: 6136 passed (11 slow), 22 skipped`.
