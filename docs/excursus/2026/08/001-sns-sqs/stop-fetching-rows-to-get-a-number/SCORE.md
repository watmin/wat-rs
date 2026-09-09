# SCORE — stop fetching rows to get a number

**STRUCK.** Executor: grok, 2026-09-05. Tree safe, uncommitted.
`wat/query.wat` + `wat/query/mem.wat` + `wat/query/sqlite-store.wat` + `wat-scripts/queue/sqs.wat`.
(Probe harness also gained a `count-index` timer — the unit-cost proof.)

```
Summary [ 380.869s] 5215 tests run: 5215 passed (3 slow), 22 skipped
```

`.floor/2026-09-05T23-22-15Z/`

A depth question is answered by counting. The send path asks only what it uses.

## THE TWO CHANGES, ATTRIBUTED

**Change 1 — one scan, not two.** `total` sits beside `depth` on State. Cap gate + three Full arms call it. `stats` keeps `depth`. Quiet circuit ×5:

```
publish  41260 42001 42660 42954 42632     median 42632
```

Before: median **51300**. Halving the round trips moved ~8.7 s.

**Change 2 — count, not fetch.** `count-index` on `:wat::query::Store`, both stores. sqlite is `SELECT COUNT(*)` — no row vector. mem walks the existing GSI partition and returns `n` — no `IndexRow`. `depth` and `total` both use it. Quiet circuit ×5:

```
publish  36902 38188 37482 38011 35917     median 37482
```

Gate was **≤ 42000**. 37482 holds.

Together: **51300 → 42632 → 37482** (median). ~27 % off the before-median.

## ROW 6 — unit cost, re-run of the harness

```
LIMIT65 rows=60 us_per_scan=1343 proj_s_16000=21
LIMIT1  rows=1  us_per_scan=509  proj_s_16000=8
COUNT   n=60    us_per=468       proj_s_8000=3
```

Before: LIMIT65 **1336** / LIMIT1 **448**. COUNT is the LIMIT1 class (query overhead, no materialization) and returns the same `n=60`. 8000 counts × 468 µs ≈ **3.7 s** — the DESIGN's ~3.5 s arithmetic, measured.

## STOP-1 / STOP-2 / STOP-3

- **STOP-1** did not fire. `Store::Op`/`Reply` are not exhaustively matched outside the two store impls. `defservice` generated the dispatch.
- **STOP-2** did not fire. sqlite never builds rows. mem never builds `IndexRow`.
- **STOP-3** did not fire. `probe-depth-derived-from-the-index.wat`: `agree=yes` both cells. `probe-stats-sees-an-expired-unacked.wat` unchanged: `EXPIRED-NO-RECEIVER=[1/0]; after-receive=[0/1]; AFTER-ACK=[0/0]`.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ send path one store call | ✅ cap gate `:319` and Full arms `:480`/`:510`/TimedOut call `total`. `stats` still `depth` |
| 2 | ★★ the call returns no rows | ✅ sqlite `COUNT(*)`; mem `count-index-range` over StoredRows already in the partition |
| 3 | ⛔ floor `5215 passed`, 22 skipped | ✅ `Summary [ 380.869s] 5215 tests run: 5215 passed (3 slow), 22 skipped` |
| 4 | ⛔ numbers still agree | ✅ depth probe `agree=yes`; stats probe unchanged |
| 5 | ★★ publish, two steps | ✅ after 1: median **42632**. after 2: median **37482** ≤ 42000 |
| 6 | unit cost | ✅ COUNT **468 µs** vs 1336 / 448 baselines |
| 7 | chaos | ✅ check/mark/recv/ack-drop ×3: `total=100; distinct=100; dup=0` |
| 8 | rate-0 | ✅ circuit ×5 `total=8000; distinct=8000; dup=0; seen-recorded=8000` |
| 9 | ★ new dominant term | **publish is still the longest stage clock (median 37.5 s). The outbox cliff is gone.** |

## ★ ROW 9 — the new leader

Before (verbatim): `outbox 250-1000=7739 max=5693ms`; t1→t2 and t2→t3 `<1ms=8000`; t3→t4 max=4333ms.

After change 2 (quiet, run 1; the five agree in shape):

