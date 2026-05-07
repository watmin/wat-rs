# Arc 157 — EXPECTATIONS (slice 1a-ii)

**Drafted 2026-05-07 by orchestrator before sonnet spawn.**
Slice 1a-ii = relaxation layer (2 config setters + opt-in gating
+ type-stability + 5 tests).

## Independent prediction

**Predicted runtime:** 18-25 min Mode A. **Time-box:** 45 min wall-
clock (~2× upper-bound).

**Why this estimate:**
- 2 SymbolTable bool fields (1 line each + Default plumbing)
- 2 config setters (mirror existing `set-capacity-mode!` exactly)
- 1 new CheckError variant (boilerplate)
- Gating logic at existing fire site (~10-20 LOC)
- Type-stability comparison (~5-10 LOC)
- 5 tests (~80-150 LOC)

Smaller than 1a-i (which was 22 min actual + 10 min runtime addendum
= ~32 min total) because no new infrastructure — gates wrap the
existing strict-default fire site.

**Mode classification:**
- **Mode A** (clean ship): 5/5 new tests pass; 2024 baseline preserved;
  eval-time scope-out documented if needed.
- **Mode B** (substrate friction): 1-2 surprise reds; sonnet patches.
- **Mode C** (gap): eval-time gating requires mutability
  infrastructure (Mutex etc.); per `feedback_zero_mutex.md`, STOP
  and report; orchestrator opens a separate arc for that.

## Expected scorecard rows

| Row | Expectation | Verification |
|---|---|---|
| **Test 15 (default flag off → strict still holds)** | Pass; `DefRedefForbidden` fires | Test diagnostic |
| **Test 16 (set-redef! true + same type)** | Pass; redef succeeds; runtime resolves to new value | Test resolves to new value |
| **Test 17 (set-redef! true + diff type)** | `DefRedefTypeChange` fires naming both types | Test diagnostic |
| **Test 18 (set-redef! false explicit)** | `DefRedefForbidden` fires | Test diagnostic |
| **Test 19 (set-eval-redef! lands)** | Form recognized at top-level; carrier flag updates | Either functional gating OR scope-out per eval-time STOP |
| **Workspace baseline** | 2024 → 2029 (or 2028 if test 19 scoped out) | Test count comparison |
| **3-4 file constraint** | Edits in check.rs / runtime.rs / special_forms.rs / tests | `git diff --stat` |
| **Uncommitted state** | Sonnet does NOT commit | `git log --oneline -3` |
| **Setter pattern match** | `:wat::config::set-redef!` reads like `set-capacity-mode!` | Code review |

## Honest delta candidates (track in sonnet's report)

- **CheckEnv-vs-SymbolTable flag read path.** Slice 1a-i used
  CheckEnv for check-time `defined_values`. The flags belong on
  SymbolTable (per memory `feedback_capability_carrier.md`). Where
  does the check arm read the flag from? Is there an existing
  CheckEnv→SymbolTable path, or does sonnet add a mirror field?
- **Single-pass vs two-pass interaction.** `set-redef!` is itself
  a top-level form whose value (true/false) needs to be known when
  subsequent defs are checked. Single-pass program-order (the
  natural fit) requires CheckEnv to be mutable through the
  top-level form sequence. Sonnet reports if the existing check
  ordering supports this cleanly.
- **Eval-time gating viability.** Per the eval-time STOP signal —
  if eval-time `def` binding requires mutability infrastructure
  not present today, sonnet ships option (b) (carrier + setter
  scaffolding, behavior scope-out).
- **`set-redef!` vs `set-eval-redef!` symmetry.** They mirror in
  shape but their gating points differ (check.rs vs runtime.rs).
  Sonnet's report should describe both.

## SCORE methodology

Orchestrator scores after sonnet returns. Each row marked ✓ / ⚠ / ✗.

- **Mode A clean ship (5/5):** commit 1a-ii; proceed to 1b consumer
  sweep + 2 closure paperwork.
- **Mode A with eval-time scope-out (4/5):** commit 1a-ii; the
  scope-out is honest as long as test 19 verifies the surface
  lands. Document the deferral in the slice's INSCRIPTION later.
- **Mode B / C:** orchestrator decides on patch vs reland.

## Pre-flight checklist (orchestrator runs BEFORE spawn)

- [x] DESIGN.md current
- [x] BRIEF-SLICE-1a-ii.md drafted
- [x] EXPECTATIONS-SLICE-1a-ii.md drafted (this commit)
- [ ] Commit BRIEF + EXPECTATIONS
- [ ] Verify baseline 2024 / 0 (assumed from b10e998 ship)
- [ ] `model: "sonnet"` set on Agent call (FM 12)
- [ ] `run_in_background: true` set on Agent call
- [ ] ScheduleWakeup at 45 min (2700s) post-spawn

## Why slice 1a-ii is the right next stone

1a-i shipped the foundation: def works end-to-end with strict-
default redef-error. 1a-ii relaxes that via opt-in flag while
preserving the type-stability contract. After 1a-ii ships, arc
157's substrate is feature-complete. Each step's verification is
cleaner per the proactive stepping-stones discipline (memory
`feedback_stepping_stones_proactive.md`).

The eval-time scope-out (if it lands as scope-out) is the honest
boundary: the user's stated need is freeze-time hot-reload safety;
eval-time gating is theoretical until a caller surfaces. Per
`feedback_pivot_not_defer.md`, scope-out is acceptable WHEN the
language is affirmative ("not active because eval-time def is
not wired") rather than punted ("future arc when X surfaces").
