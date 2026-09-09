# SCORE — the store reports time per operation

**SCORED.** Executor: claude, 2026-09-08. Did not commit. **No STOP fired.** Two files modified:
`wat-scripts/queue/sqs.wat`, `wat-scripts/fanout/circuit.wat`.

```
     Summary [ 478.541s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-09T04-14-17Z/` — `scripts/floor.sh` exit `0`, **no `ARM.txt`**, zero
`FAIL`/`TRY`/`TIMEOUT`/`ABORT`/`SIGSEGV` lines in `clean.log`. **5237** — unchanged (no test added, no
test removed). `wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime`
**PASS [478.534s]**. Run **after** all nine sweep runs, never beside one.

---

## ⭑ THE ANSWER

**`scan-index` per-call latency does not grow. `delete`, `put` and `count-index` do — and only in the
tiers whose table actually grows.**

Measured per-call, four subscriber tiers summed (each queue's table grows to `n` live rows):

| op | n=1000 | n=2000 | n=4000 | `n2000/n1000` band | `n4000/n2000` band | `4000/1000` |
|---|---|---|---|---|---|---|
| **`delete`** | 2160.3 µs | 2947.2 µs | 4247.8 µs | **1.364** [1.303–1.412] | **1.441** [1.344–1.491] | **1.966** |
| **`put`** | 1453.1 µs | 1719.6 µs | 2200.0 µs | **1.183** [1.161–1.212] | **1.279** [1.245–1.299] | **1.514** |
| **`count-index`** | 750.4 µs | 824.9 µs | 951.2 µs | **1.099** [1.081–1.118] | **1.153** [1.141–1.166] | **1.268** |
| `scan-index` | 1414.8 µs | 1418.8 µs | 1258.8 µs | 1.003 [0.938–1.085] | **0.887** [0.832–0.930] | 0.890 |

★★ **And the measurement contains its own control.** The **inbox** queue is started with `:cap 64`
(`circuit.wat:2178`), so its table never holds more than ~64 live rows however large `n` is. At the
inbox **all four ops are FLAT** — every band brackets 1.0 or sits inside ±3 %:

| op | n=1000 | n=2000 | n=4000 | `n2000/n1000` band | `n4000/n2000` band | `4000/1000` |
|---|---|---|---|---|---|---|
| `put` | 2128.0 µs | 2105.7 µs | 2097.8 µs | 0.990 [0.967–1.020] | 0.996 [0.991–1.011] | **0.986** |
| `delete` | 679.2 µs | 688.4 µs | 693.6 µs | 1.014 [0.993–1.015] | 1.008 [1.007–1.025] | **1.021** |
| `count-index` | 628.7 µs | 645.9 µs | 630.8 µs | 1.027 [0.903–1.054] | 0.977 [0.948–1.043] | **1.003** |
| `scan-index` | 755.4 µs | 686.2 µs | 718.4 µs | 0.908 [0.891–0.941] | 1.047 [0.965–1.125] | **0.951** |

The subscriber tiers run cap `8192`, so at n=4000 they hold up to 4000 live rows; the inbox holds 64.
Same code, same store implementation (`sqlite-store`, `:memory:`, one dedicated store **process per
queue** — `circuit.wat:2154` and `:2172`, so no cross-tier contention). The one thing that differs
between the flat table and the growing table is **how many rows the table holds**.

⚠ **What this does NOT establish.** It names operations, **not a mechanism.** The instrument is
caller-side: each `ns` brackets a full request→reply round trip to the store process, so it cannot
separate work inside SQLite from serialisation of a larger reply or from scheduling. Naming *why*
`delete` doubles per call needs the server-side dive the DESIGN deferred. **No claim is made here
about `mem.wat` (unused — every queue is `sqlite-store`) or about a missing index.**

★ **A batch-size artefact is excluded, not assumed.** Per-call growth would be fake if the batch per
call grew. It does not: at the subscriber tiers `put-calls` scales **1.999 / 2.002** and
`delete-calls` **1.961 / 2.000** against message counts that scale exactly 2.000 — so rows-per-`put`
and ids-per-`delete` are constant across the sweep by construction. `count-index` takes no batch.

---

## ⭑ ROW 2 — THE SPLIT RECONCILES EXACTLY, NOT "WITHIN ROUNDING"

`put + delete + count + scan` against the `store-calls` / `store-ns` aggregate it subdivides:

- **45 of 45 tier-reports** (5 tiers × 9 runs): **remainder 0 calls and 0 nanoseconds.**
- **9 of 9 whole-run sums**: remainder **0 / 0**.

There is **no unaccounted operation.** `ensure-schema` is the one op deliberately outside the split
and it is outside the *aggregate too*: it is called once in `:init` (`sqs.wat:179`), before
`store-calls`/`store-ns` are initialised to `0` in the same constructor — so it appears in neither
number, and its absence is why the two agree exactly rather than approximately.

| n | run | put c/ns | delete c/ns | count c/ns | scan c/ns | store (agg) c/ns | split − agg |
|---|---|---|---|---|---|---|---|
| 1000 | 1 | 2609/4177699849 | 2062/2022772629 | 2862/2051233545 | 2937/2735260421 | 10470/10986966444 | **0 / 0** |
| 1000 | 2 | 2603/4146062832 | 2060/2019828823 | 2847/2083293663 | 3808/3283133766 | 11318/11532319084 | **0 / 0** |
| 1000 | 3 | 2604/4094402236 | 2056/1964925692 | 2867/2028278864 | 2882/2729841560 | 10409/10817448352 | **0 / 0** |
| 2000 | 1 | 5200/9389274763 | 4099/4622899717 | 5576/4316514916 | 6017/5038985753 | 20892/23367675149 | **0 / 0** |
| 2000 | 2 | 5207/9276334581 | 4112/4591639632 | 5582/4225595152 | 6517/5288940737 | 21418/23382510102 | **0 / 0** |
| 2000 | 3 | 5190/9352893568 | 4094/4625749463 | 5621/4374547975 | 6866/5588294542 | 21771/23941485548 | **0 / 0** |
| 4000 | 1 | 10401/22643261003 | 8199/11373006666 | 11235/9610146010 | 12064/9826865572 | 41899/53453279251 | **0 / 0** |
| 4000 | 2 | 10418/22266604030 | 8207/10932050638 | 11312/9694837716 | 6315/5997175589 | 36252/48890667973 | **0 / 0** |
| 4000 | 3 | 10415/22783943596 | 8206/11468253224 | 11243/9657329635 | 13502/10256418955 | 43366/54165945410 | **0 / 0** |

⚠ These `store-calls` are **larger than the phases line's** `store-calls` (e.g. 10470 vs 1325 at
n=1000) for two reasons, both pre-existing: the phases line's `sum-store-calls` folds **the four
subscriber `qclients` only** (not the inbox), and it is read **before** the several later `stats`
rounds, each of which itself costs 2 `count-index`. The tier line is read last. Every number in this
SCORE comes from the tier line and is internally consistent.

