# DESIGN-STONE — `join_extend` skips rematch when there is no leftover

> **Origin (2026-08-18).** Fanout census `[100 20]`: FIRE **96.66 ms**.
> `hj:catchup:probe` **30.87 ms** (31.9%). That mark is `join_extend`
> × 40k. `join_extend` always `exec_compiled_under`s. Fold-the-wall
> already proved: no `SeedCmp` → rematch cannot reject a keyed member.

## The measurement

Catch-up probe: every left token, `key_of`, probe `right_idx`,
`join_extend` per bucket element. Fanout F=20, 2000 left × 20
right = 40,000 extends. `right_idx` clone is 0.23 ms — not the
row.

`join_extend` today:

```
fact_bindings_under(el, tok, compiled)  // always
extend_token(...)
```

`fact_bindings_under` is `exec_compiled_under` + unify. Populate
already ran `exec_compiled` (skips `SeedCmp`). The keyed bucket
already agrees on every shared `?var` (`gather_join_keys` /
`key_of` is that intersection).

Rematch is load-bearing **only** when the right cond has a leftover
`SeedCmp` (`where-join-left`). `CompiledCond::has_seed_cmp` names
that case — same predicate fold-the-wall uses.

## The algorithm

```
if compiled.has_seed_cmp() {
    if fact_bindings_under(...).is_none() { return None; }
}
extend_token(...)
```

No leftover → do not rematch. Bucket member is the join.

## ★ THE ONE CONTRACT DECISION

**Keyed equality on the shared vars is the join when the right
cond has no `SeedCmp`.** Leftover still rematches. Do not skip
`extend_token`. Do not change `where-join-left`.

## The gate

1. `join_extend` rematches iff `has_seed_cmp()`. Read the diff.
2. `fanout_fire_phase_census` `[100 20]`: ROUND LOOP > 0,
   hash-join > 0. Print `hj:catchup:probe`. Do not wall-gate FIRE.
3. rete lib + `binary_id(wat::rete)` (`where-join-left` lives here).
4. clippy `-D warnings`.

## Predicted win

`hj:catchup:probe` 30.87 → **~8–15 ms** (`extend_token` remains).
FIRE 96.66 → **~75–85**. Grid fanout `[40000]` 1.42 may widen.
If probe barely moves, leftover is `extend_token` — say so.

## Blast radius

`src/rete/kernel/fire/mod.rs` `join_extend` only. All four call sites
share it. No `.wat`.

## Out of scope = REJECTED

- `right_idx` as indices. Persist. HashSet insert. 297.
- Skipping `extend_token`. Changing `has_seed_cmp`.

## Sequencing

1. The `if`. Weigh the census. Stop.

## Weigh (2026-08-18) — LANDED

Fanout `[100 20]`:

| mark | before | after |
|---|---:|---:|
| FIRE | 96.66 | **72.43** |
| hash-join | 33.09 | **15.63** |
| `hj:catchup:probe` | 30.87 | **13.54** |
| production | 39.97 | **36.84** (50.9%) |

Probe halved. Leftover 13.54 is `extend_token` × 40k. Production
is now the wall on this cell. `where-join-left` still rematches.
