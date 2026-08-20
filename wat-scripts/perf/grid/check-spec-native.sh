#!/usr/bin/env bash
# check-spec-native.sh — wat-oracle (`fire-rules$oracle`) vs wat-native (`fire-rules`).
#
# Same corpus as check-where-shapes.sh (every where-*.wat). Same row lines.
# The .wat files call fire-rules. This script also runs a rewrite that calls
# fire-rules$oracle and diffs the two stdout streams byte-for-byte.
#
# Empty diff  ⇒  spec == native on every row of that family.
# A hunk      ⇒  it NAMES the row.
#
#   check-spec-native.sh              # every family
#   check-spec-native.sh where-exists # one stem
set -euo pipefail

GRID_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$GRID_DIR/../../.." && pwd)"
ONLY="${1:-}"

WAT_BIN="${WAT_BIN:-$REPO_ROOT/target/release/wat}"
[ -x "$WAT_BIN" ] || {
  echo "check-spec-native: no wat binary at $WAT_BIN — cargo build --release" >&2
  exit 1
}

OUT_DIR="$(mktemp -d)"
trap 'rm -rf "$OUT_DIR"' EXIT

rewrite_to_spec() {
  local src="$1"
  local dst="$2"
  # Production verb only. `(?!\$oracle)` so we never double-apply.
  # `(?!-)` so fire-rules-explain is untouched.
  perl -pe 's/:wat::rete::fire-rules(?!\$oracle)(?!-)/:wat::rete::fire-rules\$oracle/g' \
    < "$src" > "$dst"
}

check_stem() {
  local stem="$1"
  local wat="$GRID_DIR/$stem.wat"
  local spec_src="$OUT_DIR/$stem.spec.wat"

  rewrite_to_spec "$wat" "$spec_src"

  if ! "$WAT_BIN" "$wat" > "$OUT_DIR/$stem.native.txt" 2> "$OUT_DIR/$stem.native.err"; then
    echo "[$stem] native FAILED" >&2
    cat "$OUT_DIR/$stem.native.err" >&2
    return 1
  fi
  if ! "$WAT_BIN" "$spec_src" > "$OUT_DIR/$stem.spec.txt" 2> "$OUT_DIR/$stem.spec.err"; then
    echo "[$stem] spec FAILED" >&2
    cat "$OUT_DIR/$stem.spec.err" >&2
    return 1
  fi

  local wn sn
  wn=$(wc -l < "$OUT_DIR/$stem.native.txt")
  sn=$(wc -l < "$OUT_DIR/$stem.spec.txt")
  if [ "$wn" -lt 1 ]; then
    echo "[$stem] native emitted NO rows" >&2
    return 1
  fi
  if [ "$wn" -ne "$sn" ]; then
    echo "[$stem] row COUNT differs — native $wn, spec $sn" >&2
    diff -u "$OUT_DIR/$stem.spec.txt" "$OUT_DIR/$stem.native.txt" >&2 || true
    return 1
  fi
  if diff -u "$OUT_DIR/$stem.spec.txt" "$OUT_DIR/$stem.native.txt"; then
    echo "[$stem] $wn/$wn rows — spec == native"
    ROWS_TOTAL=$(( ROWS_TOTAL + wn ))
    return 0
  fi
  echo "[$stem] DIVERGENCE — spec vs native (hunk above names the row)" >&2
  return 1
}

FAILED=0
PAIRS=0
ROWS_TOTAL=0
for wat in "$GRID_DIR"/where-*.wat; do
  [ -e "$wat" ] || continue
  stem="$(basename "$wat" .wat)"
  if [ -n "$ONLY" ] && [ "$stem" != "$ONLY" ]; then continue; fi
  PAIRS=$(( PAIRS + 1 ))
  check_stem "$stem" || FAILED=1
done

if [ "$PAIRS" -eq 0 ]; then
  echo "check-spec-native: matched NO families${ONLY:+ for '$ONLY'}" >&2
  exit 1
fi

if [ "$FAILED" -eq 0 ]; then
  echo "spec-native: $PAIRS family(ies), $ROWS_TOTAL rows — spec == native on every shape"
else
  echo "spec-native: FAILURES above" >&2
  exit 1
fi
