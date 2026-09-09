# SCORE — the slope belongs to a phase

**SCORED.** Executor: claude, 2026-09-09. Did not commit. **No STOP fired.** **No file modified.**

```
     Summary [ 472.820s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-09T03-11-46Z/` — `exit=0`, **no `ARM.txt`**, zero `FAIL`/`TRY`/`TIMEOUT`/`ABORT`/`SIGSEGV`
lines in `clean.log`. **5237** — the `the-summary-counts-with-a-set` floor, unchanged (no test added, no
test removed). Run **after** all twelve measurements, never beside one.

---

## ⭑ THE ANSWER

**One phase is superlinear, and it is `drain`.** Every other phase is linear, flat, or sublinear.

| phase | `n2000/n1000` | `n4000/n2000` | verdict |
|---|---|---|---|
| `setup` | **0.998** | **1.003** | **constant** — not even linear. ~12.4 s at all three n |
| `fill` | **2.015** | **2.021** | **linear** |
| `arm` | 1.000 | 1.000 | 6–8 ms at every n. Integer noise; no slope is measurable |
| **`drain`** | **2.301** | **2.388** | ⭑ **SUPERLINEAR.** Both ratios' whole min–max bands lie above 2.0 |
| `collect` | 1.005 | 1.718 | **not superlinear.** The 1.005 is *inside the spread* — see STOP-4 below |
| `stop` | 2.212 | 0.932 | **uninterpretable.** 64–535 ms; the spread is 7× the median |
| `total` | 1.371 | 1.651 | **sublinear**, because `setup` is a fixed 12.4 s |
| wall clock | 1.370 | 1.638 | as `total` |

`drain` over both doublings: **4000/1000 = 5.494** against 4.0 for linear → **`drain` ≈ n^1.23**.

★ **The `+55 %` claim that started this arc pointed at the right phase and was measured the wrong way.**
The drain *is* superlinear. It is also **4.8 % / 8.0 % / 11.6 %** of the run at n=1000/2000/4000 — so the
phase with the slope is the phase with almost none of the time, and at n=4000 the run is **53 % `fill`**
(linear) and **21 % `setup`** (constant). Both halves are needed to say anything: the slope belongs to
`drain`, the *time* does not.

⚠ **This SCORE names no mechanism.** It measures slopes. The observation in "WHAT THE COUNTERS ADD" below
is offered as an observation with its own numbers, not as the mechanism.

---

## THE COMMAND LINES, QUOTED

```
./target/release/wat wat-scripts/fanout/circuit.wat 1000 4 3 8192 true 1000
./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true 1000
./target/release/wat wat-scripts/fanout/circuit.wat 4000 4 3 8192 true 1000
```

`m=4 j=3 sub-cap=8192 fill-first?=true` held fixed; **`vis-ms=1000` pinned at all three points**; only `n`
varies. Binary: `cargo build --release` at HEAD `9cc5fc007` reported `Finished` in 0.08 s — already
current, nothing rebuilt.

---

## THE MEASUREMENT — twelve runs, none merged, none dropped

All twelve completed (`rc=0`). Times in **ms** as the harness prints them; `wall` is `date +%s%N` around
the process.

