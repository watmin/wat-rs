# DESIGN — the ratio is a proxy; the mechanism has a formula

> Drawn 2026-09-06 at HEAD `1546d94f2`. Source: the row the previous strike's own measurement
> exposed. **Arithmetic verified by the orchestrator; the readings are from my own drive.**

## What the last two strikes established

- `GATHER_VISITS` now counts every gather examination (one door, lint-enforced).
- The gate is driven over every instrumented path. Readings, constant 800 elements, tokens 10 → 80:

| path | G10W80 | G80W10 | ratio |
|---|---|---|---|
| old-axis, distinct, all, group-by | 800 | 800 | 1.00x |
| `and-exists` (cartesian) | 64800 | 8800 | 0.14x |

## The defect: the ratio test is EXACTLY blind on the widest path

`keyed_gather_visits_do_not_scale_with_group_count` asserts `big/small <= 2.0` on an axis holding
`G×W = 800` constant.

On `and-exists`, keyed visits are `G·W·(1+W)` = `800(1+W)`. A whole-memory-per-token regression adds
`G·elements` = `800·G`. The sum is

```
800(1 + W) + 800·G  =  800(1 + G + W)
```

**— symmetric in G and W.** So for every swapped pair on this axis the regressed readings are
*identical*, and the ratio is **exactly 1.00**:

| (G,W) | keyed | regressed | | (W,G) | keyed | regressed | ratio (regressed) |
|---|---|---|---|---|---|---|---|
| 10,80 | 64800 | **72800** | | 80,10 | 8800 | **72800** | **1.00** |
| 20,40 | 32800 | **48800** | | 40,20 | 16800 | **48800** | **1.00** |
| 8,100 | 80800 | **87200** | | 100,8 | 7200 | **87200** | **1.00** |

This is not "the axis is weak at some points". **The ratio cannot detect this regression class at any
point of this axis**, because holding `G×W` constant makes the regressed quantity symmetric under the
swap the ratio is built from.

The simple paths are not blind — `800 → 8800` vs `800 → 64800` gives 8.00x, caught. **The blindness
is specific to the path that does its own quadratic work**, which is the one where a scan would hide
best.

## The one contract decision, pinned

**Assert the PREDICTED VISIT COUNT, not a ratio bound.**

The mechanism has a closed form: each token probes its own bucket.

| path | keyed prediction |
|---|---|
| simple keyed gather | `G · W` |
| `and-exists` (`:and` of two Leaves under `:exists`) | `G · W · (1 + W)` |

Both fit the observed readings exactly at two points each. A whole-memory scan adds `G · elements` —
which breaks the equality at **every** point, on **every** path, regardless of how G and W trade off.

⚠ **This is a formula, not a recorded number.** `[[gate-the-ratio-not-the-millisecond]]` warns against
pinning a figure you read off a run; this pins the *invariant the mechanism implies*, in terms of the
axis parameters. If the prediction and the reading disagree, one of them is wrong and the test says
which — that is the whole point.

Keep the ratio assertion as a second reading — it catches shapes the formula does not model — but the
**equality is the proof**.

## Scope

**IN:** the predicted-count assertion per path, driven at ≥3 `(G,W)` points, plus a mutation proving
it reddens where the ratio does not. Floor GREEN.

**OUT, affirmatively cut:** deriving a prediction for folds not yet driven (`min`/`max`/`User` share
walked arms already covered — say so); census B, D–M; `temperare`; A4, D2p, F2.

## Why this matters beyond the gate

Three strikes in, this instrument has failed at three different levels: the **counting** was
incomplete, then the **coverage** was incomplete, and now the **experimental design** is. Each was
invisible from the level above. A green gate was never the same claim as a held mechanism, and this
is what it costs to close the difference.
