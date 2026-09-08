# NOTE — the variant DOT spelling is accepted under `:wat::*` and SILENTLY BUILDS THE WRONG VARIANT

**Found 2026-09-08**, answering the builder's question *"when do we flip to the dot notation for
variants?"* The answer is: **not until the `:wat::*` blanket is gone**, because today the dot
spelling is a silent wrong-answer generator.

## Measured

```
(:wat::core::Option::Some {:value 7})   check=0   ->  "7"                       correct
(:wat::core::Option.Some  {:value 7})   check=0   ->  #wat.core/Option.None {}  ⛔
(wat.core/Option.Some     {:value 7})   check=0   ->  (same; -1 through a typed Option<i64> slot)
(:wat::core::Result.Ok    {:value 7})   check=0   ->  #wat.core/Option.None {}  ⛔
(:usr::Box.Full           {:payload 7}) check=1   ->  UnresolvedReference       correctly REFUSED
```

In a typed `Option<i64>` slot the dot form falls through the `Some` arm to the `None` arm and the
function returns `-1` instead of `7`. **No check error. No runtime error. A wrong number.**

## The mechanism — the reserved-prefix blanket

A USER-namespace dot spelling is caught by `resolve` (`:usr::Box.Full` -> `UnresolvedReference`). A
`:wat::core::` one is not, because `is_resolvable_call_head` short-circuits on
`is_reserved_prefix(head)` (`src/resolve/walk.rs`), and nothing downstream refuses it either.

★ This is arc 255's founding defect — *"a nonexistent `:wat::` verb type-checks clean"* — with a
consequence sharper than the one it was filed under. It is not only that a typo ships. **A
PLAUSIBLE, INTENDED-LOOKING FUTURE SPELLING is accepted today and produces the wrong value.**

## Why this matters more than the blanket's original framing

255's NOTE recorded that the registry *"has exactly ONE use so far and that use was mine"* — it
lacked a consumer that would make the work verifiable. **This is that consumer, and it is a
correctness defect rather than a hygiene one.**

And it changes the queue: the blanket was ordered after `variant <: enum` as tidy-up. It is now a
prerequisite for the **dot-notation flip**, which is itself part of the head migration
(`:wat::core::Option::Some` -> `wat.core/Option.Some`) — the crusade's main move.

## What this does NOT claim

- Not that the dot spelling is *wrong as a design*. `#wat.core/Option.Some` is ALREADY the wire
  form (296 H); the dot is the ruled variant separator. The defect is that SOURCE accepts it under
  `:wat::*` without meaning it.
- Not that the mechanism past resolve is understood. Why it yields `Option.None` specifically is
  **unmeasured** — read the expansion before theorising. `[[feedback_an_adjacent_implementation_is_not_the_subject]]`
- Not that user namespaces are safe by design; they are safe by `resolve` still checking them, which
  is the same door the blanket opens for `:wat::*`.

## The order this implies

```
Q    one authority that can answer `is this a type?`        IN FLIGHT
P-1  the annotation position validates                      needs Q
:wat::* blanket  ⛔ PROMOTED — it is a CORRECTNESS defect, not hygiene, and it gates the dot flip
P-2  a variant is a type                                    4/4, unblocked
dot flip / head migration                                   BLOCKED on the blanket
```
