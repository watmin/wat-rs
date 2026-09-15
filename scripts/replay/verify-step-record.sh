#!/usr/bin/env bash
# scripts/replay/verify-step-record.sh <from-rev> [<to-rev>] — finding 22's gate on the RECORD.
#
# For every `REPLAY(grok-rete #N)` commit in <from-rev>..<to-rev> (default HEAD), derive what the step
# gate required FROM THE COMMIT'S OWN DIFF, and check its body records each wall's verdict line:
#   binary sources (src/ crates/ Cargo.* build.rs) or any .wat changed  → `census: … --diff no STOP-8`
#                                                                         `nested-program-gate: PASS`
#   any .rs changed                                                     → `lint-subset: <n> passed`
#                                                                         `kind(lib): <n> passed`
#                                                                         `doctest: <n> passed`
# A docs-only step requires nothing. Prints one MISSING line per absent verdict; exit 1 if any.
# It proves the record exists, not that the walls passed — the orchestrator's spot re-runs do that.
set -euo pipefail
FROM=${1:?usage: verify-step-record.sh <from-rev> [<to-rev>]}
TO=${2:-HEAD}
missing=0
while IFS=$'\t' read -r h s; do
  n=$(printf '%s' "$s" | sed -E 's/.*#([0-9]+)\).*/\1/')
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
[ "$missing" -eq 0 ] && echo "step-record: complete"
exit "$missing"
