#!/usr/bin/env bash
# check-query-compat.sh — query mouth, three ways.
#
#   Clara 0.24.0  |  wat-oracle (fire-rules$oracle)  |  wat-native (fire-rules)
#
# Same row lines from where-query-*.wat / where-fact-bind (query families).
# Empty three-way diff ⇒ binding maps agree. A hunk names the row AND the pair.
#
#   check-query-compat.sh                 # every query family
#   check-query-compat.sh where-query-compat
#
# JDK: PATH, else JAVA_HOME, else $HOME/opt/jdk-*/bin/java.
set -euo pipefail

GRID_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$GRID_DIR/../../.." && pwd)"
ONLY="${1:-}"

WAT_BIN="${WAT_BIN:-$REPO_ROOT/target/release/wat}"
[ -x "$WAT_BIN" ] || {
  echo "check-query-compat: no wat binary at $WAT_BIN — cargo build --release" >&2
  exit 1
}

find_java() {
  if command -v java >/dev/null 2>&1; then
    return 0
  fi
  if [ -n "${JAVA_HOME:-}" ] && [ -x "$JAVA_HOME/bin/java" ]; then
    export PATH="$JAVA_HOME/bin:$PATH"
    return 0
  fi
  local j
  for j in "$HOME"/opt/jdk-*/bin/java; do
    [ -x "$j" ] || continue
    export JAVA_HOME="$(cd "$(dirname "$j")/.." && pwd)"
    export PATH="$JAVA_HOME/bin:$PATH"
    return 0
  done
  echo "check-query-compat: no java (PATH, JAVA_HOME, or \$HOME/opt/jdk-*)" >&2
  return 1
}

find_java || exit 1

CLARA_DEP='{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}'
OUT_DIR="$(mktemp -d)"
trap 'rm -rf "$OUT_DIR"' EXIT

rewrite_to_spec() {
  # Production verb only. `(?!\$oracle)` so we never double-apply.
  # `(?!-)` so fire-rules-explain is untouched.
  perl -pe 's/:wat::rete::fire-rules(?!\$oracle)(?!-)/:wat::rete::fire-rules\$oracle/g' \
    < "$1" > "$2"
}

query_stems() {
  local wat stem
  for wat in "$GRID_DIR"/where-query-*.wat "$GRID_DIR"/where-fact-bind.wat; do
    [ -e "$wat" ] || continue
    stem="$(basename "$wat" .wat)"
    if [ -n "$ONLY" ] && [ "$stem" != "$ONLY" ]; then continue; fi
    printf '%s\n' "$stem"
  done
}

check_stem() {
  local stem="$1"
  local wat="$GRID_DIR/$stem.wat"
  local clj="$GRID_DIR/$stem.clj"
  local spec_src="$OUT_DIR/$stem.spec.wat"

  if [ ! -f "$clj" ]; then
    echo "[$stem] has NO .clj twin" >&2
    return 1
  fi

  rewrite_to_spec "$wat" "$spec_src"

  if ! "$WAT_BIN" "$wat" > "$OUT_DIR/$stem.native.txt" 2> "$OUT_DIR/$stem.native.err"; then
    echo "[$stem] native FAILED" >&2
    cat "$OUT_DIR/$stem.native.err" >&2
    return 1
  fi
  if ! "$WAT_BIN" "$spec_src" > "$OUT_DIR/$stem.spec.txt" 2> "$OUT_DIR/$stem.spec.err"; then
    echo "[$stem] oracle FAILED" >&2
    cat "$OUT_DIR/$stem.spec.err" >&2
    return 1
  fi
  if ! clojure -Sdeps "$CLARA_DEP" -M "$clj" \
        > "$OUT_DIR/$stem.clara.txt" 2> "$OUT_DIR/$stem.clara.err"; then
    echo "[$stem] Clara FAILED" >&2
    tail -30 "$OUT_DIR/$stem.clara.err" >&2
    return 1
  fi

  local nn ns nc
  nn=$(wc -l < "$OUT_DIR/$stem.native.txt")
  ns=$(wc -l < "$OUT_DIR/$stem.spec.txt")
  nc=$(wc -l < "$OUT_DIR/$stem.clara.txt")
  if [ "$nn" -lt 1 ]; then
    echo "[$stem] native emitted NO rows" >&2
    return 1
  fi
  if [ "$nn" -ne "$ns" ] || [ "$nn" -ne "$nc" ]; then
    echo "[$stem] row COUNT differs — native $nn, oracle $ns, Clara $nc" >&2
    return 1
  fi

  local ok=0
  if ! diff -u "$OUT_DIR/$stem.spec.txt" "$OUT_DIR/$stem.native.txt"; then
    echo "[$stem] DIVERGENCE — oracle vs native" >&2
    ok=1
  fi
  if ! diff -u "$OUT_DIR/$stem.clara.txt" "$OUT_DIR/$stem.native.txt"; then
    echo "[$stem] DIVERGENCE — Clara vs native" >&2
    ok=1
  fi
  if [ "$ok" -ne 0 ]; then
    return 1
  fi
  echo "[$stem] $nn/$nn — Clara == oracle == native"
  ROWS_TOTAL=$(( ROWS_TOTAL + nn ))
  return 0
}

FAILED=0
PAIRS=0
ROWS_TOTAL=0
while IFS= read -r stem; do
  PAIRS=$(( PAIRS + 1 ))
  check_stem "$stem" || FAILED=1
done < <(query_stems)

if [ "$PAIRS" -eq 0 ]; then
  echo "check-query-compat: matched NO families${ONLY:+ for '$ONLY'}" >&2
  exit 1
fi

if [ "$FAILED" -eq 0 ]; then
  echo "query-compat: $PAIRS family(ies), $ROWS_TOTAL rows — Clara == oracle == native"
else
  echo "query-compat: FAILURES above" >&2
  exit 1
fi
