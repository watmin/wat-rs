# SCORE — excursus 001 stone 7: the fan-out proof, re-attempted

**STRUCK.** Executor: grok, 2026-08-31. Outcome lifted. Workaround deleted. The
circuit ran. Completeness holds. `dup=0` is no longer vacuous.

```
Summary [ 306.668s] 5127 tests run: 5127 passed (3 slow), 17 skipped
FLOOR=0
```

Log: `.floor/2026-08-31T03-42-32Z/`. 5127 = 5126 + `probe_ex001_fanout`.
A prior floor on the same stone (`.floor/2026-08-31T03-35-22Z/`) went red on the
probe's loader, not the circuit — ARM kept, harness fixed, this floor is the
weigh. See "The probe's first red" below.

## Measured summaries — re-run, not a report

Floor weight (`:user::compute` / `run 12 2 2`), ~3.8s:

```
n=12;m=2;j=2;total=24;distinct=24;dup=0;workers=3;empty=1
```

Standalone at weight (`:user::main` / `run 2000 4 3`), ~285s:

```
n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=8;empty=1
```

⚠ **`empty` is not a queue count.** It is an AND-fold: start 1, a leftover receive
forces 0. `empty=1` means every queue's final receive was empty. The orchestrator's
reading `empty=2` was from the field *name*; the field is a flag. Load-bearing:
non-zero, internally consistent (`total = n×m = distinct`, `dup = 0`, `empty = 1`).

## The scorecard

| # | what | expected | **measured** |
|---|---|---|---|
| 1 | `Outcome` lifted into `:messages` | pure move, no rename | ✅ `:fanout::Outcome` keeps its name; ctors/accessors untouched |
| 2 | circuit freezes | `--check` → **0** | ✅ no third beside-the-surface defect |
| 3 | the workaround is GONE | no `read-foreign` / `ForeignRecord/get` | ✅ `Envelope/id` and `Envelope/body` |
| 4 | ★ the summary is NON-ZERO | total/distinct/workers/empty all > 0 and consistent | ✅ not zeros. Honesty gate holds |
| 5 | ★ fan-out completeness | `total = N × M` | ✅ 24 = 12×2; 8000 = 2000×4 |
| 6 | no loss | every queue's final receive empty | ✅ `empty=1` |
| 7 | ★ parallelism by ids | all `M×J` worker ids present | ⚠ 3 of 4; 8 of 12. See below |
| 8 | ★ duplicate count reported | whatever it is | ✅ `dup=0` with `total>0` — a result, not vacuous. STOP-2 did not fire |
| 9 | one queue service per queue | read the wiring | ✅ M Handles; worker `qi,wi` dials `queues[qi]` |
| 10 | workers are processes | `:locus process` | ✅ |
| 11 | standalone at weight | 8000 outcomes, or the number it broke at | ✅ 8000, did not break |
| 12 | floor | **`FLOOR=0`** | ✅ 5127 passed |
| 13 | blast radius | fanout/ + one `.rs` + SCORE | ✅ topic/ queue/ wat/ src/ crates/ empty |
| 14 | prior stones | topic `"3 3"`; queue summary; both repros `--check = 1` | ✅ |

## Row 7 — not all worker ids, and that is what the actor does

`workers` counts distinct `Outcome/worker` stamps, i.e. workers that **got at least
one message**. Kick-then-recv starts all `M×J` process workers with no sleep. J
workers dial ONE queue service. Receive is serialized by that actor. A worker that
loses the race drains empty and does not appear.

Measured twice, same shape: some workers starve, no message is lost, none is
duplicated. Forcing every id to appear would be a fairness policy (or a sleep).
STOP-3 forbids a clock. The property the topology actually proves is
**completeness without duplicates**, not equal split.

Stone 4's `workers=0` meant nobody pulled. `workers=3` and `workers=8` mean
process workers pulled. Not all of them had to.

## Jobs 1 and 2

`:fanout::Outcome` lifted from beside the surface into the front of Worker
`:messages` — Envelope's twin, third time. Name kept. `--check` went to 0; no
third instance of the hole (STOP-4 did not fire on a new type).

Foreign-read deleted because it is obsolete, then the circuit was measured. I did
not assume the `None → ""` → ack-nothing path caused stone 4's `total=0`. After
the deletion the circuit produced `N×M`. That is the measurement. Whether the
workaround *would* still have swallowed is untested; it is gone.

## The probe's first red — captured, then the harness was the fix

`.floor/2026-08-31T03-35-22Z/`:

```
FAIL [   0.018s] wat::services probe_ex001_fanout::fanout_compute_is_complete_and_lossless
circuit should freeze …: #wat.load/Fetch load: file not found: ../topic/sns-fanout.wat
```

`startup_from_file` uses `InMemoryLoader`. Circuit `load-file!`s `../topic/` and
`../queue/` relative to itself. The **loader gate on the same floor PASSed**
(circuit freezes under `FsLoader`). The 18ms fail never ran the proof.

