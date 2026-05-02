# Arc 133 Slice 1 — Pre-handoff expectations

**Written:** 2026-05-02, AFTER spawning sonnet, BEFORE
deliverable.

**Brief:** `BRIEF-SLICE-1.md`
**Output:** Rust changes to `src/check.rs` + unit tests +
~250-word report.

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Single Rust file diff | All modifications in `src/check.rs`. No `.wat` files. No other Rust files. No documentation. |
| 2 | Approach picked + justified | Sonnet's report names the chosen path (in-place / walker-with-CheckEnv / third-path) with a sentence on why it fits the substrate. The trade-off surface from the brief is acknowledged. |
| 3 | Tuple-destructure shapes recognized | The new code path (whichever approach) classifies every name in `((n1 n2 ...) rhs)` correctly when RHS resolves to a tuple type whose elements include Thread-kind / Sender-kind / HandlePool. |
| 4 | Required unit tests added | All four `arc_133_*` tests in `src/check.rs::tests` block. Names match the brief literally (sonnet may add more — that's bonus). |
| 5 | **Unit tests pass** | `cargo test --release -p wat --lib check` exit=0. The four arc_133_* unit tests pass; all existing arc-117 + arc-131 unit tests still pass. |
| 6 | Existing checks intact | The typed-name binding shape continues to fire ScopeDeadlock + ChannelPairDeadlock as before. The structural walker(s) either retire cleanly or stay as documented redundancy. |
| 7 | No commits | Working tree shows uncommitted modifications; agent did not run `git commit`. |
| 8 | Honest report | ~250-word report includes approach + file:lines + unit-test counts + workspace prediction + honest deltas. Includes the trade-off observation between the in-place and walker-extension paths. |

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 9 | LOC budget | New code ~50-150 LOC (one new helper function + check call site + unit tests). Larger budget OK if the walker retires (net delta could be near-zero). >300 LOC = surface for review. |
| 10 | Diagnostic span quality | The fired error names the offending tuple-destructure binding's name + the let*-binding span (NOT the inner symbol position). Diagnostic readable as "scope-deadlock: '<name>' (a HandlePool) holds Sender clones while Thread/join-result blocks". |
| 11 | Workspace runtime | If sonnet runs the workspace test, it stays under 90s. Hung tests get caught by arc 132's 200ms default. |
| 12 | No prediction explosion | Workspace prediction (rough grep) ≤10 sites. >25 = re-evaluate the rule's strictness; tuple-destructure may be more common than estimated. |

## Independent prediction

Before reading the agent's output:

- **Most likely (~50%):** in-place approach inside
  `infer_let_star` ships clean. All 8 hard + 3-4 soft pass.
  Substrate walker retires cleanly because the inference-time
  check covers it. ~80-150 LOC.
- **Walker-extension path (~25%):** sonnet investigates and
  finds the in-place approach hits a blocker (e.g.,
  inference happens before the structural walkers run; the
  invariant ordering matters). Picks the walker-extension
  path; refactors walker signatures to take CheckEnv. All 8
  hard + 3-4 soft pass; ~120-200 LOC.
- **Hybrid (~15%):** sonnet picks the in-place check but
  KEEPS the structural walkers as belt + suspenders for
  pre-inference robustness. The in-place check covers the
  tuple-destructure gap; the walker still catches typed-name
  cases earlier. ~120-180 LOC.
- **Workspace surfaces too many newly-firing tests (~7%):**
  the bypass was hiding 8+ tests; slice 2 becomes a real
  consumer sweep. Slice 1 still ships clean as a substrate
  fix; slice 2 expands.
- **Substrate surprise (~3%):** an unexpected interaction
  between the in-place check and existing inference logic
  (e.g., tuple-destructure types aren't fully resolved when
  the new check runs because of fresh-var ordering). Sonnet
  surfaces the issue + needed substrate fix; arc 134 may
  spawn.

## Methodology

After agent reports back:

1. Read this file FIRST.
2. Score each row with concrete evidence.
3. `git diff --stat` should show only `src/check.rs`
   (one Rust file).
4. Verify hard row 5 by `cargo test --release -p wat --lib
   check 2>&1 | tail -20`.
5. Verify hard row 6 by checking the existing arc-117 + arc-131
   tests still pass.
6. Run `cargo test --release --workspace 2>&1 | grep -E
   "(failures|FAILED)"` to enumerate any newly-firing tests
   for slice 2 scope.
7. Score; commit SCORE-SLICE-1.md.

## Why this slice matters

Slice 1 is the closing of a gap surfaced by arc 131 slice 2's
honest reporting. The chain — sonnet finds gap; we name arc;
new sonnet closes gap — is the artifacts-as-teaching
discipline at work. If slice 1 ships clean, the discipline's
self-improvement cycle is validated end-to-end:

| # | Step | Source |
|---|---|---|
| 1 | Arc 131 slice 1 ships HandlePool check | sonnet sweep clean |
| 2 | Arc 131 slice 2 sweeps consumer tests | sonnet finds bypass |
| 3 | Arc 131 slice 2 SCORE names the bypass | orchestrator review |
| 4 | Arc 133 DESIGN drafted | substrate-author + brief |
| 5 | **Arc 133 slice 1 closes the bypass** | **this sweep** |

If slice 1 fails: the issue is either the brief (too vague
about the architecture choice) or the substrate (the gap is
deeper than the SCORE diagnosis suggested). Either way,
calibration data.

## What follows

After arc 133 slice 1 SCORE + commit:

- If workspace prediction ≤5 sites: skip slice 2; just
  refactor those tests in this session.
- If 5-25 sites: spawn arc 133 slice 2 sonnet sweep.
- Either way: slice 3 closure (INSCRIPTION + WAT-CHEATSHEET
  cross-reference + arc 117/131 cross-references).

The chain's discipline holds: every gap closes; every gap
that closes reveals or DOESN'T reveal further gaps; the
record stays honest.
