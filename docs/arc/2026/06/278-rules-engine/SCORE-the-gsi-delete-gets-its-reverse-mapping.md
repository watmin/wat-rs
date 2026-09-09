# SCORE — the GSI delete gets its reverse mapping

**SCORED.** Executor: claude, 2026-09-08. Did not commit. **One STOP fired: STOP-3, and it is named
below — its stated mechanism is REFUTED, not confirmed.** One file modified:
`wat/query/sqlite-store.wat` (`ensure-index-tables` only), plus two probes under
`wat-scripts/scratch-pad/`.

```
     Summary [ 486.358s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-09T05-39-07Z/` — `scripts/floor.sh` exit `0`, **no `ARM.txt`**, zero
`FAIL`/`TRY`/`TIMEOUT`/`ABORT`/`SIGSEGV` lines in `clean.log`. **5237** — unchanged (no test added, no
test removed). `wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime`
**PASS [486.351s]** with both new probes in the tree. Run **after** all eighteen sweep runs, never
beside one.

---

## ⭑ THE ANSWER — LAND IT. And the cost the stone braced for is not there; the opposite is.

**`delete` flattens, `drain` drops 24 % at n=4000, and `put` gets FASTER rather than paying an index
tax** — because `put` is clear-then-insert and its clear step (`clear-index-projections`) issues the
*same* `DELETE … WHERE pk=? AND sk=?` the stone was written about. Four subscriber tiers,
`4000/1000` per-call, medians with worst-case bands:

| op | baseline `4000/1000` (mine, this box) | after | absolute at n=4000 |
|---|---|---|---|
| **`delete`** | **1.998** [1.909–2.091] | **1.107** [1.087–1.137] | 4227.7 → **1842.9 µs** (**2.29× faster**) |
| **`put`** | **1.526** [1.517–1.586] | **1.054** [1.039–1.058] | 2206.1 → **1338.7 µs** (**1.65× faster**) |
| `count-index` | 1.286 [1.277–1.307] | 1.267 [1.250–1.286] | 958.2 → 969.2 µs (+1.1 %) |
| ⚠ `scan-index` | 0.830 [0.799–0.908] | **1.266** [1.209–1.346] | 1257.1 → 1365.3 µs (+8.6 %) |
| **store** (agg) | 1.433 [1.423–1.470] | **1.147** [1.131–1.168] | 1761.1 → 1237.0 µs |

★★ **`put`'s 1.514× was not "a separate story".** The DESIGN and EXPECTATIONS both put `put`'s growth
out of scope — *"plausibly b-tree depth or the per-call transaction"* — and braced for the added index
to make it worse. It is largely **the same defect**: `put-one-row` (`sqlite-store.wat:293`, clearing at `:307`) calls
`clear-index-projections` on every row, and that is the scanning DELETE. Give it the mapping and
`put`'s slope collapses from 1.526 to 1.054 and its absolute per-call time drops 39 % at n=4000.
**No prediction was offered and none is claimed retroactively — but the stone's own cost model is
what the numbers overturned.**

★ **The index tax is real and it is ~1 %.** It is only visible where the mapping cannot help: the
**inbox**, capped at `:cap 64` live rows, whose GSI table never grows so the scan was already cheap.
There `put` goes **+0.3 % / +1.9 % / +1.0 %** (n=1000/2000/4000) and only the n=2000 point separates
from noise. That is the whole measured cost of the third b-tree.

---

## ⭑⭑ ROWS 1 AND 2 — THE ORPHAN PROBE, WITH A CONTROL THAT CAN FAIL

`wat-scripts/scratch-pad/probe-no-orphan-gsi-rows-after-delete.wat`, file-backed store so a **second
SQLite connection reads the very tables the service wrote**. Two GSIs declared on purpose
(`index_<name>_by_key` must not collide, and under `IF NOT EXISTS` a collision would be **silent**).
The row-1 gate is a direct query, never an inference:

```sql
SELECT COUNT(*) FROM [index_X] i LEFT JOIN main m ON i.pk = m.pk AND i.sk = m.sk WHERE m.pk IS NULL
```

**AFTER the change** (`target/release/wat` at the working tree):

```
ix-by-key=[1/1]
p1=main=200,vis[rows=200,orphans=0,missing=0,dups=0],own[rows=200,orphans=0,missing=0,dups=0],depth=200/200
p2=main=120,vis[rows=120,orphans=0,missing=0,dups=0],own[rows=120,orphans=0,missing=0,dups=0],depth=120/120
neg-control=1/0
p4=main=120,vis[rows=120,orphans=0,missing=0,dups=0],own[rows=120,orphans=0,missing=0,dups=0],depth-idx=80,depth-idx2=40,depth-own=120
p5=main=0,vis[rows=0,orphans=0,missing=0,dups=0],own[rows=0,orphans=0,missing=0,dups=0],depth=0/0/0
VERDICT=NO-ORPHANS
```

**BEFORE the change** — the same probe on a binary built with the change `git stash`ed out, minutes
earlier, so the invariants are shown to hold *both* ways and the change is proven additive on
correctness:

