# BRIEF — 118.2a: the flip (core seq HOFs go lazy over Stream; clojure-named surface)

**The four-questioned decisions this strike executes** (`NOMINA NOTA, MACHINA TACITA`, 300 REALIZATIONS,
2026-07-03) — these OVERRIDE the older `DESIGN-118.2` where they differ:
- **Surface = clojure names, public in `:wat::core::`. Primitives = plumbing** (`:wat::stream::` cons/lazy/empty;
  `foldl`/`foldr`/`length` internal).
- **Decision B (core-direct):** `:wat::core::map` IS the lazy impl. `:wat::stream::` holds ONLY the type +
  `cons`/`lazy`/`empty` — **no `:wat::stream::map`.**
- **Eager = clojure names:** `mapv` `filterv` `vec` `into` `doall` `dorun`. **DROP the `:wat::seq::` namespace.**
- `length` → **`count`** (add `count`; `length` may stay as an internal primitive). `foldl` **STAYS the internal
  primitive** (250 sites untouched); **`reduce`** = the clojure surface built over it (2-arity + `reduced`-aware).
- `and`/`or` bool-strict; `nil` = unit / `#wat.core.Option/None nil` = absence — **unchanged, do not touch.**

## The work (one paragraph)
Flip wat's core sequence HOFs from eager Rust intrinsics to the clojure-faithful **lazy** surface. Replace the eager
`:wat::core::map`/`filter`/`take`/`drop` (Rust intrinsics in `src/collection/transform.rs`) with **lazy
implementations in WAT** over the `:wat::stream::` primitives (`cons`/`lazy`/`empty`) — the clojure-canonical
recursive shape — returning a `Stream`; input is any seqable (Vec|List|Stream) via the **already-polymorphic**
`first`/`rest`/`empty?` (verified: `(first (rest (stream/cons 1 (stream/cons 2 (empty)))))` → `2`). Add the eager
materializers `:wat::core::mapv`/`filterv`/`vec`/`into`/`doall`/`dorun` (force → `Vector`), `:wat::core::reduce`
(proper clojure reduce over `foldl`), `:wat::core::count` (over `length`), and `reduced` (early-exit marker). Retire
the eager Rust HOF intrinsics and the `:wat::seq::` namespace (promote its `reduce`/`fold` aliases to
`:wat::core::reduce`). The flip turns the tree RED where map callers expect a `Vector`; **drive the whole cascade to
green** by intent. Then un-ignore the RED probe.

## Read in order (the rooms)
1. `docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/DESIGN-118.2-hof-family.md` — the family design + the flip
   strategy + the cascade doctrine. (Its eager-opt-in `:wat::seq::` choice is SUPERSEDED by the clojure-names
   decision above — build the clojure names.)
2. `docs/arc/2026/07/300-wat-source-is-edn/REALIZATIONS.md` — the `NOMINA NOTA, MACHINA TACITA` interstitial (the
   ratified decisions, verbatim).
3. `tests/types/probe_arc118_2_lazy_map.{rs,wat}` — the RED probe (proven RED at HEAD: eager map forces a late
   div-by-zero). **Un-ignore it when `:wat::core::map` is lazy** — that is the load-bearing GREEN.
4. `src/stream/mod.rs` — the `Stream` type + `cons`/`lazy`/`empty` the lazy HOFs build on.
5. `src/collection/transform.rs` — the eager intrinsics to RETIRE: `eval_vec_map:320` · `eval_vec_filter:530` ·
   `eval_vec_take:107` · `eval_vec_drop:162`. **KEEP** `eval_vec_foldl:391` (the `reduce` primitive) + the `length`
   intrinsic. Check the dispatch in `src/collection/eval.rs` / `mod.rs`.
6. `wat/seq.wat` — the `:wat::seq::reduce`/`fold` aliases (promote to `:wat::core::reduce`; then retire `:wat::seq::`).

## Implementation sketch
- Lazy HOFs in WAT (Option-C recursive shape), in `:wat::core::`, over the stream primitives:
  `(defn :wat::core::map [f s] -> Stream (stream/lazy (if (empty? s) (stream/empty) (stream/cons (f (first s)) (:wat::core::map f (rest s))))))`
  — `filter`/`take`/`drop` analogous; input any seqable via polymorphic `first`/`rest`/`empty?`.
- Eager: `(defn :wat::core::mapv [f s] -> Vector (into [] (:wat::core::map f s)))`; `vec`/`into` force a `Stream`→`Vector`.
- `reduce`: proper clojure reduce (2 arities; `reduced` short-circuit) over the `foldl` primitive.
- `count`: over `length`.
- Retire the 4 eager intrinsics; keep `foldl`/`length`. Promote `:wat::seq::reduce`/`fold` → `:wat::core::reduce`; delete the `:wat::seq::` aliases.

## The cascade (the meter — SUBSTRATE-AS-TEACHER)
The flip makes `:wat::core::map` return a `Stream` where ~107 sites (~50 files) expect a `Vector`. **The red-test
count IS the progress meter** — each failure names the next site. Fix by intent:
- needs the result **eager** (length / index / `conj` / re-traverse / passed where a `Vector` is required) →
  `:wat::core::mapv` (or `(into [] …)` / `doall`).
