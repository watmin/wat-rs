# Arc 167 slice 3 — sweep all legacy fn-sigs (substrate-as-teacher)

## Goal

Migrate every legacy nested-sig `((x :T) (y :T) -> :T)` in the
codebase to the new flat shape `[x <- :T y <- :T] -> :T body`.
Two scopes bundled in this single slice:

1. **Test-driven sweep** (152 failing tests across 18 test
   targets — sonnet reads cargo-test diagnostic stream emitted by
   the `BareLegacyFnSignature` walker, applies the mechanical
   translation, iterates until 0 failed)
2. **Grep-driven stdlib sweep** (`wat/*.wat` + `wat-tests/*.wat` —
   walker doesn't fire here per slice 2 delta A; sonnet greps for
   legacy-shape patterns and migrates by hand)

End state: `cargo test --release --workspace --no-fail-fast`
returns 0 failed AND grep returns 0 legacy fn-sigs anywhere in
the workspace except substrate scaffolding (slice 4 retires
that).

## Branch + commit policy

- **Active branch: `arc-167-slice-2-fn-sig-consumer`** (yes, the
  slice 2 branch — slices 2 + 3 share a branch per the atomic-
  merge discipline)
- Multiple WIP commits + pushes welcome on the branch for backup
- DO NOT push to main; orchestrator merges atomic to main as one
  commit after slice 3 ships green
- The branch should reach 0-failed state before this slice
  considers itself done

## Background context (read these first)

- `docs/arc/2026/05/167-fn-flat-signature/BRIEF-SLICE-2.md` —
  the verbose migration message you'll see in walker errors;
  this text IS your translation rulebook
- `docs/arc/2026/05/167-fn-flat-signature/SCORE-SLICE-2.md` —
  the deltas that shaped slice 3's scope (walker is user-source
  only; legacy parser still works; both scopes need sweeping)
- `docs/arc/2026/05/167-fn-flat-signature/DESIGN.md` — full arc
- `docs/SUBSTRATE-AS-TEACHER.md` — the failure-engineering
  discipline (FM 15 in particular: failures ARE the work)
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 15, § FM 16

## The migration recipe (one rule, mechanical)

For every legacy fn-sig:

```scheme
(:wat::core::fn ((x :T) (y :T) -> :R) BODY)
```

Translate to:

```scheme
(:wat::core::fn [x <- :T y <- :T] -> :R BODY)
```

Per-element transformation:
- Each `(name :Type)` pair → `name <- :Type` (3 tokens flat)
- The `->` arrow + return type stay; they move OUT of the
  parens, becoming siblings of the new `[...]` vector
- The body stays unchanged

Same recipe for defn:

```scheme
(:wat::core::defn :name ((x :T) (y :T) -> :R) BODY)
;; → translate the sig the same way →
(:wat::core::defn :name [x <- :T y <- :T] -> :R BODY)
```

Zero-arg case:

```scheme
(:wat::core::fn (-> :R) BODY)
;; →
(:wat::core::fn [] -> :R BODY)
```

The walker's error message in `src/check.rs` shows this recipe
verbatim. When iterating from cargo test output, copy the recipe
mechanically.

## Sweep procedure (substrate-as-teacher discipline)

### Phase 1 — test-driven sweep (152 sites)

1. Run `cargo test --release --workspace --no-fail-fast`
2. Read `BareLegacyFnSignature` errors with their spans
3. For each error: open the named file/line, apply the mechanical
   recipe, save
4. Re-run cargo test
5. Repeat until cargo test returns 0 failed

Failing test targets (from slice 2 SCORE):
- `-p wat`: `test`, `wat_arc143_define_alias`,
  `wat_arc150_variadic_define`, `wat_arc154_kill_let_star`,
  `wat_arc155_fn_rename`, `wat_arc157_def`, `wat_arc166_defn`,
  `wat_core_try`, `wat_names_are_values`, `wat_sort_by`,
  `wat_spawn_fn`, `wat_stream`, `wat_tco`, `wat_typealias`,
  `wat_typed_if_match`, `wat_variadic_defmacro`
- `-p wat-telemetry --test test`
- `-p wat-telemetry-sqlite --test test`

### Phase 2 — grep-driven stdlib sweep

The walker is user-source-only (slice 2 delta A); stdlib's legacy
fn-sigs don't fire walker errors. Phase 2 grep-finds them and
migrates manually.

Sites:
- `wat/*.wat` (the substrate-bundled stdlib — `wat/core.wat`,
  `wat/stream.wat`, `wat/console.wat`, `wat/holon.wat`,
  `wat/test.wat`, `wat/runtime.wat`, etc.)
- `wat-tests/*.wat` (test fixtures that don't go through user-code
  walker — substrate-bundled tests)
- Any `crates/*/wat-tests/` directories with similar bundled tests

Recommended grep:
```bash
# Find legacy fn-sigs (lists with -> arrow inside parens at fn
# position):
grep -rn -E '\(:wat::core::(fn|defn) +\(' wat/ wat-tests/ crates/
```

This won't be perfect — false positives possible (e.g., comment
text that mentions the old shape). Visually verify each match
before editing. The mechanical recipe applies the same way.

### Verification at end of Phase 2

After Phase 2:
1. `cargo test --release --workspace --no-fail-fast` → 0 failed
2. `grep -rn -E '\(:wat::core::(fn|defn) +\(' wat/ wat-tests/
   crates/ tests/ src/runtime.rs/tests src/check.rs/tests` should
   return 0 hits (or only hits inside string literals that are
   intentionally legacy — for the walker's own tests in
   `tests/wat_arc167_fn_flat_signature.rs`)

## Discipline reminders

- DO NOT touch `src/` substrate code (parsers, walker, check). The
  substrate is opus's territory in slices 2 + 4.
- DO NOT delete `tests/wat_arc167_fn_flat_signature.rs` test cases
  5 + 6 (legacy walker firing tests). These stay valid until
  slice 4 retires the walker.
- DO NOT push to main. Branch only.
- DO NOT MODIFY the walker's migration message text in
  `src/check.rs` — it's the contract for this exact recipe
- DO commit + push WIP often for branch backup
- If a site doesn't fit the mechanical recipe (e.g., a comment
  mentioning the old shape, a test that intentionally exercises
  legacy syntax in tests/wat_arc167_fn_flat_signature.rs cases
  5/6), STOP at that site and report; orchestrator decides

## Report shape

When complete, report:
1. Final cargo test summary (passed/failed across workspace)
2. Final grep result for legacy fn-sigs across wat/ + wat-tests/
   + crates/ + tests/ (should be 0 outside intentional walker
   tests)
3. Site count by file (for calibration: how many migrations per
   file)
4. Honest deltas — sites that didn't fit the mechanical recipe;
   substrate quirks discovered; intentional false-positives
5. Branch state confirmation
6. Actual runtime in minutes vs predicted band

## Time-box

Per EXPECTATIONS-SLICE-3.md. If you exceed the upper bound still
iterating, STOP and report current state.
