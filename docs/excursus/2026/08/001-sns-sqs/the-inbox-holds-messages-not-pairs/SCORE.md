# SCORE — the inbox holds messages, not pairs

**SCORED.** Executor: claude, 2026-09-09. Did not commit. **One STOP fired — not as a failure but as
the falsification the stone asked for: STOP-3's row-6 prediction is REFUTED. The inbox still refuses.**
One functional file modified: `wat-scripts/topic/sns-fanout.wat`. Plus one Rust gate that encoded the
OLD tier-1 unit as a guarantee, and four scratch-probe headers whose prose the change made false.

```
     Summary [ 482.388s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-09T07-55-36Z/` — `scripts/floor.sh` exit `0`, **no `ARM.txt`**, zero
`FAIL`/`TRY`/`TIMEOUT`/`ABORT`/`SIGSEGV` lines in `clean.log`. **5237** — unchanged (one test renamed,
none added or removed). `wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime`
**PASS [482.381s]**. Run **once**, **after** all eighteen sweep runs, never beside one. Nothing was re-run.

---

## ⭑ THE HEADLINE — the expansion moved, nothing was lost, and the publisher stopped paying for it

`distinct = n×m` and `dup = 0` on **all eighteen runs**, before and after. The change is coherence,
not throughput — but the throughput fell out anyway, and it fell out of the **publisher's** side:

| | n=1000 | n=2000 | n=4000 |
|---|---|---|---|
| `fill` | 7697 → **1310 ms** (0.170) | 15612 → **2633** (0.169) | 32889 → **5318** (0.162) |
| `drain` | 1263 → **1194** (0.945) | 2663 → **2519** (0.946) | 5616 → **5283** (0.941) |
| `total` | 25753 → **20076** (0.780) | 35775 → **23005** (0.643) | 58332 → **31412** (0.539) |
| whole-run `store-calls` | 4886 → **2393** (0.490) | 9644 → **4676** (0.485) | 19099 → **9218** (0.483) |
| whole-run `store-ms` | 5212 → **3406** (0.653) | 10930 → **7128** (0.652) | 23035 → **14869** (0.645) |

Every one of those has **non-overlapping** min–max bands at all three `n`. The mechanism is not
subtle: tier 1 stopped carrying four copies of every message. The inbox's own store work collapses to
**~15 % of the calls and ~20 % of the time**:

| tier group | metric | n=4000 before | n=4000 after |
|---|---|---|---|
| **inbox** | store-calls | 22041 | **3249** (0.147) |
| **inbox** | store-ms | 17686 | **3500** (0.198) |
| subs (4) | store-calls | 19589 | 9673 (0.494) |
| subs (4) | store-ms | 23393 | 15555 (0.665) |
| all 5 | store-calls | 41582 | **12913** (0.311) |
| all 5 | store-ms | 40982 | **19042** (0.465) |

★ **The subscriber tiers halve their store CALLS while delivering exactly the same rows** — because
the worker now writes one 10-body batch per subscriber per tick instead of ~2.5 bodies, and acks the
inbox **once** for the tick instead of once per non-empty bucket. Inbox `delete-calls` at n=4000:
**6591 → 401 (16.4× fewer)** for the same acked work.

---

## ⚠ ROW 6 — THE PREDICTION IS REFUTED. THE INBOX STILL REFUSES.

The stone predicted inbox `refused` → **0**, on the argument that a batch is now ≤10 messages rather
than 40 pairs, so `:cap 64` would stop binding. **It did not go to zero.** `:cap 64` was NOT changed
(STOP-3 held), so this is the measurement the stone asked for, and it says the cap still binds:

| n | before, med [min–max] | after, med [min–max] | after/before | per published message |
|---|---|---|---|---|
| 1000 | **602** [599–612] | **91** [90–92] | 0.151 | 0.602 → **0.091** |
| 2000 | **1221** [1220–1223] | **190** [186–191] | 0.156 | 0.611 → **0.095** |
| 4000 | **2502** [2467–2511] | **390** [389–391] | 0.156 | 0.626 → **0.098** |

Bands are non-overlapping at every `n`. The stone's quoted **1209–1233** is the **n=2000** point, and
my own pre-change baseline on this box reproduced it exactly (1220/1221/1223).

**What the numbers do establish**, and it is the useful half of the prediction: a publish call is
refused **~6.4× less often**, from **6.0–6.3 refusals per publish call** to **0.91–0.98**. And the
consequence the refusals actually caused is nearly gone — publisher backoff `asleep` falls
**3972 → 110 / 8165 → 226 / 17510 → 469 ms** (a factor of **36**), and `publish-attempts`
**702 → 191 / 1421 → 390 / 2902 → 790**. That is where `fill`'s 6× comes from.

**What the numbers refute**: *"64 messages is a different object from 64 pairs, therefore the cap is a
non-constraint."* It is a different object and the cap **still constrains it**. `sqs.wat`'s admission
is all-or-nothing whenever `n0 <= cap` (`sqs.wat:465`), so a refusal means `room < 10`, and the inbox
still sits inside 10 slots of full often enough to refuse ~0.95 times per publish call. The drain is
still the bottleneck: `drain-store-calls` is **unchanged** (370→372, 732→742, 1451→1478) and `drain`
itself only fell 5–6 %. Fewer rows per message did not make the consumer faster; it made the producer
cheaper.

