# BRIEF — hoist the per-alpha triple out of the innermost loop

**Floor GREEN when you are done.** No wall-clock claim.

## Read in order

1. **`DESIGN.md`** — the deliverable is a COUNT, not a time, and the hasher is out of scope.
2. **`src/rete/kernel/fire/mod.rs:686`** — `join_extend`; **`rematch_compiled`** and
   **`span_from_row`**'s prologue (`bind_only.get`, `cond_key_ids.get`).
3. **The five call sites** — `hash_join.rs:239`, `:439`, `:504`, `fire/mod.rs:771`, `:874`. Confirm
   `alpha_id` is loop-invariant at each; if any is NOT, say so — that changes the hoist.
4. **`fire/mod.rs:57`** — `FireCtx`'s reason for existing.
5. **`src/rete/kernel/census.rs::gather_bucket`** and
   **`tests/rank_and_instrument.rs::keyed_gather_visits_match_the_keyed_prediction`** — the
   count-then-predict pattern this strike reuses. Copy it.

## The work, in order

**1. Instrument first, hoist second.** Add a `#[cfg(test)]` lookup counter at the three sites and
**record the current count** on an existing cost axis. That reading is the before.

**2. Hoist.** Resolve `(compiled, bind_only_fields, cond_key_ids)` once per join node; carry it in
`FireCtx`. `join_extend` takes the resolved handle rather than `alpha_id` plus three maps.

**3. Gate the predicted count.** After the hoist the lookups are **3 per join node**. Assert that as
a formula in the axis parameters — the shape
`keyed_gather_visits_match_the_keyed_prediction` uses — not a recorded integer.

**4. Prove behaviour is identical.** Same facts, same rows, same fires. The differential and the
oracle comparisons are the evidence.

## Blast radius

`src/rete/kernel/fire/` + `census.rs` + one test. **No hasher change. No wat corpus change.**

## STOP triggers

1. **If `alpha_id` is NOT loop-invariant at some call site, STOP and report which.** The hoist's
   shape depends on it and I verified only the pattern, not all five.
2. **If the hoist changes ANY fact-level result, STOP.** This is a pure resolution move; a behaviour
   change means it is not.
3. **If a `*_cost` gate moves in the WRONG direction, STOP and report the number.** Fewer lookups
   should not cost more; if it does, the borrow shape is wrong.
4. **If you find yourself switching a hasher to make a number better, STOP.** Out of scope, and it is
   a correctness question (see the DESIGN).
5. **On any RED: DO NOT RE-RUN.** Capture whole, name the arm, surface it.

## Prior result to copy for shape

`../strike-gather-predicted-visits/` — count per operation, predict from the mechanism, assert
equality, and mutation-prove that the equality reddens where a weaker check would not.
