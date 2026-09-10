# DESIGN — every round-trip is counted

## Why

**Builder's ruling:** *"we need to build this out such that it's assumed to be networking first, and IPC
exists to simulate networked apps on a single machine… writing edn over a tls channel instead of unix
domain should be moot to the app layer."*

Under that ruling, **the number of times a run crosses a process boundary is the dominant cost**, and
it is currently invisible. Measured RTTs for the same 5804 counted round-trips:

```
IPC unix socket (measured)  2.8 ms/rt      LAN 1GbE      1.0 ms/rt
TLS same host               0.5 ms/rt      cross-AZ      5.0 ms/rt
                                           cross-region 60.0 ms/rt   → minutes per run
```

⛔ **And its invisibility has already produced three wrong conclusions — all mine, all today:**

1. **`one sample per boundary`** (`202763705`) — I graded the win as *"~113 ms of a 22.6 s run,"*
   trivial. Those 40 round-trips at 60 ms RTT are **2.4 s**. Under-credited by ~20×.
2. **The poll loop's ~95 % share of all stats traffic** — cut as out of scope **twice**. Under
   networking-first it is the single largest item in the system.
3. **`collect` asking each worker twice** — dismissed as tidy-up; it is a round-trip budget decision.

Each used the *simulation's* economics to settle an *app-layer* question. That is the same defect class
as the undocumented timing constants, and I committed it three times while diagnosing others for it.

## ⛔ And my own count was more than 2× short

I reported **~5804 round-trips per run** at `2000 4 3` and labelled it "a floor". It is a bad floor.
Only **three** counters exist in the whole harness — `store-calls`, `queue-receive-calls`, `poll-calls`.
**Uncounted:** every seen-store call, every worker call, every topic call.

`:fanout::Seen/check` is called per delivery (`circuit.wat:400`) and `seen-recorded=8000` at
`2000 4 3`. **So at least 8000 round-trips — nearly 2× the entire counted `store-calls` figure — do not
appear in any number this harness has ever printed.** The real floor is **≥ 13 800**.

★ That is why this stone is an instrument and not an optimisation: **every conclusion about round-trips
so far, including all three above, was drawn against a count that was less than half the truth.**

## What it delivers

A **round-trip budget** on the phase line: the count of process-boundary crossings per run, attributed
by **peer class** — store, queue, seen, worker, topic — so a reader can price a run at any RTT and see
*which boundary* to attack.

## The one contract decision

⛔ **The counters must add ZERO round-trips.** This is the trap the whole session has been teaching:
`sample-of` was built because four folds each cost a round-trip to read one field, and adding four
`sum-handler-ns` samples would have inflated the phases they measured. So:

- A caller counts **its own** calls in local state — free.
- A callee's internal calls (a worker's seen-store traffic) ride a reply the harness **already makes** —
  the `:stop` projection, which is author-chosen and already returns final state. **One round-trip
  already spent, more data on it.**
- ⛔ **No new round-trip may be added to measure round-trips.** If a count cannot be obtained free, it is
  reported as **unknown**, explicitly, rather than obtained expensively. An honest gap beats a
  self-inflating instrument.

★ And that composes with a fix already named: folding the workers' `disrupts` into the same `:stop`
projection collapses `collect`'s two questions per worker into one.

## Out of scope = rejected

- **Reducing any round-trip count.** This stone counts; it does not optimise. The re-ranked list —
  the poll loop's 95 %, `collect`'s double question, the store's per-delivery traffic — is the follow-on,
  and it is *decidable* only once the budget exists.
- **Choosing the timing constants' network referents.** The related ruling (a constant should be declared
  at its network value with the IPC value a visible override) is a separate stone. Named, not done.
- **`fill`, `drain`, `setup`, the chaos work, tier-1 fault injection.**
- Any change to `sqs.wat` or `wat/`. If the queue must report something new, that is a STOP.

## ⚠ What must not be claimed

⚠ **This makes nothing faster.** A SCORE reporting a speedup has measured the wrong thing; the phase
timings should be unchanged within noise, and if they are not, the instrument is costing something and
that is a finding.
⚠ **A round-trip count is not a network cost model.** It is a count. Pricing it at an RTT is arithmetic a
reader does, and the serialised arithmetic above overstates by whatever concurrency is achieved —
measured ~3× across 24 processes. **The SCORE must state that, not bury it.**
