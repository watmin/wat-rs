# Arc 158 — REALIZATIONS (v1 attempt back-out)

**Inscribed 2026-05-07.** Non-shipping: arc 158 v1 substrate +
sweep landed but was reverted post-verification. v2 attempt
needs the discoveries below baked into the DESIGN before
re-spawning.

## What v1 tried

Per the original DESIGN: substrate accepts new untyped binding
shape `(name expr)` alongside legacy `((name :T) expr)`; walker
fires `LegacyTypedLetBinding` on legacy sites; per arc 145
lesson the declared `:T` is **IGNORED** at inference (inferred
type from RHS is what gets registered). Atomic substrate +
sweep per arc 154 / 155 precedent.

## What v1 broke

After 1a substrate + my Python sweep landed, workspace went
from 2029 / 0 to 757 / 24 failures. ALL 24 failures were in
lib unit tests inside `src/check.rs` and `src/runtime.rs`. The
24 failures decomposed cleanly:

### Category 1 — Scope-deadlock walker depends on declared type

Tests that broke: `arc_128_outer_scope_deadlock_still_fires`,
`arc_131_handlepool_with_sender_fires`,
`arc_133_tuple_destructure_pair_check_fires`,
`arc_133_typed_name_binding_still_fires`,
`arc_133_tuple_destructure_with_handlepool_fires`,
`arc_134_parent_allocated_channel_still_fires`,
`channel_pair_deadlock_fires_on_canonical_anti_pattern`,
`channel_pair_deadlock_diagnostic_substring`,
`drop_accepts_sender_returns_unit`,
`drop_accepts_receiver_returns_unit`,
`queue_roundtrip_via_destructure_and_send_recv`,
`try_recv_on_empty_queue_returns_none`,
`try_recv_on_ready_queue_returns_some`.

