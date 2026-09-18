# BRIEF — batch root_join_delta's token writes; measure its span branch

**Floor GREEN when you are done.** Counts only. Refuting half of §2 is a fine outcome.

## Read in order

1. **`DESIGN.md`** — it asserts no counts, and says why: my reading was wrong on §1 and §4 in
   opposite directions, both times on a population question, and this row hinges on the same one.
2. **`src/rete/kernel/fire/pass/root_join.rs:55-82`** — the three-deep nest, the `binds.len > 0`
   branch, and the per-element `record_token`.
3. **`src/rete/kernel/fire/pass/mod.rs`** — `record_token` and **`record_tokens`**, and read
   `record_tokens`' doc: it names this exact cost.
4. **`src/rete/kernel/fire/pass/production.rs`** — the buffer-then-flush cure from two strikes ago.
   **Same shape, and its SCORE recorded the order check that made it safe.**
5. **`tests/rank_and_instrument.rs::prod_entry_lookups_match_the_per_node_prediction`** — the
   count-then-predict gate. Reuse it.

## The work, in order

**1. Two counters, separable.** `span_from_row` entries from this loop, and `record_token` calls
from this loop. Report both with a denominator (elements seeded).

**2. Batch the token writes.** Buffer per `(node_id, child_id)`, flush once via `record_tokens`.
Predicted: three hash ops per `(node, child)`, not per element.

**3. ⛔ Prove the order is unchanged BEFORE you claim the batch is safe.** `wm.beta` and `d_beta` are
read by later passes, and `push_match` mutates `wm.match_pool` per element in the same loop. Show
that the token sequence landing in both memories is byte-identical to before — the differential and
oracle suites are the evidence, and say so explicitly in the SCORE.

**4. Decide on the span branch from the count.** Meaningful fraction → hoist the two `*node_id`
lookups to node scope. Rare → report the number and leave the code alone.

## Blast radius

`src/rete/kernel/fire/pass/root_join.rs` + `census.rs` + one test.

## STOP triggers

1. **If buffering changes token ORDER in `wm.beta` or `d_beta` anywhere observable, STOP.** This is
   the strike's one real risk. Same tokens in a different sequence is a behaviour change.
2. **If `push_match`'s interleaving with the buffer changes what a token's `matches` points at,
   STOP.** It mutates `wm.match_pool` inside the loop.
3. **If the span count is near zero and you hoist it anyway, that is churn** — report and leave it.
4. **If a `*_cost` gate moves in the wrong direction, STOP and report the number.**
5. **On any RED: DO NOT RE-RUN.** Capture whole, name the arm, surface it.

## Prior result to copy for shape

`../strike-production-entry-hoist/` — buffer, flush once, and a SCORE that proved order was
preserved by naming what reads the memory mid-nest.
