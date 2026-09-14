#!/usr/bin/env bash
# Whole-tree `wat --check` census. BRIEF-4b D / BRIEF-1 step gate from #61.
#
#   scripts/replay/census.sh              → .census/<utc>.txt  (path rc, one line each)
#   scripts/replay/census.sh --diff PREV CURR
#       a file going rc 0 → non-zero that the step did not produce is STOP-8.
#       Optional third arg: the step's produced .wat paths, one per line; those
#       are exempt (the step owns their rc).
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
cd "$ROOT"
WAT="$ROOT/target/release/wat"
OUTDIR="$ROOT/.census"
mkdir -p "$OUTDIR"

if [[ "${1:-}" == "--diff" ]]; then
  PREV=${2:?usage: census.sh --diff PREV CURR [produced.txt]}
  CURR=${3:?}
  PROD=${4:-}
  declare -A OWN
  if [[ -n "$PROD" ]]; then
    while read -r p; do [[ -n "$p" ]] && OWN["$p"]=1; done < "$PROD"
  fi
  declare -A PRE
  while read -r p rc; do PRE["$p"]=$rc; done < "$PREV"
  stop=0
  while read -r p rc; do
    old=${PRE[$p]:-}
    if [[ "$old" == "0" && "$rc" != "0" ]]; then
      if [[ -z "${OWN[$p]:-}" ]]; then
        echo "STOP-8 $p rc $old -> $rc"
        stop=1
      fi
    fi
  done < "$CURR"
  if [[ $stop -eq 1 ]]; then
    exit 8
  fi
  echo "census-diff: no STOP-8"
  exit 0
fi

STAMP=$(date -u +%Y-%m-%dT%H-%M-%SZ)
OUT="$OUTDIR/$STAMP.txt"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
git ls-files '*.wat' | sort > "$TMP/all"
n=$(wc -l < "$TMP/all")
mkdir -p "$TMP/parts"
# One path per worker; -P32 as measured (2047 files, ~30s). Each worker
# writes its own file so parallel stdout cannot interleave a line.
# Capture wat's rc BEFORE sha256sum — `$?` after the hash is always 0.
# `-I{}` already implies one arg; `-n1` is mutually exclusive with `-I`.
cat "$TMP/all" | xargs -P32 -I{} bash -c '
  p="$1"
  "$0" --check "$p" >/dev/null 2>&1
  rc=$?
  h=$(printf "%s" "$p" | sha256sum | awk "{print \$1}")
  echo "$p $rc" > "$2/$h"
' "$WAT" {} "$TMP/parts"
cat "$TMP/parts"/* | sort > "$OUT"
echo "$OUT  files=$n"
ln -sfn "$STAMP.txt" "$OUTDIR/latest"
