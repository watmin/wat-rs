#!/usr/bin/env bash
# Derive the recorded-migration chain between <base> and <main>, in landing order.
# Files added under wat-scripts/fixes/ between those commits, oldest-first, that
# still exist at HEAD (a deleted identity-codemod is not a chain step).
# Then apply scripts/replay/chain-order.overrides unless --no-overrides.
set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "usage: $0 <base> <main> [--no-overrides]" >&2
  exit 2
fi
BASE=$1
MAIN=$2
NO_OVERRIDES=0
if [[ "${3:-}" == "--no-overrides" ]]; then
  NO_OVERRIDES=1
fi

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
cd "$ROOT"

derive() {
  local f ci
  git diff --name-status "$BASE" "$MAIN" -- wat-scripts/fixes/ \
    | awk '$1 == "A" { print $2 }' \
    | while IFS= read -r f; do
        # A deleted identity-codemod is not a chain step on this tree.
        if ! git cat-file -e "HEAD:$f" 2>/dev/null; then
          continue
        fi
        ci=$(git log --format=%ci --diff-filter=A -1 "$BASE".."$MAIN" -- "$f")
        printf '%s %s\n' "$ci" "$f"
      done \
    | sort \
    | awk '{ print $4 }'
}

mapfile -t ORDER < <(derive)

if [[ $NO_OVERRIDES -eq 0 && -f "$ROOT/scripts/replay/chain-order.overrides" ]]; then
  while IFS= read -r line || [[ -n "$line" ]]; do
    line=${line%%$'\r'}
    [[ -z "$line" || "$line" == \#* ]] && continue
    if [[ "$line" =~ ^MOVE[[:space:]]+([^[:space:]]+)[[:space:]]+BEFORE[[:space:]]+([^[:space:]]+)[[:space:]]*$ ]]; then
      move=${BASH_REMATCH[1]}
      before=${BASH_REMATCH[2]}
      new=()
      skipped=0
      inserted=0
      for f in "${ORDER[@]}"; do
        if [[ "$f" == "$move" ]]; then
          skipped=1
          continue
        fi
        if [[ "$f" == "$before" && $inserted -eq 0 ]]; then
          new+=("$move")
          inserted=1
        fi
        new+=("$f")
      done
      if [[ $inserted -eq 0 && $skipped -eq 1 ]]; then
        new+=("$move")
      fi
      ORDER=("${new[@]}")
    else
      echo "chain-order.sh: unparseable override: $line" >&2
      exit 2
    fi
  done < "$ROOT/scripts/replay/chain-order.overrides"
fi

printf '%s\n' "${ORDER[@]}"
