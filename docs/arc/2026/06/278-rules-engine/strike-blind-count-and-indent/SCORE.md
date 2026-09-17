# SCORE — C10 + C11, weighed against the orchestrator's own re-run

> Two small corrections, both landed. **The finding is that my brief said "a union of three sources"
> and named two — and the third one means the counter is not a call count at all.**

| # | required | result, MY re-run |
|---|---|---|
| 1 | C10: cross-reference at the 80,200 gate | ✅ comment only, no gate, no engine edit |
| 2 | C11: the indent survives | ✅ verified by me in raw bytes — `  seed` / `  delta` now indented under `alpha` |
| 3 | radius | ✅ 4 files, all `src/rete/kernel/tests/`, +46 −16 |
| 4 | engine untouched | ✅ nothing under `fire/` |
| 5 | lints | ✅ 210/210 (217 in the rider's wider filter) |
| 6 | clippy | ✅ rc=0 |
| — | floor | ✅ **`5327 tests run: 5327 passed, 21 skipped`**, exit=0 |

## ⛔ A — MY BRIEF SAID "THREE SOURCES" AND LISTED TWO. THE MISSING ONE IS THE POINT.

I named `fire/delta.rs:78` and `compiled_cond.rs:928`. The third, verified by me:

```
src/rete/compiled_cond.rs:928       census_count("compiled:calls");
src/rete/kernel/fire/delta.rs:78    census_count("compiled:calls");
src/rete/kernel/fire/pass/alpha.rs:122
        census_count_n("compiled:calls", ids.len() as u64 * aids.len() as u64);
```

The third is a **multiplicative bulk add**, not a per-call bump. So `compiled:calls` is not merely
arm-blind — **it is not a call count.** Its name says otherwise, and `accum_cost.rs:46-47` glosses
80,200 as *"one per (fact, matching alpha) pair"*, which is a property of this workload rather than
of the counter. Opened as **C14**; the rider correctly left it alone as outside remit.

I had already read `alpha.rs:122` during the C4 crawl — it was in the grep output — and still wrote
"three sources" while listing two. Naming a count without naming its sites is how the label rots.

## ⭐ B — THE RIDER ENUMERATED BEFORE FIXING, WHICH THE BRIEF ONLY IMPLIED

STOP-3 said *"if you find more than a handful, stop and report the list"*. It scanned every
`\`-continuation block under the radius, kept only rendered tables, and found **4 blocks, 16 rows, 4
files** — then swept all four and said why it judged that a handful. It also scanned **outside** the
radius (all of `src/`, `tests/`, `benches/`) and found **zero** true victims: the 245 other
deep-indent continuations are embedded `.wat` source strings (whitespace-insensitive) or wrapped
prose in assertion messages, where the strip is intended. That negative is what makes "4 files" a
bounded claim instead of a sample.

## ⚠ C — STOP-2 FIRES IN A SENSE I WROTE BADLY

I wrote *"if the indent fix moves any number or column, stop."* Restoring an indent necessarily moves
those rows' printed columns right — **that is the defect** (*"it only shortens the pad"*), so any
correct fix trips the trigger as literally worded. No value changed and no other row moved. The
trigger should have said *"if any number's VALUE changes, or any row other than the 16 moves."*

## ⚠ D — THE STALE SAMPLE IS QUOTED INSIDE A GATE'S DOC

The flush-left `insert 0.00 ms` block I used as an illustration lives verbatim at
`tests/lint/census_name_read_by_a_cost_test_is_emitted.rs:19-22`, as the historical finding that
motivated that gate. It is prose quoting old stdout, correctly left alone — but anyone grepping for
the sample lands there and may read it as a current table.

## Per-arm status

| arm | status |
|---|---|
| C10 cross-reference | **proven** — the orchestrator's engine mutation (skip_span → false) is its evidence: the 80,200 gate PASSED while the C4 probe went RED |
| C11 render, 16 rows | **proven** — driven in the real test binary, raw bytes, by the rider and re-verified by me; mutation (remove the indent) visibly reverts it |
| victims outside the radius | **proven absent** — scanned, zero |
| `compiled:calls` as a *name* | **not addressed — C14** |
