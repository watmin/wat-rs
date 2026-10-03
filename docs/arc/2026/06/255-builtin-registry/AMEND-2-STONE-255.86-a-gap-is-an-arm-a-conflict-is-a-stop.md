# AMEND 2 — STONE 255.86: a coverage gap is an arm; a conflict is a STOP

**Drawn 2026-10-03.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `971cf373e`.
Commit locally on `main`; **do not push**.

## The ruling on this STOP-1

Grok's finding is accepted: `:wat::map::dissoc` (and `keys`, `values`) accept a **PersistentMap** (`#pm{}`), and
`:wat::core::dissoc`/`keys`/`values` accept only a HashMap. The `into` clause for `(PersistentVector, PersistentVector)`
is accepted.

This is not the `concat` case. There, two verbs both accepted the same inputs and **disagreed**. Here the core verb simply
**lacks an arm** the per-type verb has. G1 makes the polymorphic core verb the one name for the operation, so a missing
arm is added to it, backed by the per-type verb's implementation, exactly as you added `into`'s clause.

**The rule for the rest of the stone, stated once:**

- **A coverage gap** (the per-type verb accepts a container type the core verb does not): add that arm to the core verb
  (its declared dispatch in `wat/core.wat`, or the Rust intrinsic's type table: whichever the core verb is), backed by
  the existing implementation, with a differential proving the per-type verb and the extended core verb return the same
  value on that type. Not a STOP. List every arm added in the SCORE.
- **A semantic conflict** (both verbs accept the same input and return different values, or the per-type verb's meaning
  is a different operation, as `vector/concat` was `into`): **STOP-1**, quote the pair.

## The work

Apply that rule to `map/dissoc`, `map/keys`, `map/values` (PersistentMap arms on `core/dissoc`, `core/keys`,
`core/values`), and to every remaining name in the mapping table. Then the brief's remaining items (new homes, codemods,
R-a, retirement, tests, gates).

STOPs as in the brief. A STOP means STOP. Append to the SCORE, commit, **do not push**.
