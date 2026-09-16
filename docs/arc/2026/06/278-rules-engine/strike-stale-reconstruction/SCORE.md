# SCORE — C6, weighed against the orchestrator's own re-run

> **STOP-1 FIRED AND THAT IS THE RESULT.** No band was committed, no assertion of the declared check
> was landed, and the strike's yield is a finding neither the row nor my brief anticipated: **the
> fire calls `exec_where` ZERO times on this axis.** Three defects in my brief, one of them the
> premise the whole strike rested on.

## The scorecard

| # | required | result, MY re-run |
|---|---|---|
| 1 | ★ no frozen filter constant | ✅ `FILTER_MS_MEASURED_IN_FIRE` deleted; read live via `node_share_phase_census(N, M)`, panics naming the rows if `filter` is absent |
| 2 | ★ reconstruction uses the native arm | ✅ `F + C`, not `B + C` |
| 3 | ★ the declared check is asserted | ⛔ **deliberately NOT** — see A. Recorded in-code with its samples and mechanism |
| 4 | staleness gone | ✅ compared value is now whatever the run measures |
| 5 | headroom study survives | ✅ `A`,`B`,`D`,`E`,`B−A`,`D−A`,`B−E`,`B/F` unchanged |
| 6 | ⚠ STOP-1 honoured | ⭐ **fired, correctly.** No band widened |
| 7 | engine untouched | ✅ `git diff --stat -- src/rete/kernel/fire/` → empty |
| 8 | radius | ✅ `node_share_cost.rs` only, +96 −18 |
| 9 | lints | ✅ 210/210 |
| 10 | clippy | ✅ rc=0 |
| — | floor | ✅ **`5327 tests run: 5327 passed, 21 skipped`**, exit=0 |

## ⭐ A — THE REAL FINDING: THE FIRE CALLS `exec_where` ZERO TIMES

I drove `node_share_filter_eval_census` myself. At **every** size:

```
  rules  items |    evals    reuse    passes   wasted  waste%   evals/token
     10    200 |        0       200       200        0    0.0%        0.00  | envs  0  keyallocs  0
     25    200 |        0       200       200        0    0.0%        0.00  | envs  0  keyallocs  0
     50    200 |        0       200       200        0    0.0%        0.00  | envs  0  keyallocs  0
```

`dispatch_where_tests` finds every candidate `proven && is_pure_cmp` and takes the reuse branch,
skipping the eval. Verified at `fire/mod.rs:2038-2039`:

```rust
if proven.contains(&tid) && sink.where_tree.is_pure_cmp(tid) {
    census_count("filter:test-reuse");
    ...
    continue;
}
```

**Zero evals, zero `Environment` builds, zero key allocations.** So arm `F` is scaled to
`evals_per_round = N × tokens = 10,000` — the *pre-where-tree* count. "ONE ROUND'S WORTH", the
phrase the whole harness is built around, is itself stale: a round's worth of `exec_where` is now
**0**. And rescaling does not rescue it — at the true scale `F` contributes nothing, leaving `C`
alone (~0.13 ms) against a ~0.39 ms phase, ~34%. The missing ~66% is the per-token
`where_tree.candidates` walk, `bind_view`, two `HashSet` builds and the `d_beta` pushes, and **no arm
in this file measures any of it.**

That is why refusing the band was right. Six consecutive runs read **684 693 734 723 686 698 %** —
a 7% spread, well inside the arc's ~16% noise floor, so a **stable structural over-count**, not
noise. A band admitting 7x would have re-created the exact defect the strike was cleaning.

## ⛔ B — MY BRIEF QUOTED THE WRONG TABLE ROW, AND FIVE NUMBERS INHERIT IT

I stated the live `filter` as **0.14 ms**. It is **0.38–0.39**. The census prints **three** size
blocks and I took the first `filter` grep hit — the **10/200** row:

```
  10/200  (400 facts)   filter   0.14 ms
  25/200  (400 facts)   filter   0.23 ms
  50/200  (400 facts)   filter   0.38 ms     <- the one the comment names
```

