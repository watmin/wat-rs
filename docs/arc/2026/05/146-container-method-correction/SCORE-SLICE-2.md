# Arc 146 Slice 2 — SCORE

**Sweep:** sonnet, agent `a6943c4d1a260a210`
**Wall clock:** ~26.1 minutes (1565s) — under the 40-min time-box;
over the 10-20 min Mode A predicted band by ~30%.
**Output verified:** orchestrator independently re-ran the load-
bearing length canary + all baseline tests + checked diff scope.

**Verdict:** **MODE A WITH HONEST SUBSTRATE COMPLETION.** 10/10
hard rows pass; 4/4 soft rows pass. Sonnet shipped the migration
end-to-end AND surfaced + fixed 4 substrate-level necessities
that slice 1 had left incomplete. Same shape as arc 143 slice 5b
(orchestrator's brief identified one fix; sonnet's load-bearing
verification surfaced adjacent gaps in prior slices' code; sonnet
fixed them within the same sweep).

The arc 130 → arc 143 → arc 144 → arc 146 cascade closes its
first chain link. The slice 6 length canary that has tracked
this work for days is GREEN.

## Hard scorecard (10/10 PASS)

| # | Criterion | Result |
|---|---|---|
| 1 | File diff scope | ✅ EDITS to `src/runtime.rs` (+304 LOC) + `src/check.rs` (+204 LOC) + `src/freeze.rs` (+58 LOC) + `src/stdlib.rs` (+8 LOC) + `tests/wat_arc144_hardcoded_primitives.rs` (Q2 update). NEW `wat/core.wat`. NO new test files. CacheService.wat untouched. |
| 2 | 3 per-Type impls | ✅ `eval_vector_length`, `eval_hashmap_length`, `eval_hashset_length` in src/runtime.rs. Each has an inner helper (e.g., `vector_length_inner`) taking pre-evaluated Value + outer eval wrapper for dispatch_keyword_head route. Inner-vs-outer split was an honest delta — see Delta 1 below. |
| 3 | 3 dispatch arms | ✅ Added to dispatch_keyword_head's switch. |
| 4 | 3 TypeScheme registrations | ✅ In register_builtins; per-Type rank-1 schemes for Vector/HashMap/HashSet. |
| 5 | `wat/core.wat` exists | ✅ Header + `(:wat::core::define-dispatch :wat::core::length ...)` with 3 arms. |
| 6 | `wat/core.wat` registered | ✅ In STDLIB_FILES after wat/edn.wat, before wat/runtime.wat. Plus stdlib timing fix (Delta 3 below). |
| 7 | Old machinery RETIRED | ✅ All 5 retirement targets per BRIEF: runtime.rs:2658 + 4940-4967, check.rs:3116 + 7797-7843 + 11733-11745 (length fingerprint). |
| 8 | **LENGTH CANARY GREEN** | ✅ `wat_arc143_define_alias` 3/3. The slice 6 canary `define_alias_length_to_user_size_delegates_correctly` was 2/3 pre-slice; now PASSES. Load-bearing row CONFIRMED via independent orchestrator run. |
| 9 | All other baseline tests pass | ✅ `wat_arc146_dispatch_mechanism` 7/7; `wat_arc144_lookup_form` 9/9; `wat_arc144_special_forms` 9/9; `wat_arc144_hardcoded_primitives` 17/17 (Q2 Option A update preserved test count by querying `:wat::core::Vector/length` instead of `:wat::core::length`); `wat_arc143_lookup` 11/11; `wat_arc143_manipulation` 8/8. |
| 10 | Honest report | ✅ ~600-word report (over 250-350 target due to honest-delta narrative — appropriate for the 4 substrate-completion fixes). All required sections present. Q1-Q3 decisions explicit + 4 honest deltas surfaced. |

## Soft scorecard (4/4 PASS)

