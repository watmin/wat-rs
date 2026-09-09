# SCORE — the summary counts with a set

**SCORED.** Executor: claude, 2026-09-09. Did not commit. **No STOP fired.**

```
     Summary [ 479.528s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-09T02-19-06Z/` — `exit=0`, **no `ARM.txt`**. No tests added. **5237** = the
`seen-ids-stops-cloning` floor, unchanged. 0 FAIL, 0 TIMEOUT.

## WHAT LANDED

`:fanout::summarize`'s two folds moved from `(:wat::core::HashMap :- [String bool])` +
`:wat::hashmap::assoc` to `(:wat::core::PersistentSet :- [String])` + `:wat::set::conj`, and the
two counts from `(:wat::core::count (:wat::hashmap::keys …))` to `:wat::set::length`. This is
`:wat::set::`'s **second production caller**, after `seen-ids` (`0e026d2de`).

**Twelve sites, in one hunk, all inside `:fanout::summarize` (lines 2085–2114).**

| site | before | after |
|---|---|---|
| binding | `id-map` | `id-set` |
| closure param | `acc <- (:wat::core::HashMap :- [String bool])` | `(:wat::core::PersistentSet :- [String])` |
| closure return | `-> (:wat::core::HashMap :- [String bool])` | `(:wat::core::PersistentSet :- [String])` |
| verb | `(:wat::hashmap::assoc acc (:fanout::key-of o) true)` | `(:wat::set::conj acc (:fanout::key-of o))` |
| fold init | `(:wat::core::HashMap :- [String bool])` | `(:wat::core::PersistentSet :- [String])` |
| binding | `w-map` | `w-set` |
| closure param | as above | as above |
| closure return | as above | as above |
| verb | `(:wat::hashmap::assoc acc (:fanout::Outcome/worker o) true)` | `(:wat::set::conj acc (:fanout::Outcome/worker o))` |
| fold init | as above | as above |
| `distinct` | `(:wat::core::count (:wat::hashmap::keys id-map))` | `(:wat::set::length id-set)` |
| `wcount` | `(:wat::core::count (:wat::hashmap::keys w-map))` | `(:wat::set::length w-set)` |

★ **The BRIEF predicted an undercount that did not materialise.** It closes with *"Expect the edit
count to exceed the sketch — the accumulator type appears in the closure parameter, the closure
return, and the fold's initial value."* Those three **are already in the sketch**, and the sketch's
`;; w-set the same` expands to the second fold's five. Read in full, the sketch named **12 of 12**.
The previous stone's sketch was short because it omitted a *verb* (`contains?`) the caller needed;
this one omitted nothing. `:wat::set::` had every verb required — **STOP-4 did not fire.**

`dup (:wat::core::- total distinct)` is byte-for-byte unchanged, as the DESIGN's one contract
decision required.

## ⛔ TWO CORRECTIONS TO THE STONE'S OWN PREMISE

Both are measurements, not objections, and neither changes what was done.

### 1. `summarize` is not in an unaccounted gap. It is inside the `collect` timer.

The DESIGN and row 4 say *"`total` minus the sum of the phase timers is ~560 ms at n=2000, and
`summarize` is part of that unaccounted remainder."* **Both halves are wrong, and the file says so.**

`circuit.wat:2381` calls `summarize`; `circuit.wat:2384` takes `t-stop0`. The `collect` phase is
`(ms t-collect0 t-stop0)` (`:2400`). The bindings evaluate in order, so **`summarize` is timed
inside `collect`.** The six phases tile `[t-setup0, t-end]` contiguously, so `total` minus their sum
is only the integer-division truncation of six `ms` calls — **at most 6 ms, and measured at 3–4 ms
on all four runs below.**

The ~560 ms figure appears to be the `stop=563` phase value from the `seen-ids` SCORE's baseline
run read as unaccounted time; that same run's own numbers reconcile to a 4 ms gap
(12521+15949+7+2752+5454+563 = 37246, `total=37250`). Stated as a hypothesis about where the figure
came from, not as a measurement.

