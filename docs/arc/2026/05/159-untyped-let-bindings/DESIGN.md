# Arc 159 — Untyped `let` bindings (V2; user-visible)

**Status:** opened 2026-05-07.

## Lineage

- **Arc 158 v1** (failed; reverted at `5b51b67`) — attempted the
  binding-shape change directly; broke 24 lib unit tests because
  the pair-deadlock walker family read declared `:T` from AST and
  v1's "ignore declared :T at inference" decision left the walker
  without a type-info source. REALIZATIONS at
  `docs/arc/2026/05/158-untyped-let-bindings/REALIZATIONS.md`
  captured the gap.
- **Arc 158a** (shipped at `42a7803`) — precursor stepping stone.
  Walker family migrated to RHS pattern-match (closed-set
  channel-related shapes); accepts both binding shapes. Substrate
  prepared for new-shape bindings; legacy paths unchanged.
- **Arc 159** (this arc) — V2 of the binding-shape change.
  Substrate inference + runtime accept new shape; walker fires
  `LegacyTypedLetBinding` on legacy shape; consumer sweep.

Naming rationale per memory `feedback_v1_backout_dependency_arc.md`:
each arc gets its own number; sub-letter naming (158a/158b) was a
one-time exception; arc 159 restores the standard self-contained
arc pattern.

## Goal

Drop the per-binding type annotation `:T` from `:wat::core::let`.
Each binding's type is inferred from its expression — same lesson
as arc 145 / arc 157 (`def`), applied to the inner-binding slot.

| Before | After |
|---|---|
| `(:wat::core::let (((name :T) expr) ...) body)` | `(:wat::core::let ((name expr) ...) body)` |

The OUTER bindings list `(...)` and the body slot stay unchanged.
Only the inner per-binding shape changes — drop the type-annotation
wrapper around the binder.

End-state user goal: `(:wat::core::let ((x 2)) (:wat::core::+ x 2))`
evaluates to 4 (`:wat::core::i64`).

**Out of scope (deferred to a future arc by user direction):**
Clojure-style square-bracket binding form `[name expr name expr]`.
Arc 159 keeps the existing paren-grouped binding shape; only the
type annotation is removed.

## Migration: clean break (Path A per arc 154/155 precedent)

Substrate accepts new shape canonically; walker fires
`LegacyTypedLetBinding` CheckError on legacy `((name :T) expr)`
shape. No transitional alias for the walker; both shapes parse and
infer cleanly during the migration window so consumer sweep can
run against a working tree.

After consumer sweep: legacy shape no longer appears in any source;
walker has nothing to fire on; arc 159 closes.

## Substrate edits

### `src/check.rs`

1. **`process_let_binding` extension** (line 7570).
   Currently returns early when `kv[0]` isn't a List (silently
   skips new shape). Add new branch:
   - If `kv[0]` is `WatAST::Symbol` → name is bare keyword; infer
     RHS type; populate `out_scope` with name → inferred type.
   - Existing typed-single + destructure branches unchanged.

2. **`LegacyTypedLetBinding` CheckError variant + Display + diagnostic.**
   Mirror arc 154's `BareLegacyLetStar` shape. Variant carries the
   binding name + span; Display references arc 159 + canonical fix
   (`(name expr)`); diagnostic field includes name + canonical-form
   string + location.

3. **`walk_for_legacy_typed_let_binding` walker.**
   Walks all `:wat::core::let` forms; per-binding shape check; if
   binding is `((name :T) expr)` → emit `LegacyTypedLetBinding`.
   Mirror `validate_legacy_let_star` (arc 154) shape; one
   diagnostic per source-level legacy binding.
   Wired into `check_program` after the def-position walker
   (arc 157 precedent).

### `src/runtime.rs`

4. **`parse_let_binding` extension.**
   Current logic accepts `((name :T) rhs)` legacy + destructure.
   Add new branch: if `kv[0]` is `WatAST::Symbol` → return
   `LetBinding::Single { name, rhs }`. Mirror v1 sonnet's approach
   (which was reverted; the new branch is the same shape).

5. **`step_let` extension** (if needed).
   Verify the step path handles new shape. Sonnet reads existing
   `step_let` code to confirm; extends if necessary.

### NEW `tests/wat_arc159_let_bindings.rs`

Test set (10-13, ~250-400 LOC):

**End-to-end new shape (4-5 tests):**
1. **`(:wat::core::let ((x 2)) (:wat::core::i64::+,2 x 1))`** → `3`
   (i64). The user's stated end goal verified at runtime.
2. **Multi-binding sequential**: `(let ((a 1) (b a)) (+ a b))` → 2.
3. **Closure capture through new shape**: `(let ((x 2)) (fn () x))`
   call returns 2.
4. **Type inferred matches RHS**: `(let ((x 2)) ...)` registers `x`
   as `:wat::core::i64`; using `x` where `:String` expected → fails.
5. **Sequential semantics**: `(let ((a 1) (b (+ a 1))) b)` → 2.

**Walker on legacy (3 tests):**
6. Single legacy binding fires walker.
7. Multi-binding all-legacy: walker fires per binding.
8. Mixed (legacy + new) in one let: walker fires only on legacy
   binding(s); new binding(s) work unflagged.

**Destructure preservation (CRITICAL — v1 broke this) (2 tests):**
9. **2-element destructure**: `(let (((a b) p)) (+ a b))` where `p`
   is a pair `(1, 2)` → 3. Verifies the v1 bug doesn't recur:
   `((a b) p)` MUST parse as destructure, NOT as new-shape typed
   binding. Sonnet 1a v1 misread this; arc 159's substrate must
   correctly distinguish.
