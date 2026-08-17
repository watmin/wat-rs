# ASSESSMENT — 118.6 · the collection surface's split brain, counted, and why `Seqable` closes it

**Builder, 2026-08-17: *"we want obviousness and good ux… it may replace it… there must not be N
ways to do a thing… seqable is a thing we should probably have proper."*** Everything below is
measured on `b46b5f1f`.

## ★ N = 3, and the twins are a fourth artifact

Three different ways exist **today** to write "a verb that accepts any container":

| # | shape | verbs | count |
|---|---|---|---|
| 1 | **native Rust** — one body, dispatches on `StreamContainer` + `extract_lazyable_elem` | `map` `filter` `take` `drop` `foldl` `foldr` | 6 |
| 2 | **wat `defclause` + a `-stream` twin** — arms share a body via the twin | `keep` `dedupe` `distinct` `keep-indexed` `map-indexed` `reduce` `interpose` | 7 |
| 3 | **wat `defclause`, no twin** — arms duplicate | `remove` `take-while` `drop-while` `reductions` `take-nth` | 5 |

Plus **7 `-stream` twins** on disk (`dedupe-` `distinct-` `interpose-` `keep-` `keep-indexed-`
`map-indexed-` `reduce-stream`) that exist for no reason a user could name.

**Which shape a verb has was chosen by nobody.** 278's own words: *"`map` landed in Rust and is
linear, `filter` landed in wat and is quadratic, and **nobody chose that** — it is an artifact of
which side of the language each verb happened to be written on."*

## The four questions, on the surface as it stands today

- **Obvious? NO.** Nothing in a verb's name or signature says whether it accepts a `Stream`. `map`
  does. `keep` does, via a twin. `foldl` does not. `reduce` has a twin but `reductions` does not.
  A user cannot predict the answer and there is no rule to learn.
- **Simple? NO.** Three shapes for one concept, plus a twin family that is pure workaround.
- **Honest? NO.** `keep` and `keep-stream` are two public spellings of one idea with no stated rule
  for choosing. `mappable()`'s doc comment still claims to gate `map`/`filter` — both of which
  bypass it entirely.
- **Good UX? NO.** Clojure has exactly one `filter`. We have `filter` (native), `filterv` (eager
  convenience — fine, Clojure has it too), and a family of `-stream` twins that Clojure has no
  analogue for.

## ⛔ THE RULING THAT CREATED THIS RESTS ON A PREMISE THAT WAS FALSE WHEN WRITTEN

`278/DESIGN-STONE-seq-traversal-one-door.md` four-questioned *native* against *a real `Seqable`
surface* and chose native. The deciding cell:

> | Simple? | native: YES | `Seqable` surface: **NO** — needs a surface nature admitting builtins
> (none), builtin surface-satisfaction (nothing does), and reconciliation with no-ad-hoc-unions (R7) |

**All three are false, and two were false on the day that stone was written (2026-07-31):**

| the blocker | measured 2026-08-17 |
|---|---|
| "a surface nature admitting builtins (none)" | `:nature :wat::core::Struct` works — `probe-seqable-is-spellable-today.wat` |
| "builtin surface-satisfaction (nothing does)" | `extend-type :wat::core::Vector :geo::Shape` has been **green in the floor since 2026-06-28** (`SCORE-293.4d`) — a *month before* the stone |
| "no-ad-hoc-unions (R7)" | not a union. **N `extend-type`s of one surface** — exactly Clojure's `ISeq` |

Same false sentence as `infer.rs:638`, same date, **two decisions built on it**: the native route,
and "Seqable is blocked." That is the split brain's origin, and it is one stale claim.

★ And 278 **named its own ceiling** honestly:

> *"native reaches the **check** rung, not `no-form`. Afterwards 'sequence verbs that take any
> seqable live in Rust' is a **convention** — nothing stops a new wat-level stage with per-container
> arms and a `rest`-walk tomorrow, and it would be quadratic and green."*

## ★★ `Seqable` REPLACES the native route as the ANSWER — and keeps its engine

The two are not competitors. Measured: **`seqable->stream` has ZERO callers outside `wat/seq.wat`**
(42 uses inside, none outside — only a probe's comment mentions it). It is already an internal
normalizer, not a user verb.

So:

```wat
(:wat::core::defsurface :wat::core::Seqable<T> :nature :wat::core::Struct
  :features [(seq [self <- :wat::core::Seqable<T>] -> :wat::stream::Stream<T>)])

(:wat::core::extend-type :wat::core::Vector            :wat::core::Seqable<T>
  (seq [self] -> :wat::stream::Stream<T> (:wat::core::seqable->stream self)))
;; …List, PersistentVector, Stream likewise
```

**`seqable->stream` becomes the builtins' IMPLEMENTATION of `seq` and stops being a verb.** 278's
linear, position-stepping, registry-backed native is *kept* — it is exactly the right engine — and
gains a nameable interface over it. Nothing 278 built is discarded; what changes is that the
concept acquires a name a `.wat` signature can spell.

Then a wat verb reads:

```wat
(:wat::core::defn :wat::core::remove<T> [pred <- … xs <- :wat::core::Seqable<T>] -> …)
```

**One body. No arms. No twin.** Which is Clojure's shape, reached the way Clojure reaches it —
through `seq`.

### The four questions, on the resolved surface

- **Obvious? YES** — one rule: *a sequence verb takes a `Seqable`.* No per-verb exceptions to learn.
- **Simple? YES** — it **deletes**: 3 shapes → 1, and 7 twins → 0.
- **Honest? YES** — and it closes 278's own admitted gap. A wat verb typed `Seqable<T>` **cannot**
  write per-container arms with a `rest`-walk; the shape has no form. That is the **no-form** rung
  278 explicitly could not reach.
- **Good UX? YES** — and a user can extend `Seqable` to *their own* container, which no version of
  the native route allows.

## The honest cost, stated

**Perf.** A wat-level `remove` over `Seqable<T>` is interpreted where a native one is not. Today
that is real. Builder, 2026-08-17: *"wat will be byte code compiled… the surface will be our
expression language for optimized code it produces… interpretted wat has a death sentence."* One
polymorphic verb is strictly easier to compile than three shapes plus seven twins. **Do not hand-roll
around a cost the compiler deletes** — but do not pretend it is zero today either.

**This reopens a ruled stone.** 278's Strike 2b (*"the remaining five go native"*) should not be
struck as written; those five should take `Seqable<T>`. That is the builder's call, not mine — but
its stated reason is measurably false and it should not be executed on that reason.

## Sequence

1. **Mint `Seqable<T>`** in `wat/seq.wat` (pos 67) + extend the four builtins onto
   `seqable->stream`. Nothing else changes; nothing breaks.
2. **Move the twin-backed 7 onto it** — `keep` `dedupe` `distinct` `keep-indexed` `map-indexed`
   `reduce` `interpose` — deleting arms **and** the 7 twins.
3. **Move the armless 5** — `remove` `take-while` `drop-while` `reductions` `take-nth`. This is
   278's Strike 2b, redirected.
4. **The lint 278 tracked but never built:** a wat-level sequence verb may not take per-container
   arms. Convention → wall. **This is the rung 278 said it could not reach**, and `Seqable` is what
   makes it reachable.

## What this assessment does NOT claim

That `Seqable` is free — `118.3-B`'s `Var`-gate still excludes concrete instantiations, a surface
with >1 type param is untested, and the interpreted-vs-native perf gap is real until the compiler
lands. And it does not claim 278 was careless: it was rigorous, four-questioned, and **wrong only
because it trusted a sentence that was already false.** That is the day's lesson, not a criticism.
