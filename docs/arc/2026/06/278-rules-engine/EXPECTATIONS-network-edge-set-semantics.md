# EXPECTATIONS — network edges are a SET

Written BEFORE the strike so the result cannot move the goalposts.
Brief: `BRIEF-network-edge-set-semantics.md`. Design: `DESIGN-STONE-network-edge-set-semantics.md`.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the substrate still builds | `cargo build --release --all-targets` | exit 0, zero warnings |
| 2 | **the load-bearing row** — A8 fire-time sharing | `cargo nextest run --release -E 'test(a8_node_share_fire_census)' --no-capture` | PASS (RED at HEAD) |
| 3 | edges collapse to the node shape | the gate's `edges` column | flat-ish in N, NOT `5N` |
| 4 | RootJoin tokens flat in N | the gate's `RootJoin` column | 50 at every N (was `50·N`) |
| 5 | HashJoin tokens flat in N | the gate's `HashJoin` column | 50 at every N (was `50·N³`) |
| 6 | **results do not move** | the gate's `derived` column | 50 at every N, unchanged |
| 7 | stdlib load order intact | `(:wat::deporder::verify-stdlib)` | `[]` |
| 8 | the floor | `cargo nextest run --release` (orchestrator, central) | the A8 gate flips to pass; **no other test's result changes** |

Row 6 is the one that matters most for trust: this is a *cost* fix. If the derived set moves, the
change is wrong even if everything else is green.

Row 8 is mine, not the rider's — riders get build-only + one narrow filtered gate.

## Independent prediction

**Runtime: 10–20 minutes.** One function body, an exemplar to copy in the same file, and two gates.
Predicted mode: one-shot green.

The prediction is high-confidence for an unusual reason: the mechanism was measured rather than
theorised, the primitive is proven on this exact data shape two hundred lines away
(`wat/rete.wat:997`), and the idempotent-conj idiom already exists in the file
(`wat/rete.wat:1478`). There is very little for the rider to invent.

## Trap-doors named in advance

- **A test that encoded the duplication.** Somewhere a probe may assert a token or node-children
  count that only held because edges were duplicated. It would surface as a floor failure at row 8,
  not in the rider's gates. This is STOP-4 in the brief: report, do not adapt the assertion. If it
  happens, it is a *second* finding of the same class as the vacuous-gate sweep (`91bbb8cd`) — an
  assertion certifying a defect.
- **A second duplication source.** If rows 4/5 improve but do not go flat, the design found one
  source and missed another. That is STOP-2, and the census table names where it lives.
- **The oracle moves too.** `compile` is shared, so the wat oracle's networks change identically.
  That is intended — but it means any oracle-vs-native differential is NOT a control here: both
  sides move together. Row 6 (`derived` unchanged) is the real control.
- **Ordering.** Idempotent insert into an ordered `PersistentVector` preserves order by
  construction, so fire order is unchanged for the non-duplicate edges. If any ordering-sensitive
  assertion moves, that is a signal the change did more than dedup — investigate, do not re-baseline.

## What this does NOT claim

The stone proves the *mechanism* is fixed. It does not re-measure A8 against Clara — that is a
separate run, and it must ride the memory guard in `wat-scripts/perf/grid/run-axis.sh`, climbing a
size ladder upward. A workstation was lost learning that; the number can wait for a deliberate
measurement.
