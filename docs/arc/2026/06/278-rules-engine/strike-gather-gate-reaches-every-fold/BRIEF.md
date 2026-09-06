# BRIEF — drive the keyed-gather gate over every instrumented path

**Floor GREEN when you are done.** This strike ships a measurement; the ratio may move.

## Read in order

1. **`DESIGN.md`** — the outcome is data, and the threshold is not the variable.
2. **`src/rete/kernel/tests/rank_and_instrument.rs:13`** — `ACCUM_GATHER_WORLD`, the three-rule
   fixture, and `accum_gather_visits` / `keyed_gather_visits_do_not_scale_with_group_count`.
3. **`src/rete/kernel/fire/acc.rs`** — the `AccFold` arms. Note `Count` is `bucket.len()` (O(1)) and
   `Distinct | All | GroupBy | User` materialises the bucket.
4. **`src/rete/kernel/fire/mod.rs`** — the two no-`SeedCmp` arms: one returns `!bucket.is_empty()`
   (O(1)), one maps over the bucket. **Determine which shape reaches the mapping one** — that is the
   part of this brief I could not settle by reading.
5. **`src/rete/kernel/census.rs::gather_bucket`** — the door every examination now goes through.

## The work

**1. Extend the fixture** so every instrumented path is entered: at least one of
`distinct` / `all` / `group-by`, plus a shape that reaches `fire/mod.rs`'s mapping no-`SeedCmp` arm.
Keep the existing rules — this is an addition, and the old readings must stay comparable.

**2. Read the ratio per path, not just in total.** A single aggregate number cannot say which fold
scales. Report visits for the old axis and the new folds separately, both at `G=10 W=80` and
`G=80 W=10`.

**3. Report.** Every reading, before and after, with the per-fold split.

## Blast radius

`src/rete/kernel/tests/rank_and_instrument.rs` only. **No `src/` engine change** — if you find
yourself editing `acc.rs` or `fire/mod.rs`, you have left the strike (see STOP-1).

## STOP triggers

1. **If a newly driven fold makes the ratio exceed 2.0, STOP and report.** Do not cure it here, do
   not raise the constant, do not narrow the fixture back. That crossing is the next strike.
2. **If a fold cannot be driven from the wat surface, STOP and say which and why.** An unreachable
   fold is a finding about the surface, not a reason to skip it silently.
3. **If extending the fixture changes the OLD readings** (`800/800`), STOP — the addition perturbed
   the existing axis and the comparison is no longer honest.
4. **On any RED: DO NOT RE-RUN.** Capture whole, name the arm, surface it.

## Prior result to copy for shape

`../strike-explain-order/` — a probe that needed **eight** producers where two proved nothing. If a
fold's visit count looks flat, check whether the fixture is large enough to discriminate before
concluding the mechanism holds.
