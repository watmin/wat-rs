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

- **O1 — measured by 255.50: holds.** Every `poll` vector (177) is a `Peer` vector; the one stdlib owner vector is
  `Spawned` (`wat/bracket.wat:603`); a bare literal takes `Spawned` from the parameter. One site needs an
  annotation (`probe_select_flood_no_deadlock.wat:34`, a constructor-built `Process` vector). No `mapv` pool reaches
  `select`. So `select`/`poll` need no bound.
- The tuple "each element" binder: assessed below.
- Measured by 255.50: no corpus comparison reaches a function-holding struct, but `=` on one checks and raises at run time; records cannot hold functions. Nine newtypes, all in tests, none compared. **Ruled 2026-09-26 (Q1): only pure data is `Equatable`/`Orderable`**: one edge on the `Record` root (holon records ride it). A struct may not be compared or sorted. **D1 withdrawn**: `derive` is a hierarchy between names (the parent may be an undeclared tag, `src/types.rs:4642-4670`), Clojure's `isa?`; `extend-type` is surface (protocol) membership with the binder and its checks. Different acts, both kept. Conditional membership stays on `extend-type`. Unmeasured tightening: refuse a `derive` whose parent is a surface.

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
  `(wat.core/extend-type :- [[Ts :< wat.core/Orderable] :..] (wat.core/Tuple :- [Ts :..]) wat.core/Orderable)`.
  **Ruled 2026-09-26: the marker is the keyword `:..`** (Typed Clojure ≥ 1.3.0 removed the `...` symbol for it).
  English: *for any list of types Ts, each an Orderable, the tuple with those slot types is Orderable.* `:..` spreads
  a list of (possibly different) types, one position each; `& rest` gathers many values of one type into one name.
  The `&` form failed Obvious and Honest; the `...` symbol failed Simple and Honest (a legal name made syntax).
- **A variadic tuple parameter** (`(Tuple :- [i64 & Rest])`) is **cut**: a second rest capability with no consumer.
- **Open: a bound in a head binder binds every clause.** `sort-by`'s comparator clause never calls `<`, yet a head
  `[K :< Orderable]` would demand it.

## Ruled 2026-09-27 — the cutover's type namespace (for 251.8d)

- **F1:** a type the language itself provides (no wat declaration: the scalars, `Vector`, `HashMap`, `HashSet`,
  `PersistentVector`/`PersistentMap`, `List`, `Tuple`, `Value`, `Instant`/`Duration`, the syntax tree) is spelled
  `wat.type/…` in every type position; `wat.core/<that type>` in a type position is illegal. Names unchanged (the
  2026-09-20 ruling), except:
- **N-AST:** `:wat::WatAST` becomes **`wat.type/AST`**.
- **Declared types keep their declaring namespace:** `wat.core/Option` (`wat.core/Option.Some {:value …}`,
  `wat.core/Option.None {}`), `wat.core/Result` (`.Ok {:value …}`, `.Err {:err …}`), and every `defenum`/`defrecord`/
  `defstruct` name. The builder: *"i don't think Option belongs in type.. its a utility that core needs"*.
  `HolonAST` is a subsystem type: `wat.holon/HolonAST`.
- **P1:** `(wat.type/Vector :- [wat.type/i64])` is a **type**, "a vector of i64s", never a call. Any walk that meets a
  `(Head :- [...])` form treats it as a type (the macro-body purity gate at `src/macros/eval.rs:458` misread it as a call,
  and passed the old spelling only because `:wat::core::Vector` is also the constructor's name).
- **Enum purity (open, not blocking):** a generic enum's `wat.enum/Pure` means "holds no resource in its own fields";
  each instance is as pure as its type arguments (255.28). Measured: a `Pure` enum with a concrete struct field is
  refused; `(u/Box :- [u/Conn])` of a `Pure` `u/Box` is impure and refused inside a record. Proposed (D): rename the
  marker so the word says exactly that. Unruled.

## Ruled 2026-09-27 — `wat.type/` is closed (supersedes the F1 list above where they differ)

`wat.type/` holds exactly the hard primitives, the types the language itself provides:
**`i64`, `f64`, `u8`, `bigint`, `rational`, `char`, `String`, `bool`, `keyword`, `nil`, `Value`, `Never`, `Fn`,
`Record`, `Struct`, `Vector`, `HashMap`, `HashSet`, `List`, `Tuple`, `PersistentVector`, `PersistentMap`, `Bytes`,
`AST`** (`AST` was `:wat::WatAST`). The builder: *"the core types, the real hard primitives are in wat.type/ — other
typed things belong in their own homes"*; *"we put types in core because we didn't think this through"*.

Own homes: `wat.time/Instant`, `wat.time/Duration`, `wat.uuid/UUID`, `wat.holon/HolonAST`, `wat.holon/Record`.
Declared utilities stay where declared: `wat.core/Option`, `wat.core/Result`, `wat.core/Span`, `wat.core/Pos`,
`wat.core/Error`, `wat.core/EvalError`, the read outcomes, `wat.core/Orderable`, `wat.core/Equatable`.
**The new spelling becomes the canonical key** (C1). `(wat.type/Vector :- [T])` is the type in a type position and the
empty vector elsewhere; `(wat.core/Vector …)` is an unknown function after the cutover. The cutover runs in seven
green stones (`WEIGH-STONE-255.65`), with a temporary door (T-door) that stones 4 and 5 delete.
