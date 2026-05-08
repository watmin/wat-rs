# Arc 167 slice 4 — walker hard-retirement + transitional parser arms retirement

## Goal

Hard-retire ALL slice 2 transitional scaffolding now that slice 3
sweep is complete and workspace is green at 2069/0. Per the user's
"doesn't leave cruft" discipline, slice 4 deletes:

1. The `BareLegacyFnSignature` walker (variant + Display + walker
   body + `freeze.rs` registration + migration text)
2. The transitional legacy parser arms (`parse_legacy_fn_signature`
   + `parse_legacy_fn_signature_for_check`)
3. The `eval_fn` 2-arg legacy arm + any dual-shape branching
4. Tests #5 + #6 in `tests/wat_arc167_fn_flat_signature.rs` (walker-
   firing assertions; vacuous post-retirement)

After this slice, the substrate has zero trace of legacy nested-sig
support. Legacy syntax `((x :T) -> :R)` produces a generic
`MalformedForm` error from the parser ("fn signature must be a
vector binding").

## Branch + commit policy

- **Active branch**: `arc-167-slice-2-fn-sig-consumer` (slices 2 + 3
  + 4 share this branch per atomic-merge discipline)
- Multiple WIP commits + pushes welcome on the branch for backup
- DO NOT push to main; orchestrator merges atomic to main as a
  single squash commit after slice 4 ships green + slice 5 closure
  paperwork ships
- Use `./scripts/cargo-test-summary.sh` for progress checks (proven
  working; safe from awk-pipe denial trigger)

## Background context (read these first)

- `docs/arc/2026/05/167-fn-flat-signature/SCORE-SLICE-2.md` — slice
  2 deltas A + B explain WHY the transitional parser was kept;
  slice 4 closes that loop
- `docs/arc/2026/05/167-fn-flat-signature/BRIEF-SLICE-2.md` — for
  the original walker shape + migration message text
- `docs/arc/2026/05/167-fn-flat-signature/DESIGN.md` — full arc
  scope (slice 4 is "walker hard-retirement"; this BRIEF expands
  it per slice 2 SCORE delta scope)
- Arc 113 INSCRIPTION — orphaned-scaffolding precedent (variant +
  Display preserved). User direction 2026-05-08 explicitly
  REJECTS this pattern for arc 167; full deletion required.

## Substrate edits (per slice 2 SCORE deltas)

### 1. `src/check.rs` — walker hard-retirement

DELETE every reference to `BareLegacyFnSignature`:
- `CheckError::BareLegacyFnSignature { span: Span }` variant
- `Display` impl arm (the verbose migration message)
- `Diagnostic` impl arm
- `walk_for_legacy_fn_signature` function body
- `validate_legacy_fn_signature` function body
- The migration-hint string constants (if separately defined)

After deletion, `grep -rn "BareLegacyFnSignature" src/` returns 0
hits. `grep -rn "fn signature must be a vector binding form" src/`
returns 0 hits.

### 2. `src/freeze.rs` — walker registration retirement

DELETE the `validate_legacy_fn_signature` call from
`freeze.rs:599-616` (the user-source pre-pass region opus added in
slice 2). Other walker validations in that pre-pass stay
(BareLegacyPrimitive etc. — not arc 167's territory).

### 3. `src/runtime.rs` — `eval_fn` arity retirement + parser retirement

DELETE:
- `parse_legacy_fn_signature` function body
- The `args.len() == 2` (legacy 2-arg) arm in `eval_fn` if
  separately branched; `eval_fn` should accept ONLY the canonical
  4-arg shape (`args-vec`, `->`, `:ret-type`, `body`)
- Any dual-arm shape branching that called the legacy parser

After deletion, `eval_fn` has one arity-validation path producing
a clear error if the form doesn't match `(:wat::core::fn [args]
-> :T body)` exactly.

### 4. `src/check.rs` — check-side parser retirement

DELETE:
- `parse_legacy_fn_signature_for_check` function body
- Any dual-arm dispatching in `parse_fn_signature_for_check_diag`
  if it branched on shape; that should now ONLY handle the
  canonical Vector shape

### 5. `tests/wat_arc167_fn_flat_signature.rs` — vacuous-test retirement

Tests 5 + 6 (`legacy_nested_sig_fn_fires_walker` and
`legacy_nested_sig_defn_fires_walker_via_macro`) assert the walker
fires. After slice 4, the walker is gone. Two paths:

- **(a) DELETE** tests 5 + 6 entirely. Cleanest per "no cruft."
- **(b) REPLACE** tests 5 + 6 with new tests asserting the legacy
  shape now produces the standard `MalformedForm` parser error.

Lean **(a)** — the legacy shape is well-defined as "anything that
isn't the canonical shape," and the standard parser error covers
it without a dedicated regression test. If a `MalformedForm` error
shape regression matters, adding a coverage test later is cheap.

DO NOT keep tests 5 + 6 with the assertion text changed —
modifying assertion semantics post-retirement is dishonest. Either
delete or replace; don't half-edit.

### 6. Test 9 (`reflection_on_flat_defn_resolves`) verify

Test 9 uses `lookup-define` against a flat-shape defn. Should
still pass post-retirement. Verify it doesn't depend on legacy-arm
fallthrough.

## Verification (per scorecard in EXPECTATIONS-SLICE-4.md)

- `cargo build --release --workspace` green
- `./scripts/cargo-test-summary.sh` returns `passed: N failed: 0`
  (count may differ slightly from 2069 because tests 5 + 6 retire)
- `grep -rn "BareLegacyFnSignature" src/ tests/` returns 0 hits
- `grep -rn "parse_legacy_fn_signature" src/` returns 0 hits
- `grep -rn "fn signature must be a vector binding form" src/`
  returns 0 hits (the migration text constant is gone)

## Discipline reminders

- DO NOT push to main; only push to slice branch
- DO NOT modify the new `walk_for_bare_primitives` Vector arm at
  `src/check.rs:2200+` — that's permanent infrastructure (arc 167
  slice 3 substrate fix), NOT legacy scaffolding
- DO NOT modify the substrate fn-sig parser path
  (`parse_fn_signature` + `parse_fn_signature_for_check`) — those
  are the canonical paths that stay
- DO NOT modify `wat/core.wat`'s defn macro — it's the canonical
  shape that stays
- If a substrate decision arises (e.g., dual-arm logic was harder
  to disentangle than expected; some other subsystem held a
  reference to the legacy parser), STOP and report; orchestrator
  decides
- Use `./scripts/cargo-test-summary.sh` for progress
- If tests fail unexpectedly post-retirement (something depended
  on the legacy parser that we didn't catalog), DO NOT bridge by
  re-adding the legacy parser — STOP and report; the dependency
  is the problem to surface

## Report shape

When complete, report:
1. Final cargo test summary (passed/failed) via the script
2. Each substrate site you deleted (file + line ranges) with a
   one-line description per deletion
3. Tests 5 + 6 disposition (deleted vs replaced) with reasoning
4. Honest deltas — substrate quirks discovered during retirement;
   sites that referenced the legacy parser unexpectedly
5. Branch state confirmation
6. Actual runtime in minutes vs predicted band

## Time-box

Per EXPECTATIONS-SLICE-4.md. If you exceed the upper bound still
iterating, STOP and report current state.
