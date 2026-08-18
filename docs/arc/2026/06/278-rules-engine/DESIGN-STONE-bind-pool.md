# DESIGN-STONE — bindings live in a fire-scoped pool; `Element` is a span

> **Origin (2026-08-18).** Inline-enum tried: drop 10.49 → 5.45, FIRE
> 76.85 → 78.38. Fatter `Element`, `alpha:push` ate the win. Do not
> retry inline. This stone keeps `Element` pointer-sized and batches
> the 80k small frees.

## The measurement

Drop is two bills (~5 ms each). Unique-`Arc` free is the one inline
killed. `Value` Drop remains. Inline moved the 5 ms onto push
because `N2` is four `Value`s.

`Element` today: `fact: Value` + `Arc<[(Value, Value)]>` ≈ pointer
plus a heap slice per match. After 2b the slice is never shared.
80k malloc/free of tiny slices is the bill.

## The algorithm

```
WorkingMemory.bind_pool: Vec<(Value, Value)>   // append-only during fire
Element { fact, off: u32, len: u16 }           // span into the pool

populate:
    off = pool.len()
    write pairs into pool                       // attach_fact first if ?p
    Element { fact.clone(), off, len }

clone (HashJoin right_idx): copies fact Arc + two integers
drop Element: fact Arc only
drop-memories: alpha.clear(); beta.clear(); bind_pool.clear();
```

Indices, not raw pointers. The vec may realloc; spans stay valid.
No `unsafe`. No leak. `Token.bindings` stays `PMap`.

Rematch (`exec_compiled_under`) still materializes a small `Arc`
and becomes a `PMap` — that path is not 80k Elements.

## ★ THE ONE CONTRACT DECISION

**An Element does not own its pairs.** The fire-scoped pool does.
Span `(off, len)` names a range in `bind_pool` in write order.
Empty bindings are `len = 0`. We do not skip `Drop` of the pool.

## The gate

1. `Element` has `off`/`len`, not `Arc`. Populate writes the pool.
2. `accum_fire_phase_census` `[200 200]`: fold < 25, snapshot < 1.
   drop printed, **not** wall-gated.
3. rete lib + `binary_id(wat::rete)`.
4. clippy `-D warnings`.

## Predicted win

drop 10.49 → **~5–6 ms**. push stays thin. FIRE 76.85 → **~71–73**.
If FIRE does not fall, leftover is `Value` Drop of facts — say so.

## Blast radius

`kernel.rs` (`Element`, `WorkingMemory`, `element_fact_bindings`,
encode/decode). `compiled_cond.rs` populate `exec_compiled` writes
the pool. `matcher.rs`: `Bindings` for `[(Value, Value)]`. No `.wat`.
No crate. No `unsafe`.

## Out of scope = REJECTED

- Raw pointers / bumpalo / `mem::forget`.
- Inline-enum (tried). Token.bindings. Fact-in-the-pool.
- Persist gather. 297.

## Sequencing

1. Pool + span. Populate writes it. Readers take `&pool`.
2. Weigh FIRE and drop. Stop.

## Weigh (2026-08-18) — LANDED

Census `[200 200]`:

| mark | before | after |
|---|---:|---:|
| FIRE | 76.85 | **67.33** |
| `round:drop-memories` | 10.49 | **3.63** |
| `alpha:push` raw | 7.34 | **6.76** (net below instrument) |

Inline moved cost onto push. The pool did not. Leftover drop is `Value` Drop of facts + one vec free. Do not arena-and-forget. Do not put facts in the pool unless a census names that 3.63.
