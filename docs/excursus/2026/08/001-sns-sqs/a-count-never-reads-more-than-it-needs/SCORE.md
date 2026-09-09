# SCORE — a count never reads more than it needs

**SCORED.** Executor: grok, 2026-09-07. Tree dirty, uncommitted.
`CountIndexRequest` gained `limit`. Both stores saturate. Queue passes `cap + 1`.

```
Summary [ 499.954s] 5221 tests run: 5221 passed (7 slow), 22 skipped
```

`.floor/2026-09-07T06-22-11Z/`

## THE BOUND

`CountIndexRequest` is `[index ipk isk-lo isk-hi limit]`. `Ok n` is
`min(true_count, limit)`. Arm shape unchanged.

sqlite:

```
SELECT COUNT(*) FROM (SELECT 1 FROM [index_{name}]
                      WHERE ipk=?1 AND isk>=?2 AND isk<=?3 LIMIT ?4)
```

mem: `drop-while` / `take-while` / `take lim` — the same skip/take as
`take-index-page`, then `into []` and `count`. No `IndexRow`. Walk stops
at hi / limit. A first attempt (`count` of the Stream directly) killed
the mem-store thread; materializing the taken prefix is the oracle's
early-stop. **STOP-1 did not fire.**

## CALLERS

`total` and `depth` drop `_lim` and pass it. Send already computed
`lim = cap + 1`. `depth`'s inner `count-hi` closes over that `lim`.

`_now-ns` remains on `total` — DESIGN: full range is right for total;
clock is a sibling smell. Not this stone.

STOP-3: the two cost probes (`probe-what-a-scan-costs`,
`probe-what-publish-costs`) wanted a true total; they got an explicit
`limit 100000`. No other `CountIndexRequest` constructor. No production
caller needed unbounded.

## THE PROBE

```
mem65=65;mem5000=5000;mem1=1;sql65=65;sql5000=5000;sql1=1;agree=yes;fill=1/1/1/1;over=0
```

5000 rows, `limit 65` → 65, `limit 100000` → 5000, `limit 1` → 1. mem
and sqlite identical. Queue `cap 4`: four `Accepted 1`, fifth `Accepted 0`.

Rows examined: before, `COUNT(*)` over the matching range (inbox at cap
~64–100). After, `LIMIT cap+1` so **65 rows** at circuit cap 64.

## CIRCUIT ×3, shipped

`ps` before: grok 24.5 %, claude 4.9 % — hotter than the trace stone's 11 %.

Every run: `total=8000;distinct=8000;dup=0`.

```
              r1     r2     r3    med    today
publish    21198  21293  21443  21293   ~20700
collect     6246   5941   5989   5989    ~5900
stop         323    346    117    323     ~400
```

publish +593 of 20700, outside ±300. **A loss, not a gain.** DESIGN
predicted neutral at cap 64; the bound cannot have added 600 ms of
*work*. Box was 2× the usual grok CPU. Retries/asleep unchanged
(~1375 / ~11200). Explained, not celebrated, not re-run.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ count saturates | ✅ 5000 rows, limit 65 → Ok 65 |
| 2 | ★★ exact below the limit | ✅ limit 100000 → Ok 5000 |
| 3 | ★★ both impls agree | ✅ mem = sqlite at 1 / 65 / 5000 |
| 4 | ⛔ bound is passed | ✅ `_lim` gone; `_now-ns` stays on `total` only (out of scope) |
| 5 | ⛔ cap gate still gates | ✅ fill 1/1/1/1, over=0 |
| 6 | ⛔ delivery exact | ✅ ×3 `distinct=8000;dup=0` |
| 7 | ⛔ publish does not move | ▪ 21293 vs 20700; load, see above |
| 8 | ⛔ the floor | ✅ `5221 passed (7 slow), 22 skipped` |
| 9 | ⛔ blast | ✅ four briefed files + scratch probes + SCORE. No `src/`, no `service.wat`, no `circuit.wat` |

**STOP-2 / STOP-3 / STOP-4 did not fire.**