---

## THE COMMAND LINES, QUOTED

```
./target/release/wat wat-scripts/fanout/circuit.wat 1000 4 3 8192 true 1000
./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true 1000
./target/release/wat wat-scripts/fanout/circuit.wat 4000 4 3 8192 true 1000
```

`m=4 j=3 sub-cap=8192 fill-first?=true`, **`vis-ms=1000` pinned at all three points**; only `n` varies.
The three points were **interleaved** — `1000, 2000, 4000` × three rounds — so no `n` owns a
contiguous block of the box's history. Binary: `target/release/wat` at HEAD `a84ae20a0`; no `src/`
change was made, so the same binary that produced `docs/excursus/2026/08/001-sns-sqs/the-slope-belongs-to-a-phase/SCORE.md` produced this.

---

## THE SWEEP — nine runs, none merged, none dropped

All nine `rc=0`, **zero bytes on stderr** on every run. Times in **ms** as the harness prints them;
`wall` is `date +%s%N` around the process. `load` is `cut -d' ' -f1 /proc/loadavg` immediately before
the run.

| n | run | load | wall | `setup` | `fill` | `arm` | `drain` | `collect` | `stop` | `total` | `distinct` | `dup` | `workers` | `fill-depth` |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1000 | 1 | 0.08 | 29333 | 12497 | 7889 | 7 | 1358 | 4879 | 475 | 27107 | **4000** | **0** | 10 | [1000/0]×4 |
| 2000 | 1 | 0.53 | 38993 | 12512 | 15782 | 7 | 3178 | 4365 | 128 | 35974 | **8000** | **0** | 8 | [2000/0]×4 |
| 4000 | 1 | 1.30 | 65782 | 12557 | 32128 | 7 | 7590 | 8211 | 802 | 61297 | **16000** | **0** | 8 | [4000/0]×4 |
| 1000 | 2 | 2.16 | 29288 | 12603 | 7668 | 7 | 1409 | 4824 | 468 | 26982 | **4000** | **0** | 10 | [1000/0]×4 |
| 2000 | 2 | 1.37 | 39371 | 12566 | 15616 | 7 | 3143 | 4614 | 414 | 36363 | **8000** | **0** | 10 | [2000/0]×4 |
| 4000 | 2 | 1.68 | 63839 | 12553 | 31719 | 7 | 7164 | 7522 | 343 | 59311 | **16000** | **0** | 8 | [4000/0]×4 |
| 1000 | 3 | 2.19 | 29381 | 12600 | 7793 | 7 | 1338 | 5162 | 220 | 27122 | **4000** | **0** | 9 | [1000/0]×4 |
| 2000 | 3 | 1.74 | 40037 | 12513 | 15748 | 7 | 3196 | 5113 | 436 | 37016 | **8000** | **0** | 8 | [2000/0]×4 |
| 4000 | 3 | 2.27 | 65149 | 12584 | 32772 | 10 | 7594 | 7314 | 393 | 60669 | **16000** | **0** | 8 | [4000/0]×4 |

