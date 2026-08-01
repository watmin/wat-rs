# DESIGN-STONE — `into (PersistentVector, Vector)`: the missing clause nine harnesses hand-rolled around

## Why — the consumer surfaced it (`ALIVS ARGVIT`)

The builder noticed the Clara grid's **wall clock favours the JVM** even where our `:ratio` says we
win 2.3×. Grounded: at `fanout [40000]` the whole program is **5.33 s**, of which

| segment | time | share |
|---|---|---|
| startup (freeze a trivial world) | 0.17 s | 3% |
| seeding (4,000 facts — `keys = items/fanout² = 100`) | ~0.013 s | 0.2% |
| **the fire — the ONLY thing the grid times** | **0.046 s** | **0.9%** |
| derive + print | **~5.1 s** | **96%** |

`:native-ns` is honest for comparing ENGINES, and the engine is fine. But 96% of the program is
`derived-vector`, and half of that is a workaround for a missing substrate clause.

## The defect, and it is named in the harness's own comment

Every grid axis ends with (`fanout.wat:70-77`):

```clojure
;; vec->pvec v — materialize a Vector<i64> into a PersistentVector<i64> (the honest conj-fold
;; bridge every grid axis uses; `into` has no (PV<T>,Vector<T>) clause).
(:wat::core::defn :fan::vec->pvec [v <- :wat::core::Vector<wat::core::i64>] -> …
  (:wat::core::foldl (fn [acc x] (:wat::core::PersistentVector/conj acc x)) (:wat::core::PersistentVector) v))
```

That is **N interpreted closure invocations** to move a Vector into a PersistentVector — 40,000 of
them on this cell, on top of the 40,000 the preceding `map` already costs. Nine axes carry a copy.

**PROVEN RED, live this session** (the checker enumerates its own clauses — `RVINA ERVDIT`):

```
(:wat::core::into (:wat::core::PersistentVector 1 2) (:wat::core::Vector :wat::core::i64 3 4))
⇒ NoMatchingClauseAtCallSite: no clause of `:wat::core::into` matches arity 2 with types
  [PersistentVector<i64>, Vector<i64>]; clauses attempted:
    (Vector<T>, Vector<T>) · (Vector<T>, Stream<T>) · (PersistentVector<T>, Stream<T>)
```

## The mechanism ALREADY EXISTS below the checker

`insert-all'` does this exact bulk extend natively — `kernel.rs:3628` calls
`crate::collection::eval::vector_concat_inner(facts_val, &new_facts_vec)` where `facts_val` **is a
PersistentVector**. So the runtime handles a PV receiver today; there is simply no *checked* wat
surface for it. Confirmed by probe:

```
(:wat::core::Vector/concat <PV> <Vector>)
⇒ TypeMismatch: parameter #1 expects :wat::core::Vector<?0>; got :wat::core::PersistentVector<i64>
```

Checker scheme, `check.rs:18633`: `Vector/concat :: ∀T. Vec<T> × Vec<T> -> Vec<T>`.

## The one contract decision — mint the per-Type sibling, do NOT widen `Vector/concat`

Widening `Vector/concat` to accept a PV receiver makes its NAME lie. The substrate's established
shape is a per-Type impl behind a polymorphic surface (`Vector/length`, `PersistentVector/conj`).
So: mint **`:wat::core::PersistentVector/concat`**, and let `into` — the user-facing materializer,
clojure's own idiom — remain the only verb anyone types. Two clauses, so the name is not asymmetric:

```
:wat::core::PersistentVector/concat :: ∀T. PV<T> × Vector<T> -> PV<T>
                                     :: ∀T. PV<T> × PV<T>     -> PV<T>
```

**Four questions.** *Obvious?* YES — it is `Vector/concat` for the persistent twin. *Simple?* YES —
one native fn already written, reached through one new scheme. *Honest?* YES — the name matches the
receiver, and no existing signature is loosened to make a call fit. *Good UX?* YES — callers keep
writing `into`; nobody types the impl name, exactly as nobody types `Vector/length`.

## Out of scope = REJECTED (affirmative, not deferred)

- **Nativising the `map`+`enc` pass.** That closure is user-supplied per axis; it is the other half
  of the derive cost and it is a DIFFERENT stone (the interpreter's per-element cost — the largest
  surface on the board, tracked separately).
- **Changing what `:native-ns` measures.** The grid times the fire; this stone does not touch it and
  must not move any `:ratio`. That is a scorecard row, not a hope.
- **`insert-all` adoption in the axes.** Measured 1.6× on seeding, but seeding is 0.2% of this cell.
  Real, tiny, and a separate cleanup.
