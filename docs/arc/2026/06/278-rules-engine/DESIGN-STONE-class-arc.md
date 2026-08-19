# DESIGN-STONE — `AggregateValue.class` is `Arc<str>`

> **Origin (2026-08-18).** Fanout leftover production **26.30 ms**.
> `prod:compiled-rhs` net **8.05**. `exec_compiled_rhs` does
> `class.clone()` into `AggregateValue::record` — a fresh `String`
> per derived fact. Compiled-rhs named this: *"interning it is a
> different, out-of-scope stone."* This is that stone.

## The measurement

40,000 Pairs. Each `record()` owns a new `String` of `"fan::Pair"`.
`CompiledRhs` already holds the class for the life of the arm.
Clone should be a refcount.

## The algorithm

```
AggregateValue.class: Arc<str>
from_parts / record / struct_ / holon_record: impl Into<Arc<str>>
CompiledRhs::Record.class: Arc<str>   // interned at compile_rhs
exec_compiled_rhs: class.clone()      // Arc bump
```

Eq / Debug / `== "wat::rete::Session"` stay content compares.
Identity stamp hashes the str bytes, same as today.

## ★ THE ONE CONTRACT DECISION

**Class is interned by sharing, not by a global table.** The arm
holds one `Arc<str>`; every derived fact of that class bumps it.
`String` constructors still `Into<Arc<str>>` (one alloc at birth).

## The gate

1. `CompiledRhs` class is `Arc<str>`. `exec_compiled_rhs` does not
   `String::clone` it.
2. `fanout_fire_phase_census` `[100 20]`: print production /
   compiled-rhs. Do not wall-gate FIRE.
3. rete lib + `binary_id(wat::rete)`.
4. clippy `-D warnings`.
5. Debug golden `probe_6_debug_contains_class` still matches.

## Predicted win

`prod:compiled-rhs` net 8.05 → **~5–7**. FIRE 61.35 → **~58–60**.
If compiled-rhs barely moves, leftover is fields `Vec` + `seen` —
say so; do not intern `names` in this stone.

## Blast radius

`value.rs` (`class` type + constructors). `compiled_rhs.rs`.
Call sites that `class.clone()` into a `String` slot (errors,
`type_name`). No `.wat`.

## Out of scope = REJECTED

- Global intern table. `names` intern. Persist. 297.

## Sequencing

1. Type + constructors. CompiledRhs. Fix compile.
2. Weigh. Stop.

## Weigh (2026-08-18) — LANDED as type; FIRE wash

`record(String)` stayed (inference). Hot path is `record_arc(Arc<str>)`.
Gate: rete lib 67, `binary_id(wat::rete)` 299, clippy `-D warnings`
silent, Debug golden `probe_6_debug_contains_class` still matches.

Fanout `[100 20]`:

| mark | before | after |
|---|---:|---:|
| FIRE | 61.35 | **60.90** |
| production | 26.30 | **25.31** |
| `prod:compiled-rhs` net | 8.05 | **8.78** |
| hash-join | 15.48 | **15.97** |

The `"fan::Pair"` `String` clone was not the 8 ms. Leftover is
fields `Vec` + identity stamp + `seen` of 40k Pairs. Do not intern
`names` next. Draw after a census names that row.