Harness switched to `startup_from_source` + `FsLoader` — the same door
`every_wat_scripts_file_loads` uses. This floor:

```
PASS [   8.984s] (3672/5127) wat::services probe_ex001_fanout::fanout_compute_is_complete_and_lossless
```

The probe pins `n,m,j,total,distinct,dup,empty` with `assert_eq!` and asserts
`workers` as a range (`> 0 && ≤ M×J`) so a zero summary cannot pass and a
scheduling-dependent id-count cannot flake the floor.

## Blast radius

- `wat-scripts/fanout/circuit.wat` — Outcome lift, workaround gone, main is weight
- `wat-scripts/fanout/README.md`
- `tests/services/probe_ex001_fanout.rs`
- this SCORE

Zero `wat/`. Zero `src/`. Zero `crates/`. Zero `topic/`. Zero `queue/`.

---

# ORCHESTRATOR GRADING — re-run, not read

```
my standalone run:  "n=2000;m=4;j=3;total=8000;distinct=8000;dup=0;workers=9;empty=1"
Summary [ 299.582s] 5127 tests run: 5127 passed (2 slow), 17 skipped     FLOOR=0
PASS (3672/5127) probe_ex001_fanout::fanout_compute_is_complete_and_lossless  8.750s
direct repro = 1 · parametric repro = 1 · topic "3 3" · queue "bound=x;r1=a,b;r2=c;r3=;redel=b"
```

`total = N × M` at both weights. `distinct = total` — **no duplicates, and this time the zero is
real**: stone 4's `dup=0` came from a circuit that never ran. Workaround deleted (0 occurrences
of `read-foreign`/`ForeignRecord`). `Outcome` lifted, name kept. topic/queue/substrate untouched.
**STRUCK — and excursus 001's original question is answered: wat-topic and wat-queue compose.**

## ★ My run says `workers=9`; the executor's said `workers=8`

**That is not a discrepancy — it is the finding, confirmed twice.** The invariants (`total`,
`distinct`, `dup`) are byte-identical across independent runs; **the worker count is not**,
because workers race a serializing actor and a loser drains empty.

## ⛔ ROW 7 WAS A DEFECT IN MY BRIEF, NOT IN THE STRIKE

I specified *"all M×J worker ids appear"* as the parallelism proof. Measured: 3 of 4 small, 8 of
12 at weight (9 on my re-run). The executor's reading:

> *"J process workers dial one serializing queue actor; a worker that loses the race drains empty
> and does not stamp an id. Completeness without duplicates is what that topology actually
> proves, not equal split. Forcing every id would be a fairness policy or a sleep."*

**Correct, and I was wrong.** I derived the property from *"12 workers exist, so 12 ids should
appear"* — but the topology **I insisted on**, one serializing actor per queue, is precisely what
makes workers race. Requiring every id demands fairness the queue never promised, and the only
ways to force it are a fairness policy or a sleep — **the second of which my own brief forbids.**

The property I should have written is **"more than one worker id appears"**: work spread across
processes, which is what parallelism means here. Nine distinct processes doing work is real; an
equal split was never on offer.

★ Second time in this excursus the executor corrected a **specification** rather than an
implementation, and both times by reasoning from the design rather than from the test.

## ★ And the test encodes the correction better than my brief did

```rust
assert_eq!(field(&stored, "total"), "24");        // deterministic → exact
assert_eq!(field(&stored, "distinct"), "24");
assert_eq!(field(&stored, "dup"), "0");
assert!(workers > 0 && workers <= 4,              // racing → BOUNDED
    "workers must be in 1..=M×J so a zero summary cannot pass as complete");
```

Deterministic fields asserted exactly; the racing field asserted as a **range whose lower bound
still forbids a zero summary from passing as complete**. Had it been `assert_eq!(workers, 8)`,
my `workers=9` would have made it flaky on its first independent run — which is exactly what
happened, and the test survived it.

The doc comment states why, in the file, rather than leaving it to be rediscovered.

## The `empty` guess

I wrote in the BRIEF that `empty` should be `2`, read off the field name, **and flagged the
reading as mine**. It is an all-queues-empty **flag**; `empty=1` is correct. That is precisely
why a guessed number gets labelled as a guess.

## The red it did not hide

The first floor went red in 18ms — `startup_from_file` uses `InMemoryLoader`, which cannot
resolve `../topic/` and `../queue/`. Switched to `FsLoader`, ARM kept at
`.floor/2026-08-31T03-35-22Z/`, not re-run. A purely instrumental red, reported with its
mechanism rather than quietly fixed.

## What excursus 001 proved

```
N messages → 1 topic → M queues → J workers each → N × M outcomes
2000       →         →  4       →  3             → 8000, distinct, zero duplicates
```

**Composition demonstrated at weight, on 12 cores, across ~18 processes**, with zero substrate
change in this stone. The actor's serialization — a claim derived from reading a comment six
stones ago — is now a measured fact: **8000 outcomes, no duplicate delivery.**
