# BRIEF — the poller sweeps once

`fanout::poll-until-drained*` asks the queue services for the same `Queue/stats` data three times
per iteration. Make it ask **once** per iteration and derive all four facts from that one sweep,
and have it **count and report its own round trips** so the reduction is measured rather than
argued. `wat-scripts/fanout/circuit.wat` only.

## Read in order

1. **`circuit.wat:973-989`** — `poll-until-drained*`, the loop. `snap` (`:978`) and `box` (`:979`)
   are bound at the top of every iteration; `snap` is used **only** in the two `format` arms
   (`:981`, `:986`). This is the function you are rewriting.
2. **`circuit.wat:896-904`** — `depth-of`. One `Queue/stats`, returns `(visible, unacked)`. This is
   the call all three sweeps make. ⚠ **It has a caller outside the poller at `:1528` — keep it.**
3. **`circuit.wat:906-909`** — `queue-drained?`: `depth-of`, then `visible==0 && unacked==0`.
4. **`circuit.wat:911-917`** — `all-drained?`: folds `queue-drained?`. Short-circuits once false.
5. **`circuit.wat:937-944`** — `any-unread?`: folds `depth-of`, tests `visible == -1`. Its
   accumulator stays `false` on a healthy queue, so it always pays the full `m` calls.
6. **`circuit.wat:946-949`** — `fully-drained?`: `all-drained?` AND `topic-outbox == 0`.
7. **`circuit.wat:960-968`** — `depth-snapshot`: folds `depth-of` into the `[v/u]` string.
8. **`circuit.wat:919-925`** — `topic-outbox`: one `Topic/stats`, returns `n`.
9. **`circuit.wat:1976`** — the single call site: `_drain (require! (poll-until-drained qclients topic 4000))`.
10. **`circuit.wat:2018-2035`** — the `phases` format. The new counter is reported here.
11. **`sqs.wat:73-75`** — `StatsResponse::Ok [receive-calls ticks visible unacked]`. One reply
    already carries both fields every sweep wants.

## The work

**One sweep per iteration.** Take the `m` `Queue/stats` results **once** into a
`Vector[(visible, unacked)]`, take `Topic/stats` **once** into `box`, then derive:

- `unread?`  — any `visible == -1` in the sweep, or `box == -1`
- `drained?` — every entry `(0, 0)` and `box == 0`
- the `[v/u]` snapshot string — formatted **inside the error arms only**, from that same sweep

**Count the round trips.** Thread an accumulator through the recursion and return it, so the gate
is a counted number. `poll-until-drained*` / `poll-until-drained` return
`(:wat::core::Tuple :- [:wat::core::String :wat::core::i64])`; the call site at `:1976` unpacks —
`first` goes to `require!`, `second` is reported in `phases` as a new `poll-calls=` field.

## Sketch

```wat
(:wat::core::defn :fanout::sweep-of
  [qclients <- (:wat::core::Vector :- [:queue::Queue])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])
  ;; exactly one depth-of per queue
  ...)

(:fanout::snapshot-str  sweep)  -> String   ;; pure, no calls
(:fanout::sweep-unread? sweep)  -> bool     ;; pure, no calls
(:fanout::sweep-drained? sweep) -> bool     ;; pure, no calls

(:wat::core::defn :fanout::poll-until-drained*
  [qclients ... t ... left ... start-ns ... total ... rts <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:wat::core::String :wat::core::i64])
  (:wat::core::let
    [sweep (:fanout::sweep-of qclients)          ;; m calls
     box   (:fanout::topic-outbox t)             ;; 1 call
     rts'  (:wat::i64::+ rts (:wat::i64::+ (:wat::core::count qclients) 1))]
    ...))
```

The three `sweep-*` helpers take the already-fetched vector and make **no service calls at all** —
that is what makes the reduction structural rather than incidental.

## Blast radius

`wat-scripts/fanout/circuit.wat` **only**. No `wat/` change, no `sqs.wat` change, no
`sns-fanout.wat` change. `depth-of` stays (it has the caller at `:1528`). New helpers are
`:fanout::`-namespaced. If `depth-snapshot` / `any-unread?` / `all-drained?` / `fully-drained?` end
up with no remaining callers, delete them in the same commit — a dead helper is the graveyard this
project gates against.

## STOP triggers

- **STOP-1** — if any of the four facts (`unread?`, `drained?`, snapshot, `box`) cannot be derived
  from `(visible, unacked)` per queue plus `box`, **STOP and name the fact.** Do not add a second
  sweep to cover it.
- **STOP-2** — if threading the counter through the recursion cannot be typed, **STOP and report
  the signature that will not check.** Do not introduce ambient or mutable state to carry it.
- **STOP-3** — if `depth-of`'s caller at `:1528` needs a shape change to accommodate this, **STOP.**
  That is a different stone.
- **STOP-4** — if the floor goes red on any arm, **STOP; do not re-run.** Capture it whole and name
  the exact arm. There is no known flake here.

## Shape to copy

`BRIEF-a-count-never-reads-more-than-it-needs.md` and its `SCORE-*` in this directory — same
relationship (a bounded-read prerequisite drawn ahead of a benchmark), same "neutral today, load-
bearing at depth" framing, same discipline of gating on the structural fact rather than the clock.
