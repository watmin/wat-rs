# DESIGN-STONE — ONE DOOR for stepping an eager container (the seq-traversal waist)

> **Origin (2026-07-30/31, from the A8 chase):** the node-share axis spends 96–99% of its runtime
> outside the engine, and that part is O(M²). Splitting it phase by phase, then link by link, put
> the whole quadratic inside `query-by-type-string`'s final expression —
> `(into (PersistentVector) (filter pred all))` — and a pure-collections probe with no rete in it
> at all reproduced the same curve. The engine is clean; the collections layer underneath is not.
>
> **The builder's ruling:** *"we just found our next target — this is where we spend our effort —
> unify these such that there is precisely one way to solve things correctly."*

## The measurement (reproducible, on disk)

`wat-scripts/scratch-pad/probe-pv-lazy-materialize-cost.wat` — pure collections, no engine:

```
n      build  |  into-pv    into-vec  |  fold    rest-walk
200    0.1ms  |   19.5ms     20.9ms   |  0.3ms      5.8ms
1000   0.8ms  |  605.0ms    624.5ms   |  1.6ms    191.3ms
2000   1.6ms  | 2657.9ms   2734.0ms   |  2.8ms    932.8ms
4000   4.8ms  |11240.2ms  12108.5ms   |  7.1ms   3791.2ms
```

- `into-pv` and `into-vec` are within 8% of each other and both O(n²) — **the target container is
  irrelevant**, which kills the first hypothesis (that materialising into a PersistentVector was
  the defect).
- `fold` — the same work with no laziness — is **linear**, 7.1ms at n=4000. That is a **1,580×**
  gap against the identical computation expressed through `filter`.
- `rest-walk` — an explicit `first`/`rest` recursion, the mechanism isolated — is **O(n²)** with
  the same ×4-per-doubling shape.

And from `probe-derive-chain-split.wat`: `(into (Vector) (map f pv))` over the same 4000 elements
costs **67ms** where `(into (Vector) (filter pred pv))` costs **12,108ms**. Same source, same
target, same output cardinality. The difference is the *stage*.

## The root — and it is one function

`map` is native (`eval_vec_map`, `collection/transform.rs`) and iterates the container directly.
`filter` has no runtime dispatch at all: it is a wat `defclause` in `wat/seq.wat:59-102` whose body
steps its **eager** source by

```clojure
(:wat::core::filter pred (:wat::core::rest coll))
```

`rest` on any eager container rebuilds it: `PersistentVector` allocates a fresh `VectorSync` and
`push_back`-clones every remaining element (`collection/eval.rs:1643-1655`); `List` does
`.skip(1).cloned().collect()` into a new `LinkedList` (`:1609-1613`); `Vector` likewise. So each
step is O(n) and the walk is O(n²).

**`rest` is not the defect.** rpds 1.2.1's `Vector` exposes `drop_last`/`push_back` and has **no**
`drop_first` and **no** `push_front` — a bitmapped trie has no head operation, so the rebuild is
the only thing the container can do. Taking the tail of a vector is inherently O(n). The defect is
**asking a vector for its tail**.

### The choke point

`wat/seq.wat:251` defines `seqable->stream`, and the file's own comment at `:248` states the
intended architecture outright:

> *`seqable->stream` is the one-time normalization step that lets every clause … delegate to a
> single Stream-only `<form>-stream`*

That architecture is real and **half-adopted**. Six verbs already follow it. But `seqable->stream`
itself walks via `rest` (`:260/:265/:270/:277`), so:

| verbs | how they are quadratic |
|---|---|
| `keep` · `keep-indexed` · `take-nth` · `dedupe` · `distinct` · `map-indexed` | **transitively** — they normalise correctly, through a broken converter |
| `filter` · `remove` · `take-while` · `drop-while` · `interpose` · `reductions` · `seqable->stream` | **directly** — they hand-roll the same rest-walk instead of using the normaliser |

Either path, same result: **every lazy pipeline over an eager container in wat is O(n²)**.
Everything *downstream* of the boundary is healthy — `rest` on a `Stream` is `Arc::clone(tail)`,
O(1), so the Stream-only `<verb>-stream` implementations are all linear.

