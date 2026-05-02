# Arc 126 Slice 1 — Pre-handoff expectations

**Written:** 2026-05-01, AFTER spawning the sonnet agent and BEFORE
its first deliverable. Durable record so the post-result review
scores against what was written here, not against expectations
unconsciously revised once the work returns.

**Agent ID:** `a37104bfc10e4c6fa`
**Brief consumed:** `BRIEF-SLICE-1.md` + `DESIGN.md`
**Output channel:** `src/check.rs` modifications + ~150-word
written report.

## Hard scorecard (must-pass — failure = brief/DESIGN gap)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | **Diagnostic substring** | The literal string `channel-pair-deadlock` appears verbatim in the Display impl's emitted text. Lowercase, hyphenated, single identifier. The brief locked this; slice 2 matches against it. |
| 2 | **Single-file diff** | Only `src/check.rs` is modified. No `.wat` files. No other Rust files. No new files outside `src/check.rs`. No `Cargo.toml` change. |
| 3 | **Workspace green** | `cargo test --release --workspace` exits 0. The 6 ignored deadlock tests REMAIN ignored (slice 1 doesn't touch them). All existing tests pass. |
| 4 | **Arc 117 reuse** | The agent reuses arc 117's existing `parse_binding_for_typed_check`, `expand_alias`, `type_contains_sender_kind` rather than duplicating their logic. Code-search after report: a duplicate `parse_binding_for_typed_check`-shaped function = brief misread. |
| 5 | **No commits** | Working tree has uncommitted modifications; agent did not run `git commit` or `git push`. The brief explicitly forbids both. |
| 6 | **Honest report** | The 150-word report contains: file:line refs for the new variant + each new function; unit-test count + pass status; workspace totals; the actual panic-message string. Vague summaries like "shipped the check" without specifics fail this row. |

## Soft scorecard (signals — sonnet is well-served by docs)

| # | Criterion | Pass condition |
|---|---|---|
| 7 | **LOC budget** | Total `src/check.rs` additions in the 150-300 LOC band. <100 = under-implemented; >300 = solving more than DESIGN asked. |
| 8 | **Function quartet** | Five named functions appear: `validate_channel_pair_deadlock`, `walk_for_pair_deadlock`, `check_call_for_pair_deadlock`, `trace_to_pair_anchor`, `type_is_receiver_kind`. (Sonnet may pick slightly different names; if so, the rename should be defensible — e.g. clearer noun. Names diverging without justification = brief drift.) |
| 9 | **Unit tests covering 4 cases** | Tests for: anti-pattern fires, two-different-pairs silent, HandlePool-pop silent, diagnostic-substring contains literal. |
| 10 | **False-negative honesty** | The agent surfaces ANY deviation between DESIGN's caveats and the actual implementation — tightened OR loosened. Silence here when implementation diverges = honesty gap. |
| 11 | **No new public surface** | Walker functions are `fn` (private), not `pub fn`. Brief said this; if `pub` shows up, brief was unclear or ignored. |
| 12 | **No env/config flag** | Brief forbids feature toggles and env vars. The check always runs. |

## What we learn — discipline calibration

If **all hard rows pass + most soft rows pass**: substrate-as-teacher is intact for the deadlock-prevention class. Future arc-117-shaped rules can ship via brief + DESIGN + arc-precedent template, no in-person handholding.

If **a hard row fails**: the brief is underspecified at that row. Diagnose:
- **Substring missed** → the brief should have said "MUST appear, not 'preferably contains'" — re-read the brief's wording for ambiguity.
- **Workspace red** → either the implementation is too aggressive (substrate's own pair-by-index pattern trips false-positive) OR the agent didn't run `cargo test --workspace` to verify. The brief's verification section needs to be more imperative.
- **Diff broader than `src/check.rs`** → constraint wasn't stated in must-not-touch terms forcefully enough.

If **all hard pass + soft drifts** (e.g. function names diverge defensibly, LOC ~250): the brief produced what was asked + the agent applied judgment. Score as healthy.

If **everything passes**: the brief + DESIGN + arc 117's existing code together CONSTITUTE a complete spec for this rule's class. We've found a transferable pattern: future structural type-check rules can ship the same way.

## Independent prediction (testable bet)

Before reading the agent's output, the orchestrator predicts:

- **Most likely:** all 6 hard rows pass; 4-5 soft rows pass; LOC ~180-260; one soft row drifts (probably function naming — agent picks a slightly different verb for `walk_for_pair_deadlock`). Discipline: intact.
- **Second-most likely:** 5 of 6 hard rows pass — diagnostic substring is *similar* but not exact (e.g. "channel pair deadlock" with spaces). Discipline: brief needed a regression test for the substring lock or a code-quote of the EXACT line. Slice 2 will catch it; slice 1 needs a follow-on edit.
- **Failure mode:** workspace red because the rule fires on a substrate pattern (Console / telemetry / pipeline) the DESIGN's caveats didn't anticipate. Fix path: tighten the rule's classifier (e.g. exempt `HandlePool::pop` projections from the trace) and document the exemption in DESIGN. Discipline: caveats list missed a case; the substrate teaches the gap.

## Methodology

After the agent reports back, the orchestrator (Claude in this
session) MUST:

1. Read this file FIRST, before the agent's output, to anchor the
   measurement.
2. Score each row of both scorecards explicitly (pass / fail /
   not-applicable + one-sentence justification).
3. Diff the agent's claimed file edits against what `git diff
   --stat` shows (trust-but-verify).
4. Diagnose any failure with reference to the brief or DESIGN —
   what was unclear, what was missing, what to amend.
5. Commit the measurement record (this file + the score) so the
   substrate-as-teacher discipline is observable across sessions.

This document is the substrate's calibration record. It does not
move once the agent finishes; the score lands as a sibling
document.
