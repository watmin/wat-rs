# EXPECTATIONS — census M

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | the key moved, the call kept | both sites `census_count("bench:filter-reuse")`; the calls still there |
| 2 ★ | arm timings before/after | J and K quoted both ways. **Either result is a PASS** — a material move is a finding, not a failure |
| 3 ★ | the gate exists and is empty-exemption | `census_count` under `src/rete/kernel/tests/` may name only a `bench:` key |
| 4 ★ | the gate is mutation-proved live | restore a production key at one site → RED naming file and key; quote; restore |
| 5 | the bumps survive | not deleted — they carry the reconstruction's fidelity |
| 6 | no production code touched | `fire/mod.rs` and the census window unchanged |
| 7 | floor | **≥ 5465 — final number and what moved it.** The new lint's tests are a PASS; name them |
| 8 | clippy | rc=0 |

★ load-bearing. **Row 4 is the point**: the rename fixes the instance, the gate removes the
situation.

## Trap doors, named in advance

- **Deleting the bumps** because they look like stray instrumentation. They mirror
  `fire/mod.rs:2286` and the reconstruction under-counts without them.
- **Assuming the rename is free.** It is a per-call hash on a timed path; row 2 requires numbers.
- **A gate that trips on `census_counted(||`.** That is the window helper, not a write.
- **Widening the census window** to "make the separation explicit". The window is not the defect.
- **Re-running a red.** Capture whole, name the arm, surface it.

## What would make me reject the result

- The bumps deleted.
- Arm timings claimed unchanged without numbers.
- A gate with a non-empty exemption list.
- The mutation skipped or driven against a copy.
