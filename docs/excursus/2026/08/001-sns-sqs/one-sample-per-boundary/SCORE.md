# SCORE — one sample per boundary

Struck on `sns-sqs`, from `c62604010`. Everything uncommitted. `wat-scripts/fanout/circuit.wat` only.

**Result in one line:** the eleven-that-were-ten `Queue/stats` round-trips per queue are now **six
boundary samples**, all five phases have a busy figure, every reconciliation identity holds with
remainder 0 on 60/60 tier rows — and **STOP-3 FIRED: `collect` did not drop.** Its after band
(4646–5203 ms) sits *at or above* its before band (4052–4988 ms) over 6 runs each, and the new
instrument says why: **only ~220 ms of `collect`'s ~4.8 s is subscriber-queue handler time at all.**

**Floor, verbatim** (`.floor/2026-09-10T01-03-43Z/clean.log`):

```
     Summary [ 480.739s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

0 lines matching `^ *(FAIL|TRY|TIMEOUT|ABORT|SIGSEGV)`, no `ARM.txt`.

**STOPs fired: STOP-3.** STOP-1 (identities) did not fire — remainder 0 everywhere. STOP-2 (a reader
needing a mid-phase value) did not fire — every one of the ten readers mapped onto a boundary that
already existed. STOP-4 (a red) did not fire.

---

## ⭑⭑ STOP-3 FIRED — `collect` did not drop, and the premise is refuted

The DESIGN's premise: *"`collect` costs 4583 ms because it makes ~8 stats round-trips per queue back
to back."* It does not. **Three independent measurements say the same thing, and the first two are
new instruments this stone built.**

### 1. The direct before/after, 6 runs each, box quiet before every batch

| | r1 | r2 | r3 | r4 | r5 | r6 | min | max | mean |
|---|---|---|---|---|---|---|---|---|---|
| `collect` **before** | 4879 | 4099 | 4052 | 4988 | 4794 | 4918 | **4052** | **4988** | 4622 |
| `collect` **after** | 4735 | 4789 | 5001 | 4673 | 5203 | 4646 | **4646** | **5203** | 4841 |

**No separation, and what separation there is points the wrong way.** The after band is entirely
inside-or-above the before band. `collect` lost 3 of its 4 stats folds and got *no faster*.
⚠ I claim no *increase* either — the before distribution is visibly bimodal (4052/4099 against
4794/4879/4918/4988) and 6 runs cannot resolve that. The claim is only: **no drop.**

### 2. `collect-busy-ms` = 194–235 ms (mean 220) — the new instrument prices the ceiling

Of `collect`'s ~4841 ms, the subscriber queues' own handler time across it is **~220 ms, 4.5 %**.
Even deleting *every* subscriber-queue round-trip from `collect` could not remove 4583 ms, because
the queues are not doing 4583 ms of work during it.
⚠ **This bounds `qclients` only.** `collect` also talks to the topic (`topic-ticks`,
`topic-inbox-fails`), the 12 worker peers (`sum-disrupts`, `collect-stop`) and `seen` — deliberately
out of scope, so `collect-busy-ms` does not bound waiting on *those*.

### 3. `arm` calibrates what one sample actually costs — 11 ms, not 1100

`arm` (`t-arm0` → `t-drain0`) gained **exactly one `sample-of` and nothing else**:

| | r1 | r2 | r3 | r4 | r5 | r6 | mean |
|---|---|---|---|---|---|---|---|
| `arm` before | 7 | 7 | 7 | 7 | 6 | 6 | 6.7 |
| `arm` after | 17 | 19 | 18 | 19 | 18 | 17 | **18.0** |

**One `sample-of` at m=4 = 11.3 ms ⇒ ~2.8 ms per `Queue/stats` round-trip.** So the four folds that
used to sit in `collect` were worth **≈ 45 ms**, and the whole ten-fold defect was worth **≈ 113 ms
of a 22.6 s run (0.5 %)**. The count was real; the cost was not.

**Per STOP-3 I stopped here and did not hunt a second change.** What `collect` actually contains,
offered as a *pointer, not a measurement*: `collect-stop` over 12 worker processes returning **8000
`:fanout::Outcome` records**, then `summarize` and (after `t-end`) `traces-of` folding those same
8000 records — plus `empty-flags` (m receives at 1000 s visibility) and `sum-disrupts` over 12 peers.
⚠ **I did not measure any of that.** It is where I would look next, not a finding.

---

## The before/after phase table — all phases, 6 runs each

`2000 4 3 8192 true 1000`, `./scripts/capped.sh --limit 8g`, box verified quiet
(`ps -eo args | grep -E 'cargo|nextest|release/wat'`) before every batch. The 3 extra runs per side
were taken because STOP-3's direction was the opposite of the hypothesis and 3 runs could not carry
that; the second before batch was taken by `git stash`-ing the edit, so the binary is identical
across all 12 runs.

### Before (`c62604010`)

| ms | r1 | r2 | r3 | r4 | r5 | r6 | min | max | mean |
|---|---|---|---|---|---|---|---|---|---|
| `setup` | 12384 | 12385 | 12429 | 12444 | 12390 | 12447 | 12384 | 12447 | 12413 |
| `fill` | 2615 | 2604 | 2626 | 2603 | 2606 | 2590 | 2590 | 2626 | 2607 |
| `arm` | 7 | 7 | 7 | 7 | 6 | 6 | 6 | 7 | 6.7 |
| `drain` | 2423 | 2441 | 2461 | 2469 | 2559 | 2469 | 2423 | 2559 | 2470 |
| `collect` | 4879 | 4099 | 4052 | 4988 | 4794 | 4918 | 4052 | 4988 | 4622 |
| `stop` | 403 | 754 | 366 | 425 | 518 | 485 | 366 | 754 | 492 |
| `total` | 22712 | 22293 | 21943 | 22939 | 22876 | 22917 | 21943 | 22939 | 22613 |
| `drain-busy-ms` | 1443 | 1445 | 1457 | 1468 | 1483 | 1469 | 1443 | 1483 | 1461 |
| `store-calls` | 4656 | 4662 | 4667 | 4665 | 4667 | 4680 | 4656 | 4680 | 4666 |
| `store-ms` | 7044 | 7045 | 7069 | 7122 | 7220 | 7110 | 7044 | 7220 | 7102 |
| `drain-store-calls` | 737 | 739 | 738 | 738 | 739 | 743 | 737 | 743 | 739.0 |
| `drain-store-ms` | 1197 | 1195 | 1206 | 1216 | 1226 | 1215 | 1195 | 1226 | 1209 |
| `poll-calls` | 325 | 325 | 330 | 325 | 330 | 335 | 325 | 335 | 328.3 |
| `qticks` | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `queue-receive-calls` | 815 | 815 | 815 | 816 | 816 | 816 | 815 | 816 | 815.5 |

### After

| ms | r1 | r2 | r3 | r4 | r5 | r6 | min | max | mean |
|---|---|---|---|---|---|---|---|---|---|
| `setup` | 12407 | 12419 | 12416 | 12506 | 12414 | 12482 | 12407 | 12506 | 12441 |
| `fill` | 2601 | 2635 | 2619 | 2602 | 2627 | 2603 | 2601 | 2635 | 2615 |
| `arm` | 17 | 19 | 18 | 19 | 18 | 17 | 17 | 19 | **18.0** |
| `drain` | 2414 | 2467 | 2433 | 2468 | 2412 | 2444 | 2412 | 2468 | 2440 |
| `collect` | 4735 | 4789 | 5001 | 4673 | 5203 | 4646 | 4646 | 5203 | 4841 |
| `stop` | 435 | 96 | 154 | 241 | 506 | 486 | 96 | 506 | 320 |
| `total` | 22612 | 22427 | 22642 | 22511 | 23182 | 22682 | 22427 | 23182 | 22676 |
| **`fill-busy-ms`** | 615 | 613 | 615 | 608 | 617 | 612 | 608 | 617 | **613** |
| **`arm-busy-ms`** | 10 | 13 | 12 | 11 | 11 | 10 | 10 | 13 | **11.2** |
| `drain-busy-ms` | 1493 | 1488 | 1489 | 1499 | 1475 | 1501 | 1475 | 1501 | 1491 |
| **`collect-busy-ms`** | 226 | 233 | 221 | 209 | 235 | 194 | 194 | 235 | **220** |
| **`stop-busy-ms`** | 5 | 6 | 2 | 5 | 5 | 5 | 2 | 6 | **4.7** |
| `store-calls` | 4634 | 4641 | 4639 | 4640 | 4637 | 4627 | 4627 | 4641 | 4636 |
| `store-ms` | 7086 | 7062 | 7074 | 7088 | 7030 | 7105 | 7030 | 7105 | 7074 |
| `drain-store-calls` | 736 | 740 | 738 | 740 | 736 | 736 | 736 | 740 | 737.7 |
| `drain-store-ms` | 1225 | 1220 | 1221 | 1232 | 1210 | 1234 | 1210 | 1234 | 1224 |
| `poll-calls` | 330 | 340 | 335 | 340 | 330 | 325 | 325 | 340 | 333.3 |
| `qticks` | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `queue-receive-calls` | 817 | 816 | 815 | 815 | 814 | 816 | 814 | 817 | 815.5 |

One after phases line, verbatim (`after-1`):

```
"setup=12407;fill=2601;arm=17;drain=2414;collect=4735;stop=435;fill-depth=[2000/0][2000/0][2000/0][2000/0];qticks=0;topic-ticks=74;disrupts=0;check-exhausted=0;mark-exhausted=0;ack-retries=8;ack-exhausted=2;seen-recorded=8000;seen-skipped=0;publish-calls=200;full-retries=190;inbox-lost=0;inbox-closed=0;inbox-timedout=0;asleep=228;publish-attempts=390;poll-calls=330;drain-stale-max=0;store-calls=4634;store-ms=7086;drain-store-calls=736;drain-store-ms=1225;fill-busy-ms=615;arm-busy-ms=10;drain-busy-ms=1493;collect-busy-ms=226;stop-busy-ms=5;total=22612
```

---

## The new busy figures and their divisors

**Every one divides by `(1e6 × m)` — the same divisor `drain-busy-ms` has always used.** It is a
**per-queue mean in milliseconds, not a total** (trap-door 2 of EXPECTATIONS). `:fanout::busy-ms`
is the single site that does the arithmetic; `drain-busy-ms` now calls it instead of inlining it,
and keeps its name and its meaning.

| field | boundary pair | divisor | mean (6 runs) | as % of its phase |
|---|---|---|---|---|
| `fill-busy-ms` | `s-pub0` → `s-arm0` | `1e6 × m` | 613 | 23 % of 2615 |
| `arm-busy-ms` | `s-arm0` → `s-drain0` | `1e6 × m` | 11.2 | 62 % of 18.0 — *this is the sample itself* |
| `drain-busy-ms` | `s-drain0` → `s-collect0` | `1e6 × m` | 1491 | 61 % of 2440 |
| `collect-busy-ms` | `s-collect0` → `s-stop0` | `1e6 × m` | 220 | 4.5 % of 4841 |
| `stop-busy-ms` | `s-stop0` → `s-end` | `1e6 × m` | 4.7 | 1.5 % of 320 |

`setup` has **no** busy figure and that is deliberate: at `t-setup0` the queues do not exist yet, so
no sample is possible there — and `setup` is cold boot, ruled out of scope by the builder.

⚠ **Read `arm-busy-ms` as the instrument measuring itself**: `arm` contains one `sample-of` and
almost nothing else, so 11.2 of its 18.0 ms is the sample's own server-side handler time. That is
what makes it the calibration in STOP-3 above, and it is the honest form of "the measurement
contains the measurer" — named, not hidden.

⚠ **These are SERVER-SIDE HANDLER TIME, not interpreter time** (Row 12). They bound how much of a
phase was *waiting on a subscriber queue*. They do **not** prove the remainder is interpretation:
the remainder also contains transport, process scheduling, and every peer that is not `qclients`.

---

## Round-trips per queue, before and after

| | call sites | round-trips per queue | at m=4 | inside timed phases |
|---|---|---|---|---|
| **before** | 10 | **10** | 40 | 10 sites / 40 round-trips |
| **after** | 6 | **6** | 24 | 5 sites / 20 round-trips (`s-end` lands after `t-end`) |

⛔ **The DESIGN's count of 11 is wrong by one: it is 10.** `sum-store-calls` is called **3×**, not 4×
— `circuit.wat:2500` (`sc-before`), `:2522` (`sc-after`), `:2528` (`store-calls`). Verified by
`grep -n` over the whole file before the edit; there was no fourth site. So: `sum-calls` ×1 +
`sum-ticks` ×1 + `sum-store-calls` ×3 + `sum-store-ns` ×3 + `sum-handler-ns` ×2 = **10**.

Where they went, phase by phase (this is the *shape* of the change, and it is why `drain` and
`collect` were the only phases that could gain):

| phase | folds inside it, before | samples inside it, after |
|---|---|---|
| `fill` | 0 | 1 (`s-pub0`) |
| `arm` | 0 | 1 (`s-arm0`) |
| `drain` | **6** (`sc/ns/hn-before` + `sc/ns/hn-after`) | 1 (`s-drain0`) |
| `collect` | **4** (`calls`, `ticks`, `store-calls`, `store-ns`) | 1 (`s-collect0`) |
| `stop` | 0 | 1 (`s-stop0`) |
| *(after `t-end`)* | 0 | 1 (`s-end`) |

`Queue/stats` call sites remaining in `circuit.wat`: **3** — `:fanout::depth-of` (the poll loop, cut
by the DESIGN), `:fanout::tier-line` (after `t-end`, and already one-reply-many-fields), and
`:fanout::sample-of`.

⭑ **And the poll loop is why the absolute totals barely move.** A sub queue's `count-calls` is ~360
per run, of which the ten harness folds contributed **20**; the rest is `depth-of` inside
`poll-until-filled` / `poll-until-drained` / `poll-until-visible-zero`. The harness folds were never
more than ~6 % of the stats traffic. That loop is explicitly out of scope (`18a86fe27`, the
observer-effect cut) and I did not touch it.

---

## Rows, graded

| # | row | result |
|---|---|---|
| 1 | ⛔ identities hold exactly | ✅ **PASS** — 12 runs × 5 tiers = **60/60 rows**, `put+delete+count+scan` = `store-calls` **and** the four `*-ns` = `store-ns`, **remainder 0** on every one |
| 2 | ⛔ the run is still correct | ✅ **PASS** — `distinct=8000`, `dup=0`, `total=8000`, inbox `accepted=2000` on all 6 after runs (and all 6 before), no raise, rc 0 |
| 3 | ⛔ the stall gate still fires | ✅ **PASS** — `circuit.wat 5 1 0 32 false 0` → **rc 2**, `drained-stalled: no delivery progress in 600 polls; last=[0/0] outbox=5 acks=0 polls=601` |
| 4 | ★ the round-trips are gone | ✅ **PASS** — the five one-field folds no longer exist; **10 → 6** per queue (5 of the 6 inside a timed phase). ⚠ The target was "11 → the number of boundaries"; the real starting count was **10** and the boundary count is **6** |
| 5 | ★ every phase has a busy figure | ✅ **PASS** — `fill-busy-ms`, `arm-busy-ms`, `drain-busy-ms`, `collect-busy-ms`, `stop-busy-ms`. `drain-busy-ms` keeps its name; its arithmetic is unchanged, now via `:fanout::busy-ms` |
| 6 | ★ no sample sits inside a phase | ✅ **PASS** — all six samples are the statement **immediately after** their boundary timestamp. Each phase pays for exactly one sample (the one that opens it); `s-end` opens nothing and lands outside `total` |
| 7 | five chaos scenarios | ✅ **PASS** — `Summary [ 33.774s] 7 tests run: 7 passed, 5252 skipped` |
| 8 | corpus loads | ✅ **PASS** — `PASS [ 480.732s] (5237/5237) wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime` |
| 9 | floor holds | ✅ **PASS** — `Summary [ 480.739s] 5237 tests run: 5237 passed (7 slow), 22 skipped`. 0 FAIL, 0 TIMEOUT, no `ARM.txt` |
| 10 | blast radius | ✅ **PASS** — `git status --porcelain` = ` M wat-scripts/fanout/circuit.wat` (+ this SCORE). Nothing was forced elsewhere; the compiler and the corpus gate found nothing to change |
| 11 | ★★ `collect` drops materially | ⛔ **FAIL — STOP-3 FIRED. Your hypothesis is refuted.** After band 4646–5203 vs before band 4052–4988 over 6 runs each: **no drop, bands not separated in the claimed direction.** `collect-busy-ms`=220 ms and the `arm` calibration (11 ms per sample) say the four folds were worth ~45 ms, not 4583 |
| 12 | ⚠ no throughput / no interpreter claim | ✅ **honoured** — stated below, and no such claim appears anywhere in this SCORE |
| 13 | ⚠ every moved absolute named | ✅ **honoured** — six moved figures, each with direction and reason, below |

---

## Row 12 — what this stone did NOT do

⚠ **This is not a throughput win and I do not claim one.** It removes ~20 *harness* round-trips per
run from inside the timed phases — about **56 ms of a 22.6 s run, 0.25 %**. `drain` is 2470 → 2440 ms
and `total` is 22613 → 22676 ms; both differences are inside their own spreads. Nothing was made
faster per message. The system does exactly the same work.

⚠ **And the busy metrics do not measure interpretation.** `collect-busy-ms=220` says 220 ms of
`collect` was subscriber-queue handler time. It does **not** say the other 4621 ms is the wat
interpreter — that remainder still holds transport, process scheduling, `collect-stop`'s transfer of
8000 records, and every peer that is not `qclients`. Getting to the point where *interpretation* is
the lag needs an instrument this stone did not build.

---

## Row 13 — every absolute figure that moved, with direction and reason

Eleven-reads-at-eleven-instants became one-read-at-one-instant, and a stats call is **not free on the
server**: `wat-scripts/queue/sqs.wat:1266` shows the `stats` impl performing **two count-index calls**
(`store-calls` +2, `count-calls` +2, `store-ns` += `depth-ns`) and adding its own `handler-ns`, then
reporting the **post-increment** values. So the number of samples taken is *visible* in every store
figure the harness prints. Fewer samples ⇒ smaller reported totals — **the instrument shrinking, not
the work.**

| figure | before → after (mean) | direction | why |
|---|---|---|---|
| `store-calls` | 4666 → **4636** | **↓ 30** | The read sits at `s-collect0`, the **4th** stats call each queue has served, where it used to be the **9th**. −5 calls × 2 store-calls × 4 queues = **−40**, offset by `poll-calls` rising 328→333 (each poll round-trip is another +2). −40 + 10 = −30. Matches. |
| `store-ms` | 7102 → **7074** | **↓ 28** | Same cause: five fewer stats calls per queue means five fewer `depth-ns` contributions before the read. Small, and inside the before spread (7044–7220). |
| `drain-store-calls` | 739.0 → **737.7** | **↓ 1.3** | The delta's endpoints changed from "1st before-read → 1st after-read" (3 stats calls per queue enclosed) to "`s-drain0` → `s-collect0`" (1 enclosed): **−4**. Offset by ~+2.5 from the extra poll round-trips. **This is now the honest figure** — it no longer charges the drain for two of the harness's own reads. |
| `drain-store-ms` | 1209 → **1224** | **↑ 15** | Two opposing terms: −4 stats calls' `depth-ns` (≈ −7 ms) and +5 poll round-trips (≈ +9 ms), on a run-to-run spread of ±16 ms before. Inside the noise; the poll count, not the change, sets it. |
| `drain-busy-ms` | 1461 → **1491** | **↑ 30** | ⭑ **Not caused by the change — it tracks `poll-calls`.** `drain-busy-ms / poll-calls` is **4.44** (4.39–4.52) before and **4.47** (4.38–4.62) after: identical. `poll-calls` rose 328.3 → 333.3, and 5 × 4.45 ≈ +22 ms. The change's own effect is the other way: the old delta enclosed 3 of the harness's stats calls, the new one encloses 1, so it should read ~4 ms *lower*. Both effects are far below the spread. |
| `arm` (wall) | 6.7 → **18.0** | **↑ 11.3** | ⛔ **A real, deliberate cost.** `arm` had no stats fold and now opens with `s-arm0`. This is the price of instrumenting a phase that previously had no instrument, and it is what makes `arm-busy-ms` possible. It is also the calibration that refuted Row 11. **11.3 ms at m=4 ⇒ ~2.8 ms per `Queue/stats` round-trip.** |
| `fill`, `drain`, `collect`, `stop`, `total`, `setup` | see tables | **no separated shift** | Every one overlaps its own before band. `fill` and `stop` each gained a sample (+11 ms) and `drain`/`collect` each lost several; none of it clears the run-to-run spread. |
| `queue-receive-calls` | 815.5 → **815.5** | **unchanged** | Exactly as predicted: a `stats` call does not touch `receive-calls`, so moving the read cannot move it. The `receive_calls_are_not_triple_the_messages` floor test is untouched. |
| `qticks` | 0 → **0** | **unchanged** | `Counters/ticks` is only incremented by the tick handler, and no sub queue armed one in any of the 12 runs. |
| tier-line `count-calls` (per sub) | ~361 → ~357 | **↓ ~4** | `tier-line` reads after `t-end`, by which point each queue has served 7 stats calls instead of 11: −4 calls × 2 = **−8** expected, ~−4 observed, the rest absorbed by poll-loop variation. ⚠ I state this as *directionally consistent*, not as an exact match — `depth-of` in the poll loop supplies ~95 % of these counts and varies run to run, so a clean arithmetic check is not available here. |

**Nothing was re-added to force a shift to zero.**

---

## The edit — real site count

**One file, 10 hunks, +90 / −79 lines.** `git diff --stat`: `wat-scripts/fanout/circuit.wat | 169 +++---`.

| # | site | what |
|---|---|---|
| 1 | `:1209` | comment in `:fanout::sweep-acks` referenced `:fanout::sum-store-calls`, which no longer exists → `:fanout::sample-of`. **The kind of stale pointer the corpus gate cannot catch** (it walks forms, not comments). |
| 2 | `:1956-2024` → `:1956-2010` | the five one-field folds **deleted**, replaced by `:fanout::Sample` (defrecord, 5 i64), `:fanout::empty-sample`, `:fanout::sample-of` (the fold, shape copied from `sweep-of`/`depth-of`), and `:fanout::busy-ms` |
| 3 | after `t-pub0` | `s-pub0` — boundary sample 1 (**new** instrument point; `fill` had none) |
| 4 | after `t-arm0` | `s-arm0` — boundary sample 2 (**new**; `arm` had none) |
| 5 | after `t-drain0` | `sc-before` / `ns-before` / `hn-before` (3 folds) → `s-drain0` (1) |
| 6 | after `t-collect0` | `sc-after` / `ns-after` / `hn-after` (which sat *before* `t-collect0`) **and** `calls` / `ticks` / `store-calls` / `store-ns` (which sat after it) → `s-collect0` (1) plus four `:fanout::Sample/…` field reads. **7 → 1.** |
| 7 | after `t-stop0` | `s-stop0` — boundary sample 5 (**new**; `stop` had none) |
| 8 | after `t-end` | `s-end` — boundary sample 6, the only one inside no phase |
| 9 | phases format string | `+fill-busy-ms +arm-busy-ms +collect-busy-ms +stop-busy-ms`; `drain-busy-ms` kept in place and in name |
| 10 | phases args | `:dsc` / `:dsms` re-pointed at `s-drain0`/`s-collect0`; `:dbms` now `(:fanout::busy-ms s-drain0 s-collect0 m)`; four new busy args |

**Ten fold call sites re-pointed; six sample sites created.** Nothing outside `circuit.wat` was
forced — no `src/` change, no other `.wat`, no test edit. The compiler and the corpus gate were the
census and both came back clean.

---

## STOP-2 — checked, did not fire

All ten readers mapped onto boundaries that already existed:

| reader | needed at | boundary used |
|---|---|---|
| `sc-before` / `ns-before` / `hn-before` | `t-drain0` | `s-drain0` |
| `sc-after` / `ns-after` / `hn-after` | end of drain | `s-collect0` |
| `calls` / `ticks` / `store-calls` / `store-ns` | run totals, read at `t-collect0` | `s-collect0` |

⭑ The one placement decision worth naming: the three `*-after` reads used to sit **immediately
before** `t-collect0` (so their cost fell in `drain`) while the four total reads sat **immediately
after** it (so their cost fell in `collect`). Both groups now collapse onto `s-collect0`, which sits
immediately **after** `t-collect0`. `t-collect0` itself did not move, so the drain/collect boundary is
where it was; what changed is that `drain` no longer pays for three of the harness's own reads
(≈ 33 ms) and `collect` pays for one instead of four. **No mid-phase sample was added, and no reader
needed a value at a point where no boundary existed.**

---

## Grade

**The mechanism landed; the row it was built to move did not.**

- ✅ 12 of 13 rows pass. The five one-field folds are gone, 10 round-trips per queue are 6, all five
  phases carry a busy figure, and the identities hold with remainder 0 on 60/60 tier rows.
- ⛔ **Row 11 fails and STOP-3 fires.** `collect` is unmoved. The stone's stated purpose — *"~8.6 s of
  a 22.5 s run is measurable waiting; remove it and the run is ~14 s"* — is **not supported**. The
  measurable waiting attributable to subscriber-queue round-trips in `collect` is ~45 ms, not 4583.
- ★ **The instrument earned its place anyway, by refuting the fix it shipped with.** Without
  `collect-busy-ms` and the `arm` calibration, "collect didn't drop" would have been a shrug about
  noise. With them it is a number: 4.5 % of `collect` is queue handler time, and one round-trip costs
  2.8 ms.

## What this hands the next stone

1. **`collect`'s ~4.8 s is not queue latency.** `collect-busy-ms` = 220 ms bounds the `qclients`
   share. The unexamined candidates are `collect-stop` (12 worker processes returning 8000 `Outcome`
   records) and `summarize` folding those 8000 records — both interpreted wat over a large collection,
   which is the class the builder's objective actually names. ⚠ **Unmeasured. Measure it before
   claiming it** — this stone is itself the case for that rule.
2. **A per-round-trip price now exists: ~2.8 ms at this topology.** Any future "N round-trips are the
   cost" claim can be multiplied out *before* the edit, which is what would have caught this one.
3. **The busy metric generalises for free.** `:fanout::busy-ms` takes any two samples; a new phase
   boundary costs one `sample-of` (~11 ms at m=4) and gains a busy figure. But the same 11 ms is why
   a *mid*-phase sample is still forbidden.
4. **`fill-busy-ms` = 613 of `fill`'s 2615 ms (23 %).** `fill` is the DESIGN's named next stone, and
   it now arrives with an instrument: 2002 ms of it is *not* subscriber-queue work.
5. **The poll loop supplies ~95 % of all `Queue/stats` traffic**, and it is still cut. If stats
   round-trips are ever worth attacking, that is where they are — not in the harness's boundaries.

---

# ⭑ THE ORCHESTRATOR'S GRADING — my own runs, my own reads

**Floor from `.floor/2026-09-10T01-03-43Z/clean.log`:**
`Summary [ 480.739s] 5237 tests run: 5237 passed (7 slow), 22 skipped` — 0 failure tokens, no
`ARM.txt`. Chaos `7 tests run: 7 passed`. **Row 3 verified by me:** `5 1 0 32 false 0` → rc 2,
`drained-stalled`. **Row 10:** `circuit.wat` + the SCORE.

My own run reproduces the new instrument: `collect=5141`, `collect-busy-ms=208` — **4.0 %**.

## ⛔ ROW 11 FAILED. MY PREMISE WAS WRONG BY 40×

I claimed `collect`'s 4583 ms **is** round-trip latency. The executor's `arm` calibration is the
cleanest measurement in this campaign: `arm` gained **exactly one `sample-of` and nothing else**,
`6.7 → 18.0 ms`. So one sample at m=4 costs 11.3 ms ⇒ **~2.8 ms per `Queue/stats` round-trip**, the
four folds in `collect` were worth **~45 ms**, and the entire ten-fold defect was **~113 ms of a
22.6 s run.**

★★ **And the instrument proved the premise impossible before the timing did.**
`collect-busy-ms = 208–220 ms` means only ~4 % of `collect` is subscriber-queue handler time *at all*
— so removing every queue round-trip could not have removed 4583 ms, whatever the round-trip price
turned out to be. That is what an instrument is for, and I built the stone without it.

★ Two corrections to me that I accept: the count is **10, not 11** (`sum-store-calls` is called 3×,
not 4×), and **the poll loop supplies ~95 % of all stats traffic** (~360 `count-calls` per sub queue
per run against the harness folds' 20) — the observer effect I explicitly cut from the stall stone,
now quantified.

## ⭑⭑⭑ SO I MEASURED WHERE `collect` ACTUALLY GOES — AND IT IS NOT MESSAGES

The SCORE's unmeasured pointer was `collect-stop` over the worker processes. One run discriminates it:
hold the workload at **8000 deliveries** and change only the consumer count.

| | collect | collect-busy-ms | distinct / dup |
|---|---|---|---|
| j=3, **12 workers** | **4786** | 215 | 8000 / 0 |
| j=2, **8 workers** | **4019** | 102 | 8000 / 0 |
| j=1, **4 workers** | **2079** | 12 | 8000 / 0 |

**Identical workload, 2.3× less `collect`.** `collect` scales with **worker count, not records** — it
is process teardown, the same class as `setup`'s per-process spawn.

## ⭑⭑⭑ WHICH MAKES THE HEADLINE: 76 % OF THE RUN IS PROCESS LIFECYCLE

```
setup    12426 ms   per-process SPAWN      ┐
collect   5141 ms   per-process TEARDOWN   ┘ = 17567 ms = 76 % OF THE RUN

