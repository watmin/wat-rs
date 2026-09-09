# DESIGN — the cold counters ride one carrier

## Why

`drain` is now the whole cost. After `e0c552bf0` moved the fanout off the publisher, `fill` fell
84 % and `drain` fell 5–6 %, with `drain-store-calls` unchanged (1451→1478 at n=4000). The producer
side is done; the consumer side dominates `total` at every `n`.

And one component of `drain` has a **measured** cost with a **named** mechanism.
`the-store-reports-time-per-operation/SCORE.md:177` is a clean A/B:

```
drain WITH the instrument     1358 / 3178 / 7590
     WITHOUT                  1246 / 2866 / 6843
                              +9.0 % / +10.9 % / +10.9 %   non-overlapping spreads
```

Cause, from that SCORE: *"**allocation** (State 8 fields wider, TakeAcc 4 wider, `take`'s tuple
nested, Stats 8 wider) — no added store call, no added clock read."* This stone reverses part of that
mechanism.

## ⛔ The plan we had written down is WRONG, and the disk says so

Every prior artifact records the fix as *"one carrier record"* for the counters. **The data refutes
it.** `:queue::queue::State` is constructed at **30 sites**; per counter, the split between
*passed through unchanged* and *changed* is:

| tier | fields | changed at | passed through |
|---|---|---|---|
| **HOT** | `handler-ns` | **27** of 30 | 3 |
| **WARM** | `store-calls/ns` 16 · `put-*` 9 · `count-*` 7 · `delete-*` 5 · `receive-calls` 5 · `scan-*` 4 | 4–16 | 14–26 |
| **COLD** | `ticks` 2 · `acks` 2 · `sends-accepted` 1 · `sends-refused` 1 · `redeliveries` 1 · `expired-waiters` 1 | **1–2** | 28–29 |

(`handler-ns`'s 27 is confirmed independently: `grep -c ':handler-ns (:wat::i64::+'` → **27**, and
30 − 3 = 27.)

⛔ **A single 18-field carrier makes 27 of 30 sites WORSE.** Today a rebuild allocates one 29-field
record. Under one carrier, a site that changes any counter allocates a 12-field `State` **plus** an
18-field carrier = 30 fields. Since `handler-ns` is bumped at 27 of 30 sites, nearly every rebuild
pays *more*. The slogan was never measured against the change-frequency it depends on.

★ The win exists only for counters that are **rarely changed**, because those are the ones a
carrier lets you copy by reference instead of field-by-field.

## What it delivers

**The six COLD counters move into one `:queue::Counters` carrier**; `State` goes 29 fields → 24.

- `ticks`, `acks`, `sends-accepted`, `sends-refused`, `redeliveries`, `expired-waiters`
- **172 pass-through copy lines → 30** (one carrier copy per site): **142 lines of pure ceremony
  deleted.**
- Allocation: **≥22 of 30 sites go 29 → 24 fields.** The ≤8 sites that change a cold counter go
  29 → 24 + 6 = 30, i.e. +1 field at a handful of sites against −5 at the rest.

## The one contract decision — and my first answer was wrong

I was going to have `:queue::Stats` gain a nested `counters` field, so the six names are defined
once. **Checking the disk killed it:** `:queue::Stats/<field>` is read **32 times across 8 files**
(`circuit.wat`, `sns-fanout.wat`, and five `scratch-pad` probes). Nesting would break all 32 for
**zero** gain — because `Stats` is constructed at exactly **one** site (`sqs.wat:1319`), once per
`stats` call, so its width never touches `drain`.

⛔ **`:queue::Stats` is therefore UNTOUCHED.** It keeps its flat 19-field shape. `State` holds
`:queue::Counters`; the `stats` handler at `:1319` reads the six out of the carrier and passes them
into the unchanged flat record. **The blast radius stays inside `sqs.wat`.**

★ The elegance of one definition would have cost a 32-site corpus migration to speed up a record
built once. The tax is in `State` — rebuilt at 30 sites on every message — and nowhere else.

⛔ The report LINE format (`circuit.wat:1135`) must not change; the circuit parses it.

## Out of scope = rejected

- **`handler-ns`** — changed at 27 of 30 sites. A carrier makes it worse. Stays flat, affirmatively.
- **The WARM store-op pairs** (`put/count/delete/scan` × `calls/ns`, 8 fields) — changed at 4–9
  sites each and often *together* with `store-calls`. Whether a second carrier pays needs the
  co-occurrence measured, not the frequency; that is a different stone with a different measurement.
  **Not deferred — cut, because this stone's measurement cannot settle it.**
- `TakeAcc` (`sqs.wat:97`) and `take`'s nested return tuple — the other two allocation sites that
  SCORE named. Out: they live inside the waiter fold, not the State rebuild.
- The residual drain **slope**. ⚠ See below — this stone provably cannot touch it.

## ⚠ What this stone CANNOT do, stated before it is measured

`the-store-reports-time-per-operation/SCORE.md:290` says the instrument's cost *"is a **multiplier**,
so every ratio is unaffected"* — `drain` r21 2.340 / r42 2.388 against the baseline's 2.301 / 2.388.
**So this buys LEVEL, not SLOPE.** It will not move the 4.443 superlinearity, and any SCORE claiming
it did is measuring noise.

★ And the honest size: the measured ~10 % covers **+8 State fields AND +4 TakeAcc AND +8 Stats AND a
nested tuple.** This removes **5 net fields from State only.** Predicting a ~10 % recovery would be
arithmetic I have not done. The prediction is therefore deliberately weak and falsifiable: *`drain`
moves in the right direction by a margin whose spreads do not overlap* — and **if it does not move
at all, the allocation mechanism named in that SCORE is wrong, which is worth more than the tidy
result.**

## Baseline to beat, measured on this box this session

`circuit.wat <n> 4 3 8192 true 1000`, box quiet:

```
drain   n=1000  1194 ms    n=2000  2506 ms    n=4000  5305 ms    ratio 4000/1000 = 4.443
setup   12457 / 12477 / 12700 (constant)      fill 1295 / 2583 / 5396 (linear)
```