| n | run | load @start | wall | `setup` | `fill` | `arm` | `drain` | `collect` | `stop` | `total` | `distinct` | `dup` | `workers` | `seen-skipped` |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1000 | 1 | 0.22 | 29947 | 12484 | 8192 | 6 | 1225 | 5431 | 377 | 27718 | **4000** | **0** | 11 | 10 |
| 1000 | 2 | 3.07 | 28128 | 12378 | 7586 | 7 | 1241 | 4586 | 64 | 25864 | **4000** | **0** | 10 | 0 |
| 1000 | 3 | 3.69 | 28273 | 12431 | 7530 | 7 | 1255 | 4603 | 219 | 26048 | **4000** | **0** | 10 | 0 |
| 1000 | 4 | 0.92 | 28435 | 12412 | 7761 | 6 | 1266 | 4296 | 473 | 26217 | **4000** | **0** | 11 | 10 |
| 1000 | 5 | 5.19 | 28999 | 12426 | 8127 | 7 | 1245 | 4796 | 162 | 26764 | **4000** | **0** | 10 | 0 |
| 1000 | 6 | 2.90 | 28206 | 12419 | 7627 | 7 | 1246 | 4460 | 196 | 25957 | **4000** | **0** | 12 | 20 |
| 2000 | 1 | 1.86 | 38496 | 12386 | 15507 | 7 | 2866 | 4619 | 145 | 35532 | **8000** | **0** | 9 | 0 |
| 2000 | 2 | 1.91 | 39248 | 12395 | 15448 | 8 | 2900 | 5061 | 459 | 36273 | **8000** | **0** | 8 | 0 |
| 2000 | 3 | 1.96 | 38833 | 12409 | 15506 | 7 | 2803 | 4566 | 535 | 35828 | **8000** | **0** | 8 | 0 |
| 4000 | 1 | 1.99 | 63614 | 12434 | 31522 | 7 | 6817 | 7934 | 428 | 59145 | **16000** | **0** | 8 | 0 |
| 4000 | 2 | 1.88 | 62561 | 12399 | 30646 | 8 | 6859 | 7759 | 379 | 58053 | **16000** | **0** | 8 | 0 |
| 4000 | 3 | 1.53 | 64140 | 12468 | 31335 | 7 | 6843 | 8498 | 489 | 59642 | **16000** | **0** | 8 | 0 |

