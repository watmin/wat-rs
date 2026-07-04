# BRIEF — 118.2-Z strike A: the lazy transformer family (clojure-core, `filter` defclause shape)

## The work (one paragraph)
Build the missing clojure-core **lazy transformers** as `:wat::core::` defclauses in `wat/seq.wat`, each mirroring
the existing `:wat::core::filter` shape (a polymorphic defclause, one clause per seqable — `Vector`/`List`/
`PersistentVector`/`Stream` + bare-`PersistentVector` — each `(stream/lazy (if (empty? coll) (stream/empty) …
recursion …))`, returning `:wat::stream::Stream<…>`). Forms that carry state across the walk (an index, a
seen-set, a running accumulator, the previous element) get a private `<form>-stream` helper `defn` exactly the way
`reduce` uses `reduce-stream`. Twelve forms: `remove` · `take-while` · `drop-while` · `take-nth` · `interpose` ·
`mapcat` · `map-indexed` · `keep` · `keep-indexed` · `dedupe` · `distinct` · `reductions`. Pure wat over the existing
primitives — **no Rust, no new primitives.** Then un-ignore the RED probe.

## Read in order (the rooms)
1. `wat/seq.wat:47–102` — the `:wat::core::filter` defclause: THE shape every direct-recursion transformer mirrors
   (5 clauses; `stream/lazy` + `first`/`rest`/`empty?` + `stream/cons`/`stream/empty`). Copy this structure.
2. `wat/seq.wat:197–231` — `:wat::core::reduce` + its `reduce-stream` helper: THE pattern for a stateful walk (a
   helper `defn` carrying the extra parameter; the defclause delegates to it). `keep-indexed`/`map-indexed`/
   `dedupe`/`distinct`/`reductions` follow this.
3. `docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/DESIGN-118.2Z-family-completion.md` — the roster + the OUT
   list (`reduced`/memoization/`seq`-nil-pun are OUT; do not add them).
4. `tests/types/probe_arc118_2z_takewhile_lazy.{rs,wat}` — the RED probe (take-while laziness). Un-ignore it when
   `take-while` is a lazy defclause; that GREEN is load-bearing.

## Per-form spec (clojure semantics → wat sketch)
Every form: 5 clauses (Vector/List/PersistentVector/Stream/bare-PV) unless noted, `-> :wat::stream::Stream<…>`.

