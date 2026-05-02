# Arc 126 Slice 2 — Pre-handoff expectations

**Written:** 2026-05-01, AFTER spawning the sonnet agent for
slice 2 and BEFORE its first deliverable. Durable scorecard.

**Brief consumed:** `BRIEF-SLICE-2.md`
**Output channel:** 3 wat-test files modified + ~150-word
written report.

## What's different about this slice

Slice 1 was a substrate change (Rust code in `src/check.rs`); the
work was structural and the validation was unit-test + workspace-
green. Slice 2 is a wat-test annotation conversion (6 sites
across 3 `.wat` files); the work is mechanical BUT the
validation crosses a runtime chain (inner freeze → Result::Err
→ cargo libtest panic) that has not been runtime-verified
end-to-end.

The 5/6 row failure on slice 1 surfaced a substrate gap (arc
128). Slice 2 has its own runtime-chain unknown. If the substring
propagates cleanly, the whole arc 126 chain validates. If it
doesn't, we surface the substrate gap and open a follow-on arc.

Both outcomes are useful per failure-engineering.

## Hard scorecard (8 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Three-file diff | Exactly 3 files modified: `crates/wat-lru/wat-tests/lru/CacheService.wat`, `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`, `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat`. No other files. |
| 2 | Substring locked verbatim | The 6 `:should-panic` annotations carry the literal string `"channel-pair-deadlock"` (lowercase, hyphenated, single identifier). `grep -c '"channel-pair-deadlock"'` across the 3 files returns 6. |
| 3 | `:ignore` annotations REMOVED on the 6 sites | The 6 sites no longer carry `:wat::test::ignore`. `grep -c ":wat::test::ignore" docs/arc/...` should return 0 (the workspace's only remaining `:ignore` is in `wat-sqlite`'s arc-122 mechanism test, which is not in this slice's scope). |
| 4 | `:should-panic` annotations PRESENT on the 6 sites | `grep -c ":wat::test::should-panic" <files>` returns 6. |
| 5 | `:time-limit "200ms"` annotations PRESERVED | The 200ms safety net stays; defense-in-depth. `grep -c ":wat::test::time-limit" <files>` returns 6 (unchanged from pre-slice-2). |
| 6 | **Workspace green** | `cargo test --release --workspace` exit=0. |
| 7 | **Six tests now PASS via `:should-panic` matching** | The 6 previously-`ignored`-status tests in cargo test output now show as `... ok`. The runtime chain (arc-126 check → inner freeze Err → run-sandboxed Result → cargo libtest panic) preserved the substring. |
| 8 | Honest report | 150-word report with: 6 site:line refs, the exact final form of one annotation block, per-crate test totals, workspace totals, runtime-per-test timing data (slice 2's load-bearing question), explicit confirmation of substring propagation OR honest disclosure of where it broke. |

**Hard verdict:** all 8 must pass for slice 2 to ship clean. Row
7 is the runtime-chain test; if it fails, sonnet's report
becomes the brief for the next substrate arc.

## Soft scorecard

| # | Criterion | Pass condition |
|---|---|---|
| 9 | No commits | Working tree has uncommitted modifications; agent did not run `git commit`. |
| 10 | No other doc edits | No `docs/` files touched. No `INSCRIPTION.md`, no `DESIGN.md`. (Slice 3 handles closure docs.) |
| 11 | No `src/check.rs` or substrate changes | Only the 3 wat-test files modified. |
| 12 | Comment blocks updated | Sonnet updates the comment block above each annotation to reflect the new role ("expected to panic with the substring" instead of "we know this hangs"). Honest. |
| 13 | Six annotations match BRIEF table | The 6 sites converted are exactly the ones named in BRIEF-SLICE-2.md's table. No off-by-one, no skip, no extra. |
| 14 | Substring propagation timing data | If row 7 passes, the report includes runtime-per-test data (e.g. "step3 panics in ~50ms; well under the 200ms limit"). This proves the inner freeze is fast and the safety net is appropriate. |

## Independent prediction

Before reading the agent's output, the orchestrator predicts:

- **Most likely (~50%):** all 8 hard + 5-6 soft pass. Substring
  propagates cleanly. Each previously-ignored test panics in
  10-100ms with the substring; cargo libtest matches; the test
  reports as PASS. The whole arc 126 chain validates.
- **Second-most-likely (~30%):** all 8 hard + 5-6 soft pass, but
  the substring is wrapped (e.g. `... cargo test panic'd:
  channel-pair-deadlock at ...`). libtest's substring match is
  position-agnostic, so this still passes. Same outcome.
- **Substring-mangling (~15%):** row 7 fails. The chain wraps or
  rewrites the substring before it reaches cargo libtest's
  panic message. Tests fail with "test panicked, but did not
  contain expected string". Sonnet captures the actual panic
  message; that becomes the brief for a follow-on substrate arc
  (arc 129?: improve the test-runner's panic-message
  preservation).
- **Inner-freeze-never-fires (~5%):** row 7 fails for a deeper
  reason — maybe the inner freeze caches and doesn't re-check,
  or the deftest's inner program is somehow exempt. Would
  require digging into `run_single_deftest` + run-sandboxed-
  hermetic-ast internals. Surprising; would surface a real
  substrate confusion.

## Methodology

After the agent reports back, the orchestrator MUST:

1. Read this file FIRST.
2. Score each row of both scorecards explicitly.
3. Diff the agent's claimed file edits against `git diff --stat`.
4. Verify hard row 2 by `grep -c '"channel-pair-deadlock"'`
   across the 3 files (must equal 6).
5. Verify hard row 3 by `grep` for `:wat::test::ignore` across
   the 3 files (must return 0).
6. Verify hard row 7 by reading the cargo test output: 6 tests
   that were `ignored` are now `ok` (from `:should-panic`).
7. Diagnose any row 7 failure with reference to the BRIEF's
   "honest unknown" section — surface it as a follow-on arc if
   needed.
8. Land SCORE-SLICE-2.md as a sibling.

## Why this slice matters for the arc as a whole

Slice 2 is the **end-to-end test** of arc 126's check. Slice 1
proved the check fires structurally (unit tests pass). Slice 2
proves the check fires AT RUNTIME and that the failure mode
(panic with substring) is observable through cargo's standard
test mechanism.

The ARTIFACTS-AS-TEACHING discipline says: each delegation is
a measurement of the artifacts. Slice 2 measures whether the
artifacts (DESIGN + BRIEF-SLICE-1 + SCORE-SLICE-1 + arc 128
INSCRIPTION + slice 1's shipped check + the BRIEF-SLICE-2
itself) compose into a teaching that gets a sonnet sweep to
ship slice 2 cleanly. If yes, the discipline is fully
validated for this arc.
