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

# THE BINARY UNDER MEASUREMENT — the LOCAL build, never `cargo wat`.
#
# `cargo wat` resolves to whatever sits in ~/.cargo/bin, which is a DIFFERENT binary than the
# one in this tree: on 2026-07-30 the install was 7 hours older than target/release/wat, across
# a kernel purge and a compiler bump. A benchmark that reads one build while you reason about
# another is an instrument supplying its own result — and this runner had that bug on every
# axis, which is reason enough to distrust any number it produced before this line existed.
# Set WAT_BIN explicitly if you genuinely mean to measure an install.
REPO_ROOT="$(cd "$GRID_DIR/../../.." && pwd)"
WAT_BIN="${WAT_BIN:-$REPO_ROOT/target/release/wat}"
[ -x "$WAT_BIN" ] || {
  echo "run-axis: no wat binary at $WAT_BIN — build it first (cargo build --release)" >&2
  exit 1
}

# ── THE BLAST DOOR — a benchmark may not be able to take the workstation down ──
#
# 2026-07-30: `run-axis.sh node-share "10 2000" "50 2000"` CRASHED THE BUILDER'S MACHINE. The
# N=50 point consumed the box's 43 GiB of available RAM and the OOM killer took the desktop
# with it; the axis produced empty stdout AND empty stderr, which is the signature of a process
# being killed rather than failing. A benchmark that can do that is not a measurement tool.
#
# The cap is not caution, it is a wall: each side runs in its OWN cgroup scope with a hard
# MemoryMax and swap disabled, so a runaway dies alone and reports WHY. MemorySwapMax=0 matters
# more than the cap — swap thrash is what actually wedges a desktop before the OOM killer acts.
#
# Raise GRID_MEM_MAX deliberately for a big run; do not remove the guard. And grow a size ladder
# UPWARD from small (R24: these harnesses are O(n^2) interpreted — a point that returns fast
# says nothing about the next one).
GRID_MEM_MAX="${GRID_MEM_MAX:-6G}"
GRID_TIMEOUT="${GRID_TIMEOUT:-300}"

# ── REPEATS — one lucky sample may not produce a verdict ──────────────────────
#
# 2026-07-31: this runner reported `accum [100 200] ratio 1.0604 :winner :us` — a FLIP from a
# previously-recorded 0.653 :clara. Four more runs of the identical point read 0.6996 / 0.5911 /
# 0.8984 / 0.6134. The 1.06 was one sample at the top of an ~80%-wide distribution and the flip was
# not real. At the same time `[200 200]` read 0.664 against a prior 0.665 — two samples of a
# ~40%-wide distribution landing on top of each other, which looked like "unchanged" and was not.
#
# Single-run verdicts are therefore unreadable wherever the ratio is near parity. Each point now
# runs GRID_RUNS times and the verdict reports mean AND min/max — the SPREAD stays visible, because
# a bare mean would have concealed exactly the thing that produced the false flip.
GRID_RUNS="${GRID_RUNS:-3}"

if command -v systemd-run >/dev/null 2>&1 && systemd-run --user --scope --quiet true 2>/dev/null; then
  guard() { systemd-run --user --scope --quiet \
              -p MemoryMax="$GRID_MEM_MAX" -p MemorySwapMax=0 \
              -- timeout "$GRID_TIMEOUT" "$@"; }
  GUARD_KIND="cgroup MemoryMax=$GRID_MEM_MAX, no swap, ${GRID_TIMEOUT}s"
else
  # Fallback: address-space cap. Blunter than a cgroup (Rust reserves generously, so a false
  # trip is possible) but it still beats an unbounded run against the machine.
  GUARD_KB=$(( $(numfmt --from=iec "$GRID_MEM_MAX") / 1024 ))
  guard() { ( ulimit -v "$GUARD_KB"; exec timeout "$GRID_TIMEOUT" "$@" ); }
  GUARD_KIND="ulimit -v $GRID_MEM_MAX, ${GRID_TIMEOUT}s (no systemd-run)"
fi
echo "run-axis: guard = $GUARD_KIND" >&2

AXIS="${1:?usage: run-axis.sh AXIS SIZE...}"; shift
WAT_FILE="$GRID_DIR/$AXIS.wat"
GEN_FILE="$GRID_DIR/gen-$AXIS.sh"
[ -f "$WAT_FILE" ] || { echo "run-axis: missing $WAT_FILE" >&2; exit 1; }
[ -f "$GEN_FILE" ] || { echo "run-axis: missing $GEN_FILE" >&2; exit 1; }
[ "$#" -ge 1 ] || { echo "run-axis: need at least one SIZE" >&2; exit 1; }

