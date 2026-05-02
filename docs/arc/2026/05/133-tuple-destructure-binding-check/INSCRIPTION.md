# Arc 133 — INSCRIPTION

## Status

**Shipped + closed 2026-05-02** in commit `f717f15`. Single-
slice arc; sonnet sweep `a293ba381e3b1a32f` (~25 min). Slice 1
SCORE: 8/8 hard + 3.5/4 soft. The honest delta surfaced by the
sweep — `unify`-via-`reduce` alias expansion required adding
`rust::crossbeam_channel::Sender` to `type_contains_sender_kind`
— produced four newly-firing integration tests that **arc 134**
closed via origin-trace + body-form narrowings.

## What this arc closes

Pre-arc-133, `parse_binding_for_typed_check` (arc 117/131) and
`parse_binding_for_pair_check` (arc 126) only recognised the
typed-name binding shape `((name :type-keyword) rhs)`. Tests
using untyped tuple-destructure bindings — `((name1 name2 ...)
rhs)` — were silently skipped by both structural-deadlock
checks. Bindings could be in a deadlock-prone shape and the
checks wouldn't see them; correct-by-accident, not correct-by-
construction.

Arc 131 slice 2's sonnet sweep surfaced this in the SCORE doc:

> The slice-1 check's `parse_binding_for_typed_check` skips
> untyped tuple destructure (`((pool con-driver) ...)`), so
> several tests were also shielded by that bypass even when
> their structural shape was non-canonical.

Arc 133 closes the bypass: post-arc-133, the deadlock checks
see every binding shape uniformly.

## What shipped

### In-place post-inference deadlock check

Sonnet picked the in-place approach over the alternative
walker-with-CheckEnv path. After `process_let_binding` runs
for every binding in a `let*`, the `extended` HashMap holds
the inferred TypeExpr for every bound name — typed-name AND
tuple-destructure shapes uniformly. The new function reads
from that map directly:

