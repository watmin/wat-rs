# EXPECTATIONS 7w — replay batch 4w, grok-rete #581 → #600 (written BEFORE the strike)

## ⚠ Which rows were PRE-FLIGHTED, and which are predictions

**Executed and ALL PASS:** origin an ancestor, `refs/original/` empty, 0 replace refs, tree clean, 580
REPLAY steps, 15 docs-only, five code steps, the range is 24 `.md` + 20 `.rs` with no `.wat` and no
`wat/`, zero hazard rows, zero new gates, the `#[test]` delta is **+5** (4 at #581, 1 at #586) with no
`#[ignore]` moving, the divergence table (**#591 5/5, #586 3/3, #598 2/5, #594 0/5**), our `clippy.toml`
byte-identical to grok's with **no threshold override**, and **#594 taking
`activate_deferred_mixed_classes` from 11 parameters to 3 via a new `AlphaActivateCx`**.

**Predictions, not verified rows:** the compositions, the gate verdicts, and the floor.

⛔ **What I could NOT measure.** Whether clippy stays at 0 after #594's allow removal — **clippy is my
row, and the executor cannot run it.** E4 scores verifying the *mechanism* (struct landed, arity fell)
rather than the lint.

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh <batch-start> HEAD 581 600` | exit 0; `sources match` |
| E1b | ⛔ **every subject is GROK'S, verbatim** | per step vs `git log -1 --format=%s <C>` | 20 of 20, kind included |
| E1c | ⛔ **trailers PIPED, not typed** | per step vs `commits.tsv`, two-sided | 20 of 20 |
| E2 | docs-only steps are docs-only | the **15** | only `docs/`/`.md` |
| E2b | **and the inverse** | the 5 code steps | each carries ≥1 non-docs file |
| E3 | **the deltas landed, not the blobs** (finding 36) | the SCORE | delta-vs-delta evidence for every conflicted file at #586, #591, #598 |
| E4 | ⛔ **#594's allow removal came WITH its cure** | `git show` #594; the SCORE | `AlphaActivateCx` landed; `activate_deferred_mixed_classes` has **3** parameters, not 11; every call site passes the struct; the count is stated |
| E5 | **the ward-vocabulary gate is GREEN after #598** | `-E 'test(no_unknown_ward_rune)'` | green, N > 0; verdict quoted — `rune:sequi(ambient-context)` must be a known category |
| E6 | finding 33's class swept per code step | the SCORE | explicit for all five; 20 `.rs` this batch |
| E7 | **each code step carries its FULL record line** | `git show -s` per step | `census:` + `nested-program-gate:` + `lint-subset` + `kind(lib)` + `doctest`, each on ONE line |
| E8 | **NO PUBLISHED HISTORY REWRITTEN** | `merge-base --is-ancestor`; `for-each-ref refs/original/` | ancestor YES; empty |
| E9 | repairs visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | 0 refs; exit 0 either way |
| E10 | the checkpoint | `scripts/floor.sh` + clippy `-D warnings --all-targets` | green; **clippy 0** — the row #594 puts at risk |
| E11 | **test-count delta ACCOUNTED FOR** | predict from the diff BEFORE the floor | **5877 run, 22 skipped** (5872 + 5) |
| E12 | **every `census:` line is TRUE** | `census.sh --diff` at the tip | `no STOP-8` |
| E13 | **the SCORE discloses what BOUGHT each green** | read the SCORE | every composition, every gate verdict |
| E14 | **every deviation from the brief is REPORTED** | the SCORE | honest disagreement passes; agreement does not earn it |
| E15 | **NO COUNTERPART ACTIVITY**; no unfiltered run | `.floor/`, `git status`, `.pulsare/` mtimes | none; frozen root untouched |
| E16 | no knowingly-red commit; messages survived their heredoc | per-step landing; `git log --format=%B` | none red; no eaten backtick spans |

## ⛔ E4 is the row that catches a lazy #594

Removing an `#[allow]` is one deleted line and nothing fails locally — the lint that would object is the
orchestrator's to run, an hour later, after nineteen more commits are stacked on top. **The SCORE states
the parameter count it measured, or E4 fails.**

**Runtime prediction:** ~90–130 min. The weight is #591 (5-of-5 diverged), #586, #598 and #594's
verification.

**What would make me reject the batch:** an allow removed without its cure; a blob comparison passed off
as a delta check; a rune whose category the vocabulary gate does not know; a missing record line; a typed
trailer or paraphrased subject; any `refs/original/` entry; any `pulsare_yield`.
