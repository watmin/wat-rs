# BRIEF — shape strike D2: every raise names its activation, and a gate proves it

Excursus 003. This strike cures the two rows recorded in `AUDIT-the-shape-of-an-error.md`,
§ "Strike D landed". It builds on `661efff24`.

**Read `wat-rs/CLAUDE.md` in full first.** Every citation below is a claim. Verify each one before
relying on it, and STOP on a contradiction. Where the call is small and the intent is clear, make it
and name it.

## The two findings

1. **An omitted Rust frame is invisible.**
   - `Frame::rust_site` returns `Option<Frame>`, so the innermost Rust frame is **omitted** when the
     `CURRENT_ACTIVATION` slot (`src/value/frame.rs`) is empty.
   - No test asserts the frame is **present**, so the floor cannot see an omission.
   - Strike D did not build the per-kind census.
   - The orchestrator drove 5 real producers through the binary and all were named. That is a
     sample, not a proof.
2. **A symbol-headed call names the enclosing form.** `(f 1)`, with `f` a local bound to a
   non-callable, raises `NotCallable` with its Rust frame named `:wat::core::let`.
   - Only keyword-headed dispatch (`dispatch_keyword_head`, `dispatch_keyword_head_value`,
     `eval_tail`'s keyword arm, `src/runtime.rs`) sets the slot.
   - The application path for a symbol or expression head sets nothing, so the slot still holds
     the enclosing form.

## Target

1. **The application path names its own activation.** Where the evaluator applies a non-keyword
   head, set the slot, using the same RAII guard (`ActivationGuard::enter`), to the most honest name
   in hand at that point:
   - the callee's resolved function name when the head resolves to a function;
   - otherwise the head symbol as written (for example `f`), because that is what the user wrote and
     the raise is about it.

   Pick the rule by the four questions, and show the answers.
2. **The census is a standing gate, with presence asserted.** Drive every `RuntimeErrorKind` (all 40)
   through a **real producer**: a minimal wat program run through the real entry point, or the real
   freeze path for the startup kinds. Assert that the error's frames include a Rust frame whose `fn`
   is non-empty and is the activation you expect, per kind.
   - A table-driven test is fine. `tests/diagnostics/probe_excursus003_step3a_wat_records.rs` and
     T2/T3's gates show the shape.
   - **A kind with no real producer:** list it with the reason (for example, `WriteStopped`, or a
     `ReteCeiling` variant that needs a rete session). Drive it through the closest real path, or
     STOP on it. Do not hand-construct it, because a hand-built `RuntimeError` bypasses the very
     dispatch being proven.
   - **Any kind whose real producer leaves the slot empty is a finding.** Cure it by setting the
     slot at that producer's dispatch, or STOP and report.
3. **Once the census is green, decide whether `Option` stays.** If every real producer names its
   activation, the `None` arm exists only for a bare `RuntimeError::new` in a unit test, and
   production can never reach it.
   - Measure that, and decide by the four questions: keep the `Option` for that construction shape,
     or make the frame mandatory and give those unit tests an activation.
   - Show the answers.

## Gates (each mutation-proven in RELEASE)

- **GD2a, the census.** The 40-kind presence table.
  - Mutation: remove the slot-set from one dispatcher. RED, naming the kinds that lost their name.
    One mutation per writer: intrinsic dispatch, special form, application path, freeze phase.
- **GD2b, a symbol-headed call names itself.** `(f 1)` with a non-callable `f`: the Rust frame's
  `fn` follows item 1's rule, not `:wat::core::let`.
  - Mutation: drop the application-path guard. RED.

## Goldens

Expected: only goldens whose innermost Rust frame was named by an enclosing form change, to name the
application instead. Recapture, read every diff, and report anything else.

## Scope fence

- **IN:** items 1–3 and the gates.
- **OUT:**
  - E (`EvalError`), F (the domain `Fault`s);
  - the post-F strikes (retire provenance, the startup-message type, declarable `char`,
    `LoadOther`);
  - the stdlib-freeze excursus.

## Discipline

- **`cargo nextest run --release` ONLY, never `cargo test`.** One cargo process at a time: check with
  `pgrep -x cargo` and `pgrep -x cargo-nextest`, never `pgrep -f`.
- **Never use a git worktree with a shared `CARGO_TARGET_DIR`.**
- **Never edit files under `tests/`, `wat/` or `wat-scripts/` while any run is in flight.**
- **Never hand back while a build or floor runs.** Wait in the foreground with
  `until grep -qE '^ *Summary' <log>; do sleep 30; done`, repeating past the tool's 10-minute cap.
- Iterate with targeted runs. Run the full floor at the commit.
- The floor:
  - Run `scripts/floor.sh`: 0 failed, 0 timed out.
  - On any red: surface it before any re-run, verbatim, with the arm named.
  - **Never commit a red floor.** "Unrelated to my change" is not a disposition.
- Stage by name BEFORE the floor. Never use `git add -A`. No `cargo fmt`.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`. Push to
  `origin/reason/little-wat-findings`.
- If your budget runs short, stop at a clean boundary and report where.

## Report

- the application-path naming rule and its four answers;
- the census table (kind → producer → named activation);
- the no-producer list;
- the `Option` decision and its four answers;
- each gate's mutation RED;
- the floor `Summary` line, verbatim;
- the SHA.