**Failure shape:** test asserts `ScopeDeadlock` (or related) error
fires; under v1 substrate the error vec is empty. The test's
embedded wat had its bindings transformed from
`(((pair :wat::kernel::Channel<wat::core::i64>) ...) ...)` →
`((pair ...) ...)` (sonnet 1a's test-body sweep). The
pair-anchor walker apparently relies on the DECLARED type for
identity tracking, not on the inferred type.

**Substrate gap:** the scope-deadlock walker family
(arcs 117 / 126 / 128 / 131 / 133 / 134) reads
let-binding-declared-type for pair-anchor identity tracking.
Sonnet 1a's "ignore declared :T at inference" change accidentally
also removed the type-info source those walkers rely on.

### Category 2 — Sonnet 1a's wrong test-body sweeps (destructure mangled)

Tests that broke: `let_star_destructures_a_pair`,
`let_destructure_requires_tuple`.

**Failure shape:** sonnet 1a swept `(((a b) p))` → `((a p))` in
the test bodies — treating `(a b)` as a legacy typed-name pair
(name=a, type=b). But `(a b)` is a destructure pattern (b is a
Symbol, not a Keyword). The substrate code's
`is_typed_single = ... && matches!(&binder[1], WatAST::Keyword(_, _))`
correctly distinguishes the two; sonnet 1a's separate sweep
edit DID NOT.

### Category 3 — Tests asserting now-impossible REJECTION

Tests that broke: `bare_single_let_binding_rejected`,
`let_binding_with_any_type_rejected`,
`typed_let_binding_wrong_type_rejected`,
`typed_let_binding_lambda_declared_wrong_rejected`,
`typed_let_binding_with_lambda_value`.

**Failure shape:** these tests asserted OLD substrate REJECTION
behavior — `(name rhs)` was rejected (now accepted), declared
`:T` mismatch was rejected (now ignored), `:Any` was rejected
(now there's no `:Any` slot), etc. Per arc 158, these are
expected behavior changes; the tests need UPDATING (assert
walker firing OR delete) not preservation.

### Category 4 — let* embedded references + macro template

Tests that broke: `step_let_star_substitute`,
`step_let_star_peel_first`,
`template_identifier_carries_macro_scope`,
`step_round_trip_agrees_with_eval_ast`.

**Failure shape:** these tests used legacy let-binding shapes in
their embedded wat for unrelated purposes (testing step semantics
or macro-scope behavior). Sonnet 1a swept these too; some swept
correctly, some hit Category 1 / 2 patterns.

## What sonnet 1a did wrong

Sonnet 1a's report claimed substrate-only edits to
`process_let_binding`, `parse_let_binding`, `step_let`. Diff
review post-revert showed sonnet 1a ALSO transformed embedded
wat in unit tests within `src/check.rs` and `src/runtime.rs` —
scope creep beyond the BRIEF.

This was undeclared additional work. Some edits were correct
(legitimate `((x :T) expr)` → `(x expr)`); some were wrong (the
Category 2 destructure mangling).

## What v2 needs in the DESIGN

### Pre-flight gap audit (NEW)

Before any substrate edit, audit every reader of
let-binding-declared-type:

- **Pair-anchor scope-deadlock walker** (arc 117 / 126 / 128 /
  131 / 133 / 134) — `extend_pair_scope_with_tuple_destructure`,
  `walk_for_pair_deadlock`, related. Read sites in `src/check.rs`
  near lines 1529, 2083, 2756, 2814, 2846, 3042, 3053, 3055,
  3078, 3229, 3251.
- **`process_let_binding`** itself (already in scope)
- **Other infer arms** that may consult the declared type for
  any reason
- **Macro-expansion** sites that re-emit let bindings

For each: determine whether the walker NEEDS the declared type
for non-inference purposes. If yes, the v2 substrate must
**parse** the declared type (don't reject the legacy shape) and
make it available to those walkers via the same path it
currently uses — but inference still uses the inferred-from-RHS
type per arc 145 lesson.

This is more nuanced than v1's "ignore the declared type." The
correct framing: at inference time, declared `:T` is REDUNDANT
(arc 145 lesson); at walker time, declared `:T` carries
identity information that some walkers track. v2 must preserve
the latter while removing the former's redundancy.

OR: make those walkers work off inferred type instead of
declared type. This is a substrate refactor, possibly larger
than arc 158 itself.

### Strict scope on consumer sweep

Sonnet 1a's test-body scope creep needs to be explicitly
forbidden in v2 BRIEF: the substrate sweep is for the substrate
LOGIC ONLY. Consumer test-body sweeps (including embedded wat
in unit tests) belong to slice 1b. Slice 1a sonnet must NOT
edit embedded wat strings in its 5-file edit set.

If 1a sonnet's substrate change makes existing unit tests fail,
that's EXPECTED — slice 1b will fix them. 1a sonnet should
verify this on a single test (run one test to confirm it fails
with the EXPECTED diagnostic, not with substrate panic) but
NOT proceed to fix all the failing tests.

### Re-evaluate the slice ordering

v1 plan:
- 1a substrate (atomic with 1b)
- 1b consumer sweep (atomic with 1a)
- 1c lab sweep (separate atomic commit)
- 2 closure

v2 may need to add a slice 0 — the walker audit. If the audit
reveals the scope-deadlock walker truly needs declared-type
tracking, v2 substrate may need additional work to migrate
those walkers off the declared-type path FIRST (arc 158a),
THEN the let-binding shape change (arc 158b). The slicing
becomes a chain: walker migration → binding shape change →
sweep.

## What stays valid from v1

- The end goal: drop per-binding `:T` for inference-suffices
  reasons (arc 145 lesson)
- The Clojure-faithful direction
- The walker / Pattern 3 substrate-as-teacher recipe
- The atomic-commit-across-coordinated-sweeps pattern (recovery
  doc § 7) — substrate dirty + sweep dirty + atomic commit when
  green

## Discipline lessons

1. **Walker-vs-binding-shape audit must be a pre-flight item**
   when changing the meaning of binding components. The DESIGN
   should explicitly enumerate downstream consumers of any
   shape-component being changed.
2. **Sonnet sweep scope creep** is a real risk on multi-file
   substrate slices. Future BRIEFs should explicitly forbid
   touching embedded wat outside the named files (or expand
   the named files to include the swept ones — but with
   explicit awareness).
3. **Verify destructure detection** is in arc 158's specific
   blast radius. Any future substrate change touching
   `process_let_binding` / `parse_let_binding` must include a
   destructure round-trip test in slice 1a.
4. **The proactive stepping-stones discipline (recovery doc § 5)
   would have caught this** if applied at the audit level: "is
   the next step (consumer sweep) more tractable IF we audit
   walkers first?" — answer: YES; the walker audit IS a
   stepping stone v1 skipped.

## Current state

- HEAD: `d2baf5d` (BRIEF + EXPECTATIONS for 1a/1b; no substrate)
- Working tree: clean
- Workspace: 2029 / 0 / 0 warnings restored
- Scratch script `/tmp/wat_let_sweep.py` from v1 attempt is
  still around; v2 may reuse the parser logic but with
  destructure-shape protection added (require type-expr
  starts with `:` to be considered legacy typed binding —
  the same check the substrate's `is_typed_single` does)

## What to do next

User direction needed on:
- v2 timing (now, or after a different arc?)
- v2 shape (clean break vs walker-migration-first vs other)
- Whether to keep the BRIEF / EXPECTATIONS docs (v1) or amend

Per `feedback_inscription_immutable.md`: v1's DESIGN +
BRIEF-1a + EXPECTATIONS-1a + BRIEF-1b + EXPECTATIONS-1b stay
as historical record (already pushed in commits 7805b76 +
b46adea + d2baf5d). v2 work opens new artifacts.
