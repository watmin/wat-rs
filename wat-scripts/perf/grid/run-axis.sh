#!/usr/bin/env bash
# wat-scripts/perf/grid/run-axis.sh AXIS SIZE...
#
# THE shared Clara-grid runner (docs/arc/2026/06/278-rules-engine/DESIGN-clara-grid.md) — every
# axis (A0..A8) mirrors the SAME three artifacts and gets swept by this ONE script:
#   grid/<axis>.wat        — the wat workload; fires native `:wat::rete::fire-rules`; stdin =
#                             an i64 vector [size...]; stdout = one #grid/Result EDN line.
#   grid/gen-<axis>.sh     — emits the Clara translation (.clj) of the SAME workload; args =
#                             the size numbers (word-split, positional); stdout (of the emitted
#                             program's -main) = one #grid/Result EDN line with :clara-ns.
#   (this script)          — runs both sides per SIZE, canonicalizes + compares :derived
#                             (ACCURACY) and :native-ns vs :clara-ns (SPEED), emits a #grid/Verdict.
#
# SIZE is a space-separated size-tuple, e.g. "3 10" (strat-neg: strata items) or "20 10"
# (deep-cascade: depth width) — axis-specific shape, opaque to this runner. Pass one SIZE per
# sweep point: `run-axis.sh strat-neg "3 10" "6 2000"` runs two points.
#
# CANONICALIZATION (STOP trigger #2 in the DESIGN — tag-form must be reconciled, not faked away):
# wat's EDN printer tags every PersistentVector as `#wat.core/PersistentVector [...]` (a real,
# load-bearing round-trip-identity decision, wat/src/edn_shim.rs) while Clojure's `pr-str` of a
# plain vector is bare `[...]`. The VALUES are identical either way — only the wire wrapper
# differs — so this runner strips the wat-side tag before the string-compare. That is exactly
# the canonicalization the DESIGN calls for, not a fudge: the accuracy check still fails loudly
# on any real difference in the derived-fact SET (missing/extra/reordered elements).
#
# Emits, per SIZE, one line:
#   #grid/Verdict {:axis "<axis>" :size [<size>] :accuracy :match|:MISMATCH :ratio <f64> :winner :us|:clara|:tie}
# :ratio is clara-ns / native-ns (>1 ⇒ native faster ⇒ :us; <1 ⇒ Clara faster ⇒ :clara; within
# ±5% ⇒ :tie). A :MISMATCH also dumps both :derived sets to stderr (never hidden — DESIGN STOP #4).
set -euo pipefail

GRID_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLARA_DEP='{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}} :paths ["."]}'

AXIS="${1:?usage: run-axis.sh AXIS SIZE...}"; shift
WAT_FILE="$GRID_DIR/$AXIS.wat"
GEN_FILE="$GRID_DIR/gen-$AXIS.sh"
[ -f "$WAT_FILE" ] || { echo "run-axis: missing $WAT_FILE" >&2; exit 1; }
[ -f "$GEN_FILE" ] || { echo "run-axis: missing $GEN_FILE" >&2; exit 1; }
[ "$#" -ge 1 ] || { echo "run-axis: need at least one SIZE" >&2; exit 1; }

for SIZE in "$@"; do
  SIZE_JSON="[$(echo "$SIZE" | tr -s ' ' ',')]"

  # ── wat side: native fire-rules, timed inside the script itself ──────────
  WAT_OUT="$(echo "$SIZE_JSON" | cargo wat "$WAT_FILE" 2>/dev/null || true)"
  WAT_LINE="$(echo "$WAT_OUT" | grep -o '#grid/Result.*' || true)"
  if [ -z "$WAT_LINE" ]; then
    echo "run-axis: wat side produced no #grid/Result for axis=$AXIS size=[$SIZE]:" >&2
    echo "$WAT_OUT" >&2
    exit 1
  fi

  # ── Clara side: generate the .clj, run it in its own tmp dir, capture stdout ──
  # Clojure's classpath loader maps ns `strat-neg` -> file `strat_neg.clj` (hyphen -> underscore);
  # the emitted `(ns <axis> ...)` keeps the hyphen (readable), the FILE on disk must not.
  CLJ_TMP="$(mktemp -d)"
  AXIS_FILE="${AXIS//-/_}"
  bash "$GEN_FILE" $SIZE > "$CLJ_TMP/$AXIS_FILE.clj"
  CLARA_OUT="$(cd "$CLJ_TMP" && clojure -Sdeps "$CLARA_DEP" -M -m "$AXIS" 2>/dev/null || true)"
  rm -rf "$CLJ_TMP"
  CLARA_LINE="$(echo "$CLARA_OUT" | grep -o '#grid/Result.*' || true)"
  if [ -z "$CLARA_LINE" ]; then
    echo "run-axis: Clara side produced no #grid/Result for axis=$AXIS size=[$SIZE]:" >&2
    echo "$CLARA_OUT" >&2
    exit 1
  fi

  # ── canonicalize :derived (strip wat's PersistentVector tag) + compare ────
  WAT_DERIVED="$(echo "$WAT_LINE" | grep -oP ':derived\s+(?:#wat\.core/PersistentVector\s+)?\K\[[^]]*\]')"
  CLARA_DERIVED="$(echo "$CLARA_LINE" | grep -oP ':derived\s+\K\[[^]]*\]')"
  WAT_NS="$(echo "$WAT_LINE" | grep -oP ':native-ns\s+\K[0-9]+')"
  CLARA_NS="$(echo "$CLARA_LINE" | grep -oP ':clara-ns\s+\K[0-9]+')"

  if [ -n "$WAT_DERIVED" ] && [ "$WAT_DERIVED" = "$CLARA_DERIVED" ]; then
    ACCURACY=":match"
  else
    ACCURACY=":MISMATCH"
    echo "run-axis: MISMATCH axis=$AXIS size=[$SIZE]" >&2
    echo "  wat   :derived $WAT_DERIVED" >&2
    echo "  clara :derived $CLARA_DERIVED" >&2
  fi

  RATIO="$(awk -v n="$WAT_NS" -v c="$CLARA_NS" 'BEGIN { printf "%.4f", (n>0)? c/n : -1 }')"
  WINNER=":$(awk -v r="$RATIO" 'BEGIN { print (r > 1.05) ? "us" : (r < 0.95 ? "clara" : "tie") }')"

  echo "#grid/Verdict {:axis \"$AXIS\" :size [$(echo "$SIZE" | tr -s ' ' ' ')] :accuracy $ACCURACY :ratio $RATIO :winner $WINNER}"
done
