# BRIEF — the ack retries like the publisher does

Replace the ack's three-flat-tries ladder with the publisher's proven retry shape — exponential
backoff, jitter, cap — bounded by the message's own visibility window. Count the retries.
`wat-scripts/fanout/circuit.wat` only.

## Read in order

1. **`circuit.wat:588-607`** — `once-a` and the `aa1`/`aa2`/`aa3` ladder. `once-a` returns
   `(peer, retry?)`; `retry?` is `true` only on `DeadlineFired`; **`second aa3` is discarded.**
   This is what you are replacing.
2. **`circuit.wat:595`** — the per-call `call-by-deadline peer ack-op 200 inert-ack`. **The 200 ms
   stays** — it is a fine liveness probe for one attempt. The ladder above it is the defect.
3. **`circuit.wat:1126-1152`** — **the shape to copy.** `publish-until-accepted!*`: draw
   `backoff-delay seed attempt`, `await-timer-ms d`, recurse with `attempt+1`, `retries+1`,
   `seed1`, `asleep+d`; bound on `elapsed >= limit-ms`. Note it threads the seed through so the
   jitter stream advances.
4. **`circuit.wat:1109-1118`** — `backoff-delay`: exponential `BASE << attempt`, capped at
   `BACKOFF-CAP-MS`, drawn uniform in `[1, ceiling]`. Returns `(seed', delay)`.
5. **`circuit.wat:1098-1101`** — `BACKOFF-BASE-MS 1`, `BACKOFF-CAP-MS 100`, `BACKOFF-SEED 1`.
6. **`circuit.wat:1868-1875`** — where `vis` is chosen: `200000000` for drop runs, `1000000000000`
   otherwise. **This is the source of the bound.**
7. **`circuit.wat:250, 277, 337, 361, 424, 612-631, 904`** — `gave-back`: the **counter precedent**.
   A `worker::Record` field, incremented from the per-iteration tick (`gb-tick`, the trailing `0` of
   the ladder's result triple at `:607`), carried through Stopped, summed, surfaced.
8. **`circuit.wat:2090` and `:2103`** — the summary and phases format strings, where the new counter
   surfaces.
9. **`circuit.wat:1531`** — an unrelated `call-by-deadline`; do not touch.

## The work

**1. The ack becomes a bounded backoff loop.** A recursive `:fanout::ack-until-acked!*` in the shape
of `publish-until-accepted!*`: on `DeadlineFired`, draw a `backoff-delay`, `await-timer-ms` it,
redial, and retry — until the ack is answered or `elapsed >= limit-ms`.

**2. The bound is the visibility window.** `limit-ms = vis-ns / 1000000`, threaded from the worker's
`vis-ns` (already on `worker::Record` — `:337`, `:361` show the field being carried). Put the
derivation in a comment: after vis expiry the message is visible again and retrying is pointless.

**3. Count retries.** `ack-retries` on `worker::Record`, threaded exactly as `gave-back` is, and
surfaced in **both** the summary (`:2090`) and phases (`:2103`).

**4. `Lost` / `Closed` keep redialling** as they do today. Only the `DeadlineFired` path changes
from "up to three" to "backed off until the vis bound".

## Sketch

```wat
(:wat::core::defn :fanout::ack-until-acked!*
  [peer <- … ack-op <- … attempt <- i64 retries <- i64 seed <- i64
   start-ns <- i64 limit-ms <- i64 redial <- […]]
  -> (:wat::core::Tuple :- [… :wat::core::i64])       ;; (peer, retries)
  (:wat::core::match (:wat::service::call-by-deadline peer ack-op 200 inert-ack)
    ((:wat::service::CallOutcome::Answered _r) (:wat::core::Tuple peer retries))
    ((:wat::service::CallOutcome::Lost _c)     (… (redial) …))
    ((:wat::service::CallOutcome::Closed)      (… (redial) …))
    ((:wat::service::CallOutcome::DeadlineFired)
      (:wat::core::if (:wat::i64::>= (:fanout::elapsed-ms start-ns) limit-ms)
        (:wat::core::Tuple peer retries)          ;; vis expiry redelivers; stop
        (:wat::core::let
          [drawn (:fanout::backoff-delay seed attempt)
           seed1 (:wat::core::first drawn)
           d     (:wat::core::second drawn)
           _     (:fanout::await-timer-ms d)]
          (:fanout::ack-until-acked!* (redial) ack-op
            (:wat::i64::+ attempt 1) (:wat::i64::+ retries 1) seed1
            start-ns limit-ms redial))))))
```

## Blast radius

`wat-scripts/fanout/circuit.wat` **only**. No `wat/`, no `sqs.wat`, no `sns-fanout.wat`. The
receive path (`:452`) and the `:1531` call site are untouched. The per-call 200 ms deadline stays.

## STOP triggers

- **STOP-1** — if any **drop-run** assertion changes (`drop-after`, `drop-before`, `drop-recv-tiny`,
  `drop-ack-tiny`, and their floor tests), **STOP and report which.** Those encode intended chaos
  behaviour; the vis-derived bound is designed to preserve them, and if it does not, the derivation
  is wrong and must be re-thought, **not patched with a flag.**
- **STOP-2** — if `vis-ns` is not reachable where the ack runs, **STOP and say so.** Do not
  substitute a constant for the bound; the whole contract decision is that the bound is derived.
- **STOP-3** — if the no-args run differs in any summary field, **STOP.**
- **STOP-4** — if `2000 4 3 8192 true` still fails, **STOP and report the terminal sweep and the
  `ack-retries` count.** That number is the diagnosis; do not raise a bound to force a pass.
- **STOP-5** — floor red on any arm: **STOP; do not re-run.** Name the exact arm.

## Shape to copy

`DESIGN/BRIEF/EXPECTATIONS/SCORE-fill-deep-then-drain.md` in this directory — it is where the red
this stone fixes was found, and its GRADING section carries the mechanism in full.
