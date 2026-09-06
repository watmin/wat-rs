# DESIGN — partial only when it must

**Split the admission rule on one boundary.** `wat-scripts/queue/sqs.wat`, one `let`. Keeps every
correctness win of `6f77abd4c` and stops the fragmentation that cost 3.1×.

## WHY — the cost is OPERATIONS through one serializing actor

Five models for "why is publish slow" died today. The DESIGNs record them so nobody re-runs them:
the busy-wait (probed: moved nothing), per-call overhead (wrong model — dividing wall-clock by
call count attributes *blocked* time to waiting calls), closure allocation (~2 µs), unused match
arms (0 ns in a `defn`), and drain-bound (refuted: `dup` fell 35 % and `publish` rose).

The number that explains all of it, and that nobody had divided out:

```
queue-receive-calls   5396  →  11859     2.2x
pairs per receive       1.5 →      0.9
publish               22583 →   70244    3.1x
```

★★★ **Both sides fragmented.** The publisher tops the queue up a few slots at a time, so each
worker `receive` returns nearly empty and needs twice as many calls — and every send, receive and
ack crosses **one serializing queue actor**. Cost is per-operation; we roughly tripled the
operations while moving the same 8000 pairs.

## THE INSIGHT — partial admission is only NEEDED above `cap`

`sqs.wat:352-354` today:

```wat
room   (:wat::i64::- cap depth)
take0  (:wat::core::if (:wat::i64::< n0 room) n0 room)
take   (:wat::core::if (:wat::i64::< take0 0) 0 take0)
```

That takes a prefix **always**. But the defect partial admission was drawn to kill was the
**livelock**: a request larger than `cap` can never be admitted whole, no matter how empty the
queue gets. That is the *only* case where all-or-nothing is impossible.

Below `cap`, all-or-nothing costs nothing — the request will fit as soon as the queue drains — and
it preserves the one thing this system is bound by: **big transfers per operation.**

## ⛔ THE ONE CONTRACT DECISION

```
n0 >  cap   →  admit a prefix        (the livelock case)
n0 <= cap   →  all or nothing        (take = n0 if it fits, else 0)
```

- `nsubs 7`, 70 bodies against `cap 64` → still admits a prefix → **the cliff stays gone**
- `nsubs 4`, 40 bodies against `cap 64` → lands whole or waits → **big transfers return**

★ `Accepted [count]` is unchanged, `:Full` stays gone, no response leaks `depth` or `cap`, and
`Accepted 0` still means the 429. **This narrows when a prefix is taken; it changes no contract.**

## WHAT THIS PREDICTS, AND WHAT WOULD REFUTE IT

`queue-receive-calls` should fall back toward ~5400 and `dup` toward 0, because a whole-batch
admission cannot split a message's fanout.

⚠ **If `receive-calls` falls and `publish` does not follow, the operation-count model is wrong
too** — and that is worth more than this stone. It would be the sixth model down, and it would
mean the cost is somewhere none of us has looked.

## OUT OF SCOPE — REJECTED

- **The top-up in `sns-fanout.wat`** (`daf4a2f39`). It stays: it is correct on its own terms and
  still fires in the `n0 > cap` case. If `dup` reaches 0 without it, removing it is a later,
  measured decision — not an assumption made here.
- **`setup` / `stop`** — 20 s of the run, and the builder's order is publish first.
- **Caching `depth`** to skip the per-send `count-index` — 517 µs on every send is real, but
  `SCORE-stop-fetching-rows-to-get-a-number` deleted those counters deliberately and that stone
  must be read first.
- **Raising any `cap`.** The pressure system is the feature.
