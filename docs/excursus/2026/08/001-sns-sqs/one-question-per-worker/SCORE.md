# SCORE — one question per worker

Struck on `sns-sqs`, from `f93256921`. Everything uncommitted. `wat-scripts/fanout/circuit.wat` only.

**Result in one line:** `:fanout::worker`'s disrupt tallies now ride its `:stop` projection, `collect`
asks each of the 12 workers **once** instead of twice, `rt-worker` fell **36 → 24 in every one of 8
runs**, and `collect`'s before and after bands are **fully separated in both configurations** —
n=2000 `[4639…5249] → [2620…3117]`, n=20 `[3528…4035] → [1339…2163]`. **Row 8 HELD:** the removed
question is worth **166 ms/worker (n=2000)** and **180 ms/worker (n=20)**, against the builder's
half-of-338 = **169 ms**.

**STOP-2 FIRED.** `collect` was **not** the only caller of `:fanout::sum-disrupts` — `run-with`'s
`drained-stalled` diagnostic calls it too, on **live** workers, on a path that then raises. So
`sum-disrupts` is **not deleted**; it survives as that failure-only live read, at zero happy-path cost.
**Row 3 is therefore PARTIAL** and the DESIGN's one contract decision **is not taken**: the live read
the DESIGN offered to give up is still there.

**Floor, verbatim** (`.floor/2026-09-10T22-33-35Z/clean.log`):

