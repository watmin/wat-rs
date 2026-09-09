# DESIGN — the slope belongs to a phase

**Measurement only. No code change, deliberately.** The question this arc opened with, finally askable.

## The question

```
setup / fill / arm / drain / collect  at  n = 1000, 2000, 4000,  vis pinned
→ which phase, if any, is superlinear
```

Eleven mechanisms died in this arc chasing *"the drain is superlinear"*. Then the drain turned out to be
**6.4 %** of the run, `fill` **45 %**, `setup` **33 %** — and **no phase's slope has ever been measured.**
The `+55 %` figure that started it came from `n=500→1000`, from single runs, before `vis` was a knob.

## Why it is askable only now

Three things had to land first, and each was drawn for its own reason:

| | |
|---|---|
| `vis is swept, not chosen` (`eccf3dc79`) | n=2000 could not complete at all. `drain ≈ max(work, vis)`, so an unpinned `vis` makes every drain comparison meaningless |
| `every tier reports its backlog` (`96a840db0`) | per-tier arrival/service/refusal, verified across three independent gradings |
| `seen-ids` + `the summary counts with a set` (`0e026d2de`, `707824d86`) | removed **two quadratic terms from the harness itself** — the second measured at **≈4.3 s at n=4000** |

★★ That last one is why the ordering mattered rather than merely looked tidy: 4.3 s of *harness* time would
have landed inside a measurement of *system* scaling with no way to separate the two. **The measurer was
inside the measurement**, and this arc has paid for that four times today.

## `vis` is safe at n=4000, from the code

`circuit.wat:2328` — `poll-until-drained qclients topic (:wat::i64::* n m)`. The attempt bound is
**`n × m`**, so the drain's patience **scales with n** while `vis` stays fixed. At n=2000 the sweep found
every value from 1000 ms to 100000 ms completes; at n=4000 the patience doubles and `vis` does not.

**Pin `vis-ms=1000` at all three points.** It is the smallest swept value, so the drain reports *work*
rather than a timeout — which is the only way its slope means anything.

## ⛔ The method, and the two things that would void it

**≥3 runs per point, report medians and the observed spread.** Not one run. `fill`'s spread is
**~1.8 s** (15493–17333 across today's runs) and `collect`'s is **~800 ms**. A ratio computed from single
samples is how the `+55 %` claim was born, and this arc has an earned rule against banding a number
measured once.

**Same box, quiet, one at a time.** Every number here is a duration, and two floors have gone red this
session purely from concurrent load.

## What each outcome means — all of them are answers

| result | meaning |
|---|---|
| a phase's ratio **≈2×** per doubling | linear in that phase |
| a phase's ratio **>2×** | **superlinear, and that phase is the target.** The first grounded scaling finding in the arc |
| **every** phase ≈2× | nothing is superlinear. The original `+55 %` was `vis` contamination plus single samples, and eleven mechanisms died chasing an artifact |
| n=4000 does not complete | a finding about the bound, not a failed run. Report it with the counters |

⚠ **No prediction.** Eleven mechanisms have died here and several fitted better than any guess I could
offer now.

## `setup` is measured, not targeted

`setup` is **33 %** of the run and is cold boot — the builder has ruled it out of scope for this session
until it is the last place left. **Measuring its slope is free and informative; measuring is not fixing.**
If `setup` is the superlinear one, that is worth knowing even while it stays untouched.

## OUT OF SCOPE — REJECTED

- **Any code change.** After a day of instruments corrupting their own measurements — a quadratic
  `seen-ids`, a store-call gate blind to allocation, a phase-sum row aimed at rounding error — this stone
  adds no counters and no state. Runs only.
- **Fixing whatever the slopes implicate.** That is the next stone and it needs this first.
- **`setup`**, per the ruling above.
- **n=8000.** If n=4000 is clean, the next point is a later decision, not a scope creep.

## Files

**None.** No file is modified. The deliverable is the SCORE.
