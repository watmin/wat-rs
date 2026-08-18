# MEASURED — 118.B2c strike 1's census. ⛔ STOP-1 FIRES — but NOT on "violators".

> ## ⛔ CORRECTION, 2026-08-18, same day, before any fix
>
> **This document first called `wat/bracket.wat`'s two sites "offenders" and the commit called them
> "violators". BOTH LABELS ARE WRONG, and the record said so all along.** `wat/bracket.wat:314-316`
> documents the design precisely, names the mechanism, and cites the runtime line:
>
> > *"The `:wat::core::keyword` clause is declared FIRST — defclause dispatch is first-match-wins and
> > a Fn/generic-`W` clause is a PERMISSIVE catch-all at runtime (`value_matches_type_by_name`,
> > runtime.rs:6604-6606), so ordering is load-bearing: the keyword clause must be checked before the
> > generic one or it would never fire."*
>
> Verified this session, with a reversed-order control: **both the checker and the runtime are
> first-match-wins in declaration order, and they AGREE** — no type-safety hole, nothing malformed,
> nothing broken. This is a deliberate, understood, documented design by someone who read the
> dispatcher before writing it.
>
> What the census actually found is **code the ruling would OUTLAW** — which is a fact about the
> ruling, not a defect in the code. `[[feedback_a_superseded_design_looks_exactly_like_a_broken_check]]`
> ★ And the sharper version: **the record DID discriminate, in a comment naming the exact mechanism,
> and I labelled it a violation without reading it.**
>
> The count below (3 pairs) is accurate. Every use of "offender"/"violator" for the `bracket.wat`
> rows is not.

**Run 2026-08-18** over **1,457 corpus `.wat` files** with
`wat-scripts/scratch-pad/census-defclause-arm-overlap.wat` (form tree, recursive — a `defclause` can
sit inside a `do`). **71 distinct `defclause`s · 219 arm rows.**

## The result

```
★ UNGUARDED OVERLAPPING PAIRS ......... 3
    :my::pick                    tests/types/probe_stone_118_b2c_overlapping_arms_are_silent.wat
    :wat::bracket::thread-enter        wat/bracket.wat        ← PRODUCTION STDLIB
    :wat::bracket::process-work-forms  wat/bracket.wat        ← PRODUCTION STDLIB

  overlapping pairs where an arm carries a :guard (NOT auto-flagged) ... 4
```

`:my::pick` is **this stone's own witness**, committed hours earlier to characterise the defect — the
one row here that IS an ambiguity, and the thing the wall is supposed to refuse.

**The other TWO are `wat/bracket.wat`, they are the same shape, and they are CORRECT.**

## ⛔ THE FINDING — the corpus ALREADY USES first-match-wins AS SPECIFICITY

```wat
(:wat::core::defclause :wat::bracket::thread-enter
  ([self <- :wat::kernel::ThreadSelfPeer<…>  work-fn <- :wat::core::keyword] -> :nil   ; CONCRETE
   (:wat::bracket::thread-kwargs-runner self work-fn :wat::core::None))
  ([self <- :wat::kernel::ThreadSelfPeer<…>  work-fn <- :W] -> :nil                     ; WILDCARD
   (:wat::bracket::runner-loop self …)))
```

`:W` is a bare uppercase type-var, and `value_matches_type_by_name`'s `is_type_var` branch returns
**true for any value**. So arm 2 matches everything, including a keyword. **The concrete arm is
declared FIRST and the wildcard arm is the FALLBACK** — this is *specific case, then generic case*,
and it is correct today for exactly one reason: dispatch is first-match-wins in declaration order.
`:wat::bracket::process-work-forms` is the same pattern with one parameter.

★ **AND THE ORDERING DEPENDENCE IS DOCUMENTED IN THE SOURCE** (`wat/bracket.wat:314-316`, quoted in
the correction at the top). This is not incidental reliance that happens to work — it is a design
whose author read `value_matches_type_by_name`, cited its line number, and wrote down that ordering
is load-bearing.

