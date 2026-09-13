# SCORE — the check skips what cannot dial

**SCORED. KEEP.** Executor: grok, 2026-09-13, branch `sns-sqs`, HEAD `c0ab17d95` (DRAWN). Did not commit.

The guard ships: mean saving **33.2 s**, well above the ~6 s band.

---

## ⭑ THE HEADLINE — KEEP, −33 s

One `if` around `collect-impls-dialed`: if `addr-fields` is empty, return empty (the collector could only have returned empty). Construction-proof no-op on semantics.

```
WITHOUT  564.951 · 572.482 · 566.459 s    mean 568.0 s
WITH     535.089 · 533.940 · 535.253 s    mean 534.8 s
saving   33.2 s
```

All six floors **5241 passed**, 0 FAIL, no `ARM.txt`.

---

## Box

**Quiet of other jobs.** No competing `cargo` / `nextest` / `circuit` between runs. Sequential floors; loadavg 5-min/15-min is the previous floor's own tail, not a second workload. Timing measurement; stated.

---

## WITHOUT (HEAD, unguarded walk)

| # | Summary | dir | 1-min loadavg at start |
|---|---|---|---|
| 1 | `Summary [ 564.951s] 5241 tests run: 5241 passed (9 slow), 22 skipped` | `.floor/2026-09-13T07-27-57Z/` | 0.58 |
| 2 | `Summary [ 572.482s] 5241 tests run: 5241 passed (9 slow), 22 skipped` | `.floor/2026-09-13T07-37-29Z/` | 4.08 (floor-1 tail) |
| 3 | `Summary [ 566.459s] 5241 tests run: 5241 passed (9 slow), 22 skipped` | `.floor/2026-09-13T07-47-24Z/` | 2.66 (floor-2 tail) |

Spread **7.5 s**. Mean **568.0 s**.

`every_wat_scripts_file_loads`: 564.944 · 572.475 · 566.452 s — that test **is** the floor clock.

---

## WITH (guard)

| # | Summary | dir |
|---|---|---|
| 1 | `Summary [ 535.089s] 5241 tests run: 5241 passed (9 slow), 22 skipped` | `.floor/2026-09-13T07-59-39Z/` |
| 2 | `Summary [ 533.940s] 5241 tests run: 5241 passed (9 slow), 22 skipped` | `.floor/2026-09-13T08-08-46Z/` |
| 3 | `Summary [ 535.253s] 5241 tests run: 5241 passed (9 slow), 22 skipped` | `.floor/2026-09-13T08-17-54Z/` |

Spread **1.3 s**. Mean **534.8 s**.

`every_wat_scripts_file_loads`: 535.082 · 533.934 · 535.246 s.

Saving lives on that one test (~30 s). The rest of the floor is noise.

---

## Ruling

**KEEP.** 33.2 s is five times the ~6 s band. Not a 3 s flicker. The +27–47 s from `the-dial-declares-its-peer` is **not** dominated by services that hold Address fields; skipping the walk when they cannot dial recovers most of it.

Did not quote a service count (DESIGN forbids it).

---

## Semantics unchanged

`git diff -- wat/service.wat` is the `empty?` wrap of `impls-dialed-surfaces` only. Siblings untouched. Collector untouched.

Refusal still fires (`dial_declares` **2 passed**). `--check` sqs / circuit / sns-fanout **exit 0**. Happy `distinct=8000;dup=0`. Clippy **0**. `src/` **EMPTY**.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | construction no-op | guarded on `addr-fields` emptiness only |
| 2 | ≥3 WITHOUT | three Summary lines above |
| 3 | ≥3 WITH | three Summary lines above |
| 4 | ruling follows the number | **KEEP** — 33.2 s ≫ 6 s |
| 5 | refusal still fires | 2 passed |
| 6 | three corpus `--check` | 0 / 0 / 0 |
| 7 | semantics | only `:1029` wrapped |
| 8 | floor 5241 | all six green at 5241 |
| 9 | clippy | 0 |
| 10 | no `src/` | EMPTY |
| 11 | happy | `distinct=8000;dup=0` |
| 12 | box quiet | no competing jobs; loadavg tail is our floors |

---

# ⭑ ORCHESTRATOR'S GRADING — claude, 2026-09-13

**Graded against my OWN reads and runs**, from `c0ab17d95` + the working tree.

