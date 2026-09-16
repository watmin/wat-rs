#!/usr/bin/env bash
# scripts/replay/verify-step-record.sh <from-rev> [<to-rev>] [<first-N> <last-N>] — the gate on the RECORD.
#
# For every `REPLAY(grok-rete #N)` commit in <from-rev>..<to-rev> (default HEAD), derive what the step
# gate required FROM THE COMMIT'S OWN DIFF, and check its body records each wall's verdict line:
#   binary sources (src/ crates/ Cargo.* build.rs) or any .wat changed  → `census: … --diff no STOP-8`
#                                                                         `nested-program-gate: PASS`
#   any .rs changed                                                     → `lint-subset: <n> passed`
#                                                                         `kind(lib): <n> passed`
#                                                                         `doctest: <n> passed`
# A docs-only step requires nothing. Prints one MISSING line per absent verdict; exit 1 if any.
# It proves the record EXISTS, not that the walls passed — the orchestrator's spot re-runs do that.
#
# ⛔ FINDING 26 — WITHOUT <first-N> <last-N> THIS GATE CANNOT SEE A MIS-SUBJECTED STEP. It selects its
# subjects by grepping for `REPLAY(grok-rete #`, so a step committed under another subject (e.g. a bare
# `git cherry-pick -x`, which keeps grok's own subject) is not reported missing — IT DOES NOT EXIST TO THE
# GATE, and `step-record: complete` is printed over a batch with unexamined steps. That happened at #160
# and #161. Pass the batch's expected step range and every N is required to be present EXACTLY ONCE, with
# its cited source SHA cross-checked against commits.tsv — so a deviation goes RED BY ABSENCE.
# A contiguity check would NOT suffice: #160/#161 were the range's FIRST steps, so the surviving set
# 162…185 reads as perfectly contiguous. The expected range must come from outside the commits.
set -euo pipefail
FROM=${1:?usage: verify-step-record.sh <from-rev> [<to-rev>] [<first-N> <last-N>]}
TO=${2:-HEAD}
FIRST_N=${3:-}
LAST_N=${4:-}
missing=0
declare -A seen=()
while IFS=$'\t' read -r h s; do
  n=$(printf '%s' "$s" | sed -E 's/.*#([0-9]+)\).*/\1/')
  if [ -n "${seen[$n]:-}" ]; then
    echo "DUPLICATE #$n: ${seen[$n]} and $h"
    missing=1
  fi
  seen[$n]=$h
  files=$(git show --name-only --format= "$h")
  body=$(git show -s --format=%B "$h")
  need=()
  if printf '%s\n' "$files" | grep -qE '\.wat$|^src/|^crates/|^Cargo\.|build\.rs$'; then
    need+=('census: .*--diff no STOP-8' 'nested-program-gate: PASS')
  fi
  if printf '%s\n' "$files" | grep -qE '\.rs$'; then
    need+=('lint-subset: [0-9]+ passed' 'kind\(lib\): [0-9]+ passed' 'doctest: [0-9]+ passed')
  fi
  for pat in "${need[@]}"; do
    if ! printf '%s\n' "$body" | grep -qE "$pat"; then
      echo "MISSING #$n $h: $pat"
      missing=1
    fi
  done
done < <(git log --reverse --format='%H%x09%s' "$FROM..$TO" | grep 'REPLAY(grok-rete #')

# The range assertion — finding 26's cure. Optional so old call sites keep working, but a batch that
# does not pass it has NOT been checked for mis-subjected or skipped steps.
if [ -n "$FIRST_N" ] && [ -n "$LAST_N" ]; then
  tsv="$(dirname "$0")/../../bootstrap/era/replay-plan/commits.tsv"
  for ((n = FIRST_N; n <= LAST_N; n++)); do
    h=${seen[$n]:-}
    if [ -z "$h" ]; then
      echo "MISSING-STEP #$n: no REPLAY(grok-rete #$n) commit in $FROM..$TO"
      missing=1
      continue
    fi
    if [ -r "$tsv" ]; then
      src=$(awk -F'\t' -v n="$n" '$1 == n { print $2 }' "$tsv")
      if [ -n "$src" ] && ! git show -s --format=%B "$h" | grep -q "cherry picked from commit $src"; then
        echo "WRONG-SOURCE #$n $h: body does not cite commits.tsv's source $src"
        missing=1
      fi
    fi
  done
  [ "$missing" -eq 0 ] && echo "step-range: #$FIRST_N..#$LAST_N each present exactly once, sources match"
fi

[ "$missing" -eq 0 ] && echo "step-record: complete"
exit "$missing"
