# BRIEF — the waiter folds carry a named aggregate

Replace sqs.wat's three nested-Tuple waiter-fold accumulators with one `defstruct` carrying named
fields. `wat-scripts/queue/sqs.wat` only.

A **pure refactor**. Every number must come out identical — that is the gate.

## Read in order

1. **`circuit.wat:1428-1435`** — `:fanout::ShareStats` and its comment. **The pattern and its
   rationale, already written down**: *"Named fields so a defservice impl can read them.
   first/second/third on a Tuple inside `:impls` typechecks as an unsolved var."*
2. **`wat/cache.wat:262-272`** — `HolographicLru`, a **`defstruct`** holding live handles, with its
   own comment saying why a record cannot: *"both fields are live impure handles."* **This is the
   shape to copy** — the accumulator carries a `Peer`.
3. **`wat/query/mem.wat:123`** (`MemWrite`) and **`:643-650`** — a named record accumulator threaded
   through a `foldl`, rebuilt by name in every branch. The fold mechanics to copy.
4. **`wat/grep.wat:161`** (`ChildAcc`) and **`:230-234`** — the same in a recursive walk.
5. **`sqs.wat:445-453`, `:929-937`, `:1214`** — **the three sites.** Each is
   `(Tuple :- [Peer (Tuple :- [PersistentVector Vector i64])])` with the type spelled twice and
   hand-destructured into `st`/`inner`/`keep`/`box`/`taken`.
6. **`wat-scripts/scratch-pad/probe-a-fold-accumulator-can-be-a-struct.wat`** — the probe proving a
   `defstruct` may hold a `Peer` and thread through a step function.

## The work

**1. One `defstruct`** — `:queue::TakeAcc` or similar — with `store` (the `Peer`), `keep`, `box`,
`calls`. **`defstruct`, not `defrecord`**: a record may hold only EDN-expressible fields, and this
one carries a live `Peer` (`ImpureFieldInPureAggregate`, arc 293.W).

**2. All three folds use it.** The accumulator type in the `fn` signature and the `-> ` return type
become the one name. Reads become `(:queue::TakeAcc/keep acc)`; rebuilds name every field.

**3. Nothing else changes.** No new field, no instrument, no `none-*` hoisting, no logic edits.

## Blast radius

`wat-scripts/queue/sqs.wat` **only**. No `wat/`, no `circuit.wat`, no `sns-fanout.wat`, no probes.
`Queue::StatsResponse` is untouched, so **there is no ripple** — this is the one recent stone whose
blast radius really is one file, and it is one file because nothing crosses a service boundary.

## STOP triggers

- **STOP-1** — if a `defstruct` cannot be used as a `foldl` accumulator inside `:impls`, **STOP and
  quote the checker.** The probe says it can; a contradiction outranks the stone.
- **STOP-2** — if **any** measured number moves — `store-calls`/pair, `receive-calls`/pair,
  pairs/sec at n=500/1000/2000, `distinct`, `dup`, `seen-skipped` on a no-args run — **STOP and
  report which.** This is a refactor; a moved number means it was not one.
- **STOP-3** — floor red on any arm: **STOP; do not re-run.** Name the exact arm.
- **STOP-4** — **do not add `store-ns`.** It is the next stone and it would destroy this one's gate.
- **STOP-5** — **do not touch `circuit.wat`.** Its 69 nested-Tuple sites are a separate stone.
- **STOP-6** — if the tree does not parse at any point, **STOP and leave it parsing.** Restore rather
  than hand back a non-loading tree; the last strike left one and it blocked verification.

## Shape to copy

`wat/query/mem.wat:643-650` for the fold, `wat/cache.wat:262-272` for the struct-vs-record choice.