⚠ **What I did NOT measure, and will not guess:** whether the residual refusals are steady-state
saturation or bursts. That needs an inbox **depth distribution** over the fill, and this harness
reports only the terminal `visible`/`unacked` (both 0 on all 18 runs) and a cumulative `refused`
counter. Two worlds print `refused=390`: a queue hovering at 55–64 for the whole fill, and a queue
that is empty most of the time and slams full in bursts. **The instrument cannot separate them.**
The cap question is therefore **still open**, and the honest statement is the narrow one: `:cap 64`
binds ~6.4× less often on messages than on pairs, and it has not stopped binding.

---

## ⭑⭑ ROW 1 — THE ROW THAT OUTRANKS EVERYTHING. ALL EIGHTEEN RUNS.

`./target/release/wat wat-scripts/fanout/circuit.wat <n> 4 3 8192 true 1000` — `m=4 j=3 sub-cap=8192
fill-first?=true`, **`vis-ms=1000` pinned**, only `n` varies. Nine runs each side, **interleaved**
`1000, 2000, 4000 × 3 rounds`, so no `n` owns a contiguous block of the box's history.

### BEFORE — this box, minutes before the change, tree clean at HEAD `c96d24ce9`

| n | run | load | wall-phases: `setup` | `fill` | `arm` | `drain` | `collect` | `stop` | `total` | `distinct` | `dup` | `workers` | `fill-depth` |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1000 | 1 | 0.33 | 12430 | 7697 | 7 | 1243 | 4093 | 280 | 25753 | **4000** | **0** | 10 | [1000/0]×4 |
| 1000 | 2 | 2.99 | 12498 | 7762 | 7 | 1269 | 4623 | 457 | 26619 | **4000** | **0** | 11 | [1000/0]×4 |
| 1000 | 3 | 2.06 | 12496 | 7581 | 8 | 1263 | 3498 | 75 | 24924 | **4000** | **0** | 11 | [1000/0]×4 |
| 2000 | 1 | 1.07 | 12499 | 15546 | 7 | 2662 | 4883 | 143 | 35742 | **8000** | **0** | 10 | [2000/0]×4 |
| 2000 | 2 | 2.48 | 12470 | 15612 | 7 | 2663 | 4902 | 117 | 35775 | **8000** | **0** | 11 | [2000/0]×4 |
| 2000 | 3 | 1.58 | 12437 | 15742 | 8 | 2676 | 4551 | 432 | 35848 | **8000** | **0** | 11 | [2000/0]×4 |
| 4000 | 1 | 5.93 | 12490 | 32889 | 8 | 5623 | 7540 | 367 | 58919 | **16000** | **0** | 12 | [4000/0]×4 |
| 4000 | 2 | 2.24 | 12514 | 32127 | 8 | 5591 | 7889 | 105 | 58236 | **16000** | **0** | 12 | [4000/0]×4 |
| 4000 | 3 | 1.39 | 12539 | 33192 | 8 | 5616 | 6901 | 75 | 58332 | **16000** | **0** | 10 | [4000/0]×4 |

### AFTER

| n | run | load | `setup` | `fill` | `arm` | `drain` | `collect` | `stop` | `total` | `distinct` | `dup` | `workers` | `fill-depth` |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1000 | 1 | 3.43 | 12449 | 1310 | 7 | 1194 | 4624 | 286 | 19872 | **4000** | **0** | 10 | [1000/0]×4 |
| 1000 | 2 | 2.58 | 12463 | 1312 | 7 | 1184 | 4683 | 478 | 20130 | **4000** | **0** | 10 | [1000/0]×4 |
| 1000 | 3 | 1.74 | 12450 | 1296 | 6 | 1225 | 4671 | 425 | 20076 | **4000** | **0** | 11 | [1000/0]×4 |
| 2000 | 1 | 3.24 | 12486 | 2615 | 7 | 2519 | 4972 | 404 | 23005 | **8000** | **0** | 10 | [2000/0]×4 |
| 2000 | 2 | 1.86 | 12451 | 2636 | 7 | 2484 | 4306 | 59 | 21945 | **8000** | **0** | 11 | [2000/0]×4 |
| 2000 | 3 | 1.55 | 12528 | 2633 | 7 | 2555 | 5116 | 438 | 23279 | **8000** | **0** | 11 | [2000/0]×4 |
| 4000 | 1 | 2.77 | 12468 | 5309 | 7 | 5267 | 8402 | 234 | 31690 | **16000** | **0** | 12 | [4000/0]×4 |
| 4000 | 2 | 1.52 | 12570 | 5409 | 7 | 5283 | 7292 | 283 | 30847 | **16000** | **0** | 11 | [4000/0]×4 |
| 4000 | 3 | 1.43 | 12503 | 5318 | 7 | 5298 | 7774 | 509 | 31412 | **16000** | **0** | 10 | [4000/0]×4 |

**`distinct = n×m` and `dup = 0` on all eighteen.** `empty=1`, `seen-recorded = n×m`,
`check-exhausted=0`, `mark-exhausted=0`, `visible=0`, `unacked=0`, subscriber `refused=0`, inbox
`redeliveries=0` on all eighteen. `fill-depth` is `[n/0]` on every subscriber on every run.
`rc=0` and **zero bytes on stderr** on all eighteen. No run excluded, none merged, none dropped.

