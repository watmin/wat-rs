# BRIEF — shape strike D3: the activation census is a standing gate, and the `Option` decision

Excursus 003. This finishes `BRIEF-shape-strike-D2-every-raise-names-its-activation.md`, items 2 and 3.
D2 landed item 1 (`c003ac8fa`). Strike E (`4dc349549`) made the four eval-only kinds observable. Read
the audit's D2 and E sections first.

**Read `wat-rs/CLAUDE.md` in full first.** Every citation below is a claim. Verify each one before
relying on it, and STOP on a contradiction. Decide anything the brief leaves open by the four
questions (Obvious? Simple? Honest? Good UX?, each a flat YES or NO) and show the answers. **Never
present options.**

## Target

1. **GD2a: a standing, table-driven census of all 40 `RuntimeErrorKind`s, each checked on the path it
   actually takes.**
   - **Real producer exists:** drive it through the real entry point, and assert the Rust frame is
     PRESENT and names the expected activation. Presence is the point: an omission must go RED.
     D2's table has 17 kinds already measured, so reuse those producers.
   - **Eval-only kinds:** drive them through `eval-ast!`/`eval-step!`, which since E carries a
     `Failure` with frames. They are `BadCondition`, `PatternMatchFailed`, `EffectfulInStep` and
     `NoStepRule`. E's GE1 already does this, so fold it in or reference it.
   - **`AssertionFailed` from `assertion-failed!`** travels `AssertionPayload`, not
     `RuntimeError::new`. Check it on that path: its `Failure` carries frames with a named
     activation, or say exactly what that path carries.
   - **No real producer:** list each with its measured reason, and give its row a rune-style reason
     in the table, not a silent skip.
     - D2 found `ParamShadowsBuiltin` (dead), `NoEncodingCtx`/`NoSourceLoader`/`NoMacroRegistry`
       (unreachable from a frozen world), and `UserMainMissing` (producer not found).
     - Re-measure each. A kind with no producer from any real path is a retirement candidate. List
       it; do not retire it here.
   - **The 14 kinds D2 did not reach:** `EvalForbidsMutationForm`, `ChannelDisconnected`, the
     `ReteCeiling` variants, `MacroExpansionFailed`, `SandboxScopeLeak`, `ServiceNotRunning`,
     `EdnCoerceMismatch`, `MacroAbort`, `WriteStopped`, `ReteDefnAxisViolation`, `ReteDefnRecursive`
     and `EvalVerificationFailed`. Find a real producer for each, or measure why none exists.
   - Never hand-construct a `RuntimeError` in a census row: that bypasses the dispatch the row is
     proving.
2. **The `Option` decision, by the four questions.** With the census complete, measure whether any
   real path ever reaches `Frame::rust_site` with an empty activation.
   - If none does, the `None` arm serves only a bare `RuntimeError::new` in unit tests. Decide by the
     four questions: keep the `Option`, or make the frame mandatory and give those unit tests an
     activation. Show the answers, and do what they say.
3. **Correct `src/freeze/pass_order.rs`'s `record()` doc.** It names two freeze-phase producers; the
   census measured more.

## Gate (mutation-proven in RELEASE)

- **GD2a.** One mutation per activation writer: intrinsic/special-form dispatch, the application
  path, `apply_function`, and the freeze phase. Each drops its slot-set, and each must go RED, naming
  the kinds that lost their name.

## Scope fence

- **IN:** items 1–3.
- **OUT:**
  - retiring any kind (list candidates only);
  - F (the domain `Fault`s);
  - the post-F strikes;
  - the eval-unchecked question (E's finding);
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
  - **Never commit a red floor.**
- Stage by name BEFORE the floor. Never use `git add -A`. No `cargo fmt`.
- Commit prefix `EXCURSUS(003):`, ending with
  `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`. Push to
  `origin/reason/little-wat-findings`.
- If your budget runs short, stop at a clean boundary (the census for the reachable kinds, as a
  standing gate, on a green floor) and report where.

## Report

- the 40-row census (kind → path → producer → named activation, or the reason none);
- the retirement-candidate list;
- the `Option` decision and its four answers;
- each writer's mutation RED;
- the floor `Summary` line, verbatim;
- the SHA.
