# DESIGN — the workers stop polling

Drawn 2026-09-02. **Not struck.**

## Why

The circuit makes **144,485 queue receive calls to deliver 8,000 messages** — eighteen per message,
and **136,485 of them return nothing** (measured, `probe-circuit-sqlite.wat` prints
`queue-receive-calls`). Every worker polls its queue every millisecond, forever, whether or not
there is work:

```wat
;; wat-scripts/fanout/circuit.wat:118
rr (:queue::Queue/receive q (… :limit 10 :wait-ns 0))              ; non-blocking
…
[(:wat::service::Alarm :after (:wat::time::Millisecond 1) :op :-tick)]   ; re-poll in 1 ms
```

This is the circuit's dominant cost, and it is *self-inflicted twice over*:

**1. The hop is the unit of cost and the payload is free.** Measured (`probe-batch-vs-hops.wat`),
same 1000 items across the boundary:

```
THREAD    1000x1 = 154ms    100x10 = 17ms    10x100 = 1ms    1x1000 = 0ms
PROCESS   1000x1 = 191ms    100x10 = 19ms    10x100 = 2ms    1x1000 = 0ms
```

~100% fixed cost per hop, near-identical at both loci. So an empty poll costs the same as a useful
one, and 136,485 of them is 21 s of pure waste at 154 µs — before contention.

**2. Worse, the polls throttle themselves.** 12 workers on a 1 ms timer should produce ~840,000
polls in 70 s; we measured 144,485. The workers cannot poll faster because their own polls are
queued behind each other in the same serialized serve loop. That is congestion collapse, and it is
why the circuit feels uniformly slow rather than bottlenecked anywhere nameable.

**3. And the reason it polls is a finding that is false.** `circuit.wat:114` says:

> `;; wait-ns is 0: a parked receive (wait-ns>0) at process locus with ≥4`
> `;; waiters never completes, so Admin::Stop hangs waiting on the tick.`

Verified false 2026-09-02 (`probe-parked-waiters-stop.wat`, and the correction is recorded in
`SCORE-the-sane-circuit.md`). J = 1,2,3,4,5,8 parkers, all at **process** locus, every one provably
inside a 5-second `Queue/receive` when `Admin::Stop` fired. **All stop cleanly.**

## What it delivers

Workers that **park** instead of poll: woken by the queue when a message arrives, not by a clock.

## The one contract decision

**`wait-ns` is chosen against shutdown latency, and shutdown costs ONE park regardless of worker
count.** Measured, one variable:

```
park = 5 s     j=1: 5990 ms   j=4: 6991 ms   j=8: 8336 ms
park = 50 ms   j=1: 1275 ms   j=4: 2399 ms   j=8: 3580 ms
```

The j=1 delta is 4715 ms and the j=8 delta is 4756 ms — **both one park**, not J parks. Growth with
J is ~350 ms per worker, which is process spawn. So `Stop` waits for the in-flight receive to
return (correct: a `defservice` is a serializing actor, the arm must finish before the serve loop
can take `Admin::Stop`), the cost is bounded by `wait-ns`, and it does **not** scale with waiters.

That makes a generous park cheap. **250 ms** is the starting value: three orders of magnitude fewer
wakeups than a 1 ms timer, against a quarter-second shutdown tail on a run of tens of seconds. It is
a starting value, not a tuned one — the executor reports what it measures.

## Why the earlier withdrawal does not apply

`SCORE-queue-long-poll.md` withdrew circuit adoption as *"a measured loss"* — and was right **for
that fixture**. It says why (line 47): with the old circuit, `wait-ns 0` returning empty meant
*"not filled yet"*, not *"no work"*, because that circuit **published everything before any consumer
started**. It closes: *"the capstone should not adopt long polling until a benchmark exists whose
shape it can actually help."*

**That benchmark now exists.** The sane-circuit stone made workers start first and the producer run
alongside them, so an empty queue genuinely means "nothing yet, wait" — exactly the shape long
polling is for. The withdrawal was correct and is now spent.

## Out of scope = REJECTED, with the numbers that cut them

- **Batching the acks.** The worker acks one hop per envelope inside a `foldl` while `:limit 10`
  already hands it up to ten. Real, but it saves ~7,200 hops of ~248,000 — **~3%** — and it needs a
  surface change (`AckRequest` takes one id). Not worth carrying alongside a 136,485-hop fix.
- **The drain poller** (`wait-drained`, `nap-ms 5`, five stats hops per iteration, ~70,000 hops).
  Genuinely second-largest — but its hop count is a function of how long drain takes, and this stone
  is about to change that. Designing against a number that is about to move is how the last three
  lanes went wrong. **Re-measure after this lands, then decide.**
- **Any change to `wat/` or `src/`.** The long-poll capability shipped a stone ago; this is adoption.
- **Tuning `wait-ns` to a benchmark optimum.** Report what 250 ms does. A tuned constant with no
  stated tradeoff is a guess wearing a number.
