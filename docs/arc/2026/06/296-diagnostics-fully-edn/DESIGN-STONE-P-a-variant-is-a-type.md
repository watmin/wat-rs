# DESIGN — STONE P: a variant is a type (and the prerequisite the seam did not know about)

> ⛔ **THE SEAM'S OWN CLAIM ABOUT THIS STONE WAS WRONG, AND MEASUREMENT SAYS SO.** It read
> *"ONE ENTRY in `subtype_edges`"* — derived from the structure, never measured.
> `HAERESIS EST ITERVM ROGARE` landing on my own note, one day old.

## WHAT IS ACTUALLY TRUE — measured 2026-09-08 on the green tree

```
(1)  a variant is NOT a registered type
     :wat::runtime::type-of :usr::Shape::Circle  ->  "unknown type ':usr::Shape::Circle'"

(2)  but the ANNOTATION position ACCEPTS it — and accepts ANYTHING
     [s <- :usr::Shape::Circle]        check=0     a real variant
     [s <- :usr::Shape::Nonexistent]   check=0     ⛔ no such variant
     [s <- :usr::TotallyMadeUp]        check=0     ⛔ no such type

(3)  and comparison is BY STRING, against a type that need not exist
     :usr::f: parameter #1 expects :usr::TotallyMadeUp; got :usr::Shape

(4)  the ctor still ERASES — (Shape::Circle {:d 1}) has type :usr::Shape
```

★★★ **So `Shape::Circle` "works" in a type position today only because NOTHING VALIDATES THAT
POSITION.** It is not a type; it is an unchecked string that happens to compare unequal to
`:usr::Shape`. A stone that adds variant types on top of that would be building on a position that
cannot tell a real type from a typo — and every acceptance row would be satisfiable by a defect.
`[[feedback_an_acceptance_row_a_defect_can_satisfy_is_not_a_row]]`

## ⛔ THEREFORE THE STONE IS TWO, AND THE ORDER IS FORCED

**P-1 — THE ANNOTATION POSITION VALIDATES ITS TYPE NAME.** An annotation naming no known type is an
error, and the error says *unknown type*, not a mismatch against a phantom. This is a WALL: expect a
red floor naming every stale or typo'd annotation in the corpus, which is the worklist.

⚠ This defect exists **today, independent of variants**, and it is the same family as the `:wat::*`
call-head blanket (queue item 2) one position over: **a name that is never asked about.** It should
be measured for overlap with that stone before either is drawn as separate work.

**P-2 — A VARIANT IS A TYPE.** Only once P-1 makes the position honest:

```
register    :usr::Shape::Circle as a type, fields from the declaration (type-of must ANSWER it)
ctor        (Shape::Circle {:d 1})  :  :usr::Shape::Circle          — stop erasing
edge        :usr::Shape::Circle <: :usr::Shape                       — widening stays free
```

## THE EDGE — one entry in a general map, behind a wall built for another relation

`src/types.rs:542` `subtype_edges: HashMap<String, Vec<String>>` is **general** — a name to a LIST of
parents. `:304`: *"every parsed aggregate registers its subtype edge via `nature.root_keyword()`.
**Non-nature-root parents are rejected at parse time.**"*

★ **The question that decides this stone: is `Variant <: Enum` INHERITANCE?** Arc 293 annihilated
inheritance and KEPT subtyping — the right split. Inheritance is *"Circle inherits Shape's fields and
behaviour"*, a hierarchy. `Variant <: Enum` is **tagged-union membership**: Circle does not inherit
from Shape, it IS one of Shape's cases. **A sum type, not a hierarchy.** Two different relations
sharing an arrow, and a wall that cannot tell them apart refuses both.

## ⚠ THE MEASUREMENT THAT MUST COME BEFORE ANY DESIGN OF P-2

**Can `Shape::Circle` be a type WITHOUT becoming a second way to spell a record?** If a variant type
is fully general, someone returns `Shape::Circle` where they meant a record and the language has two
spellings for one-shape data — the exception-shaped thing this arc keeps deleting.

The likely honest answer, to be measured rather than assumed: **a NARROWING usable in parameter and
dispatch position, not a general-purpose type to build signatures around.** Decide it with the four
questions, on the measurement, before drawing P-2's brief.

## ★ THE PAYOFF, AND WHY THE BUILDER IS RIGHT THAT THIS IS DISPATCH

`defclause` is already the open-surface router — **72 live sites**, dispatching on type with a
fallback. Give a variant a type and *routing per variant falls out of the construct that already
exists*: no new dispatch mechanism, which is the entity-kind answer rather than a type-system reach
(FM 10). And `{:keys}` on a variant follows **for free** from Stone O — the destructure predicate is
already "is this an aggregate?", and a variant is a named-field aggregate the moment it is one.

**One mechanism, three symptoms, as the NOTE recorded:** `process-rect` unwritable · `defclause`
cannot route per variant · `{:keys}` has nothing to bind against. All three are constructor erasure.

## OUT OF SCOPE — AFFIRMATIVELY CUT

- **Flow typing / narrowing after a match arm.** A match arm establishing `s : Shape::Circle` inside
  its body is a separate capability. This stone gives the variant a TYPE; it does not teach the
  checker to narrow a binding.
- **`{:keys}` on a variant** — arrives free from O once P-2 lands; not separate work.
- **The `:wat::*` call-head blanket** — its own queued stone, though P-1 must be measured against it.

## THE FOUR QUESTIONS

- **P-1 Obvious?** YES — an annotation that names nothing is an error. **Simple?** YES — one
  resolution at one position. **Honest?** YES — it deletes a silent hole. **Good UX?** YES — a typo'd
  type says *unknown type* instead of a mismatch against a phantom.
- **P-2 Obvious?** YES — a variant is one of an enum's cases and can be named as such. **Simple?**
  YES *if* the answer to the narrowing question is "parameter and dispatch position only"; ⛔ **NO**
  if it is a general type, because then it duplicates `defrecord`. **THE MEASUREMENT DECIDES
  WHETHER THIS STONE IS SIMPLE, AND IT HAS NOT BEEN TAKEN.**
