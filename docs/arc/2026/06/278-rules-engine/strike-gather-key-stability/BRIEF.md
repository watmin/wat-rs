# BRIEF — measure gather key-set stability; hoist only if it is proven

**Floor GREEN when you are done.** Refuting §5 is a fine outcome. Landing a hoist without the
stability gate is not.

## Read in order

1. **`DESIGN.md`** — the hoist is gated on a correctness proof, not a count.
2. **`src/rete/kernel/fire/mod.rs:2076`** — `ensure_gather`; **`:1553`** `gather_join_keys` — note it
   takes the **sample's** bindings.
3. **`src/rete/kernel/tests/gather_probe_cost.rs:1-32`** — read the header. **Failure mode (b) is the
   thing this strike must not create**, and the stone says the gate cannot catch it.
4. **`src/rete/kernel/fire/pass/filter.rs`** — the `for tok in new_tokens` loop, and `driver` already
   hoisted beside where the key derivation would go.
5. **`../strike-col-field-of-measure/SCORE.md`** — the shape of a measurement strike whose answer
   surprised the DESIGN.

## The work, in order

**1. Instrument stability.** Per `(node_id, alpha_id)`: how many `ensure_gather` calls, and how many
**distinct** `join_keys` values across them. The distinct count is the whole strike.

**2. Drive the axes that reach it.** Negation, leading-exists, neg-consumer, accum — whatever
actually enters `ensure_gather`. Report calls and distinct-key-sets per axis, with denominators.

**3. Then branch, and say which in the SCORE:**
   - **distinct == 1 on every driven node** → hoist the derivation beside `driver`, **and land a gate
     asserting key-set stability per node**. Mutation-prove that gate: make two tokens at one node
     derive different key sets (a synthetic is fine if no real shape does it) and show it reddens.
   - **distinct > 1 anywhere** → report it, change no hot-path code, and close §5 as refuted. Name
     the shape that varies — that is a finding about the engine, not a failed strike.

## Blast radius

If refuted: `census.rs` + one test. If hoisted: `fire/mod.rs` + `fire/pass/filter.rs` + one gate.

## STOP triggers

1. **If you hoist without the stability gate, STOP.** The protection is currently by construction;
   converting it to a convention is the trade this arc has spent nine strikes undoing.
2. **If the stability gate cannot be made to redden, STOP** — then it is not a gate, and the hoist
   has no proof behind it.
3. **If a differential or oracle comparison moves at all, STOP.** Key-set collision shows up as wrong
   gather results, and that is the failure mode (b) the stone names.
4. **On any RED: DO NOT RE-RUN.** Capture whole, name the arm, surface it.

## Prior result to copy for shape

`../strike-gather-predicted-visits/` — where the deliverable was a proof that a weaker check passes
where the real one fails. Same discipline: the gate must catch what the old protection caught.