`distinct = n×m` and `dup = 0` on **all twelve**. `empty=1`, `seen-recorded = n×m`, `check-exhausted=0`,
`mark-exhausted=0`, `visible=0`, `unacked=0` on all twelve. `ack-retries` 0–9, `ack-exhausted` 0 except
n=1000 r2 (=1). `workers` 8–12 (the arc's band; not a defect signal). **`seen-skipped` was 0 on nine runs
and 10/10/20 on n=1000 r1/r4/r6** — the upstream visibility-timing variability the previous two SCOREs
recorded; reported, not treated as a regression.

**Phase sum reconciles on every run**: `total` − (`setup`+`fill`+`arm`+`drain`+`collect`+`stop`) = **1–3 ms**
on all twelve, i.e. `ms`-truncation of six divisions, exactly as `SCORE-the-summary-counts-with-a-set`
corrected. `wall` − `total` = **2218–2264 / 2964–3005 / 4469–4508 ms** at n=1000/2000/4000 — process
start-up plus tear-down outside the first timer, itself growing with n and reported for completeness.

### The box

`ps -eo args | grep -E 'cargo|nextest|release/wat' | grep -v grep` was run before **every** run and every
time showed exactly two lines, both idle MCP servers:

```
/home/john/.cargo/bin/wat --mcp
/home/john/.cargo/bin/wat --mcp
```

**No measurement ran beside another, and the floor ran after all of them.** ⚠ **Honest caveat about the
load column:** on this 12-core box the 1-minute load average does not return below ~1.5–2.0 between runs.
A 3-minute wait-for-`load<1.0` loop timed out at 5.19; `ps --sort=-pcpu` at that moment showed **no CPU
consumer at all** above the two resident agent sessions (`grok --resume holon` 7.8 %, `claude --resume
holon` 4.4 %) and **one** runnable process. The residue is the decay of the run's own child processes plus
that ambient session load. Runs 2, 3, 5 and 6 at n=1000 therefore started at loads 3.07/3.69/5.19/2.90
while every n=2000 and n=4000 run started at **1.53–1.99**. This is stated rather than hidden because it
is the one asymmetry in the sample — and it cuts **against** the finding, not for it: the noisiest starts
are all at the *smallest* n, which would *inflate* n=1000 and *depress* every ratio.

---

## MEDIANS AND OBSERVED MIN–MAX

Median over all runs at that n (6 at n=1000, 3 each at n=2000 and n=4000).

| phase | n=1000 median [min–max] | n=2000 median [min–max] | n=4000 median [min–max] |
|---|---|---|---|
| `setup` | **12422** [12378–12484] | **12395** [12386–12409] | **12434** [12399–12468] |
| `fill` | **7694** [7530–8192] | **15506** [15448–15507] | **31335** [30646–31522] |
| `arm` | **7** [6–7] | **7** [7–8] | **7** [7–8] |
| `drain` | **1246** [1225–1266] | **2866** [2803–2900] | **6843** [6817–6859] |
| `collect` | **4594** [4296–5431] | **4619** [4566–5061] | **7934** [7759–8498] |
| `stop` | **208** [64–473] | **459** [145–535] | **428** [379–489] |
| `total` | **26132** [25864–27718] | **35828** [35532–36273] | **59145** [58053–59642] |
| wall | **28354** [28128–29947] | **38833** [38496–39248] | **63614** [62561–64140] |

**Spread magnitudes, for reading the ratios against:** `drain` **41 / 97 / 42 ms** (0.6–3.4 % of its
median — the tightest phase in the table); `fill` **662 / 59 / 876 ms**; `collect` **1135 / 495 / 739 ms**;
`stop` **409 / 390 / 110 ms**; `setup` **106 / 23 / 69 ms**.

---

## THE RATIOS, EACH WITH ITS BAND

`median ratio`, then the **worst-case band** the observed extremes admit — `min(hi n)/max(lo n)` to
`max(hi n)/min(lo n)`. A phase is superlinear only if its **whole band** clears 2.0.

| phase | `n2000/n1000` | band | `n4000/n2000` | band | `n4000/n1000` |
|---|---|---|---|---|---|
| `setup` | 0.998 | 0.992–1.003 | 1.003 | 0.999–1.007 | 1.001 |
| `fill` | 2.015 | 1.886–2.059 | 2.021 | 1.976–2.041 | 4.073 |
| `arm` | 1.000 | 1.000–1.333 | 1.000 | 0.875–1.143 | 1.000 |
| **`drain`** | **2.301** | **2.214–2.367** | **2.388** | **2.351–2.447** | **5.494** |
| `collect` | 1.005 | 0.841–1.178 | 1.718 | 1.533–1.861 | 1.727 |
| `stop` | 2.212 | **0.307–8.359** | 0.932 | **0.708–3.372** | 2.063 |
| `total` | 1.371 | 1.282–1.402 | 1.651 | 1.600–1.679 | 2.263 |
| wall | 1.370 | 1.285–1.395 | 1.638 | 1.594–1.666 | 2.244 |

**Robustness of the one finding to the n=1000 sample choice.** Recomputing the n=1000 median over only
the three runs whose start load matched the larger points (r4, r5, r6 — loads 0.92/5.19/2.90) moves
`drain` to **1246** [1245–1266] → `n2000/n1000` = **2.300**. Over r1–r3 only: `drain` median **1241** →
**2.309**. The `drain` ratio is **2.30 on every subset**; it does not depend on which n=1000 runs are
counted.

### ⭑ Why `drain`'s >2× cannot be a `vis` artifact

`drain ≈ max(work, vis)` is the measured coupling. With `vis` **pinned at 1000 ms**, any `vis`-dependent
term is the **same constant at all three points**, and `drain = work(n) + c` with `c` constant makes the
measured ratio a **lower bound** on `work`'s slope — a constant additive term drags a ratio *toward 1*, it
cannot manufacture a value above 2. All three medians (1246, 2866, 6843) also exceed `vis=1000`, so no
point is reading a bare timeout floor. **2.30 and 2.39 are floors, not ceilings.**

---

## n=4000 COMPLETES — the bound was not reached

**STOP-2 did not fire.** All three n=4000 runs completed with `distinct=16000; dup=0`.

`circuit.wat:2328` bounds attempts at `n×m` = **16000** poll slots. Observed `poll-calls`:
**835 / 845 / 855** — **5.3 % of the bound**. At n=2000 it was 380–385 of 8000 (4.8 %); at n=1000,
170–180 of 4000 (4.4 %). Headroom is ~19× at every point and the utilisation is drifting up only slowly.

Wall clock **62.6–64.1 s**, against the EXPECTATIONS' untested guess of ~80 s — **faster than predicted**,
so nothing about n=4000 is pathological. Per-tier at n=4000 (r1 / r2 / r3):

| tier | accepted | refused | acks | redeliveries | expired-waiters | visible/unacked |
|---|---|---|---|---|---|---|
| inbox | 16000 / 16000 / 16000 | 2471 / 2437 / 2465 | 16000 / 16000 / 16000 | 0 / 0 / 0 | 242 / 239 / 248 | 0/0 all |
| sub[0] | 4000 ×3 | 0 ×3 | 4000 ×3 | 0 / 0 / 0 | 33 / 33 / 37 | 0/0 all |
| sub[1] | 4000 ×3 | 0 ×3 | 4000 ×3 | 0 / 0 / 0 | 45 / 45 / 51 | 0/0 all |
| sub[2] | 4000 ×3 | 0 ×3 | 4000 ×3 | 0 / 0 / 0 | 60 / 60 / 66 | 0/0 all |
| sub[3] | 4000 ×3 | 0 ×3 | 4000 ×3 | 40 / 10 / 30 | 80 / 80 / 87 | 0/0 all |

Inbox `refused` = `full-retries` on all three (2471=2471, 2437=2437, 2465=2465). `redeliveries=0` at the
inbox on every run at every n — the property `seen-ids` gates. Per-tier at n=1000 and n=2000 is in the
logs and matches the same shape (inbox `refused`=`full-retries`; `redeliveries` 0 at inbox, 0–20 at
subscriber tiers).

---

## WHAT THE COUNTERS ADD — reported as an observation, not a mechanism

Two counters are scoped to the drain alone, so they can be read without disentangling phases:

| counter | n=1000 med [min–max] | n=2000 | n=4000 | `r21` | `r42` |
|---|---|---|---|---|---|
| `drain-store-calls` | 376 [373–379] | 760 [759–762] | 1548 [1546–1552] | **2.024** | **2.037** |
| `drain-store-ms` | 672 [663–683] | 1689 [1657–1692] | 4418 [4393–4435] | **2.513** | **2.616** |
| `drain-busy-ms` | 774 [756–785] | 1907 [1873–1909] | 4873 [4850–4891] | **2.465** | **2.555** |
| `poll-calls` | 175 [170–180] | 380 [380–385] | 845 [835–855] | **2.171** | **2.224** |
| `store-calls` (whole run) | 4920 [4908–4943] | 9750 [9747–9761] | 19531 [19498–19536] | 1.982 | 2.003 |
| `store-ms` (whole run) | 5838 [5765–5899] | 13541 [13359–13646] | 33738 [33683–33774] | **2.319** | **2.492** |
| `full-retries` | 602 [599–618] | 1213 [1200–1224] | 2465 [2437–2471] | 2.015 | 2.032 |
| `asleep` | 3938 [3853–4167] | 8095 [7889–8124] | 16731 [16363–16784] | 2.056 | 2.067 |

★ **The store is called a linear number of times and takes a superlinear amount of time.**
`drain-store-calls` doubles cleanly (2.02, 2.04) while `drain-store-ms` grows 2.51 and 2.62; mean
drain-scoped store latency is **1.79 → 2.22 → 2.85 ms/call**. Whole-run: `store-calls` 1.98/2.00 vs
`store-ms` 2.32/2.49, mean **1.19 → 1.39 → 1.73 ms/call**.

⚠ **Two things this does not establish**, both stated so the next stone does not inherit a claim:
(a) it does not name *what* in the store grows with n; (b) it does not explain why `fill` stays linear at
2.015/2.021 while whole-run `store-ms` is superlinear — `asleep` also scales linearly at 2.06, so a
throttled `fill` masking a superlinear per-call cost is **a hypothesis with an obvious next measurement,
not a result.** The drain is the phase where nothing throttles, and the drain is the phase with the slope.

---

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ three points, ≥3 runs each | ✅ **twelve rows** — n=1000 ×6, n=2000 ×3, n=4000 ×3. None merged, none skipped, none dropped. All `rc=0` |
| 2 | ★ every phase reported per run | ✅ `setup fill arm drain collect stop total` **and** wall clock on all twelve, per run, before any median. Phase sum reconciles to `total` within 1–3 ms on every row |
| 3 | ★★ medians AND spread | ✅ median and observed min–max per phase per n, with the spread magnitudes stated beside them, and **every ratio carries its worst-case band**. `drain`'s bands (2.214–2.367, 2.351–2.447) clear 2.0 entirely; `collect`'s 2000/1000 and `stop`'s both bands do not exclude 1.0 and are reported as **not measurements of a slope** |
| 4 | ★ the ratios are stated | ✅ full table, both doublings, plus `4000/1000`. **`drain` is the one phase >2× — 2.301 and 2.388.** `fill` 2.015/2.021 = linear; `setup` 1.00 = constant; `collect` 1.005/1.718 = sublinear; `total` 1.371/1.651 = sublinear |
| 5 | ★ `vis-ms=1000` at every point | ✅ command lines quoted above; the sixth slot is `1000` at all three n. All three `drain` medians exceed `vis`, so none reads a bare floor, and a pinned `vis` contributes a **constant** term that can only *depress* the ratio (argued above) |
| 6 | correctness at every point | ✅ `distinct=4000/8000/16000` = `n×m` and `dup=0` on **all twelve**; `seen-recorded=n×m`, `empty=1`, `visible=0`, `unacked=0`, `check-exhausted=0`, `mark-exhausted=0` everywhere. **STOP-1 did not fire; no run was excluded** |
| 7 | the box was quiet for every run | ✅ the `ps` gate ran before every run and showed only the two idle `wat --mcp` servers, every time; one run at a time; floor after all twelve. ⚠ load stated per run **with the honest caveat** that this box's ambient 1-min load floor is ~1.5–2.0 and the four elevated starts (2.90–5.19) are all at n=1000, which biases *against* the finding |
| 8 | no file was modified | ✅ `git status --porcelain` is **empty** — zero `M` lines, and (before this SCORE) zero untracked lines; `git diff --stat` empty. Scratch went to the session temp dir, not the repo. **STOP-3 did not fire: no code change was needed or made** |
| 9 | the floor still holds | ✅ `Summary [ 472.820s] 5237 tests run: 5237 passed (7 slow), 22 skipped` — 0 FAIL, 0 TIMEOUT, `exit=0`, no `ARM.txt`, zero `FAIL`/`TRY`/`TIMEOUT` lines in `clean.log`. `wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime` **PASS [472.814s]** |

---

## STOPS

**None fired.**

- **STOP-1** (`distinct ≠ n×m` or `dup ≠ 0`) — **held on all twelve.** No run was disqualified as a timing sample; every median is over runs that delivered exactly `n×m` distinct messages with zero duplicates.
- **STOP-2** (n=4000 does not complete) — **did not fire.** All three completed in 62.6–64.1 s using **835–855 of 16000** attempt slots. `vis` was **not** raised. Counters and the per-tier line are reported above anyway, since the point was untested.
- **STOP-3** (do not modify any file) — **held.** `git status --porcelain` empty. No measurement needed a code change; no counter was added.
- **STOP-4** (no ratio without its spread) — **honoured, and it bites two rows.** ⚠ **`collect`'s `n2000/n1000` = 1.005 is meaningless as stated**: the median difference is **25 ms** while the n=1000 spread is **1135 ms** (4296–5431) — 45× larger. The defensible statement is *"`collect` at n=1000 and n=2000 are indistinguishable at this sample size"*, and its band (0.841–1.178) is quoted rather than the 1.005. ⚠ **`stop` admits no ratio at all**: 64–535 ms at n=1000 gives a band of **0.307–8.359**. The 2.212 in the table is printed for completeness and **must not be read as a slope.** `drain` is the opposite case and that is why it is the finding: its spread is 41/97/42 ms against median differences of 1620 and 3977 ms.
- **STOP-5** (no floor beside a measurement, no two measurements at once) — **held.** Twelve sequential runs, `ps` gate clean before each, floor last.
- **STOP-6** (red floor arm) — **no red arm.** The floor ran **once** and was green. Nothing was re-run.

---

## BLAST RADIUS

```
$ git status --porcelain
(empty — zero M lines, zero untracked, before this SCORE was written)

$ git diff --stat
(empty)
```

**No file modified.** The only new file is this SCORE. Left uncommitted.
