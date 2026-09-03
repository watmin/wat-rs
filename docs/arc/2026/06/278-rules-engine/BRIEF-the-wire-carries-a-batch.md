# BRIEF — the wire carries a batch

`Sub::DeliverRequest` and `Queue::SendRequest` each carry one message. Make them carry many, so one
chain traversal moves K messages instead of one. The circuit is at 661 deliveries/s and within ~20%
of its per-message chain latency; this is the only remaining way past that.

## Read in order

1. **`DESIGN-STONE-the-wire-carries-a-batch.md`** — the decomposed 3–5× estimate, the `cap`/`K`
   sweep, and the one contract decision: **no linger timer**. Read it first; the exclusion is the
   design, not a preference.
2. **`FINDING-the-buffer-was-the-bottleneck.md`** — why a linger timer is forbidden. We spent a day
   discovering that a deep buffer costs ~100× latency for no throughput. Do not rebuild it inside
   the batcher.
3. **`wat-scripts/topic/sns-fanout.wat`**, `-deliver` — it already drains K=10 per tick. Today that
   is **K rounds of (4 sends + 4 recvs)**; it becomes **one round of (4 sends + 4 recvs)** carrying
   K. The concurrent send-all/recv-all shape is unchanged — only what each send carries.
4. **`wat-scripts/fanout/circuit.wat`**, `:fanout::adapter` — receives `msgs`, stamps `t2` on each,
   forwards as `bodies`.
5. **`wat-scripts/queue/sqs.wat`**, `send` — build N `StoredRow`s and issue **one**
   `Store::PutRequest` with all of them; `pending` increases by N; the waiter-serving foldl runs
   **once**, not N times.
6. **`tests/services/probe_async_publish.rs::fanout_is_max_not_sum`** — the shape for wiring a
   constructed proof into the floor. Row 1 needs the same treatment.

## The sketch

Load-bearing: one `Store::PutRequest` for the whole batch, and no timer anywhere. Illustrative:
names.

```wat
;; topic -deliver: build the batch ONCE, fan out ONCE
msgs (…the k heads, a Vector of at most k…)
_sent (foldl over subs: send p (Sub::Op::Deliver (DeliverRequest :msgs msgs)))
_recv (foldl over subs: recv p)

;; queue send: N rows, ONE put
rows (foldl over bodies -> conj acc (StoredRow …))
put-resp (Store/put store (Store::PutRequest rows))
;; pending + N, waiters served once
```

## Blast radius

`wat-scripts/topic/sns-fanout.wat`, `wat-scripts/fanout/circuit.wat`,
`wat-scripts/queue/sqs.wat`, and `tests/services/probe_async_publish.rs` for row 1. Two message
records change shape. **`wat/` and `src/` untouched.**

## STOP triggers

1. **If `total=8000; distinct=8000; dup=0` breaks — STOP.** Two surfaces are moving; this invariant
   is the only thing standing between a batch and silent loss. Do not adjust the drain condition.
2. **If you find yourself adding a timer to fill a batch — STOP.** The DESIGN forbids it and
   `FINDING-the-buffer-was-the-bottleneck.md` is why.
3. **If `wat/` or `src/` need to change — STOP and surface it.**
4. **If the tail batch (outbox holds fewer than K) needs special-casing beyond `min(K, length)` —
   STOP and say what.** That is where a batched pipeline loses its last messages.
5. **If throughput does not improve — STOP and report it.** That would mean the chain was not the
   bound, which contradicts a measurement, and is worth more than the stone.

## Floor

`./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a red — do not
re-run, name the arm, surface it. Check `ps` before any timing.

**Five runs minimum, report the spread.** Report **deliveries/s**, not wall time — `setup`/`stop`
are process lifecycle and are not this stone's metric.

Write `SCORE-the-wire-carries-a-batch.md` when done. It will be graded by re-running.
