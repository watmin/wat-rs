# Arc 170 slice 1b — EXPECTATIONS

## Independent prediction

**Predicted runtime band: 60-120 minutes (opus agent).**

Reasoning:
- Algorithm stays unchanged from slice 1 (free-symbol walker,
  dep closure, capture encoding, portability check, topological
  sort all already correct)
- Spec doc CLOSURE-EXTRACTION.md v2 already drafted; this slice
  implements against existing spec
- Reshape work is localized:
  - `ClosurePackage` shape: 1 site
  - Entry resolution branch: 1 site (inline-lambda + keyword-path
    cases)
  - Assembly path: 1 site (no longer appends entry as trailing
    define)
  - Synthetic-name counter machinery removal: 1 site
  - fn-Value → fn-form-AST reconstruction: small new helper
- Tests stay structurally; assertions update (15 integration +
  drop 1 unit + adjust 1 unit = 16 → 15 net tests after slice 1b)

**Time-box (2× upper-bound): 240 minutes.** If opus still
iterating at 120 min, in-flight check; hard cap at 240.

## Scorecard

| Row | What to verify | Pass criterion |
|-----|----------------|----------------|
| **A — DESIGN-intent alignment** | Does the shipped `ClosurePackage` shape honor DESIGN's "the fn IS the program" intent (DESIGN.md lines 102-108 + 484-509)? Public surface contains NO entry-keyword ceremony at the Rust API level. | ✓ |
| B — `ClosurePackage` reshape | `pub struct ClosurePackage { pub prologue: Vec<WatAST>, pub entry_form: WatAST }`. NO `entry: String` field. | ✓ |
| C — Synthetic-name machinery retired | `:__closure::__pkg_<n>` counter + wrap-in-define logic removed from `closure_extract.rs`. grep for `__pkg_` returns nothing in slice-1b state. | ✓ |
| D — Inline-lambda input emits fn-form AST | For inline-lambda input, `pkg.entry_form` is the reconstructed fn-form AST `(fn [params] -> :T body...)` matching the input fn's signature. NOT a Symbol. NOT wrapped in any define. | ✓ |
| E — Keyword-path input emits Symbol AST | For keyword-path input, `pkg.entry_form` is a Symbol AST whose name matches the input keyword. The user's existing define stays in `pkg.prologue` as a regular dep. | ✓ |
| F — Prologue contains no entry-define | `pkg.prologue` does NOT contain a trailing `(:wat::core::define :__closure::__pkg_*  ...)` form. For keyword-path input, the user's define IS in prologue (as a dep), but it's not "the entry" — it's a regular dep that `entry_form`'s Symbol references. | ✓ |
| G — Body rewrite preserved | Captured local references in the fn body are still rewritten to point at capture-binding defines. The rewrite happens BEFORE `entry_form` is set; `entry_form` carries the rewritten AST. | ✓ |
| H — All 15 integration tests pass | `tests/wat_arc170_closure_extraction.rs` T1-T15 all green with updated assertions. Behavior-equivalence pattern uses the eval-then-apply consumer flow (freeze prologue → eval entry_form → apply). | ✓ |
| I — In-module unit tests | Synthetic-name uniqueness test DROPPED. Capture-name prefix test STAYS (capture-binding naming is unchanged). Net unit-test count 1 (down from 2). | ✓ |
| J — Workspace stays clean | `cargo test --release --workspace` shows `passed: 2107 failed: 0` (was 2108 pre-slice-1b; -1 from dropping the synthetic-name uniqueness unit test). | ✓ |
| K — Capture-binding naming unchanged | `__captured_X` prefix machinery (or whatever convention slice 1 used) stays. Capture binding is a separate concern from entry naming. | ✓ |
| L — Slice branch on remote | `arc-170-program-entry-points` carries slice 1b commit(s) + this scorecard; main untouched. | ✓ |
| M — Zero Mutex usage | no Mutex / RwLock / CondVar introduced (zero-mutex doctrine). | ✓ |
| N — No wat-level surface added | `extract_closure` is Rust-public; not registered in wat eval dispatch. (Same as slice 1.) | ✓ |
| O — No spawn-process / spawn-thread / fork-program changes | Slice 2's territory; slice 1b leaves invocation paths alone. | ✓ |
| P — No `:user::main` signature changes | Slice 2's territory. | ✓ |
| Q — SCORE-SLICE-1.md untouched | per `feedback_inscription_immutable.md` — slice 1's SCORE stays as historical record of the deficiency that surfaced. NOT amended. | ✓ |

