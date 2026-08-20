# DESIGN-STONE — split `materialize_into`

> **Origin (2026-08-19).** 8 ranked `M−T`: intern/
> materialize **6.18**, ops 1.90, intern-cold tax 0.22.
> `fact_bind` 0. 80,200 successes. The 6.18 is clone +
> `intern_key` + `intern_val` HashMap get + `pool.push`,
> unsplit. Guessing HashMap is how this arc interned the
> wrong row (7: tree and push were not small). This stone
> prints the split. It does not intern.

## The measurement we do not have

`materialize_into` per success:

```
clone scratch slot
intern_key  (linear, tiny unique keys)
intern_val  (FxHashMap<Value, u32> get)
pool.push   (Copy (u32, u32))
```

8's Mc−Mw 0.22 says first-insert of ~400 i64s is not the
row. The get + clone + push still runs 80,200 times.

## The algorithm

Same fixture as 8. Mean of 3. Cold intern each run
(bind_pool + intern tables reset). `intern_key` /
`intern_val` / `materialize_into` are `pub(crate)`.

Stacked on 80,200 ops-true:

```
O   T + exec_ops
C   O + clone output slots
K   C + intern_key
V   K + intern_val
P   V + pool.push
M   O + materialize_into     // control ≈ 8's Mc
```

Deltas: `C−O` clone, `K−C` intern_key, `V−K` intern_val,
`P−V` push, `M−P` leftover in the real function.

Drawable only if a lump is ≥ 1 ms **and** not 2o / names
/ stamp / Session-`Vec`.

## ★ THE ONE CONTRACT DECISION

**This stone prints the split. It does not change the
engine** besides `pub(crate)` on the three helpers. Do
not restore per-fact timers. Do not intern off this rank.

## The gate

1. `accum_materialize_split` prints O/C/K/V/P/M and
   deltas. V > 0. Do not wall-gate FIRE.
2. rete lib.
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): **`V−K` (`intern_val`
get) owns most of 6.18.** Clone of `Value::i64` small.
`intern_key` tiny (2–4 keys). Push tiny (Copy). If clone
or push owns ≥ 1 ms, say so — 7's lesson. Do not intern.

## Blast radius

`compiled_cond.rs` vis. One kernel test. No `.wat`.
Token stays two spans.

## Out of scope = REJECTED

- Per-fact timers. Intern this stone. Tagged-i64 ids.
- Intern `names`. Facts in `bind_pool`. 2e / 2o. 297.
- Fact insertion. Tree intern. Fold seen.

## Sequencing

1. Helpers `pub(crate)`. Isolated stacked loops. Print.
2. Rank. Stop. Do not intern.

## Weigh (2026-08-19) — LANDED, no intern

`accum_materialize_split`, 40,200 facts, mean of 3.
M−O **5.26** (tracks 8's 6.18).

| lump | ms |
|---|---:|
| C−O clone | **1.02** |
| K−C intern_key | 0.86 |
| **V−K intern_val** | **2.77** |
| P−V pool.push | 0.28 |
| M−P leftover | 0.34 |

Prediction held on intern_val as the largest *piece*
of the pile, not on "most of 6.18" (2.77 / 5.26 ≈ half).
Clone is also ≥ 1 ms. intern_key just under. Push tiny.

After the split, intern_val **2.77 is not the largest
FIRE leftover** — tree 4.46, Element push 3.45, and
`setup:seen` ~3.9 sit above it. Next intern if named:
**alpha-tree walk 4.46**, not tagged-i64 intern.
