# BRIEF — STONE 255.73: repair the rotted probe, and make it run

**Drawn 2026-10-01 against `main` @ `e84a58b14`.** **Executor: a Sonnet subagent.** A small strike. Commit locally on
`main` (`git add -- <paths>`, never `-A`); **do not push**. Your final message is your report.

## Why (builder, 2026-10-01)

*"repair the rotted probe — the large accumulated test corpus continues to rescue us."* 255.72 found that
`wat-scripts/probes/arc-170/probe-s3b-astsplice.wat` crashes at run time, before and after that stone, at line 59:

```
#wat.runtime/MalformedForm … "malformed :wat::core::keyword-node form: angle-bracket type parameters are illegal in a
name (arc 109, "annihilate the angle bracket") …" … :line 59 :col 16 … :line 63 :col 61 … :head ":wat::core::keyword-node"
```

It rotted silently because **nothing runs it**: the loader gate (`every_wat_scripts_file_loads`) only type-checks
`wat-scripts/` files. Read `WEIGH-STONE-255.72-the-wall-has-no-exceptions.md` and `SCORE-STONE-255.72-…` first.

## The work

1. **Learn what the probe proves.** Read its header and its arc-170 brief/score (search `docs/arc/` for the probe's name)
   so the repair keeps its claim. Report the claim in one sentence.
2. **Repair it in today's language.** Line 59-63 builds a parameterized type **name** by string concatenation (an angle
   bracket in a keyword), which arc 109 retired. Build the type as a **type form** instead (`(Head :- [args])`, the
   way 255.72's site 1 splices `~ret-ty`). Fix whatever else the run then reaches, the same way: today's spelling, no
   retired form. Keep the probe's claim; do not weaken what it checks.
3. **Make it run.** Wire it into a driven Rust test (copy how another `wat-scripts/probes/` file, or a `tests/` fixture, is
   run and asserted: `call_beside_value`, `startup_from_file`, or an EDN golden through `assert_edn_matches_file!`), so
   the floor runs it and asserts its result. A rot would then be a red.
4. **Census the sibling probes (report only, change nothing else):** how many `wat-scripts/probes/**/*.wat` files does no
   test **run** (they are only loader-checked)? Of those, how many crash when run with `./target/release/wat` today?
   List the crashing ones with their first error. This is the measurement for a follow-up the builder decides.

## Gates

| what | how | expected |
|---|---|---|
| the probe runs | its new test, and `./target/release/wat <probe>` | rc 0, the probe's claim holds |
| release floor | `scripts/floor.sh`, one run at a time, in the foreground, nothing else running | all passed; 6235 at `4969e1907`, plus your test |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

## Reds and STOPs

- A red caused by this stone's own change: capture it verbatim from `.floor/<stamp>/`, cure it, run a new floor. Never
  re-run unchanged code for a green.
- **STOP-1:** the probe's claim cannot be kept in today's language (the thing it proved no longer exists or means
  something else). Report it, with the evidence, and STOP.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you. This is one file's repair; edit it directly (a codemod is for corpus-wide migrations). Capture
`rc=$?` on the next statement. Never wait with `pgrep -f`. Never write a number, file:line or example you did not measure.
If this brief contradicts the code, the code wins: say so. Write `SCORE-STONE-255.73-repair-the-rotted-probe.md` beside
this brief, commit it, **do not push**.
