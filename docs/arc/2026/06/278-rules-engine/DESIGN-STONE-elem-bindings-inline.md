# DESIGN-STONE — `Element.bindings` is inline at width 0–2

> **Origin (2026-08-18).** Weigh after 2d. `[200 200]` FIRE **76.85 ms**.
> Largest named leftover: `round:drop-memories` **10.49 ms**. The clear
> is load-bearing (`alpha-is-fire-scoped`). This stone is the unique
> heap behind that drop.

## The measurement

`wm.alpha.clear()` drops ~80k `Element`s. Each owns
`bindings: Arc<[(Value, Value)]>`. After 2b, `d_alpha` is indices.
The only remaining `el.clone()` is HashJoin `right_idx` — idle on
this accum cell. The slice is allocated once and freed once, never
shared. `Arc` is paying a heap header for a clone we no longer do.

Width on this axis is 1 or 2 (element-bindings-array census). A
wide condition (3+) is legal and spills.

Un-Arc to `Vec` still mallocs. Skipping `Drop` leaks Arcs. The
cut is **inline**.

## The algorithm

```
ElemBindings = N0 | N1(pair) | N2([pair; 2]) | Many(Vec)
Element.bindings: ElemBindings
exec_compiled / attach_fact materialize this, not Arc
```

`as_slice()` is `[..]` / `from_ref` / the array / the vec.
`Bindings` impl is that slice. Token.bindings stays `PMap`.

## ★ THE ONE CONTRACT DECISION

**Element bindings are uniquely owned.** Width 0–2 lives in the
enum (no heap). Width 3+ spills to `Vec`. Order and pairs are
unchanged. We do not hash pointer identity. We do not leak.

## The gate

1. `Element.bindings` is `ElemBindings`. `exec_compiled` returns
   it. No `Arc<[(Value, Value)]>` on the fire populate path.
2. `accum_fire_phase_census` `[200 200]`: fold < 25, snapshot < 1.
   `round:drop-memories` printed, **not** wall-gated.
3. rete lib + `binary_id(wat::rete)`.
4. clippy `-D warnings`.

## Predicted win

drop 10.49 → **~6–8 ms** (heap free of 80k unique Arcs gone;
Value drops remain). FIRE 76.85 → **~72–74**. If drop stays ~10,
leftover is `Value` Drop (fact Arc + keywords) — say so; do not
arena-and-forget.

## Blast radius

`matcher.rs` (`ElemBindings` + `Bindings`). `compiled_cond.rs`
(materialize / attach_fact). `kernel.rs` (`Element`, helpers
that took `&Arc<[…]>`). No `.wat`. No crate.

## Out of scope = REJECTED

- `smallvec` crate. Arena / `mem::forget`. Token.bindings.
- Persist gather. Second hasher. 297.

## Sequencing

1. Type + `Bindings` + materialize.
2. Weigh drop. Stop.

## Weigh (2026-08-18) — FIRE did not fall. Reverted.

Built. Census `[200 200]`:

| mark | before | after inline |
|---|---:|---:|
| FIRE | 76.85 | **78.38** |
| `round:drop-memories` | 10.49 | **5.45** |
| `alpha:push` net | 0.32 | **7.49** |

Drop halved — the unique-Arc free was real. `ElemBindings::N2` is four `Value`s, so every `Element` got fatter and `Vec::push` of 80k of them ate the win. Mechanism true, wall false.

**Reverted.** Do not retry inline-enum / `SmallVec<[T;2]>` for this row. Killing the heap without fattening `Element` is a side-table or arena of pairs (Element stays a pointer). Not this stone. Do not arena-and-forget.
