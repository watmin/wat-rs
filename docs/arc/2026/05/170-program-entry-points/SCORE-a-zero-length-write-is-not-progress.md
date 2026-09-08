# SCORE — a zero-length write is not progress

**SCORED.** Executor: grok, 2026-09-08. Did not commit.

```
Summary [ 502.792s] 5235 tests run: 5235 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T19-28-00Z/`

No tests added. **5235**. 0 FAIL, 0 TIMEOUT.

## WHAT LANDED

The no-form rung, not a check. `WriteWait::Wrote` carries a `NonZeroUsize`.
A literal `Wrote(0)` does not compile. `send` adds `n.get()` — it cannot add
zero because the type has no zero.

`write_once`'s three-way split now **decides** `n == 0`:

```
n > 0   → Wrote(NonZeroUsize)
n < 0   → Errno(-n)
n == 0  → Err("io_uring Write returned 0 for a non-empty buffer — zero is not progress")
```

That `Err(String)` is the existing arm. `send` maps it to the existing
`SendError::Failed(value, reason)`. No new variant. No retry, budget, or
backoff. A leftover Write CQE of 0 after cancel is still `Shutdown` — the
write delivered nothing; that is not this defect.

`try_send` is untouched. Its short-write branch already refuses to loop
("never torn frames on the wire") and is the reasoning this stone adopted.

## THE LOAD-BEARING ARMS

| probe | isolated | floor |
|---|---|---|
| `partial_frame_residue` | **3.023 s** PASS | **3.038 s** PASS |
| `send_poll_arm` | 0.017 s PASS | **0.022 s** PASS |
| `the_senders_tie_break_is_a_property` | 0.017 s PASS | **0.021 s** PASS |
| `a_signal_is_an_fd` | 0.020 s PASS | **0.032 s** PASS |

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ `Wrote(0)` cannot be constructed | ✅ `Wrote(NonZeroUsize)` |
| 2 | ★ `n == 0` is reported, not looped | ✅ `SendError::Failed` with the zero-length reason |
| 3 | `send` cannot add zero | ✅ `written += n.get()` |
| 4 | contract unchanged | ✅ no new variant; existing Failed strings untouched |
| 5 | `try_send` untouched | ✅ not in the diff |
| 6 | transport still works | ✅ residue **3.038 s** |
| 7 | blast | ✅ `src/comms/process.rs` only |
| 8 | the floor | ✅ `Summary [ 502.792s] 5235 tests run: 5235 passed (7 slow), 22 skipped` |

## BLAST

```
 src/comms/process.rs | 23 +++++++++++++++++------
```

`WriteWait`, `write_once`'s three-way split, `send`'s match arm. Compiler
named the leftover `Wrote(result as usize)` site; that one already had
`result > 0` and now wraps `NonZeroUsize` the same way.
