# BRIEF — a read declares its page

Add `:max-page [field N]` bounding a response collection, generate `<op>-all` for cursor reads,
and adopt both on `Store::scan-index` and `Queue::receive`. Read
`DESIGN.md` first — it says what this stone does **not** fix.

## READ IN ORDER

| room | why |
|---|---|
| `src/types/surface.rs:442` | the `":max-request-bytes" =>` option arm — your parse exemplar |
| `src/types/surface.rs:486` | the `":max-entries" =>` arm — the `[field N]` vector parse you copy |
| `src/types.rs:3799` | where `<S>::<OP>-MAX-ENTRIES` / `-MAX-ENTRIES-FIELD` are emitted as defs |
| `wat/service.wat:2423` | `method-name` and where the op-method `defn` is emitted — `<op>-all` is a sibling |
| `wat/query.wat:556-570` | `ScanIndexRequest [.. limit cursor]` / `ScanIndexResponse::Success [rows cursor]` — the shape that qualifies |
| `wat-scripts/queue/sqs.wat:62-67` | `ReceiveRequest [.. limit wait]` — **no cursor**, so bound only |
| `wat-scripts/queue/sqs.wat:79-83` | `ReceiveResponse::Ok [envelopes]` — the unbounded collection |
| `wat-scripts/queue/sqs.wat:170` | the one `scan-index` call, passing `:cursor :wat::core::None` |
| `wat/seq.wat:76` | `(seq [self] -> (:wat::stream::Stream :- [T]))` — the return type of `<op>-all` |

## SKETCH

Declaration, on the read op:

```wat
(receive [self <- :queue::Queue  req <- :queue::Queue::ReceiveRequest]
  -> :queue::Queue::ReceiveResponse
  :max-request-bytes 524288
  :max-page [envelopes 64])
```

Emits `:queue::Queue::RECEIVE-MAX-PAGE` and `-MAX-PAGE-FIELD`, exactly as `:max-entries` does.

**Enforcement is truncation, not rejection.** The service returns at most N; `limit` is a request,
not a promise. ⚠ This is the opposite of `:max-entries`, which rejects — say so in the SCORE, it
is the one place the two sides differ and a reader will expect symmetry.

Emission of the tool, iff **all three**: request has `cursor <- Option[String]`; response success
arm has the paged collection; response success arm has `cursor <- Option[String]`.

```wat
;; pull pages lazily until the cursor comes back None
(:wat::core::defn :<Proto>/<op>-all [c req] -> (:wat::stream::Stream :- [<Item>]) …)
```

## BLAST RADIUS

`src/types/surface.rs`, `src/types.rs`, `wat/service.wat`, `wat/query.wat`,
`wat-scripts/queue/sqs.wat`, scratch probes. **No `circuit.wat`, no `sns-fanout.wat`.**
**Change no caller's `:limit`.**

## STOP TRIGGERS

- **STOP-1** — the response's success arm and its collection field cannot be identified at expand
  time. Report it; the emission condition is the design.
- **STOP-2** — `receive` acquires an `-all`. It has no cursor and its reads are **leased**; an
  `-all` would drain the queue rather than page it. If the condition emits one, the condition is
  wrong.
- **STOP-3** — truncation cannot be applied without the op's impl cooperating (i.e. it is not
  enforceable generically at the boundary the way `:max-entries` is). **Report where it would have
  to live** — do not push it into each impl by hand.
- **STOP-4** — a lazy `Stream` cannot cross a service boundary or cannot be built from a pull loop.
  Report it; an eager vector would defeat the point and re-create the unbounded response.
- **STOP-5** — anything outside the blast radius, or any `:limit` changed at a call site.

## THE PROBE YOU WILL NEED

Against a store holding 250 index rows, page bound 64:

- one `scan-index :limit 1000` returns **≤ 64** rows and a `Some` cursor — the bound truncates
- `scan-index-all` yields **all 250**, in order, across 4 pages, ending on a `None` cursor
- `Queue::receive :limit 1000` returns **≤ 64** envelopes
- `grep` shows **no** `receive-all` emitted

## GRADE AGAINST

`docs/excursus/2026/08/001-sns-sqs/a-caller-chunks-to-the-declared-limit/SCORE.md` — the write-side sibling, same emission-condition
shape.

Write `SCORE.md`, then `pulsare_yield kind=scored`.
