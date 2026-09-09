# EXPECTATIONS — item (c) stone C

Written BEFORE the strike. Scored against my own re-run.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ **the failure reaches the caller** | span with a failing sink, driven past the logs cap by `log` | `LogResponse` carries the failure variant, **never `Ok`**. RED today; this is the stone |
| 2 | ★ **and the arriving item survives** | after that failed flush, flush again against a working sink | every log lands — the un-flushed batch AND the one that triggered the failure. **A count short of that is silent data loss, strictly worse than the bug being fixed** |
| 3 | same for metrics | `incr`/`timed` past the metrics cap with a failing sink | the matching failure variant on `IncrResponse`/`TimedResponse` |
| 4 | `Ok` still means accepted | a normal `log`, no size trigger | `Ok` |
| 5 | no `_` wildcard added | `git diff` on every match over a `*Response` | real arms only (STOP-2) |
| 6 | flush fns untouched | `git diff` on `flush-logs`/`flush-metrics` | empty (STOP-3) |
| 7 | no second success value | `grep -c 'Buffered' wat/telemetry.wat` | 0 (STOP-4) |
| 8 | pass-through vocabulary | the new variants | `Constraint`/`Transient`/`Fatal` carrying `:wat::query::` types, spelled as `CloseResponse` spells them — not a new taxonomy |
| 9 | no runtime change | `git diff --stat src/runtime.rs` | empty |
| 10 | stones A and B hold | `cargo nextest run --release -E 'test(probe_arc278_span)'` | all pass. **Assertions unedited** — a Record/enum growing may force mechanical construction edits, and that is fine; a changed assertion is not (this is stone B's Row 8 lesson, written correctly this time) |
| 11 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 5148+ run, 0 failed, FLOOR=0 |

**Runtime prediction:** 30–60 minutes. Three enum edits, three arms reading a tuple element they
already hold, and the checker-named cascade.

## Trap doors, named in advance

- **Dropping the arriving item** on the failure path. Turns a silent failure into silent data loss.
  Row 2 is the only thing that catches it, and it is the one row I would not ship without.
- **A `_` wildcard** to quiet the cascade — the swallow, restored.
- **Firing on nothing:** rows 4–11 all pass if the arms still return `Ok` unconditionally. Only rows
  1 and 3 catch it. A green floor with no behaviour change is a FAILURE.
- **Fixing it in `flush-logs`** instead of the arms — the flush is already correct; a "fix" there is
  a change with no defect behind it.
