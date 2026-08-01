# DESIGN-STONE — A0 and A1 into the grid, with the accuracy differential they never had

> **Origin (2026-07-31):** asked what the matrix measures *at the rete level*, the roster came back
> **A2–A8 — seven axes of nine.** A0 (deep-cascade) and A1 (fanout / low-selectivity) exist as scripts,
> were speed-measured in **R4**, and the grid DESIGN's own table says of both: *"✅ speed done; **ADD**
> the `:derived`-set accuracy differential."* Neither ever got one. `grep deep-cascade run-all.sh` → 0.

## Why this is not bookkeeping

**A1 at 40k is the only recorded Clara win in the project's history** (`REALIZATIONS.md:201`):

```
fan-out / low-selectivity joins:   16k ours 1.17x  ·  20k ours 1.09x  ·  40k  CLARA 1.4x
```

R4 explained it as a deliberate cost — *"its residual is the per-token support-chain provenance we
deliberately carry for the deferred streaming engine. A conscious keep, not a loss."* That may still be
true. But it is a **claim from months ago about an engine that has been rewritten three times today**,
and it is the one axis absent from the board we just called a clean sweep.

So "21 of 21" is seven axes of nine, and **the missing two include the only place Clara ever beat us.**
That is the finding; adding the axes is the remedy.

A0 matters for a second reason: it is the **depth** axis — cascading derivation chains — and depth is
the complexity dimension the grid currently cannot see (the live axes carry 2–3 conditions per rule).

## What is actually missing — three conformance gaps, not "wire two files in"

Both scripts predate the grid contract (`run-axis.sh`) and violate it:

| | required by `run-axis.sh` | A0 `deep-cascade.wat` today | A1 `matrix/fanout-*` today |
|---|---|---|---|
| stdin | an i64 vector `[n …]` | a `:perf::Params` EDN record | — |
| stdout | `#grid/Result {… :derived <SORTED FACT VECTOR> :native-ns N}` | `{… :derived <a COUNT> …}` | — |
| Clara side | `grid/gen-<axis>.sh` emitting the same `:derived` | `perf/clara/gen-bench.sh`, elsewhere | a static `fanout-clara.clj`, not a generator |

## ★ THE ONE CONTRACT DECISION

**`:derived` must become the canonically-encoded SORTED FACT SET on both sides, or the axis does not
land.**

Today A0 emits `:derived 120` — a *count*. A count cannot detect a wrong answer that happens to have
the right cardinality, which is precisely the class the grid's accuracy differential exists to catch.

**A speed-only axis in the grid is worse than no axis at all**, because `run-all.sh` tallies it and the
"N of N" line then reports coverage the measurement does not have. That is exactly the failure this
session spent the day on — a green number that measures less than it appears to. If the encoding
cannot be made to agree byte-for-byte between wat and Clara, **report the gap and ship nothing**; do
not land a speed number and call the axis covered.

Each axis defines its own injective encoding, as the live ones do (`gen-accum.sh`: `kind*1e15 +
g*1e9 + val`, sorted ascending) so both sides emit an identical vector.

## The ladder — top rung is each script's own documented size

Recorded in `run-all.sh` beside the others, so the run is reproducible:

```
A0 deep-cascade   dial: depth (width fixed)   "10 5" | "20 5" | "30 5"
A1 fanout         dial: items                 "10000" | "20000" | "40000"
```

**A1's top rung is 40,000 deliberately** — that is the exact size R4 recorded as `Clara 1.4×`. The
point of adding this axis is to learn whether that still holds.

## Blast radius

`wat-scripts/perf/grid/` — two new axis `.wat` files (adapted from the existing scripts, not rewritten
from scratch), two new `gen-<axis>.sh`, and two ladder entries in `run-all.sh`. The legacy
`perf/deep-cascade.wat` and `perf/matrix/*` stay where they are; this stone does not delete them.

**No `src/` change. No `wat/` change.** Nothing in the engine moves — this is a measurement stone.

## The gate

1. `run-axis.sh deep-cascade "10 5"` and `run-axis.sh fanout "10000"` each emit a `#grid/Verdict`.
2. **`:accuracy :match` on both** — this is the whole point; a `:MISMATCH` here is a real finding about
   the engine, not a bug in the axis, and must be reported rather than encoded away.
3. The `:derived` vectors are non-empty and their lengths differ between the smallest and largest rung
   (a constant `:derived` would mean the size dial is not reaching the rules).
4. `run-all.sh` with no arguments runs **nine** axes.

## Out of scope = REJECTED (affirmative cuts)

- **Fixing A1 if it comes back `:clara`.** Measuring it is this stone; whether the support-chain
  provenance is still worth its cost is a *separate* ruling, and `DESIGN-STONE-P10-drop-dead-provenance`
  already exists for that conversation. Do not optimize on discovery.
- **Deleting the legacy scripts.** They are R4's record. Leave them.
- **A wide-conjunction axis.** Real gap (nothing on the board exceeds 3 conditions; even the
  third-party Grok reasoner in `tmp/speech-edn-reasoner` tops out at 4 vars), but it needs a rule
  nobody has written. Its own stone.
- **Re-running the whole grid.** That is the orchestrator's weigh after this lands, not part of it.
