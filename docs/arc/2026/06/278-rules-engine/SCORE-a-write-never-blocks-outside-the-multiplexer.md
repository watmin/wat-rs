# SCORE — a write never blocks outside the multiplexer

**SCORED.** Executor: grok, 2026-09-08. Tree dirty: this stone's two
Rust files, plus the uncommitted `circuit.wat` from
`a-reconnect-is-not-an-abandonment` (not this stone; not reverted,
not committed, not touched).

```
Summary [ 497.708s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T04-48-13Z/`

## WHAT LANDED

`src/io.rs` + `src/comms/process.rs`. Toggle, not construction.

`O_NONBLOCK` is armed for the duration of a **polled** write via a
`NonblockGuard` (Drop restores the original `fcntl` flags). The
no-broadcast fallback stays blocking. `EAGAIN`/`EWOULDBLOCK` continues
to poll; it is not an error and not a blind retry. Short counts still
resume (`written += n` / `write_all`'s remaining slice).

**Why toggle, not construction.** `PipeWriter` also wraps regular files
(`IOWriter/open-file`) and dups of fd 1/2 that share a file description
with the process stdio. Construction-time `O_NONBLOCK` would leak onto
those. `Sender::try_send` independently toggles the same write fd and
restores; construction-time would fight that restore. Single-writer is
still the claim on `Sender`.

The two false comments are gone (`NOT THE BUG`, `deliberately
unresolved`). Tie-break ("writable wins ties") is untouched.

No `sigaction`. No `io_uring` on the write path. `send`'s typed
outcomes unchanged. `wat-scripts/` not touched.

## THE GATE

Isolated (after rebuild):

| probe | time | result |
|---|---|---|
| `probe_arc278_partial_frame_residue` | **3.019 s** | PASS (was 20 s hang) |
| `probe_arc278_send_poll_arm` | 0.017 s | PASS |

Same floor:

```
PASS [   0.022s] probe_arc278_send_poll_arm
PASS [   3.031s] probe_arc278_partial_frame_residue
```

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ the RED goes green | ✅ 3.019 s isolated; 3.031 s on the floor. ≪ 20 s |
| 2 | sibling still passes | ✅ 17 ms isolated; 22 ms on the floor |
| 3 | no blocking write after a poll | ✅ `O_NONBLOCK` in force for the polled path |
| 4 | short counts resume | ✅ `WouldBlock` → poll; `written += n` / `write_all` loop |
| 5 | flags restored on every path | ✅ `NonblockGuard` Drop: success, Shutdown, Disconnected, Failed, WriteStopped |
| 6 | false comments gone | ✅ `grep` of `NOT THE BUG\|deliberately unresolved` in `src/` = no hits |
| 7 | circuit unaffected | ✅ no-args `total=8000;distinct=8000;dup=0;seen-recorded=8000` |
| 8 | chaos unchanged | ✅ 38 PASS lines with `drop` |
| 9 | the floor | ✅ `Summary [ 497.708s] 5221 tests run: 5221 passed (7 slow), 22 skipped` |

STOP-1 did not fire: toggle is scoped to the polled write; construction
was rejected because of stdio dups and `try_send`. STOP-2 / STOP-3 /
STOP-5 / STOP-6 / STOP-7 held. Did not re-run the floor.

0 FAIL, 0 `^error`, **38 drop tests**, `every_wat_scripts_file_loads` PASS.

## BLAST

```
 src/comms/process.rs |  79 +++++++++++++++++++++-----
 src/io.rs            |  81 +++++++++++++++++++++++----
```

`wat-scripts/fanout/circuit.wat` is dirty from the previous stone and
was not part of this diff's intent.

No `sigaction`. No `io_uring` write. No outcome-type change.
