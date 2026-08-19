# EXPECTATIONS — STONE 118.B6b · retire `foldr`

Written before the strike. Rows 6–8 are the ORCHESTRATOR's.

| # | what | who | expected |
|---|---|---|---|
| 1 | ★ `(foldr …)` is refused | rider, `--check` | refused; message points at the replacement |
| 2 | ★ `(reduce f init (reverse coll))` still gives `foldr`'s answer | rider | **2** for `1-(2-(3-0))` — a left fold gives −6 |
| 3 | the 4 tests are **rewritten, not deleted** | rider + orchestrator, read the diff | present, same assertions, new spelling |
| 4 | all three ledgers clean | rider, build | `is_pure_total`, `intrinsic_meta`, rete vocabulary |
| 5 | ★ both capability headers name their REAL consumers | rider, read the diff | `mappable()` drops `foldr`; `ordered()` says `reverse`/`concat` only |
| 6 | floor | **orchestrator** | ≥4772 run, 0 FAIL, 19 skipped |
| 7 | clippy | **orchestrator** | 0 |
| 8 | ignores | **orchestrator** | 13 |
| 9 | `wat/seq.wat` untouched | rider, `git status` | `reduce` and `foldl` unchanged |

**Row 2 is the one that proves the retirement is safe rather than merely tidy.** Deleting a verb is
easy; showing its capability survives in the replacement is the work. A test asserting `2` (not `−6`)
discriminates a right fold from a left one.

## Independent prediction

**45–70 minutes.** The deletion is mechanical across 9 files; the three ledgers and the rete
vocabulary row are where the time goes. Test rewriting is small — 4 sites.

## Trap-doors named in advance

- **The rete vocabulary row is the likeliest red.** A verb declared there and deleted from core is a
  dangling declaration; B4-0's purity gate caught exactly this shape one stone ago.
- **Three ledgers, no links between them.** Satisfying one does nothing for the others — the same
  finding today's `255/NOTE-promotion-is-not-relocation-…` recorded.
- **`ordered()`'s header is stale INDEPENDENTLY of `foldr`** — it was already wrong before this stone,
  measured. Fixing it here is finishing the sentence, not scope creep; leaving it is knowingly
  shipping a second lying comment in the file you are already editing.
- **Do not argue from the call count.** 5 sites, 4 tests and a string literal — the stone is explicit
  that this is NOT the argument. If a rider's report leans on "nothing used it", the reasoning is
  wrong even where the action is right. `[[feedback_no_consumers_does_not_mean_dead]]`

---

## ⛔ AMENDED 2026-08-18 (pre-strike) — three rows added, one row's instrument corrected

Re-measured before release; the BRIEF's amendment block carries the evidence.

| # | what | who | expected |
|---|---|---|---|
| 10 | ★ the retirement TABLE names the replacement | rider, `--check` a `.bad` fixture | `src/remedy/retirement.rs` has a `":wat::core::foldr"` row; the diagnostic carries `reduce` + the strictness note. **Row 1 is unreachable without this** — a deleted dispatch arm alone yields a generic unknown-form error |
| 11 | ★ `wat-scripts/` still LOADS | rider, scoped `nextest -E 'test(every_wat_scripts_file_loads)'` | green. `probe-arc278-57-round1b-parametric-and-hof.wat`'s `:probe-foldr` (the RETE spelling, missed by the original census) is removed with an inline note; header says four combinators, not five |
| 12 | the negative control SURVIVES | orchestrator, read the diff | `foldl_vs_foldr_differ_on_nonassoc_op` is **renamed**, not deleted — measured: it calls only `foldl` and asserts −6; the `foldr` was always in the name |

**Row 2's instrument was wrong and is corrected.** It read "a test asserting **2**" as if one had to
be written. It already exists: **`src/runtime.rs:37866 foldr_is_right_associative`**, asserting `2`
for `1-(2-(3-0))`. The row is satisfied by **rewriting that test in place** to
`(reduce f init (reverse coll))` with the assertion untouched — which is a strictly stronger
demonstration than a new fixture, because the same assertion that certified the verb now certifies
its replacement.

## ⚠ THE HOLE THIS STONE OPENS — surfaced pre-strike, the builder's to rule

The rete vocabulary has `foldl` · `foldr` · `map` · `filter` · **`reduce`** — and **no `reverse`**.
Core's replacement `(reduce f init (reverse coll))` therefore has **no rete spelling**: after this
stone a `where` body cannot express a right fold at all.

Row removal is forced regardless (a `Redispatch` alias pointing at a deleted `core_name` is a
dangling declaration). Minting `:wat::rete::core::reverse` is a surface addition — **out of the
rider's hands, reported not acted on.**
