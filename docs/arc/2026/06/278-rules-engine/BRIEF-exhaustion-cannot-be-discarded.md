# BRIEF — exhaustion cannot be discarded

Give the worker's three retry ladders one **outcome type** whose `Exhausted` variant must be
matched, and give the two that still stop after three flat tries the same vis-bounded backoff the
ack path already has. `wat-scripts/fanout/circuit.wat` only.

## Read in order

1. **`wat-scripts/scratch-pad/probe-a-local-fn-can-be-generic.wat`** — **read this first.** It
   records why one shared combinator is impossible (generic local `fn` never instantiates its type
   parameter) and demonstrates the shape that works: one enum, `Exhausted` as a variant the match
   must name. **Copy its shape.**
2. **`circuit.wat:494-518`** — the **check** ladder. `once` returns `(peer, Option<reply>, retry?)`;
   `a1`/`a2`/`a3`; `:518` returns `(Tuple 1 0)` on exhaustion → `gb-tick = 1`. This is the terminal
   event at depth.
3. **`circuit.wat:554-576`** — the **mark** ladder. `once-m` returns `(peer, retry?)`; `mm1`/`mm2`/
   `mm3`; **`second mm3` is discarded entirely.** No counter.
4. **`circuit.wat:591-680`** — the **ack** path as the previous stone left it. `ack-limit-ms` at
   `:606`, the inlined backoff at `:675-676`. **This is the loop shape to copy** for check and mark.
5. **`circuit.wat:697-720`** — `tick-pair` → `gb-tick` / `ar-tick` → `worker::Record`. The counter
   groove; new counters ride it.
6. **`circuit.wat:2086`** — `:env-fn "(:wat::program::EmptyEnv)"`. Why a top-level `defn` is not an
   option and the loops must stay local.
7. **`wat/spawn.wat:195`** and **`wat/cache.wat:172`** — parametric `defenum` precedent.
8. **`circuit.wat:2185` / `:2198` / `:2145`** — summary, phases, and the **failure path** string.
   All three carry the counters.

## The work

**1. The outcome type.**

```wat
(:wat::core::defenum :fanout::RetryOutcome :- [P R] :wat::enum::Pure
  :Got       [peer <- P  reply <- R]
  :Exhausted [peer <- P  attempts <- :wat::core::i64])
```

Script-level, so it travels into the child as a type.

**2. All three ladders return it.** Delete `a1/a2/a3`, `mm1/mm2/mm3`, and any surviving `bool` retry
flag. Each call site **matches** `Got` / `Exhausted`.

**3. Check and mark get the ack's loop.** On `DeadlineFired`, draw the inlined backoff and retry
until answered or `elapsed >= vis-ns / 1000000`. Same derivation, same comment. `Lost` / `Closed`
redial as they do today.

**4. Honest counters.** Each site counts its own exhaustion under a name that says what happened.
**Rename `gave-back`** — it measures check-ladder exhaustion and asserts the opposite; it is
confined to this file and asserted in no test. All counters surface in the summary, the phases line,
**and the failure-path string** (`:2145`) — that last one is what made the current diagnosis
possible.

## Sketch

```wat
;; one per peer type; both in the tick arm's scope
check-until (:wat::core::fn
              [peer <- (:wat::kernel::Peer :- [:fanout::Seen::Op :fanout::Seen::Reply])
               op <- :fanout::Seen::Op  attempt <- i64  seed <- i64
               start-ns <- i64  limit-ms <- i64]
              -> (:fanout::RetryOutcome :- [(:wat::kernel::Peer :- [...]) :fanout::Seen::Reply])
              (:wat::core::match (:wat::service::call-by-deadline peer op 200 inert)
                ((:wat::service::CallOutcome::Answered r) (:fanout::RetryOutcome::Got peer r))
                ((:wat::service::CallOutcome::DeadlineFired)
                  (:wat::core::if (:wat::i64::>= (elapsed) limit-ms)
                    (:fanout::RetryOutcome::Exhausted peer attempt)
                    (… draw backoff, await, recurse with attempt+1, seed' …)))
                (… Lost / Closed redial …)))
```

## Blast radius

`wat-scripts/fanout/circuit.wat` **only**. No `wat/`, no `sqs.wat`, no `sns-fanout.wat`. The receive
path (`:452`), the `:1531` call site, and the per-call 200 ms deadlines are untouched.

## STOP triggers

- **STOP-1** — if `RetryOutcome :- [P R]` cannot be constructed or matched at both peer types,
  **STOP and report the checker's exact message.** Do not fall back to two separate enums without
  saying so; the probe says the *type* is the shareable part and that claim is under test.
- **STOP-2** — if any drop-run assertion changes (`drop-after`, `drop-before`, `drop-recv-tiny`,
  `drop-ack-tiny`, the 38 `drop` floor tests), **STOP and report which.** The vis-derived bound is
  designed to preserve them; if it does not, the derivation is wrong and must be re-thought, **not
  patched with a flag.**
- **STOP-3** — if the no-args run differs in any pre-existing summary field, **STOP.** New counters
  are additive only.
- **STOP-4** — if `2000 4 3 8192 true` still fails, **STOP and report the terminal sweep with every
  counter.** That is the diagnosis; do not raise a bound to force a pass.
- **STOP-5** — floor red on any arm: **STOP; do not re-run.** Name the exact arm.

## Shape to copy

`DESIGN/BRIEF/EXPECTATIONS/SCORE-the-ack-retries-like-the-publisher.md` — the ack loop it shipped is
the exact loop check and mark now need, and its GRADING section carries the evidence that the check
ladder is the terminal event.
