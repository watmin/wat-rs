#!/usr/bin/env bash
# check-where-shapes.sh — THE `where`-EXPRESSIVITY VERDICT, and it is one diff.
#
# Runs both halves of the corpus once each and compares them byte-for-byte:
#
#   wat-scripts/perf/grid/where-shapes.wat   (wat   — every row, one process)
#   wat-scripts/perf/grid/where-shapes.clj   (Clara — every row, one JVM)
#
# Empty diff  ⇒  every row derives the same set in both engines.
# A hunk      ⇒  it NAMES the row, because each row is one line carrying its own count.
#
# ── WHY THIS IS NOT `run-axis.sh` ─────────────────────────────────────────────────────────────
#
# The other nine axes sweep a SIZE ladder and ask "which engine is faster." This one sweeps a
# SHAPE CORPUS and asks "can both engines even express this." Different question, different
# instrument — and forcing it through the size-sweeping runner is what made it un-growable:
# run-axis.sh spawns one process per cell, and MEASURED, one Clara cell costs ~3,500 ms of which
# essentially all is JVM cold boot + clojure load + clara compile. The fire is microseconds. At 3
# runs per row that is ~11 s per shape, so a 200-row corpus would have spent ~37 minutes booting a
# JVM 600 times.
#
# Here the JVM tax is paid ONCE no matter how large the corpus grows. Measured at 6 rows:
# wat 0.22 s + Clara 3.7 s, against ~67 s for the same six through run-axis.sh.
#
# ── NO TIMING, DELIBERATELY ───────────────────────────────────────────────────────────────────
#
# This script reports NO ratio and NO winner. Once the boot is amortised across the corpus, a
# per-row wall-clock comparison would be a lie (row 1 pays the boot, rows 2..N do not), and a
# fire-only ratio here would be a claim about microseconds inside a 4-second program. The nine
# perf axes are where speed is measured. This one measures whether the constraint can be said at
# all — and mixing the two is how the grid previously came to report superiority on ~1% of a
# runtime.
#
# ── THE CORPUS IS ONE PAIR PER FAMILY ─────────────────────────────────────────────────────────
#
#   where-shapes.wat        + where-shapes.clj          core: arith/accessor/string/collection/…
#   where-<family>.wat      + where-<family>.clj        one family per pair
#
# Every `where-*.wat` with a `.clj` twin is a pair, discovered not listed — so a new family is two
# new files and nothing else, and a rider adding one cannot collide with a rider adding another.
# (A gate that DISCOVERS beats a gate that LISTS; a hand-kept file list is the thing that drifts.)
#
# Each pair is SELF-CONTAINED — its own records, its own seed, its own rules. That is deliberate
# duplication: a family tuned to strings wants a string-heavy fact stream, one tuned to collections
# wants collections, and a shared stream would force every family through the same fact shape. It
# also means one family can never break another.
#
#   check-where-shapes.sh              # every pair
#   check-where-shapes.sh where-boolean # one pair, by stem — what a rider runs
#
# Oracle vs native is a different instrument: `check-spec-native.sh` on the same
# stems. Clara cannot catch a spec/native split. That script must stay green.
set -euo pipefail

GRID_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$GRID_DIR/../../.." && pwd)"
ONLY="${1:-}"

# The LOCAL build, never `cargo wat` — that resolves to whatever sits in ~/.cargo/bin, which has
# been hours stale across a compiler bump before now. A benchmark reading one build while you
# reason about another is an instrument supplying its own result.
WAT_BIN="${WAT_BIN:-$REPO_ROOT/target/release/wat}"
[ -x "$WAT_BIN" ] || {
  echo "check-where-shapes: no wat binary at $WAT_BIN — cargo build --release" >&2
  exit 1
}

# clojure needs java on PATH. This machine keeps Temurin under $HOME/opt/jdk-*.
if ! command -v java >/dev/null 2>&1; then
  if [ -n "${JAVA_HOME:-}" ] && [ -x "$JAVA_HOME/bin/java" ]; then
    export PATH="$JAVA_HOME/bin:$PATH"
  else
    for j in "$HOME"/opt/jdk-*/bin/java; do
      [ -x "$j" ] || continue
      export JAVA_HOME="$(cd "$(dirname "$j")/.." && pwd)"
      export PATH="$JAVA_HOME/bin:$PATH"
      break
    done
  fi
