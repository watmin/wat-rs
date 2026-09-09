# DESIGN — the queue can drop too

**Close the last of the chaos coverage gap.** `wat-scripts/queue/sqs.wat` +
`wat-scripts/fanout/circuit.wat` + a **recorded codemod** across the corpus. Correctness. No perf.

## WHY — two of four deadlines have never been exercised

An earlier stone put all four of the worker's client calls behind
`:wat::service::call-by-deadline`. Only two can be made to fail:

| call | droppable | deadline exercised? |
|---|---|---|
| `Seen/check` | ✅ `drop-check-bp` | ✅ |
| `Seen/mark` | ✅ `drop-mark-bp` | ✅ |
| `Queue/ack` | ⛔ | **never** |
| `Queue/receive` | ⛔ | **never** |

★ The `mark` deadline was built *because* a dropped `mark` hung a worker ~160 s. **`ack` and
`receive` have the same shape and have never been made to fail.** They are the last unexercised
fault paths in the consumer.

## WHAT A DROP MEANS ON EACH — reasoned through, because they differ

- **`receive`**: the server has already leased the envelopes when the reply is dropped. The
  client never learns which. They stay invisible until visibility expires, then return. With
  `vis = 200 ms` on drop runs, that is a redelivery — the mechanism `probe-visibility-redelivers.wat`
  already proves.
- **`ack`**: the row is already deleted when the reply is dropped. The client's retry acks a row
  that is gone. **This stone's correctness question is whether that second ack is a no-op or an
  error**, and EXPECTATIONS row 4 asks it directly rather than assuming.

★ Both are safe *given the receipt discipline*: nothing is emitted or recorded on either path, so
a lost reply costs a redelivery, never a message.

## ⛔ THIS IS A CORPUS MIGRATION, AND IT USES THE TOOL

`:queue::queue::Record` has no field defaults — checked; `defrecord` has none. So three new
`:durable` fields mean **every constructor must supply them: 48 sites across 10 files.**

```
wat-scripts/queue/sqs.wat            wat-scripts/fanout/circuit.wat
wat-scripts/topic/sns-fanout.wat     + 7 scratch-pad probes
```

⛔ **`.wat` corpus migrations go through `wat/fix.wat`, never hand-edits, never python/sed.**
The exemplar is `wat-scripts/fixes/add-event-id-to-metric-log-ctors.wat` — the same shape
(*"every kwargs constructor of those records must supply the field"*), idempotent, comment-faithful.

★ **The corpus gate is the census.** `every_wat_scripts_file_loads_on_the_current_runtime`
type-checks all 7 scratch-pad probes; a constructor the codemod misses reds the floor. The
migration cannot be silently incomplete.

## ⛔ THE ONE CONTRACT DECISION

**The knobs are per-verb and default to zero, exactly as `:fanout::seen`'s are.**
`drop-recv-bp`, `drop-ack-bp`, and a shared `drop-seed` — so aiming at one verb never darkens
another. That property is the whole lesson of the stone that restored `check`: every previous
injector was a single knob, and every move of it blinded the path it left.

## FILES

`wat-scripts/queue/sqs.wat` (the `:durable`, the two impls), `wat-scripts/fanout/circuit.wat`
(cells that turn the knobs on), and a new recorded migration
`wat-scripts/fixes/declare-queue-drop-knobs.wat` applied to all 10 files.

## OUT OF SCOPE = REJECTED

- **Dropping `Queue/send`.** The publisher is not the consumer, its failure mode is
  `never-accepted` rather than a hang, and mixing them makes the rows unattributable.
- **The topic's internal queue.** `sns-fanout.wat` gets the field (it must, to compile) but is
  not aimed at in this stone.
- **The redelivery fixture**, the rung-3 census, and all perf work.