⚠ **`seen-skipped`** — before: 0 on four runs, 10 on three, 20 on two; after: 0 on five, 10 on four.
The same upstream visibility-timing variability three previous SCOREs recorded, and the after side is
if anything quieter. Reported, not filed. `ack-retries` 1–16 before / 2–16 after and `ack-exhausted`
0–4 / 0–4: both bands overlap.

---

## ⭑ THE PER-TIER LINE — what tier 1 now counts, and the reconciliation

| n | inbox `accepted` before | inbox `accepted` after | inbox `acks` after | sub `refused` |
|---|---|---|---|---|
| 1000 | 4000 (`n×m`) | **1000** (`= n`) | 1000 | 0/0/0/0 |
| 2000 | 8000 (`n×m`) | **2000** (`= n`) | 2000 | 0/0/0/0 |
| 4000 | 16000 (`n×m`) | **4000** (`= n`) | 4000 | 0/0/0/0 |

**Row 7 lands exactly**: the inbox depth now reads in messages, on all nine post runs, with no
rounding — `accepted` is `n` to the unit, and `acks` equals it.

**Row 8**: `put + delete + count + scan` against the `store-calls`/`store-ns` aggregate they
subdivide — **remainder 0 calls and 0 nanoseconds on 45 of 45 tier reports before and 45 of 45
after** (5 tiers × 9 runs each side).

---

## ⚠ THE PER-CALL LATENCIES MOVED, AND THE MEASUREMENT CONTAINS THE MEASURER

`µs/call` is **not comparable across this change**, and quoting it as if it were would be the exact
error this campaign keeps catching. The change alters *how much work one call does*: an inbox `put`
now writes 10 rows instead of 40, a subscriber `put` writes 10 instead of ~2.5, and an inbox `delete`
(the ack) removes ~10 ids instead of ~2.4. The tables are recorded for completeness, with the unit
change named beside each row.

### Four subscriber tiers, `cap = 8192` — µs/call, median [min–max]

| op | side | n=1000 | n=2000 | n=4000 | `4000/1000` | rows per call |
|---|---|---|---|---|---|---|
| `put` | before | 1228.0 [1224.3–1252.9] | 1267.5 [1262.2–1269.7] | 1290.8 [1286.0–1294.1] | 1.051 | ~2.5 |
| `put` | after | 2099.8 [2082.9–2109.8] | 2163.3 [2143.3–2170.3] | 2210.3 [2208.5–2229.9] | 1.053 | **~10** |
| `delete` | before | 1595.2 [1563.7–1612.1] | 1679.0 [1669.0–1705.0] | 1752.5 [1742.2–1767.0] | 1.099 | ~10 |
| `delete` | after | 1486.3 [1480.2–1532.4] | 1571.1 [1539.6–1576.9] | 1649.4 [1648.1–1660.1] | 1.110 | ~10 |
| `count-index` | before | 752.3 [747.0–759.9] | 820.8 [819.3–824.6] | 948.1 [941.4–948.3] | 1.260 | 1 |
| `count-index` | after | 796.2 [790.7–804.8] | 889.2 [882.8–925.5] | 1036.4 [1033.3–1036.5] | 1.302 | 1 |
| `scan-index` | before | 1416.2 [1314.9–1534.4] | 1387.6 [1380.6–1416.1] | 1288.7 [1286.4–1302.8] | 0.910 | ≤10 |
| `scan-index` | after | 1533.2 [1481.6–1597.1] | 1494.8 [1345.9–1573.7] | 1402.0 [1381.1–1425.7] | 0.914 | ≤10 |
| **store** (agg) | before | 1092.4 [1079.8–1126.1] | 1137.4 [1137.4–1145.5] | 1194.2 [1192.1–1194.7] | 1.093 | — |
| **store** (agg) | after | 1476.7 [1464.1–1509.2] | 1558.9 [1498.8–1561.9] | 1607.3 [1606.7–1611.6] | 1.088 | — |

**Every slope is unchanged** (`put` 1.051→1.053, `delete` 1.099→1.110, `count-index` 1.260→1.302,
`scan-index` 0.910→0.914, `store` 1.093→1.088). The *levels* moved because the batch behind each call
grew ~4×; total subscriber store time still **fell** (23393 → 15555 ms at n=4000, ×0.665).

### Inbox only, `cap = 64` — µs/call, median [min–max]

| op | side | n=1000 | n=2000 | n=4000 | `4000/1000` | rows per call |
|---|---|---|---|---|---|---|
| `put` | before | 2076.2 [2059.2–2115.6] | 2072.6 [2059.1–2074.6] | 2066.4 [2057.2–2073.5] | 0.995 | ~40 |
| `put` | after | 1674.1 [1666.8–1681.8] | 1692.1 [1680.4–1702.3] | 1699.3 [1670.1–1728.3] | 1.015 | **~10** |
| `delete` | before | 685.9 [673.4–693.6] | 678.4 [678.0–684.4] | 683.5 [680.5–686.6] | 0.996 | ~2.4 |
| `delete` | after | 1057.1 [1047.1–1075.9] | 1061.7 [1047.0–1067.6] | 1058.5 [1053.3–1068.8] | 1.001 | **~10** |
| `count-index` | before | 626.2 [606.8–638.5] | 620.3 [619.8–637.7] | 632.3 [619.1–638.4] | 1.010 | 1 |
| `count-index` | after | 685.8 [671.9–698.7] | 696.6 [692.1–723.4] | 708.7 [705.0–720.1] | 1.033 | 1 |
| `scan-index` | before | 788.4 [694.5–831.2] | 683.0 [681.6–688.0] | 672.5 [661.0–686.6] | 0.853 | ≤10 |
| `scan-index` | after | 1088.6 [1037.6–1133.6] | 1036.8 [992.6–1139.5] | 984.6 [940.4–1049.4] | 0.904 | ≤10 |
| **store** (agg) | before | 866.2 [819.0–881.3] | 808.3 [807.8–809.9] | 807.4 [802.4–819.5] | 0.932 | — |
| **store** (agg) | after | 1097.6 [1070.5–1115.9] | 1082.5 [1072.4–1131.1] | 1081.2 [1077.4–1091.9] | 0.985 | — |

