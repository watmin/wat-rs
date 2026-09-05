# BRIEF — the queue can drop too

Give `:queue::queue` per-verb drop knobs for `receive` and `ack`, so the last two undeadlined-
until-recently client calls can actually be made to fail.

Read `DESIGN-the-queue-can-drop-too.md` first.

## ⛔ THE MIGRATION IS A CODEMOD. DO NOT HAND-EDIT `.wat`.

Three new `:durable` fields mean **48 constructor sites across 10 files**. Per `CLAUDE.md`, a
structural rewrite across many `.wat` files is a **wat-fix codemod** — `wat/fix.wat`, recorded
under `wat-scripts/fixes/`.

**Copy `wat-scripts/fixes/add-event-id-to-metric-log-ctors.wat`** — same shape, idempotent,
comment-faithful.

```
# census FIRST — prints matches unapplied
./target/release/wat --grep ./wat-scripts/fixes/declare-queue-drop-knobs.wat < paths.json
# then apply
printf '["p1" "p2" …]\n' | ./target/release/wat ./wat-scripts/fixes/declare-queue-drop-knobs.wat
```

⚠ **Count occurrences, not lines** — the finder emits one long line and `grep -c` undercounts.
⚠ **Comments are not rewritten**; prose is a separate manual pass if any is needed.

The ten files: `wat-scripts/queue/sqs.wat`, `wat-scripts/fanout/circuit.wat`,
`wat-scripts/topic/sns-fanout.wat`, and `wat-scripts/scratch-pad/` — `probe-three-waiters-wake`,
`probe-instrument-reports`, `probe-visibility-redelivers`, `probe-parked-waiters-stop`,
`probe-stats-sees-an-expired-unacked`, `probe-depth-derived-from-the-index`,
`probe-refused-retry-self-consumes`.

## READ IN ORDER

| room | why you are there |
|---|---|
| `sqs.wat:111-117` | the queue `defservice` `:durable` — gains `drop-recv-bp`, `drop-ack-bp`, `drop-seed` |
| `circuit.wat:113-148` | `:fanout::seen`'s `mark` impl — **the drop pattern to copy verbatim**: `hit?`, seeded `int-from`, reply hidden as `:wat::core::None` |
| `sqs.wat` `receive` impl | hides the reply on a hit. **The lease still happens** — that is the fault being modelled |
| `sqs.wat` `ack` impl | hides the reply on a hit. **The delete still happens** |
| `circuit.wat:1425` | `:user::drop-check-tiny` — the cell shape to copy for two new cells |
| `tests/services/probe_arc278_sane_circuit.rs:152` | `drop_check_tiny` — the `#[ignore]`d cell shape to copy |

## BLAST RADIUS

`sqs.wat`, `circuit.wat`, the new `wat-scripts/fixes/declare-queue-drop-knobs.wat`, the 8 other
`.wat` files the codemod touches (field addition only), and two `#[ignore]`d cells in
`probe_arc278_sane_circuit.rs`. **No `wat/`, no `src/`.**

## STOP TRIGGERS

- **STOP-1** — if the codemod cannot be made idempotent (re-running it double-inserts), STOP.
  A migration that is not safe to re-run is not a recorded migration.
- **STOP-2** — if a second ack of an already-deleted row **errors** rather than being a no-op,
  STOP and report. That is a real finding about `ack` idempotency and it changes the stone.
- **STOP-3** — if hiding the `receive` reply requires *not* leasing the envelopes, STOP. The
  lease-then-lose-the-reply is the fault; a drop that skips the work models nothing.
- **STOP-4** — do not aim at `Queue/send`, do not touch the redelivery fixture, no perf work.

## PRIOR RESULT TO COPY FOR SHAPE

`SCORE-a-give-back-is-counted.md` — the immediately prior strike, and the counter discipline it
established: every fault the circuit can suffer is counted and printed.
