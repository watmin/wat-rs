# DESIGN STONE — 118.B5 · `into`'s Stream arms are the last INTERPRETED drain. Native kernel + wat oracle.

**Route B, stone 5 — and its premise is not the one B1 wrote down.**

## ⛔ B1's SKETCH WAS WRONG, AND THE DISK SAYS SO

`DESIGN-STONE-118.B1-mint-seqable.md:16` planned B5 as:

> *"`into` absorbs the drain; `stream->pvec` / `stream->vec` deleted."*

That assumed the two verbs were **redundant public siblings** of `into` — a second way to do what
`into` already does. They are not. They are **`into`'s own implementation**:

```wat
;; wat/seq.wat:166 — into, five arms
([to <- Vector<T>            from <- Vector<T>]    (concat to from))                    ← NATIVE
([to <- Vector<T>            from <- Stream<T>]    (stream->vec to from))               ← wat walk
([to <- PersistentVector<T>  from <- Stream<T>]    (stream->pvec to from))              ← wat walk
([to <- PersistentVector<T>  from <- Vector<T>]    (PersistentVector/concat to from))   ← NATIVE
([to <- Vector<T>            from <- PersistentVector<T>] …)                            ← NATIVE
```

`into` already absorbs the drain — measured live, `(into [] s)` and `(into (PersistentVector) s)`
both work today. **The verbs are not competitors to delete; they are the machinery underneath.** So
the stone is not a retirement. The question is whether that machinery deserves to stay interpreted.

## ★★ THE MEASUREMENT — decomposed, order-swapped, non-vacuity guarded

`wat-scripts/scratch-pad/bench-118B5-into-stream-vs-native-concat.wat`, n=200,000, capped:

```
map + drain      612 / 605 ms
DRAIN ONLY       529 / 532 ms      ← 87% of it
native concat     12 /  14 ms
```

All six runs land exactly 200,000 elements — the non-vacuity guard, printed, so a drain that
silently short-circuited could not post a flattering number. Both orderings agree, so it is not a
warm-cache artifact.

★ **THE DECOMPOSITION INVERTED MY ATTRIBUTION.** The obvious read is that `map`'s per-element
interpreted closure dominates. It is **13%**. The Stream round-trip is the other **87%**, and it is
**~44× the native concat** for the same materialization.
`[[feedback_measure_the_decomposition_never_read_it]]`

⚠ **State what the control can see.** `drain-only` uses `(Seqable/seq v)`, which builds a lazy chain
over the Vector; the walk that consumes it and the chain that produces it are the same traversal and
cannot be separated further by this instrument. So the honest claim is **"materializing a Vector
*through a Stream* costs 529ms against 12ms direct"** — the Stream round-trip as a whole — not "the
conj loop costs 529ms." Do not sharpen this number past what the control measured.

## What it does

The house shape, third application — `wat/rete.wat:1508`, B6's `foldl`, B4-0's `nth`:

```
stream->pvec-spec   wat/seq.wat     THE ORACLE — today's `next`-walk, unchanged, still wat
stream->vec-spec    wat/seq.wat     ditto
stream->pvec        src/            the native kernel — realize-and-collect, one pass
stream->vec         src/            ditto
into                unchanged       its two Stream arms keep calling the same public names,
                                    which now resolve to the native
                    differential    the spec keeps the native honest
```

`into`'s call sites do not move. Its Stream arms already name `stream->vec`/`stream->pvec`; those
names simply stop being interpreted.

## The four questions

- **Obvious? YES.** Every other arm of `into` is one native call. These two were the exception, and
  the exception cost 44×.
- **Simple? YES.** No new concept: the oracle/native split exists twice in this arc already, and the
  public surface is untouched.
- **Honest? YES.** It stops the collection surface having one materialization path that is two orders
  of magnitude off its siblings for no reason a caller can see. And the oracle keeps the native from
  drifting.
- **Good UX? YES.** Nothing changes at any call site; a pipeline that already read as "drain this
  into a vector" stops costing what it costs today.

## ⚠ THE TRAPS

**The native must not drain a Stream it should walk.** `nth`'s native was allowed to be O(n) because
positional lookup on a lazy seq IS a walk. This one is different: `into` is a *terminal* — it forces
everything by contract — so collecting in one native pass is honest here where it would have been a
regression there. Do not generalize B4-0's "walk, never drain" rule to this stone; the contracts
differ.

**Retention.** B3 measured lazy-pipeline retention flat at 0.38 B/elem. A native collector that holds
the whole realized chain alive while building would put that back. **The retention slope is an
acceptance row, not an afterthought** — `probe-118B-dorun-retention-slope.wat` already exists.

**`dorun` is `(into [] coll)` and bins the result** (`wat/seq.wat:194`). It builds an entire Vector to
discard it. That is a real waste and it is **NOT this stone** — it is a consumer question (should
`dorun` walk with `next` and discard?), tracked separately so this stone stays one thing.

## ACCEPTANCE

| | assertion | instrument |
|---|---|---|
| 1 | ★ **differential: native ≡ spec** on Vector and PersistentVector receivers, empty / 1 / n | a new `wat-tests/` differential + non-vacuity control |
| 2 | ★ **retention stays flat** | `probe-118B-dorun-retention-slope.wat` at 100k→800k |
| 3 | the drain closes most of the 44× | this bench, re-run |
| 4 | `into`'s public arms unchanged | read the diff — `wat/seq.wat:166` untouched except the spec rename |

Plus: floor ≥4760/0, clippy 0, ignores 13.

## Out of scope — affirmative cuts

- **`dorun`'s build-and-bin** — named above, tracked, not here.
- **`extract_lazyable_elem`** (13 hits in `src/`) — B2 deferred its deletion to "B5 / a Rust stone".
  It is a *checker* helper, not a drain; it rides with whichever stone touches `infer_map`/`infer_filter`.
- **`map`/`filter`/`foldr` over a Stream** — the rest of `mappable()`'s gap, still unowned.
