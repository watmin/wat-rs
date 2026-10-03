# AMEND — STONE 255.86: `vector/concat` is `into`

**Drawn 2026-10-03.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `5b5647e03`
(grok's STOP-1). Commit locally on `main`; **do not push**.

## The ruling on STOP-1

Grok's differential is accepted: `:wat::vector::concat` on `(PersistentVector, Vector)` returns `#pv[1, 2]`;
`:wat::core::concat` refuses the mixed pair; same-kind pairs agree.

**The mixed case is a designed behaviour, and it already has its G1 name.** `src/intrinsic/vector.rs:170` cites
`DESIGN-STONE-into-pv-from-vector.md`: `(:wat::vector::concat to from)` puts `from`'s contents into `to`. That is
**`into`**, and `wat.core/into` exists: the polymorphic `defclause` at `wat/seq.wat:181` (498 uses in `.wat`), whose
`(PersistentVector, Vector)` clause is itself `(:wat::vector::concat to from)`. Clojure draws the same line: `into`
returns `to`'s collection type; `concat` does not mix kinds.

So **`:wat::vector::concat` maps to `wat.core/into`**, not `wat.core/concat`. Its implementation stays as the body of
`into`'s clause (unregistered, like every other backing impl under G1), and the name retires with a remedy naming
`wat.core/into`.

## The work

1. **`into` covers every pair `vector::concat` accepts.** Measure which `(to, from)` kinds `:wat::vector::concat` accepts
   today; for each, `into` must have a clause returning the same value (a differential per pair). Add the missing clause
   only if a pair has none (e.g. `(PersistentVector, PersistentVector)`), backed by the same implementation.
2. **The mapping table is per name, not per operation:** each retiring name gets its own target, proven by its own
   differential (this case shows a family can split). Continue the stone from item 1 of the brief with that discipline;
   any other name whose differential fails is STOP-1 again.
3. The rest of the brief as written (new homes, codemods, R-a, retirement, tests, gates).

STOPs as in the brief. A STOP means STOP. Append to the SCORE, commit, **do not push**.
