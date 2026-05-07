# Arc 158 — Consumer sweep BRIEF (slice 1b)

**Drafted 2026-05-07.** Slice 1b of arc 158.

## Workspace state pre-spawn

- HEAD: `b46adea` (arc 158 slice 1a BRIEF + EXPECTATIONS)
- Working tree: DIRTY with prior sonnet's slice 1a edits (substrate
  + walker + 10 tests). DO NOT revert.
- 1a sonnet's report: 10/10 new tests pass; 73 workspace failures =
  all expected `LegacyTypedLetBinding` firings from stdlib +
  embedded sites. NO unexpected reds.

```
 M src/check.rs        (+170 LOC)
 M src/runtime.rs      (+85 LOC)
?? tests/wat_arc158_let_bindings.rs  (+254 LOC)
```

## Goal

Mechanical sweep across wat-rs consumer sites: every legacy
let-binding `((name :T) expr)` → `(name expr)`. Count is
irrelevant per user direction; the work is the work. Atomic
commit with 1a per recovery doc § 7
atomic-commit-across-coordinated-sweeps.

| Before | After |
|---|---|
| `((name :T) expr)` | `(name expr)` |
| `(((floor :wat::core::f64) (some-call ...))` | `((floor (some-call ...))` |

The OUTER bindings list `(...)` STAYS. Only the inner per-binding
shape changes — drop the type-annotation wrapper around the
binder.

## Sweep targets

### wat/ stdlib

- `wat/holon.wat`
- `wat/stream.wat`
- `wat/test.wat`
- `wat/std/sandbox.wat`
- `wat/std/hermetic.wat`
- `wat/holon/Circular.wat`
- `wat/holon/Sequential.wat`
- (any others under `wat/` — sweep recursively)

### wat-tests/

- `wat-tests/holon/*.wat` (Reject, Trigram, Hologram, coincident,
  Filter, etc.)
- `wat-tests/edn/*.wat`
- `wat-tests/core/*.wat`
- `wat-tests/stream.wat`
- `wat-tests/tmp-*.wat`
- (any others under `wat-tests/` recursively)

### crates/*/wat-tests/ + crates/*/wat/

- `crates/wat-lru/wat-tests/`
- `crates/wat-holon-lru/wat-tests/`
- `crates/wat-telemetry/wat-tests/`
- `crates/wat-telemetry-sqlite/wat-tests/`
- `crates/wat-sqlite/wat-tests/`
- `crates/*/wat/` (any wat sources in crate-private dirs)

### examples/

- `examples/with-loader/**/*.wat`
- `examples/with-lru/**/*.wat`
- (any others)

### Embedded wat strings in Rust

- `src/diagnostic.rs`
- `src/special_forms.rs`
- `src/form_match.rs`
- `src/freeze.rs`
- `tests/wat_string_ops.rs`
- `tests/wat_sort_by.rs`
- `tests/wat_arc072_letstar_parametric.rs`
- `tests/wat_arc113_raise_round_trip.rs`
- `tests/wat_spawn_lambda.rs`
- `tests/wat_eval_result.rs`
- (any others — `grep -rl ':wat::core::let' src/ tests/`)

### EXPLICITLY EXCLUDE

- `tests/wat_arc158_let_bindings.rs` — the new test file
  intentionally exercises BOTH new and legacy shapes. DO NOT
  touch its tests; they verify the walker fires on the legacy
  shape.

## Mechanical transform

For each occurrence of `:wat::core::let`, find its bindings list
and transform each binding:

- Old binding: `((<keyword> <type-expr>) <expr>)` — a 2-element
  list whose FIRST is a 2-element list `(<keyword> <type-expr>)`
- New binding: `(<keyword> <expr>)` — a 2-element list with bare
  keyword first

