# Arc 168 slice 3 — substrate retirement

## Goal

Hard-retire all slice 1 transitional scaffolding for legacy
nested-pair-list let-bindings. After this slice, the substrate
has zero trace of legacy outer-list let support; `(let ((...)
...))` in user source produces a clean `MalformedForm` from the
Vector-only parser path; the walker is gone.

Per user direction (arc 167 precedent + arc 168 § "doesn't leave
cruft"): hard deletion, not preserved scaffolding. After slice 3
the legacy shape is "anything that isn't `[name expr ...]`" and
the standard parser error covers it.

## Branch + commit policy

- **Active branch**: `arc-168-let-flat-shape` (slices 1, 2, 3, 4
  share this branch per atomic-merge discipline)
- WIP commits welcome on the branch
- DO NOT push to main; orchestrator merges atomic to main as one
  squash commit after slice 5 closure paperwork

## Read first (in order)

1. `docs/arc/2026/05/168-let-flat-shape/DESIGN.md` — full arc scope
2. `docs/arc/2026/05/168-let-flat-shape/SCORE-SLICE-2.md` — what
   slice 2 shipped + the 3 follow-up gaps slice 1 left (closed
   in commit `b220846`); slice 3 builds on this state
3. `docs/arc/2026/05/167-fn-flat-signature/SCORE-SLICE-4.md` —
   the precedent slice this one mirrors (arc 167 slice 4 was
   exactly this shape — substrate retirement after sweep clears)
4. `docs/COMPACTION-AMNESIA-RECOVERY.md` § 6 (FM 5, FM 11) —
   discipline floor

## Substrate edits

### 1. `src/check.rs` — walker hard-retirement

DELETE every reference to `BareLegacyLetBindings`:
- `CheckError::BareLegacyLetBindings { span: Span }` variant
- `Display` impl arm (the migration message text)
- `Diagnostic` impl arm
- `walk_for_legacy_let_bindings` function body
- `validate_legacy_let_bindings` function body
- The migration-hint string constants if separately defined

After deletion: `grep -rn "BareLegacyLetBindings" src/` returns
0 hits. `grep -rn "let bindings must be a vector" src/` returns
0 hits.

### 2. `src/freeze.rs` — walker registration retirement

DELETE the `validate_legacy_let_bindings` call from the user-
source pre-pass region (around the same place arc 167 slice 4
deleted its walker registration; mirror that pattern). Other
walker validations in that pre-pass stay (they're not arc 168's
territory).

### 3. `src/runtime.rs` — `eval_let` legacy arm retirement

`eval_let` (around `:4013`) currently emits `MalformedForm` if
`args[0]` isn't `WatAST::Vector` (clean error; no legacy
fallback). VERIFY this is the case post-slice-1; if any legacy
List-outer fall-through remains, DELETE it. After deletion,
non-Vector outer shapes produce one clear `MalformedForm` shape
naming the canonical form.

### 4. `src/runtime.rs` — `parse_let_binding` typed-legacy retirement

`parse_let_binding` (around `:4167`) handles three binder shapes:
- `WatAST::Symbol` (canonical bare-symbol — KEEP)
- `WatAST::List(name_type)` legacy typed-single `(name :T)` —
  DELETE (arc 159 retired this in user code; sweep is complete)
- `WatAST::Vector(symbols)` destructure — KEEP

After deletion, malformed binders produce one clear `MalformedForm`
naming the canonical Symbol or Vector-of-Symbols shape.

### 5. `src/check.rs` — check-side parallel retirement

`infer_let` (around `:6253`) + the binding-parse paths on the
check side mirror the runtime retirements. Remove the legacy
List-outer fall-through and typed-single binder support.

### 6. Tests — vacuous-test retirement

