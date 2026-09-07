# BRIEF — a count never reads more than it needs

Add `limit` to `CountIndexRequest`, make both store impls saturate at it, and pass the bound the
queue already computes. Read `DESIGN-a-count-never-reads-more-than-it-needs.md` first — especially
*what this is and is not worth*.

## READ IN ORDER

| room | why |
|---|---|
| `wat/query.wat:572` | `CountIndexRequest [index ipk isk-lo isk-hi]` — gains `limit <- i64` |
| `wat/query.wat:578` | `CountIndexResponse::Ok [n]` — `n` becomes saturating; the arm shape does not change |
| `wat/query/sqlite-store.wat:457` | the `COUNT(*)` with no `LIMIT` — the subquery goes here |
| `wat/query/mem.wat` `count-index` impl | the in-memory twin must saturate identically, or the oracle and the backend disagree |
| `wat-scripts/queue/sqs.wat` `total` closure | `[st q _now-ns _lim]` — `_lim` is the bound, already computed as `cap + 1`, currently discarded |
| `wat-scripts/queue/sqs.wat` `depth` closure | the sibling that counts visible-only; it needs a limit too, and it already has `lo`/`hi-ns` |
| `wat-scripts/queue/sqs.wat:101` | `Queue::send`'s feature line — where the cap gate's caller lives |

## SKETCH

Surface:

```wat
(:wat::core::defrecord :wat::query::Store::CountIndexRequest
  [index  <- :wat::core::String
   ipk    <- :wat::core::String
   isk-lo <- :wat::core::String
   isk-hi <- :wat::core::String
   limit  <- :wat::core::i64])
```

sqlite:

```sql
SELECT COUNT(*) FROM (SELECT 1 FROM [index_{name}]
                      WHERE ipk=?1 AND isk>=?2 AND isk<=?3 LIMIT ?4)
```

mem: stop folding the whole index — count while walking and **stop at `limit`**.

Callers: `total` and `depth` drop the underscores and pass their bound (`cap + 1` — one more than
the cap, so "at cap" and "over cap" stay distinguishable).

## BLAST RADIUS

`wat/query.wat`, `wat/query/sqlite-store.wat`, `wat/query/mem.wat`,
`wat-scripts/queue/sqs.wat`, scratch probes. **No `src/`, no `wat/service.wat`, no
`circuit.wat`, no `sns-fanout.wat`.**

## STOP TRIGGERS

- **STOP-1** — the mem impl cannot stop early (its structure forces a full fold). Report it; a
  saturating sqlite and an exhaustive mem are two different contracts wearing one surface, and
  `mem` is the oracle other tests compare against.
- **STOP-2** — `cap + 1` is not sufficient to distinguish "at cap" from "over cap" at some call
  site. Report which; the off-by-one is the whole correctness of the gate.
- **STOP-3** — any existing caller of `count-index` needs a true unbounded count. Report it and
  give it an explicitly large limit rather than an optional field — absence of a bound is what
  this stone removes.
- **STOP-4** — anything outside the blast radius.

## THE MEASUREMENT

⚠ **This stone is not judged on the wall clock.** At `cap 64` the DESIGN predicts roughly neutral.

1. A probe proving saturation: an index with 5000 rows, `limit 65` → `Ok 65`, and the same query
   with `limit 100000` → `Ok 5000`.
2. `circuit.wat` ×3, shipped: `publish` ~20700 ±300, `distinct=8000`, `dup=0` — **unchanged**.
3. Both store impls give the same answer for the same request.

## GRADE AGAINST

`SCORE-a-read-declares-its-page.md` — the sibling bound, on rows returned rather than rows
examined.

Write `SCORE-a-count-never-reads-more-than-it-needs.md`, then `pulsare_yield kind=scored`.