10. **3-element destructure**: `(let (((a b c) tup)) ...)` —
    arity-3 destructure works.

**Regression on scope-deadlock walker (1-2 tests):**
11. Existing `arc_128_outer_scope_deadlock_still_fires`-style
    pattern in NEW shape — `ScopeDeadlock` (post-inference walker)
    fires correctly because `process_let_binding` now populates
    `extended` for new-shape bindings (closes the gap arc 158a's
    INSCRIPTION named).

## Slice plan

### Slice 1 — substrate

- Edits per § Substrate edits above. ~6 files: `src/check.rs`,
  `src/runtime.rs`, NEW `tests/wat_arc159_let_bindings.rs`. NO
  consumer wat edits. NO embedded-wat edits in src/ test modules
  (forbidden per arc 158 v1 lesson — sonnet-scope-creep).
- 10-13 tests covering end-to-end + walker + destructure + regression.
- Atomic with consumer sweep slices per recovery doc § 7.

### Slice 2 — wat-rs consumer sweep

- Mechanical transform across wat-rs sites: `((name :T) expr)` →
  `(name expr)`. ~951 sites.
- Use Python script `/tmp/wat_let_sweep.py` (from v1; refined to
  protect destructure shapes).
- Atomic commit with slice 1 when wat-rs workspace = 0-failed.

### Slice 3 — holon-lab-trading consumer sweep

- Same sweep across lab repo. ~965 sites.
- Separate atomic commit in lab repo.

### Slice 4 — substrate retirement + closure paperwork

- `walk_for_legacy_typed_let_binding` walker body retired per
  substrate-as-teacher § "Retire the hint" (arc 154/155 precedent).
- `LegacyTypedLetBinding` variant + Display retained as orphaned
  scaffolding (arc 113 precedent).
- INSCRIPTION + 058 changelog row + USER-GUIDE update +
  WAT-CHEATSHEET update.
- Pre-INSCRIPTION grep mandatory per FM 11.

## What v1's lessons explicitly address in arc 159's plan

1. **Destructure preservation:** v1's sonnet swept `(((a b) p))`
   → `((a p))` (treating `(a b)` as legacy typed name). Arc 159
   substrate's `process_let_binding` must correctly distinguish
   destructure (binder children all Symbols) from typed (binder[1]
   is Keyword). v1's `is_typed_single` check WAS correct; sonnet's
   embedded-wat sweep was the bug. Arc 159 BRIEF explicitly
   forbids src/ embedded-wat edits in slice 1; that scope-creep
   is what broke v1.

2. **Walker dependency settled:** arc 158a closed it. Arc 159's
   `process_let_binding` extension finishes the work — populating
   `extended` for new-shape bindings restores full `ScopeDeadlock`
   coverage (arc 158a's INSCRIPTION § Honest deltas point 1).

3. **Sonnet scope creep prevention:** BRIEF explicitly enumerates
   which files sonnet may touch and which are off-limits. The 5-
   file constraint of slice 1 is hard.

## Cross-references

- **Arc 145** (typed-let backout) — paid-for lesson on type-
  annotation redundancy
- **Arc 154** (kill let*) — closest precedent for let-related
  substrate change with walker + sweep
- **Arc 155** (fn rename) — clean-break retirement (no transitional
  alias) precedent
- **Arc 157** (def) — sibling form ships untyped via same lesson
- **Arc 158 v1** — REALIZATIONS at
  `docs/arc/2026/05/158-untyped-let-bindings/REALIZATIONS.md`
  captures the failed first attempt
- **Arc 158a** — INSCRIPTION at
  `docs/arc/2026/05/158a-let-binding-walker-migration/INSCRIPTION.md`
  ships the walker migration that arc 159 builds on
- **Memory `feedback_substrate_already_typed.md`** — paid-for
  lesson driving the no-annotation decision
- **Memory `feedback_stepping_stones_proactive.md`** — slicing
  framework
- **Memory `feedback_v1_backout_dependency_arc.md`** — naming
  pattern: arc 159 (binding-shape v2) ships separately from arc
  158a (walker prep)

## Four questions

- **Obvious?** YES — same lesson as arc 145 / def, applied to
  parallel slot. Walker dependency closed.
- **Simple?** YES — substrate path extends existing branches; new
  walker mirrors arc 154's recipe.
- **Honest?** YES — clean break with walker firing on legacy;
  destructure preservation explicitly tested.
- **Good UX?** YES — Clojure-faithful, less ceremony, consistent
  with `def`.

## Stepping-stones

- **Tractability of next steps?** Arc 159 closes the user-visible
  binding-shape change goal. After 159 ships, future "Clojure
  brackets" arc operates on a settled foundation.
- **Dependencies?** Arc 158a (walker migration) shipped. No
  remaining substrate dependencies.
- **Composition?** Substrate slice + wat-rs sweep (atomic) + lab
  sweep (separate atomic) + closure. Each piece simple.

## Estimated effort

- Slice 1: ~30-45 min Sonnet (substrate + 10-13 tests)
- Slice 2: ~25-40 min Sonnet (~951 wat-rs sites, mechanical via
  Python script; atomic with slice 1)
- Slice 3: ~25-40 min Sonnet (~965 lab sites, cross-repo)
- Slice 4: ~25 min orchestrator (closure paperwork)
- Total: ~2-2.5 hours wall-clock if Mode A clean throughout