`src/check.rs::check_let_star_for_scope_deadlock_inferred`
— called from `infer_let_star` after the binding-processing
loop. Classifies every name in `extended` (filtered to those
introduced by THIS let*'s bindings) via `type_is_thread_kind`
/ `type_contains_sender_kind`. Fires `ScopeDeadlock` on
sibling Thread + Sender + `Thread/join-result` in body or
sibling RHS. Same shape as the retired structural walker;
new input is the inferred-types map.

### ChannelPairDeadlock walker extension

`src/check.rs::extend_pair_scope_with_tuple_destructure` —
extends the existing `walk_for_pair_deadlock` (arc 126) to
attach synthetic pair-scope entries for tuple-destructure
bindings. The trace through `(first|second pair)` projections
now also resolves through `((tx rx) (make-bounded-channel
...))` direct destructure with a shared anchor name. When a
helper call receives both `tx` and `rx`, the trace finds the
same anchor → `ChannelPairDeadlock` fires.

### Structural walkers retired

`validate_scope_deadlock`, `walk_for_deadlock`,
`check_let_star_for_scope_deadlock`, and
`parse_binding_for_typed_check` all marked
`#[allow(dead_code)]` with retirement notes. The `check_program`
call sites were retired with explanatory comments redirecting
to the inference-time check.

`validate_channel_pair_deadlock` (arc 126) was NOT retired —
it remains the pair-deadlock entry point, extended via the
new helper rather than replaced.

### `type_contains_sender_kind` extension

The `unify`-via-`reduce` alias expansion path canonicalises
`:wat::kernel::Sender` → `:rust::crossbeam_channel::Sender`
inside compound types before binding the unification variable.
Inferred types in `extended` therefore carry the expanded form.
The surface-match arm gained the `rust::crossbeam_channel::Sender`
head alongside `wat::kernel::Sender` and `wat::kernel::Channel`
to recognise both spellings.

This extension was the "honest delta" sonnet flagged in the
slice-1 SCORE — and it IS what surfaced the rule-precision gap
arc 134 closed. See arc 134 INSCRIPTION for the chain.

### Required unit tests

Four new tests in `src/check.rs`:

- `arc_133_typed_name_binding_still_fires` — regression guard;
  the typed-name shape continues to fire `ScopeDeadlock` after
  walker retirement.
- `arc_133_tuple_destructure_with_handlepool_fires` — the new
  path: `((pool driver) (some-spawn-fn))` where the spawn
  returns `(HandlePool<...>, Thread<...>)` fires correctly
  with `offending_kind = "HandlePool"`.
- `arc_133_tuple_destructure_silent_when_clean` — negative
  guard; tuple-destructure where neither element is Sender-
  bearing does not fire.
- `arc_133_tuple_destructure_pair_check_fires` — sibling
  check: `((tx rx) (make-bounded-channel ...))` direct
  destructure followed by a helper call passing both halves
  fires `ChannelPairDeadlock`.

(Two of the four — `arc_131_handlepool_with_sender_fires` and
`arc_133_typed_name_binding_still_fires` — were updated by arc
134 to use recv-bearing lambda bodies, since the empty-body
fixtures stopped modelling the deadlock-prone shape after arc
134's body-form narrowing.)

## What got surfaced

### Arc 134 — rule precision

Sonnet's slice-1 SCORE predicted zero newly-firing tests in
`wat-tests/`. Accurate for that scope — but missed
`tests/wat_*.rs` (Rust integration tests with embedded wat
source strings). Four tests in those files newly fired:
three in `wat_spawn_lambda` (canonical Thread<I,O> usage —
`Sender` from `Thread/input thr` sibling to `thr`) and one in
`wat_typealias` (parent-allocated channel + closure with no
recv).

The user diagnosed this as a logic error, not a coverage gap:
the rule fires on type-coexistence without checking whether
the Sender's pair-Receiver is actually held by a recv-loop in
the spawned function. Arc 134 closed the precision gap with
two structural narrowings (origin-trace + body-form). The
arc 133 substrate work itself stayed; arc 134 sharpened its
behavior on the calibration set.

This continues the failure-engineering chain: arc 131 slice 2
SCORE surfaced arc 133 (binding-shape bypass); arc 133
workspace failures surfaced arc 134 (rule precision). Each
substrate-fix arc closes a gap surfaced by the previous arc.

## The four questions

**Obvious?** Yes. The rule's binding-parser was typed-name-only;
extending it to tuple-destructure was the natural completion.

**Simple?** Medium. ~470 LOC delta in `src/check.rs` (518+/42-).
The in-place approach reused inference's already-computed
per-binding types; no new walker passes; no plumbing
refactors. Walker retirement was clean.

**Honest?** Yes. Sonnet's SCORE surfaced the alias-expansion
delta as a load-bearing observation — and the workspace
failures it produced led directly to arc 134. The chain stayed
honest about scope and consequences.

**Good UX?** Phenomenal for the binding-shape coverage. The
precision side was carried by arc 134.

## Cross-references

- `DESIGN.md` + `BRIEF-SLICE-1.md` + `EXPECTATIONS-SLICE-1.md`
  + `SCORE-SLICE-1.md`
- `docs/arc/2026/05/131-handlepool-scope-deadlock/SCORE-SLICE-2.md`
  § "Latent gap surfaced" — the SCORE that surfaced arc 133.
- `docs/arc/2026/05/134-scope-deadlock-origin-trace/INSCRIPTION.md`
  — the precision arc that closed the rule-precision gap arc
  133's `rust::` extension exposed.
- `docs/arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md`
  + `docs/arc/2026/05/131-handlepool-scope-deadlock/INSCRIPTION.md`
  — the parent rules.
- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/INSCRIPTION.md`
  — the sibling rule arc 133 also extended.
- `src/check.rs::check_let_star_for_scope_deadlock_inferred`
  — the in-place check.
- `src/check.rs::extend_pair_scope_with_tuple_destructure` —
  the arc 126 walker extension.
- `src/check.rs` retired functions (`#[allow(dead_code)]`):
  `validate_scope_deadlock`, `walk_for_deadlock`,
  `check_let_star_for_scope_deadlock`,
  `parse_binding_for_typed_check`.