**This refutes the premise the ruling rests on.** `DESIGN-STONE-118.B2c` affirmatively rejected
clause specificity, saying: *"If a verb later needs a fast concrete path plus a generic fallback,
that is a NEW ruling on whether the language has specificity at all."* The census says **two verbs
already do, today, in the shipped stdlib.** Specificity is not a hypothetical future want — it is a
live, load-bearing dependency of the thread-pool bracket machinery.

**A wall that refuses overlapping arms cannot be armed at zero offenders. It would refuse
`wat/bracket.wat`.**

## What is NOT decided here

Per **STOP-1**: *"Do NOT arm the wall over them and do NOT 'fix' them silently. Report the list;
each one is a live ambiguity whose disposition is the builder's."* The list is above. The disposition
is not taken.

The shapes available, stated without preference:

⚠ **A LIMIT OF THIS INSTRUMENT, now visible:** the census cannot tell a DELIBERATE documented
fallback from an ACCIDENTAL ambiguity. It reports both identically, and I read both as the second.
If a wall is ever built, it needs that distinction — either specificity makes it moot, or deliberate
fallbacks need an explicit marker.

1. **Rule specificity IN** — most-specific-wins becomes language semantics, `bracket.wat` is already
   conformant, and the wall narrows to refusing only *equally*-specific overlaps (which is exactly the
   redef case, `:my::pick`).
2. **Rule specificity OUT and migrate the two sites** — e.g. an explicit dispatch inside one arm. The
   wall then arms at zero offenders as originally planned, and `bracket.wat` changes first.
3. **Rule that a type-var param is not a wildcard at dispatch** — a different fix entirely, aimed at
   `is_type_var` rather than at overlap. Would break both sites loudly rather than silently.

Each needs its own four questions, and **(1) versus (2) is a dialect ruling**, the same kind as the
three doors.

## ⚠ The instrument caught its own bug — recorded so the count is auditable

The FIRST run reported **4** unguarded pairs. One was `:p05::pick` with `[]` vs `[]` — two EMPTY
type-lists, which is absurd on its face and is what exposed the defect. A `defclause` may carry a
**shared return type on its head line**:

```wat
(:wat::core::defclause :p05::pick -> :wat::core::i64
  ([x <- :i64] x)
  ([x <- :i64  y <- :i64] (…)))
```

The census took "every child past the name" as an arm, so `->` and `:wat::core::i64` were counted as
two arms with no binders. Fixed: an arm is a child that HAS children AND whose first child is a
binder vector. `:p05::pick`'s real arms are arity 1 and arity 2 — **no overlap**. Corrected count:
**219 rows (was 223), 3 unguarded pairs (was 4)**.
`[[feedback_a_file_count_is_not_an_item_count]]`

## ⚠ What the instrument can see — state this before quoting the count

- **It reads SOURCE.** A macro-emitted `defclause` appears unexpanded, so macro-generated arm lists
  are NOT resolved. None appeared in these rows, but the census cannot prove their absence.
- **It reports `:guard` and does not interpret it.** A guard narrows an arm's domain
  (`ClauseFailureReason::GuardFalse` is a real dispatch outcome), so the 4 guarded pairs are not
  ambiguities on their face — they are the language's *existing* same-types-different-domains
  mechanism, and they are the builder's to read.
- **The overlap predicate is CONSERVATIVE by construction** — it models every wildcard path in
  `value_matches_type_by_name` (`is_type_var`, `Fn`/`Tuple` via `_ => true`, and a `Parametric` head
  outside the seq-container set via `None => true`). It over-reports rather than under-reports, which
  is the correct bias for a wall census.
- **Positive-controlled**: it reproduces the known offender (`:my::pick`) and passes the healthy
  three-arm `:my::describe` and the shared-return `:p05::pick`. A zero from it would have meant
  something. It did not return zero.