## Honest-delta categories (if surfaced, report; don't bridge)

- **Q-impl-2 captured-fn-value gap (slice 1 honest delta A).**
  Still applies post-reshape. If a real consumer needs
  closure-of-closure recursive sub-extraction, surface as honest
  delta — not slice 1b's territory.
- **Value-kind encoding gaps (slice 1 honest delta C).** Still
  applies (HolonAST, WatAST, RustOpaque, holon::Vector, Instant,
  Duration). Not slice 1b's territory.
- **fn Value → fn-form AST reconstruction.** New territory in
  slice 1b. The fn Value's `Function::body` is HolonAST; params
  + ret_type need to be re-emitted as the fn-form's signature.
  Look for an existing helper before writing one. If the
  substrate has a `function_to_form` or similar, use it. If
  not, the inline reconstruction is small mechanical work.
  Surface if you find this surprisingly hard.
- **Test fixture migration shape.** T1-T15's existing assertions
  reference `pkg.entry` and the entry's defining form in
  `pkg.forms.last()`. Migrating to `pkg.entry_form` shape might
  surface ergonomics issues with the WatAST equality assertions
  (matching fn-form ASTs is more involved than matching strings).
  If T1-T15 update needs a helper for fn-form-AST shape matching,
  surface — that's reasonable.
- **Body-rewrite ordering.** Slice 1's body rewrite operates on
  the AST INSIDE the synthesized define. Slice 1b emits the
  rewritten AST as `entry_form` directly. Verify the rewrite
  produces the right AST and `entry_form` carries the rewritten
  version. If the rewrite was tightly coupled to the
  define-wrap, surface.
- **FM 5 trap.** Same rule as slice 1. If a TODO is tempting, STOP.

## Calibration row

Actual runtime: ___ minutes (Mode A clean / Mode B partial /
Mode C failed). Compare to predicted 60-120 min band.

Reshape sites touched:
- `ClosurePackage` definition: ___ lines changed
- Entry resolution: ___ lines changed
- Assembly path: ___ lines changed
- Synthetic-name machinery removal: ___ lines deleted
- fn-form-AST reconstruction: ___ lines added (helper or inline)
- Test assertion updates: ___ lines changed (T1-T15)
- Unit test dropped: ___ lines deleted

Honest deltas surfaced: ___ (count + brief).

## What's next (orchestrator-side, post-slice-1b)

When slice 1b ships green:
- SCORE-SLICE-1B.md authored + committed (with explicit DESIGN-
  intent alignment row scoring)
- Slice 2 BRIEF + EXPECTATIONS updated to reference the corrected
  ClosurePackage shape (currently they reference v1's
  `{ forms, entry }` — that update happens after 1b ships, per
  recovery doc § FM 6 "no speculative DESIGN/BRIEF updates")
- Slice 2 spawn proceeds against the corrected foundation

## SCORE artifact

Opus's report writes to chat; orchestrator commits SCORE-SLICE-1B.md
to slice branch after scoring all rows + reviewing the diff +
re-running the inline pipeline locally for FM 9 verification.

The SCORE-SLICE-1B.md will document this as the SLICE-1 RESHAPE
correction — frame it as forward progress, not retroactive
revision. SCORE-SLICE-1.md stays untouched (immutable historical
record of the deficiency that surfaced).
