# BRIEF — STONE 255.87: cutover 5c-ii — the stdlib speaks faithful Clojure

**Drawn 2026-10-03 against `main` @ `76cc805ed`.** **Executor: grok via pulsare, working solo** (it runs the floor). The
65 `wat/*.wat` stdlib files convert to symbol heads by the recorded converter; the reds this causes are cured by
mechanism. Commit locally on `main` (`git add -- <paths>`, never `-A`; `git status` clean before the floor you report);
**do not push**.

## Where this sits (H2)

5a refused unbound heads; 5b made every head decision go through its identity; **5c-i** made the tooling and the names
ready (255.85, 255.86). **5c-ii** (this) converts the stdlib. 5c-iii converts the corpus, 5c-iv embedded wat, 5d makes
keyword heads illegal. Until 5d both spellings are accepted, so a converted stdlib must load and run beside an
unconverted corpus.

## What is known (read first)

- `SCORE-STONE-255.84-…` § 3: the clone's converted stdlib **built and loaded**; its floor was 2391/6389, with 3873
  panic sites one startup error (`not a member of wat.type: :wat::type::Error`): the converter's type rule, **cured by
  255.85**. Seven gates that read stdlib text were blind on the converted spelling: **cured by 255.85** (green on a
  converted clone). § 6: the `DuplicateMacro` on `wat/holon/Ngram.wat` is symmetric across which side is converted.
- The 8d-ii line (`251-types-as-forms/BRIEF-STONE-251.8d-ii-*` and their WEIGHs) and the SEAM's 8d sections: every reason
  the stdlib conversion stopped before, eight draws. Several were cured since (5a, 5b, 255.85, 255.86); measure, do not
  assume.
- **`wat/` is `include_str!`'d** (`src/load/stdlib.rs`): a converted `wat/` is live only after `cargo build --release`.
  **`wat/fix.wat` is the tool:** convert it with a pristine binary built **before** the conversion, never the tool over
  itself (`wat/fix.wat`'s STASH-DANCE note; `[[feedback_a_tool_is_never_its_own_input]]`).

## The work

1. **Pre-census:** `scripts/replay/census.sh` and `scripts/replay/delta.sh` (the committed 179-file sample) on the
   unconverted tree, kept as the baseline.
2. **Convert** all 65 `wat/*.wat` with `wat-scripts/fixes/to-faithful-clojure.wat` run by a pristine binary (copy it out
   of `target/` first). Report heads before and after per file, and any file the converter refused. Idempotent: a second
   run changes nothing. Commit the conversion **as its own commit** (only the codemod's output), so its diff is
   reviewable apart from the cures.
3. **Build, then floor.** Group every red **by mechanism** (not by test) with one verbatim block per mechanism.
4. **Cure by mechanism**, each in its own commit after the conversion commit, each with the probe or test that names it.
   A cure is a door, a dispatch arm, a reader, a gate's input: not a second spelling, not a hand-edit of converted
   output. A golden whose only change is the stdlib's printed spelling is re-captured only after showing, as data, that
   it is the old value with the spelling changed.
5. **The corpus beside it:** after the floor is green, `census.sh --diff` (no rc flips) and `delta.sh` on the 179-file
   sample run against the converted stdlib (NEW and RECOVERY listed by file and mechanism; NEW must be 0 or each one
   explained as a defect this stone cured, never a spelling difference).

## Gates

| what | how | expected |
|---|---|---|
| conversion | the codemod over `wat/`; again | heads → 0 (or each remaining one listed and why); second run 0 changes |
| census | `census.sh` pre and `--diff` after | no rc flips |
| delta | `delta.sh` on the committed sample | NEW 0, or each explained |
| release floor | `scripts/floor.sh`, in the foreground, nothing else running, `git status` clean | all passed; the count against 6397 at `fd5321bd4` (`.floor/2026-10-03T05-24-19Z`) |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

## Reds and STOPs

- A red is expected and is the work (item 4). Capture each mechanism **verbatim** from `.floor/<stamp>/`; never re-run
  unchanged code for a green.
- **STOP-1:** a cure would change what an **unconverted** program does (a keyword-spelled program's value, error or
  admission). Describe it and STOP on that mechanism.
- **STOP-2:** a mechanism needs a ruling (a name with no single canonical identity, a form whose converted spelling
  means something else, a declaration two spellings both claim). Describe it and STOP on that mechanism.
- **STOP-3:** more than 12 distinct mechanisms after the first floor. Report the grouped table and STOP before curing, so
  the cure can be split.
- A STOP means STOP. If one fires, finish the cures that do not depend on it first.

## Doctrine

`holon/CLAUDE.md` binds you: `.wat` moves only by the recorded codemod; a converted file is never hand-edited (a cure
lives in the substrate or in the converter, then the conversion is re-run). Capture `rc=$?` on the next statement. Never
wait with `pgrep -f`. Never write a number, file:line or example you did not measure. If this brief contradicts the code,
the code wins: say so. Write `SCORE-STONE-255.87-cutover-5c-ii-the-stdlib-speaks-clojure.md` beside this brief, commit
it, **do not push**.
