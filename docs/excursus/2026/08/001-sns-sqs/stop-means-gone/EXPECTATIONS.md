# EXPECTATIONS — stop means gone

**Written BEFORE the strike**, from `ec3ea95c8`.

## ⛔ THE ROW THAT IS THE STONE, AND A QUIET FLOOR CANNOT GIVE IT

The floor runs on a box that is otherwise idle, and **30/30 quiet runs showed no race at all**. So a green
floor is compatible with the bug being completely untouched. Row 1 is the entire proof, and it must be run
**under load, before and after**.

| # | what | check | expected |
|---|---|---|---|
| 1 | ⭑⭑ the race is GONE under load | the 8-spinner / 20-iteration harness in BRIEF, **run before AND after** | before: ≥1 × `Sent` · **after: 20/20 `Lost` or `Closed`** |
| 2 | it is an fd event, not a sleep | read the diff | **no** sleep, **no** attempt counter, **no** new budget |
| 3 | `Stopped` still carries the state | the probe's own output | the projected state still returned, unchanged |
| 4 | `GaveUp` is reachable for the new cause | a service that acks then does not close | `GaveUp` with the waited-ms — **not** a false `Stopped` |
| 5 | the sibling was measured | SCORE says so | whether `hibernate` shares the shape, **stated either way** |
| 6 | the budget is shared, not doubled | read the diff | one `t0`, 10 000 ms total for ack + close |
| 7 | no new surface | `git diff --stat` | `wat/service.wat` only; **no** `close'`, **no** Handle field, **no** `src/` |
| 8 | floor | `./scripts/floor.sh` → **Summary line** | **5237 passed, 0 FAIL**, no `ARM.txt` |
| 9 | tests compile | `nextest --release --no-run` | clean |
| 10 | clippy | `-D warnings` | exit **0** |
| 11 | happy / chaos | the two commands | `distinct=8000;dup=0` · `distinct=100;dup=0` |
| 12 | ⚠ `stop=` phase cost | happy-path phase line vs **149 ms** today | reported either way; a large move is a finding |
| 13 | the census cells are unchanged | re-run `probe-crash-surface-send-closed-thread.wat` | still `Lost` — this stone does **not** claim `SendOutcome::Closed` |

## ⚠ Two things I will not accept as success

- **Row 1 "after" run on a quiet box.** I have already proven a quiet box is silent here. A 20/20 on an
  idle machine is not evidence, and I will re-run it under load myself.
- **A `Stopped` that is returned without the close being observed.** If `TimedOut` or `Malformed` is folded
  into "gone", the race becomes a *lie* instead of a race, which is strictly worse (STOP-2).

## Runtime prediction

**25–40 minutes.** One quasiquote arm, one sibling to measure, two load runs. Floor ~490–520 s.

## Trap-doors, ranked

1. ⛔ **Verifying on a quiet box** (row 1). The single most likely way this stone reports success while
   fixing nothing.
2. ⛔ **Folding `TimedOut` into gone** (row 4 / STOP-2).
3. **`Closed` vs `Lost` by lineage kind** — thread → `Closed`, process → `Lost`. Accept either; requiring
   one hangs on the other.
4. **The sibling** (`hibernate`). Five one-sided fixes in this campaign already.
5. **`stop=` cost** — 9 services × one extra recv. Watch it; do not hide it.