- **`remove`** `(remove pred coll)` — `filter` with the predicate negated: `(if (pred (first coll)) (recurse on rest) (cons (first coll) (recurse on rest)))`.
- **`take-while`** `(take-while pred coll)` — cons while pred true; at the first false → `(stream/empty)` (stop): `(if (pred (first coll)) (cons (first coll) (recurse rest)) (stream/empty))`.
- **`drop-while`** `(drop-while pred coll)` — skip while pred true, then emit the remainder unchanged: `(if (pred (first coll)) (recurse rest) <the-rest-as-a-stream>)`. (When dropping stops, return the remaining coll realized as a Stream — reuse the `filter`-clause seqable handling.)
- **`take-nth`** `(take-nth n coll)` — every nth element (indices 0, n, 2n…): `(cons (first coll) (take-nth n (drop n coll)))`; empty → empty.
- **`interpose`** `(interpose sep coll)` — `sep` between elements. Helper `interpose-stream` (a boolean/first-flag or a "prepend sep" split): emit `(first coll)`, then for each later element emit `sep` then the element.
- **`mapcat`** `(mapcat f coll)` — `f` returns a seqable per element; concatenate lazily: `(concat (f (first coll)) (mapcat f (rest coll)))`. **STOP-1: verify `:wat::core::concat` is LAZY over `Stream` (does not force its second arg).** If it is not, STOP and surface — do not hand-roll a lazy concat without flagging it.
- **`map-indexed`** `(map-indexed f coll)` — `f : Fn(i64,T)->U`. Helper `map-indexed-stream` carrying `idx`: `(cons (f idx (first coll)) (map-indexed-stream (+ idx 1) f (rest coll)))`; the defclause seeds `idx = 0`.
- **`keep`** `(keep f coll)` — **DIALECT (pinned): `f : Fn(T)->Option<U>`; keep the `Some` values, drop `None`** (wat's None-drop is clojure's nil-drop — the honest dialect form, `VIRTVTE PARES`). `(match (f (first coll)) ((Some v) (cons v (recurse rest))) (None (recurse rest)))`.
- **`keep-indexed`** `(keep-indexed f coll)` — as `keep`, `f : Fn(i64,T)->Option<U>`, helper carrying `idx`.
- **`dedupe`** `(dedupe coll)` — drop CONSECUTIVE duplicates. Helper `dedupe-stream` carrying `prev : Option<T>`: emit `x` when `prev` is `None` or `x != prev`, recurse with `prev = (Some x)`.
- **`distinct`** `(distinct coll)` — drop ALL duplicates (keep first). Helper `distinct-stream` carrying `seen : HashSet<T>`: emit `x` when `(not (contains? seen x))`, recurse with `(conj seen x)`. Uses `:wat::core::HashSet`/`contains?`/`conj` (grounded: exist — `wat/deporder.wat:46-48`). **STOP-2: if a generic `T` cannot seed a `HashSet<T>` (a type-bound gap), STOP and surface — do not silently pin `T` to `i64`.**
- **`reductions`** `(reductions f init coll)` + `(reductions f coll)` — emit `init`, then each successive accumulation. `(cons init (if (empty? coll) (stream/empty) (reductions f (f init (first coll)) (rest coll))))`; the 2-arity seeds from `(first coll)` (empty coll raises via `first`, mirroring `reduce`'s 2-arity — the same honest located failure).

## Blast radius
`wat/seq.wat` only (append the 12 defclauses + their helpers) + un-ignore `tests/types/probe_arc118_2z_takewhile_lazy.rs`. **No Rust edits. No new primitives. No new namespaces.** (`interleave`/variadic-`map`/`partition`/`min`/`max`/generators are OTHER strikes — do not build them here.)

## STOP triggers (rejection criteria — ship nothing, surface the gap)
- **STOP-1** — `:wat::core::concat` is not lazy over `Stream` (mapcat needs it). Surface; do not hand-roll silently.
- **STOP-2** — `distinct`'s `HashSet<T>` needs a type-bound the checker rejects for a generic `T`. Surface.
- **STOP-3** — `keep`'s `Option`-returning `f` won't type-check in a defclause body (the `match`/`Some`/`None` path). Surface; do not fall back to a nil-sentinel (`seq`-nil-punning is OUT per the DESIGN).
- **STOP-4** — any form needs a primitive beyond `first`/`rest`/`empty?`/`stream/{cons,lazy,empty}`/`concat`/`HashSet`/`contains?`/`conj`/`match`/`Option`. Surface it; do not invent a workaround.

## Expectations (scorecard — fixed before the strike)
| # | what | command | expected |
|---|---|---|---|
| 1 | the RED probe flips GREEN | `cargo nextest run --release -p wat --test types -E 'test(lazy_take_while_stops_before_forcing_late_boom)'` (un-ignored) | PASS |
| 2 | a lazy transformer returns a Stream | `(:wat::core::type (:wat::core::remove (fn [x] (:wat::core::= x 0)) [1 2 3]))` via `cargo wat` | `wat::stream::Stream` |
| 3 | keep drops None, keeps Some | `(:wat::core::into [] (:wat::core::keep (fn [x] (:wat::core::if (:wat::core::= x 0) :wat::core::None (:wat::core::Some x))) [0 1 0 2]))` | `[1 2]` |
| 4 | distinct dedups all; dedupe only consecutive | `(into [] (distinct [1 1 2 1 3]))` → `[1 2 3]`; `(into [] (dedupe [1 1 2 1 3]))` → `[1 2 1 3]` | as shown |
| 5 | whole workspace green | `cargo nextest run --release` | floor 0 (only the inline-wat meter red) |

## How to work
Capture the suite ONCE to a file, grep the FILE; targeted `cargo test -p wat --test <X>` runs — never re-run the
5-min suite to re-grep. Mirror `filter` for the direct-recursion forms and `reduce`/`reduce-stream` for the stateful
ones — copy the shape, do not invent it. Each form is one defclause; add them one at a time and keep the gate green
as you go. Report: the forms shipped; any STOP hit (with the grounding); the probe result; the nextest Summary line.
