# BRIEF — exhaustion is a named variant

Give the worker's three retry ladders **two closed enums** whose `Exhausted` variant every match
must name, and give the two that still stop after three flat tries the same vis-bounded backoff the
ack path already has. `wat-scripts/fanout/circuit.wat` only.

Re-draw of `exhaustion-cannot-be-discarded` (NOT STRUCK on STOP-1). One parametric enum is
**refuted**; two closed enums replace it. Everything else stands.

## Read in order

1. **`wat-scripts/scratch-pad/probe-a-local-fn-can-be-generic.wat`** — **read this first, and read
   its header.** It records the corrected measurements: a local generic `fn` freezes after its first
   use; a top-level generic `defn` instantiates per call site. It also records that its own earlier
   version was **wrong** (`:wat::core::T` where the parameter is `T`) — do not re-derive from the
   retracted claim.
2. **`SCORE-exhaustion-cannot-be-discarded.md`** — why one parametric enum fails: the child
   classifies it `MatchShape::Open` (`src/check.rs:6933`) and keyword variants become illegal.
3. **`circuit.wat:494-518`** — the **check** ladder. `once` returns `(peer, Option<reply>, retry?)`;
   `a1`/`a2`/`a3`; `:518` returns `(Tuple 1 0)` on exhaustion. **The terminal event at depth.**
4. **`circuit.wat:560-576`** — the **mark** ladder. `once-m` returns `(peer, retry?)`; `mm1`/`mm2`/
   `mm3`; **`second mm3` is discarded.** No counter.
5. **`circuit.wat:606-692`** — the **ack** path. `ack-limit-ms` at `:606`, inlined backoff at
   `:675-676`, `ar-tick` at `:691`. **This is the loop shape to copy.**
6. **`circuit.wat:698-718`** — `tick-pair` → `gb-tick` / `ar-tick` → `worker::Record`. The counter
   groove; new counters ride it.
7. **`circuit.wat:519`** — `(:fanout::Seen::CheckResponse::Ok hits)`. **Proof a non-parametric enum
   keyword-matches inside the EmptyEnv child** — it runs every tick.
8. **`circuit.wat:2153` / `:2188` / `:2201`** — the **failure-path** string, summary, and phases. All
   three carry the counters; `:2153` is what made the current diagnosis possible.
9. **`circuit.wat:2086`** — `:env-fn "(:wat::program::EmptyEnv)"`. Why the loops stay local.

## The work

**1. Two closed enums.** `:fanout::SeenRetry` (peer + `Seen::Reply`) serving **check and mark**;
`:fanout::QueueRetry` (peer + `Queue::Reply`) serving **ack**. Both `:wat::enum::Pure`, both
**non-parametric** — that is the whole point. Each carries `:Got [peer reply]` and
`:Exhausted [peer attempts]`.

**2. All three ladders return one of them.** Delete `a1/a2/a3`, `mm1/mm2/mm3`, and every surviving
`bool` retry flag. Each call site **matches** `Got` / `Exhausted` by keyword variant.

**3. Check and mark get the ack's loop.** On `DeadlineFired`, draw the inlined backoff and retry
until answered or `elapsed >= vis-ns / 1000000`. Same derivation, same comment. `Lost` / `Closed`
redial as they do today.

**4. Honest counters.** Each site counts its own exhaustion under a name that says what it measures.
**Rename `gave-back`.** All counters surface in the summary, the phases line, **and the failure-path
string** (`:2153`).

## Sketch

```wat
(:wat::core::defenum :fanout::SeenRetry :wat::enum::Pure
  :Got       [peer <- (:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])
              reply <- :fanout::Seen::Reply]
  :Exhausted [peer <- (:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])
              attempts <- :wat::core::i64])

;; local fn — ONE peer type each, which is why there are two enums
seen-until (:wat::core::fn
             [peer <- (:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])
              op <- :fanout::Seen::Op  attempt <- i64  seed <- i64
              start-ns <- i64  limit-ms <- i64]
             -> :fanout::SeenRetry
             (:wat::core::match (:wat::service::call-by-deadline peer op 200 inert)
               ((:wat::service::CallOutcome::Answered r) (:fanout::SeenRetry::Got peer r))
               ((:wat::service::CallOutcome::DeadlineFired)
                 (:wat::core::if (:wat::i64::>= (elapsed) limit-ms)
                   (:fanout::SeenRetry::Exhausted peer attempt)
                   (… draw backoff, await, recurse with attempt+1, seed' …)))
               (… Lost / Closed redial …)))
```

⚠ Variant constructors take **positional** arguments, not `:field val` pairs.

## Blast radius

`wat-scripts/fanout/circuit.wat` **only**. No `wat/`, no `sqs.wat`, no `sns-fanout.wat`. The receive
path (`:452`), the `:1531` call site, and the per-call 200 ms deadlines are untouched.

## STOP triggers

- **STOP-1** — if a **non-parametric** enum cannot be keyword-matched in the child, **STOP and quote
  the checker.** `:519` says it can; if that is somehow not general, the finding outranks the stone.
- **STOP-2** — if any drop-run assertion changes (`drop-after`, `drop-before`, `drop-recv-tiny`,
  `drop-ack-tiny`, the 38 `drop` floor tests), **STOP and report which.** The vis-derived bound is
  designed to preserve them; if it does not, the derivation is wrong and must be re-thought, **not
  patched with a flag.**
- **STOP-3** — if the no-args run differs in any **pre-existing** summary field, **STOP.** New
  counters are additive only.
- **STOP-4** — if `2000 4 3 8192 true` still fails, **STOP and report the terminal sweep with every
  counter.** That is the diagnosis; do not raise a bound to force a pass.
- **STOP-5** — floor red on any arm: **STOP; do not re-run.** Name the exact arm.
- **STOP-6** — **do not promote anything to `wat/`.** That is the builder's ruling and it is not on
  the table; this stone is how it gets earned.

## Shape to copy

`SCORE-the-ack-retries-like-the-publisher.md` — the ack loop it shipped is the exact loop check and
mark now need, and its GRADING carries the evidence that check is the terminal event.
