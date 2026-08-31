# `wat-scripts/queue/` — **wat-queue**, maturing toward a wat feature

A **queue**: send / receive / ack, with a visibility timeout and redelivery. The SQS half of
excursus 001.

Built here on the `wat-grep` / `wat-gen` precedent. **Promoted to `wat/queue.wat` when it
demonstrates excellence — the builder's ruling, never a side effect.** The grep precedent sets
the standard for the move: *"the counts are the proof it moved intact."*

## The design — every primitive already ships

```
pk  = the queue name
sk  = a STABLE message id                    (never changes; `ack` names it forever)
GSI "by-visible-at":  ipk = queue name,  isk = when the message becomes visible

send     → put   a row with isk = now
receive  → scan-index isk <= now, take N, then RE-PUT each with isk = now + timeout
ack      → delete by (pk, sk)
```

★ **The visibility timeout is a re-put that moves the index key into the future.** No lock, no
timer, no side state — redelivery is what happens when nobody moved it again.

A stable `sk` means `ack` names the same key forever (no receipt-handle drift), and making a
message invisible is ONE atomic `put` rather than put-at-new-key + delete-at-old-key, which has
a crash window that would **duplicate the message**.

## What this cost to make possible

Drawing this is what uncovered the rest of excursus 001: `receive` needs a re-put (which found
`mem-store` appending where `sqlite-store` replaced), and `ack` needs a `delete` (which
`:wat::query::Store` did not have). Both landed, along with the `journal` data-loss bug the
first of those exposed.

## What is here

- **`sqs.wat`** — the queue service (holds a `Store` peer, GSI `by-visible-at`) plus a
  both-backends lifecycle that is the gate: send 3 / receive 2 / the two go invisible /
  the third is returned / ack / the unacked message reappears once `now` steps past the
  window. `now` is an argument — `:wat::time::now` cannot be stepped, and a sleep is a
  guess. Prints one agreed summary from mem and sqlite, or `DIFFERENTIAL-MISMATCH`.

```bash
./target/release/wat wat-scripts/queue/sqs.wat
# => bound=x;r1=a,b;r2=c;r3=;redel=b
```

## Sibling

`wat-scripts/topic/` — **wat-topic**, the SNS half. A topic fanning out to N durable queues is
the shape the pair exists for.
