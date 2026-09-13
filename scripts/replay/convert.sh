#!/usr/bin/env bash
# Write to stdout <path> at <rev>, converted through the recorded-migration chain
# by this tree's ./target/release/wat. Each codemod runs only if <path> is inside
# its ;; SCOPE:. Work happens in a temp dir; the tree is never written.
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <rev> <path>" >&2
  exit 2
fi
REV=$1
PATH_IN=$2

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
cd "$ROOT"
WAT="$ROOT/target/release/wat"
CHAIN="$ROOT/scripts/replay/chain-order.sh"
ORDER_BASE=de827fb4c
ORDER_MAIN=a3218644d

if [[ ! -x "$WAT" ]]; then
  echo "convert.sh: missing $WAT — cargo build --release first" >&2
  exit 2
fi

in_scope() {
  local codemod=$1
  local path=$2
  python3 - "$codemod" "$path" <<'PY'
import sys, fnmatch
codemod, path = sys.argv[1], sys.argv[2]
scope = None
with open(codemod) as f:
    for line in f:
        t = line.strip()
        if t.startswith(";;") and "SCOPE:" in t:
            rest = t.split("SCOPE:", 1)[1].strip()
            scope = rest
            break
if not scope:
    sys.exit(1)
for ent in scope.split():
    if ent == "corpus":
        if path.endswith(".wat") and not path.startswith("wat-scripts/fixes/"):
            sys.exit(0)
        continue
    if ent.endswith("/"):
        if path.startswith(ent):
            sys.exit(0)
        continue
    if fnmatch.fnmatch(path, ent):
        sys.exit(0)
sys.exit(1)
PY
}

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
WORK="$TMP/work.wat"
LOG="$TMP/log"

if ! git show "$REV:$PATH_IN" > "$WORK" 2>"$TMP/show.err"; then
  echo "convert.sh: git show $REV:$PATH_IN failed" >&2
  cat "$TMP/show.err" >&2
  exit 2
fi

mapfile -t STEPS < <("$CHAIN" "$ORDER_BASE" "$ORDER_MAIN")
: > "$LOG"
for cm in "${STEPS[@]}"; do
  if in_scope "$cm" "$PATH_IN"; then
    {
      echo "=== $cm ==="
      # one-param-spec reads TWO path vectors (context, then targets).
      if [[ "$(basename "$cm")" == "one-param-spec.wat" ]]; then
        printf '["%s"]\n["%s"]\n' "$WORK" "$WORK"
      else
        printf '["%s"]\n' "$WORK"
      fi | "$WAT" "$cm"
    } >> "$LOG" 2>&1 || {
      echo "convert.sh: $cm failed on $REV:$PATH_IN" >&2
      cat "$LOG" >&2
      exit 1
    }
  fi
done
cat "$WORK"
