# BRIEF — the server manages its own capacity

Make `Queue::send` admit a prefix of what it is handed, report `Accepted [count]`, and stop
returning `depth`/`cap`. Make `Topic::publish` do the same in messages. Remove `:max-entries` from
`Queue::send`. Read `DESIGN-the-server-manages-its-own-capacity.md` first — it opens with a ruling
of mine that this stone reverses, and says why.

## READ IN ORDER

| room | why |
|---|---|
| `wat-scripts/queue/sqs.wat:344-362` | the admission gate. `depth`, `cap`, `n0`, and the all-or-nothing `if` at `:353` |
| `sqs.wat:48-54` | `SendResponse` — `:Ok` and `:Full [depth cap]` collapse into `:Accepted [count]` |
| `sqs.wat:103` | the `send` feature — **remove** `:max-entries [bodies 64]`; keep `:max-request-bytes` |
| `sqs.wat:363-380` | rows are built from `bodies` in order with a 1 ns `isk` stagger per index — this is why a PREFIX is the honest unit |
| `wat-scripts/topic/sns-fanout.wat:84-99` | `publish` fans **msg-major**; a prefix of pairs is whole messages plus a partial tail |
| `sns-fanout.wat:42-48` | `PublishResponse` — same collapse |
| `circuit.wat:1002-1028` | `publish-until-accepted!*` — the retry loop. It must now resend **from `count`**, not resend the whole batch |
| `circuit.wat:1467`, `:1791` | the two publish loops |

## SKETCH

Admission, replacing the `if` at `sqs.wat:353`:

```wat
room  (:wat::i64::- cap depth)                       ;; may be <= 0
take  (:wat::core::if (:wat::i64::< n0 room) n0 room)
take  (:wat::core::if (:wat::i64::< take 0) 0 take)
```

- `take == 0` → reply `Accepted 0`, write nothing
- `take > 0`  → build rows from the **first `take`** bodies, one `Store/put`, reply `Accepted take`

The store write stays exactly as it is — one atomic put of the admitted rows.

Topic:

```wat
pairs-accepted  <- from the inbox's Accepted
msgs-accepted   (:wat::i64::/ pairs-accepted nsubs)      ;; floor: a partial tail message is NOT accepted
```

Driver — the retry loop resends the tail:

```wat
;; Accepted count  → drop the first `count`, resend the rest; Accepted 0 → wait 1ms, resend all
```

## BLAST RADIUS

`wat-scripts/queue/sqs.wat`, `wat-scripts/topic/sns-fanout.wat`, `wat-scripts/fanout/circuit.wat`,
scratch probes. **No `wat/`, no `src/`.** Do not change any `:cap`.

⚠ `probe-a-batch-declares-how-many.wat` currently asserts `RequestTooManyEntries(65,64)` on
`Queue/send`. Removing that cap makes it stale — **retarget it to `Topic/publish`**, which keeps
its cap of 10, rather than deleting the coverage.

## STOP TRIGGERS

- **STOP-1** — a prefix cannot be expressed because rows are not built in body order. Report it;
  the contract depends on order.
- **STOP-2** — `Accepted 0` and "no room" are not the same condition (something else can yield
  zero). Report it — a single arm meaning two things is the defect this arc exists to close.
- **STOP-3** — the topic cannot compute `floor(pairs/nsubs)` because the inbox's Accepted count
  does not correspond to a prefix of the pairs it sent. Stop; msg-major ordering is load-bearing.
- **STOP-4** — removing `:max-entries` from `Queue::send` requires touching `wat/` or `src/`.
  It should not: absent means uncapped, and no def is emitted.
- **STOP-5** — anything outside the blast radius, or any change to a `:cap`.

## THE PROBE YOU WILL NEED

Through the surface spelling, on a queue with `cap 10` holding 6:

- send 8 → `Accepted 4`, depth 10, and **exactly the first 4 bodies** are the ones present
- send 3 more → `Accepted 0`, depth unchanged
- drain 5, send 8 → `Accepted 5`
- and: **no response carries `depth` or `cap`** — `grep` the response enums

## GRADE AGAINST

`SCORE-the-topic-publishes-a-batch.md` — same files, one stone earlier.

Write `SCORE-the-server-manages-its-own-capacity.md`, then `pulsare_yield kind=scored`.
