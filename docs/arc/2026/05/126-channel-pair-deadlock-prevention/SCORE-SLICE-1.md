# Arc 126 Slice 1 — Score against pre-handoff expectations

**Written:** 2026-05-01, AFTER reading sonnet's report and BEFORE
acting on its content. Scores the agent's deliverable against
`EXPECTATIONS-SLICE-1.md` row-by-row.

**Agent ID:** `a37104bfc10e4c6fa`
**Agent runtime:** 814 seconds (~13.5 min)
**Verification commands run:** `git status --short`, `git diff
--stat`, `grep -n` for function names + substring on `src/check.rs`,
read of `parse_binding_for_pair_check` body, grep for `pub fn` on
new functions.

## Hard scorecard

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Diagnostic substring | **PASS** | `src/check.rs:409` emits header `channel-pair-deadlock at <span>: ...` verbatim. Unit test at line 10918 asserts the substring presence. |
| 2 | Single-file diff | **PASS** | `git diff --stat` shows ONLY `src/check.rs \| 560 +++++`. No other files touched. |
| 3 | **Workspace green** | **FAIL** | Workspace test reports `1632 passed; 2 failed; 5 ignored`. The 2 failures are `HologramCacheService::test_step1_spawn_join` + `..._test_step2_counted_recv` — collateral from step3-6's bodies tripping the file-level freeze. Honest disclosure from the agent; expected to be green but isn't. |
| 4 | Arc 117 reuse | **PASS (with caveat)** | New `parse_binding_for_pair_check` is NOT a duplicate of arc 117's `parse_binding_for_typed_check` — it returns `(String, String, WatAST)` (name, type-ann, RHS) where arc 117 returns `(String, String, Span)`. The RHS-bearing variant is required for chain-tracing through `(first|second pair)` projections. Sonnet correctly identified the signature mismatch and added the sibling parser. The brief said "REUSE"; the implementation diverged for sound reasons. Caveat: brief should be more precise. |
| 5 | No commits | **PASS** | `git status` shows only modified `src/check.rs`; no commit, no push. |
| 6 | Honest report | **PASS** | Report includes file:line refs for variant (144), Display (401), mapping (610), and 6 new functions; unit test count (4 added at 10774/10814/10858/10918); workspace totals (1632/2/5); the actual exact panic message; and the failure-mode disclosure on step1+step2 collateral. Substantively complete. |

**Hard verdict: 5 of 6 pass.** Row 3 (workspace green) failed; the
failure mode was both predictable in retrospect and concretely
named by the agent in the report.

## Soft scorecard

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 7 | LOC budget | **DRIFT — under-budget claim, over-budget reality** | DESIGN budgeted ~200 LOC; hard rows said 150-300 band; actual = 560 LOC additions. Significantly over. Two factors: (a) Display message is verbose (the full Fix:-block prose adds ~30 lines); (b) two type-classifier functions (`type_is_sender_kind` + `type_is_receiver_kind`) instead of one polymorphic helper; (c) detailed doc comments on each function. Code is straightforward; no algorithmic over-engineering visible. Acceptable drift but worth recognizing. |
| 8 | Function quartet | **PASS+** | All five DESIGN-named functions present (`validate_channel_pair_deadlock`, `walk_for_pair_deadlock`, `check_call_for_pair_deadlock`, `trace_to_pair_anchor`, `type_is_receiver_kind`) plus the sibling `type_is_sender_kind` and `parse_binding_for_pair_check`. Naming matches DESIGN. |
| 9 | Unit tests covering 4 cases | **PASS** | All 4 named cases present at lines 10774 (anti-pattern fires), 10814 (two-different-pairs silent), 10858 (HandlePool-pop silent), 10918 (substring assertion). All 4 pass under `cargo test --release -p wat --lib check`. |
| 10 | False-negative honesty | **PASS** | Agent confirms DESIGN's "Cross-function tracing skipped" caveat is observable via the HandlePool-pop test (trace gives up at user-fn boundary). No tightenings or loosenings reported. Honest. |
| 11 | No new public surface | **PASS** | `grep -n "pub fn"` on the new function names returns nothing — all are `fn` (private). |
| 12 | No env/config flag | **PASS** | No conditional compilation, env var, or feature gate visible in the additions. |

**Soft verdict: 5 PASS + 1 DRIFT.** LOC came in 2.8x DESIGN's
estimate; not a discipline failure but a calibration miss on the
DESIGN's part.

## What this scores tells us

### What was right

- The brief's structural anchor strategy — pointing at arc 117's
  existing functions with file:line refs — produced a competent
  implementation. Sonnet templated correctly; the trace algorithm
  is sound; the diagnostic substring landed verbatim.
- The substring lock worked exactly as intended. The unit test at
  line 10918 enshrines it; slice 2's `:should-panic` will match.
- The agent surfaced the workspace-collateral failure HONESTLY
  rather than glossing over it — exactly the substrate-as-teacher
  discipline the brief called for.

