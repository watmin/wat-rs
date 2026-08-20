# DESIGN-STONE — insert' resolves `facts` from the value's names

> **Origin (2026-08-20).** FIRE internable ≥ 1 is scratch STOP /
> fanout OUT refused. Insertion unparked. Probe on this tip
> n=20 000: insert − conj **2037 ns/fact** (witnesses held).
> `eval_insert_native` on every success: `format!(":{}", class)`,
> TypeEnv lookup, `available: Vec<String>` of every field name
> (`to_string` × 8). The vec is only read on miss. Arc 296 G:
> the Aggregate already carries `names`.

## The measurement we have

`probe-insert-cost-split.wat` n=20 000, release, this tip:

| arm | ns/fact |
|---|---:|
| baseline | 3230 |
| conj | 2665 |
| insert | 4702 |
| **insert − conj** | **2037** |

Parked note named this alloc. TypeEnv is a second source of
"what is `facts`" next to `agg.names`.

## The algorithm

```
facts_idx = agg.names.iter().position(|n| n == "facts")
```

Miss: `UnknownField` with `available` cloned from `names`
(the only path that needs it). Class must be
`wat::rete::Session`. No TypeEnv. No `format!`. Same helper
for `insert-all'`. Token / fire path untouched.

1. **STOP intern** if insert − conj does not fall ≥ 0.5 µs.
2. Do not hardcode slot 5. Do not Session-`Vec`.
3. Do not route 2-ary `insert` through `insert-all`.

## ★ THE ONE CONTRACT DECISION

**`insert'` / `insert-all'` resolve `facts` from the
Aggregate's carried `names`.** Still by name, never a
positional literal. TypeEnv is not on the hot path.
`available` exists only on miss.

## The gate

1. `probe-insert-cost-split.wat` n=20 000 prints insert −
   conj. Witnesses = n. Do not wall-gate a µs number.
2. rete lib (insert differentials).
3. clippy `-D warnings` (`--lib`).

## Predicted win

Independent guess (written first): insert − conj
**2037 → ~900 ns**. Eight `to_string`s + `format!` +
TypeEnv get own ~1 µs. Conj / Session rebuild remain.

## Blast radius

`kernel.rs` `eval_insert_native` + `eval_insert_all_native`.
No `.wat`. No fire path.

## Out of scope = REJECTED

- Session-`Vec`. Hardcoded facts index. 2e / 2o. 297.
- Routing 2-ary insert through insert-all. Scratch. Intern
  `names` of facts. Identity stamp skip (already 0 on Session).

## Sequencing

1. Helper from `agg.names`. Both primes. Weigh insert − conj.
   Stop.

## Weigh (2026-08-20) — LANDED, under the 0.5 µs bar

Gate: rete lib 99, insert differentials 7/7, clippy
`-D warnings` silent. Probe n=20 000 witnesses held.

| | ns/fact |
|---|---:|
| insert − conj before | 2037 |
| insert − conj after | **1650** |
| cut | **387** |

Predicted → ~900 **missed**. Eight `to_string`s were not
1 µs. Kept: `available` only on miss; TypeEnv off the
hot path; still by name. Do not hardcode slot 5.

Leftover **1650 ns** is Session rebuild + PV conj +
defclause. Unique-owner `make_mut` is the next intern
if named. Do not Session-`Vec`. Do not route 2-ary
through insert-all.
