# DESIGN-STONE — extend does not copy left binds

> **Origin (2026-08-18).** Match-pool landed. Probe **11.39** (wash).
> FIRE **44.23**. Leftover probe is bind-pool append + `key_of` +
> fact clone. `key_of` is 2,000 left tokens. Append is **40,000**
> copies of the left pairs.

## The measurement

`extend_token` after 2m/2n:

```
concat left matches + (fact, alpha)     // 2n wash
concat left binds + right-only keys     // copies ?k ?l every join
```

Fanout: left has 2 pairs. 40k × clone of those pairs is the
append. Root already holds them. The keyed bucket unified `?k`.

## The algorithm

```
Token { matches, binds, extra }   // all BindSpan, still Copy

root-join: binds = el.binds, extra = empty
extend:
    binds = tok.binds                 // share, no copy
    extra = tok.extra + right-only    // new keys only
get / iter: binds then extra
```

Fanout extra starts empty; the one join writes `?r` only.
Deep cascade copies `extra` (small) not the root binds.
PMap / harvest / accum still flatten both spans.

## ★ THE ONE CONTRACT DECISION

**Left bindings are the root span. Extend only writes keys
the left token does not hold.** Token stays `Copy`. Do not
inline. Do not skip matches.

## The gate

1. `Token.extra` exists. `extend_token` does not copy
   `tok.binds` pairs. Readers search both spans.
2. `fanout_fire_phase_census` `[100 20]`: print probe. Do not
   wall-gate FIRE.
3. rete lib + `binary_id(wat::rete)`.
4. clippy `-D warnings`.

## Predicted win

`hj:catchup:probe` 11.39 → **~7–10**. FIRE 44.23 → **~40–43**.
If probe barely moves, leftover is `key_of` + fact clone —
say so.

## Blast radius

`kernel.rs` Token + extend + `Bindings` view + harvest/encode/
accum flatten. No `.wat`.

## Out of scope = REJECTED

- SmallVec. Skip matches. `key_of` rewrite. Persist. 297.

## Sequencing

1. Extra span. Flatten at the PMap door. Weigh. Stop.

## Weigh (2026-08-18) — TRIED, not a FIRE win. Reverted.

Fanout `[100 20]`:

| mark | before | after |
|---|---:|---:|
| FIRE | 44.23 | **46.95** |
| `hj:catchup:probe` | 11.39 | **12.54** |
| production | 19.34 | **21.76** |

Two-span `get` (binds then extra) cost more than the left-pair
copy. Same class as 2e: the intern missed, the view ate the
win. Reverted. Leftover probe is not left-bind copy. Production
**19.34** is the wall.
