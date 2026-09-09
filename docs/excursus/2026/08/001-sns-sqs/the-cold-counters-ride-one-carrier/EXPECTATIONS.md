# EXPECTATIONS — the cold counters ride one carrier

Written **before** the strike.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ⛔ **the numbers still reconcile** | the per-tier report line | `put+delete+count+scan` calls and ns sum to `store-calls`/`store-ns`, **remainder 0**, on every tier of every run |
| 2 | ⛔ **the report line is byte-identical in format** | diff a line against a pre-change run | same keys, same order, same separators. `circuit.wat` parses it |
| 3 | ⛔ **fanout is still complete** | n=1000/2000/4000 | `distinct = n×m`, `dup = 0`, inbox `accepted = n` |
| 4 | ★ **`State` sheds exactly six fields** | read the `:ephemeral` block | 29 → 24 plus `counters`; the six are gone as top-level fields |
| 5 | ★ **the ceremony is gone** | `grep -c` per field | **0** remaining `:<f> (:queue::queue::State/<f> s)` lines for the six; net ≈ **−142** lines |
| 6 | ★ **`:queue::Stats` is untouched** | `git diff` | its `defrecord` at `:113` is not in the diff, and no `:queue::Stats/` read outside `sqs.wat` changes |
| 7 | **`handler-ns` stays flat** | read the `:ephemeral` block | still a top-level `State` field. It is changed at 27 of 30 sites; a carrier makes it worse |
| 8 | **the corpus loads** | `every_wat_scripts_file_loads` | PASS |
| 9 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5237 passed, 22 skipped, **0 FAIL, 0 TIMEOUT** |
| 10 | **blast radius** | `git status --porcelain` | `sqs.wat` + the SCORE. Anything the compiler or gate forced, named |

## ★★ Row 11 — the row whose failure is MINE

| # | what | how | expected |
|---|---|---|---|
| 11 | ★★ **`drain` falls, with non-overlapping spreads** | 3 runs × 3 sizes, before and after, box quiet | `drain` median lower at every `n`, and the before/after bands **must not overlap** |

⛔ **This row is designed so my mechanism can be wrong.** The stone rests entirely on
`the-store-reports-time-per-operation/SCORE.md:177–192`, which measured `+9.0/+10.9/+10.9 %` on
`drain` and named the cause as **allocation** — *"State 8 fields wider … no added store call, no
added clock read."* If removing 5 net fields from a record rebuilt at 30 sites on every message
moves `drain` by nothing, **that mechanism is refuted**, and that is the finding. Report it with the
numbers and STOP; do not go looking for a second change to make the row pass.

⚠ **I am deliberately NOT predicting ~10 %.** That figure covers +8 `State` fields **and** +4
`TakeAcc` **and** +8 `Stats` **and** a nested tuple. This removes 5 net fields from `State` only. Any
number I named would be arithmetic I have not done, so the row asks only for **direction with
separated bands**.

## ⚠ Row 12 — what this stone must NOT be credited with

| # | what | expected |
|---|---|---|
| 12 | ⚠ **the drain SLOPE does not move** | `drain` 4000/1000 stays ≈ **4.44** |

That same SCORE says the instrument's cost *"is a **multiplier**, so every ratio is unaffected"*
(r21 2.340 / r42 2.388 vs baseline 2.301 / 2.388). **This stone buys level, not slope.** A SCORE
claiming the superlinearity improved is measuring noise — and if the ratio *does* move materially,
that is unexplained and belongs in the SCORE as an open question, not as a win.

## Runtime prediction

**60–90 minutes.** The edit is mechanical across 30 rebuild sites; the measurement is 18 circuit runs
(~9 min) plus the floor (~8 min). Expect more edit sites than the sketch names — every prior stone
did.

## Trap-doors

- **`ticks` and `acks` are changed at 2 sites each**, the other four at 1. Those ≤8 sites rebuild the
  carrier; the other ≥22 copy it by reference. Getting that backwards costs the whole benefit.
- **`:init` initialises the carrier** (`:417`, `:434`) — six zeros become one `Counters` construction.
- **`visible` and `unacked` are NOT counters** — they are read from the store. They live in `Stats`
  only and must not enter the carrier.
- **`TakeAcc` (`:97`) and `take`'s nested return tuple** are the other allocation sites that SCORE
  named. **Out of scope** — they are inside the waiter fold, not the `State` rebuild.
- **Do not raise or lower `:cap`, `:max-entries [msgs 10]`, or `vis-ms`.** Baseline comparability
  depends on all three being frozen.
- **The box must be quiet before any timing run**, and heavy runs go through `scripts/capped.sh`.

## What this stone does NOT claim

⚠ It does not touch the residual drain superlinearity (row 12 says so explicitly), `scan-index`'s
+45 % level shift, `count-index`'s 1.27×, or the `:cap 64` question.
⚠ It does not touch `wat/`, `src/`, the store, or `:queue::Stats`.
⚠ It does not address the WARM store-op counter pairs — that needs co-occurrence measured, not
frequency, and is affirmatively **cut**, not deferred.
