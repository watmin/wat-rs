# DESIGN — STONE F: a type parameter does not launder impurity into a pure aggregate

> ⛔ **CLOSED 2026-09-24 — SUPERSEDED BY `RULING-purity-is-parametric.md`.** This DESIGN's invariant
> ("a Pure aggregate never holds an impure value, at any instantiation") is the BOUNDED reading, and
> the builder ruled PARAMETRIC. Its executor's census found 166,330 deliberate stdlib hits. Read the
> ruling; what survives of this stone is the generic wire-wall question recorded there.

**Drawn 2026-09-24.** Found by stone E's executor; reproduced independently here.

## The defect, driven

```
(defrecord :t::Box [x <- (Lru :- [i64 i64])])      check=1  ImpureFieldInPureAggregate   ✓ refused
(defrecord :t::Box :- [T] [x <- T]) + (Box :- [Lru]) {:x h}   check=0 run=0              ⛔ accepted
the same, T INFERRED — (:t::Box {:x h})                       check=0 run=0              ⛔ accepted
(:wat::edn::write b)  →  #t/Box {:x {:x #rust.cache/Lru nil}}      (the writer is CORRECT — see below)
```

⛔ **The last row is why this matters — and the writer is NOT the defect.** An opaque has no EDN
representation by design: `src/edn/render.rs` renders a live handle as `opaque_nil`, *"only
genuinely-opaque LIVE values nil"* (builder, 2026-09-24: *"opaques are meant to produce nil - they
have no edn repr"*). The doctrine is arc 294's, `BRIEF-294.i-opaque-the-death-warrant.md:15`: *"A resource has
**no EDN representation**. The tag says what it was; the `nil` body says you learn nothing more.
That is correct and final — do not write encoders for anything."* The purity rule is what promises a `Pure` record contains NONE, so that
writing it is whole. The defect is that the promise is false here; `nil` is the writer correctly
declining to serialize something the record should never have held. Stone E's executor also measured a **generic `:Pure` enum** getting past the
check the same way — reproduce that case yourself first (the fixture shape is in stone E's probe).

## The root

`is_pure_type` (`src/check.rs:15035`) ends its `Path` arm in `None => true`, commented *"unknown
path ⇒ a formal type parameter ⇒ portable by convention."* So a declared `T` is PURE BY ASSUMPTION
at declaration, and **nothing re-checks the assumption when `T` is filled in** — annotated or
inferred. `validate_aggregate_containment` runs once, over declarations, where `T` is still `T`.

## The invariant — handed down as the goal, not a mechanism

⛔ **A `Pure` aggregate or `Pure` enum never holds an impure value — at any instantiation, written
or inferred.**

## The shape — two layers, the same shape stone E used

1. **Check time, where the type is known:** when a pure aggregate/enum is instantiated with concrete
   type arguments (an annotation `(Box :- [X])`, or a constructor call whose field types infer
   concretely), every argument bound to a parameter that reaches a field must satisfy
   `is_pure_type`. This catches both the annotated and the inferred rows above.
2. **Runtime, for what the checker cannot see:** a constructor inside a GENERIC function builds
   `Box<T>` with `T` still a variable. Refusing there at check time would outlaw a truth — a generic
   fn that builds `Box<T>` for a pure `T` is legal. So the pure constructor itself checks its field
   VALUES deeply for an impure value, and refuses as a **wat runtime error carrying the user's span**
   (constructors are evaluated in the runtime, not the dispatch macro, so a raise with a span is
   available there — as `i64::/` does it).

## The ONE contract decision

**Refuse, never coerce.** The EDN writer turning a handle into `nil` is the failure mode, not an
option. A pure aggregate that would hold an impure value is refused at the earliest layer that can
see it.

## Out of scope — REJECTED, but recorded

- **The EDN writer's `opaque_nil`.** Deliberate, not a defect (an opaque has no EDN repr). ⛔ Do
  not touch it. An earlier draft of this DESIGN called it a separate lossy path; that was wrong.
- Type-parameter BOUNDS as a language feature (`T : Pure`). A real design, and the builder's call —
  this stone enforces the invariant without adding syntax.
- Where raised errors report their location (the F-006 family) — separate stone.
