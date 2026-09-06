# EXPECTATIONS — census G

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | both gone | `grep -rn 'prod:vec-alloc\|prod:record-alloc' src/ tests/` → **no hits**, comments included |
| 2 ★ | floor GREEN | the evidence that nothing read them; quote the Summary line |
| 3 ★ | no NAMES list moved | no exact-equality list edited; if one was, STOP-1 |
| 4 ★ | the reason is recorded | at the deletion site: constant not measurement, zero readers, real-allocation count rejected, and what still covers the path |
| 5 | siblings kept | `prod:class-alloc` and `prod:derivations` untouched, and the SCORE says why `class-alloc` stays |
| 6 | no replacement counter | nothing added under any name |
| 7 | floor count | **≥ 5465 — final number and what moved it.** New arms are a PASS; name them |
| 8 | clippy | rc=0 |

★ load-bearing. **Row 2 is the proof, such as it is.** No mutation is available — a counter nothing
reads cannot red when deleted, and inventing a reader to produce one would be the defect this
strike is removing.

## Trap doors, named in advance

- **Replacing them with an honest counter.** Still unread, still nobody served — it converts a
  false row into a true useless one. Rejected.
- **Making them count real allocations.** Census F's rejection, same reasoning.
- **Deleting `prod:class-alloc` for symmetry.** It is TRUE and merely unread; the line is
  falsehood, not unreadness (`[[a-guard-that-never-fires-is-not-dead-code]]`).
- **Silently editing a NAMES list to make the floor green.** That is STOP-1 wearing a fix.
- **Re-running a red.** Capture whole, name the arm, surface it.

## What would make me reject the result

- Either name still present anywhere in code or comments.
- A replacement counter under any name.
- A NAMES list quietly edited.
- A fabricated mutation proof.