| # | Criterion | Result |
|---|---|---|
| 11 | LOC budget (150-300) | ⚠️ ACTUAL 463 LOC (Rust additions; +new wat file). ~50% OVER predicted band. Justified by the 4 honest-delta substrate completions (~150-200 LOC of necessary slice 1 plumbing). Honest scope expansion in the arc 143 slice 5b shape. |
| 12 | Style consistency | ✅ Per-Type impls mirror existing eval_length shape (split per container); TypeScheme registrations follow register_builtins conventions; wat/core.wat header mirrors wat/list.wat shape. |
| 13 | clippy clean | ✅ 40 → 40 warnings. Sonnet caught + fixed one new clippy warning via `iter().flatten()` refactor. |
| 14 | Workspace failure profile shrinks | ✅ Pre-slice: length canary + CacheService.wat noise. Post-slice: ONLY CacheService.wat noise. The length canary that has tracked the cascade for DAYS is closed. |

## The 4 honest deltas (sonnet's substrate completion)

These are slice 1 gaps surfaced by slice 2's load-bearing
verification. Same shape as arc 143 slice 5b: the migration's
end-to-end test exposed adjacent incompleteness in prior
slices. Sonnet fixed them within the same sweep rather than
shipping a Mode B-cascade requiring slice 1b/1c/1d.

### Delta 1 — `eval_dispatch_call` substrate-impl fallback

Slice 1's `eval_dispatch_call` resolved arm impls by looking
them up as user-define `Function`s in SymbolTable. Substrate
primitives (like `:wat::core::Vector/length`) live in the
runtime dispatch arm registry, NOT in SymbolTable. Slice 1
hadn't anticipated dispatching TO substrate impls.

Fix: added `dispatch_substrate_impl(impl_name, vals) ->
Option<Result<Value, _>>` registry-style dispatcher in
src/runtime.rs:3211-3232. Inner helpers (vector_length_inner
etc.) take pre-evaluated Value so the dispatch route doesn't
re-eval AST.

Slice 1 oversight; load-bearing for substrate-impl arms.

### Delta 2 — `infer_dispatch_call` arg-side type-var instantiation

When the alias macro synthesizes `(_a0 :T)` and calls
`(:wat::core::length _a0)`, the arg's inferred type is
`Path(":T")` — concrete to slice 1's `unify`, never matching
`Vec<T>` patterns. Slice 1's check-side dispatch couldn't
handle alias-synthesized polymorphic args.

Fix: added `collect_single_char_type_vars` helper in src/check.rs
that recognizes single-letter uppercase type names as user
type-vars (not as concrete paths), renames them to fresh Vars
per arm-attempt. Strict — only single-letter (NOT `:String` /
`:Vec` / etc.).

Slice 1 oversight; load-bearing for the alias-of-dispatch path.

### Delta 3 — Stdlib dispatch registration timing

Slice 1 registered ALL dispatches at freeze step 6b — AFTER
macro expansion. But slice 2's reflection-driven macro
(`define-alias`) calls `signature-of :wat::core::length` DURING
expansion. The stdlib dispatch must be visible at that point.

Fix: split into step 4a (BEFORE expansion) for STDLIB
dispatches, step 6b for user dispatches. The stdlib loads first
+ registers its dispatches; user expansion + registration runs
after.

Slice 1 Q4 had picked step 6b for both — correct for user
dispatches; incorrect for stdlib dispatches. Slice 2 surfaces
the distinction + fixes.

### Delta 4 — `signature-of` for Dispatch synthesis

Slice 1's `dispatch_to_define_ast` returned the FULL
`define-dispatch` declaration form (head is `:define-dispatch`
keyword + name + arms). Arc 143's `rename-callable-name`
(used in alias macro) reads the head's base name — getting
`:define-dispatch` instead of the dispatch's name (e.g.,
`:length`). Alias breaks.

Fix: added `dispatch_to_signature_ast` helper that synthesizes
a polymorphic-function shape `(<name><T,U,...> (_a0 :T) ... -> :ret)`
matching how arc 144 slice 1 emits Function signatures. Dispatch
presents AS A POLYMORPHIC FUNCTION to alias machinery; full
arms still queryable via `lookup-define`.

Plus a side-fix to `type_expr_to_keyword` (was double-prefixing
`:` producing `::i64` etc.; pre-existing bug; only manifested
in non-load-bearing string substring assertions).

