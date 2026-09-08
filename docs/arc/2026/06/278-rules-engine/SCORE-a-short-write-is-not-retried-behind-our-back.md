# SCORE — a short write is not retried behind our back

**SCORED.** Executor: grok, 2026-09-08. Measurement only. The table is
the deliverable. No production code.

```
Summary [ 493.722s] 5224 tests run: 5224 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T07-31-21Z/`

## THE TABLE

ROOM=4096, POLLOUT set, fresh pipe per trial, `filled=65536` every row
(this box's capacity). Isolated 81.8 ms:

| payload | filled | POLLOUT | peek Write | delivered | cancel | write final | drain |
|---|---|---|---|---|---|---|---|
| 8192 | 65536 | `revents=0x4` SET | **ok n=4096** | 4096 | ENOENT(2) | ok n=4096 | empty |
| 16384 | 65536 | `revents=0x4` SET | **ok n=4096** | 4096 | ENOENT(2) | ok n=4096 | empty |
| 65536 | 65536 | `revents=0x4` SET | **ok n=4096** | 4096 | ENOENT(2) | ok n=4096 | empty |
| 131072 | 65536 | `revents=0x4` SET | **ok n=4096** | 4096 | ENOENT(2) | ok n=4096 | empty |

Control (8192) reproduces v2: CQE at peek, `ok n=4096`. All four ran.
None skipped, none merged.

Floor: `PASS [ 0.089s] probe_short_write_retry_sweep`.

## Which sizes, if any, parked while holding bytes

**None of them.** At every swept size the Write CQE arrived at the 20 ms
peek with `n=4096` — a faithful short write of one slot — and cancel
was `ENOENT` because the request was already complete. No row showed
"no CQE + FIONREAD grew by 4096". The re-issue-and-park shape was not
observed on this box at 8192, 16384, 65536, or 131072.

That is this sweep, not a claim that re-issue never exists at other
sizes or kernels.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ every swept size has a row | ✅ 8192, 16384, 65536, 131072 |
| 2 | ★ peek Write CQE named per row | ✅ all four `ok n=4096` |
| 3 | control reproduces | ✅ 8192: peek `ok n=4096` |
| 4 | writer slot per trial | ✅ POLLOUT SET, `revents=0x4`, every row |
| 5 | delivery measured per trial | ✅ 61440 → 65536, growth 4096, every row |
| 6 | cancel where parked | ✅ all completed before cancel; ENOENT + write_final `ok n=4096` |
| 7 | fresh pipe per trial | ✅ `filled=65536` printed four times |
| 8 | ring clean per trial | ✅ final drain empty (PollAdd woken) |
| 9 | terminates | ✅ 81.8 ms isolated; 89 ms on the floor. Bound 2 s |
| 10 | measurement only | ✅ `git diff --stat -- src/` empty |
| 11 | the floor | ✅ `Summary [ 493.722s] 5224 tests run: 5224 passed (7 slow), 22 skipped` |

Did not stop the sweep early. Did not migrate. Did not touch `src/`.

## BLAST

```
?? tests/comms/probe_arc278_short_write_retry_sweep.rs
```

No `src/` changes.