**The inbox stays FLAT in `n` on every op, before and after** — it is still the sweep's own control,
and it still controls. What changed is the per-call batch, in both directions at once (`put` cheaper,
`delete` dearer), which is why only the totals above carry a claim.

⚠ **Inbox `scan-index` rose from 673–788 to 985–1089 µs/call and I cannot name the mechanism with this
instrument.** Candidates I did not separate: a scan now returns up to 10 *distinct messages* where it
used to return up to 10 rows of which several were copies of one message (different page/cache
behaviour), and a changed poller mix — inbox `store-calls` fell 6.8×, so the surviving calls are a
different sample, not the same one measured again. **On the record as open.** Its `4000/1000` stays
flat (0.853 → 0.904), so it is a level shift, not a new slope.

---

## ⚠ ONE PHASE MOVED THE WRONG WAY, AND IT IS ONE MILLISECOND FROM NOISE

`collect` at **n=1000**: before 4093 [3498–4623], after 4671 [4624–4683], **+14 %**. The bands are
technically non-overlapping — **by 1 ms** (before max 4623, after min 4624) — with n=3 a side, against
a before-spread of 1125 ms and an after-spread of 59 ms. `collect` at n=2000 (+1.8 %) and n=4000
(+3.1 %) both overlap freely. I report it as **a shift I cannot distinguish from the before side's own
variance**, not as a regression, and not as noise either. `stop` rose 1.5–2.8× on the medians but every
band overlaps (before 75–457, after 59–509) — that phase has always been noise-dominated.

Everything else: **no phase regresses.** `setup` **12449–12570 ms across all 18 runs** (constant, as
the stone required). `fill` linear (slope 4.273 → 4.060). `drain` `4000/1000` **4.447 → 4.425** —
inside the known 4.44–4.58 band at the low edge, i.e. very slightly flatter, and 5–6 % faster in
absolute terms at every `n` with non-overlapping bands. `arm` 6–8 ms throughout.