for SIZE in "$@"; do
  SIZE_JSON="[$(echo "$SIZE" | tr -s ' ' ',')]"

  # The Clara program is generated ONCE per size and re-run; generation is not under measurement.
  CLJ_TMP="$(mktemp -d)"
  AXIS_FILE="${AXIS//-/_}"
  bash "$GEN_FILE" $SIZE > "$CLJ_TMP/$AXIS_FILE.clj"

  RATIOS=""
  ACCURACY=":match"

  for RUN in $(seq 1 "$GRID_RUNS"); do
    # ── wat side: native fire-rules, timed inside the script itself ──────────
    # stderr is CAPTURED, not discarded: `2>/dev/null` made a wat-side failure loud but
    # REASONLESS — you learned the axis produced nothing and never why.
    WAT_ERR="$(mktemp)"
    set +e
    WAT_OUT="$(echo "$SIZE_JSON" | guard "$WAT_BIN" "$WAT_FILE" 2>"$WAT_ERR")"
    WAT_RC=$?
    set -e
    WAT_LINE="$(echo "$WAT_OUT" | grep -o '#grid/Result.*' || true)"
    if [ -z "$WAT_LINE" ]; then
      echo "run-axis: wat side produced no #grid/Result for axis=$AXIS size=[$SIZE] run=$RUN (rc=$WAT_RC)" >&2
      echo "  binary: $WAT_BIN" >&2
      # rc is the diagnosis, and empty-output-with-no-error is the tell of a KILL, not a failure.
      case "$WAT_RC" in
        124) echo "  ⇒ TIMED OUT at ${GRID_TIMEOUT}s. Raise GRID_TIMEOUT, or the size is too big." >&2 ;;
        137) echo "  ⇒ KILLED (SIGKILL) — almost certainly the memory cap ($GRID_MEM_MAX)." >&2
             echo "    That is the guard working. Do NOT just raise it; a blowup at this size IS the finding." >&2 ;;
        *)   echo "  ⇒ exit $WAT_RC" >&2 ;;
      esac
      echo "  ── stdout ──" >&2; echo "$WAT_OUT" >&2
      echo "  ── stderr ──" >&2; cat "$WAT_ERR" >&2
      rm -f "$WAT_ERR"; rm -rf "$CLJ_TMP"
      exit 1
    fi
    rm -f "$WAT_ERR"

    # ── Clara side ────────────────────────────────────────────────────────────
    CLARA_OUT="$(cd "$CLJ_TMP" && clojure -Sdeps "$CLARA_DEP" -M -m "$AXIS" 2>/dev/null || true)"
    CLARA_LINE="$(echo "$CLARA_OUT" | grep -o '#grid/Result.*' || true)"
    if [ -z "$CLARA_LINE" ]; then
      echo "run-axis: Clara side produced no #grid/Result for axis=$AXIS size=[$SIZE] run=$RUN:" >&2
      echo "$CLARA_OUT" >&2
      rm -rf "$CLJ_TMP"
      exit 1
    fi

    # ── canonicalize :derived (strip wat's PersistentVector tag) + compare ────
    # Checked on EVERY run, not just the first: an accuracy divergence that only appears
    # sometimes is a worse finding than one that always does, and must not be sampled away.
    WAT_DERIVED="$(echo "$WAT_LINE" | grep -oP ':derived\s+(?:#wat\.core/PersistentVector\s+)?\K\[[^]]*\]')"
    CLARA_DERIVED="$(echo "$CLARA_LINE" | grep -oP ':derived\s+\K\[[^]]*\]')"
    WAT_NS="$(echo "$WAT_LINE" | grep -oP ':native-ns\s+\K[0-9]+')"
    CLARA_NS="$(echo "$CLARA_LINE" | grep -oP ':clara-ns\s+\K[0-9]+')"

    if [ -z "$WAT_DERIVED" ] || [ "$WAT_DERIVED" != "$CLARA_DERIVED" ]; then
      ACCURACY=":MISMATCH"
      echo "run-axis: MISMATCH axis=$AXIS size=[$SIZE] run=$RUN" >&2
      echo "  wat   :derived $WAT_DERIVED" >&2
      echo "  clara :derived $CLARA_DERIVED" >&2
    fi

    RATIOS="$RATIOS $(awk -v n="$WAT_NS" -v c="$CLARA_NS" 'BEGIN { printf "%.4f", (n>0)? c/n : -1 }')"
  done

  rm -rf "$CLJ_TMP"

  # ── mean + spread, and a verdict only where every run agrees ───────────────
  # :us / :clara require EVERY run to fall on the same side of the ±5% band. If the spread
  # straddles parity the honest answer is :unresolved — not the mean's coin flip. A matrix that
  # reports :unresolved for two points and stands behind nineteen is worth more than one that
  # reports twenty-one verdicts, two of which are samples.
  STATS="$(echo "$RATIOS" | awk '{ n=0; s=0; mn=1e18; mx=-1e18;
      for (i=1; i<=NF; i++) { v=$i+0; n++; s+=v; if (v<mn) mn=v; if (v>mx) mx=v }
      printf "%.4f %.4f %.4f", s/n, mn, mx }')"
  MEAN="$(echo "$STATS" | cut -d' ' -f1)"
  MIN="$(echo "$STATS"  | cut -d' ' -f2)"
  MAX="$(echo "$STATS"  | cut -d' ' -f3)"
  WINNER="$(awk -v mn="$MIN" -v mx="$MAX" 'BEGIN {
      if      (mn > 1.05) print ":us"
      else if (mx < 0.95) print ":clara"
      else                print ":unresolved" }')"

  echo "#grid/Verdict {:axis \"$AXIS\" :size [$(echo "$SIZE" | tr -s ' ' ' ')] :accuracy $ACCURACY :runs $GRID_RUNS :ratio $MEAN :min $MIN :max $MAX :winner $WINNER}"
done