Corrected consequences: the constant is **~18x** stale, not 49x; `F + C` is **~7x** over, not 19x.
DESIGN § Why and § THE STOP THAT MATTERS, and EXPECTATIONS rows 4 and 6, all carried the wrong
figures — **corrected in place**. Worse, STOP-2 said *"if the live read comes back at a very
different scale than ~0.14 ms, stop"*: 0.39 is 2.8x that, so my own brief would have tripped a
trigger that existed only because I misread a table. The rider continued on STOP-2's stated
*rationale* (both tests at `[50 200]`) rather than its number, and was right to.

**The lesson is narrow and mine:** a grep for a row name in a multi-block table returns the first
block, not the block you meant. Take the row *with* its block header.

## ⛔⛔ C — AND MY CENTRAL PREMISE WAS FALSE

BRIEF item 2 asserted `evals_per_round = N × tokens` makes the arms and the phase commensurable —
*"This is why the comparison is legitimately on the same scale."* It does not. I had even corrected
myself toward this belief while drawing: I briefly suspected the fire evaluates ~200 times, checked
`evals_per_round`, found 10,000, and concluded the scaling was sound. **I never asked how many times
the fire actually calls the verb.** The answer is zero.

So my DESIGN framed swapping `B → F` as the second defect, when it is cosmetic beside the scale being
dead. Had the rider followed the brief literally and found a band that fit, it would have pinned a
benchmark whose native arm measures work the engine deleted.

## ⛔ D — AND THE CITATION WAS THE WRONG LANDMARK

I cited `fire/mod.rs:1996` for the native path. It is the tail of `exec_stashed_where`, a one-line
wrapper — it shows a path that *reaches* `exec_where` and gives no hint the tree short-circuits it.
The landmark is `dispatch_where_tests` at `:2012`, and the decision at `:2038`. Reading my item 5 as
written cannot surface the zero-eval fact. Third strike running where my read-list was the weak part.

## ⭐ E — WHAT THE RIDER LEFT, AND WHY IT IS RIGHT

The declared check is now *runnable for the first time* — live read, native arm — and it fails. Rather
than assert a lie or delete the question, the rider recorded the refusal in code: the six samples, the
zero-eval census, the `fire/mod.rs` citations, and the sentence that a band admitting 7x would
re-create the defect. Two non-vacuity assertions remain and both interpolate the whole table, so a red
carries its own evidence — proven by its probe, which rendered the full table after the message.

It also corrected two further stale claims in the same doc header, in radius and unasked: the
present-tense "10,000 `Environment` builds and 10,000 key allocations per fire" (measured: 0 and 0),
and "the same scale as the 6.83 ms `filter` reading".

## ⚠ F — THE MUTATION CONTRACT COULD NOT BE FULLY HONOURED, AND THE RIDER SAID SO

With no committed band, mutations 1 and 2 cannot both be red/green discriminations against committed
code. Mutation 1 *is* a genuine flip against a probe (`filter` 0.41 → `production` 0.099, green→red).
Mutation 2 is red before and after, so its value is arithmetic — `2.727 → 25.971` matches `10×F + C`,
which positively proves the line is fed by `F` and not `B`. The rider disclosed this rather than
presenting two clean reds. That disclosure is worth more than the proof it withholds.

## Per-arm status

| arm | status |
|---|---|
| live `filter` read | **proven** — 10 reads, 0.386–0.423 ms; row identity mutation-proved |
| `F` (`exec_where`) | **proven as a measurement, NOT reachable as a reconstruction** — it is the verb the fire calls, and the fire calls it 0 times here |
| `A`, `B`, `D` | **proven as measurements** — and all three measure `Environment` builds the fire performs **0** of |
| `C` (token clone) | **proven** — the one arm still corresponding to real filter-phase work |
| `E` (Rust floor) | **proven**, unchanged |
| the declared check | **NOT asserted, by decision** — the mechanism that makes it fail is measured and recorded |