**The eager→lazy boundary is the entire bug.** 67 `rest` call-sites across 29 stages in `seq.wat`
are the symptom; one converter is the cause.

## The one contract decision — the normaliser goes NATIVE, under the seq-container registry

`seqable->stream` becomes a Rust primitive that produces a `Stream` stepping its source **by
position**, materialising nothing per element.

**Four-questioned against the alternative** (keep it in wat, over an index-carrying cursor form):

- **Obvious?** Native: YES — container-stepping knowledge belongs beside container-classification
  knowledge, which is exactly what the seq-container registry already is (R14). Wat-cursor: NO —
  it puts per-container traversal strategy back into the surface language.
- **Simple?** Native: YES — one implementation, dispatching through `StreamContainer` which already
  classifies every container. Wat-cursor: **NO** — it needs indexed access per container, and
  `List` is an `Arc<LinkedList>` with *no* indexed access; a cursor over it is O(n) per step and
  the quadratic survives on that arm. The native form can snapshot a non-indexable container once
  (O(n)) and then step O(1), staying linear overall. This is the deciding answer.
- **Honest?** Native: YES. Wat-cursor: **NO** — it would read as fixed while `List` stayed
  quadratic, which is the same silent divergence this stone exists to kill.
- **Good UX?** Native: YES — invisible. Every verb keeps its signature; user code is unchanged and
  simply stops being quadratic.

Native wins on all four; the wat-cursor form fails Simple and Honest on the `List` arm. Ruled.

**This is not a new substrate.** It is finishing one the codebase already chose and documented, and
that six of thirteen verbs already use — R2's "assembly, not invention".

## Why this is R14's waist, completed

R14 (`Phoenix`) named exactly this quarry and built the seq-container registry for it:

> *container-classification knowledge scattered as hand-rolled, per-op, per-side arms — `first`
> knows its container set in `check.rs` AND again in `runtime.rs`; `rest` separately; `conj`
> separately. That scatter IS the quarry.*

The registry unified **classification** — *what kind of container is this?* It never unified
**traversal** — *how do I step it?* That half is still hand-arms, and this is the drift it breeds:
`map` landed in Rust and is linear, `filter` landed in wat and is quadratic, and **nobody chose
that** — it is an artifact of which side of the language each verb happened to be written on.

After this stone a new stage cannot get the walk wrong, because no stage writes the walk.

## Scope + sequencing

- **Strike 1 (this stone)** — `seqable->stream` native + linear, under the registry. Six verbs go
  linear immediately, by delegation. RED-gated (below).
