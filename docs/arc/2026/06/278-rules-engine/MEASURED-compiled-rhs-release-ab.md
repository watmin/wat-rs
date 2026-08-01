# MEASURED — the compiled RHS (#43), A/B in RELEASE — the row the EXPECTATIONS left open

`EXPECTATIONS-compiled-rhs.md` row 7 said *"the fire, A/B in ONE batch — recorded with ranges"* and
carried no threshold, deliberately: *"PERF: RECORDED, NOT GRADED."* This is that row, filled — and it
had to wait for a **release** instrument, for the reason the seam flagged.

## Why a test-build A/B was invalid here (and was NOT for any other stone this arc)

The compiled path emits **1 mark pair per fact against the interpreter's 4**. On the 40,000-pair
fanout cell that is ~120,000 mark pairs that simply cease to exist on arm B. At the seam's own
~120–165 ns/pair calibration, that is **14–20 ms of INSTRUMENT removal** — plausibly several times
the real effect, and invisible from inside the test build. Every other A/B this arc was safe because
both arms carried identical marks; this one is the exception.

## The instrument

`target/release/wat` (a NON-test binary — `phase_start`/`phase_end` are `cfg(not(test))` no-ops there,
so no marks exist on either arm) running `wat-scripts/perf/grid/fanout.wat`, stdin `[40000]`
⇒ keys=100 × fanout=20, R4's 40,000-pair cell. The workload times **only the fire**
(`n0` → `fire-rules` → `n1`) and prints `:native-ns`.

**Arm A** = the interpreter path, produced by a one-line toggle at the cache construction
(`rhs.iter().map(|_| None)`), which forces every form down the `build_insert_fact` fallback —
byte-for-byte the pre-#43 derivation path, everything else identical. Verified semantics-preserving:
both arms' `:derived` sets hash identically (`c067a23f…`, `:native-ns` stripped).

**Method:** two prebuilt binaries, no rebuild between arms; **interleaved A,B,A,B…**; 1-minute
loadavg gated **< 1.5** before the first measurement.

## The result — 12 interleaved pairs

```
paired Δ (A−B), ms:  21.95  9.12  4.31  2.40  −2.45  12.74  −0.27  7.81  −7.21  2.63  11.87  3.67

A sorted: 54.20 54.27 54.99 55.06 55.09 55.27 56.60 60.77 62.14 62.16 62.97 74.94   median 55.94
B sorted: 48.79 48.90 49.40 50.97 51.87 52.37 52.99 53.85 55.36 56.64 58.49 62.26   median 52.68
```

- **9 of 12 pairs favour the compiled RHS; 3 go the other way.**
- **Median paired delta 3.99 ms** (mean 5.55) on a ~55 ms fire — roughly **7%**.
- Distributions **overlap heavily**. Sign test on 9/12 ≈ p 0.07 — suggestive, **not conclusive**.

**Recorded as a range, not a headline: ~4 ms median, direction consistent, significance marginal.**

## The small-sample trap this caught, kept visible

A FIRST batch of 5 interleaved pairs gave a median delta of **~19 ms** and would have been reported.
Twelve pairs show why that was wrong: arm A has a slow tail, and five samples caught it three times.
At n=12 the slow cluster is a single outlier (74.94), not a mode. **The 5-pair number was ~5× the
truth.** A small sample of a heavy-tailed arm does not merely add noise — it hands you a specific,
flattering, reproducible-looking figure.

## What is NOT in doubt

The **mechanism** is proven by COUNT, not by timing, and is gated in the floor:
`fanout_rhs_key_alloc_census` asserts `match:key-alloc == 0` (was 120,000) **and**
`prod:derivations == 40_000` as a non-vacuity guard — a dead fire would also read zero allocations,
and without the second assertion the two would be indistinguishable. 240,000 heap allocations
(120,000 `String` + 120,000 `Arc`) do not happen. That claim needs no stopwatch and never did.

## Bounds on this claim

- ONE cell (fanout, 2-condition, 40,000 pairs). Not swept across the grid.
- ONE box, a live desktop gated to loadavg < 1.5 — not a quiet benchmark machine.
- The two batches (5 pairs, then 12) are separate batches; only the 12-pair batch is quoted.
  Pooling them would be the cross-batch comparison the arc has already ruled void.