**Consequence: `collect` is the instrument for this stone, not the phase-sum gap.** It is reported
below.

### 2. The term is ~1.1 s at n=2000, not "small" — and the isolated probe measures it

The DESIGN calls the term small and forbids claiming it; the second half stands, the first is
understated by an order of magnitude. A one-shot probe timed the two folds **in isolation from the
40 s harness**, three runs:

| fold, 8000 elements | `:wat::hashmap::assoc` | `:wat::set::conj` |
|---|---|---|
| 8000 distinct ids | **1 101 935 µs / 1 098 950 µs / 1 054 637 µs** | **18 602 µs / 17 991 µs / 17 618 µs** |
| 9 distinct workers | 9 443 µs / 8 999 µs / 8 963 µs | 7 984 µs / 7 908 µs / 7 742 µs |

Both forms returned `count=8000` and `count=9`. So the id fold cost **≈1.05–1.10 s** and now costs
**≈18 ms**; the worker fold was **≈9 ms** and is **≈8 ms** — the DESIGN's ⚠ about `w-map` was
correct, its cost is trivial and no win is claimed for it. Total removed: **≈1.06–1.09 s at n=2000**,
growing as N² → **≈4.3 s at n=4000**, which is the stone's actual reason to exist.

The probe lived at `wat-scripts/scratch-pad/278-summarize-fold-cost.wat` and was **deleted** so row
7's blast radius reads exactly `circuit.wat`. Reproducible source:

```wat
(:wat::core::defn :probe::mk-keys [n <- :wat::core::i64]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])
                     i   <- :wat::core::i64]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::vec::conj acc (:wat::core::format "id-{i}" :i i)))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::range 0 n)))

(:wat::core::defn :probe::mk-workers [n <- :wat::core::i64 w <- :wat::core::i64]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])
                     i   <- :wat::core::i64]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::vec::conj acc (:wat::core::format "w-{i}" :i (:wat::i64::mod i w))))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::range 0 n)))

(:wat::core::defn :probe::via-map [ks <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::core::let
    [m (:wat::core::foldl
         (:wat::core::fn [acc <- (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
                          k   <- :wat::core::String]
           -> (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
           (:wat::hashmap::assoc acc k true))
         (:wat::core::HashMap :- [:wat::core::String :wat::core::bool])
         ks)]
    (:wat::core::count (:wat::hashmap::keys m))))

(:wat::core::defn :probe::via-set [ks <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::core::let
    [s (:wat::core::foldl
         (:wat::core::fn [acc <- (:wat::core::PersistentSet :- [:wat::core::String])
                          k   <- :wat::core::String]
           -> (:wat::core::PersistentSet :- [:wat::core::String])
           (:wat::set::conj acc k))
         (:wat::core::PersistentSet :- [:wat::core::String])
         ks)]
    (:wat::set::length s)))
```

`:user::main` builds `ids`/`ws`, brackets each of the four calls with
`(:wat::time::epoch-nanos (:wat::time::now))`, and prints the four deltas in µs.

## THE MEASUREMENT

★ **Both baselines were run on this box, minutes before the post-change runs**, on the same binary,
not compared to figures from another day. Command, all four:
`./target/release/wat wat-scripts/fanout/circuit.wat 2000 4 3 8192 true 1000`. Binary
`cargo build --release` at HEAD `5217ff203` (`Finished` with no work — already current). Box quiet
each time: `ps -eo args | grep -E 'cargo|nextest|release/wat'` showed only the two idle
`wat --mcp` servers, nothing else, before every run. **Two runs per side**, because one-against-one
cannot separate this term from `collect`'s own spread.

