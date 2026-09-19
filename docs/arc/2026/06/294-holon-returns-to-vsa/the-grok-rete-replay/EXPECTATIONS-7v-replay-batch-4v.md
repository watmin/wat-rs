# EXPECTATIONS 7v — replay batch 4v, grok-rete #561 → #580 (written BEFORE the strike)

## ⚠ Which rows were PRE-FLIGHTED, and which are predictions

**Executed and ALL PASS:** origin an ancestor, `refs/original/` empty, 0 replace refs, tree clean, 560
REPLAY steps, 18 docs-only, #565 and #576 the only code steps, the range is 62 `.md` + 3 `.rs` + 2 `.sh`
+ 1 `.wat`, **zero** `#[test]`/`#[ignore]` delta, zero hazard rows, zero new gates, **#576's `wat/` edit
is 3 comment lines and 0 code lines**, **`clause.rs` / `eval_test.rs` / `fire.wat` all differ from grok's
pre-image here**, **our `no_unknown_ward_rune.rs` is byte-identical to grok's pre-image**, and **no `.rs`
drives #565's new `.sh`**.

**Predictions, not verified rows:** the conflict resolutions and the gate verdicts.

⛔ **What I could NOT measure.** Whether the vocabulary gate stays green here after #576's
`"shape-contract"` addition — **this tree carries 7 excusare runes against grok's floor of 5**, a
divergence older than this batch. **E4 scores running it and quoting what it says.**

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh <batch-start> HEAD 561 580` | exit 0; `sources match` |
| E1b | ⛔ **every subject is GROK'S, verbatim** | per step vs `git log -1 --format=%s <C>` | 20 of 20, kind included |
| E1c | ⛔ **trailers PIPED, not typed** | per step vs `commits.tsv`, two-sided | 20 of 20 |
| E2 | docs-only steps are docs-only | the **18** | only `docs/`/`.md` |
| E2b | **and the inverse** | #565, #576 | each carries ≥1 non-docs file |
| E3 | **#576's `wat/` edit stayed comments-only** | `git show` #576 on `wat/` | 3 comment lines, 0 code; a code line is reported |
| E4 | ⛔ **the vocabulary gate is GREEN and its verdict READ** | `-E 'test(no_unknown_ward_rune)'` at #576 and the tip | green, N > 0, verdict quoted — not assumed from grok's arithmetic |
| E5 | **the deltas landed, not the blobs** (finding 36) | the SCORE | delta-vs-delta for `clause.rs`, `eval_test.rs`, `fire.wat` |
| E6 | **finding 33 swept on both `.sh`** | the SCORE | explicit for `peragrare-bad-census.sh` and `run-all.sh` |
| E7 | **grok's measurements were NOT edited** | hash each touched `.md` vs grok's blob | byte-identical except where a conflict resolution is recorded |
| E8 | **NO PUBLISHED HISTORY REWRITTEN** | `merge-base --is-ancestor`; `for-each-ref refs/original/` | ancestor YES; empty |
| E9 | repairs visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | 0 refs; exit 0 either way |
| E10 | the checkpoint | `scripts/floor.sh` + clippy `-D warnings --all-targets` | green; clippy 0 — **the orchestrator's row** |
| E11 | **#576 carries its full record line** | `git show -s` #576 | `census:` + `nested-program-gate:` + `lint-subset` + `kind(lib)` + `doctest`, each on ONE line |
| E12 | **test-count delta is ZERO** | predict from the diff BEFORE the floor | **5872 run, 22 skipped — unchanged** |
| E13 | **every `census:` line is TRUE** | `census.sh --diff` at the tip | `no STOP-8` |
| E14 | **every deviation from the brief is REPORTED** | the SCORE | honest disagreement passes; agreement does not earn it |
| E15 | **NO COUNTERPART ACTIVITY**; no unfiltered run | `.floor/`, `git status`, `.pulsare/` mtimes | none; frozen root untouched |
| E16 | no knowingly-red commit; messages survived their heredoc | per-step landing; `git log --format=%B` | none red; no eaten backtick spans |

## ⛔ E4 and E5 are the rows that catch a lazy #576

The step is six small files and will apply almost everywhere. The risk is not size: it is that three of
those files have diverged here, and that a vocabulary gate's arithmetic is grok's, not ours. **The SCORE
shows delta-vs-delta and the gate's own words, or those rows fail.**

**Runtime prediction:** ~45–75 min — eighteen docs steps and one small code step with three conflicts.

**What would make me reject the batch:** a blob comparison passed off as a delta check; the vocabulary
gate's verdict assumed; a code line slipped into the `wat/` comments-only edit; an edited grok figure; a
typed trailer or paraphrased subject; any `refs/original/` entry; any `pulsare_yield`.
