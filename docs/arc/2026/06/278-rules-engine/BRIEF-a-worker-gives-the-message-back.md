# BRIEF — a worker gives the message back

Restore a drop knob on `Seen/check`, then make retry-exhaustion release the envelope instead of
killing the worker. `wat-scripts/fanout/circuit.wat` only.

Read `DESIGN-a-worker-gives-the-message-back.md` first — in particular *why the 0/6 is not a fix*.

## ⛔ ORDER MATTERS: PROVOKE BEFORE YOU REPAIR

**EXPECTATIONS row 1 requires you to make the crash happen again, and record it, BEFORE changing
the worker.** Restore the `check` knob, run the tiny cell, and capture the assertion. A fix
whose defect was never re-observed is indistinguishable from a defect that is merely
unprovoked — which is the exact confusion this stone exists to end.

## READ IN ORDER

| room | why you are there |
|---|---|
| `circuit.wat:78-84` | the seen `defservice`'s `:durable` — `drop-rate-bp` / `drop-seed` / `drop-after?`. Split the rate into **two independent knobs**, one per verb; the seed may stay shared |
| `circuit.wat:113-148` | the `mark` impl and its `hit?` — the drop pattern to copy verbatim for `check` |
| `circuit.wat:86-112` | the `check` impl — pure today; it gains the same `hit?` → `:wat::core::None` reply |
| `circuit.wat:440-458` | the `once` retry loop, `a1`/`a2`/`a3`, and **`:455-458`, the assertion to remove** |
| `circuit.wat:460-530` | the fold over `envs`. Exhaustion must return the accumulator **unchanged for that envelope** — no emit, no `mark`, **no ack** — and continue |
| `circuit.wat:1740-1746` | the fixtures that construct seen `Record`s; every one needs the new field |

## SKETCH

At the exhaustion point, instead of `assertion-failed!`:

```wat
;; The budget is spent. Nothing was emitted and no receipt was written, so this
;; envelope is untouched work: leave it unacked, let visibility redeliver it.
(:wat::core::Tuple q0 seen1 outs0)     ;; outs0, not outs1 — nothing emitted
```

## BLAST RADIUS

`circuit.wat` only. No `wat/`, no `sqs.wat`, no `src/`, no `.rs`, no nextest config.

## STOP TRIGGERS

- **STOP-1** — if the crash cannot be provoked with the `check` knob on, STOP and report. Either
  the knob is not wired or the exhaustion arm is reached another way; both change the stone.
- **STOP-2** — if give-back requires acking the envelope to keep the fold's types happy, STOP.
  **An ack destroys the message.** The whole contract is that it is left unacked.
- **STOP-3** — if a run stops converging (the drain reports `drained-never` at the new rate),
  STOP and report the rate and the depth. That is a real finding about lossy `check`, not
  something to tune away by lowering the rate.
- **STOP-4** — do not touch the retry budget of 3, the queue-side calls, or anything perf-shaped.

## PRIOR RESULT TO COPY FOR SHAPE

`SCORE-a-ledger-is-a-receipt.md` — the stone that made give-back safe, and the reason exhaustion
can now be handled by doing nothing.
