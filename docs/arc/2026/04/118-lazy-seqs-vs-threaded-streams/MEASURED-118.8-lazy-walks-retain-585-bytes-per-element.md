# ⛔ MEASURED — a lazy walk retains **585 bytes per element**, linearly. This is the DoS.

**Builder, 2026-08-17: *"i really, really really dislike the idea that we keep items around in memory
beyond their read… if i'm in a consumer loop who grabs a million items, i need to keep a million in
memory? that's not good."*** Measured on `09d81830`. **He is right, and the number is worse than the
intuition.**

## The measurement

Same program at four sizes — build a `range`, lazily `map` it, drain with `into`:

```wat
(:wat::core::let
  [src (:wat::core::range 0 N)
   out (:wat::core::length (:wat::core::into (:wat::core::Vector :wat::core::i64)
          (:wat::core::map (:wat::core::fn [x <- :i64] -> :i64 x) src)))] …)
```

| N | maxRSS | Δ per element |
|---|---|---|
| 100,000 | 96,304 KB | — |
| 250,000 | 184,016 KB | **585 B** |
| 500,000 | 330,232 KB | **585 B** |
| 1,000,000 | 622,928 KB | **585 B** |

**Perfectly linear, constant per element.** And the control:

| | maxRSS | wall |
|---|---|---|
| 1M `range` + `length` (no stream) | **90,984 KB** | 0.23s |
| 1M `range` → lazy `map` → `into` | **622,928 KB** | 3.37s |

**6.8× the memory and 14.7× the wall clock** for a walk whose extra data is two 8 MB vectors.

## What it means

The payload is **8 bytes** per element. We retain **585** — **73× the data.**

- 1M items → **~585 MB**
- 10M items → **~5.8 GB** — an OOM, not a slowdown

**A lazy pipeline in wat is O(n) in memory, not O(1).** That is the entire point of laziness, and we
do not have it.

## ★ Why this settles a design question that was being argued on taste

The `Seqable` thread had been circling *"should `empty?`/`first`/`rest` exist, and is memoization
Clojure-like or Ruby-like."* **This measurement removes the argument.** A cached/persistent lazy-seq
model — Clojure's — retains by construction, and wat's `let` scoping makes it unavoidable: **binding
a stream to a name holds its head for the whole body**, so every realized cell stays alive until the
binding leaves scope. In wat you essentially always hold the head, because you always name the thing.

Clojure survives this because *"don't hold the head"* is a discipline its community learned the hard
way, with `doseq`/`dorun` as the escape hatches. **We would be importing that hazard into a strongly
typed substrate whose stated direction is a bytecode compiler and "amazing perf."**

## ⚠ WHAT IS **NOT** MEASURED — do not let the next self skip this

1. **The mechanism behind 585 B is NOT isolated.** Candidates: the memoized forced value, the
   per-cell closure environment, `Arc` + `rpds` node overhead, the interpreted `fn` frame. **I have
   not decomposed it**, and `[[feedback_measure_the_decomposition_never_read_it]]` says reading the
   code to attribute it would be a guess in a measurement's clothes. **Decompose before fixing.**
2. **No proposed API shape has been shown to fix it.** `next → Item[v, rest]` *should* let each cell
   drop, but that is a prediction, not a result. The fix must be measured with this same instrument.
3. **`into` is NATIVE and still retains** — so this is not "the interpreter is slow at walking." The
   retention survives a native drain, which points at the cell chain rather than the walker.
4. **Wall-clock is 14.7×**, and that is a separate finding from memory. Not decomposed either.

## The instrument

`/usr/bin/time -f 'maxRSS=%M KB'` over the four-point scaling series above. Reproducible in one
command; **re-run it against any proposed fix and require the per-element delta to go to ~0.** That
is the acceptance test for whatever replaces the current model — not a green floor, which this
already has.

## Where this lands in the arc

`Seqable` was scoped as *"the missing type"* — a naming and dispatch problem. This is a **second,
independent defect in the same tier**, and arguably the more urgent one: today a user who writes the
idiomatic lazy pipeline over a large input gets an OOM, and nothing in the surface warns them.

The two interact — whatever `Seqable`'s pull primitive turns out to be will decide whether cells can
drop — so they should be designed together. But `Seqable` shipping without this fixed would put a
nameable, extensible, **still-quadratic-in-memory** interface in front of users.