fill      2668 ms   sub-queue busy  622  (23 %)   ┐
arm         17 ms                    13           │
drain     2440 ms                  1484  (61 %)   │ the MESSAGING PATH = 5397 ms = 24 %
stop       272 ms                     5           ┘   of which 2124 ms is measured sub-queue work
```

★★ **The builder ruled `setup` out of scope as cold boot. `collect` is the same cost at the other end
of the process lifecycle** — teardown, not message processing. On the same reasoning it is arguably
out too, and that is a ruling I owe rather than take: it decides whether the in-scope budget is
**5.4 s** (messaging only) or **10.5 s** (messaging + teardown).

★ Either way the objective is much closer than this morning's arithmetic said. If `collect` is out,
the entire messaging path is **5.4 s of a 22.9 s run**, of which **2.1 s is measured work** — so
**~3.3 s of removable non-work remains**, not the ~8.6 s the DESIGN projected.

⚠ And `collect-busy-ms` bounds **sub-queue** handler time only. It does not follow that the other
96 % is idle waiting — worker processes, IPC payload and interpreted folding are all outside it. The
metric answers a narrower question than "is this phase interpreter-bound", which is the fifth time
today I have had to write that sentence about an instrument.

## Where the messaging path's non-work actually is

★ `fill` is **23 % busy** — 2046 ms of its 2668 is not sub-queue handler time, and `p=1` means one
serial publisher issuing 200 batched calls. That is the next stone and it is now the *largest*
in-scope non-work term, ahead of `drain`'s 956 ms.

## Grade

`1 ✅ (60/60 tier rows, remainder 0) · 2 ✅ · 3 ✅ (run by me) · 4 ✅ with my count corrected 11→10 ·
5 ✅ · 6 ✅ · 7 ✅ · 8 ✅ · 9 ✅ · 10 ✅ · 11 ⛔ FAIL, STOP-3, my premise refuted 40× · 12 ✅ · 13 ✅`

**STOP-3 firing is the whole value of this stone.** It cost me a hypothesis and bought three facts:
the price of a round-trip (2.8 ms), the poll loop's 95 % share, and — from the follow-on — that 76 %
of the run is process lifecycle. The 113 ms the change actually saves is real but trivial; the
instrument it added is what re-ranked everything.

★ And the change **lands** rather than reverting: `arm` 6.7 → 18.0 ms is a deliberate, named price for
instrumenting a phase that had none, and the five per-phase busy figures are the finish-line
instrument the objective requires.
