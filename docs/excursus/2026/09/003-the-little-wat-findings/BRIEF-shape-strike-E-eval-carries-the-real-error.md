# BRIEF — shape strike E: the eval family's failure is the real error, frames and all

Excursus 003. Design: `AUDIT-the-shape-of-an-error.md` F4 (read it with § CORRECTION 2026-10-04,
which re-measures F4's blast radius), § RULING 2026-09-27 item 3, and the D2 row on
`eval-ast!`/`eval-step!`. This builds on `f2d4375ca`.

**Read `wat-rs/CLAUDE.md` in full first.** Every citation below is a claim. Verify each one before
relying on it, and STOP on a contradiction. Where the call is small and the intent is clear, make it
and name it. Decide anything the brief leaves open by the four questions (Obvious? Simple? Honest?
Good UX?, each a flat YES or NO) and show your answers. **Never present options.**

## Why

`:wat::core::EvalError` (`wat/core.wat:~2208`) is `{kind <- String, message <- String}`. It predates
the declared error taxonomies, and it now duplicates them lossily: `runtime_error_to_eval_error_value`
(`src/runtime.rs:~12476`) flattens every `RuntimeError` into a kebab-case string plus a sentence.
The error's own record, its fields, its `location` and its `frames` are all discarded.

D2 found the cost from below. `BadCondition`, `PatternMatchFailed`, `EffectfulInStep` and
`NoStepRule` are reachable only through `:wat::eval-ast!` / `:wat::eval-step!`. They raise with a
correctly named frame, then the frame is thrown away before anything can observe it.

## The measured blast radius (corrected; verify)

Outside `wat-scripts/scratch-pad/` there are **32 mentions** of `EvalError`. The consumers:
- `wat/eval.wat:~105`, `FormOutcome.Raised [cause <- :wat::core::EvalError]` (the REPL);
- `wat/doctest.wat:~132,~140`, which reads only `EvalError/message`;
- `tests/value/wat_eval_result.wat`, which reads `kind` and `message`;
- the check schemes for the eval verbs (`src/check.rs:~20047,~20075`, `Result<T, EvalError>`);
- the Rust builder and its readers (`src/runtime.rs:~12476,~12550,~12562,~16056`);
- the registration (`src/types.rs:~1861`) and the Stone-Q census row (`:~8570`).

`wat-scripts/scratch-pad/` holds nearly all of F4's `kind` reads: 16 stone-probe files. Those files
are gated (they must load), so they move with the change.

## Target

1. **An eval-family failure carries the real `:wat::core::Error`.**
   - The eval verbs' `Err` holds the error that happened: the runtime record (strike 3a), or the
     check, resolve, parse or load record (sweeps S1–S3), with its own `location`, its fields, and
     the `Failure`'s frames wherever a failure carries them.
   - Do not flatten, and do not re-describe.
   - Find every eval verb: `eval-ast!`, `eval-step!`, and any `eval-edn!`/`eval-digest!`/`eval-signed!`
     still in the tree (`wat/core.wat`'s comment names those). Measure which exist.
2. **Decide the `Err` type by the four questions** and show your answers.
   - Either the `Err` is the error itself (`:wat::core::Error`, the surface), or it is a record that
     carries the error together with the frames the eval produced (for example a `Failure`, the
     shape a death already uses).
   - The deciding fact is whether frames have a home: `:wat::core::Error` is `{message location}`,
     and frames live on `Failure`. Measure what the eval path holds when it raises, and choose so
     that nothing raised is discarded.
3. **`EvalError` retires.**
   - Delete the declaration, its registration, its Stone-Q census row, `runtime_error_to_eval_error_value`,
     and every `kind` string.
   - Consumers move to the new shape:
     - `doctest.wat` reads `Error/message`, or whatever item 2 decides;
     - `FormOutcome.Raised`'s `cause` takes item 2's type;
     - the check schemes follow.
4. **Branching on the kind of failure is by class, not by string.**
   - `tests/value/wat_eval_result.wat` and the scratch-pad probes branch on `EvalError/kind` strings.
   - Measure how wat can ask a record its class today (a `match` on the record's type, or a class
     accessor). Use that.
   - If no honest way exists, STOP and report it. Do not invent a string accessor that reproduces
     `kind`.
5. **The scratch-pad probes.** Per `wat-rs/CLAUDE.md`, scratch either conforms or is deleted if it
   is truly dead.
   - Measure each of the 16 probes: is it a reference something cites, or a dead stone's scratch?
   - Migrate the live ones with a wat-fix codemod, with a dry-run diff and a replay fixture.
   - Delete the dead ones, naming each in the commit.

## Gates (each mutation-proven in RELEASE)

- **GE1, nothing raised is discarded.** For each of `BadCondition`, `PatternMatchFailed`,
  `EffectfulInStep` and `NoStepRule`, drive its real producer through `eval-ast!`/`eval-step!`.
  Assert that the `Err` carries that kind's declared record class, its fields, and the frames,
  including the named Rust activation (D's frame shape).
  - Mutation: restore the flattening. RED.
- **GE2, a check failure inside eval is typed.** An `eval-ast!` of a form that fails to type-check
  yields the check record (S1), not prose.
  - Mutation: restore the flattening. RED.
- **GE3, `EvalError` is gone.** Its name resolves nowhere: drive a lookup, do not grep. The Stone-Q
  census row is updated.
- The existing gates stay green.

## Goldens

Recapture with `UPDATE_EDN=1`, and **read every diff**. The only allowed change is an eval failure
moving from `{kind message}` to the real error and frames. Report anything else.

## Scope fence

- **IN:** items 1–5 and the gates.
- **OUT:**
  - the D2 census and the `Option` decision (next, after E);
  - F (the domain `Fault`s);
  - the post-F strikes (retire provenance, the startup-message type, declarable `char`,
    `LoadOther`, `ParamShadowsBuiltin`);
  - the stdlib-freeze excursus.

## Discipline

- **`cargo nextest run --release` ONLY, never `cargo test`.** One cargo process at a time: check with
  `pgrep -x cargo` and `pgrep -x cargo-nextest`, never `pgrep -f`.
- **Never use a git worktree with a shared `CARGO_TARGET_DIR`.**
- **Never edit files under `tests/`, `wat/` or `wat-scripts/` while any run is in flight.**
- **Never hand back while a build, recapture or floor runs.** Wait in the foreground with
  `until grep -qE '^ *Summary' <log>; do sleep 30; done`, repeating past the tool's 10-minute cap.
- Iterate with targeted runs and `cargo wat` dry-runs. Run the full floor at the commit.
- The floor:
  - Run `scripts/floor.sh`: 0 failed, 0 timed out.
  - On any red: surface it before any re-run, verbatim, with the arm named.
  - **Never commit a red floor.** "Unrelated to my change" is not a disposition.
- Stage by name BEFORE the floor. Never use `git add -A`. No `cargo fmt`.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`. Push to
  `origin/reason/little-wat-findings`.
- **If your budget runs short, stop at a clean boundary and report where.** "Item 2 decided, plus the
  Rust and eval path carrying the real error with GE1 and GE2, on a green floor, before the
  scratch-pad migration" is a valid boundary.

## Report

- the eval verbs found;
- item 2's decision and its four answers;
- the class-branching mechanism;
- the scratch-pad census (each probe kept, migrated or deleted, with its reason);
- each gate's mutation RED;
- any golden that changed beyond the allowed change;
- the floor `Summary` line, verbatim;
- the SHA(s).
