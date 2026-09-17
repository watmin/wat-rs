# FINDING — promoting a SERVICE into the stdlib taxes every process start, and the freeze is why

**Filed 2026-09-17** from the blocked strike of `the-queue-matures-into-the-stdlib`. The promotion is
**correct and NOT LANDED**: it is parked, pushed, on branch
**`queue-promotion-blocked-on-startup-cost`** (`6136d144f`). `sns-sqs` is back at its last green code.

## What happened

The queue split was right — 21 definitions moved, all 23 `:user::` names correctly stayed, codemods
recorded and idempotent, load order verified, five rust probes 13/13, circuit **byte-identical**. And
the floor went **RED twice**, both captured, neither re-run away:

```
.floor/2026-09-17T04-25-38Z   610.015s  5272 passed, 2 timed out   ARM.txt
.floor/2026-09-17T04-41-40Z   610.016s  5272 passed, 2 timed out   ARM.txt   (of record)
  TIMEOUT   35.123s  wat::value probe_rational_C5c_nan_unordered::c5c_nan_is_unordered_gate
  TIMEOUT  610.010s  wat::lint  wat_scripts_fixes_load::every_wat_scripts_file_loads…
```

⭐ **Every assertion in both arms PASSED. They were killed for being slow.**

## The measurement, and it is not what the first reading said

Bisected by isolating exactly one variable — comment out only the `wat/queue.wat` manifest row,
rebuild, time, restore (sha256-verified), rebuild:

```
bare CLI startup            0.430 s → 0.556 s        +0.126 s   +29%
orchestrator's own          0.572 / 0.550 / 0.558 s              (agrees)
arm A, 28 startups          26.229 s → 34.903 s      +0.310 s/startup under contention
arm A in ISOLATION          14.020 s                             ⇒ ~2.5× contention amplification
arm B                       687 startups and almost nothing else (0.546 s/file vs 0.556 s bare)
```

⛔ *"Passes in isolation"* is **not** offered as a disposition — it is forbidden as one. It is recorded
as a fact about amplification.

### ★ THE NUMBER THAT REFRAMES IT

```
stdlib today   22,979 lines / 55 files   → startup 0.430 s
queue adds      1,910 lines  (+8.3%)     → startup 0.556 s  (+29%)
```

**8.3 % more source bought 29 % more startup.** The cost is therefore **macro expansion, not parsing**.
A `defsurface` + `defservice` pair expands into a client method per op, a serve loop, `child-main`, and
the `Status`/`Reply` enums — and **the frozen stdlib embeds SOURCE, so every process re-expands all of
it, every time.** The marginal cost of a *service-bearing* stdlib file is several times the average
file's.

⚠ **My first reading of this was wrong and is corrected here.** I initially blamed the load gate's
**687 process startups** (standalone 375.03 s, ~100 % startup cost, 8 % headroom against its 600 s
kill). The gate is the **detector**, not the cause. Batching it would have hidden a genuine **29 %
startup regression from every wat program** — CI merely noticed first. A test-harness fix aimed at a
substrate cost is how a regression becomes folklore.

## What this means for the roadmap

⛔ **Stdlib maturation is currently capped by the freeze's design, not by whether a library deserves
promotion.** Queue deserves it — *"battle tested … used in anger revealed and demanded corrections for
several substrate items"* is precisely the case for it. But each service-bearing promotion is paid by
every process that ever starts, including production, and `topic` is **larger**.

★ **The enabling stone is to make the freeze PRE-COMPUTE rather than embed source** — serialize the
expanded, checked environment at build time so a process start maps it instead of re-deriving it. That
would make promotion nearly free **and** make every wat program start faster, which is a user-visible
property and not merely a CI cost. It is arc-level and it is the builder's ruling.

Alternatives, stated so the ruling is informed, not pre-empted:

| option | what it really does |
|---|---|
| **pre-compute the freeze** | fixes the cause; unblocks all future promotion; helps every user |
| batch the load gate | hides the regression from CI and ships +29 % startup to users |
| raise the timeouts | the same, with less effort and more dishonesty |
| leave queue in `wat-scripts/` | no regression, no maturation — the status quo |

## Kept from the blocked strike, because it outlives the stone

⭐ **The executor rejected this DESIGN's split on all 13 "CLIENT API" names, with a measurement.** Every
failure arm of those 11 panic-gate helpers is `assertion-failed!`, and the two real consumers —
`circuit.wat` (189 refs) and `sns-fanout.wat` (194) — call **zero** of them, having written their own
outcome-facing versions. Promoting a panicking gate to manifest position 50 would have enshrined the
exact ungraceful failure this excursus is a crusade against. The real client API is the
`defsurface`-generated `Queue/send|receive|ack|stats`, which did move.

Also worth keeping:
- the codemod prefix `":queue::"` is a **silent zero-edit no-op** (right-boundary rule), and `":queue"`
  **corrupts** the live `:queue` kwarg — `":queue:"` is the one that works;
- the DESIGN named **1** `load-file!` to delete; there were **18**;
- `":wat::"` is reserved to the stdlib, so the rename is illegal *until* the split — **there is no green
  state in between**, which the executor stated rather than implying a verified sequence;
- the DESIGN's "147 query references" is **unreproducible** (143/146/174 on re-measurement) — I had
  copied it into a brief before measuring it.