- **Strike 2** — migrate the hand-rollers onto the normaliser: each becomes
  `(<verb>-stream … (seqable->stream coll))`, the shape its siblings already have. This also
  deletes the per-container body duplication in `filter` and `remove`.

  **Census re-run AFTER Strike 1 landed** (`seqable->stream` is native now, so it leaves the
  list). Six verbs remain, and the work is bigger than "delegate" for five of them — the
  Stream-only twin they would delegate TO does not exist yet:

  | verb | eager arms | rest-walks | `<verb>-stream` twin |
  |---|---|---|---|
  | `filter` | 5 | 10 | **must be minted** |
  | `remove` | 4 | 10 | **must be minted** |
  | `take-while` | 4 | 5 | **must be minted** |
  | `drop-while` | 4 | 5 | **must be minted** |
  | `reductions` | 8 | 10 | **must be minted** |
  | `interpose` | 4 | 5 | exists (`interpose-stream`) |

  Existing twins to model on: `keep-stream` (`seq.wat:476`), `dedupe-stream`,
  `distinct-stream`, `keep-indexed-stream`, `map-indexed-stream`, `reduce-stream`.

  ### ⛔ THE TWIN ROUTE IS DEAD — Strike 2 goes NATIVE (ruled 2026-07-31)

  The census above scoped Strike 2 as "mint five `-stream` twins, then delegate." **That was
  wrong**, and the builder's challenge is what surfaced it: *"does clojure differentiate filter
  over data types? why do we need a -stream version? isn't that just filter vs filterv?"*

  **Clojure does not differentiate.** There is exactly one `filter`; it calls `seq` — the universal
  coercion — and walks an `ISeq`. One body, no per-container variants, no `filter-stream`.
  `filterv` is just the eager-vector convenience, which wat already has.

  wat cannot write that, and the reason is a **missing type**, grounded:
  - there is no `Seqable` type in any `.wat` file — the concept lives only in the Rust checker as
    `extract_lazyable_elem` (`collection/infer.rs:637`), a hardcoded match on four heads;
  - so a wat-level verb accepting several containers has exactly one option — a `defclause` with
    one arm per concrete container — and the `-stream` twin exists purely so those arms can share
    a body instead of duplicating it.

  **The twins are a workaround for the missing type, not a pattern.** Seven exist; the census would
  have minted five more. A twelfth is not "precisely one way to solve things correctly."

  The tell is which verbs need no twin at all: `map`, `take`, `drop`, and (since Strike 1)
  `seqable->stream` are **native**, so they dispatch on `StreamContainer` at runtime and
  `extract_lazyable_elem` at check time — and get Clojure's one-body-any-seqable shape for free.
  That is the same fault line that caused the original bug: `map` native and linear, `filter` wat
  and quadratic, chosen by nobody.

  **Four-questioned, native vs. a real `Seqable` surface:**

  | | native (chosen) | `Seqable` surface |
  |---|---|---|
  | Obvious? | YES — already what four shipped verbs are | YES — R28's model of what a surface is |
  | Simple? | YES — one body per verb, no new concept; deletes 12 twins + ~29 arms | **NO** — needs a surface nature admitting builtins (none), builtin surface-satisfaction (nothing does), and reconciliation with no-ad-hoc-unions (R7) |
  | Honest? | YES | *unreached* |
  | Good UX? | YES — signatures unchanged, quadratic gone | *unreached* |

  Native wins on a grounded NO, not a preference. The surface route is **an arc, not a stone** —
  filed as `109-kill-std/NOTE-seqable-has-no-name-in-wat.md`, with the blockers written down and
  the note that this stone is its **precondition** (it collapses the seqable set from ~30 spellings
  to one, which is what makes naming it tractable later).

  **The ladder, named honestly:** native reaches the **check** rung, not `no-form`. Afterwards
  "sequence verbs that take any seqable live in Rust" is a *convention* — nothing stops a new
  wat-level stage with per-container arms and a `rest`-walk tomorrow, and it would be quadratic and
  green. So this stone carries two things beyond the fix: the seqable set is **named** at its single
  definition site with its blockers beside it (so the 109 note is a marked delta, not an intention),
  and a lint is tracked to convert the convention into a wall.

  **Sequenced as 2a then 2b**, per prove-one-exemplar-then-fan:
  - **2a — `filter` alone**, made native end to end. It has the most arms (5), it is the verb this
    chase started from, and it is the one `query-by-type-string` calls — so 2a closes the A8 derive
    quadratic outright and the payoff is directly measurable, not inferred.
  - **2b — the remaining five** (`remove`, `take-while`, `drop-while`, `interpose`, `reductions`)
    fanned against 2a's proven shape, deleting their arms and the twins they'd have needed.
- **OUT — affirmatively cut, not deferred:**
  - `PersistentList` (an `rpds::ListSync`-backed persistent cons list). It is a genuine gap in the
    eager/persistent pairing (`Vector`/`PersistentVector`, `HashMap`/`PersistentMap`, `List`/—),
    but it is **not what this defect points at**: it would make head/tail cheap on *lists* while
    the `Vector` and `PersistentVector` arms — where the traffic is — stayed broken. Adding a
    container to accommodate a walk that should not exist is the wrong rung. Tracked, not built.
  - Making `rest` on a vector cheap. It cannot be, and it should not be.
  - Re-measuring the Clara grid. Separate, behind the memory guard, on a size ladder.

## The RED gate

A quadratic at n=4000 is ~12,000ms where linear is ~10ms — three orders of magnitude. So the gate
is a **wall, not a stopwatch**: `(into [] (filter pred coll))` over 4000 elements must complete in
under one second. That is RED today by 12×, GREEN after by 100×, and no plausible machine variance
crosses a 100× margin — it asserts the *absence of a complexity class*, not a performance number.