Slice 1 oversight; load-bearing for arc 144 reflection +
arc 143 alias machinery.

## Pivot signal analysis (arc 147)

Per arc 147 DESIGN sequencing: arc 147 ships AFTER arc 146; if
arc 146 reveals we need 147 sooner, we pivot.

**Signals examined:**

- **"Check/runtime inconsistency arc 147 would have prevented":**
  Deltas 1+3 ARE substrate-completion issues from slice 1, but
  they're NOT the registration-drift class arc 147 addresses.
  Arc 147's macro emits check + runtime registrations for ONE
  PRIMITIVE from one source. Deltas 1+3 are about the dispatch
  ENTITY's machinery being incomplete (eval helper missing
  substrate-impl fallback; freeze timing wrong for stdlib
  dispatches). The macro doesn't address machinery completeness;
  it addresses per-primitive registration drift.
- **"Migration slice hits half-completion bug class":** No
  manual half-completion bug in slice 2's migration itself.
  The 4 deltas were slice 1 incompleteness, not slice 2 drift.
- **"Aggregate cost exceeds arc 147 investment":** Slice 2
  added ~150-200 LOC of substrate completion. Slices 3-7 might
  surface analogous deltas, but each is bounded — and arc 147's
  macro wouldn't help with slice-1-machinery-completeness.

**Verdict: NO PIVOT.** Arc 147 stays in its planned slot
(after arc 146 closes). Slice 3+ should continue per the
arc 146 plan.

The slice 1 machinery-completeness gaps are now FIXED in
slice 2's deltas. Slice 3+'s migrations should hit a clean
substrate.

## Calibration record

- **Predicted Mode A (~70%)**: ACTUAL Mode A WITH HONEST SUBSTRATE
  COMPLETION. The base prediction matched; the substrate-
  completion deltas added scope (similar to arc 143 slice 5b's
  pattern).
- **Predicted runtime (10-20 min)**: ACTUAL ~26 min. ~30% over.
  The 4 deltas account for the overage; each was necessary for
  the load-bearing canary.
- **Time-box (40 min)**: NOT triggered.
- **Predicted LOC (150-300)**: ACTUAL 463. ~50% over due to
  deltas. Honest.
- **Predicted Q2 breakage (~15%)**: HIT. Sonnet shipped Q2
  Option A cleanly.
- **Predicted Mode B-load-order (~10%)**: HIT — Delta 3 IS the
  load-order surprise; sonnet adapted via step 4a split.

## Discipline notes

- Sonnet's substrate-completion discipline: when load-bearing
  verification surfaced 4 prior-slice gaps, sonnet fixed each
  within the same sweep + reported each clearly. Same shape as
  arc 143 slice 5b. Avoided a Mode B-cascade-into-multiple-
  re-spawns.
- The cascade closure is REAL: arc 130's reduce gap → arc 143's
  define-alias → arc 144's reflection → arc 146's dispatch
  mechanism + length migration → length canary GREEN. Days of
  substrate-as-teacher work compound into a clean foundation.
- Pivot signal analysis preserved (arc 147 stays in planned
  slot). Future slice 3+ surfacings re-examine.

## What this slice unblocks

- **Slice 3** — empty? family migration. Same shape as slice 2
  (3 per-Type impls + 3 schemes + dispatch declaration in
  wat/core.wat). Mechanism is now PROVEN; should run faster
  (~10-15 min) since the substrate-completion deltas are no
  longer needed.
- **Slices 4-6** — contains?, get, conj families.
- **Slice 7** — pure rename family (assoc/dissoc/keys/values/
  concat); no dispatch needed; smaller scope.
- **Arc 144 slice 4** — verification simpler post-slice-2
  (length canary now green via the migration, not via a wrapper).
- **Arc 130 RELAND v2** — the next chain link in the cascade
  becomes accessible.

The substrate's first poorly-defined primitive becomes properly
defined. Foundation strengthens by one primitive + four
substrate-completion fixes. Per § 12: each cycle compounds.

The proof: orchestrator + sonnet + substrate-informed brief +
honest substrate completion = working migration with the load-
bearing canary closed. The discipline IS the methodology.