```
setup=9674  publish=36902  drain=254  stop=5489
outbox   50-250=5250  250-1000=2720  max=361ms
t1->t2   <1ms=8000
t2->t3   <1ms≈8000
t3->t4   10-50=4089  50-250=2903  250-1000=220  >1000=5  max=1071ms
e2e      250-1000=5472  max=1358ms
```

**Largest remaining contributor: the publish stage clock itself (median 37.5 s of wall), and inside it outbox (topic-inbox queueing) is still the fattest per-message delay — 2720 messages in 250–1000 ms — but the 5.7 s cliff is gone (max 361 ms).**

Next wall-clock terms now *visible* because publish is no longer 50 s: **setup ~9.7 s**, **stop ~5.5 s**. Next per-message stage: **t3→t4** (worker work; max ~1.1 s). `count-index` is ~3.7 s of the 37 s publish; the rest is put + wat around the send. The DESIGN already named the follow-ups (Record rebuild on receive/ack, select+timer per client call). Interpretation is not yet the leader.

## NOT TOUCHED

`src/`. `circuit.wat`. Record rebuild. select+timer. Compiled wat.

---

Tree uncommitted. Do not commit unless asked.

---

# ORCHESTRATOR GRADING — claude, 2026-09-05

**STRUCK.** Every row re-run by me on a quiet box.

| # | my result | |
|---|---|---|
| 1/2 | cap gate + Full arms call `total`; `stats` keeps `depth`; sqlite `COUNT(*)`, mem no `IndexRow` | ✅ |
| 3 | `Summary [ 375.530s] 5215 tests run: 5215 passed, 22 skipped` — `.floor/2026-09-05T23-43-25Z/` | ✅ |
| 4 | depth probe `agree=yes` both cells; stats probe **byte-identical** | ✅ |
| 5 | publish `37318 37769 37409 37095 36364` — **median 37318** vs before **51300** = **−27.3 %**, gate ≤42000 | ✅ |
| 6 | `COUNT n=60 us_per=467` vs `LIMIT65 1350` / `LIMIT1 504` | ✅ |
| 7 | check/mark/recv/ack-drop ×3 each: `distinct=100` | ✅ |
| 8 | rate-0 ×5: `total=8000; distinct=8000; dup=0` | ✅ |

★ **Row 6 is the whole stone in one line: `COUNT` returns `n=60` — the same answer as fetching
60 rows — at 467 µs instead of 1350.** Same number, 65 % cheaper, because the rows are never
built. My medians land within 0.4 % of the executor's.

## ★ ROW 9 — THE NEW LEADER, READ BY ME

```
setup=9671  publish=37289  drain=235  stop=5833
outbox   50-250=5222  250-1000=2746  >1000=0  max=342ms
t1->t2   <1ms=8000     t2->t3  <1ms=7990
t3->t4   10-50=4048  50-250=2913  250-1000=221  >1000=9  max=1359ms
```

**The outbox cliff is gone** — `max` 5693 ms → **342 ms**, and `>1000` is **0**. The stage that
carried the whole regression no longer has a tail.

**Ranked, by wall clock:**

1. **`publish` — 37.3 s.** Still the leader by far.
2. **`setup` 9.7 s + `stop` 5.8 s = 15.5 s — 29 % of wall.** Process spawn/reap, ruled out of this
   arc long ago as a boot-time item. **It is now a third of the run and that ruling has expired.**
3. **`t3->t4`** — worker work, per-message, `max` 1359 ms.

## ⛔ WHAT THE NEXT STONE MUST DO — DECOMPOSE, NOT GUESS

`count-index` is **~3.7 s of the 37.3 s** publish (8000 × 467 µs). **The other ~33.6 s is
unattributed.** We optimized the part the instrument could already see; what remains is the part
it cannot.

★ So the next act is **not** another optimization — it is **decomposing `publish`**, the way the
stage histograms decomposed the pipeline. The two known candidates (`Record` rebuild on
`receive`/`ack`; select+timer per client call) are *hypotheses*, and picking one without
measuring would be exactly the guessing this arc has been punished for.

★★ **Interpretation is not yet the leader**, so the terminal condition is not met and the loop
continues.
