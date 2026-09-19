# EXPECTATIONS — insert reports the verb you wrote

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | the driven before/after | the probe from DESIGN, quoted both ways: `:op` `insert-all` → `:op` `insert` |
| 2 ★ | two arms | both entries gated, each asserting the **`:op` field** |
| 3 ★ | two mutations | re-hardcode each caller in turn; each REDs its own arm; quote both; restore |
| 4 ★ | the const is gone | `grep -n 'const OP' insert.rs` → no `insert-all` const in `insert_facts_on_session` |
| 5 | `insert-all` unchanged | its arm still reports `insert-all` |
| 6 | floor | **≥ 5470 — final number and what moved it.** The probe's tests are a PASS; name them |
| 7 | clippy | rc=0 |

★ load-bearing. **Row 3 is the one that matters**: one mutation cannot distinguish "threaded" from
"hardcoded to the other constant", and that ambiguity is how the defect survived.

## Trap doors, named in advance

- **A single-arm gate.** Checking only `insert-all` passes over the live defect today.
- **Asserting on the rendered message** rather than the structured `:op` — the message is prose and
  will drift; the field is the claim.
- **A runtime "what did the user write" lookup.** The entry point knows statically.
- **Fixing L2-1 or L2-3 in passing.** Both confirmed live, both out of scope, both need their own
  instrument first.
- **Re-running a red.** Capture whole, name the arm, surface it.

## What would make me reject the result

- One mutation instead of two.
- A gate that reads the message text.
- The `insert-all` arm's behaviour changed.
- L2-1 or L2-3 touched.
