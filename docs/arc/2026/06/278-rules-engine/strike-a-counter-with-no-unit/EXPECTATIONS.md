# EXPECTATIONS — one counter, one unit

> ⚠ **This strike does NOT make the accum axis exercise the compiled path.** It makes the
> instruments say what is true about it. A report claiming the compiled path is now measured on this
> axis is wrong.

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,407 plus every arm you drive.**

## The scorecard — every pre-value driven at HEAD `d7464c95e`

| # | what | state AT HEAD (driven) | required after |
|---|---|---|---|
| 1 | ★ each counter carries ONE unit | `compiled:calls` sums calls **and** `ids×aids` | the product emits its own key; `compiled:calls` is calls only |
| 2 | ★ a lost call site is detectable | ⛔ **it is not** — renaming the product takes `compiled:calls` to **0**, so the two per-call sites contribute nothing and their loss is invisible | mutation 1: deleting `delta.rs:78`'s bump REDs **after** the split — and demonstrably does **not** before |
| 3 | ★ the duplicate pin is gone | **two tests pin 80,200**: `alpha_elements` (correctly named) and `compiled:calls` (not) | `accum_alpha_memory_shape` keeps its pin; `accum_matcher_op_census` stops restating it |
| 4 | the honest value is stated | the compiled path is entered **0** times on this workload | asserted **as zero, with the reason** — or the workload that does enter it is NAMED, not dialled |
| 5 | the liveness guard stops over-claiming | `calls > 0` names *"occupancy fill / skip-span / exec_compiled"* and can observe only the first | it names only what it can see |
| 6 | the rotted citation | `accum_cost.rs:52` cites `alpha.rs:122`; the site is **`:195`** | corrected — and a symbol beats a line number |
| 7 | not the split C10 forbade | — | stated, with the distinction argued; or STOP-2 fired |
| 8 | nothing else silently re-defined | — | every other pin/quote of 80,200 found and named |
| 9 | floor / lints / clippy | **`5407 tests run: 5407 passed, 21 skipped`** (439.6 s, 0 FAIL rows), lints **258**, clippy rc=0 | ≥ 5407 + arms, 0 FAIL, lints ≥ 258, rc=0 |

## Runtime prediction

**50–80 minutes.** The rename is one string. Deciding what `accum_matcher_op_census` should honestly
assert — and proving mutation 1's *before* half — is the work.

## Trap doors named in advance

- **⛔ MUTATION 1 NEEDS BOTH HALVES.** "Deleting a call site REDs the test" proves nothing on its own
  unless you also show it **did not** RED before the split. The before-half is the finding.
- **80,200 is load-bearing in two files.** Changing what `compiled:calls` means without finding every
  consumer re-defines a number someone else asserts on. STOP-3.
- **`git checkout <sha> -- <path>` STAGES.** Restore by hash.
- **A hot-path edit for an instrument's benefit is forbidden** and C10 says so. One string at one
  existing call site is not that — but argue it, do not assume it.

## What would make this strike a failure even if every test passes

**Re-pinning `compiled:calls` at a new constant and calling it done.** The defect is not that 80,200
is wrong; it is that the number is a pair count wearing a call count's name, and that its liveness
guard vouches for two mechanisms it has never observed. A new constant under the same confusion is
the same defect with fresher digits.

**And quietly making the assertion pass.** If the honest answer is "the compiled path is entered zero
times on this axis", that sentence belongs in the test where the next reader will meet it — not
deleted because a zero looks like a broken test.