| | base #1 | base #2 | after #1 | after #2 | stated baseline |
|---|---|---|---|---|---|
| load at start (1 min) | 0.29 | 1.14 | 0.45 | 0.95 | 0.59–0.87 |
| wall clock | — (timer failed: no `bc`) | **40.204 s** | **38.839 s** | **39.257 s** | ~40 s |
| setup | 12452 | 12546 | 12460 | 12552 | — |
| fill | 15395 | 15549 | 15473 | 15776 | 15.5–17.3 s |
| arm | 7 | 7 | 7 | 7 | — |
| drain | 2835 | 2808 | 2844 | 2887 | — |
| **collect** ← the phase `summarize` is in | **5210** | **6012** | **4606** | **4585** | 5454 / 5490 (`seen-ids` SCORE) |
| stop | 312 | 364 | 442 | 446 | — |
| **phase sum** | **36211** | **37286** | **35832** | **36253** | — |
| **`total`** | **36215** | **37289** | **35835** | **36256** | — |
| **`total` − phase sum** | **4** | **3** | **3** | **3** | ~560 (**not reproduced** — see correction 1) |
| store-calls | 9736 | 9760 | 9736 | 9770 | 9727–9759 |
| store-ms | 13463 | 13388 | 13495 | 13503 | — |
| full-retries | 1202 | 1218 | 1213 | 1231 | 1202–1238 |
| topic-ticks | 541 | 386 | 458 | 350 | — |
| **distinct / dup** | 8000 / 0 | 8000 / 0 | **8000 / 0** | **8000 / 0** | 8000 / 0 |
| **workers** | 9 | 9 | **9** | **9** | 8–12 across this arc |

⚠ **STOP-2 held: no speedup is claimed.** `setup`, `fill`, `arm`, `drain`, `stop` and wall clock are
all inside their run-to-run spread — `fill` moved 15395→15776 across four runs of two different
programs, a 381 ms band that swamps anything this change could do to them. Wall clock 38.8–40.2 s
spans both sides.

What is worth stating without dressing it as a win: **`collect` is the one phase whose two
post-change values (4606, 4585 — a 21 ms spread) fall outside both baseline values (5210, 6012 — an
802 ms spread), and it is the phase the changed code is timed inside.** Two runs a side is not a
distribution, and `collect`'s own baseline spread is 802 ms, so this is **consistent with** the
isolated probe's ≈1.07 s and is **not** offered as a measurement of it. The probe is the measurement;
the harness only fails to contradict it.

## PER-TIER (after #1 / after #2)

| tier | accepted | refused | acks | redeliveries | expired-waiters | visible/unacked |
|---|---|---|---|---|---|---|
| inbox | 8000 / 8000 | 1213 / 1231 | 8000 / 8000 | 0 / 0 | 154 / 156 | 0/0 |
| sub[0] | 2000 / 2000 | 0 / 0 | 2000 / 2000 | 10 / 10 | 31 / 25 | 0/0 |
| sub[1] | 2000 / 2000 | 0 / 0 | 2000 / 2000 | 0 / 0 | 38 / 32 | 0/0 |
| sub[2] | 2000 / 2000 | 0 / 0 | 2000 / 2000 | 10 / 10 | 43 / 40 | 0/0 |
| sub[3] | 2000 / 2000 | 0 / 0 | 2040 / 2050 | 10 / 10 | 48 / 48 | 0/0 |

Refusals reconcile on both: inbox `refused=1213` = `full-retries=1213`; `1231` = `1231`.
`redeliveries` 0 at the inbox, nonzero at subscriber tiers — the property the previous stone's row 3
gates, on both runs.

