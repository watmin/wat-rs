# EXPECTATIONS — a single value is a batch of one

Written before the strike. Every "before" is a run of mine on `26873d2de`, quiet box.

| # | what | expected |
|---|---|---|
| 1 | ★★ **no singular form remains** | `grep` finds **0** `AckRequest :id`, `CheckRequest :seq`, `MarkRequest :seq`. Only `:ids` / `:seqs` |
| 2 | ★★ **round trips collapse** | the worker makes **one** check, **one** mark, **one** ack per received batch; the topic worker **one** ack per bucket |
| 3 | ★★ **throughput** — the metric the cap cannot move | report `publish + drain` ×5. Before: **37.5 s** (37.3 + 0.24). **Target ≤ 30 s** |
| 4 | ⛔ **nothing is lost** | `total=8000; distinct=8000` ×5 at rate 0 |
| 5 | ⛔ **`distinct` holds under chaos** | check/mark/recv/ack-drop ×3 each: `distinct=100`. **`dup` may rise — report it** |
| 6 | ⛔ **the s3 gate still holds** | `redelivery_mid_processing_never_loses` passes; report its `total`/`dup` |
| 7 | **the floor** | `5215 passed`, 22 skipped |
| 8 | blast radius | `wat-scripts/` only — 5 files, **no `wat/`, no `src/`** |
| 9 | ★ **the new dominant term** | from the stage histograms: **name the largest remaining contributor**, with numbers |

### Before-state, recorded verbatim

```
row 3  publish 37318 37769 37409 37095 36364 (median 37318); drain ~235 ms
       publish+drain ≈ 37.5 s   =  8000 deliveries / 37.5 s  =  213/sec
row 4  total=8000; distinct=8000; dup=0 x5
row 5  distinct=100 on all twelve chaos runs
row 6  total=2; distinct=1; dup=1  (6/6 identical)
row 7  Summary [371.027s] 5215 passed, 22 skipped  .floor/2026-09-06T00-11-34Z/
row 9  setup=9671 publish=37289 drain=235 stop=5833
       outbox 50-250=5222 250-1000=2746 max=342ms; t3->t4 max=1359ms
units  rt_process=179us  put=675us  count=517us  rt_thread=143us
```

## ⛔ ROW 3 IS `publish + drain`, NOT `publish`

Last stone proved `publish` alone is a **Goodhart metric**: raising the inbox cap moved 15 s out
of `publish` into `drain` with throughput unchanged to 0.3 %. **This stone is measured on the sum**,
which the cap cannot move. `≤ 30 s` is a directional gate with a wide margin — it cannot red on
noise, and it does fail if the round trips did not actually collapse.

## ⛔ ROW 5 EXPECTS `dup` TO POSSIBLY RISE, AND THAT IS NOT A FAILURE

Batching widens the s3 window from one message to ten: between `check` and `mark` the whole batch
is unmarked. **`distinct` is the invariant and is gated. `dup` is an observation and is reported.**

★ Stating it this way because I have three times in this arc gated an observation and had it fire
wrongly — and once, in the DupSelf stone, gated `dup=0` against this arc's own standing ruling.

## RUNTIME PREDICTION

90–150 min. The surface changes are small; the worker fold is the real work, and STOP-2 (mark
must stay after emit) is the trap inside it.

## TRAP DOORS

1. **`mark` before `emit`** to simplify the fold. STOP-2 — it is claim-before renamed.
2. **Results not aligned to input order.** STOP-1; the caller cannot use an unordered answer set.
3. **A `Failed[]` that is always empty.** STOP-4.
4. **Gating `dup`.** Row 5.
5. **Measuring `publish` alone.** Row 3.
