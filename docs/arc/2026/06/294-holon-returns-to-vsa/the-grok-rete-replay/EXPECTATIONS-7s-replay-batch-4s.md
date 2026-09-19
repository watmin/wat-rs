# EXPECTATIONS 7s — replay batch 4s, grok-rete #501 → #520 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended.

## ⚠ Which rows were PRE-FLIGHTED, and which are predictions

**Executed against HEAD `9cc844d23` and ALL PASS:** E8 (origin an ancestor; `refs/original/` empty), E9
(0 replace refs), tree clean, 500 REPLAY steps, E2 (19 docs-only), E2b (#501 the only code step), E4 (the
whole range is 34 `.md` + 1 `.wat`; zero `src/`, zero hazard rows, zero new gates, zero
`wat-scripts/fixes/`, zero `wat/`), the **zero** `#[test]`/`#[ignore]` delta, and **all 72 added
`path:line` citations resolving in range here**.

**Predictions, not verified rows:** E3, E5–E7, E10–E16.

⛔ **What I could NOT measure and did not pretend to.** Whether any vigilia doc makes a claim that trips
a gate I have not anticipated — these are 34 files of dense prose citing this tree's internals.
**E5 scores running the docs gates and reporting their verdicts.**

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh 9cc844d23 HEAD 501 520` | exit 0; `sources match` |
| E1b | ⛔ **every subject is GROK'S, verbatim** (finding 39) | per step vs `git log -1 --format=%s <C>` | 20 of 20, kind included |
| E1c | ⛔ **trailers COPIED, not typed** | per step vs `commits.tsv`, two-sided | 20 of 20 — five were fabricated across the last two batches |
| E2 | docs-only steps are docs-only | the **19** (#502–#520) | only `docs/`/`.md` |
| E2b | **and the inverse** | #501 | one `.wat` under `wat-scripts/scratch-pad/`, comment-only |
| E3 | ⛔ **GROK'S MEASUREMENTS WERE NOT "CORRECTED"** | `git diff` the landed docs vs grok's blobs | the vigilia prose lands as grok wrote it; any figure false of THIS tree is noted in the SCORE, never edited in place |
| E4 | ZERO `src/`, zero hazard paths, zero new gates | `git diff --name-only` over the range | none |
| E5 | **the docs gates are GREEN and their verdicts were READ** | `-E 'test(no_stale_path_in_doc)'` and the docs-wat gate | green, N > 0; verdicts quoted |
| E6 | **#501's loader gates ran** | `-E 'test(every_wat_scripts_file_loads_on_the_current_runtime)'` | green, N > 0 at #501 |
| E7 | finding 33's class swept at #501 | the SCORE | explicit; it is a `.wat`, so the question is its own syntax, not a `.rs` literal |
| E8 | **NO PUBLISHED HISTORY REWRITTEN** | `merge-base --is-ancestor`; `for-each-ref refs/original/` | ancestor YES; empty |
| E9 | repairs visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | 0 refs; exit 0 either way |
| E10 | the checkpoint | `scripts/floor.sh` + clippy `-D warnings --all-targets` | green; clippy 0 — **the orchestrator's row** |
| E11 | **the SCORE discloses what BOUGHT each green** | read the SCORE | including any grok figure that is false of this tree |
| E12 | **test-count delta is ZERO** | predict from the diff BEFORE the floor | **5872 run, 22 skipped — unchanged.** A different number is a result to report |
| E13 | **every `census:` line is TRUE, and on ONE line** | grep each verdict | no wrapped line |
| E14 | **every deviation from the brief is REPORTED** | the SCORE | honest disagreement passes; agreement does not earn it |
| E15 | **NO COUNTERPART ACTIVITY**; no unfiltered run | `.floor/`, `git status`, `.pulsare/` mtimes | none; frozen root untouched |
| E16 | no knowingly-red commit; messages survived their heredoc | per-step landing; `git log --format=%B` | none red; no eaten backtick spans |

## ⛔ E3 is the row that catches a lazy 4s

This batch is 34 files of prose, and prose lands without complaint. The temptation is to read a grok
figure that is false here — "all 25 counters", "65 exemptions" — and quietly correct it. **That falsifies
the record being replayed.** The SCORE names the divergence; grok's text keeps its own words.

**Runtime prediction:** ~40–70 min — nineteen docs steps and one comment fix.

**What would make me reject the batch:** an edited grok measurement; a fabricated trailer or paraphrased
subject; a wrapped verdict line; a docs gate's verdict assumed rather than run; any `refs/original/`
entry; any `pulsare_yield`.
