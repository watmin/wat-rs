# BRIEF — entry three of ten

Build a store that fails entry *k* of *n*, point a queue at it, and report what the stack does.
`wat-scripts/scratch-pad/` only. **No production change.**

Read `DESIGN.md` first — especially the prediction and its mechanism.

## READ IN ORDER

| room | why |
|---|---|
| `wat/query.wat:519-539` | `PutRequest [rows]` / `DeleteRequest [keys]` and their **single-outcome** responses. This is the surface being provoked |
| `wat-scripts/queue/sqs.wat` — the `send` arm | `total` (count) then `Store/put`. What a partial put breaks |
| `wat-scripts/queue/sqs.wat` — the `ack` arm | ids → Keys → **one** `Store/delete`. What a partial delete breaks |
| `wat-scripts/scratch-pad/probe-a-ledger-is-a-receipt-not-a-lock.wat` | the shape of a **hand-written service in a probe** — surface, service, impls, dial |
| `wat-scripts/fanout/circuit.wat:113-148` | the seeded `hit?` drop pattern (`int-from`, rate in basis points). Copy it for the entry selector |

## THE WRAPPER

`:fs::failing-store` satisfies `:wat::query::Store`, holds a real store peer, and forwards every
verb unchanged **except**:

- `put` — with `put-fail-bp > 0`, apply **k of n** rows (drop one, seeded), then return the
  response the surface allows.
- `delete` — same, for keys.

⚠ **It must actually apply the partial write.** A wrapper that returns an error *without* writing
k of n models "the store did nothing", which is a different and already-handled fault. **The
whole point is that some entries landed and the caller cannot tell which.**

## THE PROBE

Two cells, each on its own queue + failing-store:

1. **partial put** — send a batch of 10, `put-fail-bp` on. Report the store's response, the
   queue's `SendResponse`, and then drain: `total`, `distinct`.
2. **partial delete** — send 10 cleanly, receive them, ack the batch with `delete-fail-bp` on.
   Report the store's response, the queue's `AckResponse`, and whether the rows survive to be
   redelivered.

## BLAST RADIUS

`wat-scripts/scratch-pad/` only. **No `wat/`, no `src/`, no `sqs.wat`, no `circuit.wat`.**

## STOP TRIGGERS

- **STOP-1** — if the wrapper cannot return a response that is *both* truthful and expressible
  (the partial case has no honest variant), **STOP and report that as the finding.** That is the
  DESIGN's whole thesis and it is worth more than the behavioural numbers.
- **STOP-2** — if a queue cannot be pointed at a probe-local store service, STOP and report. The
  queue takes a `store-addr`; a wrapper satisfying the same surface should drop in.
- **STOP-3** — do not fix anything. Do not add per-entry outcomes. Do not touch the real stores.
- **STOP-4** — nothing outside `scratch-pad/`.

## WHAT TO REPORT

1. The literal response the store returns on a partial put and a partial delete.
2. What the queue above can infer from it.
3. `total` / `distinct` — **whether a message is lost or duplicated.**

★ (3) is the row the types cannot answer, and the reason this is chaos rather than a code review.

## PRIOR RESULT TO COPY

`docs/excursus/2026/08/001-sns-sqs/a-ledger-is-a-receipt/SCORE.md` — a probe that built a miniature service to isolate a semantic
question, with its cells named so a failure says which half broke.
