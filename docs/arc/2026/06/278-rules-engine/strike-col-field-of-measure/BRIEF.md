# BRIEF — count `col_field_of`, then decide

**Floor GREEN when you are done.** The deliverable is a NUMBER and a DECISION. Refuting the row is a
successful outcome — do not hoist something the count does not justify.

## Read in order

1. **`DESIGN.md`** — it asks for a measurement, not a cure, and explains why.
2. **`src/rete/kernel/fire/mod.rs:1719`** — `key_of_el`, and **the `if el.binds.len > 0` early
   return**. This is the branch that may make §4 vacuous.
3. **`fire/mod.rs:1662`** — `col_field_of`: two map lookups plus two linear `position` scans, one
   over `bind_keys` with `Value` equality.
4. **The seven `key_of_el` call sites** — `hash_join.rs:187/:320/:441`, `fire/mod.rs:815/:895/:1797/:1961`.
5. **`tests/rank_and_instrument.rs::prod_entry_lookups_match_the_per_node_prediction`** — the
   count-then-decide shape from §3. Reuse it.

## The work

**1. Two counters, separable.** `col_field_of` entries, and `key_of_el` taking the `binds.len == 0`
branch. Report both on every axis you drive, alongside a denominator that makes them meaningful
(elements considered, or pairs).

**2. Drive the axes that exist.** The fanout cell, the accum cell, and whatever the three
`hash_join.rs` sites sit on. Do not invent a new workload to make a number look big.

**3. Decide, and say which in the SCORE:**
   - **`col_field_of` runs on a meaningful fraction** → hoist it out of the per-element loops (the
     hoist already exists twice in `fire/mod.rs` — copy it), and gate the predicted count.
   - **It runs rarely or never** → **report that plainly and change no code.** Quote the counts.
     `temperare` §4 is then refuted and the board row closes as measured.

**4. Answer the `from_wm` question too.** Is constructing it in the loop a cost, or eight reference
copies the compiler removes? A count or a clear reading either way; I do not believe it is a cost and
the SCORE should say whether I am right.

## Blast radius

If refuted: `census.rs` + one test, and the counters may be reverted or kept — say which and why.
If hoisted: `fire/mod.rs` + `hash_join.rs` + one test.

## STOP triggers

1. **If you find yourself widening a fixture to make `col_field_of` run more, STOP.** That is fitting
   the workload to the desired answer. Drive the axes that exist.
2. **If the hoist would change key ORDER or key identity, STOP.** `key_of_el` feeds join keys; F1 is
   why order gets its own trigger.
3. **If a `*_cost` gate moves, STOP and report the number.**
4. **On any RED: DO NOT RE-RUN.** Capture whole, name the arm, surface it.

## Prior result to copy for shape

`../strike-join-extend-hoist/` — where the rider's count contradicted the DESIGN and it said so in
the SCORE. That is the outcome this strike is most likely to have.
