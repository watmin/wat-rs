# BRIEF 7m ADDENDUM — #388: `--check` is a UNIT checker here, and the cure narrows to say so

**Read `BRIEF-7m-replay-batch-4m.md` first.** This addendum resolves the STOP the executor raised at #388.
It supersedes only what it names. The STOP was correct: reporting it rather than inventing an answer is
exactly what the tier asks for.

## The defect, as measured

#388's cure adds an entry-point check to the CLI's `--check` branch:

    if let Err(m) = crate::freeze::validate_user_main_signature(&world) { … return ExitCode::from(1) }

`validate_user_main_signature` errors when `:user::main` is **absent**, not merely malformed. So
`wat --check <file>` now refuses every file that is not itself a runnable program. Measured on this tree:

- `scripts/replay/census.sh` → **STOP-8, 1052 of 2165 tracked `.wat` files** flip rc 0 → rc 1, all with
  the identical `MainSignatureError {":user::main not defined — a wat program needs an entry point"}`.
  **823 under `tests/`, 103 `wat-scripts/`, 89 `wat-tests/`, 27 under `wat/` — the stdlib itself.**
- **445 of 600** sampled tracked `.wat` files declare no entry point. That is the normal shape of this
  corpus, not an edge case.
- The `cargo nextest` floor does **not** see it: the change is in `run_with_args`'s `check_only` branch,
  while `every_wat_scripts_file_loads…` and `every_docs_wat_loads_or_declares_why_not` call
  `startup_from_source` directly. Only tools that shell out to the binary see it.

## Why this is a divergence and not a defect to import

**The two questions are different, and this tree separates them deliberately.**

- `startup_from_source` / `startup_from_file` — the library driver — loads, type-checks and freezes a
  translation unit. **It does not require an entry point.** Batch 4k's #360 gate rests on exactly this:
  its own header records that the binary and `startup_from_file` give *opposite* verdicts, and grok
  withdrew a draft for using the wrong one.
- The run path EVALs `:user::main`, so it requires one.
- `--check` is the CLI face of the **library** driver. The root `CLAUDE.md` — injected into every session
  and every rider — documents it that way: *"`target/release/wat --check <f.wat>` (~0.2s) … `--check` to
  see whether it type-checks"*, for macro debugging on arbitrary files. `scripts/green-gate.sh` and
  `scripts/replay/census.sh` both depend on it.

**Grok broke this for itself and never saw it.** In a 400-file sample of grok's own tip, **307 lack
`:user::main`**. Grok has no whole-corpus `--check` census, and its mode-parity suite exercises three
curated fixtures, so the corpus-wide consequence never appeared in its own strike docs. **Grok never
revisits the `check_only` branch again** — measured across every commit from #388 to its tip — so this is
permanent there, and unexamined.

## THE RULING — 4-YES, 2026-09-17, option B

**Keep grok's cure where it is right; narrow it where it conflates the two questions.**

1. **Land the `RLIMIT_STACK` hoist VERBATIM.** Hoisting it above every mode return is unambiguously
   correct and it cures the real LIVENESS defect measured here (`--check` SIGABRT 134 on a program the
   run path completes rc 0). Nothing about this half is in dispute.
2. **Narrow the entry-point check to a DECLARED entry point.** In the `check_only` branch, run
   `validate_user_main_signature` **only when the world declares `:user::main`**. A declared-but-wrong
   entry point (bad arity, bad parameter types, bad return type) must still fail `--check` — that is
   grok's stated motive, *"a bad main must not pass `--check`"*, and it is kept. A file with **no** entry
   point stays a unit, and `--check` accepts it.
   ⛔ Find the real predicate for "declares `:user::main`" in `freeze.rs`/`FrozenWorld` — do not
   pattern-match the error message string.
3. **Adapt #387's SOUNDNESS arm to this tree's semantics, and say so in its own text.** Grok's arm reads
   "`--check` Accepted ⇒ the run path does not Reject". Here, absence of an entry point is a MODE
   difference, not an unsoundness: `--check` answers "does this unit type-check", the run path answers
   "is this a program I can start". Restate the arm as: **`--check` Accepted ⇒ the run path does not
   reject it for a reason `--check` could itself have seen** — with the missing entry point named as the
   one excluded reason, and why. Keep `mode_parity__empty.wat` as the fixture that PINS the documented
   difference (`--check` rc 0, run rc 4), never as a claimed violation.
4. **Add the fixture that proves the kept half**: a file declaring a MALFORMED `:user::main` must be
   rejected by `--check` (rc 1) and by the run path. If one already exists in `tests/cli/`, use it and
   say which; otherwise add one beside the parity fixtures.
5. **Both banked arms from #387 are un-ignored at #388** and must pass under the narrowed cure, as the
   brief's ruling already requires. `mode_parity` runs with 0 ignored at #388.

## How to land it

- The adaptation lands **AT #388**, not as a later repair, and #388's body states: the measured STOP-8
  (with the 1052/2165 figure and the per-directory breakdown), which half of grok's cure was kept
  verbatim, which half was narrowed and why, and that grok's own tip carries the unnarrowed form.
- Re-run `scripts/replay/census.sh` at #388. **It must return to no STOP-8**, and the `census:` verdict
  line must say so truthfully — never a phrase engineered to satisfy the gate's substring.
- Re-run the walls at #388 (lint-subset, `kind(lib)`, doctest) and `-E 'test(mode_parity)'` (N > 0, 0
  ignored, all green).
- ⛔ If the narrowed cure does NOT clear the census — if some other class of rc 0 → rc 1 remains — that is
  a different finding: **STOP again and report it** with the verbatim list and its categories.

## Then continue #389 → #400 under BRIEF-7m unchanged

The rest of the batch is untouched by this ruling. #398's new gate is still predicted green here
(measured: zero of our six `wat/rete/oracle/*.wat` trips its ban) — read what it actually reports.

## Rows this affects

`EXPECTATIONS-7m` is **not amended** (finding 34 — a contract is not edited after its results are seen).

- **E4** (#388 un-ignores both arms and they pass) is now satisfied by the NARROWED cure; the SCORE must
  show the rc values before and after.
- **E17** gains a required disclosure: the narrowing, its measurement, and the divergence from grok.
- **E20** scores this STOP itself as an honest deviation reported — it was.
- A new row the orchestrator will check: **the census at #388 is clean, and the `census:` line says what
  is true.**

## For the record

This is the second deliberate, permanent divergence from grok's branch in this replay, after #324's
`:then`-match fence. Both share a shape: grok's change is correct against grok's tree and collides with a
semantic this tree established first and depends on. Both are landed at the step, disclosed in the body,
and recorded in `FINDINGS-composition.md` — this one as **finding 40**, which the SCORE must write:
*a cure that conflates two questions the host tree keeps apart breaks the tool that answers the other one,
and the floor cannot see it because the floor never shells out.*