- just **consumes once** (feeds `reduce`/`for-each`/another lazy stage) → **leave it lazy** (correct, often free).

Never stash-and-revert. Watch the red-count fall to zero.

## Blast radius
`src/collection/transform.rs` (retire 4 intrinsics) · `src/collection/eval.rs`/`mod.rs` (dispatch) · the lazy family's
wat home (`wat/core.wat` or a `wat/seq.wat`-successor) · `wat/seq.wat` (retire `:wat::seq::`) · the ~107-site cascade
across `src/` · `wat/` · `wat-tests/` · `crates/` · `tests/` (the WHOLE workspace).

## Out of scope (rejected — named, not deferred)
- The **extended lazy roster** (`remove`/`keep`/`mapcat`/`take-while`/`drop-while`/`distinct`/`partition`/
  `interleave`/`map-indexed`) + the **∞ generators** (`iterate`/`repeat`/`repeatedly`/`cycle`) — those are **118.2-Z**,
  a follow-on. 118.2a is the FLIP of the existing family + the eager materializers + `reduce`/`count`.
- Memoization / rewind — single-pass is the law.

## STOP triggers (reject, do not defer)
- STOP if a broken cascade site's eager-vs-lazy intent is genuinely ambiguous — surface it, do not guess.
- STOP if a lazy HOF needs a primitive that doesn't exist (beyond `first`/`rest`/`empty?`, which work) — surface it.
- STOP if `reduce`'s 2-arity / `reduced` semantics can't be expressed over `foldl` cleanly — surface it, do not hack.

## Done = green
- The RED probe (`probe_arc118_2_lazy_map`, **un-ignored**) is GREEN — `:wat::core::map` is lazy (does not force late elements).
- `(:wat::core::type (:wat::core::map inc [1 2 3]))` → `wat::stream::Stream`; `(:wat::core::type (:wat::core::mapv inc [1 2 3]))` → `wat::core::Vector`.
- `reduce` `count` `mapv` `filterv` `vec` `into` work (clojure semantics); `:wat::seq::` retired.
- `cargo nextest run --release`: **floor 0** (the whole cascade driven green) — read the **Summary line**, never a grep. Commit **GREEN**.

## How to work
Capture the suite ONCE to a file, grep the file; targeted `cargo test -p wat --test <X>` runs — never re-run the
5-min suite to re-grep. The cascade is the meter; each error names the next site. Report: files changed; the flip;
the cascade sites fixed (by intent, with counts); the nextest Summary line; any STOP hits.

---

## RATIFIED mid-flight (four-questioned, 2026-07-03) — decisions from the build's surfacings

- **`vec` → `(into [] coll)`, NOT `to-vec`.** Four-questions: (a) `to-vec` fails Obvious/Simple/Good-UX (a
  wat-ism in a clojure surface); (b) `into []` SWEEPS (clojure's actual eager forcer, no new name, respects arc-109's
  verb-equals-type retirement of `:wat::core::vec` → `:wat::core::Vector`); (c) un-retire `vec` reopens 109. **Ruling:
  (b).** Eager materialize is `(into [] coll)`; `mapv`/`filterv` are the map/filter shortcuts; **DROP `to-vec`**; the
  `:wat::core::vec` retirement error redirects → `into []`. (A future deliberate `vec`-revival is a separate
  109-revisit, not now.)

- **`reduced`/`reduced?` — STOP ACCEPTED; tracked as a follow-on stone.** wat's type universe is closed (`:Any`
  banned), so a reducing fn's `T | Reduced<T>` return can't be typed without a control-flow-signal mechanism
  (like `Result`/`try`) = new Rust plumbing = its own stone. `reduce` ships WITHOUT early-exit — behavior-identical
  to the old `:wat::seq::reduce`, **no regression**; workaround is `(reduce f (take n stream))`. (Strategic note: this
  same "user reducers" capability is what unblocks rete's **custom accumulators** — 278 BACKLOG, blocked on the
  collection grid whose one future-type was lazy-seq; finishing 118 opens it.)

- **The cascade fan-out — split by shape.** SHAPE-1 (mechanical `.wat` fixes: wrap eager consumers in
  `mapv`/`into`, leave lazy where it feeds `first`/`rest`/`reduce`/`empty?`) MAY be fanned out per-crate
  (`secare`-clean — each dependent crate has its own disjoint `wat/` tree). SHAPE-2 (Rust fixtures asserting the OLD
  eager/container-preserving contract) is **NOT fan-out work** — hand-triaged, orchestrator-weighed: a fixture is
  "obsolete" ONLY if it asserts the *retired eager contract* (map-returns-Vector); if it asserts something that should
  still hold, that is a regression to FIX, not a rewrite to pass (296's bar — never weaken a test to green). ONE green
  gate: whole-workspace `cargo nextest run --release` at floor-0 (the dependent crates' `wat/` trees load at
  full-binary startup), so nothing commits until all crates are green.
