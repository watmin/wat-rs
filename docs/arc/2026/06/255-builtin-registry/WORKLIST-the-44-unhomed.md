# WORKLIST — the 44 names `intrinsic_meta` still answers for. As of 2026-08-30, after total-T5.

> **Builder, 2026-08-30:** *"annihilation is our greatest joy… reap the fields to see what remains?
> …we have more homes to build… but we must remove the bridges we no longer need. Ensure our work
> list is on disk — do not forget these as we move forward."*

This file is that list. **It is REPLACED in place, never appended.** Every row retires when its
verb gets a registration site, and when the last row goes, `intrinsic_meta` has nothing left to say.

## Where this came from

`total-T5` (`1d4a53349`) made `intrinsic_meta` derive purity, determinism and totality from the
registry for any verb the registry knows. Measured immediately after, by probing the registry
itself:

```
177  names still literally present in intrinsic_meta
133  SHADOWED — registered, so the derivation answers FIRST. Unreachable dead text.
 44  LIVE — the registry has no entry, so the hand-list still answers.
```

⛔ **The 133 are not inert.** A verb added to that list later would be silently ignored, because the
registry answers before it. That is the graveyard-reading-as-live-code hazard sitting inside the
file we just made derive — and deleting them is the stone this worklist accompanies.

## ★ THE 44, SORTED BY WHAT THEY ACTUALLY ARE — measured, not assumed

The naive reading is "44 unhomed verbs." **It is not.** Probed with the real binary
(`wat-scripts/scratch-pad/255-probe-are-the-orphans-live.wat`):

### A — dispatched verbs awaiting a home (34)

Each has a literal dispatch arm and a census row. This is ordinary homing work.

```
INTRINSIC-READY (18)   = · List? · aggregate-new · bool::to-string · contains? · filter · foldl
                         get · kwargs-construct · map · mapv · not · not= · record? · str
                         stream->pvec · stream->vec · u8
NEEDS-SHAPE (9)        < · <= · > · >= · first · second · third · i64/to-f64 · i64/to-string
COMPLEX (5)            HashMap · HashSet · PersistentMap · PersistentVector · Vector
UNKNOWN-RULED-PENDING  and · or · do          (3 — the census has a ruling it cannot resolve)
```

★ `map · mapv · filter · foldl` are the **W7 HOF family**, parked behind the
`effectful_by_prefix` question (`NOTE-the-prefix-guess-does-not-scale-to-a-mixed-namespace.md`).
They are not merely unhomed; they are blocked.

### B — LIVE but dispatched off the literal-arm path (5)

```
+ · - · * · /        measured: (:wat::core::+ 1 2) -> 3. They RESOLVE.
reduce               measured: NoMatchingClauseAtCallSite — it EXISTS, as a defclause.
```

⚠ **No literal dispatch arm, and no census row.** They reach a handler some other way, and this
worklist does not yet know which. **That mechanism must be identified before any of the five is
homed** — a home built on a guess about how a verb is reached is the defect this campaign keeps
finding. `[[feedback_an_adjacent_implementation_is_not_the_subject]]`

### C — ⛔ DEAD. A hand-list row for a verb that does not exist (1)

```
:wat::core::when
```

Measured, verbatim:

```
unknown function: :wat::core::when
```

`intrinsic_meta` carries a purity ruling for a verb the language does not have. Nothing dispatches
it, nothing registers it, and calling it is an `UnknownFunction`. **This is a third kind of rot** —
not a stale ruling, not a shadowed one, but a ruling about *nothing*. It should be deleted with the
133 and named separately in that stone, because it is a different finding.

### D — not verbs at all (3)

```
:wat::edn::  ·  :wat::regex::  ·  :wat::string::
```

These are `head.starts_with(...)` **namespace rules** inside `intrinsic_meta` (`purity.rs:296`,
`:328`), not names. They will survive the deletion stone and are not homing work; they retire when
their namespaces are fully registered and the rule becomes unreachable — the same way the 133 did.

## The count, restated honestly

```
34  homing work      — of which 4 are BLOCKED (the W7 HOF family)
 5  mechanism unknown — must be identified before homing
 1  DEAD             — delete, do not home
 3  not verbs        — namespace rules, retire by coverage
--
43  … and the 44th is `:wat::core::=`, counted in A above.
```

## What retires this file

Every row in A and B. When they are homed, `intrinsic_meta`'s residue is the three namespace rules
in D and nothing else — at which point the question is whether the function should exist at all.

★ **That is the endpoint worth naming:** after T5, homing is the ONLY thing between here and
`intrinsic_meta` being empty. It is no longer a hand-list to maintain or a second opinion to
reconcile. **It is a countdown.**

## Rules this list obeys

- ⛔ **Replaced in place, never appended.** A worklist with strata is a worklist nobody trusts.
- ⛔ **No row is retired without its verb being registered** — verified by `lookup_entry`, not by
  the row looking done.
- ⛔ **Category B is not homing work until its mechanism is named.** Five verbs reached by an
  unidentified path are five chances to home the wrong thing.