```
floor   (my own run, guarded)  Summary [ 528.833s] 5241 tests run: 5241 passed, 22 skipped
        0 failure tokens · no ARM.txt        clippy → 0        happy  distinct=8000;dup=0
diff    wat/service.wat: the `empty? addr-fields` wrap and NOTHING else · src/ EMPTY
```

**STRUCK — KEEP. And my own numbers say something stronger than "saved 33 s."**

## ⭑⭑⭑ THE FULL ACCOUNTING — the check is now indistinguishable from FREE

The SCORE frames the result as a 33.2 s saving, which is true and well-measured. But the number the builder
actually needs is the check's **residual cost**, and that needs my pre-check baseline:

```
pre-check  (before the 3rd check existed)   523.1 · 523.6 · 527.5 · 529.3 s   mean 525.9   ← my runs
unguarded  (49bdf2b41)                      556.5 s (mine) · 570.2 s (grok's)
GUARDED    (this tree)                      528.833 s (mine) · 534.8 mean (grok's)
```

⭐ **My guarded run, 528.833 s, sits INSIDE my own pre-check band (523.1–529.3).** So on my box the third
bijection check went from costing **+30.6 s** to costing **+2.9 s** — at the noise floor.

⚠ **Stated honestly, not over-claimed:** one run inside a band is *"indistinguishable from free on one run,"*
not proof of free. grok's three-run mean (534.8) sits ~9 s above my pre-check mean. **So the residual is
3–9 s, at or near the band — down from 31–44 s.** The guard recovers essentially all of the cost.

## ★ And the saving has a single, legible home

```
528.827s  wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime
143.461s  wat::cli retirement_table_reachable
 87.280s  wat::rete wat_scripts_grid_axes_live
```

**That one test IS the floor clock** — `528.827` of a `528.833` wall. It type-checks every `.wat` under
`wat-scripts/`, so it expands every `defservice` in the corpus and pays the walk once per service. ⭑ That
explains the whole effect and makes the result mechanistic rather than statistical: the cost was one test, the
guard removed it from that test, and the rest of the floor never mattered.

## Row 4 — the ruling follows the number, and I would have accepted the other answer

**33.2 s is five times the ~6 s band.** KEEP is correct. ⭑ I had committed in advance that a 3 s difference
would mean **revert**, and that a reverting SCORE with three clean pairs would be a *pass* — so this is the
ruling the measurement compelled, not the one the stone wanted.

## The rows

| # | row | my verdict |
|---|---|---|
| 1 | construction no-op | ✅ guarded on `addr-fields` emptiness alone; the collector's result for that case was already empty |
| 2 | ≥3 WITHOUT | ✅ 564.951 · 572.482 · 566.459, spread **7.5 s** |
| 3 | ≥3 WITH | ✅ 533.940 · 535.089 · 535.253, spread **1.3 s** — ⭑ and my own independent 528.833 corroborates |
| 4 | ruling follows the number | ✅ **KEEP**, 33.2 s ≫ 6 s |
| 5 | refusal still fires | ✅ **my run** — `dial_declares` 2 passed in 0.46 s |
| 6 | three corpus `--check`s | ✅ **my runs** — sqs / circuit / sns-fanout all exit 0 |
| 7 | semantics untouched | ✅ **my read** — the diff is 2 lines out, 4 in, the wrap only |
| 8 | floor at 5241 | ✅ my own run, 0 FAIL, no `ARM.txt` |
| 9 | clippy | ✅ my own run, 0 |
| 10 | no `src/` | ✅ EMPTY |
| 11 | happy path | ✅ `distinct=8000;dup=0` |
| 12 | box state stated | ✅ and honestly — the SCORE notes the loadavg tail is **its own previous floor**, not a second workload. ⭑ That is the right level of care for a timing claim, and it is the campaign's own lesson applied |

## What I'd credit above all

**It stated the loadavg per run and named the tail as its own previous floor.** A timing measurement that
volunteers its box state — including a non-zero loadavg with an explanation — is the shape this campaign
learned to demand after publishing a rate measured against a background compile. The spread tells the same
story: **1.3 s across the three guarded runs**, which is tighter than my four-run pre-check band.

## What this hands the next stone

The guard is in and the check is effectively free, so **the floor has its ~30 s back** — which is exactly the
budget the second named item needs. ⛔ **Next: the store-fault proxy is in NO floor test** (`grep -rln
faulting-store tests/**/*.rs` → nothing). That is D2 applied to the other injector, and `the-store-can-fail`'s
own probe plus `the-chaos-gate-has-teeth`'s harness shape are both already on disk.
