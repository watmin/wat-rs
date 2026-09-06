# EXPECTATIONS — the retract-multiplicity proof axis

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | the axis stages a genuine DUPLICATE | the same fact twice, retracted once |
| 2 ★ | neither side collapses multiplicity | no `set`/`distinct`; sorted, not deduped |
| 3 ★ | verdict recorded verbatim | `:accuracy :MISMATCH :oracle-accuracy :MISMATCH :port-accuracy :match` |
| 4 ★ | floor GREEN | a recorded MISMATCH is data, not a failure |
| 5 | both size tables updated and identical | `SIZES` + `CORRECTNESS_SIZES` |
| 6 | the `.wat` gates run | loads + rete-name resolution |
| 7 | clippy | rc=0 |

★ load-bearing. **Row 3 is the proof; row 4 is what makes it landable.**

## Trap doors, named in advance

- **Collapsing multiplicity on either side.** The existing Clara fixtures all do `(count (set …))`;
  copying that shape reproduces the blindness. STOP-2.
- **A red floor.** The proof is a grid verdict, not a test assertion. STOP-3.
- **Forgetting `CORRECTNESS_SIZES`** — the floor mirrors the script's table and will redden.
- **Drifting toward the perf ladder.** This is a correctness axis; the header forbids the two size
  tables converging.
- **Re-run the floor at FINAL state.**

## What would make me reject the result

- A verdict claimed without the raw `#grid/Verdict` line quoted.
- Either side set-collapsed.
- A red floor.
- The cure attempted here.
