# Arc 157 — EXPECTATIONS (slice 1a-i)

**Drafted 2026-05-07 by orchestrator before sonnet spawn.**
Slice 1a-i = substrate (`:wat::core::def` form + position rule +
`defined_values` carrier + strict-default redef-error + 11 tests).

## Independent prediction

**Predicted runtime:** 25-35 min Mode A. **Time-box:** 60 min wall-
clock (~2× upper-bound).

**Why this estimate:**
- One new special form (mechanical: register + check arm + eval arm)
- One new SymbolTable carrier field (1 line + Default plumbing)
- 2 new CheckError variants (boilerplate)
- Position predicate (the only novel logic — needs parent-context
  threading; ~20-40 LOC depending on how `check.rs` already tracks
  parents)
- 11 tests (each is 5-15 lines of wat in `assert_check_*` /
  `assert_eval_*` style; ~150-300 LOC total)
- Freeze integration (one-line list addition)

Smaller than the originally-bundled 1a (90-min predicted) because
slice 1a-i excludes:
- `redef_allowed` / `eval_redef_allowed` bools (1a-ii)
- 2 config setters mirroring `set-capacity-mode!` (1a-ii)
- `DefRedefTypeChange` variant + type-stability logic (1a-ii)
- 4 redef-discipline tests (1a-ii)

Comparable to arc 154 slice 1a (kill let* — 60 min predicted, 28
min actual).

**Mode classification:**
- **Mode A** (clean ship): 9-11 of 11 tests pass; 0 unexpected
  reds; pre-existing baseline preserved.
- **Mode B** (substrate friction): 1-2 surprise reds requiring
  brief patches; sonnet completes within budget.
- **Mode C** (gap): position predicate or `defined_values`
  reference-resolution integration hits a substrate gap requiring
  orchestrator intervention.

## Expected scorecard rows

| Row | Expectation | Verification |
|---|---|---|
| **Tests 1-4 (basic binding + reference resolution)** | All pass | `cargo test --release --test wat_arc157_def basic` |
| **Tests 5-8 (legal positions: top-level, do-splice, let-splice, recursive)** | All pass; let captures closure correctly | `cargo test --release --test wat_arc157_def position_legal` |
| **Tests 9-10 (illegal positions: if, define-body)** | `DefNotTopLevel` fires naming the wrapper | Test diagnostic readable |
| **Test 11 (strict-default redef)** | `DefRedefForbidden` fires naming prior location | Test diagnostic readable |
| **Workspace baseline** | Pre-existing 2010 tests stay green; total = 2010 + 11 (or 2010 + 9-10 if Mode B) | `cargo test --release --workspace` count comparison |
| **5-file constraint** | Edits ONLY in 5 named files | `git diff --stat` |
| **Uncommitted state** | Sonnet does NOT commit | `git log --oneline -3` shows no new commits |
| **`:wat::core::def` registry entry** | Visible via reflection | Special-forms registry includes it |
| **CheckError Display strings** | Reference arc 157, name canonical alternatives + (for redef) the slice 1a-ii flag that doesn't exist yet | Test failure messages readable |

## Honest delta candidates (track in sonnet's report)

- **Parent-context tracking shape:** does `check.rs` already
  thread parent context, or does sonnet need to add it? Arc 144's
  `Binding` enum work touched this area; there may be a clean
  hook OR a fresh tunnel needed. Worth a careful read of
  `infer_let` / `infer_do` before writing the predicate.
- **Keyword-reference resolution:** when post-`def` code uses
  `:pi`, the lookup must consult `defined_values`. Where in the
  current keyword-resolution chain does this insertion go?
  Honest delta if existing code already has a similar map under
  a different name.
- **`define` interaction:** if `:wat::core::define` already
  populates a similar map, do we share or keep `defined_values`
  separate? Sonnet should report. (DESIGN says separate; sonnet
  may surface friction.)
- **`Default` derive on SymbolTable:** if SymbolTable has manual
  `Default` impl, sonnet adds the new field; if derived, the
  empty HashMap works automatically.
- **`SourceLocation` shape:** what's the current span/loc type
  in `check.rs` post-arc-138? Sonnet uses whatever's standard.

## SCORE methodology (after sonnet returns)

Orchestrator scores when sonnet reports complete. Each scorecard
row marked:
- ✓ = expectation met
- ⚠ = met with caveat (sonnet's honest delta surfaces)
- ✗ = not met (Mode classification adjusts)

If 8+ rows ✓ and Mode A: clean ship; commit 1a-i; proceed to
1a-ii.

If Mode B (1-2 reds patched): score B-recovery; record the
substrate-friction shape for the foundation log.

If Mode C: sonnet kill + brief retune required.

## Pre-flight checklist (orchestrator runs BEFORE spawn)

- [x] DESIGN.md current and reflects Path B (let splices)
- [x] BRIEF-SLICE-1a-i.md committed
- [x] EXPECTATIONS-SLICE-1a-i.md committed (this commit)
- [x] `cargo test --release --workspace` baseline = 2010 / 0
      FAILED (verified)
- [ ] `model: "sonnet"` set on Agent call (FM 12)
- [ ] `run_in_background: true` set on Agent call
- [ ] ScheduleWakeup at 60 min (3600s) post-spawn

## Why slice 1a-i first

Stepping-stone discipline (recovery doc § 5 + memory
`feedback_stepping_stones_proactive.md`):

- 1a-i ships a complete, testable, useful piece on its own —
  `def` works at top-level with strict-default redef-error.
- 1a-ii operates on the settled `defined_values` foundation
  1a-i ships, rather than introducing infrastructure AND using
  it in one breath.
- Verification per piece is cleaner; rollback is per-piece.
- Each step's "did it work" test is smaller cognitive surface.

Simple steps enable complex steps.

## What slice 1a-ii will add (for sonnet's mental model)

NOT in this BRIEF's scope, but for context:

- `redef_allowed: bool` + `eval_redef_allowed: bool` carrier fields
- `:wat::config::set-redef!` + `:wat::config::set-eval-redef!`
  primitives (mirror `set-capacity-mode!`)
- Gating logic at the `defined_values` write site:
  - if `redef_allowed = false` → fire `DefRedefForbidden` (this
    1a-i behavior becomes the strict path)
  - if `redef_allowed = true` → check type-stability; if same
    type → replace; if different type → fire
    `DefRedefTypeChange`
- `DefRedefTypeChange` CheckError variant
- 4 additional tests in `wat_arc157_def.rs` (or a sibling file):
  redef-with-flag-on-same-type, redef-with-flag-on-diff-type,
  eval-redef analogs

Sonnet 1a-i should NOT preemptively scaffold these — clean
foundation; 1a-ii layers on cleanly.
