# DESIGN — the keyed-gather gate measures two folds of seven

> Drawn 2026-09-06 at HEAD `c186e3e11`. Source: the open row the previous strike left behind —
> reported by the rider against its own work. **Verified on disk at THIS HEAD.**

## What the last strike bought, and what it did not

`GATHER_VISITS` now counts every gather examination: `census::gather_bucket` is the one door, and a
lint refuses a raw bucket walk in the gather modules. That is the **form** closed.

**The gate's coverage did not move.** Driven, before and after: `800/800 = 1.00x`. The three
newly-counted paths are never entered by the fixture, so instrumenting them changed nothing the gate
can see.

## The gap, measured

`ACCUM_GATHER_WORLD` (`tests/rank_and_instrument.rs:13`) declares three rules:
`acc::count`, `acc::sum`, and an `exists`.

The tree has **seven** folds — `count`, `sum`, `min`, `max`, `group-by`, `distinct`, `all` — and
`fire/acc.rs`'s arms split them:

| arm | folds | what it does to the bucket |
|---|---|---|
| `AccFold::Count` | `count` | `bucket.len()` — **O(1), examines nothing** |
| `AccFold::Sum/Min/Max` | 3 of 7 | walks, counted before and after |
| `AccFold::Distinct \| All \| GroupBy \| User` | **3 of 7** | materialises the whole bucket — **the path just instrumented, and never driven** |

Plus `fire/mod.rs`'s no-`SeedCmp` arm that maps over the bucket — also newly counted, also unentered.

So `keyed_gather_visits_do_not_scale_with_group_count` — whose failure message says *"the gathers are
still scanning the whole memory per token instead of probing a key index"* — asserts that over an
axis exercising **two folds of seven and one of the two exists arms**.

## The one contract decision, pinned

**Extend the fixture until every instrumented path is entered, then read the ratio and report it.**

This strike ships a **measurement**, not a cure. There may be nothing to fix. The deliverable is that
the gate's guarantee stops being structural-only and becomes observed.

## ⛔ The outcome is DATA, and the threshold is not the variable

`ratio <= 2.0` currently holds at 1.00x over two folds.

- **If it still holds with every fold driven** — the keyed-gather mechanism is intact across the whole
  surface, and the gate finally proves what its message claims.
- **If it crosses 2.0** — the gathers scale with the token count on a path nothing has ever watched.
  **That is a FINDING and it needs its own strike.** Do not raise the constant, do not narrow the
  fixture, do not split the assertion to keep the old axis green in isolation.

Both outcomes are a successful strike. Only a tuned threshold is a failure.

## Scope

**IN:** the fixture extension (a `distinct` / `all` / `group-by` rule, and a shape reaching the
no-`SeedCmp` map arm), the ratio read per fold, and the report. Floor GREEN.

**OUT, affirmatively cut:** curing any scaling this reveals — that is the next strike, drawn from
what this measures; census sections B, D–M; `temperare`'s cost rows; A4, D2p, F2.

## Why this is worth a strike on its own

Every previous cure this session made a wrong thing unwritable. This one asks whether a *right* thing
is actually true on the two thirds of the surface nobody has measured. The lint says the counting is
complete; only a fixture can say the mechanism is.
