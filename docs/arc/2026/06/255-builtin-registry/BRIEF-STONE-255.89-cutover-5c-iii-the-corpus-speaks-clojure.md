# BRIEF — STONE 255.89: cutover 5c-iii — the corpus speaks faithful Clojure

**Drawn 2026-10-03 against `main` @ `f9ff0e930`.** **Executor: grok via pulsare, working solo** (it runs the floor).
Every tracked `.wat` outside `wat/` converts to symbol heads by the recorded converter; the reds this causes are cured
by mechanism. Commit locally on `main` (`git add -- <paths>`, never `-A`; `git status` clean before the floor you
report); **do not push**.

## Where this sits (H2)

5c-ii converted the stdlib (255.87, 65 files, 11,131 heads → 0); 255.88 made the first slash the only partition.
**5c-iii** (this) converts the corpus. 5c-iv converts the wat embedded in Rust strings; 5d makes keyword heads illegal.
Until 5d both spellings are accepted.

## What is known (orchestrator, measured 2026-10-03)

| | |
|---|---|
| the corpus | `git ls-files '*.wat' \| grep -v '^wat/'`: **2228** files (255.85 measured 2219; ~10 added since). Largest: `wat-scripts/scratch-pad` 439, `tests/types` 398, `tests/rete` 158, `wat-scripts/fixes` 124, `tests/services` 95 |
| 255.85's re-measure | a fresh clone at `e935b3093`: **2219 OK, 0 FAIL**, 86,593 heads → 0, 56 batches, ~657 s summed. "OK" means *converted without error*, not *behaves the same*: that is this stone's floor |
| the census | `.census/2026-10-03T13-06-27Z.txt`: 2293 rows (corpus + `wat/`), **210 non-zero** |
| the floor | `.floor/2026-10-03T13-09-11Z` at `30cd8b69c`: **6412 passed / 24 skipped**; HEAD since is docs-only |
| the converter | `wat-scripts/fixes/to-faithful-clojure.wat` (`SCOPE: corpus`); it has **no per-file exemption**: what it converts is exactly the path list it is handed |
| recorded codemods | `wat-scripts/fixes/*.wat` are programs **and** carry old spellings as data. 11 of them hold quoted or quasiquoted forms (`grep -lE "quasiquote\|:wat::core::quote\|'\(:" wat-scripts/fixes/*.wat`). Their replays (`fixes/replay/<stem>/{pre,post,ORACLE}`, not `.wat`, untouched) are gated by `tests/cli/every_recorded_migration_replays.rs` |
| a retired-form fixture | 255.85 noted the converter rewrites the retired `:wat::core::Tuple(i64)` keyword in an old fixture to `wat.core/Tuple(i64)` instead of leaving it (it does leave `:fn(`) |

## The work

1. **Pre-census:** `scripts/replay/census.sh` on the unconverted tree, kept as the baseline.
2. **The KEEP list, first, as its own commit:** `wat-scripts/fixes/to-faithful-clojure.keep` (one path per line, each
   followed by `;; <reason>`), naming every corpus file that must stay in keyword spelling, and **why**. A file belongs on
   it only when its purpose is the old spelling itself:
   - a negative proof of a retired keyword form (the `:fn(` fixtures, the `:wat::core::Tuple(i64)` fixture, and any
     like them: find them by asking which fixtures assert a refusal of a keyword form);
   - a recorded codemod whose **matching data** is an old spelling: the conversion would change what it matches. The
     replay gate decides this, not a reading: a codemod whose replay changes after conversion goes on the list (its
     replay goldens are never re-captured to fit).
   Everything else converts. Report the list's size by reason. **The list is 5d's input** (a kept file will not load
   once keyword heads are illegal); this stone only records it.
3. **Convert** every corpus path not on the KEEP list, with **a copy of the pristine binary and a copy of the codemod
   taken out of the tree first** (the tool is never its own input; the converter's own file is just another target).
   Batch as 255.85 did. Report heads before and after (total and by top-level directory) and any file the converter
   refused. Idempotent: a second run changes nothing. Commit the conversion **as its own commit**, only the codemod's
   output, so its diff is reviewable apart from the cures.
4. **Build, census, floor.** `census.sh` again; list every rc change **in both directions** (`0 → non-zero` = NEW,
   `non-zero → 0` = RECOVERY) by file. Then the floor. Group every red **by mechanism** (not by test), one verbatim block
   per mechanism.
5. **Cure by mechanism**, each in its own commit after the conversion commit, each with the probe or test that names it.
   A cure is a door, a dispatch arm, a reader, a gate's input, or a converter rule (then the conversion is re-run on the
   affected files and that output committed separately): never a second spelling, never a hand-edit of converted
   output. A golden whose only change is a printed spelling is re-captured only after showing, **as data**, that it is the
   old value with the spelling changed (255.87's audit is the shape). A Rust test that reads a fixture's **text** and
   breaks because the text changed is a reader to cure (read it by identity), not a fixture to revert.

## Gates

| what | how | expected |
|---|---|---|
| KEEP list | the committed file | every entry has a reason of one of the two kinds; sizes reported |
| conversion | the codemod over the corpus minus KEEP; again | heads → 0 outside KEEP (each remaining one listed and why); second run 0 changes |
| census | `census.sh` pre and post | NEW 0 and RECOVERY 0, or each one explained by file and mechanism (a RECOVERY is a refusal that stopped refusing: a STOP unless it is a defect this stone cured) |
| replay | `every_recorded_migration_replays` in the floor | green with no replay golden changed |
| release floor | `scripts/floor.sh`, in the foreground, nothing else running, `git status` clean | all passed; the count against 6412 at `30cd8b69c`, plus your tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| ignores | the SEAM's ledger command | 18 |

## Reds and STOPs

- A red caused by this stone is expected and is the work (item 5). Capture each mechanism **verbatim** from
  `.floor/<stamp>/`; cure it; run a **new** floor. Never re-run unchanged code for a green.
- **STOP-1:** a cure would change what an **unconverted** program does (a keyword-spelled program's value, error or
  admission). Describe it and STOP on that mechanism.
- **STOP-2:** a mechanism needs a ruling (a name with no single canonical identity, a form whose converted spelling
  means something else, a declaration two spellings both claim, a file that fits neither KEEP reason yet cannot
  convert). Describe it and STOP on that mechanism.
- **STOP-3:** more than 12 distinct mechanisms after the first floor. Report the grouped table and STOP before curing,
  so the cure can be split.
- A STOP means STOP. If one fires, finish the cures that do not depend on it first.

## Doctrine

`holon/CLAUDE.md` binds you: `.wat` moves only by the recorded codemod; a converted file is never hand-edited. New
fixtures are written in the target spelling. No time limit is raised; no source is edited to fit a test; no golden is
re-captured without the as-data audit; equality is data, never a string compare. Capture `rc=$?` on the next statement.
Never wait with `pgrep -f`. Never write a number, file:line or example you did not measure. If this brief contradicts the
code, the code wins: say so. Write `SCORE-STONE-255.89-cutover-5c-iii-the-corpus-speaks-clojure.md` beside this brief,
commit it, **do not push**.