```
ix-by-key=[0/0]
p1=main=200,vis[rows=200,orphans=0,missing=0,dups=0],own[rows=200,orphans=0,missing=0,dups=0],depth=200/200
p2=main=120,vis[rows=120,orphans=0,missing=0,dups=0],own[rows=120,orphans=0,missing=0,dups=0],depth=120/120
neg-control=1/0
p4=main=120,vis[rows=120,orphans=0,missing=0,dups=0],own[rows=120,orphans=0,missing=0,dups=0],depth-idx=80,depth-idx2=40,depth-own=120
p5=main=0,vis[rows=0,orphans=0,missing=0,dups=0],own[rows=0,orphans=0,missing=0,dups=0],depth=0/0/0
VERDICT=NO-ORPHANS
```

**The only difference between the two runs is `ix-by-key=[0/0]` → `[1/1]`.** Every correctness number
is byte-identical.

⚠ **`neg-control=1/0` is the reason this gate means anything** (R59 — a gate that cannot fail proves
nothing). Phase 3 INSERTs a ghost row straight into `index_by-visible-at` and the orphan query
reports **exactly 1**; the ghost is removed and it reports **0** again. The instrument can fail, and
it did on demand.

What each phase gates:

| phase | what it exercises | result |
|---|---|---|
| p1 | 200 rows put into both GSIs | main=200, both GSI tables 200 rows, `count-index` depth 200/200 |
| p2 | **delete 80 rows by BASE KEY** — the DDB contract | main=120, **orphans=0** both tables, **missing=0**, **dups=0**, depth **120/120** |
| p3 | ⚠ negative control | ghost injected → **orphans=1**; removed → **0** |
| p4 | **re-put 40 rows into a DIFFERENT GSI partition** — `put`'s clear step must remove the stale projection. A stale row has a LIVE base key, so the LEFT JOIN cannot see it; only `rows` and `dups` can | GSI still **120 rows** (not 160), `dups=0`, depth-idx **80** + depth-idx2 **40** = 120 |
| p5 | delete everything, including keys already deleted (idempotent DELETE of 0 rows) | main=0, both GSI tables **0 rows**, depth **0/0/0** |
| — | `ensure-schema` called **twice** on the same store (the `IF NOT EXISTS` trap-door) | both calls `Success`, one index each, no collision |

★★ **And admission — the thing an orphan would actually break — is unmoved in the live sweep.** An
orphan inflates `count-index`, `depth` rises, `room = cap - depth` shrinks, and the queue refuses
publishers. Across all 9 pre-change and all 9 post-change runs:

- the four subscriber tiers report **`refused=0`** on every run, before and after;
- the inbox's `refused` (its cap-64 back-pressure, refusals by design) is **statistically identical**:
  pre 603/603/605 · 1220/1233/1234 · 2479/2507/2521 → post 601/603/605 · 1224/1227/1239 ·
  2504/2519/2526 — every band overlaps;
- `visible=0` and `unacked=0` on all 5 tiers on all 18 runs.

**No capacity was refused that the queue had.**

---

## ⚠ STOP-3 FIRED — AND ITS STATED MECHANISM IS REFUTED

STOP-3: *"if `count-index` or `scan-index` latency moves materially, STOP and name it. SQLite has
chosen a different query plan, and that is a finding about the schema."*

**`scan-index` moved materially, in both directions, with non-overlapping spreads at two of three
points** (four subscriber tiers, µs/call, median [min–max]):

| n | before | after | delta | spreads overlap? |
|---|---|---|---|---|
| 1000 | 1514.3 [1456.8–1551.1] | 1078.2 [1027.1–1111.0] | **−28.8 %** | **NO** |
| 2000 | 1243.6 [1188.7–1336.6] | 1293.6 [1272.9–1315.0] | +4.0 % | yes |
| 4000 | 1257.1 [1238.8–1323.2] | 1365.3 [1343.3–1382.5] | **+8.6 %** | **NO** |

Its `4000/1000` therefore flips from **0.830** to **1.266** — the one op the previous SCORE
exonerated as flat now has a slope. So I stopped and asked the planner, rather than reasoning about
it: `wat-scripts/scratch-pad/probe-which-index-the-gsi-queries-use.wat` runs
`EXPLAIN QUERY PLAN` on the **verbatim statement text** the store issues, against a schema that
carries **both** an indexed GSI table and a schema-identical control table that does not get the new
index — the before and after in **one run**, no rebuild, no remembered number:

```
gsi-indexes=[index_by-visible-at_by_key+sqlite_autoindex_index_by-visible-at_1]
control-indexes=[sqlite_autoindex_index_control_1]
BEFORE del0=SCAN index_control
AFTER  del=SEARCH index_by-visible-at USING INDEX index_by-visible-at_by_key (pk=? AND sk=?)
BEFORE six0=SEARCH index_control USING INDEX sqlite_autoindex_index_control_1 (ipk=? AND isk>? AND isk<?)
AFTER  six=SEARCH index_by-visible-at USING INDEX sqlite_autoindex_index_by-visible-at_1 (ipk=? AND isk>? AND isk<?)
cix=CO-ROUTINE (subquery-1)|SEARCH index_by-visible-at USING COVERING INDEX sqlite_autoindex_index_by-visible-at_1 (ipk=? AND isk>? AND isk<?)|SCAN (subquery-1)
scn=SEARCH main USING INDEX sqlite_autoindex_main_1 (pk=? AND sk>? AND sk<?)
mdl=SEARCH main USING INDEX sqlite_autoindex_main_1 (pk=? AND sk=?)
```

Two things are settled by this and nothing else in the stone settles either of them:

