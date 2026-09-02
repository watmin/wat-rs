# DESIGN — the wakeup is level-triggered

Drawn 2026-09-02. **Not struck.**

## Why

The circuit deadlocks at `M=4, N≥1000` **when consumers park instead of poll**. The stuck state,
snapped every 500 ms (`SCORE-the-workers-stop-polling.md`):

```
drain n=1100  out=1   q0=p1f1; q1=p1f1; q2=p1f1; q3=p1f1;
drain n=2000  out=1   q0=p1f1; q1=p1f1; q2=p1f1; q3=p1f1;
```

**Nothing is blocked.** Every actor answers `stats`. One topic outbox message, one pending and one
in-flight per queue, frozen for thousands of drain iterations. That is not mutual exclusion — it is
**unscheduled work**. A wakeup was lost.

### The busy-poll is a liveness crutch, and it is what hid this

A worker re-asking every millisecond never needs a wakeup: it re-examines the queue's state
**141,297 times per run** (measured), so any wakeup the queue fails to deliver is invisible — the
next poll finds the message anyway. Park the consumer and the crutch is gone, so a lost wakeup
becomes permanent.

**Therefore the defect is not in long polling.** Long polling is correct and measured: with the park
adopted, `500×4×3` is **2012 calls for 2000 outcomes** — one call per message, the consumer
registered and pushed to. It is the shape queue consumption should have. It merely stopped hiding a
bug that is in the tree *today*.

### The bug: a self-tick armed on an edge

Both services keep the same invariant — *"a self-tick is outstanding whenever this collection is
non-empty"* — and both maintain it by hand, arming only on the **empty→non-empty edge**:

```
wat-scripts/topic/sns-fanout.wat:131,140   was-empty? (empty? box)      -> arm :-deliver
wat-scripts/queue/sqs.wat:352,369          was-empty? (empty? waiters)  -> arm :-tick
```

with re-arming scattered across `send`, `-tick`, `receive`, `ack`, `publish` and `-deliver`. Nothing
checks it. **Any path that returns with the collection non-empty and no alarm loses the wakeup
permanently** — and the observed stuck state is precisely *one item, no tick*, in both services at
once.

★ **The failure modes are wildly asymmetric.** A redundant tick is a no-op: it fires, finds nothing
ready, and does not compound. A missing tick is a permanent deadlock. Edge-triggering buys nothing
and risks everything.

## What it delivers

Arming becomes a **property of the state**, not of the transition that produced it: *if the
collection is non-empty when the arm returns, an alarm exists.* One rule, one place, every arm.

## The shape

Every arm ends by calling **one helper** that derives its `arms` from the state it is about to
return. The composed `Outcome` landed today is what makes this possible — before it, an arm that had
to both reply and arm had no form.

```wat
;; illustrative, not prescriptive — the helper's NAME and home are the executor's
(:wat::core::defn :queue::arms-for [s <- :queue::queue::State  now-ns <- :wat::core::i64]
  -> (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])])
  …if waiters non-empty and no tick outstanding -> [(Alarm :after (Nanosecond delay0) :op :-tick)]
    else -> [])
```

### The one contract decision: bound the ticks with an explicit flag

Naive level-triggering **amplifies**. If every arm arms whenever `waiters` is non-empty, and
`receive`/`send`/`ack` run thousands of times while waiters stay non-empty, outstanding alarms grow
without bound. So the state carries an explicit `tick-armed?` (queue) / `deliver-armed?` (topic),
**cleared at the top of the tick and set when the helper arms.** The rule becomes:

> arm **iff** the collection is non-empty **and** no alarm is already outstanding.

This is still one invariant in one function rather than six call sites — but it is *bookkeeping*,
and if the flag is ever wrongly `true` the deadlock returns. That is why row 4 asserts the invariant
directly rather than trusting it.

★ **The rung-3 version is a substrate change and is NOT this stone:** an outcome that says *ensure an
alarm exists for op X* rather than *add an alarm*, so "armed twice" and "armed zero times" both
become unrepresentable and no flag is needed. Named here so it is not re-derived; it wants `wat/`
and its own stone.

## Why the fix and the adoption must ship together

With polling, this bug is **invisible** — no test can fail before or after, because the poller is the
wakeup. The only way to observe the repair is to remove the crutch. **The fix's acceptance criterion
IS the park surviving at scale.** Shipping them apart would mean shipping an unverifiable change.

## Out of scope = REJECTED

- **`wat/` and `src/`.** The substrate `ensure-alarm` outcome above is the rung-3 repair and is a
  different stone.
- **Tuning `wait-ns`.** 250 ms as before; report, do not optimise.
- **Ack batching and the drain poller.** Still cut, still for the numbers in
  `DESIGN-STONE-the-workers-stop-polling.md`. Re-measure after this lands.
- **Promoting `wat-scripts/{topic,queue}` to `wat/`.** The builder's ruling, unchanged.
