# BRIEF — the accumulator is declared where the child can see it

Replace sqs.wat's three nested-Tuple waiter-fold accumulators with `(Tuple TakeAcc box)`, where
`TakeAcc` is a `defstruct` **declared inside `:queue::Queue`'s `:messages`**.
`wat-scripts/queue/sqs.wat` only. A **pure refactor** — every number must come out identical.

**This is your candidate 2 from the last SCORE**, with the half neither of us had tested now proven:
a `:messages` type may hold a live `Peer`.

## Read in order

1. **`docs/excursus/2026/08/001-sns-sqs/the-waiter-folds-carry-a-named-aggregate/SCORE.md`** — your STOP-1 and the GRADING under it.
   The visibility rule and both probes.
2. **`sqs.wat:86`** — `:queue::Waiter`, declared inside `:messages`. **`TakeAcc` goes beside it**,
   and this is why `keep <- PersistentVector<Waiter>` is legal there.
3. **`circuit.wat:236`** — `:fanout::Outcome`, a `defrecord` in `:messages`, used throughout a
   process-child `:impls`. The proof that a `:messages` type reaches the child.
4. **`sqs.wat:445-453`, `:929-937`, `:1214`** — the three sites. Each spells the accumulator type
   **twice** (parameter and `->` return) and hand-destructures into `st`/`inner`/`keep`/`box`/`taken`.
5. **`wat/query/mem.wat:643-650`** — a named accumulator threaded through a `foldl`, rebuilt **by
   name** in every branch. The fold mechanics to copy.
6. **`wat/cache.wat:262-272`** — `HolographicLru`, the `defstruct`-not-`defrecord` rationale for a
   handle-carrying aggregate.

## The work

**1. `:queue::TakeAcc`** — a `defstruct` in `:messages` with `store` (the `Peer`), `keep`, `calls`.

**2. `box` stays outside**: the accumulator becomes `(Tuple TakeAcc box)`. One level of nesting, not
two. Slot two keeps its `Vector<Directed<Queue::Reply>>` type inline, exactly as today.

**3. All three folds use it.** Signature and `->` return become the one shape. Reads become
`(:queue::TakeAcc/keep …)`; rebuilds name every field.

**4. Nothing else changes.** No new field, no instrument, no logic edits, no `none-*` hoisting.

## Blast radius

`wat-scripts/queue/sqs.wat` **only**. `Queue::StatsResponse` is untouched, so **no ripple** — the
15-site `StatsResponse::Ok` list is not in play. `TakeAcc` is a new `:messages` type, so confirm
nothing else must change to accept it.

## STOP triggers

- **STOP-1** — if a `:messages` `defstruct` is still not visible inside `:impls`, **STOP and quote
  the checker.** The probe and `circuit.wat:236` both say it is; a contradiction outranks the stone.
- **STOP-2** — if **any** measured number moves — `store-calls`/pair, `receive-calls`/pair,
  pairs/sec at n=500/1000/2000, `distinct`, `dup`, `seen-skipped` on no-args — **STOP and report
  which.** A refactor that moves a number is not one.
- **STOP-3** — floor red on any arm: **STOP; do not re-run.** Name the exact arm.
- **STOP-4** — **do not add `store-ns`.** Next stone; it would destroy this one's gate.
- **STOP-5** — **do not touch `circuit.wat`**, and **do not put `box` inside `TakeAcc`** — S4c,
  already measured.
- **STOP-6** — **leave the tree parsing.** You did this last time; keep doing it.
