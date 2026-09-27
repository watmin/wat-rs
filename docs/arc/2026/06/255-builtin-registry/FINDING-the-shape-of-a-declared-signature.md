# FINDING — the shape of a declared signature (builder-aligned, 2026-09-26)

The syntax the builder aligned on while weighing 255.46–.49. Written in the **target spelling** (dotted symbols,
`:-` for "has type", `:->` for function types only). The corpus still uses the colon spelling and `<-`/`->`. The
ongoing clojure-compliance migration retires those; **no new arrow is introduced**.

## The pieces

```clojure
[t :- T]           ; value binder: t has type T
[T :< X]           ; bounded type binder: T is a subtype of X (Typed Clojure's `:<`)
[K V :-> U]        ; function type: (K V) returns U
```

`[T :< X]` is the same three-part shape as `[t :- T]`. "Subtype of" gets its own keyword because it is a different
relation from "has type". Typed Clojure writes `(t/All [[x :< Upper]] …)`.

## The three positions a bound goes

```clojure
;; a generic defn
(wat.core/defn u/max :- [[T :< wat.core/Orderable]]
  [a :- T  b :- T]
  :- T
  (wat.core/if (wat.core/< a b) b a))

;; conditional membership (ruling C): a Vector of T is Orderable when T is
(wat.core/extend-type :- [[T :< wat.core/Orderable]] (wat.core/Vector :- [T]) wat.core/Orderable)

;; a clause of a defintrinsic (ruling A)
```

## `defintrinsic` is `defclause`'s clause shape, bodiless, with the binder at the head

```clojure
(wat.core/typealias wat.spawn/Owners :- [S R] (wat.core/Vector :- [(wat.spawn/Spawned :- [S R])]))
(wat.core/typealias wat.kernel/Peers :- [I O] (wat.core/Vector :- [(wat.kernel/Peer :- [I O])]))

(wat.core/defintrinsic wat.kernel/select :- [S R I O]
  ([peers :- (wat.spawn/Owners :- [S R])] :- (wat.spawn/OwnerEvent :- [R]))
  ([peers :- (wat.kernel/Peers :- [I O])]  :- (wat.kernel/PeerEvent :- [I O])))
```

Read: *`select`, over S R I O: given `peers` as owners of S/R, it returns an owner event of R; given `peers` as
peers of I/O, a peer event of I/O.* One parameter, two clauses, chosen by the argument's type.

- The type binder is declared **once, at the head**, as `defn` does. A clause instantiates only the letters it uses,
  fresh per call. Every letter is declared (today's `defclause` collects clause letters implicitly,
  `src/check.rs:5650`; the head binder is how C-c closes for it too).
- A per-clause prefix `(:- [S R] …)` was drafted and rejected: it did not read.

## Open, to be measured (not ruled)

- **O1:** with the `Owners`/`Peers` aliases, `select`/`poll` need no bound. This holds if owner vectors are typed as
  owners where they are built. Measured so far: every `poll` caller passes a `Peer` vector; `wat/bracket.wat:593`
  types its owner vector as `Spawned`. Three test sites pass a `Process` vector or a bare `[a b]`
  (`probe_select_flood_no_deadlock.wat:34`, `peer_select_prime_process.wat:26`,
  `probe_arc214_stone46b_select_prime.wat:32`). Unmeasured: how real pools are built (e.g. `mapv` over spawns),
  and whether a bare vector literal takes its element type from the expected parameter.
- The tuple "each element" binder (Typed Clojure uses a dotted variable, `b ...`; wat has `& rest` for values).
- Records and enums that hold a function reaching `=`/`<`; how a newtype gets its inner type's classes.

## Ruled 2026-09-26 — `sort`/`sort-by` take a `Seqable` and return a seq

The builder: *"needs to be upgraded to not impose vec … they call into a vec if they need a vec on the out."*
Today both are `Vector → Vector` defclauses (`wat/core.wat`). They become the house sequence shape, as
`take-while`/`dedupe`/`distinct` already are (`wat/seq.wat:380-448`):

```clojure
(wat.core/defn wat.core/sort-by :- [T [K :< wat.core/Orderable]]
  [keyfn :- [T :-> K]  coll :- (wat.core/Seqable :- [T])]
  :- (wat.stream/Stream :- [T])
  …)
```

A `Vector` is Seqable (not a seq); a caller needing a vector writes `(into [] …)`. Measured at the ruling: `sort`
has 75 calls in 72 files (5 in the stdlib); `sort-by` 4 in 3 (3 in the stdlib). Not yet measured: which callers
use the result as a vector (index, `conj`, a `Vector` parameter), and whether `reverse`, which keeps its
container, takes a `Stream` (`wat/fix.wat:1287` is `(reverse (sort eds))`).

## Assessed 2026-09-26 (four questions on the page)

- **The tuple class declaration** decomposes to Typed Clojure's dotted form, the bound inside the repeated entry:
  `(wat.core/derive :- [[Ts :< wat.core/Orderable] ...] (wat.core/Tuple :- [Ts ...]) wat.core/Orderable)`. The
  `&` form failed Obvious and Honest: wat's value `& rest` names the collection, not each element. `...` parses
  as a type-parameter name today and must be reserved.
- **A variadic tuple parameter** (`(Tuple :- [i64 & Rest])`) is **cut**: a second rest capability with no consumer.
- **Open: a bound in a head binder binds every clause.** `sort-by`'s comparator clause never calls `<`, yet a head
  `[K :< Orderable]` would demand it.