fi
if ! command -v java >/dev/null 2>&1; then
  echo "check-where-shapes: no java (PATH, JAVA_HOME, or \$HOME/opt/jdk-*)" >&2
  exit 1
fi

CLARA_DEP='{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}'
OUT_DIR="$(mktemp -d)"
trap 'rm -rf "$OUT_DIR"' EXIT

check_pair() {
  local stem="$1"
  local wat="$GRID_DIR/$stem.wat"
  local clj="$GRID_DIR/$stem.clj"

  # stderr is CAPTURED and echoed on failure, never discarded: a side that dies silently teaches
  # nothing, and a harness that hides the reason for its own failure is a mask.
  if ! "$WAT_BIN" "$wat" > "$OUT_DIR/$stem.wat.txt" 2> "$OUT_DIR/$stem.wat.err"; then
    echo "[$stem] the wat side FAILED" >&2
    cat "$OUT_DIR/$stem.wat.err" >&2
    return 1
  fi

  if ! clojure -Sdeps "$CLARA_DEP" -M "$clj" \
         > "$OUT_DIR/$stem.clj.txt" 2> "$OUT_DIR/$stem.clj.err"; then
    echo "[$stem] the Clara side FAILED" >&2
    tail -30 "$OUT_DIR/$stem.clj.err" >&2
    return 1
  fi

  # A pair that runs zero rows would diff clean and prove nothing — the vacuous-gate class (R59).
  # Assert both sides produced rows AND agree on how many, before trusting the comparison.
  local wn cn
  wn=$(wc -l < "$OUT_DIR/$stem.wat.txt")
  cn=$(wc -l < "$OUT_DIR/$stem.clj.txt")
  if [ "$wn" -lt 1 ]; then
    echo "[$stem] the wat side emitted NO rows — the pair did not run" >&2
    return 1
  fi
  if [ "$wn" -ne "$cn" ]; then
    echo "[$stem] row COUNT differs — wat $wn, Clara $cn (a row exists on one side only:" >&2
    echo "        check row-count in the .wat against the rows table in the .clj)" >&2
    diff "$OUT_DIR/$stem.wat.txt" "$OUT_DIR/$stem.clj.txt" >&2 || true
    return 1
  fi

  if diff -u "$OUT_DIR/$stem.wat.txt" "$OUT_DIR/$stem.clj.txt"; then
    echo "[$stem] $wn/$wn rows agree"
    ROWS_TOTAL=$(( ROWS_TOTAL + wn ))
    return 0
  fi
  echo "[$stem] DIVERGENCE — the hunk above names the row" >&2
  return 1
}

# DISCOVER the pairs; never list them. A `.wat` with no `.clj` twin is a HARD failure, not a skip —
# a half-authored family that silently reports nothing is exactly how a corpus rots into theatre.
FAILED=0
PAIRS=0
ROWS_TOTAL=0
for wat in "$GRID_DIR"/where-*.wat; do
  [ -e "$wat" ] || continue
  stem="$(basename "$wat" .wat)"
  if [ -n "$ONLY" ] && [ "$stem" != "$ONLY" ]; then continue; fi
  if [ ! -f "$GRID_DIR/$stem.clj" ]; then
    echo "[$stem] has NO .clj twin — a pair is two files; author it or delete the .wat" >&2
    FAILED=1; continue
  fi
  PAIRS=$(( PAIRS + 1 ))
  check_pair "$stem" || FAILED=1
done

if [ "$PAIRS" -eq 0 ]; then
  echo "check-where-shapes: matched NO pairs${ONLY:+ for '$ONLY'} — nothing was checked" >&2
  exit 1
fi

if [ "$FAILED" -eq 0 ]; then
  echo "where-shapes: $PAIRS pair(s), $ROWS_TOTAL rows — wat == Clara on every shape"
else
  echo "where-shapes: FAILURES above" >&2
  exit 1
fi
