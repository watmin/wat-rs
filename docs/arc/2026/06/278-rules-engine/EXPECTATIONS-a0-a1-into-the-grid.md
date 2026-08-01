# EXPECTATIONS — A0/A1 into the grid (written BEFORE the strike)

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | A0 runs | `run-axis.sh deep-cascade "10 5"` | one `#grid/Verdict`, `:accuracy :match` |
| 2 | A1 runs | `run-axis.sh fanout "10000"` | one `#grid/Verdict`, `:accuracy :match` |
| 3 | the sweep is nine | read `LADDER` + `ORDER` in `run-all.sh` | nine entries each, `deep-cascade` + `fanout` among them — verified STATICALLY; executing the no-arg sweep is the orchestrator's weigh (see trap-doors) |
| 3b | each new axis climbs | `run-all.sh deep-cascade`, `run-all.sh fanout` | three rungs each, under the memory guard, climbed upward |
| 4 | the new wat loads | `-E 'test(every_wat_scripts_file_loads)'` | green |
| 5 | floor untouched | `cargo nextest run --release` | unchanged — this stone touches no `src/`, no `wat/` |
| 6 | `:derived` is real | inspect a `#grid/Result` at two rungs | a non-empty SORTED VECTOR whose length DIFFERS between rungs |

## Independent prediction

- **Runtime:** 40–70 min. The logic exists; the work is contract conformance and inventing two
  injective `:derived` encodings.
- **A0:** expect `:us`. R4 had it ours at every cell (1.2×–6.3×) and three stones have landed since.
- **A1 at 40,000:** **genuinely unknown, and that is why we are running it.** R4 recorded `Clara 1.4×`
  and attributed it to per-token support-chain provenance we carry deliberately. A `:clara` result is
  a legitimate, expected-possible outcome — **do not read it as the rider failing.**

## Trap-doors named in advance

- **Row 6 is the one that matters.** A `:derived` that is constant across rungs means the size dial
  is not reaching the rules, and rows 1–3 would still be green — a beautifully conformant axis
  measuring nothing. This is the same vacuous-gate shape as everything else this arc.
- **A `:MISMATCH` is a finding, not a bug.** The temptation is to adjust the encoding until the two
  sides agree. That would be encoding a real engine divergence away — the single worst outcome
  available here, worse than not doing the stone.
- **A speed-only axis is worse than no axis.** `run-all.sh` tallies it, and "N of N" then overstates
  coverage. STOP-1 exists for exactly this.
- **Do not re-run the whole grid as part of the strike** — that is the orchestrator's weigh. Row 3 was
  originally written as "`run-all.sh` (no args)", which *is* the whole grid — the row contradicted this
  trap-door and has been corrected to a static check (2026-07-31, caught pre-send).
- **Every grid run goes under the memory guard** (`systemd-run --user --scope -p MemoryMax=8G
  -p MemorySwapMax=0 -- timeout 600 …`), ladder climbed UPWARD. A1 @ 40,000 with a JVM beside us is the
  largest unvalidated thing in this stone; an uncapped first run at the top rung is how a workstation
  dies.
- **Timings here are provisional.** The box is not guaranteed quiet; accuracy is the deliverable and the
  orchestrator re-weighs the ratios. A rider chasing a ratio is chasing an instrument artifact.
- **Nothing here should move a perf number on A2–A8.** If it does, something touched the engine.

## What would make me reject the strike outright

`:derived` landed as a count; a `:MISMATCH` silently encoded away; any edit under `src/` or `wat/`; or
A1's ladder topping out below 40,000.