### What was wrong (the predicted "second-most-likely" outcome partially fired)

The orchestrator's pre-prediction listed three possibilities. The
actual outcome is closest to the **failure mode** (third option):
"workspace red because the rule fires on a substrate pattern the
DESIGN's caveats didn't anticipate." But the failure isn't a
substrate pattern — it's the file-level freeze checking ALL forms
in `HologramCacheService.wat`, including the step3-6 deftest
bodies that contain the deadlock anti-pattern.

**The brief's load-bearing claim was wrong:**

> "Existing 6 ignored deadlock tests stay ignored — they remain
> off the runtime path until slice 2 unignores them."

`:wat::test::ignore` is a runtime-test marker (cargo's
`#[ignore]` attribute on the generated `#[test] fn`). It skips
test EXECUTION but does NOT gate the file's freeze. When
`run_single_deftest` is called for step1, it loads
HologramCacheService.wat → freezes it → walks all forms in the
file → arc 126's check fires on step3-6's bodies → freeze fails
→ step1's runner panics. Same for step2.

Step3-6 themselves are correctly caught (they would have shown as
panicking-with-the-substring under `:should-panic`), but the
collateral is step1+step2 also panic with the same substring,
which is NOT what the file currently expects — those tests have
no `:should-panic` annotation; they expect to PASS.

### What this teaches the brief / DESIGN

**For the BRIEF:** The "workspace stays GREEN" success criterion
needs a precondition: deadlock-bearing forms must be RELOCATED to
a separate `.wat` file before slice 1 ships, OR slice 1 must NOT
land on a workspace where deadlock-bearing forms exist
unannotated, OR slice 2 must immediately follow slice 1 in the
same commit cycle.

**For the DESIGN:** The False-negative caveats list should add a
"Quoted-forms-still-checked" caveat: the check fires on any AST
shape regardless of whether it's inside a `:wat::core::forms`
quasi-quote. This is BY DESIGN — quoted forms are still parsed
ASTs and might be evaluated. But it means deftest bodies (which
expand to forms inside `run-sandboxed-hermetic-ast`) participate
in the outer file's freeze check. Files that mix passing and
failing deftest bodies will see EVERY deftest fail to run, not
just the offending ones.

### Substrate-as-teacher calibration

The discipline is **mostly intact** — the brief + DESIGN + arc
117 precedent together produced a correct, honest implementation
in ~13.5 min wall-clock.

The discipline gap is at the SLICE BOUNDARY: slice 1 cannot land
on a green workspace alone; slice 2 must immediately follow OR
slice 1 must include the file-relocation work. The brief's
slice boundary (slice 1 = check + tests; slice 2 = test
annotations) is wrong; the slice-1 unit cannot stand alone.

Future arcs of this class should bundle "land the check + relocate
deadlock-bearing test forms" as ONE atomic unit, with the
`:should-panic` annotation conversion as a third trailing slice.

## Methodology audit

The orchestrator (this Claude session):

1. ✓ Read `EXPECTATIONS-SLICE-1.md` first; held the criteria fixed.
2. ✓ Verified each row with concrete evidence (`git diff --stat`,
   `grep -n` for substrings/function names, file:line cross-checks).
3. ✓ Scored each row pass/fail/drift with one-sentence justification.
4. ✓ Diagnosed the workspace-failure collateral with reference to
   the brief's faulty `:ignore-gates-freeze` premise.
5. ✓ Names the discipline gap (slice boundary) and the corrective
   action (slices 1+2 must bundle, OR relocate forms first).

Score document lands as a sibling to EXPECTATIONS; both stay
durably for cross-session calibration.

## Next steps — decided

Both paths A (relocate first) and B (bundle slice 2) were
considered and overruled by a third option: **arc 128**. The user
named the deeper structural fix:

> "it skips execution but does NOT gate file-level freeze.
> how do we attack this problem?"

The substrate-level answer: the check walker should NOT descend
into the first argument of `run-sandboxed-ast` /
`run-sandboxed-hermetic-ast` / `fork-program-ast` /
`spawn-program-ast` calls. Inner programs are separate freeze
units; their forms get checked at runtime when sandbox-freeze
fires. Arc 117 has the same latent issue (no deftest today
exercises it).

**Arc 128 ships first.** Then arc 126 slice 1 reland is a clean
re-spawn with the boundary respected.

Sonnet's slice-1 work preserved as a patch:
`/tmp/arc-126-slice-1-sonnet-a37104bf.patch` (604 lines). The
working tree is reverted to HEAD; the durable scorecard above
captures the structural choices (function names, line numbers,
false-negative confirmation, exact panic message) so the next
agent run can re-derive the implementation from the brief alone.
The patch is the backup; the docs are the source of truth.

Continuity record: arc 126 task (#213) is BLOCKED on arc 128
(#214). Slice 1 reland follows arc 128's landing.