1. ★★ **`BEFORE del0=SCAN index_control`.** The DESIGN's diagnosis is confirmed **by the planner
   itself**, not by a latency: the GSI-row delete really was a full table scan, and
   `AFTER del=SEARCH … USING INDEX index_by-visible-at_by_key (pk=? AND sk=?)` is the seek.
2. ⚠ **STOP-3's stated mechanism is REFUTED.** `six0` and `six` are the **same plan on the same
   index** (`sqlite_autoindex_index_<name>_1`, `ipk=? AND isk>? AND isk<?`), and `cix` is still a
   **COVERING** scan of that primary key. SQLite has **not** chosen a different index for any read
   query. `scan` and `main`'s DELETE are untouched — there is no new index on `main` at all.

⚠ **So what did move `scan-index`? I cannot separate it, and I will not guess.** Two candidates
survive and this instrument cannot tell them apart:

- **page-cache pressure.** At n=4000 the GSI table now carries a second 4000-entry b-tree competing
  for the same cache. That fits +8.6 % at n=4000 and ~nothing at n=1000.
- **a different poller mix.** `workers` — the count of *distinct* worker identities that produced an
  outcome — went from **8–10** (the arc's long-standing band) to **11–12 on all nine post runs**,
  because the drain finishes 24 % sooner and more of the pool participates. `scan-index` is the op
  called once per `take`, and the previous SCORE established that its **call count is poller-driven
  and has the widest spread of any counter in this harness** (2882–3808 at n=1000 there; band
  1.580–2.382, *"admits no slope statement at all"*). A changed empty-vs-full page mix moves its
  per-call mean without any SQLite work changing.

**This is reported as an open observation, not a disposition.** It is the one number in this SCORE
whose mechanism is unnamed, and it is named as unnamed.

⚠ **`count-index` also has a small, one-point real shift**, reported rather than rounded away: subs
n=1000 pre [739.5–749.8] vs post [754.0–768.1] is **non-overlapping, +2.7 %**; n=2000 +0.8 % and
n=4000 +1.1 % both overlap. Its slope is unchanged (1.286 → 1.267) and its plan is unchanged
(covering, on the primary key).

---

## ⭑ ROW 5 — THE NET EFFECT ON `drain`, AND ON EVERY OTHER PHASE

Medians over three runs at each `n`, `[min–max]` the observed extreme pair, times in **ms** as the
harness prints them.

| phase | n | before | after | after/before | spreads overlap? |
|---|---|---|---|---|---|
| **`drain`** | 1000 | 1338 [1334–1358] | 1307 [1275–1326] | **0.977** | **NO** (−2.3 %) |
| **`drain`** | 2000 | 3151 [3134–3169] | 2762 [2762–2788] | **0.877** | **NO** (−12.3 %) |
| **`drain`** | 4000 | 7609 [7455–7723] | 5800 [5733–5822] | **0.762** | **NO** (−23.8 %) |
| `drain-store-ms` | 1000 / 2000 / 4000 | 695 / 1783 / 4737 | 617 / 1356 / 2840 | 0.888 / 0.760 / **0.600** | NO at all three |
| `drain-busy-ms` | 1000 / 2000 / 4000 | 808 / 2042 / 5277 | 729 / 1626 / 3410 | 0.902 / 0.796 / **0.646** | NO at all three |
| ⚠ `fill` | 1000 | 7830 [7721–7881] | 7876 [7849–7923] | 1.006 | yes |
| ⚠ `fill` | 2000 | 15960 [15663–15962] | 16103 [16011–16272] | **1.009** | **NO** (+0.3…+3.9 %) |
| ⚠ `fill` | 4000 | 32793 [31965–33250] | 33327 [32866–33747] | 1.016 | yes |
| `setup` | 1000 / 2000 / 4000 | 12474 / 12504 / 12529 | 12463 / 12458 / 12444 | 0.999 / 0.996 / 0.993 | yes |
| `collect` | 1000 / 2000 / 4000 | 5192 / 4430 / 7683 | 4383 / 4197 / 7612 | 0.844 / 0.947 / 0.991 | yes |
| `total` | 1000 | 27024 [26117–27111] | 26577 [25552–26975] | 0.983 | yes |
| `total` | 2000 | 36578 [36022–36852] | 35832 [35718–35985] | **0.980** | **NO** (−2.0 %) |
| `total` | 4000 | 61010 [60005–61479] | 59123 [58983–59804] | **0.969** | **NO** (−3.1 %) |
| `wall` | 1000 / 2000 / 4000 | 29289 / 39577 / 65536 | 28798 / 38795 / 63579 | 0.983 / **0.980** / **0.970** | NO at n=2000, n=4000 |

**`drain`'s slope**: `4000/1000` **5.687 → 4.438**; `drain-store-ms` **6.816 → 4.603**;
`drain-busy-ms` **6.531 → 4.678**.

★ **The win is per-call, not fewer calls.** `drain-store-calls` barely moved (374 → 368, 753 → 734,
1539 → 1454 — a 5 % drop at n=4000, all poller-driven `scan-index`), so the drain's mean per-store-call
latency carries it:

| n | before, med [min–max] | after, med [min–max] | ratio |
|---|---|---|---|
| 1000 | 1.858 [1.829–1.913] ms | 1.663 [1.658–1.730] ms | 0.895 |
| 2000 | 2.367 [2.365–2.368] ms | 1.854 [1.847–1.864] ms | 0.783 |
| 4000 | 3.074 [3.023–3.172] ms | **1.952 [1.940–1.953] ms** | **0.635** |

Per-run `drain-store-ms / drain-store-calls`, then the median — not a ratio of medians. Spreads are
non-overlapping at all three n. Per-call `4000/1000`: **1.654 → 1.174.**

⚠ **`fill` pays, and it is the same ~1 % the inbox `put` tax predicts** — `fill` is dominated by
publishing into the capped inbox, where the mapping cannot help and the third b-tree is pure cost.
+0.6 % / +0.9 % / +1.6 %, non-overlapping at n=2000 only. **`setup` is flat** (12434–12567 ms across
all 18 runs): creating an index on an empty table costs nothing measurable, exactly as the trap-doors
predicted, and this stone did not drift into setup work.

**Net: `drain` −2.3 / −12.3 / −23.8 %, `fill` +0.6 / +0.9 / +1.6 %, `total` −1.7 / −2.0 / −3.1 %.**
The put cost does **not** exceed the delete win — it is an order of magnitude smaller, and at the
growing tiers `put` is itself a beneficiary. **Row 5's verdict: LAND.**

---

## THE FULL PER-OP TABLES — medians and observed min–max, at every n

µs per call. Median over the three runs at that `n`; `[min–max]` the observed extreme pair. Ratio
bands are the worst case the extremes admit, `min(hi n)/max(lo n)` to `max(hi n)/min(lo n)`.
**An op grows only if its whole band clears 1.0.**

### A. FOUR SUBSCRIBER tiers summed, `cap = 8192` — the tables that grow to `n` live rows

**BEFORE** (this box, minutes before the change, binary at HEAD `71eeefd1f` unmodified)

| op | n=1000 | n=2000 | n=4000 | `n2000/n1000` | `n4000/n2000` | `4000/1000` |
|---|---|---|---|---|---|---|
| **`delete`** | 2116.1 [2081.2–2176.0] | 2912.2 [2903.6–2915.0] | 4227.7 [4152.9–4352.1] | 1.376 [1.334–1.401] | 1.452 [1.425–1.499] | **1.998** |
| **`put`** | 1445.9 [1415.7–1447.4] | 1717.5 [1706.9–1719.9] | 2206.1 [2195.6–2245.7] | 1.188 [1.179–1.215] | 1.285 [1.277–1.316] | **1.526** |
| `count-index` | 745.0 [739.5–749.8] | 827.9 [824.4–831.8] | 958.2 [957.8–966.6] | 1.111 [1.099–1.125] | 1.157 [1.152–1.173] | 1.286 |
| `scan-index` | 1514.3 [1456.8–1551.1] | 1243.6 [1188.7–1336.6] | 1257.1 [1238.8–1323.2] | 0.821 [0.766–0.918] | 1.011 [0.927–1.113] | 0.830 |
| **store** (agg) | 1228.7 [1222.3–1236.7] | 1406.1 [1397.2–1410.4] | 1761.1 [1759.9–1796.9] | 1.144 [1.130–1.154] | 1.252 [1.248–1.286] | 1.433 |

**AFTER**

| op | n=1000 | n=2000 | n=4000 | `n2000/n1000` | `n4000/n2000` | `4000/1000` |
|---|---|---|---|---|---|---|
| **`delete`** | 1664.1 [1623.9–1681.2] | 1783.1 [1760.6–1788.0] | 1842.9 [1826.7–1845.7] | 1.072 [1.047–1.101] | 1.034 [1.022–1.048] | **1.107** |
| **`put`** | 1270.5 [1265.2–1284.6] | 1308.2 [1300.1–1310.2] | 1338.7 [1334.7–1339.0] | 1.030 [1.012–1.036] | 1.023 [1.019–1.030] | **1.054** |
| `count-index` | 764.8 [754.0–768.1] | 834.9 [828.6–835.5] | 969.2 [960.1–969.8] | 1.092 [1.079–1.108] | 1.161 [1.149–1.170] | 1.267 |
| ⚠ `scan-index` | 1078.2 [1027.1–1111.0] | 1293.6 [1272.9–1315.0] | 1365.3 [1343.3–1382.5] | 1.200 [1.146–1.280] | 1.055 [1.021–1.086] | **1.266** |
| **store** (agg) | 1078.0 [1061.5–1089.0] | 1156.9 [1151.7–1161.5] | 1237.0 [1231.7–1239.7] | 1.073 [1.058–1.094] | 1.069 [1.060–1.076] | 1.147 |

### B. INBOX only, `cap = 64` — the control table, and where the index tax is isolated

| op | n | before | after | after/before | overlap? |
|---|---|---|---|---|---|
| ⚠ `put` | 1000 | 2138.4 [2118.6–2161.1] | 2145.0 [2144.2–2192.0] | 1.003 | yes |
| ⚠ `put` | 2000 | 2121.5 [2098.8–2124.8] | 2161.4 [2154.7–2175.6] | **1.019** | **NO** (+1.4…+3.7 %) |
| ⚠ `put` | 4000 | 2115.3 [2103.9–2140.8] | 2136.4 [2131.8–2141.7] | 1.010 | yes |
| `delete` | 1000/2000/4000 | 695.3 / 709.9 / 707.1 | 707.9 / 702.1 / 696.1 | 1.018 / 0.989 / 0.984 | yes |
| `count-index` | 1000/2000/4000 | 619.2 / 627.3 / 623.7 | 624.9 / 632.3 / 626.8 | 1.009 / 1.008 / 1.005 | yes |
| `scan-index` | 1000/2000/4000 | 740.7 / 707.2 / 711.7 | 660.6 / 686.7 / 704.3 | 0.892 / 0.971 / 0.990 | n=1000 **NO** |
| **store** | 1000/2000/4000 | 856.3 / 834.6 / 834.5 | 807.3 / 823.8 / 835.8 | 0.943 / 0.987 / 1.002 | yes |

`4000/1000` at the inbox stays flat on all four ops before and after (`put` 0.989 → 0.996, `delete`
1.017 → 0.983, `count-index` 1.007 → 1.003, `scan-index` 0.961 → 1.066) — **the control still
controls.** `delete` at the inbox is **flat and unchanged** (707.1 → 696.1 µs at n=4000): with 64 live
rows the scan it used to do was already short, which is exactly why the mapping buys nothing there
and why the ~1 % `put` movement is the tax, isolated.

### C. WHOLE RUN, all five tiers summed

| op | n | before | after | after/before |
|---|---|---|---|---|
| `delete` | 1000/2000/4000 | 974.6 / 1139.5 / 1394.5 | 896.4 / 915.5 / 920.8 | 0.920 / 0.803 / **0.660** |
| `put` | 1000/2000/4000 | 1588.0 / 1802.9 / 2186.9 | 1466.0 / 1486.9 / 1505.7 | 0.923 / 0.825 / **0.688** |
| `count-index` | 1000/2000/4000 | 708.5 / 773.6 / 861.8 | 726.0 / 777.1 / 869.7 | 1.025 / 1.005 / 1.009 |
| `scan-index` | 1000/2000/4000 | 919.9 / 806.3 / 803.4 | 752.5 / 786.8 / 821.7 | 0.818 / 0.976 / 1.023 |
| `store` | 1000/2000/4000 | 1042.1 / 1104.1 / 1270.4 | 940.1 / 978.3 / 1024.5 | 0.902 / 0.886 / **0.806** |

Whole-run `4000/1000`: `delete` **1.431 → 1.027**, `put` **1.377 → 1.027**, `store` 1.219 → 1.090.

### D. The split still reconciles EXACTLY

`put + delete + count + scan` against the `store-calls`/`store-ns` aggregate it subdivides:
**remainder 0 calls and 0 nanoseconds on 45 of 45 tier reports before and 45 of 45 after** (5 tiers ×
9 runs, each side). No unaccounted operation appeared or disappeared.

---

## THE COMMAND LINES, QUOTED

```
./target/release/wat wat-scripts/fanout/circuit.wat 1000 4 3 8192 true 1000
./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true 1000
./target/release/wat wat-scripts/fanout/circuit.wat 4000 4 3 8192 true 1000
```

`m=4 j=3 sub-cap=8192 fill-first?=true`, **`vis-ms=1000` pinned at all three points**; only `n`
varies. Each side of the comparison is **9 runs, interleaved** `1000, 2000, 4000 × 3 rounds`, so no
`n` owns a contiguous block of the box's history.

⚠ **`wat/query/*.wat` is frozen into the binary at build time, so the baseline needed its own build.**
The BEFORE sweep ran on `cargo build --release` at HEAD `71eeefd1f` with a clean tree; the change was
then made and the binary rebuilt; the AFTER sweep ran on that. **The baseline was measured on this
box minutes before the change rather than quoted from the stone** — and it reproduces the stone's
`ba63bedf3` figures closely (`delete` 1.998 vs 1.966, `put` 1.526 vs 1.514, `count-index` 1.286 vs
1.268, `scan-index` 0.830 vs 0.890).

---

## THE SWEEP — eighteen runs, none merged, none dropped

All 18 `rc=0`, **zero bytes on stderr** on every run. `load` is `cut -d' ' -f1 /proc/loadavg`
immediately before the run.

### BEFORE

| n | run | load | wall | `setup` | `fill` | `arm` | `drain` | `collect` | `stop` | `total` | `distinct` | `dup` | `workers` | `fill-depth` |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1000 | 1 | 0.12 | 29340 | 12488 | 7881 | 7 | 1338 | 5192 | 202 | 27111 | **4000** | **0** | 9 | [1000/0]×4 |
| 2000 | 1 | 0.34 | 39019 | 12454 | 15663 | 7 | 3169 | 4288 | 439 | 36022 | **8000** | **0** | 9 | [2000/0]×4 |
| 4000 | 1 | 1.17 | 65536 | 12529 | 32793 | 8 | 7609 | 7683 | 386 | 61010 | **16000** | **0** | 8 | [4000/0]×4 |
| 1000 | 2 | 2.27 | 29289 | 12453 | 7830 | 7 | 1334 | 5204 | 192 | 27024 | **4000** | **0** | 10 | [1000/0]×4 |
| 2000 | 2 | 1.49 | 39577 | 12532 | 15960 | 7 | 3151 | 4430 | 496 | 36578 | **8000** | **0** | 9 | [2000/0]×4 |
| 4000 | 2 | 1.60 | 64569 | 12483 | 31965 | 8 | 7455 | 7776 | 316 | 60005 | **16000** | **0** | 8 | [4000/0]×4 |
| 1000 | 3 | 2.49 | 28392 | 12474 | 7721 | 7 | 1358 | 4094 | 461 | 26117 | **4000** | **0** | 10 | [1000/0]×4 |
| 2000 | 3 | 1.98 | 39884 | 12504 | 15962 | 7 | 3134 | 4996 | 246 | 36852 | **8000** | **0** | 8 | [2000/0]×4 |
| 4000 | 3 | 1.47 | 65958 | 12567 | 33250 | 8 | 7723 | 7415 | 513 | 61479 | **16000** | **0** | 8 | [4000/0]×4 |

### AFTER

| n | run | load | wall | `setup` | `fill` | `arm` | `drain` | `collect` | `stop` | `total` | `distinct` | `dup` | `workers` | `fill-depth` |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1000 | 1 | 0.87 | 27778 | 12517 | 7876 | 7 | 1307 | 3638 | 204 | 25552 | **4000** | **0** | 11 | [1000/0]×4 |
| 2000 | 1 | 1.28 | 38795 | 12501 | 16103 | 7 | 2762 | 4197 | 259 | 35832 | **8000** | **0** | 12 | [2000/0]×4 |
| 4000 | 1 | 2.10 | 63378 | 12504 | 32866 | 7 | 5800 | 7612 | 191 | 58983 | **16000** | **0** | 12 | [4000/0]×4 |
| 1000 | 2 | 2.72 | 28798 | 12463 | 7923 | 8 | 1326 | 4383 | 472 | 26577 | **4000** | **0** | 12 | [1000/0]×4 |
| 2000 | 2 | 2.44 | 38668 | 12458 | 16272 | 7 | 2788 | 3942 | 248 | 35718 | **8000** | **0** | 12 | [2000/0]×4 |
| 4000 | 2 | 2.62 | 64217 | 12434 | 33747 | 7 | 5733 | 7618 | 262 | 59804 | **16000** | **0** | 12 | [4000/0]×4 |
| 1000 | 3 | 2.81 | 29605 | 12434 | 7849 | 7 | 1275 | 4673 | 734 | 26975 | **4000** | **0** | 12 | [1000/0]×4 |
| 2000 | 3 | 2.23 | 38963 | 12439 | 16011 | 7 | 2762 | 4468 | 295 | 35985 | **8000** | **0** | 11 | [2000/0]×4 |
| 4000 | 3 | 2.72 | 63579 | 12444 | 33327 | 7 | 5822 | 7405 | 114 | 59123 | **16000** | **0** | 12 | [4000/0]×4 |

`distinct = n×m` and `dup = 0` on **all eighteen**. `empty=1`, `seen-recorded = n×m`,
`check-exhausted=0`, `mark-exhausted=0`, `visible=0`, `unacked=0`, subscriber `refused=0` on all
eighteen. `fill-depth` is `[n/0]` on every subscriber on every run, so the subscriber tables really do
reach `n` live rows.

### ⚠ Two behavioural shifts in the AFTER runs, reported not filed

- **`workers` 8–10 → 11–12 on all nine.** Distinct worker identities that produced an outcome. The
  drain finishes 24 % sooner, so more of the pool participates. Outside the band the previous three
  SCOREs recorded (8–10). Named; it is also one of the two surviving candidates for `scan-index`'s
  movement above.
- **`ack-retries` 0–6 → 10–21 and `ack-exhausted` 0–1 → 1–5.** Retries on the ack path and acks whose
  retry budget ran out. Both were already non-zero before. `dup=0`, `unacked=0` and
  `seen-recorded = n×m` on every run say **nothing was lost or double-delivered**, and `acks ≥ n` per
  tier is the retried-ack over-count the harness already prints. ⚠ **I cannot name the mechanism with
  this instrument**: more distinct workers contending on one queue actor is the obvious candidate and
  I did not measure it. It is a real, non-overlapping shift and it is on the record as unexplained.
- `seen-skipped` was 0 or 10 (pre: 0 on eight runs, 10 on one; post: 0 on five, 10 on four) — the same
  upstream visibility-timing variability the previous three SCOREs recorded. Reported.

### The box

`ps -eo args | grep -E 'cargo|nextest|release/wat' | grep -v grep` was captured **before every one of
the eighteen runs** and every time showed exactly two lines, both idle MCP servers:

```
/home/john/.cargo/bin/wat --mcp
/home/john/.cargo/bin/wat --mcp
```

(36 captured lines = 18 runs × 2, all identical.) **No run ran beside another** — a single sequential
driver, 10 s between runs, one sweep at a time, and the two rebuilds and both probes ran between the
sweeps, never during one. **The floor ran after all eighteen.** Start loads were **0.12–2.49**
(before) and **0.87–2.81** (after); the interleaved ordering spreads the within-sweep drift across all
three `n` rather than correlating it with one. ⚠ The AFTER sweep's loads are ~0.5 higher on average
than the BEFORE sweep's, which if anything works **against** the measured win.

---

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ **no orphan GSI rows** | ✅ **PASS, directly queried.** `LEFT JOIN main` over each `index_*` table on a file-backed store, read by a **second SQLite connection**: **orphans=0** after every put/delete/re-put/delete-all cycle, on **both** declared GSIs. Reverse join **missing=0** (no live row lost its projection) and **dups=0** (no stale projection under a changed `(ipk,isk)` — the orphan the LEFT JOIN cannot see). ⚠ **`neg-control=1/0`**: an injected ghost row makes the query report 1, so the gate can fail. The identical probe on a pre-change binary prints byte-identical numbers — **the change is additive on correctness** |
| 2 | ★★ **`count-index` still reports the true depth** | ✅ **PASS.** depth 200/200 → 120/120 after deleting 80 → 80+40 across two GSI partitions after a re-put → 0/0/0 after deleting all. Equals the live row count at every point, on both GSIs. ★★ And in the live sweep the thing depth drives is unmoved: subscriber **`refused=0` on all 18 runs**, inbox `refused` statistically identical before/after (bands all overlap), `visible=0`/`unacked=0` everywhere |
| 3 | ★ **`delete`'s per-call latency flattens** | ✅ **PASS.** Subscriber `4000/1000` **1.998 [1.909–2.091] → 1.107 [1.087–1.137]**, against the stone's 1.966 baseline. Absolute at n=4000: **4227.7 → 1842.9 µs (2.29× faster)**. Whole-run 1.431 → 1.027. The planner confirms the mechanism: `SCAN` → `SEARCH … USING INDEX index_<name>_by_key (pk=? AND sk=?)` |
| 4 | ★ **`put`'s cost is measured, not assumed** | ✅ **PASS — and it inverts the stone's cost model.** Subscriber `4000/1000` **1.526 → 1.054**, absolute **2206.1 → 1338.7 µs at n=4000**: `put` got **faster**, because `put-one-row` calls the same `clear-index-projections`. The isolated index tax is only visible at the **cap-64 inbox**, where the scan was already cheap: **+0.3 % / +1.9 % / +1.0 %**, non-overlapping at n=2000 only. `fill` (inbox-dominated) pays the matching **+0.6 / +0.9 / +1.6 %** |
| 5 | ★ **the NET effect on `drain` is stated** | ✅ **PASS — it is a WIN.** `drain` **1338 → 1307 (−2.3 %) / 3151 → 2762 (−12.3 %) / 7609 → 5800 (−23.8 %)**, **non-overlapping spreads at all three n**. Slope `4000/1000` **5.687 → 4.438**; `drain-store-ms` 6.816 → 4.603; mean drain per-store-call `4000/1000` **1.654 → 1.174**. `total` −1.7 / −2.0 / −3.1 %; `wall` −1.7 / −2.0 / −3.0 %. **The put cost does not exceed the delete win — it is ~1 % against 24 %. RECOMMEND LANDING** |
| 6 | **the read path is unmoved** | ⚠ **SPLIT — and this is STOP-3, named in full above.** `count-index` **1.286 → 1.267** (≈1.27 as required; one non-overlapping +2.7 % at subs n=1000, reported). **`scan-index` MOVED**: 0.830 → **1.266**, −28.8 % at n=1000 and **+8.6 % at n=4000**, both non-overlapping. ★★ `EXPLAIN QUERY PLAN` on the verbatim statements, with a schema-identical control table that lacks the new index, shows **the read plans are IDENTICAL** (`SEARCH … USING INDEX sqlite_autoindex_index_<name>_1 (ipk=? AND isk>? AND isk<?)`; `count-index` still **COVERING**). **STOP-3's stated mechanism — "SQLite has chosen a different index" — is REFUTED.** Two candidates remain (page-cache pressure from the extra b-tree; a changed poll mix, `workers` 8–10 → 11–12) and **this instrument cannot separate them**. Reported as open |
| 7 | **correctness at every point** | ✅ **PASS.** `distinct = 4000 / 8000 / 16000 = n×m` and `dup = 0` on **all 18 runs** (9 before, 9 after). `empty=1`, `seen-recorded = n×m`, `check-exhausted=0`, `mark-exhausted=0`, `visible=0`, `unacked=0`, subscriber `refused=0` everywhere. `rc=0` and **zero stderr bytes** on all 18. No run excluded. ⚠ `ack-retries`/`ack-exhausted` and `workers` shifted — reported above, unexplained, no message lost |
| 8 | **blast radius** | ✅ **PASS.** `git diff --stat`: `wat/query/sqlite-store.wat | 38 +++--` (**35 insertions, 3 deletions**), inside `ensure-index-tables` **only**. Plus two untracked probes under `wat-scripts/scratch-pad/`. **No `mem.wat`. No `sqs.wat`. No `circuit.wat`. No `src/`.** `git diff --name-only | grep -v '^wat/query/sqlite-store\.wat$' | wc -l` = **0** |
| 9 | **the corpus loads** | ✅ **PASS.** `wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime` **PASS [486.351s]**, with both new probes present and type-checked on the changed runtime |
| 10 | **the floor holds** | ✅ **PASS.** `Summary [ 486.358s] 5237 tests run: 5237 passed (7 slow), 22 skipped` — **0 FAIL, 0 TIMEOUT**, exit `0`, **no `ARM.txt`**, zero `FAIL`/`TRY`/`TIMEOUT`/`ABORT`/`SIGSEGV` lines in `clean.log`. Ran **once**, after the whole sweep. Nothing was re-run |

---

## STOPS

- **STOP-1** (any orphan GSI row can exist after a delete cycle → STOP) — **did not fire.** Asked of
  SQLite directly, twice per cycle in both join directions plus a duplicate-projection check, on two
  GSI tables, with a negative control that proves the query can report a non-zero. **orphans=0,
  missing=0, dups=0** at every phase, and `count-index` equals the live count at every phase. The
  live sweep's `refused=0` on all 18 runs is the behavioural corroboration.
- **STOP-2** (`put`'s growth exceeds the `delete` win → STOP and recommend against landing) — **did
  not fire, and the axis it guarded runs the other way.** `put` at the growing tiers got **faster**
  (`4000/1000` 1.526 → 1.054; −39 % absolute at n=4000) because it issues the same scanning DELETE in
  its clear step. The genuine index tax, isolated at the cap-64 inbox where the mapping cannot help,
  is **+0.3 % to +1.9 %**, against a **24 %** drain win. **Recommendation: LAND.**
- ⚠ **STOP-3** (`count-index` or `scan-index` latency moves materially → STOP and name it) —
  **FIRED.** `scan-index` moved: −28.8 % at n=1000 and **+8.6 % at n=4000**, non-overlapping spreads,
  `4000/1000` 0.830 → 1.266; `count-index` has one non-overlapping +2.7 % at subs n=1000. I stopped
  and named it with `EXPLAIN QUERY PLAN` against a control table, and **the mechanism STOP-3 asserts
  is refuted**: the read plans and chosen indexes are identical with and without the new index. The
  surviving candidates (page-cache pressure; a changed poll mix with `workers` 8–10 → 11–12) **cannot
  be separated by a caller-side instrument** and are on the record as open, not resolved.
- **STOP-4** (do not touch `mem.wat`, `sqs.wat`, `circuit.wat`, or `src/`) — **held.** One tracked
  file changed, `wat/query/sqlite-store.wat`, inside `ensure-index-tables`. `git status --porcelain`
  shows that one `M` plus two `??` probes and this SCORE, and nothing else.
- **STOP-5** (do not change the `DELETE` statement, delete semantics, or what rows a delete removes) —
  **held.** The diff adds one `bykey` binding, one `execute-ddl` call and one `match` level inside
  `ensure-index-tables`. `clear-index-projections`, `delete-one-key`, `delete-rows` and every SQL
  string in the file are **untouched** — `git diff` contains no line from any of them. The probe is
  the behavioural proof: identical row counts, depths, orphan counts and duplicate counts before and
  after, differing only in `ix-by-key=[0/0]` → `[1/1]`.
- **STOP-6** (on any red floor arm: capture whole, name the arm, do not re-run) — **no red arm.** The
  floor ran **once**, exit 0, no `ARM.txt`, zero `FAIL`/`TRY`/`TIMEOUT`/`ABORT`/`SIGSEGV` lines.
  Nothing was re-run.

---

## WHAT THE NEXT STONE INHERITS

1. ★★ **`put`'s 1.514× was never a separate story.** It was largely the same missing reverse mapping,
   reached through `put-one-row`'s clear step. Two SCOREs filed it as "plausibly b-tree depth or the
   per-call transaction" and out of scope. What remains of `put`'s slope is **1.054**.
2. **`delete` and `put` are both essentially flat now** at the growing tiers. The steepest remaining
   op is **`count-index` at 1.267**, which is the same 1.27 the previous SCORE measured — the one op
   this stone did not touch and whose plan is a **covering** scan of the primary key. That is the next
   slope, and it is not an index problem.
3. ⚠ **`scan-index` now has a slope (1.266) where it had none (0.830), and the plan is provably
   unchanged.** Whatever moves it is not the schema. Page-cache pressure and the poller mix are the
   two live candidates; separating them needs a **server-side per-query instrument**, the same one the
   previous DESIGN deferred.
4. ⚠ **`ack-retries` and `ack-exhausted` are 3–5× higher post-change**, with `workers` up from 8–10 to
   11–12. Nothing is lost (`dup=0`, `unacked=0`, `seen-recorded = n×m`) but the ack path's retry
   budget is being consumed far more often than any previous SCORE recorded. **Unexplained.**
5. **`mem.wat` still has the same contract with no index structure at all** (`key-hits-row?`,
   `:96-107`). Named and still out of scope; the drain uses `sqlite-store`.
6. **`setup` is unmoved** (12434–12567 ms across all 18 runs). Creating an index on an empty table is
   free, and the `IF NOT EXISTS` idempotence is checked directly (`ensure-schema` called twice).

---

## BLAST RADIUS

```
$ git status --porcelain
 M wat/query/sqlite-store.wat
?? docs/arc/2026/06/278-rules-engine/SCORE-the-gsi-delete-gets-its-reverse-mapping.md
?? wat-scripts/scratch-pad/probe-no-orphan-gsi-rows-after-delete.wat
?? wat-scripts/scratch-pad/probe-which-index-the-gsi-queries-use.wat

$ git diff --stat
 wat/query/sqlite-store.wat | 38 +++++++++++++++++++++++++++++++++++---
 1 file changed, 35 insertions(+), 3 deletions(-)

$ git diff --name-only | grep -vE '^wat/query/sqlite-store\.wat$' | wc -l
0
```

The whole functional change, inside `ensure-index-tables` (the rest of the 35 insertions is the
comment that explains why):

```wat
bykey (:wat::core::format
        "CREATE INDEX IF NOT EXISTS [index_{name}_by_key] ON [index_{name}] (pk, sk)"
        :name name)
```

chained after the `CREATE TABLE` with one more `match` level. **One tracked file, `ensure-schema`
only, additive. Left uncommitted.**
