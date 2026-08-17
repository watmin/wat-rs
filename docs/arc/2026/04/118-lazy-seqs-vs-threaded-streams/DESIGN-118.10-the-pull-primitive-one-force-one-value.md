# DESIGN — 118.10 · the pull primitive: one force, one value, a named exhaustion

**Builder, 2026-08-17:** *"a user's func must never be called 3 times… we don't know if the func has
side effects… that's a massive failure outright."* And: *"we just need to make an ergonomic `next`
that behaves correctly… boxed in an enum that either has a value or a named exhaustion… that thing
must support filter, map, etc."*

Everything below is measured on `f9db1929`, including a throwaway variant build. **Source restored;
nothing here was committed to `src/`.**

## The root, and it is one thing

```
the walk protocol is  empty? → first → rest        three separate forces of one cell
  ⇒ USER CODE RUNS 3× PER ELEMENT                  MEASURED: 15 calls for 5 elements
  ⇒ patch: cache the forced cell (OnceLock)        MEASURED: restores exactly 1×, 5 for 5
  ⇒ but the cache links every cell to its tail
  ⇒ the head reaches the whole realized chain
  ⇒ +297 B/element retained, forever               MEASURED: linear, and it IS the whole overhead
```

**Every stream defect found today is a leaf of that root:**

| defect | how it descends |
|---|---|
| user fn called 3× (held closed only by the cache) | the protocol itself |
| **+297 B/element** retention | the cache that patches the protocol |
| `dorun` builds a full Vector and bins it | no walk-without-build primitive exists |
| `length` on a Stream type-checks then **raises** | no faced pull |
| `first` on an exhausted Stream returns bare `nil` | no faced pull |

## The measurement that quantifies the trade

A throwaway build with the memo bypassed, against the eager control:

| | `f` per element | 250k | 1M | Δ/element |
|---|---|---|---|---|
| memo ON (today) | **1×** ✓ | 184,016 KB | 622,928 KB | 585 B |
| memo OFF | **3×** ⛔ | 109,768 KB | 325,972 KB | 288 B |
| `mapv` — eager, no stream | 1× | 109,836 KB | 326,188 KB | 288 B |

★ **Memo-off and eager are IDENTICAL** — within 200 KB on 326 MB. So the 288 B is the output
`Vector` the caller asked for, and **297 B/element was pure stream retention.** The memo is the
entire lazy overhead.

**Neither column is shippable.** Memo-on is silently wrong for any effectful `f`; memo-off OOMs.
That is why this never converged while it was argued on taste — there is no acceptable point on
that axis.

## The primitive

One call, one force, one value, a named end:

```wat
(:wat::stream::next s) -> :wat::stream::NextOutcome<T>
   :Item      [value <- T  rest <- :wat::stream::Stream<T>]
   :Exhausted []
```

- **one force per cell** → nothing to dedupe → **no cache** → no cell→tail link → cells free behind
  the cursor
- **`f` runs exactly once**, structurally, not because a cache is holding it to one
- the end is a **named arm**, not a bare `nil` — which closes `first`'s unfaced hole and `length`'s
  check-then-raise in the same motion
- it is the house's own idiom: `ReadOutcome` · `RecvOutcome` · `AcceptOutcome` · `ReadFrameOutcome`
  — ten of them already. wat answers *"did I get something?"* with a matchable enum everywhere else.
- **the thunk stays.** No channel, no spawn, no crossbeam — arc 118 killed thread-per-stage on
  2026-06-27 and this does not walk back toward it.

### And it is Rust's shape, which is the substrate's shape

`Iterator::next(&mut self) -> Option<Item>` — one call, advance-and-read fused, no protocol to
mis-sequence. Ours returns the tail instead of mutating, because wat is persistent. Same primitive.

## What the verbs become

```wat
(:wat::core::defn :wat::core::keep<T,U>
  [f <- :wat::core::Fn(T)->wat::core::Option<U>  coll <- :wat::core::Seqable<T>]
  -> :wat::stream::Stream<U>
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next (:wat::core::seq coll))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty))
      ((:wat::stream::NextOutcome::Item v rest)
        (:wat::core::match (f v)
          ((:wat::core::Some u) (:wat::stream::cons u (:wat::core::keep f rest)))
          (:wat::core::None                          (:wat::core::keep f rest)))))))
```

**One `match`, one force, `f` applied once, `rest` bound by the arm.** The three-call sequence has no
form — you cannot write `empty?` then `first` then `rest`, because there is one call and it hands you
both halves.

## The corrections that fall out for free

```wat
;; TODAY — the verb that exists to avoid the allocation pays it in full
(:wat::core::defn :wat::core::dorun<T> [coll <- Stream<T>] -> nil
  (:wat::core::do (:wat::core::into [] coll) nil))          ;; 295 MB at 1M

;; AFTER — walks, discards, builds nothing
(:wat::core::defn :wat::core::dorun<T> [coll <- :wat::core::Seqable<T>] -> :wat::core::nil
  …pull to Exhausted, retaining nothing…)                    ;; ~baseline
```

`doall` is already correct (Clojure's `doall` *does* realize) and keeps its meaning. `run!` stays
eager-container-only by deliberate design.

## Sequencing — and `Seqable` is downstream, not parallel

1. **`NextOutcome` + `next`, and delete the memo.** The acceptance test is fixed and already written:
   `f` runs exactly N times for N elements, **and** the per-element delta reaches the eager baseline
   (288 B, not 585). Both, or it has not landed.
2. **Migrate the walkers** — 7 `-stream` twins + the drain verbs — from three-call to `match`. They
   are the only sites that can observe the change.
3. **Fix `dorun`; face `length` and `first`.** Consequences, not separate stones.
4. **THEN `Seqable`.** Its `seq` returns a `Stream`, and a `Stream` is pulled with `next`. Minting
   the interface first would freeze the broken protocol into a user-facing type.

## ⚠ Not measured — do not let these pass as settled

- **That the fused pull actually reaches O(1).** Predicted, not run. My prediction that *removing the
  memo* would reach O(1) was **wrong** — it reached eager parity. Predictions in this area have a
  poor record today; measure it.
- **Chained-stage cost.** `(filter p (map f xs))` with no memo — does each stage force its upstream
  once, or does the shape re-force? This is what the memo was originally protecting, and the fused
  pull's claim to fix it is structural reasoning, not a run.
- **`dorun`'s corrected cost.** Predicted ~baseline; unmeasured.
- **Stack depth.** The recursive `keep` above is TCO-shaped, but 1M-deep forcing is exactly task
  #58's silent-SIGSEGV territory.
