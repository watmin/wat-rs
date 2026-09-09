# DESIGN — the accumulator is declared where the child can see it

**sqs.wat's three waiter-fold accumulators become `(Tuple TakeAcc box)`, where `TakeAcc` is a
`defstruct` declared inside `:queue::Queue`'s `:messages`.** `wat-scripts/queue/sqs.wat` only.

A **pure refactor**. "Nothing moved" is the pass condition. Second draft; the first put the struct
at script level, where a process child cannot see it.

## ⛔ THE VISIBILITY RULE, VERIFIED IN BOTH DIRECTIONS

> **A type reaches an EmptyEnv process child iff it is stdlib or declared in the surface's
> `:messages` — and a `:messages` declaration may itself name only `:messages` types and stdlib.**

Evidence, all this session:

| | |
|---|---|
| `circuit.wat:236` | `:fanout::Outcome` is a `defrecord` **inside `:messages`**, and is used throughout the worker's process-child `:impls`. The live proof of the first half. |
| grok's checker | a **script-level** `defstruct` gives `12 unresolved references` — `:queue::TakeAcc/store` *"call head — not a builtin, not a registered function"* |
| my probe | a `defstruct` **in `:messages`** holding a live `:wat::kernel::Peer` → **`"ok"`** |
| my probe | the same struct **plus a field naming the generated `Queue::Reply`** → **S4c**, verbatim |

★★★ **S4c was never about the `Peer`.** It fires only because `box`'s type,
`Vector<Directed<Queue::Reply>>`, names `Queue::Reply` — a type **generated** by `defsurface`, not
declared in `:messages`. I had derived the opposite and probed it rather than shipping it.

## ⛔ THE ONE CONTRACT DECISION

```wat
;; inside :queue::Queue's :messages, beside :queue::Waiter (sqs.wat:86)
(:wat::core::defstruct :queue::TakeAcc
  [store <- (:wat::kernel::Peer :- [:wat::query::Store::Op :wat::query::Store::Reply])
   keep  <- (:wat::core::PersistentVector :- [:queue::Waiter])
   calls <- :wat::core::i64])
```

**`defstruct`, not `defrecord`** — it holds a live `Peer` (`ImpureFieldInPureAggregate`, arc 293.W).
**`box` stays outside**, as slot two of `(Tuple TakeAcc box)`.

Today's shape, at `sqs.wat:445`, `:929`, `:1214` — the type spelled **twice** per site, then
hand-destructured:

```wat
(Tuple :- [Peer (Tuple :- [PersistentVector<Waiter> Vector<Directed<Reply>> i64])])
…
[st (first acc)  inner (second acc)  keep (first inner)  box (second inner)  taken (third inner)]
```

★ **Two levels become one.** `wat/lint.wat:428` marks that as the readable floor —
*"acc = Tuple(template, kwarg-names)"*; two is where a Tuple is still readable.

★★★★★ **And it delivers the objective, which was never the fourth slot.** A **fifth costs nothing**:
`store-ns` becomes a named field on `TakeAcc` — no nesting, no paren depth change, no re-balancing
by trial. The edit that killed two strikes becomes ordinary.

## WHAT THIS IS AND IS NOT

⚠ **No behaviour change, and that is the gate.** `store-calls`/pair, `receive-calls`/pair, the
pairs/sec curve at three depths, `distinct`/`dup`, 38 drop tests, floor 5221 — **every one
identical.** A pure refactor is the rare stone where "nothing moved" is the pass condition.

⚠ **No instrument rides along.** `store-ns` would destroy that gate. It lands next, as one field.

⚠ **`circuit.wat` is untouched** — 36 nested-Tuple access chains and 33 nested constructions there,
against sqs.wat's 5. Its own stone.

## OUT OF SCOPE — REJECTED

- **A script-level `defstruct`.** Invisible in the child; refuted by grok's checker.
- **`box` inside the aggregate.** S4c; refuted by my probe.
- **Threading `State` through the fold.** It *is* visible and it already names all four values
  (`store`, `waiters`, `outbox`, `store-calls` — `sqs.wat:122-128`), but rebuilding 16 fields per
  waiter is **worse** than the nesting it replaces, and those rebuilds are where sqs.wat's
  1253-char lines come from.
- **Putting the aggregate in `wat/`.** `HolographicLru`'s locus, and unnecessary now that
  `:messages` is proven to work. Stdlib promotion remains **the builder's ruling**.
- **A lint rule for nested Tuples.** Arc 277 owns the linter and is STRIKE-READY. The builder's call.
