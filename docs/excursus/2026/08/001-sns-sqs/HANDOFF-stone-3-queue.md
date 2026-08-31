# HANDOFF → grok — excursus 001 stone 3: SQS in userland

Same branch, `sns-sqs`. Read in full:

- `docs/excursus/2026/08/001-sns-sqs/BRIEF-stone-3-queue.md`
- `docs/excursus/2026/08/001-sns-sqs/EXPECTATIONS-stone-3-queue.md`

**This is the thing the excursus was opened to build.** SNS shipped at stone 1; every stone
since was substrate debt that drawing *this* one uncovered — `receive` needs a re-put (which
found mem/sqlite diverging), `ack` needs a delete (which the Store lacked). All landed. Floor
is **5121, FULLY GREEN.**

⛔ **ONE DECISION IS NOT YOURS AND NOT MINE:** stdlib or demo. Build it as
`wat-scripts/queue/`, matching SNS. **If you conclude it belongs in `wat/queue.wat`, say so
and STOP — do not put it there.** Promoting an experiment into the stdlib is the builder's
call, the same way opening an arc is.

**The design, and every primitive is proven:**
`pk` = queue · `sk` = a STABLE message id · GSI `by-visible-at` (`ipk` = queue, `isk` = when it
becomes visible). `send` puts; `receive` scan-indexes `isk <= now` then **re-puts** each row
with `isk = now + timeout`; `ack` deletes by `(pk, sk)`.

★ **The visibility timeout is a re-put that moves the index key into the future.** No lock, no
timer, no side state. `IndexRow` carries `pk sk ipk isk data` (`wat/query.wat:46`) — everything
needed to re-put, with no base-table read.

★ **ROW 5 IS THE ROW THAT CAN BE FAKED.** A fixture testing send/receive/ack passes without ever
proving **redelivery** — and a queue that never redelivers silently loses every message a
consumer failed to ack. The window must actually elapse. If there is no deterministic way to
step past it, **say so** — that is a finding about testability, not a reason to drop the row.
Do not reach for a sleep: *"sleep is a guess; guesses race."*

⚠ **The floor is fully green for the first time in this excursus. There is no known-red to
point at.** Any failure is yours: capture whole, do NOT re-run, name the exact assertion.
