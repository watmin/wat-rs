# DESIGN STONE — 118.B1 · mint `:wat::core::Seqable<T>`. Additive. Nothing consumes it yet.

**Builder, 2026-08-17: *"B has been reasoned…. do your measurements and build."*** Route B ruled
(`DECISIONS-118.B-four-questioned.md`); the measurements are in `MEASURED-118.B-memo-off-is-flat.md`.

This is **stone 1 of 5**, and the split is chosen so that **no body is written twice**. An earlier
sketch had the 8 walkers migrated to `next` first and *then* collapsed into `Seqable` clauses —
that rewrites the same bodies in two stones. `[[feedback_do_not_defer_content_on_mechanisms_difficulty]]`

```
B1  mint Seqable<T> + 4 extend-types                    ← THIS STONE. purely additive.
B2  collapse each verb to ONE clause over Seqable, body walking with `next`.
    the 7 twins and `seqable->stream` die here — in the stone that removes their last caller.
B3  delete BOTH memos (LazyCell:66 and NativeLazyCell:124). measure.
B4  close the three doors — first/rest/empty? on Stream become unrepresentable.
B5  `into` absorbs the drain; `stream->pvec` / `stream->vec` deleted.
```

## What it is

```wat
(:wat::core::defsurface :wat::core::Seqable<T> :nature :wat::core::Struct
  :features [(seq [self <- :wat::core::Seqable<T>] -> :wat::stream::Stream<T>)])
```

plus four `extend-type`s — `Vector`, `PersistentVector`, `List`, `Stream` — exactly the four heads
`extract_lazyable_elem` hardcodes (`src/collection/infer.rs:665`).

`seq` is **Clojure's name and Clojure's contract**: the universal coercion every collection
implements, returning something walkable. It is also the chain doc's own signature
(`255/CHAIN-rendering-before-the-string-home.md` step D). **`:wat::core::seq` is free** — verified,
no existing binding.

## Why this and not `as-vec`

`probe-seqable-parametric-all-four.wat` used `as-vec` because it only had to prove *satisfaction*.
A real `as-vec` would **materialize**, which is the opposite of the arc's purpose. `seq` returns a
`Stream<T>` and stays lazy.

## The four questions

- **Obvious? YES.** It is `ISeq`. One name for "any sequence", the thing the checker has believed
  privately since `extract_lazyable_elem` was written.
- **Simple? YES.** One `defsurface`, four `extend-type`s, on a mechanism proven green
  (118.3-B, `a15f4ea9`) and re-verified running this session: the parametric four-container probe
  prints `3,4,5,2`.
- **Honest? YES.** It is additive and claims nothing else: no verb collapses, no twin dies, the
  checker's hardcoded four-head match is untouched. B1 does **not** claim the split brain is closed.
- **Good UX? YES.** After B2 a user can write a lazy stage in wat over `Seqable<T>` — the same
  thing the stdlib writes. That is the whole point of route B, and B1 is its precondition.

## Rooms

| what | where |
|---|---|
| the surface + 4 impls | **top of `wat/seq.wat`**, above the materializers |
| load position | `src/stdlib.rs:67` — after `core.wat` (`:40`), which already carries a `defsurface`, so the form is proven to work this early |
| the four heads to match | `src/collection/infer.rs:665` `extract_lazyable_elem` |
| the normalizer the impls delegate to | native `seqable->stream` (`eval_seqable_to_stream`, `src/collection/transform.rs`) |
| worked exemplar | `wat-scripts/scratch-pad/probe-seqable-parametric-all-four.wat` — runs, `3,4,5,2` |

## ⚠ Trap — named before the strike

**★ THE STDLIB HAS NEVER `extend-type`'d A BUILTIN.** Verified: zero occurrences in `wat/`. Every
proof we have that this works is a **user-level file loaded after the entire stdlib**. B1 does it
*inside* the stdlib at load position 4. If builtin surface-satisfaction has a load-order dependency,
this is exactly where it surfaces — and the failure would be a load-time error, not a check error.
That is the stone's single real risk and its first gate row.

Secondary: `seqable->stream` is **kept** by this stone. That is not the preserve-and-extend reflex —
its last caller is a `-stream` twin's parent clause, and **all of those die in B2.** A name dies in
the stone that removes its final caller, not before. B2 deletes it; that is written into B2's scope,
not deferred.

## The gate

| # | assertion |
|---|---|
| 0 | ★ **NON-VACUITY** — before the edit, `(:wat::core::Seqable/seq …)` errors as unknown; captured verbatim |
| 1 | ★★ **the stdlib LOADS** with a builtin `extend-type` in it — the trap above |
| 2 | `(:wat::core::Seqable/seq v)` on a **Vector** yields a Stream that drains to the same elements, in order |
| 3 | same for **PersistentVector** |
| 4 | same for **List** |
| 5 | same for **Stream** (identity-ish: drains to the same elements) |
| 6 | ★ a **generic wat fn** `[s <- :wat::core::Seqable<T>]` is **CALLED** with all four — not merely declared. 118.3-B's lesson: declarations alone prove nothing |
| 7 | laziness preserved — `seq` on a Stream does **not** force the whole chain |
| 8 | `seqable->stream` unchanged; no verb collapses; no twin deleted |
| 9 | floor GREEN via `scripts/floor.sh` — the Summary line |
| 10 | `cargo clippy --release --all-targets` → 0 |
| 11 | `#[ignore]` count **13** |

Row 6 is the stone. Rows 8 and 9 are what make it additive.

## Out of scope — affirmative cuts, each with its owning stone

- **Collapsing any verb to a single `Seqable` clause** — **B2**.
- **Deleting the 7 twins or `seqable->stream`** — **B2**, which removes their last callers.
- **Deleting `extract_lazyable_elem`** — **B2**; it cannot go while the native verbs still consult it.
- **Deleting the memos** — **B3**. Measured today: memo-off is flat, and the migrate-then-delete
  order is load-bearing (a surviving three-call walker runs user code 3× — proven, 15-for-5).
- **The three doors** — **B4**.
- **`stream->pvec` / `stream->vec`** — **B5**.

## What B1 does NOT claim

That the split brain is closed, that any twin is gone, or that a user can yet write a sequence verb
the way the stdlib does. B1 mints the type. **B2 is where the payoff lands**, and B1 exists so B2
has one thing to write instead of two.
