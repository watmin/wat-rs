# ⛔ NOTE — `Seqable`'s three recorded blockers are STALE. It is spellable today.

**Measured 2026-08-17, by probe, at the builder's question: *"is Seqable a thing in wat right now?
did we plan this and never make it?"***

Answers: **no, it does not exist** (11 occurrences in code, every one a comment; zero in `wat/`,
`wat-tests/`, `wat-scripts/`). **Yes, it was planned and never built** — named across 18 documents
from 2026-06-21 (arc 255) to 2026-08-16 (the seam).

And the reason it was never built is **a comment that was wrong when it was written.**

## The instrument

`wat-scripts/scratch-pad/probe-seqable-is-spellable-today.wat` — type-checks clean, runs, prints
**`"3,4"`**. Two different builtins satisfying one surface, and a function whose parameter type *is*
the surface:

```wat
(:wat::core::defsurface :sq::Seqable :nature :wat::core::Struct
  :features [(as-vec [self <- :sq::Seqable] -> :wat::core::Vector<wat::core::i64>)])

(:wat::core::extend-type :wat::core::Vector           :sq::Seqable …)
(:wat::core::extend-type :wat::core::PersistentVector :sq::Seqable …)

(:wat::core::defn :sq::count-of [s <- :sq::Seqable] -> :wat::core::i64
  (:wat::core::length (:sq::Seqable/as-vec s)))
```

## What it refutes, line by line

`src/collection/infer.rs:638` calls `Seqable` *"the type wat cannot currently spell"* and lists
three blockers, *"none is a small fix"*:

| # | the blocker, as written | measured |
|---|---|---|
| 1 | *"no `defsurface` `:nature` admits a builtin container (only `:wat::core::Record` and `:wat::kernel::Peer'` exist)"* | **REFUTED** — `:nature :wat::core::Struct`, two builtins extended |
| 2 | *"no builtin (`Vector`/`PersistentVector`/`List`) satisfies any surface today — builtins sit outside it"* | **REFUTED**, twice over |
| 3 | *"wat has no ad-hoc unions, deliberately (R7) — a bound over four concrete builtins is structurally a union"* | **DISSOLVED** — it is not a union. It is N `extend-type`s of ONE surface. Exactly Clojure's `ISeq`. |

## ★ The comment postdates its own refutation by a month

```
2026-06-28   SCORE-293.4d  GREEN — "a FOREIGN built-in taught to be a Shape it never declared"
2026-07-31   infer.rs:638  "no builtin satisfies any surface today"
```

`tests/types/probe_arc293_acceptance_demo.wat:33` does literally this, and has been green in the
floor for seven weeks:

```wat
(:wat::core::extend-type :wat::core::Vector :geo::Shape
  (area [self] -> :wat::core::f64 (:wat::core::i64::to-f64 (:wat::core::length self))))
```

Substitute `Shape → Seqable` and `area → seq` and that is the chain doc's D design verbatim. **The
mechanism `Seqable` was waiting for shipped a month before the note declaring it impossible.**

## What arc 118 actually got, and what the absence cost

```
118.2a  lazy map/take/drop ........ BUILT (infer.rs: "the now-LAZY map/take/drop")
118.2Z  the transformer family .... BUILT — as SEVEN `-stream` TWINS in wat/seq.wat
118.1   seq foundation ............ NOT BUILT
        Seqable ................... NEVER BUILT
```

Arc 118 has **no INSCRIPTION and not one SCORE doc**; `infer.rs` records where the work actually
landed — *"arc 118.2a (**was arc 278 stone 0d**, eager)."* Its pieces were built inside other arcs.

The seven twins — `dedupe-stream` · `distinct-stream` · `interpose-stream` · `keep-stream` ·
`keep-indexed-stream` · `map-indexed-stream` · `reduce-stream` — are the receipt. Arc 109's note says
they exist *"only because this type doesn't."* **We built the workaround, twice, in other arcs'
names, and never built the thing that made the workaround unnecessary.**

## ⚠ WHAT THE PROBE DOES NOT PROVE — the real remaining work

Stated so the next self does not read this note as a green light:

1. **Only `Vector` + `PersistentVector` are extended.** `List` and `Stream` are **untested**. The
   four-head set in `extract_lazyable_elem` needs all four.
2. **The surface is NOT parametric.** The probe hardcodes `Vector<i64>`; real `Seqable<T>` needs a
   type parameter. `BRIEF-293.4e-pre-ii-generic-surface-methods.md` claims generic surface methods
   with type params ship — **that claim is unverified by me.**
3. **Per-element dispatch cost is UNMEASURED.** `join`/`map`/`filter` walk every element; a surface
   dispatch per element is a real perf question and this probe says nothing about it.

Those three are the stone. They are not the three blockers on record, and they are a different order
of difficulty.

## The lesson, which is the arc's own

A blocker note is a **claim about the disk with a date on it**, and it rots exactly like any other
claim. This one was consulted repeatedly for two months — it is quoted in the seam, in the chain doc,
in three 279 stones, in my own briefs today — and **nobody re-ran it.** I quoted it three times in
one session before the builder asked "is it a thing right now?" and the probe took four minutes.

`[[feedback_a_design_sentence_is_not_the_disk]]` · `[[feedback_impose_the_check_and_read_the_screams]]`
· and the seam's own closing line: **the arc's own directory holds the answer more often than your
reasoning does.**

⛔ **`infer.rs:638`'s comment should be corrected in place** — it is actively misleading and is the
single most expensive stale sentence found this session. Not done here: it is a `src/` edit and needs
a floor run. Named so it is not lost.
