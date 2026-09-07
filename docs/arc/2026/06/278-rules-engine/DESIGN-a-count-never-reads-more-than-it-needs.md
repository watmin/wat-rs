# DESIGN — a count never reads more than it needs

**`CountIndexRequest` gains a `limit`; the count saturates.** `wat/query.wat` +
`wat/query/sqlite-store.wat` + `wat/query/mem.wat` + `wat-scripts/queue/sqs.wat`.

## WHY — the cap gate asks a question it does not need answered

The queue's cap gate wants one bit: **is depth ≥ cap?** What it does instead, on **every send**:

```sql
SELECT COUNT(*) FROM [index_by-visible-at] WHERE ipk=?1 AND isk>=?2 AND isk<=?3
```

No `LIMIT`. `O(matching rows)`. And the range it passes (`sqs.wat`, the `total` closure):

```wat
:isk-lo (edn::write (time::at-nanos 0))
:isk-hi (edn::write (time::at-nanos 4000000000000000000))
```

**The entire key range.**

★★★ And the smoking gun is in the closure's own signature:

```wat
[st <- Peer  q <- String  _now-ns <- i64  _lim <- i64]
```

**`_lim` and `_now-ns` are underscore-prefixed — passed in and discarded.** The caller computed
the bound (`cap + 1`) and the clock it needed and had **nowhere to put them**, because
`CountIndexRequest` is `[index ipk isk-lo isk-hi]` and carries neither.

## ⛔ THE DEFECT CLASS — we bounded what is RETURNED, never what is EXAMINED

`:max-page` bounds the rows a read *returns*. `count-index` returns a single i64 and was therefore
never suspected — but it **examines** an unbounded number of rows to produce it. The same class,
one level down, and invisible to every bound we have built.

★ Contrast the sibling, which is already correct:

```sql
SELECT … FROM [index_{name}] WHERE ipk=?1 AND isk>=?2 AND isk<=?3 … ORDER BY isk ASC LIMIT ?5
```

`scan-index` reads exactly what was asked for. `count-index` reads everything.

## ⛔ THE ONE CONTRACT DECISION — a saturating count

`CountIndexRequest` gains `limit <- i64`. `CountIndexResponse::Ok [n]` becomes **"n, saturating at
limit"** — the caller learns `min(true_count, limit)`, which answers "is depth ≥ cap?" exactly.

```sql
SELECT COUNT(*) FROM (SELECT 1 FROM [index_{name}]
                      WHERE ipk=?1 AND isk>=?2 AND isk<=?3 LIMIT ?4)
```

`O(min(depth, limit))`. At `cap 64` it never examines more than 65 rows, whatever the depth.

★★ A saturating count is not a weaker answer — it is the **exact** answer to the question the
caller is asking. A caller wanting a true total passes a limit large enough to mean "no bound",
and that is then a deliberate, visible choice rather than an accident of the surface.

## WHAT THIS IS AND IS NOT WORTH — stated before measuring

⚠ **At `cap 64` this should be roughly neutral.** The index holds visible + in-flight rows; the
count is already small, so removing an `O(depth)` term that is currently `O(~100)` buys little.

★★ **Its value is at depth**, and it is a **prerequisite** for the benchmark the builder actually
wants: fill deep, then measure drain rate. Without it, a 15-million-message drain test would spend
its time in *our own* `COUNT(*)` and measure the instrument instead of the system.

⚠ So this stone is not judged on the circuit's wall clock. It is judged on the count being bounded
and behaviour being unchanged.

## OUT OF SCOPE — REJECTED

- **Restricting the range to visible-only.** `total` is *meant* to be visible + unacked; the full
  range is semantically right for it. `_now-ns` being discarded is a separate smell and the
  sibling `depth` closure already uses a clock. Not this stone.
- **The deep-drain benchmark.** The thing this unblocks; its own stone.
- **Caching depth in service state.** `SCORE-stop-fetching-rows-to-get-a-number` deleted those
  counters deliberately. Bounding the read is the cheap fix that does not re-open that ruling.
