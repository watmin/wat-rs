# BRIEF — the consumer is idempotent

The circuit asserts exactly-once on an at-least-once system, and its duplicate detector cannot see
the duplicates at-least-once produces. Give messages a stable identity, dedupe on it in the
consumer, and count identities rather than envelope ids.

## Read in order

1. **`DESIGN-STONE-the-consumer-is-idempotent.md`** — the contract decision (identity is a domain
   fact assigned at publish, not a transport fact minted per send).
2. **`FINDING-durability-is-store-op-bound.md`**, the red-arm section — the actual failure, captured
   at `.floor/2026-09-03T04-22-46Z/ARM.txt`. **Do not re-derive it.**
3. **`wat-scripts/fanout/circuit.wat:441`** — `:fanout::key-of` (keys on `queue/id`, the envelope)
   and **`:fanout::body-key`** immediately below it (keys on the body, **defined and never called**).
   The second is the shape you want; find out why it was never wired before reusing it.
4. **`wat-scripts/queue/sqs.wat`**, `send` — `sk = edn::write(uuid::v4)`, minted per call. This is
   why a redelivery is a *new* envelope and why `dup` stays 0.
5. **`tests/services/probe_ex001_fanout.rs:48-54`** — the assertions that must become meaningful.

## The sketch

Load-bearing: identity comes from the publisher and dedupe happens in the consumer. Illustrative:
where the id sits in the body.

```
publish   body already begins with <seq>  (the trace's first field, placed first for exactly this)
worker    on receive:  id = seq-of(body)
                       if seen(queue, id) -> ack and DROP   (idempotent: already processed)
                       else               -> record outcome, mark seen, ack
summary   distinct = count of DISTINCT (queue, message-id), not (queue, envelope-id)
```

## Blast radius

`wat-scripts/fanout/circuit.wat` (the worker's dedupe + `summarize`), and
`tests/services/probe_ex001_fanout.rs` if the assertions change shape.
**`wat/`, `src/`, `sqs.wat` and `sns-fanout.wat` untouched.**

## STOP triggers

1. **If you find yourself deduping in the queue or the store — STOP.** At-least-once is the queue's
   correct contract; idempotence belongs to the consumer.
2. **If `distinct` can no longer detect a genuinely lost message — STOP.** The invariant must still
   catch loss; this stone makes it *also* catch duplication.
3. **If the visibility window needs widening to make a row pass — STOP.** That is the thing this
   stone exists to stop relying on.
4. **If `wat/` or `src/` need to change — STOP and surface it.**

## Floor

`./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a red — do not
re-run, name the arm, surface it. Check `ps` before any timing. **Five runs, report the spread.**

Write `SCORE-the-consumer-is-idempotent.md` when done. It will be graded by re-running.