---

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ⛔ **fanout is complete** | ✅ **PASS.** `distinct = 4000 / 8000 / 16000 = n×m` and `dup = 0` on **all 18 runs** (9 before, 9 after), `vis-ms=1000` at every point. `seen-recorded = n×m`, `empty=1`, `visible=0`, `unacked=0`, subscriber `refused=0`, `fill-depth=[n/0]×4` everywhere. `rc=0` and zero stderr bytes on all 18. **Not one subscriber missed one message at any size** |
| 2 | ⛔ **the ack still follows the sends** | ✅ **PASS, and strictly stronger than before.** `Queue/send` to the sub queue is `sns-fanout.wat:476`; `Queue/ack` on the inbox is `:556` — send before ack, unchanged in direction. It moved **out of the per-subscriber fold**: previously sub *i*'s bucket was acked before sub *i+1* was written (safe only because a pair belonged to one sub); now **no ack happens until every subscriber has returned Accepted for the prefix.** A worker dying mid-expansion leaves the entry to expire and be re-processed; duplicates are absorbed by `seen` (`dup=0`, `seen-skipped` unchanged) |
| 3 | ★ **`Accepted c` is in MESSAGES** | ✅ **PASS.** `publish` no longer reads `nsubs` at all — `grep 'Record/nsubs' sns-fanout.wat` returns **one** line, `:220`, inside `topic-inbox-fail`, which only copies the field forward. `Accepted c` is the queue's own count of admitted **bodies**, one body per message. Gate `:user::unit-is-per-msg` at nsubs=3 → `rows=1;unit=per-msg`; `probe-a-batch-declares-how-many` at nsubs=2 → `10=Accepted(10);depth 0->10` (was `->20`). Live: inbox `accepted` = `n` on all nine post runs |
| 4 | ★ **the `rem`/`need` top-up is DELETED** | ✅ **PASS — removed, not unreached.** `grep -n 'rem\b\|need\|floor\|pairs' wat-scripts/topic/sns-fanout.wat` returns **only comment lines** (`:22`, `:122`, `:730` — an unrelated "60000 ms is the floor" — `:855`, `:1022`). No `:wat::i64::mod`, no second `Queue/send` to `"inbox"` in `publish`, no `ntop`/`need`/`tail` bindings. The 71-line block and its four `RecvOutcome` arms are gone from the file. ★ It was already unreachable in the circuit (`send-all` chunks at 64, so 40 pairs went as one all-or-nothing chunk and `rem` was always 0) — dead code that read as safety, which is precisely why it had to go rather than be left |
| 5 | ★ **the `"{i}|"` tag is gone** | ✅ **PASS.** `publish` formats `"{m}|{t0b}"`. `grep 'split' wat-scripts/topic/sns-fanout.wat` → **empty**: the `:wat::string::split`, the `:wat::edn::read` of `parts[0]`, the `rest` re-join and the whole bucketing fold are deleted. No subscriber index is written into a body and none is parsed out of one |
| 6 | ★★ **the inbox stops refusing** | ⚠ **REFUTED — reported, not rounded.** `refused` **602→91 / 1221→190 / 2502→390** (medians; bands non-overlapping at every `n`). **It did not reach 0.** `:cap 64` UNCHANGED, so the measurement stands on its own: 64 *messages* is a different object from 64 *pairs* **and the cap still binds**, ~6.4× less often (6.0–6.3 → 0.91–0.98 refusals per publish call). The consequence is nearly gone (`asleep` ÷36, `publish-attempts` ÷3.7, `fill` ÷6.2). ⚠ I cannot say whether the residue is saturation or bursts — that needs an inbox depth distribution this harness does not report. **The cap question is still open** |
| 7 | **inbox depth reads in messages** | ✅ **PASS, exactly.** inbox `accepted` **4000→1000, 8000→2000, 16000→4000** — `= n`, not `n×m`, on all nine post runs. `acks` equals it. Inbox `store-calls` 22041 → 3249 at n=4000, `delete-calls` **6591 → 401** |
| 8 | **the per-op counters still reconcile** | ✅ **PASS.** `put+delete+count+scan` calls and ns equal the `store-calls`/`store-ns` aggregate with **remainder 0 on 45/45 tier reports before and 45/45 after** |
| 9 | **no phase regresses** | ✅ **PASS**, with one shift named. `setup` 12449–12570 ms constant across all 18. `fill` linear and **6.2× faster** (slope 4.273→4.060). `drain` `4000/1000` **4.447 → 4.425**, inside the 4.44–4.58 band and 5–6 % faster absolute at every `n`, non-overlapping. `total` ×0.78 / ×0.64 / ×0.54. ⚠ `collect` at n=1000 +14 % with bands separated **by 1 ms** — reported above as indistinguishable from the before side's own 1125 ms spread |
| 10 | **blast radius** | ✅ **PASS on the functional file, with 5 extra files named.** `wat-scripts/topic/sns-fanout.wat` is the only file whose behaviour changed. **No `wat/`. No `src/`. No `circuit.wat`. No `mem.wat`. No store.** Also modified: `tests/services/probe_async_publish.rs` (a gate that asserted the OLD tier-1 unit — see below) and **four scratch-probe headers** whose prose the change made false (comment-only). Full `git diff --stat` below |
| 11 | **the corpus loads** | ✅ **PASS.** `every_wat_scripts_file_loads_on_the_current_runtime` **PASS [482.381s]**. All eight `scratch-pad` probes that `load-file!` `sns-fanout.wat` were additionally **run** individually against the changed runtime — all rc=0 |
| 12 | **the floor holds** | ✅ **PASS.** `Summary [ 482.388s] 5237 tests run: 5237 passed (7 slow), 22 skipped` — **0 FAIL, 0 TIMEOUT**, exit `0`, **no `ARM.txt`**, zero `FAIL`/`TRY`/`TIMEOUT`/`ABORT`/`SIGSEGV` lines in `clean.log`. Ran **once**, after the whole sweep. Nothing was re-run |

---

## STOPS

- **STOP-1** (`distinct ≠ n×m` or `dup ≠ 0` at any size → STOP) — **did not fire.** 18/18 runs exact.
- **STOP-2** (do not change the ack ordering) — **held, and the ordering is now stronger.** Send `:476`,
  ack `:556`. The ack left the per-subscriber loop entirely: it is issued once, after every subscriber
  write, over the longest prefix **every** subscriber accepted (`ok = min over subs of Accepted`, 0 for
  a sub whose send tore). A sub that refuses or tears drives `ok` to 0 and **nothing** in that tick is
  acked. ⛔ `nsubs = 0` seeds `ok = 0` deliberately — with nowhere to deliver there is nothing to
  consume, and the inbox entry must survive.
- ⚠ **STOP-3** (do not change `:cap 64`; if refusals do not drop, that is a finding) — **held on the cap,
  and the finding it protects FIRED.** The sweep's inbox is `circuit.wat:2178`, `:cap 64`, and
  `circuit.wat` is **not in the diff at all**. `:cap 64` appears 26 times across `wat-scripts/` (10 of
  them in `sns-fanout.wat`) and `git diff` contains **no `cap` line outside comment text**; `cap` is
  still a field on the Queue record. Refusals dropped 6.4× but **not to zero** — the full report is
  above. Because the cap was not touched, the number means what it says.
- **STOP-4** (worker cannot reach the subscriber addresses without new plumbing → STOP) — **did not fire.**
  The worker already held `subs` (dialled from durable `sub-addrs` in `:init`) and already looped
  `0..nsubs`. No new field, no new grant, no new surface. The `:ephemeral` `subs` vector and the
  process-locus grants were sufficient exactly as the DESIGN said.
- **STOP-5** (the top-up cannot be deleted because something depends on it → STOP) — **did not fire.**
  Nothing depended on it. It was already unreachable through the circuit.
