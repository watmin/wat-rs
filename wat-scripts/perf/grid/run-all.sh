#!/usr/bin/env bash
# wat-scripts/perf/grid/run-all.sh — sweep EVERY axis × its size ladder → the whole verdict grid.
#
# The DESIGN (docs/arc/2026/06/278-rules-engine/DESIGN-clara-grid.md:31) called for this script and
# it was never written, so every recorded grid was a set of ratios whose SIZES lived only in the
# session that produced them. The 2026-07-31 seam's "20 of 22 :us" table is un-reproducible for
# exactly that reason: it lists three ratios per axis and names the size for one axis only.
#
# A benchmark number whose input is not written down is not a measurement, it is an anecdote.
# THE LADDER BELOW IS THE ARTIFACT. Change it deliberately, never casually — a grid run at
# different sizes is not comparable to the one before it, and the ratios will silently disagree.
#
# Each ladder's TOP rung is the exemplar size documented in that axis's own `<axis>.wat` header
# ("Usage (stdin = an i64 vector …): => #grid/Result {… :size [N M] …}"), with two smaller rungs
# below it on that axis's documented free scale dial. The dial is named per axis below.
#
# Usage:  bash wat-scripts/perf/grid/run-all.sh            # every axis
#         bash wat-scripts/perf/grid/run-all.sh accum negation   # only the named axes
#
# Env:    GRID_RUNS=3  (honoured by run-axis.sh — a verdict needs EVERY run to agree, else
#                       :unresolved; see run-axis.sh)
#
# Emits every `#grid/Verdict` line from every axis, then a summary tally on stderr.
set -euo pipefail

GRID_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ── THE LADDER ───────────────────────────────────────────────────────────────────────────────
# axis        : dial swept (the other params are held fixed)   : the three rungs
declare -A LADDER=(
  [min-finding]="500 3|1000 3|2000 3"        # dial: stations   (threshold fixed at 3)
  [negation]="250|500|1000"                  # dial: items
  [asym-join]="500|1000|2000"                # dial: items
  [strat-neg]="6 500|6 1000|6 2000"          # dial: items      (strata fixed at 6 — chain depth)
  [user-reduce]="10 25|20 50|40 100"         # dial: locs × reads together
  [node-share]="10 200|25 200|50 200"        # dial: rules      (items fixed at 200)
  [accum]="50 200|100 200|200 200"           # dial: groups     (readings fixed at 200)
)

# Deterministic order — the table reads the same every run.
ORDER=(min-finding negation asym-join strat-neg user-reduce node-share accum)

AXES=("$@")
if [ ${#AXES[@]} -eq 0 ]; then AXES=("${ORDER[@]}"); fi

rc=0
for axis in "${AXES[@]}"; do
  spec="${LADDER[$axis]:-}"
  if [ -z "$spec" ]; then
    echo "run-all: unknown axis '$axis' — known: ${ORDER[*]}" >&2
    exit 2
  fi
  # Split the |-separated rungs into positional SIZE args for run-axis.sh.
  IFS='|' read -r -a rungs <<< "$spec"
  echo "── $axis ──────────────────────────────────────────────" >&2
  # A failing axis must not abort the sweep — the whole point is the WHOLE grid, and a broken
  # axis is itself a finding. Record it and carry on.
  if ! bash "$GRID_DIR/run-axis.sh" "$axis" "${rungs[@]}"; then
    echo "run-all: axis '$axis' FAILED (rc=$?) — see its stderr above" >&2
    rc=1
  fi
done

exit $rc
