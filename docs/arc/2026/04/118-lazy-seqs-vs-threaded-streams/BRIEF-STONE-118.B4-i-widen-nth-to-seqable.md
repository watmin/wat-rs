# BRIEF — STONE 118.B4-i · `nth` becomes the general positional door

`nth` today is `Vector<T>`-only. `(first (drop X n))` — the idiom 44 corpus sites use instead — works
over any `Seqable<T>`, and stone B4-iii closes it. Widen `nth` so the capability survives the wall:
keep the existing O(1) body for the containers that have `get`, and add a `Seqable<T>` arm that walks
with `next`. Nothing closes in this strike; this is the door being built before the old one is taken
away.

## Read in order

1. **`wat/core.wat:1393–1407`** — `nth`'s header and its one-line body,
   `(Option/expect (get v i) "nth: index out of range")`. The header carefully distinguishes a
   total-CONTRACT from a partial-FUNCTION; **that argument is correct and stays** — you extend it,
   you do not rewrite it.
2. **`wat/seq.wat:293–305`** — `reduce`'s `defclause`. This is the exact shape to copy: a
   `Seqable<T>` parameter, `(Seqable/seq coll)` → `(:wat::stream::next …)` → `match` on
   `NextOutcome`, and `assertion-failed!` **by name** on `Exhausted`.
3. **`wat/seq.wat:75–92`** — the `Seqable<T>` surface and its four `extend-type`s (Vector,
   PersistentVector, List, Stream). This is the set your new arm reaches.
4. **`src/collection/seq_container.rs:65`** and the `gettable()` table (~:300) — the record of which
   containers have `get` (Vector, PersistentVector, List, WatAstList, HashSet) and the standing
   statement that *"Stream has no O(1) nth"*. Your Stream cost is documented before you write it.

## The strike path

Turn the single `defn` into a `defclause`. Concrete arms first, the surface arm last — first-match
wins, in declaration order.

```wat
(:wat::core::defclause :wat::core::nth
  ;; O(1) — the existing body, once per container that has `get`.
  ([v <- :wat::core::Vector<T> i <- :wat::core::i64] -> :T
    (:wat::core::Option/expect (:wat::core::get v i) "nth: index out of range"))
  ([v <- :wat::core::PersistentVector<T> i <- :wat::core::i64] -> :T
    (:wat::core::Option/expect (:wat::core::get v i) "nth: index out of range"))
  ([v <- :wat::core::List<T> i <- :wat::core::i64] -> :T
    (:wat::core::Option/expect (:wat::core::get v i) "nth: index out of range"))
  ;; O(n) — everything else Seqable, i.e. Stream. One force per cell visited.
  ([coll <- :wat::core::Seqable<T> i <- :wat::core::i64] -> :T
    (:wat::core::nth-walk (:wat::core::Seqable/seq coll) i)))

(:wat::core::defn :wat::core::nth-walk<T>
  [s <- :wat::stream::Stream<T> i <- :wat::core::i64] -> :T
  (:wat::core::match (:wat::stream::next s)
    ((:wat::stream::NextOutcome::Item value rest)
      (:wat::core::if (:wat::core::<= i 0) value (:wat::core::nth-walk rest (:wat::core::- i 1))))
    (:wat::stream::NextOutcome::Exhausted
      (:wat::kernel::assertion-failed! "nth: index out of range" :wat::core::None :wat::core::None))))
```

The three O(1) arms are byte-identical modulo the receiver type. **That duplication is expected and
tracked** — collapsing it needs a type meaning "eager indexable container", the same gap the 294 seam
already records for `reduce`'s three eager arms. Leave it; do not invent that type here.

## Blast radius

`wat/core.wat` (the definition) and new tests. **No `src/` edits. No other `.wat` file changes** — the
44 `(first (drop X n))` sites migrate in B4-ii, not here, and every existing `nth` caller passes a
Vector and keeps hitting arm 1 unchanged.

## Tests to add

New deftests, one assertion each, in the wat-tests file that owns core positional access (find it by
where the existing `nth` tests live; if none exist, a new `wat-tests/core/core-nth.wat`):

- `nth` at index 0, middle, and last, on **Vector**, **PersistentVector**, **List**, and **Stream** —
  same values, same answers.
- `nth` past the end raises `"nth: index out of range"` on **all four**.
- `nth` on a Stream visits **exactly i+1 cells** — build the generator from
  `wat-scripts/scratch-pad/probe-118B4-forces-per-element-by-walk-shape.wat`, whose `:user::gen`
  prints one line per realization, and assert the count.

## STOP triggers — each is "ship nothing, report the gap"

**STOP-1.** The reachability wall (`71099a2b`) refuses an arm no input can reach. If registering the
four arms is refused, STOP and report the exact `UnreachableClause` payload — the arm order or the
subsumption rule is the finding, and guessing a different order buries it.

**STOP-2.** If `get` is not accepted on `PersistentVector<T>` or `List<T>` at the checker, STOP and
report which one and its verbatim error. The `gettable()` table says both are supported; a
disagreement between that table and the checker is a substrate finding worth more than this stone.

**STOP-3.** If the `Seqable<T>` arm type-checks but does not dispatch for a Stream at runtime, STOP
and report. Door 1 (`13b27e8d`) is what makes surface-typed arms fire; a regression there is the
finding.

**STOP-4.** If any existing test changes its result, STOP and report the test name and both outputs.
This strike adds capability and changes nothing that already worked.

## Verification

Run everything in the FOREGROUND and block on it — your turn ends when the numbers are in your hands.

```
cargo build --release
systemd-run --user --scope -q -p MemoryMax=12G -p MemorySwapMax=0 timeout 1500 scripts/floor.sh
cargo clippy --release --all-targets -- -D warnings
```

Read the floor's **Summary line** from `.floor/latest/clean.log`. On any red: do not re-run; copy the
failing test's whole block from that log verbatim into your report, name the exact assertion that
fired, and stop.

## Prior result to copy for shape

`docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/DESIGN-STONE-118.B6-native-foldl-over-seqable.md`
and its tests — the same arc, the same `Seqable`/`next` vocabulary, and a differential test design
worth mirroring.