Tests in `tests/wat_arc168_let_flat_shape.rs` that asserted
walker firing (e.g. test #6 `legacy_outer_list_fires_walker`)
become vacuous post-retirement — the walker is gone; the form
now produces standard `MalformedForm`. Two paths:

- **(a) DELETE** vacuous walker-firing tests entirely. Cleanest
  per "no cruft."
- **(b) REPLACE** with new tests asserting the legacy shape now
  produces the standard `MalformedForm` parser error.

Lean **(a)** — the legacy shape is well-defined as "anything
that isn't `[name expr ...]`" and the parser's `MalformedForm`
covers it without a dedicated regression. If a `MalformedForm`
shape regression matters, add coverage later cheaply.

DO NOT keep walker-firing tests with assertion text changed —
modifying assertion semantics post-retirement is dishonest.
Either delete or replace; don't half-edit.

## Verification

- `cargo build --release --workspace` green
- Inline pipeline: `cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk '{p+=$4; f+=$6} END {print "passed:", p, "failed:", f}'`
  shows the 5 pre-existing kernel/signal failures only; arc-168
  territory clean
- `grep -rn "BareLegacyLetBindings" src/ tests/` returns 0 hits
- `grep -rn "validate_legacy_let_bindings" src/`  returns 0 hits
- `grep -rn "let bindings must be a vector" src/` returns 0 hits
  (the migration text constant is gone)
- `grep -rn "is_typed_single\|legacy.*let\|typed-legacy.*binder" src/` returns 0 hits

## Discipline reminders

- DO NOT push to main; only push to slice branch
- DO NOT modify the slice 1 substrate consumer paths
  (`parse_let_binding`'s Symbol + Vector branches; `eval_let`'s
  Vector-outer + multi-form-body handling)
- DO NOT modify the canonical fn-sig parser from arc 167 — that's
  a different form's territory
- DO NOT modify `wat/core.wat`'s defn macro
- DO NOT bridge by re-adding legacy parser arms — that's exactly
  the FM 5 the slice 2 follow-up caught and reverted
- USE the inline pipeline for verification (no scripts)
- DO NOT pipe `cargo test` through `awk` — use the inline pipeline
  with `awk '{p+=$4; f+=$6}'` directly (inline awk on cargo
  output is empirically allowed for sonnet subagents)

## FM 5 GUARDRAIL — explicit

If a substrate quirk surfaces (some path you don't expect
references the legacy code; some test depends on the typed-legacy
binder shape; some code path was harder to disentangle than
expected):

- STOP and report
- DO NOT bridge by re-adding the legacy support
- DO NOT modify the test to "make it work"
- The right answer is always: STOP, name the gap, let
  orchestrator decide

The slice 2 follow-up commit `b220846` caught FM 5 in another
flavor (legacy List arm added back to make a fix work); the
right answer was Vector-only. Same discipline applies here.

## Report shape

When complete, report:

1. Final cargo test summary via the inline pipeline (should be
   passed: 2077 failed: 5 — the 5 pre-existing unrelated)
2. Each substrate site you deleted (file + line ranges) with a
   one-line description per deletion
3. Tests #6 disposition (deleted vs replaced) with reasoning
4. Honest deltas — substrate quirks discovered during retirement;
   sites that referenced the legacy parser unexpectedly
5. Branch state confirmation
6. Actual runtime in minutes vs predicted band

## Time-box

30-60 min predicted, 120 min hard cap. If you exceed 60 min still
iterating, STOP and report current state.

## What's next (post-slice-3)

When slice 3 ships green:
- Slice 4 — sweep src/ lib unit-test fixtures (mirror of arc 167
  slice 4b precedent — substrate-internal `mod tests` fixtures
  hidden from sweep stream by walker scoping; surface post-
  retirement)
- Slice 5 — closure paperwork (SCOREs + INSCRIPTION + 058 row +
  USER-GUIDE update + atomic squash-merge)

Plus arc 168a (or 169 — number reserved post-slice-5) opens to
investigate the 5 pre-existing kernel/signal failures.
