# DESIGN-STONE — production walks `d_beta`, does not clone Tokens

> **Origin (2026-08-18).** Join rematch skip landed. Fanout `[100 20]`
> FIRE **72.43 ms**. Production **36.84 ms** (50.9%). Children:
> `prod:compiled-rhs` net 9.74, `prod:dedup-store` net 1.94.
> Unmarked inside production ≈ **25 ms**.

## The measurement

Production does this before the 40k RHS loop:

```
let mut new_tokens: Vec<Token> = Vec::new();
for pid in parents {
    new_tokens.extend(d_beta[pid].iter().cloned());
}
for tok in new_tokens { exec_compiled_rhs(&tok.bindings, ...) }
```

`d_beta` already owns the tokens. Production only **reads**
`tok.bindings`. Clone copies `matches: Vec` + `PMap` × 40k.
That is the unmarked 25 ms. `support` is `None` on `fire-rules`
(the clone of `tok` there is not this cell).

## The algorithm

```
for pid in parents {
    for tok in d_beta[pid] {          // borrow
        for compiled in rhs_forms {
            exec_compiled_rhs(..., &tok.bindings, ...)
            seen.insert ...
        }
    }
}
```

No `Vec<Token>`. Same order: parents in `parents_of` order, tokens
in `d_beta` order. Empty parents still skip.

## ★ THE ONE CONTRACT DECISION

**Production reads parent `d_beta` in place.** It does not own a
copy. `seen` / `wm.production` / `next_delta` still take the
derived `Value`. `support` still clones the token when armed.

## The gate

1. No `new_tokens.extend(...cloned())` in the production pass.
   Read the diff.
2. `fanout_fire_phase_census` `[100 20]`: hash-join > 0. Print
   production. Do not wall-gate FIRE.
3. rete lib + `binary_id(wat::rete)`.
4. clippy `-D warnings`.

## Predicted win

production 36.84 → **~12–20 ms** (RHS + seen remain). FIRE 72.43
→ **~50–60**. If production barely moves, leftover is RHS
construct — say so; do not intern class `String` in this stone.

## Blast radius

`src/rete/kernel.rs` production loop only. No `.wat`.

## Out of scope = REJECTED

- Interning `AggregateValue::record` class `String`.
- `right_idx` rewrite. Persist. 297. HashSet insert.

## Sequencing

1. Walk `d_beta`. Weigh. Stop.

## Weigh (2026-08-18) — LANDED, half the unmarked

Fanout `[100 20]`:

| mark | before | after |
|---|---:|---:|
| FIRE | 72.43 | **61.35** |
| production | 36.84 | **26.30** |
| `prod:compiled-rhs` net | 9.74 | **8.05** |
| hash-join | 15.63 | **15.48** |

Token clone was ~10 ms, not the whole unmarked 25. Leftover
production is RHS construct + `seen` + loop. Do not intern
class `String` unless a census names that row.
