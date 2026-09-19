# EXPECTATIONS — census F

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | rename complete | `grep -rn '"dbeta:alloc"' src/ tests/` → **no hits** |
| 2 ★ | the mutation REDs | delete the bump → `node_share_cost.rs:314`'s `fire_gathers > 0` fails; quote verbatim; restore |
| 3 ★ | the human-facing label is true | `gather_probe_cost.rs`'s column header no longer says `allocating`; quote the printed table |
| 4 ★ | quantity unchanged | the counted expression is still `u64::from(!out.is_empty())`; the identity at `:314` still holds with the same numbers |
| 5 ★ | the rejection is recorded | counting real allocations named as considered-and-rejected at the bump site, with the arm-L reason |
| 6 | siblings untouched | `dbeta:calls`, `dbeta:tokens`, `dbeta:multi` unchanged |
| 7 | NAMES list still sorted | `accum_cost.rs`'s exact-equality list updated and in order |
| 8 | floor | **≥ 5465 — final number and what moved it.** New arms are a PASS; name them |
| 9 | clippy | rc=0 |

★ load-bearing. **Row 2 is the one this strike can actually produce** — D and E could not, and
saying so was right there. Here a red exists, so a red is required.

## Trap doors, named in advance

- **"Improving" the counter to count real allocations.** It would exceed `dbeta:calls`, break the
  identity at `node_share_cost.rs:314`, and mis-scale arm L's replay silently. Rejected.
- **Renaming the struct field but not the column header** (or vice versa) — the printed word is the
  live defect; the field is bookkeeping.
- **Touching a correctly-named sibling.**
- **Leaving the NAMES list unsorted** after the substitution.
- **Re-running a red.** Capture whole, name the arm, surface it.

## What would make me reject the result

- `dbeta:alloc` still emitted anywhere in code.
- A mutation proof that drove a copy instead of the live test, or was skipped.
- The printed table still saying `allocating`.
- Any change to what is counted.