⚠ **One field moved between the two post-change runs and must be named:** `seen-skipped` was **0**
on base #1, base #2 and after #1, and **10** on after #2 (with sub[3] `acks=2050` instead of `2040`
— the same ten envelopes). **The changed code cannot have caused it, and the file shows why:** the
counter is snapshotted at `circuit.wat:2360` (`spair (:fanout::seen-stats seenh)`), and `summarize`
is called at `circuit.wat:2381` — twenty-one bindings later in the same in-order `let`. The value is
read **before** the edited code runs. It is the same upstream visibility-expiry variability the
`seen-ids` SCORE recorded for `redeliveries`, and it moved **between my two baseline runs too**
(sub[1] 10→0, sub[2] 10→0, unedited binary). `distinct=8000; dup=0; seen-recorded=8000;
visible=0; unacked=0` on **all four** runs.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ `distinct` and `dup` unchanged | ✅ **`distinct=8000`, `dup=0`** — on both post-change runs *and* both baselines. STOP-1 did not fire |
| 2 | ★ `workers` same shape | ✅ **`workers=9`** on all four runs; inside the 8–12 band this arc's SCOREs span |
| 3 | ★ no cloning verb in `summarize` | ✅ `sed -n '2085,2114p' … \| grep -o ':wat::hashmap::' \| wc -l` → **0**. Whole file → **8** occurrences on 8 lines, all four `:user::` scenario deftests (`:2639 :2644` `:2727 :2732` `:2831 :2836` `:2888 :2893`), untouched by decision |
| 4 | ★ unaccounted time measured | ✅ reported — **and the premise corrected.** `total` − phase sum = **3 ms** (after #1) / **3 ms** (after #2) vs **4 ms** / **3 ms** baseline. The ~560 ms was **not reproduced and is not reproducible**: the phases tile the whole interval, so the gap is `ms`-truncation only. `summarize` is inside **`collect`**, which read 5210/6012 → 4606/4585. Isolated probe: the two folds cost **≈1.07 s → ≈26 ms** at 8000 |
| 5 | wall clock and phases hold | ✅ wall **38.839 / 39.257 s** vs **40.204 s** same-box baseline (stated ~40 s); `fill` **15473 / 15776** vs **15395 / 15549** (band 15.5–17.3 s). Every phase within spread. **No speedup claimed at n=2000** |
| 6 | other reported fields untouched | ✅ `total=8000`, `empty=1`, `seen-recorded=8000`, `check-exhausted=0`, `mark-exhausted=0`, `ack-retries=4`, `ack-exhausted=1`, `publish-calls=200`, `inbox-lost/closed/timedout=0` identical on all four runs; `full-retries` 1213/1231 in family with 1202/1218 and equal to inbox `refused`. ⚠ `seen-skipped` 0→10 on after #2 only — named above, snapshotted 21 bindings **before** the edit |
| 7 | blast radius | ✅ `git diff --stat` → `wat-scripts/fanout/circuit.wat \| 24 ++++------`, **1 file, 12 insertions, 12 deletions, one hunk** at `@@ -2089,22 +2089,22 @@`, entirely inside `:fanout::summarize`. No `src/`, no other `.wat`, `git status --short` shows that one file and nothing else |
| 8 | the corpus loads | ✅ `wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime` **PASS [479.521s]** |
| 9 | the floor holds | ✅ `Summary [ 479.528s] 5237 tests run: 5237 passed (7 slow), 22 skipped` — 0 FAIL, 0 TIMEOUT, no `ARM.txt`, `exit=0` |

## STOPS

**None fired.**

- **STOP-1** (`distinct`/`dup` move by even one) — **held.** 8000 / 0 on both post-change runs, identical to both baselines.
- **STOP-2** (do not claim a speedup) — **honoured.** No speedup claimed at n=2000. `collect`'s move is reported with its baseline spread beside it and explicitly not offered as a measurement of the term; the term is measured by the isolated probe instead, which is a different instrument from the one row 5 warns about.
- **STOP-3** (do not touch the scenario deftests, `wat/`, the queue, the topic, admission, the cap) — **held.** The diff is one hunk in one function; the 8 deliberate `:wat::hashmap::` sites are byte-identical.
- **STOP-4** (`:wat::set::` lacks something) — **held.** `conj` and `length` were the only verbs needed and both exist. No gap in the type to report.
- **STOP-5** (red floor arm) — **no red arm.** Nothing was re-run; the floor ran once.

## BLAST

```
 wat-scripts/fanout/circuit.wat | 24 ++++++++++++------------
 1 file changed, 12 insertions(+), 12 deletions(-)
```

One hunk, `@@ -2089,22 +2089,22 @@`, inside `:fanout::summarize`. Left uncommitted.
