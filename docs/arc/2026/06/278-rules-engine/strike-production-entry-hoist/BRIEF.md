# BRIEF — hoist `production_delta`'s entry out of the innermost loop

**Floor GREEN when you are done.** Counts only, no milliseconds.

## Read in order

1. **`DESIGN.md`** — note that it asserts NO lookup count. The last strike's DESIGN was wrong by 3×
   from reading source; measuring is your first job, not confirming mine.
2. **`src/rete/kernel/fire/pass/production.rs`** — `production_delta`'s four-deep nest, the
   `wm.production.entry(*node_id)` in the innermost body, and `census_count("prod:derivations")`
   nine lines above it.
3. **`src/rete/kernel/fire/pass/hash_join.rs:286-292`** — the same hoist, done, with its reasoning.
   **This is the shape to copy.**
4. **`src/rete/kernel/tests/fanout_cost.rs:214-240`** — the axis, and `prod:derivations == 40_000`.
5. **`tests/rank_and_instrument.rs::join_alpha_lookups_match_the_per_node_prediction`** — the
   count-then-predict gate from the previous strike. Reuse its shape.

## The work, in order

**1. Instrument and READ THE BEFORE.** A `#[cfg(test)]` counter on `wm.production.entry`. Record it
on the fanout cell alongside `prod:derivations`. **Report that number before you change anything** —
if it is not what the DESIGN's shape implies, that is a finding and the DESIGN is wrong again.

**2. Hoist.** A local `Vec<Value>` per production node, `extend`ed into `wm.production` once. Do not
hold an `entry()`'s `&mut` across the inner loops — the explain arm takes `&wm` whole.

**3. Gate the prediction.** One `entry` per production node that derives anything. A formula, not a
recorded integer.

**4. Behaviour identity.** Same facts, same rows, same order. `wm.derived_facts` ordering matters —
if buffering changes the sequence anything observes, that is STOP-2.

## Blast radius

`src/rete/kernel/fire/pass/production.rs` + `census.rs` + one test.

## STOP triggers

1. **If the before count is not one-per-derived-fact, STOP and report it.** The DESIGN's shape may be
   as wrong as the last one's; the measurement decides.
2. **If buffering changes derived-fact ORDER anywhere observable, STOP.** Order has bitten this
   engine before (F1). Same rows in a different sequence is a behaviour change.
3. **If the borrow forces holding `&mut wm.production` across the inner loops, STOP** — that is the
   shape the DESIGN rejected, and the explain arm is why.
4. **If a `*_cost` gate moves in the wrong direction, STOP and report the number.**
5. **On any RED: DO NOT RE-RUN.** Capture whole, name the arm, surface it.

## Prior result to copy for shape

`../strike-join-extend-hoist/` — and read its SCORE, where the rider corrected the DESIGN's count.
Do that again if the numbers say so.
