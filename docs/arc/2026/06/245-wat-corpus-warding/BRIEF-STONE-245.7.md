# BRIEF — Stone 245.7: the leak-contained integration runner

## The work (one paragraph)

Author `scripts/integration-run.sh`: a per-binary integration-test runner for the
wat crate whose defining property is **containment** — each test binary runs in
its own session (`setsid`) under a `timeout`, and the runner reaps the entire
session after each binary (`pkill -s`), so leaked child processes structurally
cannot escape and a hang costs one binary's timeout, never the run. It produces a
TSV failure inventory (the triage's input) plus an error-class histogram. Then RUN
it (default tier) and deliver the inventory + the leak-safety proof.

## The contract (pinned)

Containment over classification. The mechanic is PROVEN (orchestrator probe):
```
setsid bash -c 'sleep 300 & exit' &  →  orphan survives in session <leader-pid>
pkill -s <leader-pid>                →  session empty
```
Your loop per binary `<name>`:
```bash
setsid timeout "$TIMEOUT" cargo test --release -p wat --test "$name" > "$cap" 2>&1 &
sid=$!
wait "$sid"; code=$?
pkill -s "$sid" 2>/dev/null || true     # ALWAYS — success, fail, or timeout
```
(`timeout` exit 124 = the binary timed out. The capture file is parsed afterward.)

## Read first (the rooms)

1. `docs/arc/2026/06/245-wat-corpus-warding/DESIGN-STONE-245.7-integration-runner.md` — the full design; mirror it exactly.
2. `scripts/green-gate.sh` — the sibling script; mirror its header style (the
   why-this-exists discipline, the usage block).
3. `Cargo.toml` lines ~88-100 — the `[[test]]` module-group names (part of enumeration).

## The script, precisely

`scripts/integration-run.sh [--all] [--timeout SECS] [--out FILE]`

- **Enumerate**: basenames of `tests/*.rs` (strip `.rs`) + each `[[test]]` `name =`
  from `Cargo.toml`. Sort for determinism.
- **Tier** (default): exclude any binary whose source file matches
  `grep -lE 'spawn_process|run_hermetic|fork|pidfd|lifeline|ambient'` (for module
  groups, grep the group's directory). Document IN the script that this is a
  HEURISTIC for the arc-170 process class and containment backstops it. `--all`
  skips the exclusion.
- **Build once first**: `cargo build --release --tests -p wat` (fail the script if
  the build fails — that is the green-gate's job, but a broken build makes every
  binary "fail" misleadingly).
- **Run loop**: the contract block above. Default `--timeout 60`.
- **Inventory line per binary** (TSV):
  `name<TAB>pass|fail|timeout<TAB>P/F/I<TAB>comma-joined failing test names (empty if none)`
  parsed from the capture (`test result: ... N passed; M failed; K ignored`; the
  `failures:` block for names). `timeout` rows: status `timeout`, counts blank.
- **Footer** (after a `#` comment marker): totals (binaries run/passed/failed/
  timed-out; tests P/F/I summed) and the error-class histogram —
  `grep -hoE 'NoMatchingClause|UnresolvedReference|MalformedForm|TypeMismatch|UnboundSymbol'`
  across all captures, counted.
- **Default out**: `target/integration-inventory.tsv`. Print the footer to stdout
  too.
- **Exit code**: 0 iff all run binaries passed; else 1. (It will exit 1 today —
  the tier is red; the deliverable is the inventory.)
- `set -euo pipefail` BUT the per-binary loop must tolerate individual failures
  (capture the exit code; do not let one red binary abort the loop).

## Then RUN it and capture the proof

1. **Before**: `pgrep -c -f 'target/release/deps' || true` (and note any
   pre-existing strays).
2. `./scripts/integration-run.sh` (default tier, default timeout).
3. **After**: the same pgrep — the delta must be ZERO (the leak-safety proof).
   Also `pgrep -f 'sleep|wat-test'` sanity.
4. Report the inventory footer verbatim + attach the path.

## Boundaries

- New file `scripts/integration-run.sh` (mode 755) ONLY. No edits to
  `green-gate.sh`, no src/ or tests/ changes, no Cargo.toml changes.
- Do NOT run `cargo test --workspace` or any un-contained integration run.
- Do NOT run any git command; do NOT commit — the orchestrator scores and commits.

## STOP triggers (rejection criteria — surface, do not improvise)

- **STOP-1**: the after-run process delta is NOT zero (something escaped the
  session reap) — report what survived and which binary it came from; do not
  pkill it globally yourself beyond the session mechanism; the orchestrator must
  see the escape.
- **STOP-2**: more than ~25% of the default tier hits the timeout — the heuristic
  tier is wrong (leaky binaries inside the default tier en masse); report the
  timeout list rather than burning an hour.
- **STOP-3**: a binary name can't be run via `--test <name>` (enumeration
  mismatch) — report it; do not silently drop it from the tier.

## Verify (the load-bearing checks)

```
bash -n scripts/integration-run.sh          # syntax
./scripts/integration-run.sh                # the real run (red exit expected)
# leak proof: process snapshots before/after as above
# completeness: inventory line count == enumerated tier count
```

## Comparable prior result (copy for shape)

`scripts/green-gate.sh` — the sibling gate script (arc 239): same header
discipline, same honest why-this-exists commentary, same role in the routine.
This stone is its run-tier counterpart for the integration tier.
