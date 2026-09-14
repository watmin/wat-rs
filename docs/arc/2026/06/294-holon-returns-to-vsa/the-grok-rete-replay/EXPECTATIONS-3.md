# EXPECTATIONS 3 — every spawned program starts (written before the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended.

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | the gate discriminates | the gate on a temp fixture of `f2e0ac26b^:wat-scripts/probes/arc-170/probe-m1-ann-erase.wat` | RED, naming the file, line `34`, and the startup error (`…CMsg… is not namespaced`) |
| E2 | the gate is green on the tree | `cargo nextest run --release -E 'test(<the gate>)'` | PASS |
| E3 | its count reconciles to the census | the gate's printed tally | 122 checked (121 `spawn-peer` + 1 `Locus/launch`), 7 assembled, 5 templates, 7 data = 141 — or the difference explained by a literal the stone itself added |
| E4 | the positions are derived | read the gate: no list of verb names, only the root position and the forwarding rule | no hand list |
| E5 | every pinned failure names a live test | the gate refuses a rune whose test does not exist: sabotage one rune's test name | RED |
| E6 | the 7 start | the gate's rows for the seven; each probe run | each child STARTS; each probe reaches its stated outcome |
| E7 | the migrations are recorded | `tests/cli/every_recorded_migration_replays.rs`; each new `wat-scripts/fixes/*.wat` has a fixture reaching a nested `(forms …)` child | GREEN; idempotent (second run: 0 changes) |
| E8 | no hand edit | the seven probes' diffs | produced by the new migrations only |
| E9 | D3 | `arc112_scheme_probe.wat:12` | classified in the SCORE; fixed if a defect |
| E10 | the floor | `scripts/floor.sh` + clippy; the gate's added wall time | green; clippy 0; the cost reported |

**Runtime prediction:** 2–3 h (the gate with its derivation, two recorded migrations with fixtures,
one new deftest, D3).

**Trap doors:**
- the derivation following a `let` binding (the only route for `Locus/launch`) — the likely STOP-1;
- a wrap codemod reaching INTO a `(forms …)` child rather than the parent — STOP-3;
- the UselessMain four: the gate must use the child path, not `--check`, or it refuses children
  production starts (`tests/kernel/wat_run_sandboxed.rs:67` asserts one closes `"closed"`).
