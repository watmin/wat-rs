# EXPECTATIONS 7m — replay batch 4m, grok-rete #381 → #400 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended. Fixed now so the result cannot move
the goalposts.

## ⚠ Which rows were PRE-FLIGHTED, and which are predictions

A row is RUN before it is demanded (finding 34 §2). Stated honestly:

**Executed against HEAD `f00eed601` and ALL PASS:** E14 (origin an ancestor; `refs/original/` empty), E15
(0 replace refs), tree clean, 380 REPLAY steps, E2 (12 docs-only), E2b (8 code), E7 (zero hazard rows;
both M-status-absent paths created earlier in the range), **#398's ban finds ZERO violations in our six
`wat/rete/oracle/*.wat`**, and both mode-parity arms measured live here with grok's own fixtures
(`--check` rc 0 vs run rc 4; run rc 0 vs `--check` SIGABRT 134 once the fixture's retired
`:wat::core::i64::+` spelling is corrected).

**Predictions, not verified rows:** E3–E6, E8–E13, E16–E22.

⛔ **One thing I could NOT measure and did not pretend to.** Whether #384's un-banked D2 test and #390's
A1 fix go green on THIS tree is unknown — main's rete has diverged before (D10/D11 at 4k were already
covered here, and the executor re-composed them as narrow fallbacks). **E5 scores measuring it, and
reporting a divergence as a result.**

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh f00eed601 HEAD 381 400` | exit 0; `sources match` |
| E1b | ⛔ **every subject is GROK'S, verbatim** (finding 39) | per step, `git log -1 --format=%s <ours>` vs `REPLAY(grok-rete #N): $(git log -1 --format=%s <C>)` | 20 of 20 identical — including the step's own kind (`fix(tests):` is not `perf:`) |
| E2 | docs-only steps are docs-only | the **12** (#382 #385 #386 #389 #391–#397 #399) | only `docs/`/`.md` |
| E2b | **and the inverse** | the 8 code steps | each carries ≥1 non-docs file |
| E3 | ⛔ **#387 landed GREEN, banked — not red, not weakened** | `git show` #387; `-E 'test(mode_parity)'` | the two live arms `#[ignore]`d with assertions INTACT and a rune naming #388; every other arm green; N > 0 |
| E4 | ⛔ **#388 UN-IGNORED them and they PASS** | `git show` #388; `-E 'test(mode_parity)'` at #388 | both ignores removed; all arms green; the cure is what bought it |
| E5 | **#384 un-banks D2 green, #390's A1 fix holds** | `-E 'test(right_index_counter_invariant)'`, #390's own tests | green, or a measured divergence reported as a result |
| E6 | ⛔ **#387's generator and fixture were RE-SPELLED** | `tests/cli/gen_mode_parity_deep.sh` + the fixture | `:wat::i64::+`, not the retired `:wat::core::i64::+`; the SCORE states the measured rc values before and after |
| E7 | ZERO hazard paths in range | `git diff --name-only` vs `absent-on-main.tsv` | none |
| E8 | **#398's gate is GREEN, and its verdict was READ** | `-E 'test(no_raw_network_keys_in_oracle)'` | green; the SCORE states what it reported over our six oracle files, not what I predicted |
| E9 | finding 33's class swept per code step | the SCORE | each code step answers explicitly; #387's `.sh` generator that WRITES wat is named |
| E10 | the checkpoint | `scripts/floor.sh` + clippy `-D warnings` | green; clippy 0 — **the orchestrator's row** |
| E11 | **test-count delta ACCOUNTED FOR** | predict from the diff BEFORE the floor | predicted == actual. My pre-flight, read off the diff: **+2** #383 (one of them banked), **+7** #387, **+2** #390, **+8** #398 = **+19**, and the two bankings are un-ignored by #384/#388, so at the tip **5827 run, 24 skipped**. The executor's own count is the check on mine |
| E12 | spot re-run of the walls | lint-subset + `kind(lib)` + doctests vs the last code step's lines | identical |
| E13 | no knowingly-red commit | every step green at its own landing | none — #387 is banked, not red |
| E14 | **NO PUBLISHED HISTORY REWRITTEN** | `merge-base --is-ancestor origin/replay/grok-rete HEAD`; `for-each-ref refs/original/` | ancestor YES; empty |
| E15 | repairs visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | 0 refs; exit 0 either way |
| E16 | **trailers are COPIED, not typed** | per step, trailer vs `commits.tsv`, two-sided | 20 of 20 match |
| E17 | **the SCORE discloses what BOUGHT each green** | read the SCORE | every banking, re-spelling, adaptation and regeneration appears in the row it affects |
| E18 | no verdict line WRAPPED | grep each pattern | one-line matches; `--diff`/`no STOP-8` contiguous |
| E19 | every `-E` filter selected N > 0 | each run's own `N tests run` | N > 0 |
| E20 | **every deviation from the brief is REPORTED** | the SCORE | honest disagreement passes this row; agreement does not earn it |
| E21 | **NO COUNTERPART ACTIVITY** | `.floor/` for foreign runs; `git status` for foreign artifacts; `.pulsare/` mtimes | none; frozen root untouched; no `mcp__pulsare__*` call |
| E22 | **a red was never re-run into green** | the SCORE | any red carries its verbatim block and its arm, and the batch STOPPED |

## ⛔ E3, E4 and E6 are the rows that catch a lazy #387

The cheap ways through #387 are to weaken an assertion until it passes, to `#[ignore]` an arm without a
rune or a measurement, or to leave grok's stale fixture in place so LIVENESS passes vacuously on 1000
type errors. All three look green from the outside. **The SCORE shows the measured rc values, the runes,
and the re-spelling — or those rows fail.**

## ⛔ Every `-E` filter must select a NON-ZERO count

A filterset matching nothing runs zero tests and **exits 0**.

**Runtime prediction:** ~110–160 min. The weight is #384 (14 files, 11 `.rs`), #390 (11 files, 9 `.rs`),
#381 (47 files, mostly docs) and the #387/#388 pair's measurements.

**Trap doors named in advance:**
- **#387/#388 — the red-then-cure pair.** Bank, do not weaken; re-spell the generator; #388 un-ignores.
- **#383/#384 — grok's own banked finding.** Do not pull the cure backward.
- **#398 — a new gate.** Predicted green here; read what it actually says.
- **Finding 39 — the subject is grok's, copied from `git log -1 --format=%s`.**

**What would make me reject the batch:** a weakened assertion anywhere; an `#[ignore]` without a rune and
a measurement; grok's stale `:wat::core::i64::+` fixture left in place; a paraphrased subject; a
hand-typed trailer; a timing or gate red re-run instead of reported; any `refs/original/` entry; any
`pulsare_yield`.
