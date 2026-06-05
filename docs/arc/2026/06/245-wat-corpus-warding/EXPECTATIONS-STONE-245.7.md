# EXPECTATIONS — Stone 245.7: the leak-contained integration runner

Written BEFORE the strike. The Score grades against THIS, re-run independently by
the orchestrator.

## Scorecard

| # | What | Command | Expected |
|---|---|---|---|
| 1 | Leak-safety (THE contract) | process snapshot before vs after the full default-tier run | delta = ZERO surviving processes |
| 2 | Completeness | inventory line count vs enumerated tier count | equal — every tier binary has a row (timeouts as `timeout` rows, not gaps) |
| 3 | The baseline inventory exists | `target/integration-inventory.tsv` + footer | per-binary TSV + totals + error-class histogram |
| 4 | Containment loop is the proven mechanic | read the script | `setsid timeout … & wait; pkill -s` per binary, reap on ALL paths |
| 5 | Tier heuristic documented | read the script header | the leaky-signal regex named AS a heuristic; `--all` documented |
| 6 | Bounded blast radius | `git status --short` | ONLY `scripts/integration-run.sh` (new, executable) |
| 7 | Sibling discipline | compare to `green-gate.sh` | header style: usage + why-this-exists + the arc-239-style honesty |

Rows 1–3 load-bearing (orchestrator re-runs the runner himself — the leak proof
must reproduce under MY snapshot, not the executor's say-so). 4–7 are read-checks.

## Runtime prediction

20–35 min total (Mode A): script authoring ~10 min; the default-tier run = ~187
binaries × (0.3–2s each) ≈ 5–15 min with a warm build; parsing + report the rest.
The exit code will be 1 (the tier is red today) — that is EXPECTED and not a
failure of the stone.

## Trap-doors named

- **The reap must run on every path** — a `set -e` abort between `wait` and
  `pkill -s` leaks the very thing the script exists to contain. The loop must
  capture exit codes, never abort mid-binary.
- **`--test <name>` vs harness quirks** — `tests/test.rs` (the wat-corpus
  aggregate) is a normal harness binary and belongs in the tier; the 4
  `[[test]]` module groups run by their `name =`. STOP-3 catches mismatches.
- **The timeout double-bind** — `timeout` must kill the *session leader*; if a
  child ignores TERM the follow-up `pkill -s` is the backstop. If many binaries
  time out (STOP-2 >25%), the tier heuristic is wrong — stop, don't grind.
- **Pre-existing strays** — the before-snapshot may already contain leaked procs
  from earlier work; the contract is the DELTA, not an empty machine.

## What this stone unlocks (not in scope)

The fresh inventory replaces the stale "~190" estimate and becomes the triage's
work-list: conferre each failing binary (real substrate gap = fill / stale test =
modernize-or-delete). When the tier greens, the runner's invocation folds into
`green-gate.sh` and the integration tier can never rot silently again — the
campaign's #151 endgame.
