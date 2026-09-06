# EXPECTATIONS — FactBag: one owner for the fact base

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | command | expected |
|---|---|---|---|
| 1 ★ | **behaviour-neutral** | `scripts/floor.sh` | **5460 passed**, same as HEAD — not "green", the SAME NUMBER |
| 2 ★ | one owner in wat | `grep -rn "Session/facts\|FactBag/items" wat/ \| grep -v factbag.wat` | **no hits** |
| 3 ★ | two doors in Rust | `grep -rn '"facts"' src/rete/` | only inside `session.rs`'s two door bodies |
| 4 ★ | the gate is mutation-proved | drive the LIVE gate, once per rule | **3 REDs**, quoted verbatim |
| 5 ★ | the codemod is the migration | `wat-scripts/fixes/<name>.wat` committed; re-run | **0 changes** (idempotent) |
| 6 | `retract` semantics untouched | `remove-every-equal` body | byte-identical fold to today's |
| 7 | the grid does not move | `check-grid-three-way.sh` | same verdicts as HEAD |
| 8 | clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 |

★ load-bearing. **Row 1 is the whole claim: a migration that changes a value is a failed migration.**

## Trap doors, named in advance

- **`retain` losing the sub-multiset property.** `fire.wat:316-322`'s convergence test is exact
  *only* because the step is an intersection with F itself. A `retain` that reorders or dedups
  turns a termination proof into a false fixpoint — *"a silent wrong answer, worse than the defect."*
- **The codemod wrapping a site that already unwraps**, producing `items(items(x))`.
- **Claiming rung 3.** The gate is a build gate. The record cannot hold the seal today
  (`NOTE-a-records-restricted-to-is-stored-and-never-enforced.md`). The header must say so.
- **A floor number that moves in either direction.** More tests passing is also a change.
- **Re-running a red.** Capture whole, name the arm, surface it.

## What would make me reject the result

- A floor number that is not exactly 5460.
- A gate whose mutation proof drove a copy instead of the live gate.
- Hand-edited `.wat` where a codemod was the method.
- Any behaviour change smuggled in — especially `retract`.
- The gate's header claiming the type enforces this.
