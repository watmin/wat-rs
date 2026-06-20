# BRIEF — Stone 8-i: the wat accumulator fold library (`:wat::rete::acc::*`)

**You are a single-hop executor. Do NOT spawn sub-agents. Do NOT run git. Do NOT run `./target/release/wat`
(orchestrator-only; you MAY `cargo build`/`cargo test`).** Work ONLY in `/home/watmin/work/holon/wat-rs`.

## The work (one paragraph)

Add a small library of **pure wat fold fns** at `:wat::rete::acc::*` in `wat/rete.wat`, each folding a
`PV<Element>` into an `Option<…>` result. Value-folds read a BOUND `?var` (a `String` key) from each
element's bindings map. `mean` = `sum / count` (composition). Each returns `Option`: `None` = no token on
empty (min/max/mean of nothing); `Some(v)` otherwise. NO Rust, NO new primitive — these are wat folds over
existing core ops. (`apply-accumulator` + the AccumulateNode are 8-a; this stone is ONLY the folds.)
Contract: `DESIGN-STONE-8-accumulators.md`.

## Read in order (the rooms — all in `wat/rete.wat`)

1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-8-accumulators.md` — "the accumulators are simple pure
   folds" + the set + empty-set behavior.
2. The `Element` record (`~:30`) + its accessors `(:wat::rete::Element/fact e)` /
   `(:wat::rete::Element/bindings e)` — how a gathered element exposes its fact (a `:wat::Record`) + its
   bindings (`:wat::core::PersistentMap`).
3. Existing `foldl` usage in this file (e.g. `merge-facts`, `cross-join-node`) — the fold idiom over a PV.
4. The pure core ops you'll compose (all on the 6a pure allow-list): `:wat::core::foldl`,
   `:wat::core::PersistentMap/get` (→ `Option`), `:wat::core::PersistentVector` / `…/conj` / `…/length` /
   `…/contains?`, `:wat::core::+`, `:wat::core::<` / `:wat::core::>`, `:wat::core::/`, `:wat::core::Some` /
   `:wat::core::None` / `Option/expect`.
5. `tests/probe_arc278_8i_accumulator_folds.rs` — the 10 assertions to green (do NOT edit it). It builds
   Elements with bindings `{?bytes 100/200/300, ?port 80/443/80}` + Packet facts.

## The fold fns (signatures + behavior)

```
(:wat::rete::acc::count    [els <- PV<Element>])                     -> Option<i64>
   Some(length els).  empty → Some(0).
(:wat::rete::acc::sum      [var <- String  els <- PV<Element>])      -> Option<i64>
   Some(Σ bindings[var]).  empty → Some(0).
(:wat::rete::acc::min      [var <- String  els <- PV<Element>])      -> Option<i64>
   Some(min bindings[var]) via a <-comparison fold.  empty → None.
(:wat::rete::acc::max      [var <- String  els <- PV<Element>])      -> Option<i64>   empty → None.
(:wat::rete::acc::mean     [var <- String  els <- PV<Element>])      -> Option<i64>
   COMPOSITION: (/ (sum var els) (count els)) — reuse the two fns above.  empty → None.
(:wat::rete::acc::distinct [var <- String  els <- PV<Element>])      -> Option<PV<...>>
   Some(distinct bindings[var]) via a fold + contains? dedup.  empty → Some([]).
(:wat::rete::acc::all      [els <- PV<Element>])                     -> Option<PV<:wat::Record>>
   Some(PV of each element's fact).  empty → Some([]).
(:wat::rete::acc::group-by [var <- String  els <- PV<Element>])      -> Option<PersistentMap>
   Some(map bindings[var] → PV of facts), foldl into a PersistentMap.  empty → Some(empty map).
```

Read a var from an element: `(:wat::core::Option/expect -> :T (:wat::core::PersistentMap/get (:wat::rete::Element/bindings e) var) "acc: var unbound")`.

## Typing note (v1)

Value-folds (`sum`/`min`/`max`/`mean`) are **i64** in v1 (the probe uses i64). If a generic numeric type
forces a hard choice, you MAY type the read value as `:wat::core::Value` (the STONE-Value universal top, R7)
and narrow — but prefer i64 if it types cleanly. `distinct`'s element type can be `:wat::core::Value` or
i64; the probe only checks `length`. STOP-2 if the typing can't be made to compile.

## Blast radius (bounded)

- `wat/rete.wat` ONLY (a new `acc::` section). NO Rust. NO `apply-accumulator` (8-a). NO `compile-condition`
  / `fire-once` change (8-a). Do NOT touch the `render-dag` compound-concat fixture.

## STOP triggers (halt + surface; do not improvise)

1. If `Element/fact` / `Element/bindings` accessors don't exist as expected — STOP, report the real Element shape.
2. If the i64 typing of a fold can't compile and `:wat::core::Value` doesn't resolve it cleanly — STOP, report.
3. If a needed core op (a generic `min`/`max`, dedup, map-fold) isn't available — STOP, report what IS
   available (build min/max from `<`/`>`, dedup from `contains?` — do NOT invent a missing primitive).
4. If greening needs Rust or touching `compile-condition`/`fire-once` — STOP (that's 8-a).

## Done = green

`cargo test --release -p wat --test probe_arc278_8i_accumulator_folds` → 10/10. Then `cargo build --release`
clean + `cargo test --release --test test | grep "test result"` → 264/1 (rete.wat still compiles + loads).