`distinct = n×m` and `dup = 0` on **all nine**. `empty=1`, `seen-recorded = n×m`, `check-exhausted=0`,
`mark-exhausted=0`, `visible=0`, `unacked=0` on all nine. `workers` 8–10 (the arc's band).
`ack-retries` 0–8, `ack-exhausted` 0 except n=1000 r1/r2/r3 (=1) and n=2000 r2 (=2).
`seen-skipped` was 0 on six runs and 10/10/20 on n=1000 r1, n=1000 r2 and n=2000 r2 — the same
upstream visibility-timing variability the previous three SCOREs recorded; reported, not treated as a
regression. `fill-depth` is `[n/0]` on every subscriber on every run, confirming the subscriber tables
really do reach `n` live rows.

### The box

`ps -eo args | grep -E 'cargo|nextest|release/wat' | grep -v grep` was run **before every one of the
nine runs** and every time showed exactly two lines, both idle MCP servers:

```
/home/john/.cargo/bin/wat --mcp
/home/john/.cargo/bin/wat --mcp
```

**No run ran beside another** (a single sequential driver, 10 s between runs), and **the floor ran
after all nine.** Start loads were **0.08–2.27**, lower and tighter than the previous SCORE's
0.22–5.19, and the interleaved ordering means the load drift (low in round 1, ~2 in round 3) is spread
across all three `n` rather than correlated with one.

---

## THE PREVIOUS FINDING REPRODUCES ON THIS BUILD

Before trusting the split, the handover it subdivides was re-measured on the instrumented binary:

| counter | n=1000 med [min–max] | n=2000 | n=4000 | `r21` | `r42` |
|---|---|---|---|---|---|
| `drain-store-calls` | 377 [376–378] | 752 [748–756] | 1535 [1530–1553] | **1.995** | **2.041** |
| `drain-store-ms` | 707 [685–726] | 1793 [1776–1798] | 4726 [4451–4736] | **2.536** | **2.636** |
| `drain-busy-ms` | 818 [796–834] | 2051 [2030–2053] | 5252 [4955–5271] | 2.507 | 2.561 |
| mean drain per-call | **1.875 ms** | **2.372 ms** | **3.085 ms** | | |

Linear calls, superlinear time — `1b2c65034`'s 2.024/2.037 vs 2.513/2.616 against 1.995/2.041 vs
2.536/2.636 here. **The premise held.**

---

## ⚠ ROW 4 — THE INSTRUMENT IS NOT FREE, AND HERE IS ITS PRICE

Row 4 asked for wall clock and phase times against the n=2000 band, because the axis the previous
stone got wrong was *time*, not syscalls. **It passes as written and it fails the claim behind it.**
Medians against `docs/excursus/2026/08/001-sns-sqs/the-slope-belongs-to-a-phase/SCORE.md`, same binary, same box:

| phase | mine, n=1000 / 2000 / 4000 | baseline, n=1000 / 2000 / 4000 | delta |
|---|---|---|---|
| wall | 29333 / **39371** / 65149 | 28354 / 38833 / 63614 | **+3.5 % / +1.4 % / +2.4 %** |
| `setup` | 12600 / 12513 / 12557 | 12422 / 12395 / 12434 | +1.4 % / +1.0 % / +1.0 % |
| `fill` | 7793 / **15748** / 32128 | 7694 / 15506 / 31335 | +1.3 % / +1.6 % / +2.5 % |
| **`drain`** | 1358 / 3178 / 7590 | 1246 / 2866 / 6843 | **+9.0 % / +10.9 % / +10.9 %** |
| `collect` | 4879 / 4614 / 7522 | 4594 / 4619 / 7934 | +6.2 % / −0.1 % / −5.2 % |
| `total` | 27107 / 36363 / 60669 | 26132 / 35828 / 59145 | +3.7 % / +1.5 % / +2.6 % |

- **On the letter, row 4 passes.** n=2000 wall median **39371 ms** is inside the 39–40 s band and
  n=2000 `fill` **15616–15782 ms** is inside 15.5–17.3 s. (⚠ Honest edge: n=2000 r3 came in at
  **40037 ms** — 37 ms, 0.09 %, above the band's top.)
- **On the claim, it does not.** `drain` is **+9 to +11 % at all three n and the spreads do not
  overlap the baseline's** (1338–1409 vs 1225–1266; 3143–3196 vs 2803–2900; 7164–7594 vs 6817–6859).
  The instrument added **no store call and no clock read** — every `ns` value it banks is one that
  `store-ns` already computed. What it added is **allocation**: `:queue::queue::State` is now 8 fields
  wider and is rebuilt 2–3× per handler invocation; `TakeAcc` is 4 wider and is rebuilt per waiter per
  fold; `take`'s third slot is now a nested `Tuple` rather than an `i64`; `:queue::Stats` is 8 wider.
  **This is the same class of error EXPECTATIONS warned about — "clone-cost is allocation, not
  syscalls" — arriving on the axis row 4 was built to watch. It is reported, not filed under variance.**
- ★ **It does not touch the finding.** The cost is a near-constant **~+10 % multiplier on `drain` at
  every n**, so it cancels in every ratio: `drain` here is r21 **2.340**, r42 **2.388** against the
  baseline's 2.301 / 2.388. A multiplier moves levels, not slopes.

---

## MEDIANS AND OBSERVED MIN–MAX — the full per-op tables

Median over the three runs at that `n`; `[min–max]` is the observed extreme pair. Ratio bands are the
worst case the extremes admit, `min(hi n)/max(lo n)` to `max(hi n)/min(lo n)`. **An op grows only if
its whole band clears 1.0.**

### A. `ns/calls` — WHOLE RUN, all five tiers summed (µs per call)

| op | n=1000 [min–max] | n=2000 [min–max] | n=4000 [min–max] | `n2000/n1000` band | `n4000/n2000` band | `4000/1000` |
|---|---|---|---|---|---|---|
| `put` | **1592.8** [1572.4–1601.3] | **1802.1** [1781.5–1805.6] | **2177.0** [2137.3–2187.6] | **1.131** [1.113–1.148] | **1.208** [1.184–1.228] | 1.367 |
| `delete` | **980.5** [955.7–981.0] | **1127.8** [1116.6–1129.9] | **1387.1** [1332.0–1397.5] | **1.150** [1.138–1.182] | **1.230** [1.179–1.252] | 1.415 |
| `count-index` | **716.7** [707.5–731.8] | **774.1** [757.0–778.3] | **857.0** [855.4–859.0] | **1.080** [1.035–1.100] | **1.107** [1.099–1.135] | 1.196 |
| `scan-index` | **931.3** [862.2–947.2] | **813.9** [811.6–837.5] | **814.6** [759.6–949.7] | 0.874 [0.857–0.971] | 1.001 [0.907–1.170] | 0.875 |
| **store** (agg) | **1039.2** [1018.9–1049.4] | **1099.7** [1091.7–1118.5] | **1275.8** [1249.0–1348.6] | 1.058 [1.040–1.098] | **1.160** [1.117–1.235] | 1.228 |

### B. `ns/calls` — INBOX only, `cap = 64` (the control)

| op | n=1000 [min–max] | n=2000 [min–max] | n=4000 [min–max] | `n2000/n1000` band | `n4000/n2000` band | `4000/1000` |
|---|---|---|---|---|---|---|
| `put` | **2128.0** [2068.8–2151.0] | **2105.7** [2080.1–2110.9] | **2097.8** [2091.5–2102.5] | 0.990 [0.967–1.020] | 0.996 [0.991–1.011] | 0.986 |
| `delete` | **679.2** [678.1–690.1] | **688.4** [685.5–688.5] | **693.6** [693.5–702.5] | 1.014 [0.993–1.015] | 1.008 [1.007–1.025] | 1.021 |
| `count-index` | **628.7** [615.3–670.9] | **645.9** [605.8–648.4] | **630.8** [614.4–631.7] | 1.027 [0.903–1.054] | 0.977 [0.948–1.043] | 1.003 |
| `scan-index` | **755.4** [745.0–763.4] | **686.2** [680.2–701.1] | **718.4** [676.8–765.3] | 0.908 [0.891–0.941] | 1.047 [0.965–1.125] | 0.951 |
| **store** (agg) | **856.3** [838.0–867.8] | **815.5** [808.3–830.0] | **833.3** [811.9–890.7] | 0.952 [0.931–0.990] | 1.022 [0.978–1.102] | 0.973 |

### C. `ns/calls` — FOUR SUBSCRIBER tiers summed, `cap = 8192`

| op | n=1000 [min–max] | n=2000 [min–max] | n=4000 [min–max] | `n2000/n1000` band | `n4000/n2000` band | `4000/1000` |
|---|---|---|---|---|---|---|
| **`delete`** | **2160.3** [2090.9–2196.9] | **2947.2** [2862.5–2951.9] | **4247.8** [3968.5–4267.1] | **1.364** [1.303–1.412] | **1.441** [1.344–1.491] | **1.966** |
| **`put`** | **1453.1** [1422.9–1465.0] | **1719.6** [1701.4–1725.2] | **2200.0** [2148.0–2210.5] | **1.183** [1.161–1.212] | **1.279** [1.245–1.299] | **1.514** |
| **`count-index`** | **750.4** [742.6–755.0] | **824.9** [816.5–829.9] | **951.2** [946.9–952.2] | **1.099** [1.081–1.118] | **1.153** [1.141–1.166] | **1.268** |
| `scan-index` | **1414.8** [1336.9–1510.4] | **1418.8** [1416.1–1450.4] | **1258.8** [1206.6–1316.4] | 1.003 [0.938–1.085] | 0.887 [0.832–0.930] | 0.890 |
| **store** (agg) | **1226.3** [1218.0–1227.2] | **1427.0** [1410.5–1433.3] | **1761.9** [1720.3–1763.3] | **1.164** [1.149–1.177] | **1.235** [1.200–1.250] | 1.437 |

### D. Calls and total ns behind those ratios (medians [min–max])

**Whole run, five tiers summed**

| op | calls n=1000 / 2000 / 4000 | calls `r21`/`r42` | ns n=1000 / 2000 / 4000 | ns `r21`/`r42` |
|---|---|---|---|---|
| `put` | 2604 / 5200 / 10415 | 1.997 / 2.003 | 4146 / 9353 / 22643 ms | **2.256 / 2.421** |
| `delete` | 2060 / 4099 / 8206 | 1.990 / 2.002 | 2020 / 4623 / 11373 ms | **2.289 / 2.460** |
| `count-index` | 2862 / 5582 / 11243 | 1.950 / 2.014 | 2051 / 4317 / 9657 ms | **2.104 / 2.237** |
| `scan-index` | 2937 / 6517 / 12064 | 2.219 / 1.851 | 2735 / 5289 / 9827 ms | 1.934 / 1.858 |
| **store** | 10470 / 21418 / 41899 | 2.046 / 1.956 | 10987 / 23383 / 53453 ms | 2.128 / 2.286 |

**Four subscriber tiers summed**

| op | calls n=1000 / 2000 / 4000 | calls `r21`/`r42` | ns n=1000 / 2000 / 4000 | ns `r21`/`r42` |
|---|---|---|---|---|
| `put` | 2052 / 4101 / 8209 | 1.999 / 2.002 | 2986 / 7044 / 18044 ms | **2.359 / 2.562** |
| `delete` | 408 / 800 / 1600 | 1.961 / 2.000 | 881 / 2358 / 6796 ms | **2.675 / 2.883** |
| `count-index` | 2070 / 4006 / 8015 | 1.935 / 2.001 | 1553 / 3295 / 7632 ms | **2.122 / 2.316** |
| `scan-index` | 754 / 1159 / 2113 | 1.537 / 1.823 | 1071 / 1644 / 2701 ms | 1.535 / 1.643 |
| **store** | 5274 / 10080 / 19963 | 1.911 / 1.980 | 6467 / 14320 / 35135 ms | 2.214 / 2.454 |

**Inbox only**

| op | calls n=1000 / 2000 / 4000 | calls `r21`/`r42` | ns n=1000 / 2000 / 4000 | ns `r21`/`r42` |
|---|---|---|---|---|
| `put` | 552 / 1099 / 2206 | 1.991 / 2.007 | 1175 / 2309 / 4630 ms | 1.966 / 2.005 |
| `delete` | 1652 / 3299 / 6606 | 1.997 / 2.002 | 1121 / 2268 / 4582 ms | 2.023 / 2.020 |
| `count-index` | 791 / 1581 / 3225 | 1.999 / 2.040 | 498 / 1021 / 2037 ms | 2.051 / 1.995 |
| `scan-index` | 2180 / 5358 / 9918 | 2.458 / 1.851 | 1664 / 3645 / 7126 ms | 2.190 / 1.955 |
| **store** | 5180 / 11338 / 21936 | 2.189 / 1.935 | 4495 / 9165 / 18280 ms | 2.039 / 1.995 |

### ⚠ The one op whose spread forbids a ratio: `scan-index`'s CALL COUNT

`scan-calls` is the only counter in the sweep with a spread comparable to its own median difference.
Whole run: **2882–3808 at n=1000** (a 926-call spread, 32 % of the median) and **6315–13502 at
n=4000** (7187, 60 %). Its bands are therefore **0.857–0.971** and **0.907–1.170** on per-call latency
and **1.580–2.382** / **0.920–2.244** on call count — the latter admits no slope statement at all and
is printed for completeness only. Every other op's per-call spread is under 3 % of its median.

★★ **This is itself the arc's answer to a puzzle in the previous SCORE.** The whole-run aggregate
`store-calls` looked noisy there (36252–43366 at n=4000, a 20 % spread) and it is now visible **which
of the five ops carries all of that noise**: `put`, `delete` and `count-index` call counts are
reproducible to **±0.3 %**, while `scan-index` alone swings 2×. `scan-index` is the op called once per
`take`, and `take` is driven by how many times a worker happened to poll — so the aggregate's
variance was never the store's; it was the poller's.

---

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ four op pairs, distinct | ✅ `put-calls/put-ns`, `delete-calls/delete-ns`, `count-calls/count-ns`, `scan-calls/scan-ns` on `:ephemeral` (`sqs.wat:170-177`) and in `:queue::Stats`. **Never summed** — eight independent `i64` accumulators, each incremented only at its own call site. **`store-ns` is not reused**: the split is accumulated separately and then *compared* to the aggregate |
| 2 | ★★ the split reconciles | ✅ **exactly, not within rounding.** Remainder **0 calls / 0 ns** on **45 of 45** tier reports and **9 of 9** whole-run sums. No unaccounted operation. `ensure-schema` is outside both numbers (`:init` only) and that is *why* the agreement is exact |
| 3 | ★ per-op mean latency at all three n | ✅ nine runs, three per `n`, `vis-ms=1000` pinned. `ns/calls` per op at each `n` with medians, observed min–max, and both ratios with worst-case bands — three ways (whole run / inbox / subscribers). **`delete` 1.966×, `put` 1.514×, `count-index` 1.268× over `4000/1000` at the growing tiers; `scan-index` 0.890× — it does not grow** |
| 4 | ★ the instrument is free | ⚠ **PASSES the letter, FAILS the claim — reported in full above.** n=2000 wall median **39371 ms** inside 39–40 s (one run at 40037, +0.09 % over the top edge) and n=2000 `fill` 15616–15782 inside 15.5–17.3 s. **But `drain` is +9.0 / +10.9 / +10.9 % with non-overlapping spreads**, and wall clock +1.4 to +3.5 %. Cause: **allocation** (State 8 fields wider, TakeAcc 4 wider, `take`'s tuple nested, Stats 8 wider) — no added store call, no added clock read. It is a **multiplier, so every ratio is unaffected** (`drain` r21 2.340 / r42 2.388 vs the baseline's 2.301 / 2.388) |
| 5 | counters live on `:ephemeral` | ✅ all eight on `:ephemeral`. **`:queue::queue::Record` is untouched** — the only `Record` text in the diff is the pre-existing accessor `Record/store-addr` on a one-line arm changed for the `store-ns` split; the `:durable` declaration and every `Record` constructor are byte-identical. `git diff` carries no `Record` field change |
| 6 | correctness at every point | ✅ `distinct = 4000 / 8000 / 16000 = n×m` and `dup = 0` on **all nine**; `seen-recorded = n×m`, `empty=1`, `visible=0`, `unacked=0`, `check-exhausted=0`, `mark-exhausted=0` everywhere. `rc=0` and zero stderr bytes on all nine. No run excluded |
| 7 | blast radius | ✅ `wat-scripts/queue/sqs.wat` (+351 lines changed) and `wat-scripts/fanout/circuit.wat` (+16). **Zero files under `wat/` or `src/`.** `git status --porcelain` shows exactly those two `M` lines |
| 8 | the box was quiet | ✅ the `ps` gate ran before **every one of the nine runs** and showed only the two idle `wat --mcp` servers each time; one run at a time; load stated per run (0.08–2.27); the three points **interleaved** so no `n` owns a block of box history; **floor after all nine, never beside one** |
| 9 | the floor holds | ✅ `Summary [ 478.541s] 5237 tests run: 5237 passed (7 slow), 22 skipped` — **0 FAIL, 0 TIMEOUT**, exit `0`, no `ARM.txt`, zero `FAIL`/`TRY`/`TIMEOUT`/`ABORT`/`SIGSEGV` lines in `clean.log`. The `wat-scripts` load gate **PASS [478.534s]** |

---

## STOPS

**None fired.** Each was live and each is answered with its numbers.

- **STOP-1** (the pairs do not reconcile with `store-ns` → stop and report the gap) — **did not fire.**
  Remainder is **0 calls and 0 nanoseconds** on 45/45 tier reports and 9/9 runs. There is no gap to
  report, and the reason it is exact rather than approximate is stated: `ensure-schema` sits outside
  both numbers, so no unattributed op remains inside the aggregate.
- **STOP-2** (a counter needs a store call, a new clock read, or new state beyond the eight fields) —
  **did not fire on cost; here is the plumbing it did require, stated rather than glossed.** No store
  call and **no clock read** was added: every `ns` banked is a value `store-ns` already computed at the
  same bracket. Beyond the eight `:ephemeral` fields, two *carriers* widened, because the numbers have
  to leave a closure and a fold: `:queue::TakeAcc` gained `scan-calls/scan-ns/put-calls/put-ns`
  (transport out of the waiter fold, `calls`/`ns` kept beside them so the two can be reconciled), and
  `take`'s return tuple's third slot became a nested `(scan-ns put-ns)` pair — nested rather than
  widened because **there is no fourth Tuple accessor** (`wat/core.wat:1737`). Neither is new state:
  both are return values, neither is on `:ephemeral`, neither persists past the call. ⚠ The cost that
  *did* materialise is row 4's allocation, and it is reported there in full rather than here.
- **STOP-3** (do not touch anything under `wat/`) — **held.** `git diff --name-only` matches nothing
  under `wat/` or `src/`. `wat/query/sqlite-store.wat` and `wat/query/mem.wat` are untouched.
- **STOP-4** (do not change admission, visibility, ack semantics, the cap, or `setup`) — **held.** The
  `cap`/`room`/`take` admission arithmetic, `vis-ns` handling, the ack path's delete-then-reply order
  and `:init` are unchanged apart from banking eight additional integers. `setup` is 12513–12603 ms at
  all three `n` (constant, as the previous SCORE established) and `distinct=n×m, dup=0` on all nine
  runs is the behavioural proof.
- **STOP-5** (no ratio without its spread; no culprit the numbers do not support) — **honoured, and it
  bites.** Every ratio in this SCORE carries its observed min–max **and** the worst-case band the
  extremes admit. ⚠ **`scan-index`'s call count admits no slope**: 2882–3808 at n=1000 against a
  median difference of 3580, band **1.580–2.382**; it is printed for completeness and must not be read
  as a rate. On the culprit: three ops are named with bands that clear 1.0 entirely, one is named as
  **flat**, and **no mechanism is claimed**. The caller-side instrument cannot separate SQLite work
  from round-trip cost, and this SCORE does not pretend it can.
- **STOP-6** (red floor arm) — **no red arm.** The floor ran **once**, exit 0, no `ARM.txt`. Nothing
  was re-run.

---

## EDIT SITES — 63, against a sketch that named 2

The BRIEF predicted more sites than the sketch shows and asked for the real count. **63 sites in two
files** (27 `Edit` calls; 36 of the sites were folded into six `replace_all` groups whose replacement
text is identical by construction — the safest way to touch 30 near-duplicate constructors):

| what | sites |
|---|---|
| `:queue::queue::State` constructors threading the eight fields | **30** |
| `:queue::TakeAcc` constructors threading the four transport fields (3 fold seeds + 9 in-fold arms) | **12** |
| local-binding blocks (`scan-only`, `both`, `cc0`/`cn0`, `receive`'s split, 3× fold accessors, 3× fold `take-split`) | **10** |
| declarations (`TakeAcc` defstruct, `Stats` defrecord, `:ephemeral` `take` type, `:ephemeral` eight fields, `take`'s return type) | **5** |
| `take`'s return sites (1 empty-scan + 3 `dial-store` arms) | **4** |
| `:queue::Stats` construction in the `stats` arm | **1** |
| `:fanout::tier-line` in `circuit.wat` | **1** |

★ A structural cross-check that no constructor was missed: `:put-calls` and `:scan-ns` each occur
**exactly 43 times** in `sqs.wat` = 30 State + 12 TakeAcc + 1 Stats. The type checker is the real
gate — a dropped field is a type error — and `./target/release/wat wat-scripts/queue/sqs.wat` returns
its differential self-test result `"bound=x;r1=a,b;r2=c;r3=;redel=b"`, so mem-store and sqlite-store
still agree.

⚠ **Trap-door checked, not assumed.** EXPECTATIONS warned that threading a *wrong* value (the
pre-increment one) silently under-counts and is not a type error. The exact-zero reconciliation on
45/45 tier reports is the check that would have caught it: any site banking a stale accumulator would
leave a non-zero remainder against `store-ns`.

---

## WHAT THE NEXT STONE INHERITS

1. **`scan-index` is exonerated on per-call latency** at every tier and both doublings. A server-side
   dive should not start there.
2. **`delete` is the steepest** — 1.97× per call over 4× `n` at the growing tiers, nearly linear in
   row count. `put` 1.51×, `count-index` 1.27×.
3. **The inbox at `cap 64` is flat on all four ops** — the same code, the same store implementation, a
   table that does not grow. Row count, not call count, is the axis that moves per-call time.
4. ⚠ **This names operations, not mechanisms.** The instrument brackets a round trip; it cannot say
   whether the growth is inside SQLite, in reply serialisation, or in scheduling. That is the
   server-side per-query instrument the DESIGN deferred, and it now has **one op** to point at.
5. ⚠ **The instrument costs ~10 % of `drain`.** Anything measuring `drain` levels against
   `1b2c65034` must either revert these counters or subtract the multiplier. Slopes are safe.

---

## BLAST RADIUS

```
$ git status --porcelain
 M wat-scripts/fanout/circuit.wat
 M wat-scripts/queue/sqs.wat

$ git diff --stat
 wat-scripts/fanout/circuit.wat |  16 +-
 wat-scripts/queue/sqs.wat      | 351 ++++++++++++++++++++++++++++++++++-------
 2 files changed, 310 insertions(+), 57 deletions(-)

$ git diff --name-only | grep -E '^(wat|src)/' | wc -l
0
```

**Two files, no `wat/`, no `src/`, no `Record`.** Left uncommitted.
