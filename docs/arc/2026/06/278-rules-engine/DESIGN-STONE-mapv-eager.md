# DESIGN-STONE — mapv is eager native over seqable

> **Origin (2026-08-20).** Protocol query split on fanout
> `[40000]`: query-read **0.10 ms**, encode `(into [] (map
> f pv))` **287.7 ms**, sort **54.8 ms**, into-pv **2.7 ms**.
> Clara query+map+sort is **16.9 ms**. query-read is not the
> wall. Lazy map built 40k NativeThunk cons cells then
> applied `f` per force.

## The algorithm

`mapv` is a native eager walk of Vector / PersistentVector
/ List (position, not `rest`). Stream input still maps
then drains. Return `Vector<U>`. Token / fire / query-read
untouched. Fanout derived-vector uses `mapv`.

## ★ THE ONE CONTRACT DECISION

**`mapv` does not allocate a lazy Stream.** `map` stays
lazy. Query answers (PersistentVector of binding maps)
type-check as Seqable.

## Predicted win

encode **287.7 → ~80–150 ms** if thunks were material.
If apply_function owns the 288 ms, encode barely moves
and the leftover is the wat fn, not map.

## Gate

1. QuerySplit encode-ns printed. Insert diffs / rete lib.
2. clippy `-D warnings` (`--lib`).

## Weigh (2026-08-20) — LANDED, wash on encode

Fanout `[40000]` QuerySplit:

| lump | before | after |
|---|---:|---:|
| query-read | 0.10 | **0.10** |
| encode | 287.7 | **266.2** |
| sort | 54.8 | 50.6 |
| into-pv | 2.7 | 3.0 |

Thunks were **~21 ms**. apply_function owns encode.
Kept: PersistentVector query answers type-check;
eager mapv is the right verb. Next leftover: wat fn
per answer (266 ms) and sort' comparator (51 ms).
query-read is not the Clara gap.
