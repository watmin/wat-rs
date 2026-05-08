# Arc 167 slice 4 — SCORE

Slice 4 hard-retired all slice 2 transitional scaffolding in one
commit (`0e519be`). Substrate deletions per BRIEF-SLICE-4 ran clean.
Workspace landed at 2053/16 — 16 lib unit-test fixtures inside
`#[test]` blocks surfaced as failing because slice 2's walker scoping
(per delta A) had hidden them from slice 3's diagnostic stream.

The 16-failure surface IS slice 4b's input. Slice 4 ships honestly
as "substrate retirement complete; lib unit-test sweep follows."

## Scope as shipped

All deletions targeted the slice 2 transitional scaffolding:
- `CheckError::BareLegacyFnSignature` (variant + Display + Diagnostic arms)
- `validate_legacy_fn_signature` + `walk_for_legacy_fn_signature` bodies
- `parse_legacy_fn_signature` (runtime) + `parse_legacy_fn_signature_for_check` (check)
- `eval_fn` 2-arg legacy arm + `try_parse_fn_shape_def` 3-arm branch (now require 5-element canonical form)
- `freeze.rs` `validate_legacy_fn_signature` pre-pass call
- Tests 5 + 6 in `tests/wat_arc167_fn_flat_signature.rs` (BRIEF lean-(a): standard `MalformedForm` parser error covers the legacy shape now; no dedicated regression needed)

After deletion, the substrate has zero trace of legacy nested-sig support. Per "doesn't leave cruft" discipline.

## Scorecard

| Row | Verified by | Pass |
|-----|-------------|------|
| A — Walker variant + Display + Diagnostic deleted | `grep -rn "BareLegacyFnSignature" src/`: 0 hits | ✓ |
| B — Walker body + registration deleted | `grep -rn "validate_legacy_fn_signature\|walk_for_legacy_fn_signature" src/`: 0 hits | ✓ |
| C — Legacy parsers deleted (both sides) | `grep -rn "parse_legacy_fn_signature" src/`: 0 hits | ✓ |
| D — Migration message text gone | `grep -rn "fn signature must be a vector binding form" src/`: 0 hits | ✓ |
| E — `eval_fn` arity-checks to 4 only | `eval_fn` now produces clear error if form doesn't match `(:wat::core::fn [args] -> :T body)` exactly | ✓ |
| F — Tests 5 + 6 disposition: DELETED | per BRIEF lean-(a); legacy shape produces standard `MalformedForm` | ✓ |
| G — Test 9 (`reflection_on_flat_defn_resolves`) still passes | runtime check post-deletion; doesn't depend on legacy-arm fallthrough | ✓ |
| H — `cargo build --release --workspace` green | substrate compiles cleanly post-retirement | ✓ |
| I — Slice 3 substrate fix preserved | `walk_for_bare_primitives` Vector arm at `src/check.rs` (commit `066e3ac`) untouched | ✓ |
| J — Canonical fn-sig parsers untouched | `parse_fn_signature` (runtime) + `parse_fn_signature_for_check` (check) unchanged in this slice | ✓ |
| K — `wat/core.wat` defn macro untouched | the canonical macro shape stays | ✓ |
| L — Workspace test count post-retirement | `cargo test --release --workspace`: 2053 passed / 16 failed (16 lib unit-test fixtures need slice 4b) | ✓ partial |

## Honest deltas

### Delta A — 16 lib unit-test fixtures surface

This was the predicted slice-3-boundary issue from SCORE-SLICE-3 delta B. Slice 2 delta A scoped the walker to user-source forms via `freeze.rs:599-616`, which deliberately skipped `mod tests` fixtures inside `src/runtime.rs` + `src/check.rs`. Slice 3 swept `wat/`, `wat-tests/`, `tests/wat_*.rs` — every site the walker fired on.

The 16 unit-test sites depended on the legacy parser arm (not the walker). Slice 4 deleted that parser arm. Tests now fail with clean `MalformedForm` errors naming the legacy shape — exactly the substrate-as-teacher pattern.

Sites enumerated (line numbers pre-edit):
- `src/check.rs` — 1: line 13782 (`typed_let_binding_with_fn_value`)
- `src/runtime.rs` — 15: lines 18776, 18789, 18804, 20760, 20783, 20874, 20885, 20895, 20959, 21192, 21432, 21702, 21718, 21732, 21755

These 16 sites are the input to slice 4b. **The slice boundary is honest**: slice 4 retires substrate machinery; slice 4b sweeps the orphaned fixtures.

### Delta B — no other substrate quirks

Slice 4 was clean retirement work. No `eval_fn` callsite required workarounds; no other subsystem held a reference to the legacy parser; the `freeze.rs` walker registration was a clean delete next to the still-needed `validate_bare_legacy_primitives` (out of arc 167 scope; stays). The discipline reminders in BRIEF-SLICE-4 were honored without surface deviation.

## Calibration row

| Predicted | Actual | Mode |
|-----------|--------|------|
| 30-60 min opus (3 deletion regions + 2 vacuous-test deletions) | ~30 min opus, single commit | A clean |

Lower-bound of the predicted band. Mechanical retirement matched the BRIEF's explicit deletion list.

## Discipline check

- ✓ FM 14 internal-identifier sweep was complete for the retired surface (substrate cruft fully deleted; no orphaned scaffolding kept under arc 113 precedent — per user direction "doesn't leave cruft")
- ✓ Slice 3 substrate fix (`066e3ac` Vector arm) preserved as permanent infrastructure
- ✓ Canonical parsers + `wat/core.wat` defn macro left alone (slice 2's settled foundation)
- ✓ Branch isolation held: main untouched

## What's next

Slice 4b: sweep the 16 lib unit-test fixtures via mechanical translation `((x :T) (y :T) -> :R)` → `[x <- :T y <- :T] -> :R`. This is a pure test-fixture sweep — no substrate work. Predicted 15-30 min sonnet (the first sonnet sweep on the new `./scripts/cargo-test-summary.sh` infrastructure).

Slice 4 ships partial because the workspace is not yet 0/0; slice 4b closes that gap. Atomic merge to main waits on slice 4b green.
