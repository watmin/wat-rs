# BRIEF — the message carries its trace

Make each message carry timestamps through the circuit, and report per-stage latency as a histogram
at the end. The circuit currently reports aggregates only, and cannot say which component is slow —
which is why a 2× drain variance has no identified cause after two dead hypotheses.

## Read in order

1. **`DESIGN-STONE-the-message-carries-its-trace.md`** — the stamps, the one contract decision
   (in-band, not via the telemetry service), and why the queue counters are deferred rather than cut.
2. **`FINDING-the-drain-variance.md`** — what is established and what is already ruled out.
   **Do not re-derive it, and do not re-test the park duration.**
3. **`wat-scripts/fanout/circuit.wat`** — `:fanout::key-of` at `:441` (the invariant keys on the
   queue-generated envelope **uuid**, not the body, so stamping the body cannot disturb `distinct`),
   `:fanout::summarize`, and `collect-stop` which already returns all 8000 outcomes.
4. **`wat-scripts/topic/sns-fanout.wat`**, `-deliver` — where `t1` is stamped, at dequeue.
5. **`wat-scripts/queue/sqs.wat`**, `send` — where `t3` is stamped, on entry. **This is the only
   line that changes in this file.**

## The sketch

Load-bearing: the stamp points, `seq` first, and a histogram rather than a mean. Illustrative: the
separator and the bucket edges.

```
publisher   body = "<i>"                          then |t0
topic       -deliver, at dequeue                  body = body + "|" + t1
adapter     deliver, on entry                     body = body + "|" + t2
queue       send, on entry                        body = body + "|" + t3
worker      -tick, at receive: parse, t4 = now, bucket each interval
```

Report one line per stage:

```
outbox   <1ms=… 1-10=… 10-50=… 50-250=… 250-1000=… >1000=… max=…ms
t1->t2   …
t2->t3   …
t3->t4   …          <-- pending residency, the one to watch
e2e      …
```

## Blast radius

`wat-scripts/fanout/circuit.wat`, `wat-scripts/topic/sns-fanout.wat`, and **one line** in
`wat-scripts/queue/sqs.wat`. No surface changes — the body is already a `String` on both
`Sub::DeliverRequest` and `Queue::SendRequest`. `wat/` and `src/` untouched.

## STOP triggers

1. **If `total=8000; distinct=8000; dup=0` breaks — STOP.** The invariant keys on the envelope
   uuid; if stamping the body moves it, something depends on the body that should not.
2. **If the instrument changes the drain distribution — STOP and report it.** The body grows from
   ~4 bytes to ~120. Payload was measured free relative to hop cost, but that was a microbenchmark.
   **An instrument that perturbs its subject is not an instrument** — see row 2.
3. **If you need `Queue::StatsResponse` to change — STOP.** The counters are deferred with a trigger;
   adding them here is 11 match sites in 4 files and it is not this stone.
4. **If you need `wat/` or `src/` — STOP and surface it.**

## Shape to copy

`SCORE-the-tick-drains-a-batch.md` for reporting a distribution rather than a sample — and note that
its own headline number was a single sample that did not survive re-running.

## Floor

`./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a red — do not
re-run, name the arm, surface it. Check `ps` before any timing.

**Run the circuit at least five times** and report the spread, not one number. That is now a
standing requirement for any timing row in this arc.

Write `SCORE-the-message-carries-its-trace.md` when done. It will be graded by re-running.
