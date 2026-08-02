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
set -euo pipefail

GRID_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$GRID_DIR/../../.." && pwd)"

# The LOCAL build, never `cargo wat` — that resolves to whatever sits in ~/.cargo/bin, which has
# been hours stale across a compiler bump before now. A benchmark reading one build while you
# reason about another is an instrument supplying its own result.
WAT_BIN="${WAT_BIN:-$REPO_ROOT/target/release/wat}"
[ -x "$WAT_BIN" ] || {
  echo "check-where-shapes: no wat binary at $WAT_BIN — cargo build --release" >&2
  exit 1
}

CLARA_DEP='{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}'
OUT_DIR="$(mktemp -d)"
trap 'rm -rf "$OUT_DIR"' EXIT

# stderr is CAPTURED and echoed on failure, never discarded: a side that dies silently teaches
# nothing, and a harness that hides the reason for its own failure is a mask.
if ! "$WAT_BIN" "$GRID_DIR/where-shapes.wat" > "$OUT_DIR/wat.txt" 2> "$OUT_DIR/wat.err"; then
  echo "check-where-shapes: the wat side failed" >&2
  cat "$OUT_DIR/wat.err" >&2
  exit 1
fi

if ! clojure -Sdeps "$CLARA_DEP" -M "$GRID_DIR/where-shapes.clj" \
       > "$OUT_DIR/clara.txt" 2> "$OUT_DIR/clara.err"; then
  echo "check-where-shapes: the Clara side failed" >&2
  tail -30 "$OUT_DIR/clara.err" >&2
  exit 1
fi

# A corpus that runs zero rows would diff clean and prove nothing — the vacuous-gate class (R59).
# Assert both sides produced rows AND agree on how many, before trusting the comparison.
WAT_ROWS=$(wc -l < "$OUT_DIR/wat.txt")
CLARA_ROWS=$(wc -l < "$OUT_DIR/clara.txt")
if [ "$WAT_ROWS" -lt 1 ]; then
  echo "check-where-shapes: the wat side emitted NO rows — the corpus did not run" >&2
  exit 1
fi
if [ "$WAT_ROWS" -ne "$CLARA_ROWS" ]; then
  echo "check-where-shapes: row COUNT differs — wat $WAT_ROWS, Clara $CLARA_ROWS" >&2
  echo "  (a row exists on one side only: check row-count in the .wat against `rows` in the .clj)" >&2
  diff "$OUT_DIR/wat.txt" "$OUT_DIR/clara.txt" >&2 || true
  exit 1
fi

if diff -u "$OUT_DIR/wat.txt" "$OUT_DIR/clara.txt"; then
  echo "where-shapes: $WAT_ROWS/$WAT_ROWS rows agree — wat == Clara on every shape"
else
  echo "where-shapes: DIVERGENCE — the hunk above names the row" >&2
  exit 1
fi
