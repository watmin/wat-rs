# DESIGN STONE — 118.B2 · collapse the sequence verbs to ONE `Seqable<T>` clause each

**Route B, ruled 2026-08-17.** B1 (`488eacd0`) minted `Seqable<T>`; B1a (`eab12e05`) made a
CONCRETE instantiation satisfiable, which is what unblocks this stone. **This is the payoff: the
stone where the split brain actually closes.**

## What it does

Six lazy verbs — `interpose` · `keep` · `keep-indexed` · `map-indexed` · `dedupe` · `distinct` —
each currently a `defclause` with one arm per container, every arm's body **byte-identical**,
delegating to a `<verb>-stream` TWIN. Each becomes **ONE** definition over `Seqable<T>` whose body
walks with `:wat::stream::next`. The seven twins are deleted. `reduce`'s Stream arm and
`stream->pvec` (the language's single materializer) migrate to `next` in the same motion.

```wat
;; before: 5 arms + a twin.        after: one definition.
(:wat::core::defn :wat::core::keep<T,U>
  [f <- :wat::core::Fn(T)->wat::core::Option<U>
   coll <- :wat::core::Seqable<T>] -> :wat::stream::Stream<U>
  (:wat::stream::lazy
    (:wat::core::match (:wat::stream::next (:wat::core::Seqable/seq coll))
      ((:wat::stream::NextOutcome::Item value rest)
        (:wat::core::match (f value)
          ((:wat::core::Some v) (:wat::stream::cons v (:wat::core::keep f rest)))
          (:wat::core::None (:wat::core::keep f rest))))
      (:wat::stream::NextOutcome::Exhausted (:wat::stream::empty)))))
```

**This exact shape is PROVEN, not proposed** — `wat-scripts/scratch-pad/probe-118B2-one-clause-lazy-producer.wat`
runs green today and covers the lazy-producer body, the state-carrying variant, the recursive
`Stream`-into-`Seqable<T>` call, and laziness over an infinite source.

## ⛔ NOT a wat-fix codemod — ruled, with the reason

R21 says a structural `.wat` rewrite across MANY files is a codemod. **This is one file**
(`wat/seq.wat` — `Stream<` appears nowhere else in `wat/`), and the six bodies are each
*different*, not one mechanical substitution. A codemod would cost more than it saves and could not
express six distinct rewrites. **Call sites do not move**: every verb keeps its name and arity, so
there is no corpus migration at all.

## The four questions

- **Obvious? YES.** One `keep`, the way Clojure has one `filter`. The five identical bodies were the
  missing type rendered as code.
- **Simple? YES.** Six definitions replace six `defclause` families plus seven twins. Strictly less.
- **Honest? YES.** It is the stone that makes the claim true — after B1/B1a the type exists and is
  satisfiable, but nothing uses it. Until B2, `Seqable` is a type with no consumers.
- **Good UX? YES.** ★ **The builder's own criterion:** *"there must not be N ways to do a thing."*
  After B2 a user writing a lazy stage in wat writes **exactly what the stdlib writes.**

## Rooms — exact, `wat/seq.wat` only

| line | what |
|---|---|
| **148** | `stream->pvec` — ★ THE DRAIN. `doall`/`dorun`/`mapv`/`filterv`/every `into` Stream clause funnel through it |
| **226 / 232** | `reduce-stream` twin / `reduce`'s 8-arm defclause (only its Stream arms change) |
| **458 / 466** | `interpose-stream` / `interpose` |
| **500 / 509** | `keep-stream` / `keep` |
| **527 / 540** | `keep-indexed-stream` / `keep-indexed` |
| **558 / 568** | `map-indexed-stream` / `map-indexed` |
| **586 / 600** | `dedupe-stream` / `dedupe` |
| **617 / 627** | `distinct-stream` / `distinct` |

## ⚠ What must NOT change

**`seqable->stream` STAYS.** It is now the implementation behind `Seqable/seq`'s four
`extend-type` bodies (top of `seq.wat`). It leaves the verb bodies in this stone but it is not
deleted — its public-name retirement is B5, and `extract_lazyable_elem`'s deletion is a separate
Rust stone. **A name dies in the stone that removes its last caller; this stone does not remove
that one.**

`reduce`'s eager arms (`foldl` over Vector/List/PersistentVector) stay exactly as they are —
`foldl` is native and not part of this tier.

## Out of scope — affirmative cuts

- **Deleting the memos** — B3. The migrate-then-delete order is measured, not stylistic: with a
  three-call walker still alive, deleting the memo makes user code run **3×** (proven, 15-for-5).
- **Closing `first`/`rest`/`empty?` on Stream** — B4, and it is a dialect ruling the builder owns.
- **`extract_lazyable_elem`, `seqable->stream`'s public name, `stream->vec`** — B5 / a Rust stone.
- **`dorun` building a Vector and binning it** — a leaf of B3/B5, not this stone.
- **`keep-stream`'s deep-recursion-on-long-None-runs** — pre-existing, unmeasured, tracked with
  tasks #58/#86. Do not fix it here; do not make it worse.
