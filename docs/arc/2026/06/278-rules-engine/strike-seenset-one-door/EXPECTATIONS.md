# EXPECTATIONS — A4 / SeenSet

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | behaviour-neutral | `prod:derivations` and `seen_facts` unchanged on a named axis, quoted before and after |
| 2 ★ | one door | `grep -rn 'seen_ids\|seen_rest' src/` → hits only inside `SeenSet`'s own module |
| 3 ★ | the bypass does not compile | probe from **outside** the owning module; quote `error[E0616]`/`E0609` |
| 4 ★ | the guard, or its refusal | the `debug_assert` shipped — or STOP-2 with the measured cost and the reason |
| 5 ★ | severity honest | the SCORE says the hazard is NOT live (`from_parts` is the sole stamping site) and that this is hygiene plus a guard |
| 6 | the hand-summed cardinality is gone | `round_census.rs:133` reads `seen.len()` |
| 7 | floor | **≥ 5471 — final number and what moved it.** New arms are a PASS; name them |
| 8 | clippy | rc=0 |

★ load-bearing. **Row 1 is the whole claim** — a refactor that changes a value is a failed refactor.
Row 5 is what stops the type reading as a correctness fix it is not.

## Trap doors, named in advance

- **Letting `SeenSet` read as a cure for the cross-path hazard.** It is not; an unstamped-but-shallow
  aggregate still takes the `rest` arm inside the door. The guard is what addresses that, and only
  in debug.
- **Proving the bypass from inside the module**, where a private field is legitimately reachable
  (`[[a-mutation-proof-must-drive-the-gate-not-a-copy]]`).
- **Merging the halves** into one `FxHashSet<Value>` — discards a measured fast path.
- **"Fixing" the arc 109 note** by touching `src/value/`.
- **Re-running a red.** Capture whole, name the arm, surface it.

## What would make me reject the result

- Any fixpoint count moving.
- `seen_ids`/`seen_rest` still reachable outside the module.
- A bypass probe written inside the owning module.
- The severity written up as a correctness fix.