- **STOP-6** (do not raise `:max-entries [msgs 10]`) — **held.** Unchanged; `probe-a-batch-declares-how-many`
  still prints `11=RequestTooManyEntries(11,10)`.
- **STOP-7** (on any red floor arm: capture whole, name the arm, do not re-run) — **no red arm.** The
  floor ran once, exit 0, no `ARM.txt`.

---

## THE EDIT SITES — the real count

**11 sites in `wat-scripts/topic/sns-fanout.wat`** (the sketch named 4):

1. file header — the shape paragraph: tier 1's unit is the MESSAGE, tier 2's is the pair
2. `publish` — the `nsubs` binding, **deleted** (the publisher's admission no longer mentions it)
3. `publish` — the `bodies` fold: nested `msgs × nsubs` → flat over `msgs`; `"{i}|{m}|{t0b}"` → `"{m}|{t0b}"`
4. `publish` — the `Accepted` arm: the whole `floor`/`rem`/`need` top-up **and its four `RecvOutcome`
   arms**, deleted (71 lines) → a direct `Accepted accepted`
5. worker `-tick` — `empty-bucket`/`empty-buckets`/`buckets` deleted; `items` (envelope-id, stamped-body)
   added. This is where the `split`, the `edn::read` of the index, the `rest` re-join and the bucketing
   fold all go
6. worker `-tick` — the fold head: accumulator `(inbox-peer, subs)` → `(subs, ok)`, `bucket` → `items`
7. worker `-tick` — the `Accepted nacc` arm: the in-loop ack removed, running minimum carried
8. worker `-tick` — **3×** failure-arm tuples `(Tuple inb ss')` → `(Tuple ss' 0)`
9. worker `-tick` — **2×** stale `"do not ack the bucket"` comments
10. worker `-tick` — the fold seed, `ack-ids`, and the ack **after** the fold (the Row-2 site)
11. gate `:user::unit-is-per-sub` → `:user::unit-is-per-msg`, predicate inverted `(= n 3)` → `(= n 1)`

**5 more files, named because the BRIEF's radius did not anticipate them:**

- ⚠ **`tests/services/probe_async_publish.rs`** — `unit_is_per_subscription` asserted
  `rows=3;unit=per-sub`: **the old tier-1 unit encoded as a floor guarantee.** This stone changes
  exactly that, so the gate had to be inverted, not deleted — renamed `unit_is_per_message`, asserting
  `rows=1;unit=per-msg`. It is now the instrument that detects the expansion creeping back into
  `Topic::publish`. The other 8 tests in that file are untouched and all pass.
- **4 `wat-scripts/scratch-pad/` probe headers, comment-only** — their prose asserted the deleted
  mechanism as current behaviour, which the scratch-pad rule forbids ("a scratch program that rots goes
  RED rather than becoming a graveyard that reads like live code" — the load gate catches rot, not lies):
  - `probe-a-message-is-fanned-once.wat` — its entire premise (6 pairs msg-major, a refused top-up,
    `Accepted 1 (floor)`, bodies `"{i}|p0"`) no longer exists. Retargeted: it now records that **the
    split case cannot occur**, printing `split=Accepted(2)` with both rows present.
  - `probe-a-batch-declares-how-many.wat`, `probe-the-topic-publishes-a-batch.wat` — `depth +10×nsubs`
    → `depth +10`.
  - `probe-the-server-manages-its-own-capacity.wat` — its `nsubs 7, publish 10` case used to offer
    70 pairs against cap 64 and cross the `n0 > cap` prefix branch; it now offers 10 and does not. The
    prefix branch is still exercised directly by the file's own `cap 10, send 15` case.

---

## WHAT THE NEXT STONE INHERITS

1. ⚠ **`:cap 64` still binds, and the cap question is not settled.** Row 6's prediction is refuted.
   Separating steady-state saturation from bursts needs an **inbox depth distribution over the fill** —
   the harness reports only terminal `visible`/`unacked` and a cumulative `refused`.
2. ★★ **`nsubs` is now INERT on the topic.** The only surviving read is `topic-inbox-fail` copying it
   forward. It is a durable field no decision consults — the same "dead state that reads as
   configuration" class this stone just removed from `publish`. Removing it touches **20 `:nsubs` sites**
   across `sns-fanout.wat`, `circuit.wat` and six `scratch-pad` probes, which is outside this stone's
   radius. **It is the obvious next stone**, and its gate is
   `grep -rn ':nsubs' --include=*.wat . | grep -v '^./docs/' | wc -l` → **0**.
3. **`drain` did not benefit and is now the whole cost.** `drain-store-calls` is unchanged
   (1451→1478 at n=4000) and `drain` fell only 5–6 %; `fill` fell 84 %. The producer side is done;
   the consumer side is untouched and now dominates `total` at every `n`.
4. ⚠ **Inbox `scan-index` per call rose ~45 % with an unnamed mechanism** (level shift only — slope
   0.853 → 0.904). Two unseparated candidates: a different page/cache profile now that a scan returns
   10 distinct messages rather than ~10 rows of ~2.5 messages, and a changed sample (inbox
   `store-calls` fell 6.8×, so the surviving calls are not the same population). Open.
5. **The two-tier retry granularity the DESIGN argued for is now actually realised.** Tier 1 unacked =
   "accepted, not yet expanded"; tier 2 unacked = "expanded, not yet delivered to THIS sub". Before
   this stone tier 1 held pairs, so both states lived in one tier and the index had to be smuggled
   through the payload to tell them apart.

---

## THE BOX

`ps -eo args | grep -E 'cargo|nextest|release/wat' | grep -v grep` was captured **before every one of
the eighteen runs** and every time showed exactly two lines, both idle MCP servers:

```
/home/john/.cargo/bin/wat --mcp
/home/john/.cargo/bin/wat --mcp
```

(36 captured lines = 18 runs × 2, all identical.) **No run ran beside another** — a single sequential
driver, 10 s between runs, one sweep at a time. `cut -d' ' -f1 /proc/loadavg` immediately before each
run: **0.33–5.93** (before), **1.43–3.43** (after). The interleaved ordering spreads within-sweep drift
across all three `n`. The targeted 16-test verification and the eight individual probe runs happened
**between** the sweeps, never during one. **The floor ran after all eighteen.**

⚠ **`wat-scripts/` is read from disk, not frozen into the binary**, so both sides ran on the **same**
`target/release/wat` build (`cargo build --release` at HEAD `c96d24ce9`, clean tree). No rebuild sits
between the baseline and the change. The baseline was measured on this box minutes before the edit and
reproduces the stone's quoted inbox figure at n=2000 (**1220/1221/1223** against the stone's 1209–1233).

---

## BLAST RADIUS

```
$ git status --porcelain
 M tests/services/probe_async_publish.rs
 M wat-scripts/scratch-pad/probe-a-batch-declares-how-many.wat
 M wat-scripts/scratch-pad/probe-a-message-is-fanned-once.wat
 M wat-scripts/scratch-pad/probe-the-server-manages-its-own-capacity.wat
 M wat-scripts/scratch-pad/probe-the-topic-publishes-a-batch.wat
 M wat-scripts/topic/sns-fanout.wat

$ git diff --stat
 tests/services/probe_async_publish.rs              |  14 +-
 .../probe-a-batch-declares-how-many.wat            |   3 +-
 .../scratch-pad/probe-a-message-is-fanned-once.wat |  16 +-
 .../probe-the-server-manages-its-own-capacity.wat  |   5 +-
 .../probe-the-topic-publishes-a-batch.wat          |   3 +-
 wat-scripts/topic/sns-fanout.wat                   | 316 +++++++++------------
 6 files changed, 156 insertions(+), 201 deletions(-)
```

**One functional file. Net −45 lines.** No `wat/`, no `src/`, no `circuit.wat`, no `mem.wat`, no store.
`./target/release/wat wat-scripts/topic/run.wat` still prints **`"3 3"`** — the thread/process
differential is unmoved. **Left uncommitted.**

The whole fanout, now in the worker and nowhere else:

```wat
;; publish — one body per MESSAGE, no index
bodies (:wat::core::foldl
         (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])
                          msg <- :wat::core::String]
           -> (:wat::core::Vector :- [:wat::core::String])
           (:wat::core::conj acc (:wat::core::format "{m}|{t0b}" :m msg :t0b t0b)))
         (:wat::core::Vector :- [:wat::core::String]) msgs)
...
((:queue::Queue::SendResponse::Accepted accepted)
  (:wat::service::Outcome::Continue s
    (:wat::core::Some (:demo::Topic::Reply::Publish
      (:demo::Topic::PublishResponse::Accepted accepted)))
    sends none-alarms))

;; worker — every item to every sub, then ONE ack over the common prefix
((:queue::Queue::SendResponse::Accepted nacc)
  (:wat::core::Tuple ss (:wat::core::if (:wat::i64::< nacc ok) nacc ok)))
...
inb2 (:wat::core::if (:wat::i64::<= ok 0)
       inbox
       (:wat::core::match
         (:queue::Queue/ack inbox
           (:queue::Queue::AckRequest :queue "inbox" :ids ack-ids))
         ...))
```

---

# ⭑ THE ORCHESTRATOR'S GRADING — my own runs, my own reads

Written after re-running the circuit myself on a quiet box and reading every load-bearing site on
the live source. **Nothing below is credited to the report.** Where a row is graded from a number,
that number came out of my terminal.

## What I re-ran

Three runs, `./target/release/wat wat-scripts/fanout/circuit.wat <n> 4 3 8192 true 1000`, box
verified quiet first (no `cargo`, no `nextest`, no stray `release/wat`).

| | n=1000 | n=2000 | n=4000 |
|---|---|---|---|
| `distinct` | **4000** | **8000** | **16000** |
| `dup` | **0** | **0** | **0** |
| inbox `accepted` | **1000** | **2000** | **4000** |
| inbox `refused` | **90** | **191** | **390** |
| `setup` | 12457 | 12477 | 12700 |
| `fill` | 1295 | 2583 | 5396 |
| `drain` | 1194 | 2506 | 5305 |

**Row 1 ✅** — `distinct = n×m` and `dup = 0` at every size, on my runs, not theirs.
**Row 7 ✅** — inbox `accepted` is exactly `n`. Tier 1 counts messages.
**Row 9 ✅** — drain 4000/1000 = **4.443**, inside the known 4.44–4.58 band. `fill` linear
(×1.995, ×2.089). `setup` constant 12457–12700.
**Row 8 ✅** — reconciliation exact, remainder **0** on both calls and nanoseconds, checked by
hand on three tiers: inbox n=1000 `201+101+270+350 = 922` and `340564369+105908137+187336327+342393311
= 976202144`; sub[0] n=1000 `648` / `952060493`; inbox n=4000 `3373` / `3579903637`.

**Rows 2, 4, 5 ✅ read on the live source, not from the diff summary.** `rem`/`need`/`ntop` return
**no matches at all**. `split` survives only inside a comment. The ack sits after the fan at `:556`
under a seven-line comment naming the property — and `ok` is now `min` over subscribers seeded at
`nitems`, with `nsubs = 0` seeding `0` so an entry with nowhere to go survives rather than being
dropped. **That is stronger than row 2 asked for**; previously sub *i* was acked before sub *i+1*
was written.

**Row 3 ✅** — the topic's `nsubs` has exactly one live read left, the field copy at `:220`.
Admission does not consult it.

**Row 12 ✅ from the log, never the report.** `.floor/2026-09-09T07-55-36Z/clean.log`:
`Summary [ 482.388s] 5237 tests run: 5237 passed (7 slow), 22 skipped`, **0** lines matching
`FAIL|TRY|TIMEOUT|ABORT|SIGSEGV`, no `ARM.txt`, corpus gate `PASS [ 482.381s]`.

## ⚠ ROW 6 IS REFUTED, AND I CAN NOW NAME THE MECHANISM THE SCORE LEFT OPEN

My runs reproduce the refutation: **90 / 191 / 390**, not 0. `:cap 64` untouched. The prediction
was mine and it was wrong.

The SCORE says: *"I cannot say whether the residue is saturation or bursts — that needs an inbox
depth distribution this harness does not report."* **The harness does report it.** Two identities
already on the same output line settle both halves:

**(a) There are zero partial accepts.** Reading `:fanout::publish-until-accepted!*`
(`circuit.wat:1342`): `retries` increments only on `c <= 0`; a partial accept (`0 < c < n`)
recurses with `attempt` reset and `retries` **unchanged**, but `att` incremented. So
`attempts = calls + retries + partials`, and

```
n=1000   190 = 100 + 90  + 0
n=2000   391 = 200 + 191 + 0
n=4000   790 = 400 + 390 + 0
```

**Partials = 0 at every size.** A 10-message batch against 64 slots is never partially admitted.

**(b) The refusals are shallow, and the depth does not grow with load.** `backoff-delay`
(`circuit.wat:1331`) draws uniform in `[1, min(100, 1 << attempt)]`, so a depth-0 retry sleeps
**exactly 1 ms** and the mean climbs 1.0 → 1.5 → 2.5 → 4.5 with consecutive refusals of the *same*
call. Observed mean sleep per retry:

```
108/90 = 1.200 ms     229/191 = 1.199 ms     468/390 = 1.200 ms
```

**Constant to three digits across a 4× range in n.** Three consecutive refusals alone would put the
mean at 1.67. So the retries are spread thin across many calls at depth 1–2, with no deep-backoff
tail. **The cap is being brushed, not saturated** — which is why `refused` scales linearly with
`publish-calls` and cannot climb much further: it is already near its ceiling of about one refusal
per call.

★ The instrument was printed beside the number the whole time. This is the failure mode this
campaign has hit repeatedly, and here it cost an open question that was already answered on disk.

### ⛔ And one inference of my own that a grep killed

I first read `refused == full-retries` (90/90, 191/191, 390/390) as *proof* of zero partial accepts.
It is not. `sqs.wat:489` increments `sends-refused` **only** on the arm replying `Accepted 0` — the
same event `full-retries` counts. The equality is a **tautology** and carries no information. The
proof is identity (a) above, which is independent. One `sed` on the increment site turned a
would-be finding back into arithmetic.

## ⛔ ROW 10 IS MY DEFECT, NOT THE EXECUTOR'S

The executor changed five files outside the radius I wrote, named every one, and was right to.

`tests/services/probe_async_publish.rs` held `unit_is_per_subscription`, asserting *"one publish to
N=3 must write 3 rows, not 1"* — **the removed defect encoded as a floor guarantee.** Leaving it
reddens the floor; changing it breaks my radius. **Rows 10 and 12 contradict each other**, and the
executor could not satisfy both. Inverting it to `unit_is_per_message` (`rows=1`) was correct: the
test now detects the expansion creeping back, which is worth more than the row I wrote.

The four `scratch-pad` probes are **comment-only** — verified: every changed line in all four begins
`;;`. Their prose asserted the deleted mechanism as current, so leaving them would have seeded
exactly the graveyard-that-reads-as-live the campaign keeps pulling out.

**The root:** I named a path list where I should have named a property. Row 10 should have read
*"no change to `wat/`, `src/`, `mem.wat`, or the store; anything the floor or the compiler forces
is in radius and must be named."* The floor **is** the census. A hand-written list of sites is a
proxy for it, and this is the same substitution that has cost this campaign repeatedly.

## Grade

`1 ✅ · 2 ✅✅ · 3 ✅ · 4 ✅ · 5 ✅ · 6 ⚠ REFUTED — and the refutation is the deliverable · 7 ✅ ·
8 ✅ · 9 ✅ · 10 ⛔ my defect · 11 ✅ · 12 ✅`

**Row 6 firing is this stone's most valuable result.** It was written to be falsifiable with the cap
frozen precisely so it *could* fail, and it did — killing my claim that 64 messages would not bind,
and replacing it with a measured shape: shallow, linear, near its ceiling.
