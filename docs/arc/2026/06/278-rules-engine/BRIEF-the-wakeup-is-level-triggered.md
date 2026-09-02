# BRIEF — the wakeup is level-triggered

Both the queue and the topic arm a self-tick **on the empty→non-empty edge** and re-arm it from
scattered places. Any path that returns with the collection non-empty and no alarm loses the wakeup
forever. With polling that is invisible — the poller re-checks 141,297 times a run. With the park
adopted it deadlocks at `M=4, N≥1000` with one frozen item everywhere. Make arming a property of the
**state**: one helper, called by every arm, that arms iff the collection is non-empty and nothing is
already outstanding. Then adopt the park and prove it at weight.

## Read in order

1. **`DESIGN-STONE-the-wakeup-is-level-triggered.md`** — the design, the amplification argument, and
   the one contract decision (an explicit `tick-armed?` flag). Read first.
2. **`SCORE-the-workers-stop-polling.md`** — the previous strike. **The scale matrix and the stuck
   tail are your reproduction; do not re-derive them, and do not re-run the 2000 hang casually.**
3. **`wat-scripts/queue/sqs.wat:352,369`** — `receive`'s park path and the `was-empty?` edge. This
   is the bug's home in the queue.
4. **`wat-scripts/queue/sqs.wat`** — the other arms that arm or fail to: `send`'s waiter-serving
   tail (`keep` empty vs not), `-tick`'s tail, `ack`. **Every one of these is a return path that
   must go through the new helper.** Find them all; a missed one is the whole bug again.
5. **`wat-scripts/topic/sns-fanout.wat:131,140`** (`publish`'s `was-empty?`) and **`:192-195`**
   (`-deliver`'s `rest` empty vs not). Same shape, same repair.
6. **`wat-scripts/fanout/circuit.wat:112-132`** — the worker's `-tick` and the comment recording why
   `wait-ns` is 0. Both change: the park goes in, the comment goes out.
7. **`wat-scripts/scratch-pad/probe-three-waiters-wake.wat`** — grok's wake probe; small-N wake is
   already proven there. Extend it rather than starting over.

## The sketch

Load-bearing: **one helper, every return path, flag cleared at the top of the tick.** Illustrative:
names and the delay computation (keep the existing `delay0`).

```wat
;; ONE function; every arm's `arms` field comes from it and from nowhere else.
(:wat::core::defn :queue::arms-for
  [s <- :queue::queue::State  now-ns <- :wat::core::i64]
  -> (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])])
  (:wat::core::if
    (:wat::core::and (:wat::core::not (:wat::core::empty? (…/waiters s)))
                     (:wat::core::not (…/tick-armed? s)))
    [(:wat::service::Alarm :after (:wat::time::Nanosecond delay0) :op :-tick)]
    (:wat::core::Vector :- [(:wat::service::Alarm :- [:queue::queue::Op])])))
```

`-tick` sets `tick-armed?` false as its first act; the helper sets it true when it arms. The topic
gets the same treatment over `outbox` / `:-deliver`.

## Blast radius

`wat-scripts/queue/sqs.wat`, `wat-scripts/topic/sns-fanout.wat`, `wat-scripts/fanout/circuit.wat`.
Two new `:ephemeral` fields (the flags). **`wat/` and `src/` untouched.**

## STOP triggers

1. **If it still hangs at `1000×4×3` — STOP.** Snap the state (outbox, per-queue `p`/`f`, and the
   new flags), name the sizes, and surface it. **Do not re-run to see if it passes.** A hang with the
   flags visible is worth more than a green run without them.
2. **If bounding the ticks with a flag cannot be made to hold — STOP and say so.** That is the
   DESIGN's one contract decision failing, and it means the rung-3 substrate outcome is the real
   answer. Do not invent a third mechanism.
3. **If you need `wat/` or `src/` — STOP.** The substrate `ensure-alarm` outcome is explicitly a
   different stone.
4. **If a row can only pass by putting `wait-ns` back to 0 — STOP.** That re-hides the bug, which is
   how it survived this long.
5. **If `total=8000; distinct=8000; dup=0` breaks — STOP.** Never adjust the drain condition to make
   it pass.

## Shape to copy

`SCORE-the-sane-circuit.md` for proving a term load-bearing by **removing** it and requiring failure.
`SCORE-the-workers-stop-polling.md` for how a scale matrix with named drivers beats a re-run.

## Floor

`./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a red — do not
re-run, name the exact arm, surface it. Check `ps` for a running `wat`/`cargo` before any timing.

Write `SCORE-the-wakeup-is-level-triggered.md` when done. It will be graded by re-running.
