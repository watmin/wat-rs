# BRIEF — assert the predicted visit count, not a ratio

**Floor GREEN when you are done.** Tests only; no engine change.

## Read in order

1. **`DESIGN.md`** — the symmetry argument is the whole finding; read the table.
2. **`src/rete/kernel/tests/rank_and_instrument.rs`** — `accum_gather_visits`,
   `keyed_gather_visits_do_not_scale_with_group_count`, and
   `keyed_gather_visits_per_instrumented_path` (added last strike).
3. **`src/rete/kernel/census.rs::gather_bucket`** — one count per element yielded, which is what
   makes a closed-form prediction possible at all.

## The work

**1. Predict, then assert.** For each driven path, assert the visit count **equals** the keyed
prediction:

```
simple keyed gather        G · W
and-exists (:and of two)   G · W · (1 + W)
```

Drive at **≥3 `(G,W)` points** per path — e.g. `(10,80) (20,40) (40,20) (80,10)`. A formula that fits
one point is a coincidence; three is a law.

**2. Mutation-prove it against the ratio's blind spot.** Simulate the whole-memory regression the
ratio cannot see — add `G · elements` to the observed count in the test's own arithmetic, or drive a
deliberately unkeyed variant if one can be built — and show:
   - the **ratio** assertion still passes (that is the point), and
   - the **equality** assertion goes RED.

Quote both. If you cannot build the regression, say so and simulate it in the assertion's arithmetic;
state clearly which you did.

**3. Keep the ratio.** It models shapes the formula does not. It is no longer the proof.

## Blast radius

`src/rete/kernel/tests/rank_and_instrument.rs` only. **Zero lines in `src/rete/kernel/fire/`.**

## STOP triggers

1. **If a prediction does not fit at some `(G,W)` point, STOP and report the point and both numbers.**
   That is either a wrong formula or a real scaling defect, and which one matters more than the gate.
2. **If a fold's prediction needs a constant fudge to fit, STOP.** A formula with a tuning term is a
   recorded number wearing algebra.
3. **If you cannot make the equality redden under the simulated regression, STOP** — then it is not a
   proof either, and this strike has not moved anything.
4. **On any RED: DO NOT RE-RUN.** Capture whole, name the arm, surface it.

## Prior result to copy for shape

`../strike-mark-is-a-prefix/` — where the rider reported that its own fire-level assertion did **not**
redden, and named the unit-level test that did. Be that honest about which assertion is the proof.
