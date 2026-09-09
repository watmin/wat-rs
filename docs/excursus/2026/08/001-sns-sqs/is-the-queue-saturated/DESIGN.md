# DESIGN — is the queue saturated?

**The queue accumulates the time it spends inside op handlers; the drain's delta is reported per
queue as `drain-busy-ms`.** `sqs.wat` + `circuit.wat` + the ripple files.

The instrument that decides whether the slope is **induced queueing** or something else entirely.

## ⛔ WHY — a correction to the previous stone's reading

`the drain reports its own store time` measured, mine and grok's agreeing:

```
                        n=500 → n=2000
drain / pair               +55%
  ├─ store / pair          +40%     share of drain FALLING  52% → 47%
  └─ NON-store / pair      +72%
```

I read that as *"the store is largely exonerated."* **That was too strong**, and the reason is
structural rather than numerical:

★★★ **The queue is a serializing actor.** It is blocked inside store calls for **47–52 % of the
drain**. During that time it can serve nobody — so every worker waiting to `receive` or `ack` is
queued behind it, and **that waiting is counted in the "non-store" 72 % while its cause is the
store.** A saturating serializer converts its own service time into everyone else's delay.

So store-vs-non-store was the wrong axis. The question is:

> **How much of the drain is the queue actually busy?**

| | |
|---|---|
| `busy / wall` **near 1** | the queue is saturated; the non-store growth is **induced queueing**, and the store is the root cause after all, reached through the serializer |
| `busy / wall` **well below 1** | the queue is idle much of the drain; the time is genuinely elsewhere — IPC, scheduling, worker-side waiting — a direction nobody has looked at |

★ We already have a **lower bound**: the queue is busy at least `drain-store-ms / drain` = 47–52 %,
because it is blocked in store calls that long. Its own compute sits **on top** of that. The
instrument closes the gap between that floor and 1.

## ⛔ AN ELIMINATION FIRST — the poller is NOT the slope

Before drawing this, one hypothesis was tested and killed. The depth check is `O(min(depth, cap+1))`,
verified through the chain `circuit.wat:2116` (`:cap sub-cap` = 8192) → `sqs.wat:389/962`
(`lim = cap + 1` = 8193) → `count-index :limit 8193` → `SELECT COUNT(*) … LIMIT 8193`. The shipped
`cap 32` makes that O(32); **the benchmark's `sub-cap 8192` makes it O(depth)** — and `stats` calls
it twice, on every queue, every poll iteration.

So: raise the poll interval 5 ms → 50 ms and measure. Nothing else changed.

```
  n     poll-calls        drain ms         drain-store-ms
        5ms   50ms      5ms    50ms       5ms    50ms
  500    85     35      532     492       279     265
  1000  230     85     1245    1236       638     597
  2000  650    215     3308    3049      1567    1407
```

★★★ **Poller iterations cut ~2.5–3×; the drain moved 0–8 %; the slope is unchanged at +55 %.** The
O(depth) count is real and is **not** the slope. Eighth mechanism killed in this arc.

The cleaner decomposition it left:

```
store / pair                +33 %
NON-store / pair            +81 %     ← the superlinear term
ms per store round trip     +26 %
store calls / pair           +5 %     ← flat
```

## ⛔ THE MODEL THIS STONE TESTS — stated as a model, not a finding

Same round trips, same payloads, worker compute flat at `<1 ms` — yet non-store time per message
grows **+81 %** while the store's *service time* grows only **+26 %**.

★★★ **The queue is a single server**, and in any single-server system waiting rises far faster than
utilisation: `W ∝ ρ/(1−ρ)`. A 26 % rise in service time raises ρ by ~26 %; from ρ ≈ 0.5 that alone
produces roughly the +81 % observed.

⚠ **That is arithmetic that fits, not evidence.** Eight mechanisms have been killed in this arc that
also fitted. What makes this one worth a stone is that it is **directly measurable**: `ρ` *is*
`busy / wall`, which is exactly what `handler-ns` reports.

★ And it names the shape of "stay linear" if it holds: linearity is not about making one operation
faster, it is about holding utilisation off the knee — **fewer ops through the one server, cheaper
ops, or more servers.** Only the last changes the exponent.

## ⛔ THE ONE CONTRACT DECISION

```
:ephemeral        … store-calls  store-ns  handler-ns …
StatsResponse::Ok [receive-calls ticks visible unacked store-calls store-ns handler-ns]
phases            drain-busy-ms=   (delta at the drain boundaries) / m
```

`handler-ns` accumulates `(epoch-nanos (now)) − (Invocation/start-ns ctx)` at each point an arm
returns. `Invocation/start-ns` is already used in this file at `:745`, `:959`, `:998`.

⚠ **This is the widest mechanical edit of the arc**: **30** `queue::State` constructions and **18**
`Outcome::Continue` sites in `sqs.wat`, plus the **16**-site `StatsResponse::Ok` ripple across eight
files. It is uniform and every miss is a compile error — but it is not small, and saying so is part
of the design.

★ The named-field work is what makes it survivable. Before `TakeAcc`, an edit of this shape needed
paren surgery at every site and killed two consecutive strikes.

## WHAT IT CAN AND CANNOT SEE

⚠ `handler-ns` is **queue-side service time**, including its store calls. So:

```
busy            = handler-ns                      (store + queue compute)
queue compute   = handler-ns − store-ns           (derived, both already per-queue)
not-in-queue    = drain wall − handler-ns         (IPC, scheduling, worker-side)
```

★★ **That is the three-way split the arc has been circling**, and the second term is the one nobody
has measured. All three come from counters that already exist plus this one.

⚠ **`stats` is itself an op**, so the poller's handler time lands in `handler-ns` — the same
observation cost already quantified at 18–29 % of drain store calls. Derivable from `poll-calls`;
reported, not netted out.

## WHAT THIS IS AND IS NOT

⚠ **It does not explain the slope.** It decides which of two searches to run next.

⚠ **No prediction is offered.** The previous three stones each named a winner inside their own fork
and two were wrong — including this one's predecessor, above, in my own grading.

## OUT OF SCOPE — REJECTED

- **Making `stats` cheaper.** Its 2 store calls derive `visible`/`unacked`; the arc ruled against
  caching depth (`docs/excursus/2026/08/001-sns-sqs/stop-fetching-rows-to-get-a-number/SCORE.md`).
- **Timing inside the store service** (stdlib). Only if the answer points there.
- **Worker-side round-trip timing.** A second, larger instrument; this one's `not-in-queue` term
  bounds it first.
- **`circuit.wat`'s 36 nested-Tuple chains**, `collect`, the `Lost`/`Closed` conflation, the
  `seen-skipped` drift. All open, none on this path.
