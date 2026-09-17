# EXPECTATIONS 7k — replay batch 4k, grok-rete #341 → #360 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended. Fixed now so the result cannot move
the goalposts.

## ⚠ Which rows were PRE-FLIGHTED, and which are predictions

A row is RUN before it is demanded (finding 34 §2). Stated honestly:

**Executed against HEAD `5ba45a81f` and ALL PASS:** E16 (origin an ancestor; `refs/original/` empty),
E17 (0 replace refs), tree clean, 340 REPLAY steps, E2 (13 docs-only), E2b (7 code), E8 (0 `wat/` or
`wat-scripts/fixes/` paths), E9 (23 `.wat`, none under `wat/`), **0 M-status-absent paths that are not
created earlier in the same range**, #352's three `QUARANTINE` paths all **present**, corpus counts
**284** (#352, after quarantine) and **290** (#360) against floors of 200, and **0 of our 31 main-only
`.wat.bad` carry a `banked-by:` rune**.

**Predictions, not verified rows:** E3–E7, E10–E15, E18–E23.

⛔ **One thing I could NOT measure and did not pretend to.** How many of our 31 main-only `.wat.bad`
return `Ok` under `startup_from_file` is **unknown** — the gate does not exist yet, and the gate's own
header says the binary and `startup_from_file` give *opposite* verdicts. A `--check` proxy suggested 11,
but `--check` is a third driver again. **E4 scores measuring it properly, not matching my hint.**

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh 5ba45a81f HEAD 341 360` | exit 0; `sources match` |
| E2 | docs-only steps are docs-only | the **13** (#341 #343 #345 #346 #347 #348 #350 #351 #353 #354 #356 #358 #359) | only `docs/`/`.md` |
| E2b | **and the inverse** | the 7 code steps (#342 #344 #349 #352 #355 #357 #360) | each carries ≥1 non-docs file |
| E3 | **#360's gate is GREEN** | `-E 'test(every_wat_bad_fixture_actually_fails)'` | all shards pass; corpus floor ≥200 holds |
| E4 | ⛔ **the mis-named set was MEASURED with the right driver** | the SCORE | states how many of the 31 main-only `.wat.bad` returned `Ok` from **`startup_from_file`** — not the binary, not `--check`. **This row passes on a real number, even if it differs from the brief's hint** |
| E5 | ⛔ **each mis-named file was DISPOSITIONED deliberately** | per file, in the SCORE | renamed to `.wat` / banked with a rune / reported — **with that file's own test read first**, and the shape named (retired premise · starts-then-INVOKEs · asserts startup succeeded) |
| E6 | every banking rune is a DECLARATION | each `rune:lint(bad-is-banked)` added | category exactly `bad-is-banked`, reason ≥24 chars naming why the substrate *should* reject it, `banked-by:` naming a real test fn |
| E7 | **#352's gate is GREEN, all shards** | `-E 'test(diagnostic_output_is_deterministic)'` | pass; `the_determinism_quarantine_is_pinned_and_its_paths_exist` green |
| E8 | ⛔ **no fourth QUARANTINE entry was slipped in** | `git diff` on #352's gate file | `QUARANTINE_LEN` still **3**. A new determinism defect is a **finding to REPORT**, per the gate's own words — not a line to add |
| E9 | ZERO hazard paths in range | `git diff --name-only` vs `wat/`, `wat-scripts/fixes/`, `absent-on-main.tsv` | none |
| E10 | the 23 `.wat` are NEW fixtures | `git diff --name-status` | all `A`; none under `wat/`; no `wat-scripts/fixes/` edit |
| E11 | finding 33's class looked for | the SCORE | each code step answers explicitly; "not applicable" is an answer, silence is not |
| E12 | the checkpoint | `scripts/floor.sh` + clippy `-D warnings` | green; clippy 0 — **the orchestrator's row** |
| E13 | **test-count delta ACCOUNTED FOR** | predict from the diff BEFORE the floor | predicted == actual |
| E14 | spot re-run of the walls | lint-subset + `kind(lib)` + doctests + stone-3 vs the last code step's lines | identical |
| E15 | no knowingly-red commit | no repair commit after #360 | none |
| E16 | **NO PUBLISHED HISTORY REWRITTEN** | `merge-base --is-ancestor origin/replay/grok-rete HEAD`; `for-each-ref refs/original/` | ancestor YES; empty |
| E17 | repairs visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | 0 refs; exit 0 either way |
| E18 | no verdict line WRAPPED | grep each pattern | one-line matches; `--diff`/`no STOP-8` contiguous |
| E19 | **the SCORE discloses what BOUGHT each green** | read the SCORE | every rename, bank, rune or fixture change appears in the row it affects |
| E20 | every `-E` filter selected N > 0 | each run's own `N tests run` | N > 0 |
| E21 | **NO COUNTERPART ACTIVITY** | `.floor/` for foreign runs; `git status` for foreign artifacts; `.pulsare/` mtimes | none; frozen root untouched |
| E22 | **every deviation from this brief is REPORTED** | the SCORE | honest disagreement passes this row; agreement does not earn it |
| E23 | **a `.wat.bad` was never renamed merely to silence a gate** | the SCORE + diffs | every rename is justified by the file's own test, quoted |

## ⛔ E4, E5 and E23 are the rows that catch a lazy #360

The cheap way to green this gate is to rename every offender to `.wat` and move on. That would be wrong
in exactly the way grok's own strike warns about: **three of its 16 were NOT mis-named** — their tests
assert `is_err()` and are `#[ignore]`d. A rename there destroys a real negative fixture. **The file's own
test decides, and the SCORE shows the reading.**

## ⛔ Every `-E` filter must select a NON-ZERO count

A filterset matching nothing runs zero tests and **exits 0**.

**Runtime prediction:** ~110–160 min. The weight is #360 (32 files, 14 `.wat`, 9 `.rs`, plus dispositioning
however many of our 31 main-only fixtures start clean), then #344 (16) and #349 (13).

**Trap doors named in advance:**
- **#360 — measure with `startup_from_file`, never the binary or `--check`.** grok withdrew its own first
  draft for using the wrong driver.
- **#360 — 31 main-only fixtures have never faced this check, and none is banked.** Rename, bank, or
  report; read each test first.
- **#352 — `QUARANTINE_LEN` is pinned at 3.** A fourth defect is a finding.
- **#342 is a FINDING, not a fix** (D10 is live); **#344 is its cure.** Do not pull #344 backward.

**What would make me reject the batch:** a `.wat.bad` renamed to silence the gate rather than because its
test says so; a banking rune with a shrug reason or a `banked-by:` naming no real test; a fourth
`QUARANTINE` entry added silently; the mis-named count measured with the binary or `--check`; any
`refs/original/` entry; a fabricated trailer SHA; any `pulsare_yield`.
