# Arc 170 slice 1d — closure-extraction walker substrate fixes

## Context

Surfaced during slice 3 phase B verification: ~162 deftest_*
failures all trace to closure-extraction's `walk_free_symbols`
not tracking some binding forms it should. Phase A's honest
delta B named match-arm pattern bindings; reality is broader.

Slice 3's atomic-commit (phase A + phase B) cannot land until
workspace = 0-failed. Slice 1d fixes the walker substrate gaps
so the existing test bodies extract cleanly; slice 3's atomic
commit covers phase A + phase B + 1d as ONE commit per recovery
doc § 7.

## Goal

Investigate `closure_extract::walk_free_symbols` (and its
helpers `walk_let_form` / `walk_fn_form` / `walk_define_form`).
Extend scope-tracking to cover EVERY binder form the substrate
supports. Each gap surfaces as a `free symbol 'X' does not
resolve` error when the affected wat code is closure-extracted.

The user's principle: "we own wat; we fix what we break."
Closure-extraction must handle every binding form the substrate
supports — match arms, struct-destructure, tuple-destructure,
nested fn params, etc.

## Read first (in order)

1. docs/arc/2026/05/170-program-entry-points/BRIEF-SLICE-1D.md (this doc)
2. docs/arc/2026/05/170-program-entry-points/EXPECTATIONS-SLICE-1D.md (scorecard)
3. docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-1B.md (slice 1b deliverable; honest delta B is the surfaced gap)
4. docs/arc/2026/05/170-program-entry-points/REALIZATIONS-SLICE-1.md (six framing passes)
5. docs/arc/2026/05/170-program-entry-points/CLOSURE-EXTRACTION.md v2 (algorithm spec)
6. docs/COMPACTION-AMNESIA-RECOVERY.md § 6 (FM 5/9/10/11/12/16) + § 7 atomic-commit
7. docs/SUBSTRATE-AS-TEACHER.md (the failure stream IS the migration brief — applies to substrate-bug fixing too)

## Working tree state

DIRTY — slice 3 phase A + phase B work uncommitted on
`arc-170-program-entry-points`. Workspace state: 1966 passed /
162 failed.

**DO NOT REVERT phase A + phase B work.** Their changes are
correct; they're waiting on slice 1d's substrate fix. Slice 1d
ADDS substrate fixes; orchestrator atomically commits all three
phases when workspace = 0-failed.

Files in dirty tree (do not touch except per below):
- src/check.rs, src/runtime.rs, src/spawn_process.rs,
  src/stdlib.rs (phase A + phase B)
- wat/test.wat, wat/std/hermetic.wat (deleted),
  wat/std/sandbox.wat (deleted) (phase A)
- ~50 test fixture files (phase B sweep)

## Scope

### 1. Investigation

Run cargo test --release on the 162 failing deftests. Each
failure surfaces a specific free-symbol-not-resolving error
naming the symbol + span. The symbol is some binder
closure-extraction's walker missed.

Sample failures to start with:
- `deftest_wat_tests_holon_Sequential_test_self_identity` —
  free symbol `head` (match-arm binding in `wat/holon/Sequential.wat:34`)
- `deftest_wat_tests_holon_eval_coincident_test_arithmetic_equivalence` —
  free symbol `b` (location to investigate)
- Sample 5-10 more to map the gap categories

Categorize the missed-binder gaps. Possibilities (investigate):
- Match-arm pattern bindings — `(:wat::core::match scrut ((:wat::core::Some name) ...))`
- Struct-destructure bindings — `[{field1 field2} struct-val]` (arc 169)
- Tuple-destructure bindings — `[[a b c] tuple-val]` (arc 168)
- Nested fn params already known? Verify
- Anything else surfaced by the failure stream

### 2. Walker fixes

Extend `walk_free_symbols` (and its helpers in
`src/closure_extract.rs`) to track every binder category found
in step 1. Each category needs its scope-introduction logic
added to the walker — names bound by the form are LOCAL inside
the form's body; not free symbols.

Per arc 167 + arc 168 + arc 169 substrate work, the AST shapes
are settled; walker just needs to recognize them.

### 3. Tests

Add Rust integration tests in
`tests/wat_arc170_closure_extraction.rs` (where slice 1b's tests
live). For each gap category fixed, add at least one test that
extracts a fn body using that binder + verifies the binder's
name does NOT appear as a free symbol in the prologue.

T16+ — agent picks numbering.

### 4. Verification

Run cargo test --release --workspace; verify the 162 deftest_*
failures drop to 0 (or near-zero; surface any residual that
isn't walker-related).

Do NOT touch phase A or phase B's work. Do NOT commit.
Orchestrator atomically commits all three phases when workspace
green.

## Critical syntax shapes

Per arc 167 + arc 109 + arc 153:
- fn-form: `(:wat::core::fn [name <- :T ...] -> :Ret body)`
- defn: `(:wat::core::defn :name [params] -> :Ret body)`
- match arm: `((:wat::core::Some pattern-name) body-using-pattern-name)`
- struct-destructure: `[{field1 field2} struct-value]`
- tuple-destructure: `[[a b c] tuple-value]`
- Type names: `:wat::core::nil` (NOT bare `:nil`)

## Honest delta categories

- **Diversity of missed binders** — investigation may reveal more
  categories than match-arm. Surface each + fix.
- **Walker design refactor** — if the walker's scope-tracking
  needs structural changes (e.g., to support binders that
  introduce names through transformation), surface the shape
  before implementing.
- **Sub-cases that aren't walker bugs** — some of the 162 might
  fail for OTHER reasons (different substrate gap, test-body
  shape that genuinely doesn't extract). Surface; orchestrator
  decides.
- **FM 5 trap** — TODOs verboten.

## Branch + commit policy

- Active branch: `arc-170-program-entry-points` (DIRTY tree)
- DO NOT COMMIT. DO NOT PUSH.
- Orchestrator atomically commits phase A + phase B + slice 1d
  as ONE commit when workspace = 0-failed
- DO NOT edit SCOREs 1, 1B, 1C, 2 (immutable)

## Predicted runtime

90-180 min opus. Hard cap 360 min.

Comparable to slice 1's substrate work (90-180; actual ~150).
Slice 1d adds walker categories; existing slice 1b walker
infrastructure provides the framework.

## Reporting

Report to chat with:
- Categories of missed binders found + fixed
- Test additions in tests/wat_arc170_closure_extraction.rs
- Workspace state via cargo test --release --workspace
  (expected: 2128 passed 0 failed = 1966 baseline + 162 swept;
  surface any residual)
- Honest deltas surfaced
- Wall-clock minutes
- Confirm: "Working tree dirty including phases A + B + 1d; not
  committed; ready for orchestrator atomic commit."
