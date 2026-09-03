# BRIEF — the queue is bounded

`Queue/send` always accepts, so bounding the topic only moved the reservoir downstream. Give the
queue a depth cap that refuses, and have the adapter **block** until accepted — so backpressure
propagates worker → queue → adapter → topic → producer, and the chain behaves like one function call.

## Read in order

1. **`DESIGN-STONE-the-queue-is-bounded.md`** — the one contract decision (block, do not drop and do
   not buffer), and the known wart (the retry is a poll, with its replacement trigger).
2. **`FINDING-batching-needs-a-linear-store.md`** — the 2×2 this is responding to. **Do not re-derive
   it.** Batching is correct and pays 2.0× on a linear store; the latency cost is what this fixes.
3. **`wat-scripts/topic/sns-fanout.wat`** — `publish` returning `PublishResponse::Full` when the
   outbox is at `cap`. **This is the exact pattern to mirror**, already working, already proven by
   `probe_async_publish::full_outbox_refuses_not_drops`.
4. **`wat-scripts/fanout/circuit.wat`**, `:fanout::accept!` — the caller-side retry loop for a
   `Full`. Same shape for the adapter.
5. **`wat-scripts/queue/sqs.wat`**, `send` — where the cap check goes, before the store put.
   `pending` and `in-flight` are already in state; depth is their sum.

## The sketch

Load-bearing: the check is **before** the store put, and the adapter blocks rather than returning.
Illustrative: the cap value and the nap.

```wat
;; queue send, first thing:
depth (:wat::i64::+ (…/pending s) (…/in-flight s))
(if (:wat::i64::>= depth cap)
  (Outcome::Continue s (Some (Queue::Reply::Send (SendResponse::Full depth cap))) … )
  …existing path…)

;; adapter deliver: retry until accepted — this is the block that propagates
```

## Blast radius

`wat-scripts/queue/sqs.wat`, `wat-scripts/fanout/circuit.wat`, and any `SendResponse` match site.
One response variant, one durable field. **`wat/` and `src/` untouched.**

## STOP triggers

1. **If `total=8000; distinct=8000; dup=0` breaks — STOP.** A refusal that loses a message is the
   failure this whole design exists to avoid.
2. **If you find yourself buffering in the adapter — STOP.** That moves the reservoir one stage
   further up and repeats the mistake this stone is correcting.
3. **If it deadlocks — STOP, capture, name the sizes, do not re-run.** Every stage blocking on the
   next is exactly the shape that can deadlock, and `FINDING-the-drain-variance.md` is what a
   too-quick diagnosis costs.
4. **If `wat/` or `src/` need to change — STOP.**

## Floor

`./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a red — do not
re-run, name the arm, surface it. Check `ps` before any timing.

**Five runs, report the spread. Metric is deliveries/s and e2e max — not wall.**

Write `SCORE-the-queue-is-bounded.md` when done. It will be graded by re-running.
