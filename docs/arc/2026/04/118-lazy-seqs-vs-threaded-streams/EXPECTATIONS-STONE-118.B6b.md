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
