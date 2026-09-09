# EXPECTATIONS — the topic forgets its subscriber count

Written **before** the strike.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ⛔ **the worker's `nsubs` survives** | `sed -n '425p' wat-scripts/topic/sns-fanout.wat` | still `[nsubs (:wat::core::count subs)]`. **The one thing this stone must not break.** |
| 2 | ⛔ **the corpus loads** | `every_wat_scripts_file_loads_on_the_current_runtime` | PASS. It parses and type-checks every affected file, so an over-match on `:425` cannot ship quietly |
| 3 | ⛔ **the field is gone from the record** | `grep -n 'nsubs <- ' wat-scripts/topic/sns-fanout.wat` | **no output** |
| 4 | ★ **no `:nsubs` kwarg survives** | `grep -rn ':nsubs' --include='*.wat' .` | only comment lines, if any |
| 5 | ★ **the codemod is recorded and idempotent** | re-run the applier on the migrated corpus | second run changes **zero bytes**; the file is committed under `wat-scripts/fixes/` |
| 6 | ★ **census before apply** | the `--grep` output, kept in the SCORE | every match listed with its file, *before* any file was rewritten |
| 7 | **behaviour is unchanged** | `./target/release/wat wat-scripts/topic/run.wat` | `"3 3"` |
| 8 | **the circuit is unchanged** | `circuit.wat 2000 4 3 8192 true 1000` | `distinct=8000`, `dup=0`, inbox `accepted=2000` |
| 9 | **the declared surface is untouched** | read `:demo::Topic::StatsResponse` | still `[depth ticks inbox-lost inbox-closed inbox-timedout]` |
| 10 | **blast radius** | `git status --porcelain` | the codemod, the migrated `.wat` files, the SCORE. **No `wat/`, no `src/`, no store** — and anything the gate forced, named |
| 11 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5237 passed, 22 skipped, **0 FAIL, 0 TIMEOUT** |

## ★★ Row 12 — the row whose failure is MINE

| # | what | expected |
|---|---|---|
| 12 | ★★ **the DESIGN's form analysis was complete** | the finder reports **exactly** the two forms the DESIGN names (kwarg pair, binder triple) and no third |

**If `wat --grep` turns up a fourth form of `nsubs` — an accessor that is not the value of a kwarg
pair, a quasiquote, a macro expansion, a `nsubs` in a type position — then my DESIGN's table is
wrong and this row fails on me, not on the executor.** Report it and STOP rather than extending the
rule set to cover it silently. **That is a complete and valuable result:** it would mean my
three-form reading of the corpus, which I built by grepping, missed a form — the same class of error
that already cost me a site count in this stone's own DESIGN.

## Runtime prediction

**45–90 minutes.** Two rules, both with committed exemplars, plus census/diff/apply and the floor
(~8 min). The risk is not volume; it is rule 2's parentage predicate.

## Trap-doors

- ⛔ **`nsubs` occurs 23 times in `sns-fanout.wat` and most must survive** — the worker's `let`
  binding at `:425` and its references at `:506 :527 :528 :533 :534`. A name-only rule breaks the
  worker. This is STOP-1.
- **The multi-line construction at `:217-222`** — the `:nsubs` value there is
  `(:demo::topic::Record/nsubs d)`, an accessor. Deleting the kwarg pair takes it; a separate
  accessor rule should therefore find **nothing** left to do. If it finds something, see row 12.
- **Comments are not rewritten** — the tool walks forms. Prose mentioning `nsubs` is a separate
  manual pass, and row 4 expects comment-only survivors.
- **Seven files, and my list may be short.** Pass every path the finder names, not the DESIGN's.
- **`:demo::topic-worker::Record` has no `nsubs` field** — do not add one, and do not "move" the
  field to the worker. The worker already derives it.

## What this stone does NOT claim

⚠ It does not touch the residual drain superlinearity, `scan-index`'s +45 % per-call level shift, or
the counter-carrier tax.
⚠ It does not change `:cap`, the batch limit `:max-entries [msgs 10]`, or any declared surface.
⚠ It does not promote anything to `wat/` — if that turns out to be necessary, that is STOP-3.
