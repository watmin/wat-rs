# DESIGN — the poller sweeps once

**`poll-until-drained*` issues three identical `Queue/stats` sweeps per iteration; it needs one.**
`wat-scripts/fanout/circuit.wat` only. No substrate change.

## WHY — the drain instrument is a load generator against the thing it measures

`fanout::poll-until-drained*` (`circuit.wat:973-989`) is the loop that decides when the drain is
over. Every iteration, before its 5 ms wait, it calls into the queue services **three times over**
— and all three are the *same* request, `Queue/stats`, differing only in which field of the reply
they read:

| what runs | defined at | calls | reads | used for |
|---|---|---|---|---|
| `depth-snapshot` → `depth-of` | `:960`, `:896` | m × `Queue/stats` | `visible`, `unacked` | **an error string** |
| `any-unread?` → `depth-of` | `:937`, `:896` | m × `Queue/stats` | `visible == -1` | the liveness check |
| `fully-drained?` → `all-drained?` → `queue-drained?` → `depth-of` | `:946`, `:911`, `:906`, `:896` | ≥1 × `Queue/stats` | `visible==0 && unacked==0` | the termination check |
| `topic-outbox` | `:919` | 1 × `Topic/stats` | `n` | both |

Per iteration during an active drain that is **2m + 2** round trips minimum — **10 at m=4** — every
one of them a request into a **serializing** queue actor that a worker is simultaneously trying to
`receive` from. `any-unread?` never short-circuits (its accumulator stays `false`), so it always
pays the full m.

★★★ **`snap` is the worst of the three.** It is computed unconditionally at the top of every
iteration and is used **only** in the two error `format`s (`:981`, `:986`). On a clean drain it is
fetched and discarded on every single pass.

★ And all three want data the *same one call* already carries. `Queue::StatsResponse::Ok` is
`[receive-calls ticks visible unacked]` (`sqs.wat:73-75`) — `visible` and `unacked` together answer
snapshot, unread, and drained. The loop asks three times because three helpers each grew their own
sweep, not because three answers were needed.

## ⛔ THE DEFECT CLASS — an instrument that samples its target harder than the target is worked

Rule 10 of this arc reads *"we bounded what a read RETURNS and never what it EXAMINES."* This is
its sibling one level out: **we never asked how much traffic the measurement itself puts through
the thing being measured.** The count stone stopped the *service* reading more than it needed; this
one stops the *harness* asking more than it needs.

⚠ Scale, from the numbers already banked in the tracker: the uncapped run's drain is 22.8 s. At
~7 ms per iteration that is ~3,000 iterations × 10 = **~30,000 `Queue/stats` round trips injected
into the queue services during the exact phase whose rate we want to plot** — against roughly
64,000 worker round trips of real work. The instrument is the same order of magnitude as the load.

## ⛔ THE ONE CONTRACT DECISION — one sweep, derived four ways

`poll-until-drained*` takes **one** sweep per iteration and derives every fact from it:

```
sweep : Vector[(visible, unacked)]   ← m × Queue/stats
box   : i64                          ← 1 × Topic/stats
```

- `unread?`   = any `visible == -1` in `sweep`, **or** `box == -1`
- `drained?`  = every `(0, 0)` in `sweep` **and** `box == 0`
- `snapshot`  = formatted from `sweep` **only inside the error arms**

**m + 1 round trips**, down from 2m + 2. At m=4: 5 instead of 10.

★ This is not a weaker check. It is the same four facts from a **consistent** sample — today's three
sweeps read the queue at three different instants, so the snapshot printed in an error can disagree
with the check that fired it.

## WHAT THIS IS AND IS NOT WORTH — stated before measuring

⚠ **At the shipped `cap 64/32` this should be neutral.** Drain is 190 ms ≈ 38 iterations; halving
10 round trips on 38 passes is invisible, and the total is dominated by publish anyway.

★★ **Its value is at depth, and it is a prerequisite** — exactly as
`a count never reads more than it needs` was. The next stone plots **drain rate against queue
depth**. A drain phase that carries ~30,000 of its own round trips measures the poller as much as
the system, and the deeper the fill the longer the poller runs.

⚠ **So this stone is NOT judged on the circuit's wall clock.** It is judged on the round-trip count
per iteration falling, and on behaviour being byte-identical. A drain-time improvement would be a
welcome *observation*; a drain-time regression would be a finding; neither is the gate.

## OUT OF SCOPE — REJECTED

- **The depth benchmark itself.** Parameterising `:cap` (hardcoded `32` at `circuit.wat:1817`,
  `64` at `:1827`) and deferring the worker go-signal (`_go` at `:1949`, `_twgo` at `:1888`) so the
  fill completes before the drain begins. That is the stone this one unblocks, and it is next.
- **Replacing the poll with a wire event.** `poll-until-drained` carries an explicit ruling at
  `:969-972` — *"Conjunction across N queues plus the topic inbox. No single wire event. Bounded,
  and it reports what it last saw — the check rung, taken only where the shape rung is
  unavailable."* Changing the rung is a larger, separate ruling. This stone keeps the check rung
  and makes it cheap.
- **Counting `stats` calls inside the queue service.** `receive-calls` counts receives only
  (`sqs.wat:74`). A harness-local counter proves this stone without touching `sqs.wat`.
- **Tuning the 5 ms poll interval.** A different variable; changing it at the same time would make
  the round-trip row unreadable.
