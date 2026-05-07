# Arc 158a — EXPECTATIONS (slice 1)

**Drafted 2026-05-07 by orchestrator before sonnet spawn.**

## Independent prediction

**Predicted runtime:** 25-35 min Mode A. **Time-box:** 45 min wall-
clock.

**Why this estimate:**
- Single function extension (`parse_binding_for_pair_check`);
  ~30-50 LOC change
- New helper (`derive_type_ann_from_rhs`); ~40-60 LOC
- 5-7 tests; ~100-200 LOC
- No consumer sweep; no other crate
- Closest precedent (`extend_pair_scope_with_tuple_destructure`)
  is well-understood

**Mode classification:**
- **Mode A**: 5-7 of 5-7 new tests pass; 2029 baseline preserved.
- **Mode B**: walker placeholder type-ann doesn't trace correctly;
  sonnet patches by reading actual element type.
- **Mode C**: walker has additional internal assumptions about
  type-ann structure that break the migration. Sonnet STOPs and
  reports.

## Expected scorecard rows

| Row | Expectation | Verification |
|---|---|---|
| **Walker fires on new-shape Channel binding** | Pass | Test diagnostic |
| **Walker traces `(:second pair)` in new shape** | Pass | Test diagnostic |
| **Legacy shape still works (regression)** | Pass | Pre-existing tests stay green |
| **Mixed-shape let** | Pass | Test diagnostic |
| **Unrecognized new-shape RHS gives up gracefully** | Pass | No false positive |
| **Arc 128 pattern in new shape** | `ScopeDeadlock` fires | Test diagnostic |
| **Workspace baseline** | 2029 → 2029 + N (5-7 new); 0 failed | `cargo test --release --workspace` |
| **2-file constraint** | Edits in `src/check.rs` + new test file | `git diff --stat` |
| **Uncommitted state** | Sonnet does NOT commit | `git log --oneline -3` |

## Honest delta candidates

- **Type-ann string format** — does the existing walker code
  expect a specific format for the inner type? `Channel<i64>` vs
  `Channel<:i64>` etc. Sonnet should spot-check
  `extend_pair_scope_with_tuple_destructure`'s output format and
  match it.
- **Placeholder `nil` element type** — for `first`/`second`
  patterns, the walker uses `:wat::kernel::Sender<wat::core::nil>`
  (matching arc 133's existing approach). If the trace machinery
  requires the inner element type to MATCH the parent Channel's
  element type for some downstream check, the placeholder breaks.
  Sonnet must verify by walking through `trace_to_pair_anchor`'s
  uses of the type-ann.
- **Inner-arg colon syntax** — arc 115's
  `InnerColonInCompoundArg` rule says inner args inside
  `Channel<...>` must NOT have leading `:`. The
  `trim_start_matches(':')` BRIEF-suggested approach handles
  this, but sonnet should verify against the existing format.

## SCORE methodology

Orchestrator scores after sonnet returns. Each row ✓ / ⚠ / ✗.

- **Mode A clean ship**: commit slice 1; proceed to slice 2 (closure).
- **Mode B**: patch / verify; commit if green.
- **Mode C**: orchestrator decides; if walker requires deeper
  refactor, opens a sub-arc.

## Pre-flight checklist

- [x] DESIGN.md current
- [x] BRIEF-SLICE-1.md drafted
- [x] EXPECTATIONS-SLICE-1.md drafted (this commit)
- [x] Workspace baseline = 2029 / 0 / 0 warnings (verified)
- [ ] Commit BRIEF + EXPECTATIONS
- [ ] `model: "sonnet"` set on Agent call (FM 12)
- [ ] `run_in_background: true` set on Agent call
- [ ] ScheduleWakeup at 45 min (2700s) post-spawn

## Why slice 1 standalone

Arc 158a has no consumer-sweep slice — the walker change is
purely additive (new shape now works; legacy shape continues
to work). User-visible behavior: nothing changes (let-binding
syntax unchanged until 158b). This single slice is the entire
substrate piece; slice 2 is closure paperwork only.

This shape matches arc 132 / arc 124 / similar single-substrate-
piece arcs in the recent past — substrate-only with no consumer
sweep needed.
