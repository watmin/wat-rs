# BRIEF — the workers stop polling

Make the circuit's workers **park** on their queue instead of polling it. Today each worker calls
`Queue/receive` with `:wait-ns 0` and re-arms a `-tick` every millisecond forever, producing
**144,485 receive calls to deliver 8,000 messages, 136,485 of them empty**. The queue's long-poll
capability already exists and is already committed; this is adoption, not construction.

## Read in order

1. **`DESIGN-STONE-the-workers-stop-polling.md`** — the design and the one contract decision
   (`wait-ns` is chosen against shutdown latency, and shutdown costs one park regardless of worker
   count). Read first; it rules on the tradeoff and lists what is deliberately out of scope.
2. **`wat-scripts/fanout/circuit.wat:112-160`** — `:fanout::worker`'s `-tick`. **This is the change.**
   `:118` is the arm; `:129` is the `:wait-ns 0` receive; `:111`/`:158` are the `Millisecond 1`
   re-arms. Note `:112-117` is a comment block asserting the false finding — it goes too.
3. **`wat-scripts/queue/sqs.wat:283-302`** — the queue's park path (`Waiter`, `conj` onto `waiters`,
   the `Nanosecond wait` alarm) and **`:420-445`** — the wake path (`ReplyTo` with the collected
   `Directed`s). This is the machinery you are switching on. **Read it; do not modify it.**
4. **`wat-scripts/fanout/circuit.wat:228-250`** — `:fanout::held-worker`, which already long-polls at
   `:wait-ns 50000000`. **The nearest working exemplar in the tree; copy its shape.**
5. **`wat-scripts/scratch-pad/probe-parked-waiters-stop.wat`** — the verification that `Admin::Stop`
   does not hang, with the two-park sweep. Your safety argument, already run.
6. **`docs/arc/2026/06/278-rules-engine/SCORE-queue-long-poll.md:47,57,92`** — why adoption was
   withdrawn, and why that reasoning was about the *old* fixture. Read it so you do not re-derive it.

## The sketch

Load-bearing: the park replaces the clock, and an empty return still re-arms (a park that expires is
"nothing yet", not "no work" — the producer may still be running). Illustrative: the constant.

```wat
rr (:queue::Queue/receive q
     (:queue::Queue::ReceiveRequest
       :queue name :now-ns now :visibility-ns vis
       :limit 10 :wait-ns 250000000))          ; was 0
…
;; got envelopes -> process, then immediately receive again (no timer at all)
;; empty return   -> re-arm, but the park WAS the wait; a 1 ms timer on top of a
;;                   250 ms park is the polling this stone deletes
```

The productive path should not carry a timer at all: a worker that just received work should go
straight back to receiving. Only the empty path needs an arm, and it needs it because the serve loop
must get a turn to take `Admin::Stop`.

## Blast radius

**`wat-scripts/fanout/circuit.wat` only** — `:fanout::worker`'s `-tick` and the comment above it.
No new types. `wat/` and `src/` untouched. Do not touch `:fanout::held-worker` (it is row 2's
delayed-ack fixture and proves a different property).

## STOP triggers

1. **If adopting long polling needs a change to `wat/` or `src/` — STOP and surface it.** The
   capability shipped a stone ago; if it does not work from the outside, that is a finding about the
   capability, not something to patch around in the circuit.
2. **If `total=8000; distinct=8000; dup=0` breaks — STOP.** Do not tune around it and do not adjust
   the drain condition to make it pass. A lost message here is the whole point of the invariant.
3. **If `Admin::Stop` hangs — STOP, capture whole, and name the exact worker count and `wait-ns`.**
   That would contradict a verification I ran this session, which makes it valuable, not
   embarrassing. Do not re-run to see if it goes away.
4. **If you need the ack batching or the drain poller to make a row pass — STOP.** Both are cut in
   the DESIGN with numbers. If a row cannot pass without them, the row or the design is wrong and
   saying so is worth more than shipping.

## Shape to copy

`SCORE-the-sane-circuit.md` for how a row is proven by **removing** the thing and requiring a
failure. `SCORE-async-publish.md` for how a wall-time regression is reported rather than promised.

## Floor

`./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a red — do not
re-run, name the exact arm, surface it. Check `ps` for a running `wat`/`cargo` before taking any
timing measurement; a contended box produced a false reading here earlier today.

Write `SCORE-the-workers-stop-polling.md` when done. It will be graded by re-running.