Type-expr can be ANYTHING:
- Bare keyword: `((x :wat::core::i64) ...)` → `((x ...))`
- Parametric: `((xs :wat::core::Vector<:wat::core::i64>) ...)` → `((xs ...))`
- Function type: `((f :wat::core::Fn(:wat::core::i64)->:wat::core::i64) ...)` → `((f ...))`
- Tuple type: `((p :wat::core::Tuple<:wat::core::i64,:wat::core::i64>) ...)` → `((p ...))`

The pattern is structural: `((NAME TYPE-EXPR) EXPR)` → `(NAME EXPR)`. Sonnet picks the right tool (sed regex with care, ast-aware sweep, manual edit per file, etc.).

## Constraints

- **Wat sources + embedded Rust strings only.** NO substrate edits
  (sonnet 1a already did those). NO new files (the new test file
  is sonnet 1a's; do not add to it).
- **Both 1a substrate edits AND 1b consumer edits live in the
  same atomic commit.** Sonnet 1b operates on the dirty tree
  left by sonnet 1a; do NOT revert 1a's changes.
- **DO NOT COMMIT.** Orchestrator commits 1a + 1b atomically when
  workspace = 0-failed.
- **Workspace MUST go from 73 failures to 0.** Every
  `LegacyTypedLetBinding` firing should be silenced by sweep
  1b's transform.
- **STOP at unexpected red.** Distinguish:
  - **Expected:** workspace failures decrease as you sweep
  - **Unexpected:** new failures from your edits — stop and report
- **No grinding.** No bracket form. No restructuring beyond the
  mechanical transform.
- **Time-box: 60 min wall-clock.**

## Pre-flight crawl (mandatory)

1. `docs/arc/2026/05/158-untyped-let-bindings/DESIGN.md` — full read
2. `docs/arc/2026/05/158-untyped-let-bindings/BRIEF-SLICE-1a.md` — what 1a sonnet shipped
3. `docs/arc/2026/05/154-kill-let-star/BRIEF-SLICE-1b.md` (if it exists) or arc 154 INSCRIPTION — closest precedent for a let-related sweep
4. `docs/arc/2026/05/155-fn-rename/INSCRIPTION.md` — multi-bucket sweep pattern (~476 sites)
5. Spot-read 2-3 wat files to confirm the legacy binding shape pattern: `wat/holon.wat`, `wat/stream.wat`, `wat-tests/holon/Hologram.wat`

## Verification (after sweep)

```bash
cargo test --release --workspace 2>&1 | grep -E "test result|FAILED" | tail -10
cargo test --release --workspace 2>&1 | grep -cE "LegacyTypedLetBinding"
```

Expect: workspace = 2010 baseline + 14 arc 157 + 5 arc 157 + 10 arc 158 = ~2039 passed; 0 failed; 0 LegacyTypedLetBinding firings.

(Actually exact numbers may differ — sonnet 1a's report said pre-sweep workspace was 708/73; post-sweep should be ~2039/0 if no other tests regress.)

## Reporting (~250 words)

- Pre-flight crawl confirmation
- Sweep summary by bucket: file count + binding count per bucket
- Tooling used (sed regex / ast-aware / manual / mix); honest deltas if any tooling claim was wrong (per memory `feedback_collapse_to_llm_in_loop.md`)
- Workspace count after sweep (should be 0 failed)
- Any sites that needed manual handling (multi-line bindings, unusual type expressions, comments referencing legacy shape, etc.)
- Path classification (Mode A / B / C)

DO NOT commit. DO NOT write a SCORE doc. Orchestrator commits 1a + 1b atomically + scores after.

## Time-box

60 minutes wall-clock. ScheduleWakeup will fire if you stall.

## Why this matters

User direction 2026-05-07: *"i do not give a shit how many
occurrences there are - we do the hard work."* Slice 1b is the
grunt work that closes the substrate change end-to-end.
Mechanical, high-volume, but each edit is one structural
transform per binding.

After 1b ships clean (0 failed), orchestrator commits 1a + 1b
atomically and proceeds to slice 1c (cross-repo lab sweep).

Begin.