```
     Summary [ 485.817s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

0 lines matching `^ *(FAIL|TRY|SIGSEGV|ABORT|TIMEOUT)`, no `ARM.txt`.

**STOPs fired: STOP-2 only.** STOP-1 did not fire — a 9-field `defrecord` in the surface's
`:messages` carried outcomes *and* every counter, and it compiled first try. STOP-3 did not fire —
`collect` fell, with separated bands. STOP-4 did not fire — no red anywhere.

---

## ⭑⭑ STOP-2 FIRED — there were TWO callers, and the second one is a LIVE read

The BRIEF said *"verify from the code, not from my sentence."* Verified. `grep -rn "sum-disrupts"`
over `wat-scripts/ wat-tests/ wat/ src/` (non-`.md`) before the strike returned **two call sites, both
inside `:fanout::run-with`**:

| site (pre-strike line) | phase | when it runs |
|---|---|---|
| `circuit.wat:2900` — `dpair (:fanout::sum-disrupts wpeers)` | `collect` | **every run** — this is the one the stone removes |
| `circuit.wat:2866` — `(:wat::core::let [dp (:fanout::sum-disrupts wpeers)] …)` | `drain` | **only when `drain-err ≠ ""`** — inside `_drain`'s `require!` |

The second site is the `drained-stalled` message's tail. It is the only way `check-exhausted` /
`mark-exhausted` / `ack-retries` / `ack-exhausted` are ever seen on a stalled run, because a raise
means the phase line is never printed.

### Why I kept it rather than deleting it

⛔ **It cannot come off `stop`.** At that point nothing is stopped — the whole purpose is to say *why
the running system stopped making progress*. Routing it through the new `collect-stop` would:

1. **stop all twelve workers mid-diagnosis**, and
2. trade a **tolerant** fold for a **raising** one. `sum-disrupts` falls back to `acc` on every
   `RecvOutcome` arm (`Lost` / `Stopped` / `Closed` / `TimedOut`); the generated `worker/stop`
   **raises** on all three of `Lost`/`Closed`/`Stopped` (`wat/service.wat:2946-2968`). On the one path
   built for diagnosing a wedged system, that swaps the stall verdict for `defservice stop: …`.

So the fold stays, **narrowed** from six counters to the four that diagnostic prints (`hits` and
`ack-calls` were only ever wanted by `collect`, which now lifts them off `stop`). It costs the happy
path nothing: `drain-err` is `""` there and the `if` never evaluates it.

**Both stall gates prove the diagnostic survived byte-for-byte** — see row 6.

### ⚠ What this does to the DESIGN's "one contract decision"

The DESIGN said: *"the stop projection must carry the counters WITHOUT the harness losing the ability
to read them on a live worker … `collect` is the only caller … if any future caller needs disrupts
from a live worker, this stone has taken that away."*

**It has not, and it must not.** A future caller already exists — it is the stall diagnostic — and it
is the one class of caller for which a live read is not a luxury. The `disrupts` feature, its impl, and
`sum-disrupts` all remain. **The DESIGN's trade was not available to make**, and the cost it feared
paying for keeping the live read (*"it costs a crossing and waits out the worker's poll"*) is not paid,
because the caller is on a path that only runs when the run has already failed.

---

## Row-by-row

| # | what must be true | verdict |
|---|---|---|
| 1 | ⛔ correctness untouched | ✅ **PASS** — `distinct=8000 dup=0` in 4/4 after runs and 4/4 before; `empty=1`; no raise, `rc 0` in all 16 timing runs. Inbox tier `accepted=2000 refused=0` (`tier=inbox` on every after line). n=20's summary line is **byte-identical across all 8 runs, both sides.** |
| 2 | ⛔ every summary field survives | ✅ **PASS** — see the field table below. All five printed on every line; n=20 identical byte-for-byte; the failure-path diagnostic's four identical byte-for-byte; and the chaos gate carries them **non-zero** through the new path (`mark-exhausted=23`, `ack-exhausted=8`). |
| 3 | ★ one question per worker | ⚠ **PARTIAL — STOP-2** — `collect` asks once; `rt-worker` 36 → 24 proves it. But **`sum-disrupts` is NOT gone**, and the file therefore still holds two folds over the workers. They are **mutually exclusive** (`drain-err == ""` selects one), so exactly one fold runs per run — but row 3 as written said *gone*, and it is not. |
| 4 | ★ `rt-worker` falls by 12 | ✅ **PASS, exactly** — 36 → 24 in **8/8** runs (4 pairs × 2 configs), zero variance on either side. |
| 5 | ★ `collect` falls materially | ✅ **PASS** — bands fully separated in both configurations, gaps of 1522 ms (n=2000) and 1365 ms (n=20). |
| 6 | the stall gates still fire | ✅ **PASS** — both, both sides, `rc 2`. Messages below. |
| 7 | chaos, corpus, floor | ✅ **PASS** — `Summary [ 31.044s] 7 tests run: 7 passed, 5252 skipped`; `PASS [ 485.811s] (5237/5237) wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime`; `Summary [ 485.817s] 5237 tests run: 5237 passed (7 slow), 22 skipped`. |
| 8 | ★★ removing one of two questions removes about half the per-worker term | ✅ **HELD** — 166 and 180 ms/worker removed against a predicted 169. See below. |
| 9 | ⚠ no throughput win; no "worker is now answerable" | ✅ **stated**, below, and neither is claimed. |

---

## ★★ Row 8 — the builder's attribution HELD

The claim under test: *`collect`'s ~338 ms/worker at the shipped 250 ms poll is **two** questions each
waiting out ~half a poll, so removing one should remove about half of it.* Half of 338 = **169 ms**.

| config | `collect` before (mean) | `collect` after (mean) | Δ | ÷ 12 workers |
|---|---|---|---|---|
| `2000 4 3 8192 true 1000` | 4885.5 | 2893.0 | **−1992.5 ms** | **−166.0 ms/worker** |
| `20 4 3 8192 true 1000` | 3781.0 | 1616.2 | **−2164.8 ms** | **−180.4 ms/worker** |

**166 and 180 against a predicted 169.** Two independent configurations, four interleaved pairs each,
and the bands do not touch:

| config | before band | after band | gap |
|---|---|---|---|
| n=2000 | **4639 … 5249** | **2620 … 3117** | 1522 ms |
| n=20 | **3528 … 4035** | **1339 … 2163** | 1365 ms |

★ **A third, independent consistency check the measurement volunteered.** At n=20 there are only 80
outcomes, so `collect`'s record fold is negligible and what is left is nearly pure round-trip:
after-`collect` = 1616 ms ÷ 12 = **135 ms/worker for ONE question** — which is the
`where-the-time-actually-goes` FINDING's own ~150 ms-per-question figure, arrived at from the other
direction. At n=2000 the residual is 241 ms/worker, and the difference between the two is the 8000-record
fold + `empty-flags` the DESIGN put out of scope at 36–112 ms.

⚠ **I name a percentage nowhere and do not need to.** The row asked for direction with separated bands
and per-worker arithmetic against 169 ms; that is what is above. The two configurations disagree by
14 ms/worker (166 vs 180) and **I do not have a mechanism for that difference** — 4 runs a side cannot
resolve it, and I am not going to invent one.

---

## The interleaved before/after table — every phase, plus `rt-worker` and `rt-total`

**Method.** 4 pairs × 2 configs = **16 runs**, strictly interleaved
`before-n2000 → before-n20 → after-n2000 → after-n20`, repeated. The **binary is byte-identical across
all 16 runs** (`target/release/wat`, built once, `cargo build --release` a no-op at start) — the only
thing that changes is which copy of `circuit.wat` is on disk, `cp`'d in before each run. Every run
through `./scripts/capped.sh --limit 8g`. Box verified quiet before the batch
(`ps -eo args | grep -E 'cargo|nextest|release/wat'` → only the two idle `wat --mcp` servers;
`uptime` load 1.15 decaying from the smoke runs). Nothing else ran during the batch.

### `2000 4 3 8192 true 1000`

| ms | r1 | r2 | r3 | r4 | min | max | mean |
|---|---|---|---|---|---|---|---|
| `setup` before | 12503 | 12497 | 12468 | 12444 | 12444 | 12503 | 12478 |
| `setup` after | 12420 | 12528 | 12420 | 12415 | 12415 | 12528 | 12446 |
| `fill` before | 2626 | 2637 | 2625 | 2639 | 2625 | 2639 | 2632 |
| `fill` after | 2666 | 2649 | 2624 | 2612 | 2612 | 2666 | 2638 |
| `arm` before | 20 | 17 | 18 | 35 | 17 | 35 | 22.5 |
| `arm` after | 38 | 17 | 18 | 17 | 17 | 38 | 22.5 |
| `drain` before | 2442 | 2426 | 2456 | 2455 | 2426 | 2456 | 2445 |
| `drain` after | 2445 | 2458 | 2407 | 2427 | 2407 | 2458 | 2434 |
| **`collect` before** | 4639 | 5249 | 4680 | 4974 | **4639** | **5249** | **4885.5** |
| **`collect` after** | 3031 | 3117 | 2804 | 2620 | **2620** | **3117** | **2893.0** |
| `stop` before | 538 | 688 | 428 | 409 | 409 | 688 | 515.8 |
| `stop` after | 425 | 397 | 528 | 390 | 390 | 528 | 435.0 |
| `total` before | 22770 | 23517 | 22677 | 22959 | 22677 | 23517 | 22981 |
| `total` after | 21027 | 21169 | 20803 | 20483 | 20483 | 21169 | 20871 |
| `fill-busy-ms` before | 618 | 622 | 619 | 616 | 616 | 622 | 618.8 |
| `fill-busy-ms` after | 623 | 615 | 613 | 615 | 613 | 623 | 616.5 |
| `arm-busy-ms` before | 15 | 10 | 12 | 21 | 10 | 21 | 14.5 |
| `arm-busy-ms` after | 19 | 10 | 12 | 10 | 10 | 19 | 12.8 |
| `drain-busy-ms` before | 1488 | 1497 | 1497 | 1493 | 1488 | 1497 | 1494 |
| `drain-busy-ms` after | 1490 | 1507 | 1479 | 1477 | 1477 | 1507 | 1488 |
| `collect-busy-ms` before | 164 | 216 | 204 | 255 | 164 | 255 | 209.8 |
| `collect-busy-ms` after | 90 | 111 | 99 | 71 | 71 | 111 | 92.8 |
| `stop-busy-ms` before | 5 | 5 | 6 | 6 | 5 | 6 | 5.5 |
| `stop-busy-ms` after | 6 | 5 | 5 | 5 | 5 | 6 | 5.2 |
| **`rt-worker` before** | 36 | 36 | 36 | 36 | **36** | **36** | **36.0** |
| **`rt-worker` after** | 24 | 24 | 24 | 24 | **24** | **24** | **24.0** |
| `rt-store` before | 6612 | 6682 | 6654 | 6691 | 6612 | 6691 | 6660 |
| `rt-store` after | 6384 | 6495 | 6390 | 6425 | 6384 | 6495 | 6424 |
| `rt-queue` before | 2150 | 2163 | 2158 | 2172 | 2150 | 2172 | 2161 |
| `rt-queue` after | 2055 | 2063 | 2052 | 2044 | 2044 | 2063 | 2054 |
| `rt-q-recv` before | 1304 | 1326 | 1317 | 1327 | 1304 | 1327 | 1318 |
| `rt-q-recv` after | 1210 | 1214 | 1202 | 1199 | 1199 | 1214 | 1206 |
| `rt-q-ack` before | 813 | 804 | 808 | 812 | 804 | 813 | 809.2 |
| `rt-q-ack` after | 812 | 816 | 817 | 812 | 812 | 817 | 814.2 |
| `rt-q-stats` before | 33 | 33 | 33 | 33 | 33 | 33 | 33 |
| `rt-q-stats` after | 33 | 33 | 33 | 33 | 33 | 33 | 33 |
| `rt-seen` before | 1601 | 1601 | 1601 | 1601 | 1601 | 1601 | 1601 |
| `rt-seen` after | 1601 | 1603 | 1603 | 1601 | 1601 | 1603 | 1602 |
| `rt-topic` before | 391 | 393 | 391 | 391 | 391 | 393 | 391.5 |
| `rt-topic` after | 391 | 391 | 393 | 391 | 391 | 393 | 391.5 |
| `rt-tw` before / after | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| `rt-pub` before / after | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| `rt-poll` before | 353 | 363 | 358 | 358 | 353 | 363 | 358.0 |
| `rt-poll` after | 358 | 358 | 353 | 353 | 353 | 358 | 355.5 |
| **`rt-total` before** | 11150 | 11245 | 11205 | 11256 | 11150 | 11256 | **11214** |
| **`rt-total` after** | 10820 | 10941 | 10822 | 10845 | 10820 | 10941 | **10857** |

### `20 4 3 8192 true 1000`

| ms | r1 | r2 | r3 | r4 | min | max | mean |
|---|---|---|---|---|---|---|---|
| `setup` before | 12566 | 12454 | 12444 | 12466 | 12444 | 12566 | 12483 |
| `setup` after | 12469 | 12484 | 12506 | 12484 | 12469 | 12506 | 12486 |
| `fill` before | 71 | 78 | 73 | 73 | 71 | 78 | 73.8 |
| `fill` after | 72 | 72 | 73 | 71 | 71 | 73 | 72.0 |
| `arm` before | 16 | 15 | 15 | 15 | 15 | 16 | 15.2 |
| `arm` after | 16 | 20 | 16 | 15 | 15 | 20 | 16.8 |
| `drain` before | 31 | 31 | 36 | 39 | 31 | 39 | 34.2 |
| `drain` after | 41 | 36 | 32 | 31 | 31 | 41 | 35.0 |
| **`collect` before** | 3528 | 3808 | 4035 | 3753 | **3528** | **4035** | **3781.0** |
| **`collect` after** | 1586 | 1339 | 2163 | 1377 | **1339** | **2163** | **1616.2** |
| `stop` before | 604 | 596 | 669 | 667 | 596 | 669 | 634.0 |
| `stop` after | 645 | 636 | 586 | 600 | 586 | 645 | 616.8 |
| `total` before | 16818 | 16985 | 17274 | 17015 | 16818 | 17274 | 17023 |
| `total` after | 14831 | 14591 | 15377 | 14580 | 14580 | 15377 | 14845 |
| `fill-busy-ms` before | 12 | 13 | 12 | 13 | 12 | 13 | 12.5 |
| `fill-busy-ms` after | 12 | 12 | 12 | 12 | 12 | 12 | 12.0 |
| `arm-busy-ms` before | 10 | 10 | 10 | 11 | 10 | 11 | 10.2 |
| `arm-busy-ms` after | 13 | 13 | 10 | 9 | 9 | 13 | 11.2 |
| `drain-busy-ms` before | 7 | 7 | 9 | 9 | 7 | 9 | 8.0 |
| `drain-busy-ms` after | 8 | 7 | 7 | 8 | 7 | 8 | 7.5 |
| `collect-busy-ms` before | 168 | 208 | 214 | 168 | 168 | 214 | 189.5 |
| `collect-busy-ms` after | 48 | 45 | 69 | 47 | 45 | 69 | 52.2 |
| `stop-busy-ms` before | 2 | 6 | 3 | 2 | 2 | 6 | 3.2 |
| `stop-busy-ms` after | 5 | 1 | 2 | 2 | 1 | 5 | 2.5 |
| **`rt-worker` before** | 36 | 36 | 36 | 36 | **36** | **36** | **36.0** |
| **`rt-worker` after** | 24 | 24 | 24 | 24 | **24** | **24** | **24.0** |
| `rt-store` before | 624 | 646 | 703 | 673 | 624 | 703 | 661.5 |
| `rt-store` after | 408 | 394 | 449 | 384 | 384 | 449 | 408.8 |
| `rt-queue` before | 299 | 307 | 324 | 306 | 299 | 324 | 309.0 |
| `rt-queue` after | 191 | 188 | 210 | 182 | 182 | 210 | 192.8 |
| `rt-q-recv` before | 258 | 266 | 283 | 265 | 258 | 283 | 268.0 |
| `rt-q-recv` after | 150 | 147 | 169 | 141 | 141 | 169 | 151.8 |
| `rt-q-ack` before / after | 8 | 8 | 8 | 8 | 8 | 8 | 8 |
| `rt-q-stats` before / after | 33 | 33 | 33 | 33 | 33 | 33 | 33 |
| `rt-seen` before / after | 17 | 17 | 17 | 17 | 17 | 17 | 17 |
| `rt-topic` before / after | 4 | 4 | 4 | 4 | 4 | 4 | 4 |
| `rt-tw` before / after | 6 | 6 | 6 | 6 | 6 | 6 | 6 |
| `rt-pub` before / after | 1 | 1 | 1 | 1 | 1 | 1 | 1 |
| `rt-poll` before / after | 18 | 18 | 18 | 18 | 18 | 18 | 18 |
| **`rt-total` before** | 1005 | 1035 | 1109 | 1061 | 1005 | 1109 | **1052.5** |
| **`rt-total` after** | 669 | 652 | 729 | 636 | 636 | 729 | **671.5** |

### ⚠ `rt-total` fell by 357 and 381, not by 12 — and only 12 of that is this stone

`rt-worker` is the term the stone bought: **−12, exactly, 8/8 runs.** The rest of the `rt-total` drop
is `rt-store` (−236 / −253) and `rt-q-recv` (−112 / −116), and those are **server-side counters that
keep climbing while `collect` is open**: the workers keep issuing `Queue/receive` and the queues keep
scanning the store for the whole phase. A `collect` that is 2 s shorter is 2 s less idle polling.

⛔ **So the honest attribution is: 12 crossings removed by design, ~345 fewer crossings incurred
because a phase got shorter.** The second number is real and it is on the line, but it is a
*consequence* of the first, not a second win, and it would vanish if the workers stopped polling
between the drain and the stop. **It is not evidence for the stone; it is evidence that the phase
shortened.**

### One verbatim phase line each

**after** (`after-n2000-p1`):

```
"setup=12420;fill=2666;arm=38;drain=2445;collect=3031;stop=425;fill-depth=[2000/0][2000/0][2000/0][2000/0];fill-excess=0;fill-stale-max=0;qticks=0;topic-ticks=66;disrupts=0;check-exhausted=0;mark-exhausted=0;ack-retries=12;ack-exhausted=3;seen-recorded=8000;seen-skipped=0;publish-calls=200;full-retries=189;inbox-lost=0;inbox-closed=0;inbox-timedout=0;asleep=228;publish-attempts=389;poll-calls=340;drain-stale-max=0;store-calls=4642;store-ms=7140;drain-store-calls=739;drain-store-ms=1224;fill-busy-ms=623;arm-busy-ms=19;drain-busy-ms=1490;collect-busy-ms=90;stop-busy-ms=6;rt-store=6384;rt-queue=2055;rt-q-recv=1210;rt-q-ack=812;rt-q-stats=33;rt-seen=1601;rt-worker=24;rt-topic=391;rt-tw=6;rt-pub=1;rt-poll=358;rt-total=10820;rt-unknown=topic-inbox-send+tw-sub-send+tw-inbox-ack;rt-unknown-max=1675;total=21027
```

**before** (`before-n2000-p1`):

```
"setup=12503;fill=2626;arm=20;drain=2442;collect=4639;stop=538;fill-depth=[2000/0][2000/0][2000/0][2000/0];fill-excess=0;fill-stale-max=0;qticks=0;topic-ticks=71;disrupts=0;check-exhausted=0;mark-exhausted=0;ack-retries=13;ack-exhausted=3;seen-recorded=8000;seen-skipped=0;publish-calls=200;full-retries=189;inbox-lost=0;inbox-closed=0;inbox-timedout=0;asleep=229;publish-attempts=389;poll-calls=335;drain-stale-max=0;store-calls=4637;store-ms=7096;drain-store-calls=737;drain-store-ms=1221;fill-busy-ms=618;arm-busy-ms=15;drain-busy-ms=1488;collect-busy-ms=164;stop-busy-ms=5;rt-store=6612;rt-queue=2150;rt-q-recv=1304;rt-q-ack=813;rt-q-stats=33;rt-seen=1601;rt-worker=36;rt-topic=391;rt-tw=6;rt-pub=1;rt-poll=353;rt-total=11150;rt-unknown=topic-inbox-send+tw-sub-send+tw-inbox-ack;rt-unknown-max=1775;total=22770
```

---

## Row 2 — proof that every summary field survived, with its values

Four independent proofs, because the CLI cannot reach `disrupt-rate` (`:user::main` pins `rate 0`), so
the timing configs alone can only prove the zeros.

### 1. The n=20 summary line is byte-identical across all 8 runs, both sides

```
"n=20;m=4;j=3;total=80;distinct=80;dup=0;workers=8;empty=1;seen-recorded=80;seen-skipped=0;check-exhausted=0;mark-exhausted=0;ack-retries=0;ack-exhausted=0"
```

before ×4, after ×4 — the same 154 bytes. `disrupts=0;check-exhausted=0;mark-exhausted=0;ack-retries=0;ack-exhausted=0` on all 8 phase lines.

### 2. The n=2000 configuration — the two non-deterministic fields stay in band

| field | before (4 runs) | after (4 runs) |
|---|---|---|
| `disrupts` | 0 0 0 0 | 0 0 0 0 |
| `check-exhausted` | 0 0 0 0 | 0 0 0 0 |
| `mark-exhausted` | 0 0 0 0 | 0 0 0 0 |
| `ack-retries` | 13 4 8 12 | 12 15 16 12 |
| `ack-exhausted` | 3 1 2 3 | 3 3 4 3 |
| `distinct` / `dup` | 8000 / 0 ×4 | 8000 / 0 ×4 |

⚠ `ack-retries` / `ack-exhausted` are **not deterministic on either side** — a retry happens when the
queue actor is busy at the moment a worker acks, across 12 concurrent processes. Before spans 4–13,
after 12–16. **I do not claim these are unchanged and I do not claim they moved**; 4 runs a side cannot
separate 9.2 from 13.8 for a quantity whose before-range is already 3.3×. What is proven is that the
fields are **printed, non-negative, and of the same order** — and `distinct=8000 dup=0` holds on all 8.

### 3. ★ The chaos gate carries them NON-ZERO through the new path

`scripts/floor.sh --run-ignored ignored-only --success-output immediate -E 'test(probe_arc278_sane_circuit)'`,
run on **both** copies of the file. Both: `Summary … 7 tests run: 7 passed, 5252 skipped`.

| cell | before | after |
|---|---|---|
| `R2 BEFORE-WRITE` | `mark-exhausted=22;ack-retries=0;ack-exhausted=8` | `mark-exhausted=22;ack-retries=0;ack-exhausted=9` |
| `R2 AFTER-WRITE` | `mark-exhausted=21;ack-retries=0;ack-exhausted=8` | `mark-exhausted=23;ack-retries=0;ack-exhausted=8` |
| `DROP-CHECK-TINY` | `distinct=100;dup=0;seen-skipped=0` | `distinct=100;dup=0;seen-skipped=10` |

**`mark-exhausted` in the low twenties and `ack-exhausted` at 8–9 on both sides.** This is the field
that matters most for row 2: it is a counter incremented deep in the worker's tick handler, written
into the durable `Record`, and it now reaches the harness by a **completely different wire** — the
`:stop` projection instead of a `disrupts` reply — and it arrives with the same magnitude.
`total=8000;distinct=8000;dup=0` in every cell, both sides.

### 4. The failure path's four counters are byte-identical

See row 6.

### ⚠ One observation I am not calling a finding: `seen-skipped`

`seen-skipped` was 0 in 4/4 before runs at n=2000 and `0, 10, 10, 0` in the after runs. **This is not
attributable to the stone**: `seen-skipped` is recorded by the `seen` service during the **drain**,
which is upstream of `collect` and completely untouched here — a redelivery on visibility expiry that
the consumer then absorbed. `distinct=8000 dup=0` in all 8 runs, which is the property that matters.
n=8 and I have no mechanism; **ship the absence.**

---

## Row 6 — both stall gates, both sides, and the diagnostic proved intact

`5 1 0 32 false 0` → **`rc 2`**, both sides. The message, with the four counters `sum-disrupts`
supplies, **byte-identical apart from the elapsed wall-clock and the moved line number**:

```
before: "drained-stalled: no delivery progress in 600 polls; last=[0/0] outbox=5 acks=0 polls=601 elapsed=13140;check-exhausted=0;mark-exhausted=0;ack-retries=0;ack-exhausted=0"  (circuit.wat:2863)
after:  "drained-stalled: no delivery progress in 600 polls; last=[0/0] outbox=5 acks=0 polls=601 elapsed=12976;check-exhausted=0;mark-exhausted=0;ack-retries=0;ack-exhausted=0"  (circuit.wat:2931)
```

★ **This is the row that justifies STOP-2's resolution.** Had `sum-disrupts` been deleted, that tail
would have had to come off `collect-stop` — stopping twelve workers to describe a system that has
already stopped, and raising a `defservice stop:` error instead of the verdict if any of them were the
thing that was wedged.

`50 2 2 32 true 0` → **`rc 2`**, both sides:

```
before: "filled-stalled: no arrival progress in 600 polls; last=[31/0][31/0] outbox=19 want=50 arrived=62 polls=602 elapsed=16066"
after:  "filled-stalled: no arrival progress in 600 polls; last=[31/0][31/0] outbox=19 want=50 arrived=62 polls=602 elapsed=16906"
```

Identical but for elapsed — and `last`, `outbox`, `want`, `arrived`, `polls` all match exactly.

---

## The edit — the real site count

**One file: `wat-scripts/fanout/circuit.wat`.** `git status --porcelain` = ` M wat-scripts/fanout/circuit.wat`
(plus this SCORE). **Nothing was forced elsewhere** — no `src/`, no other `.wat`, no test edit, no
stdlib change. The compiler was the census and it named every site on the first build.

**12 diff hunks: 9 code sites, 3 comment-only.** +159 / −77 lines (comments included).

| # | after-line | what | kind |
|---|---|---|---|
| 1 | `:282` | `(:wat::core::defrecord :fanout::WorkerFinal …)` — 9 fields, added to `:fanout::Worker`'s `:messages` | **new code** |
| 2 | `:296` | the `ack-calls`-rides-`disrupts` comment → rides `:stop` | comment |
| 3 | `:385` | `:stop` projection: `PersistentVector[Outcome]` → `:fanout::WorkerFinal` | **code** |
| 4 | `:2408` | `sum-disrupts` narrowed 6 counters → 4; `((hits,aks),(ce,me),(ar,ae))` → `((ce,me),(ar,ae))` | **code** |
| 5 | `:2437` | `(:wat::core::defrecord :fanout::Collected …)` — the harness-side accumulator, 8 fields | **new code** |
| 6 | `:2454` | `collect-stop` returns `:fanout::Collected`, folding tallies as well as outcomes | **code** |
| 7 | `:2940` | the `drained-stalled` diagnostic's accessors re-indexed for the narrowed tuple | **code** |
| 8 | `:2990` | `collect`: `dpair` + `worker-disrupt-rts` **deleted**; `dhits`/`wack`/`ce`/`me`/`ars`/`aes` read off `collected` | **code** |
| 9 | `:3065` | budget contract comment: `ack-calls` on the `stop` reply | comment |
| 10 | `:3086` | budget INSTANTS comment: worker terms are as of `collect-stop` | comment |
| 11 | `:3119` | `rt-worker` loses its `worker-disrupt-rts` term | **code** |
| 12 | `:3518`, `:3613`, `:3670` | three `:user::` fixtures unwrap `WorkerFinal/outcomes` (4 `worker/stop` calls) | **code** |

⭑ **STOP-1 did not fire, and the trap-door was avoided by heeding it.** Nine values had to leave the
worker; `Tuple` has no fourth accessor (`wat/core.wat:1737`), so both new types are `defrecord`s.
`:fanout::WorkerFinal` lives in the surface's `:messages` because the projection runs in the **forked
child** and is deserialised in the parent — the same reason `:fanout::Outcome` is there
(`wat-tests/service-stop-resp.wat` is the exemplar; it projects to an `i64`, this projects to a record
containing a vector of records, and the process tier deserialised it without a change to `src/`).
`:fanout::Collected` never crosses a boundary, so it is a plain top-level `defrecord`.

⛔ **Nothing prints from a handler or from the projection.** Every value out of a worker rides the
`stop` reply. The one thing I wanted mid-strike — to see a per-worker tally — I got by widening the
reply, not by a `println`.

---

## ⚠ Row 9 — what is NOT claimed

⚠ **This does not make a parked worker answerable.** `:fanout::worker` still calls `Queue/receive`
with a 250 ms `Wait::UpTo` from inside its tick handler, and for up to that long it is still deaf to
`disrupts`, to `stop`, and to everything else. **The stone reduces how often we ask, not what asking
costs.** The residual measured here says so directly: after-`collect` at n=20 is still **135 ms per
worker** for the one remaining question. The real fix is the `asks` gap named at `a4f2d7f7b` — a fifth
`asks` field on the outcome, a third superset source, a select set admitting a dialed peer, **all
reaching `src/`** — and that is an arc and the builder's to open.

⚠ **There is no throughput win here and the numbers do not offer one.** `distinct` is 8000 before and
after; `dup` is 0 before and after; `fill` (2632 → 2638) and `drain` (2445 → 2434) are unchanged
inside noise; `drain-busy-ms` is 1494 → 1488. The `total` drop (22981 → 20871) is `collect` and
nothing else, and `collect` is process-lifecycle and observability — the 12 stop round-trips, the
8000-record fold, `empty-flags`. **Twelve fewer crossings and a shorter shutdown; not a faster system.**

⚠ **And `collect` is not fixed.** After the strike it is still **2893 ms** at n=2000 with
`collect-busy-ms` at **93** — the subscriber queues do ~3 % of it as work. The remaining bulk is the
one question × 12 workers plus the record fold the DESIGN put out of scope.

---

## What I would look at next — pointers, not measurements

1. ⭑ **`:fanout::worker` waits 250 ms; `:fanout::held-worker` waits 50 ms.** The
   `where-the-time-actually-goes` FINDING flagged this as *"third time today that a constant differed
   from its siblings for no recorded reason"* and it is **still true and still undocumented** after
   this stone. It is a one-line change with a measured 4× effect on `collect` and no owner.
2. **`collect-stop` is serial.** Twelve `send`-then-`recv` pairs in a `foldl`, so the 135 ms/worker
   adds up instead of overlapping. Sending all twelve `Admin::Stop`s and then reaping twelve replies
   would pay one poll wait instead of twelve — but the generated `stop` method is one atomic
   send+recv (`wat/service.wat:2928-2968`) and splitting it reaches the substrate. **⚠ I did not
   measure this** and it may be exactly the `asks` arc in different clothing.
3. **The `disrupts` feature now has exactly one caller and it is a failure path.** If a future stone
   makes the stall diagnostic tolerant some other way, `disrupts` and `sum-disrupts` both become
   deletable — and *then* row 3 can be met as written.

---

# ⭑ THE ORCHESTRATOR'S GRADING — my own runs, my own reads

**Floor** `.floor/2026-09-10T22-33-35Z/clean.log`: `Summary [ 485.817s] 5237 tests run: 5237 passed
(7 slow), 22 skipped` — 0 failure tokens, no `ARM.txt`. Chaos `7 passed` on **both** sides.

**My own run:** `collect=3405`, **`rt-worker=24`** (was 36, exactly −12), `distinct=8000 dup=0`, and the
summary line carries `disrupts`/`check-exhausted`/`mark-exhausted`/`ack-retries`/`ack-exhausted` intact.

⚠ **My single run sits above their after-band** (3405 against 2620–3117) — inside `collect`'s known ±20 %
spread, so the *direction* is confirmed on my box and the *band* is not reproduced at n=1. Their 4
interleaved pairs are the better instrument and I say so rather than averaging my one run into theirs.

## ★ ROW 8 HELD — and it is the first prediction of mine to survive with a cross-check

I claimed `collect`'s ~338 ms/worker is **two** questions each waiting out ~half a poll, so removing one
should remove ~169 ms/worker.

```
predicted          169 ms/worker
measured  n=2000   166.0
measured  n=20     180.4
```

★★ And the data volunteered a **third, independent** check: at n=20 the *after*-residual is
1616/12 = **135 ms/worker for one question** — reaching the FINDING's ~150 ms-per-question figure from the
opposite direction. Two configs disagree by 14 ms/worker and the executor **offers no mechanism**, saying
4 runs a side cannot carry one. Correct.

## ⛔ STOP-2 FIRED, AND MY CONTRACT DECISION WAS WRONG TWICE OVER

`:fanout::sum-disrupts` is called from **`run-with`'s `drained-stalled` diagnostic** — on **live** workers,
on a path that then raises. Verified on my own read: it survives at `circuit.wat:2940` under a comment
naming it *"THE ONE SURVIVING `sum-disrupts` CALLER."*

⛔ My DESIGN wrote *"`collect` is the only caller"* and then hedged it: *"if any future caller needs
disrupts from a live worker, this stone has taken that away."* **The caller already existed, in the same
file.** I wrote a **hypothetical about a fact one grep would have settled** — and then told the executor to
verify it from the code rather than from my sentence, which is the only reason it was caught. **Ninth
quantitative or factual claim of mine refuted this session.**

## ⭑⭑ And the executor's reasoning for why it CANNOT move is better than my design

Routing the diagnostic through `collect-stop` would:

1. **stop twelve workers mid-diagnosis** — nothing is stopped yet at that point, and
2. trade a **tolerant** fold (every `RecvOutcome` arm → `acc`) for `worker/stop`, which **raises** on
   `Lost`/`Closed`/`Stopped` —
3. **replacing the stall verdict with `defservice stop: …` on exactly the path built for diagnosing a
   wedged system.**

★★★ **A fix that destroys the diagnostic it serves.** So `sum-disrupts` stays as a failure-only live read,
narrowed 6 counters → 4, **zero happy-path cost** — and row 3 is graded **PARTIAL** rather than passed:
`collect` asks once, two folds remain in the file, mutually exclusive on `drain-err`.

## ★ Two disciplines worth naming

**Attribution, not credit.** `rt-total` fell 11214 → 10857, but the executor attributes **only 12** of that
357 to the stone — the rest is `rt-store`/`rt-q-recv`, server-side counters that climb less because the
phase got shorter. **Declining 345 crossings of free credit** is the same instinct that declined a
favourable −1.68 % two stones ago.

**An absence shipped as data.** `seen-skipped` was `0,0,0,0` before and `0,10,10,0` after — recorded by
`seen` during the **drain**, upstream of anything this stone touched, n=8, **no mechanism offered.** Not
buried, not explained away.

## Grade

`1 ✅ · 2 ✅ (n=20 summary byte-identical across all 8 runs both sides; chaos carries the counters
non-zero through the new wire) · 3 ⚠ **PARTIAL** — `collect` asks once, two folds remain, and my contract
decision was not taken because its premise was false · 4 ✅ (36 → 24, zero variance, 8/8 both sides) ·
5 ✅ (disjoint bands, both configs) · 6 ✅ · 7 ✅ · **8 ✅ HELD** · 9 ✅ honoured`

**STOP-1 did not fire** — nine values out became two `defrecord`s (not a wider tuple, avoiding the
`Tuple`-has-no-fourth-accessor trap this campaign hit twice), with `:fanout::WorkerFinal` in the surface's
`:messages` because the projection runs in the forked child. **It compiled and crossed the process tier
first try, with no `src/` change.**

## What stands after this

`collect` **4885 → 2893 ms** at the standard size, 12 crossings gone, and the remaining question still
costs ~135 ms/worker because **a parked worker cannot answer.** That floor is the `asks` gap at
`a4f2d7f7b` — outbound YES, inbound NO — and it reaches `src/`.
