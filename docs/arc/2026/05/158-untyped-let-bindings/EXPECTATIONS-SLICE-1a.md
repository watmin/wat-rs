# Arc 158 — EXPECTATIONS (slice 1a)

**Drafted 2026-05-07 by orchestrator before sonnet spawn.**
Slice 1a = substrate (accept new binding shape + walker for
legacy + 10 tests).

## Independent prediction

**Predicted runtime:** 25-35 min Mode A. **Time-box:** 60 min wall-
clock (~2× upper-bound).

**Why this estimate:**
- 1 new CheckError variant (boilerplate, mirrors `BareLegacyLetStar`)
- 1 new walker (~30-50 LOC, mirrors `validate_legacy_let_star`)
- `infer_let` binding-extract path adjustment (~10-30 LOC)
- `eval_let` likely no change (runtime consumes post-check shape)
- Special-forms registry sketch likely no change
- 10 tests (~150-300 LOC)

Comparable to arc 154 slice 1a (60 min predicted, 28 min actual).
Arc 158's walker is structurally similar (Pattern 3 substrate-as-
teacher) but matches a deeper AST shape (binding within bindings
list) instead of an outer-head keyword. Marginal added complexity.

**Mode classification:**
- **Mode A** (clean ship): 8-10 of 10 new tests pass; many
  `LegacyTypedLetBinding` errors firing on existing legacy sites
  AS EXPECTED; no unexpected reds.
- **Mode B** (substrate friction): 1-2 surprises; sonnet patches.
- **Mode C** (gap): `infer_let`'s binding-extract path doesn't
  cleanly accept both shapes; orchestrator decides on path forward.

## Expected scorecard rows

| Row | Expectation | Verification |
|---|---|---|
| **Tests 1-4 (canonical new shape)** | All pass | `cargo test --release --test wat_arc158_let_bindings canonical` |
| **Tests 5-8 (legacy shape walker)** | `LegacyTypedLetBinding` fires per binding | Test diagnostic asserts variant name + canonical fix wording |
| **Tests 9-10 (behavior parity)** | Type inference identical regardless of shape; sequential semantics preserved | Test passes with both shapes |
| **Workspace failures** | ~951 `LegacyTypedLetBinding` firing across wat-rs | NOT a regression; expected; clears in 1b sweep |
| **No unexpected red** | All non-let-related tests stay green | `cargo test --release --workspace` shows only let-related failures |
| **4-file constraint** | Edits in named files only | `git diff --stat` |
| **Uncommitted state** | Sonnet does NOT commit | `git log --oneline -3` shows no new commits |
| **CheckError Display** | References arc 158, names canonical fix | Test diagnostic readable |

## Honest delta candidates (track in sonnet's report)

- **`infer_let` binding-extract restructuring.** Did the existing
  logic accept both shapes by adding one alternative branch, or
  did it need broader restructuring? Arc 144's `Binding` enum
  work touched this area; precedent may help.
- **Runtime change footprint.** Did `eval_let` need touching, or
  was check-side flexibility sufficient?
- **Walker pattern depth.** Walking INTO the bindings list (one
  level deeper than arc 154's outer-keyword walker) — any
  surprise in the AST traversal shape?
- **Legacy-`:T`-ignored semantic change.** Per BRIEF: legacy
  bindings still parse, but the declared `:T` is IGNORED at
  inference (sonnet uses expr's inferred type). This is the
  arc 145 lesson applied uniformly. Did this surface any
  caller-visible behavior change beyond the walker firing?
  E.g., did a previously-failing type-check now pass because
  the declared `:T` was a lie?

## SCORE methodology (after sweep 1b ships + atomic commit lands)

Orchestrator scores when workspace = 0-failed (post-1b sweep).
Each scorecard row marked ✓ / ⚠ / ✗.

- **Mode A clean ship:** commit 1a + 1b atomically; proceed to 1c
  (lab sweep), then 2 (closure).
- **Mode B (1-2 reds patched):** record substrate-friction shape;
  proceed.
- **Mode C:** orchestrator decides patch vs reland.

## Pre-flight checklist (orchestrator runs BEFORE spawn)

- [x] DESIGN.md current
- [x] BRIEF-SLICE-1a.md drafted
- [x] EXPECTATIONS-SLICE-1a.md drafted (this commit)
- [x] `cargo test --release --workspace` baseline = 2029 / 0 / 0
      warnings (verified post arc 157 closure)
- [ ] Commit BRIEF + EXPECTATIONS
- [ ] `model: "sonnet"` set on Agent call (FM 12)
- [ ] `run_in_background: true` set on Agent call
- [ ] ScheduleWakeup at 60 min (3600s) post-spawn

## Why this slice now

User direction 2026-05-07: *"we are doing the hard grunt work to
enable what i have planned... we just need to do the mass
refactors step by step."* Arc 158 is the next step in the
ergonomic-consolidation sequence: arc 153 (nil) + arc 136 (do)
+ arc 154 (let sequential) + arc 155 (fn/Fn) + arc 157 (def)
all narrowed the user-facing surface toward Clojure-faithful
shapes; arc 158 closes the let-binding-type-annotation gap so
`let` matches `def` and brackets-coming-soon arc operates on a
clean foundation.

The four questions all hold; the stepping-stones questions all
hold; the path is obvious by every discipline.
