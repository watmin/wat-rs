# BRIEF — the client backs off intelligently

Replace the publisher's fixed `await-timer-ms 1` with exponential backoff, full jitter and a cap.
`wat-scripts/fanout/circuit.wat` only. Read `DESIGN-the-client-backs-off-intelligently.md` first —
it carries the sweep and the reason a fixed value is wrong even at its optimum.

## READ IN ORDER

| room | why |
|---|---|
| `circuit.wat:1016` | `publish-until-accepted!*` — the retry loop. It already threads `attempts`; it must now thread a **seed** too |
| `circuit.wat:866-873` | `await-timer-ms` — the timer, unchanged. Only its argument becomes computed |
| `src/intrinsic/rand.rs:10` | `(:wat::rand::int-from state lo hi) -> (Tuple i64 i64)` — **threaded**: returns `(state', value)`. Pure ∧ deterministic |
| `wat-scripts/scratch-pad/probe-chaos-is-a-rate.wat:136` | a worked `int-from` call site — copy its shape for threading the state |
| `circuit.wat` `queue::queue::Record` construction | where a `drop-seed` already rides as a Record field — the same shape a backoff seed can use |

## SKETCH

```wat
;; ceiling doubles per attempt, bounded by CAP; the draw is uniform in [1, ceiling].
;; 1 not 0: :wat::time::NonZeroDuration makes a zero wait unrepresentable, and a
;; zero wait is a spin, which measured WORSE than any wait.
(:wat::core::defn :fanout::backoff-delay
  [seed <- :wat::core::i64  attempt <- :wat::core::i64]
  -> (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])
  (:wat::core::let
    [shifted (:wat::i64::* :fanout::BACKOFF-BASE-MS (:fanout::pow2 attempt))
     ceiling (:wat::core::if (:wat::i64::> shifted :fanout::BACKOFF-CAP-MS)
               :fanout::BACKOFF-CAP-MS shifted)]
    (:wat::rand::int-from seed 1 ceiling)))
```

In `publish-until-accepted!*`:

- carry `seed` alongside `attempts`
- on `Accepted 0` → `(seed', d) = backoff-delay seed attempt`; `await-timer-ms d`; recurse with
  `seed'` and `attempt + 1`
- on **any** acceptance (`c > 0`, whole or partial) → recurse with `attempt` reset to **0**

`BACKOFF-BASE-MS = 1`, `BACKOFF-CAP-MS = 100`, both as named `def`s so they are readable and so a
future reader can see they are bounds rather than a tuned point.

## BLAST RADIUS

`wat-scripts/fanout/circuit.wat` and scratch probes. **No `sqs.wat`, no `sns-fanout.wat`, no
`wat/`, no `src/`.** This is a client change; the server learns nothing.

## STOP TRIGGERS

- **STOP-1** — the seed cannot be threaded through the retry recursion without restructuring it.
  Report the shape; do **not** reach for an unthreaded random, which would break reproducibility.
- **STOP-2** — `int-from` cannot produce a bounded draw with `lo = 1`. Report it; the `1` is
  required by `NonZeroDuration`, not a preference.
- **STOP-3** — adaptive backoff does **not** beat the fixed-25 ms `publish` of **18472 ms**.
  **Report it plainly as a tie or a loss.** Do not tune `BASE`/`CAP` to win — that re-creates the
  magic value with extra steps, and the DESIGN rejects it.
- **STOP-4** — `distinct` is not 8000 on any run. A backoff change cannot affect delivery; if it
  does, something else is wrong and that is the finding.
- **STOP-5** — anything outside the blast radius.

## THE MEASUREMENT

`circuit.wat` ×5 reporting `publish`, `full-retries`, `queue-receive-calls`, `distinct`.
Three baselines, all measured this session on this box:

```
fixed  1ms   publish 22395   retries 3807   receives 5290    ← today
fixed 25ms   publish 18472   retries  540   receives 4687    ← the swept optimum
spin  (none) publish 22794   retries 4088   receives 5313    ← the floor of doing nothing
```

## GRADE AGAINST

`SCORE-partial-only-when-it-must.md` — same file family, same measurement shape.

Write `SCORE-the-client-backs-off-intelligently.md`, then `pulsare_yield kind=scored`.
