# HANDOFF — accept, then fan out

Phase instrumentation located the circuit's latency exactly:

```
setup  ~10.4 s | publish 24.3 s (12.2 ms each) | drain 0.02 s | stop+collect 1.1 s
```

**The consumers were never the problem.** Drain is 21 ms — the workers keep up in real time. The
publisher is what waits, because `publish` is a `foldl` over subscribers calling `Sub/deliver`
synchronously, and each of those blocks on a queue write which blocks on a store write. **13
sequential blocking round-trips per message, and the publisher waits for all of them.**

Start here, in order:

1. `DESIGN-STONE-the-async-publish.md` — the two contract decisions.
2. `BRIEF-async-publish.md` — the rooms as exact `file:line`, four STOP triggers.
3. `wat-scripts/queue/sqs.wat` — the outbox and tick pattern to copy, not reinvent.

Three things to hold:

**★ The drain condition needs a third term.** With delivery asynchronous, a message can be *accepted
but not yet delivered* — resting in the topic's outbox, invisible to `pending` and `in-flight`. The
circuit would stop before it arrives. Row 3 proves the term is load-bearing by removing it and
requiring a failure, because row 2 alone would pass on a lucky run and fail on an unlucky one later.

**★ A full outbox refuses; it does not drop.** Async publish is what *creates* the unbounded-buffer
risk — the publisher can now outrun the fan-out. The span learned this by shipping the hole first
(item (c) stone D); here it is known in advance, so the bound lands with the change. And unlike a
dropped log, a dropped publish is data loss with the caller standing right there holding the message:
hand the refusal back.

**Use a non-zero timer delay.** A duration-0 `after` **never fires at process tier** — found in the
sane circuit, verified independently, still unfixed in the substrate. 1 µs works; 0 is silence.

`total=8000; distinct=8000; dup=0` is not negotiable — it is what proves the new asynchrony lost
nothing.

Report the wall time and the phase split; do not promise either.

The floor is `./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a
red — do not re-run, name the arm, surface it.

Write `SCORE-async-publish.md` when done. It will be graded by re-running.
