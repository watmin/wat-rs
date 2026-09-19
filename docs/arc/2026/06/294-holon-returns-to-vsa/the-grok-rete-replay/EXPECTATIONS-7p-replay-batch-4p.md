# EXPECTATIONS 7p — replay batch 4p, grok-rete #441 → #460 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended. Fixed now so the result cannot move
the goalposts.

## ⚠ Which rows were PRE-FLIGHTED, and which are predictions

A row is RUN before it is demanded (finding 34 §2). Stated honestly:

**Executed against HEAD `5602dfdc4` and ALL PASS:** E10 (origin an ancestor; `refs/original/` empty), E11
(0 replace refs), tree clean, 440 REPLAY steps, E2 (11 docs-only), E2b (9 code), E5 (zero `.wat`, zero
`wat/`, zero hazard rows, no new `tests/lint/` gate, no `wat-scripts/fixes/` edit, every touched non-docs
path present here), **the net `#[test]` delta is 0**, **our copy of the census-name gate is byte-identical
to grok's pre-image**, the name-exposure table in the brief, and the divergence counts (#444 4-of-5, #442
3-of-6, #453 3-of-4).

**Predictions, not verified rows:** E3–E4, E6–E9, E12–E18.

⛔ **One thing I could NOT measure and did not pretend to.** Whether any cost test in THIS tree reads a
name #455 deletes or #459 renames — grok's readers are its own, and ours have diverged before. The table
says our `tests/` exposure is 0 for both, but `src/rete/kernel/tests/` is where cost tests live and those
files are `src/`. **E4 scores running the gate after each name change and reporting what it says.**

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh 5602dfdc4 HEAD 441 460` | exit 0; `sources match` |
| E1b | ⛔ **every subject is GROK'S, verbatim** (finding 39) | per step vs `git log -1 --format=%s <C>` | 20 of 20, kind included |
| E1c | **trailers COPIED, not typed** | per step vs `commits.tsv`, two-sided | 20 of 20 |
| E2 | docs-only steps are docs-only | the **11** (#441 #443 #445 #447 #448 #449 #452 #454 #456 #458 #460) | only `docs/`/`.md` |
| E2b | **and the inverse** | the 9 code steps | each carries ≥1 non-docs file |
| E3 | ⛔ **the test-count delta really is ZERO** | `git diff` over the range + the floor | no `#[test]` added or removed; **5856 run, 24 skipped** at the tip. A different number is a result to report, not to smooth |
| E4 | ⛔ **the census-name gate ran after EVERY name change** | the SCORE | `-E 'test(census_name_read_by_a_cost_test_is_emitted)'` after #442 #444 #446 #453 #455 #459, N > 0 each, with its verdict quoted — not assumed from the previous step |
| E5 | ZERO hazard paths; the range stays out of `wat/` | `git diff --name-only` over the range | no `.wat`, no `wat/`, no `wat-scripts/fixes/`, no `absent-on-main.tsv` row |
| E6 | **a retired name left no orphan reader** | the SCORE | for `prod:record-alloc`, `prod:vec-alloc` and `seed:mixed-class-activate`: every remaining reader in `src/` and `src/rete/kernel/tests/` is named and dispositioned — converted, or runed with this tree's `rune:lint(census-name-retired)` idiom and a reason |
| E7 | **the deltas landed, not the blobs** (finding 36) | the SCORE | for each conflicted step, the comparison is delta-vs-delta |
| E8 | finding 33's class swept per code step | the SCORE | explicit per step; "not applicable" is an answer, silence is not |
| E9 | the checkpoint | `scripts/floor.sh` + clippy `-D warnings` | green; clippy 0 — **the orchestrator's row** |
| E10 | **NO PUBLISHED HISTORY REWRITTEN** | `merge-base --is-ancestor`; `for-each-ref refs/original/` | ancestor YES; empty |
| E11 | repairs visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | 0 refs; exit 0 either way |
| E12 | **every `census:` line is TRUE** | `scripts/replay/census.sh --diff` at the tip | `no STOP-8` |
| E13 | **the SCORE discloses what BOUGHT each green** | read the SCORE | every rune, conversion and re-composition in the row it affects |
| E14 | no verdict line WRAPPED | grep each pattern | one-line matches |
| E15 | every `-E` filter selected N > 0 | each run's own `N tests run` | N > 0 |
| E16 | **every deviation from the brief is REPORTED** | the SCORE | honest disagreement passes; agreement does not earn it |
| E17 | **NO COUNTERPART ACTIVITY** | `.floor/`, `git status`, `.pulsare/` mtimes | none; frozen root untouched; no `mcp__pulsare__*` call |
| E18 | **a timing red was never re-run into green** | the SCORE | if a cost test reds at #442/#444/#453, the verbatim block and the arm are in the SCORE and the batch STOPPED |
| E19 | no knowingly-red commit | every step green at its own landing | none |
| E20 | **commit messages survived their heredoc** | `git log --format=%B` per step | no eaten backtick spans |

## ⛔ E3, E4 and E6 are the rows that catch a lazy census campaign

A renamed counter is invisible until something reads the old name, and `unwrap_or(0)` answers "absent"
and "zero" identically. The cheap way through is to land the renames and trust the floor. **The SCORE
shows the gate's verdict after each name change, and the disposition of every orphaned reader — or those
rows fail.**

## ⛔ Every `-E` filter must select a NON-ZERO count

A filterset matching nothing runs zero tests and **exits 0**.

**Runtime prediction:** ~70–110 min — the lightest batch in a while. No `.wat`, no corpus work, no new
gate; the weight is the six name-change steps and their conflicts.

**Trap doors named in advance:**
- **Zero net tests** — verify it rather than inherit it.
- **#455 deletes two counters; #459 renames one.** Run the census-name gate after each.
- **Wall-clock cost tests** are touched at #442/#444/#453 — a timing red is the 4i class.
- **Conflicts**: compose, compare deltas not blobs.
- **Subjects and trailers copied from git; heredocs quoted; messages read back.**

**What would make me reject the batch:** a retired census name still read by a cost test; a rune without a
reason or a cited commit; the census-name gate run once at the end instead of after each change; a timing
red re-run instead of reported; a paraphrased subject or typed trailer; any `refs/original/` entry; any
`pulsare_yield`.
