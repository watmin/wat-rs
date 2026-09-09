# BRIEF v2 — `vis` is swept, not chosen

**v1 was drawn before eight stones landed under it. Re-verified against the tree; the DESIGN stands,
the BRIEF did not.** Two corrections:

1. **v1's blast radius said "circuit.wat only".** True about *files*, silent about *sites*. Adding a
   parameter changes `run-with`'s arity from **14 to 15** and touches **seven** call sites. This arc
   has already paid for that exact confusion once — a change declared as two files that was eight
   files and fifteen sites.
2. **v1 had no way to drive the sweep.** The only external entry is the argv path, whose contract is
   `usage: circuit.wat [n m j sub-cap fill-first?]` — five slots. A parameter on `run-with` that argv
   cannot reach is a knob with no handle.

## Verified rooms (this session, current tree)

```
circuit.wat:2084-2091   defn run-with — 14 params, ending sub-cap / fill-first?
circuit.wat:2098-2101   vis = 200000000 if any drop rate else 1000000000000
circuit.wat:522         limit-ms (:wat::i64::/ vis 1000000)   ← the coupling, intact
circuit.wat:2503        usage "usage: circuit.wat [n m j sub-cap fill-first?]"
circuit.wat:2508-2514   the argv entry — reads argv 2..6
```

**Seven call sites:** `:2389` `:2394` `:2400` `:2407` `:2425` `:2428` `:2508`.

## Read in order

1. **`circuit.wat:2084-2101`** — the signature and the `vis` conditional you are parameterising.
2. **`circuit.wat:2503-2514`** — the argv entry. **`sub-cap` and `fill-first?` are the exemplar**: a
   prior stone extended this same list, and the shape to copy is right there.
3. **`circuit.wat:520-522`** — the coupling `limit-ms = vis / 1000000`, which is *why* the bound is
   1 000 000 ms today. You are not changing it; the fork's second branch would.

## Implementation sketch

```wat
;; run-with gains a 15th parameter, LAST so positional callers read naturally:
   vis-ms <- :wat::core::i64

;; and the vis conditional becomes: 0 means "exactly today's behaviour"
vis (:wat::core::if (:wat::i64::> vis-ms 0)
      (:wat::i64::* vis-ms 1000000)
      <the existing drop?/non-drop conditional, unchanged>)

;; the six existing internal callers pass 0.
;; the argv entry gains a sixth slot, OPTIONAL so today's command line still works:
usage "usage: circuit.wat [n m j sub-cap fill-first? [vis-ms]]"
```

## The sweep

Geometric, across the band the DESIGN derived (~1 s to 100 s), driven from the shell against
`2000 4 3 8192 true`:

```
vis-ms ∈ { 1000, 3000, 10000, 30000, 100000 }
```

**The gate is binary and empirical:** does n=2000 *complete*? Report, per point: completed or not,
`distinct`, `dup`, `drain`, and every `*-exhausted` / `ack-retries` counter.

⚠ **Runtime is the risk.** A completing run is ~40 s; a non-completing one burns the drain's full
patience (~215 s). Five points is **~4 to 18 minutes** of wall clock. Run the sweep on a quiet box and
**do not** run it beside a floor.

## Blast radius

`wat-scripts/fanout/circuit.wat` only — but **all seven call sites plus the usage string**. No
`sqs.wat`, no `wat/`, no `StatsResponse`.

## STOP triggers

**STOP-1** — if `vis-ms = 0` cannot mean *exactly* today's behaviour, STOP. Every existing caller and
every floor test must be byte-identical; that is what makes this safe to land before the sweep runs.

**STOP-2** — if the no-argument `circuit.wat` invocation differs in **any** reported field, STOP.

**STOP-3** — if the sixth argv slot cannot be optional, STOP. Breaking the existing command line to
add a knob is not a trade this stone makes.

**STOP-4** — do **not** decouple `limit-ms` from `vis`. That is the fork's *second branch* and doing it
now pre-empts the measurement that decides whether it is needed.

**STOP-5** — if no swept value completes, that is a **result, not a failure**. Report the table.

**STOP-6** — on any red floor arm: capture whole, name the arm, do not re-run.

## What "done" looks like

`run-with` takes `vis-ms`; `0` reproduces today exactly; all seven call sites compile; the argv entry
accepts an optional sixth slot and the five-slot command line is unchanged. The sweep table is on
disk with one row per value. Floor Summary reads 5235 / 22 skipped / 0 FAIL / 0 TIMEOUT on a quiet box.

The SCORE reports the table **and** names which branch of the fork it lands in: a `vis` exists that
completes n=2000 (the coupling was fine, the value was wrong), or none does (`limit-ms = vis / 1e6` is
the wrong coupling — a finding no choice of constant could have produced).
