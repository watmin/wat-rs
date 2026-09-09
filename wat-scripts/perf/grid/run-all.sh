#!/usr/bin/env bash
# wat-scripts/perf/grid/run-all.sh — sweep EVERY axis × its size ladder → the whole verdict grid.
#
# The DESIGN (docs/arc/2026/06/278-rules-engine/DESIGN-clara-grid.md:31) called for this script and
# it was never written, so every recorded grid was a set of ratios whose SIZES lived only in the
# session that produced them. The 2026-07-31 seam's "20 of 22 :us" table is un-reproducible for
# exactly that reason: it lists three ratios per axis and names the size for one axis only.
#
# A benchmark number whose input is not written down is not a measurement, it is an anecdote.
# THE LADDER BELOW IS THE ARTIFACT. Change it deliberately, never casually — a grid run at
# different sizes is not comparable to the one before it, and the ratios will silently disagree.
#
# Each ladder's TOP rung is the exemplar size documented in that axis's own `<axis>.wat` header
# ("Usage (stdin = an i64 vector …): => #grid/Result {… :size [N M] …}"), with two smaller rungs
# below it on that axis's documented free scale dial. The dial is named per axis below.
#
# Usage:  bash wat-scripts/perf/grid/run-all.sh            # every axis
#         bash wat-scripts/perf/grid/run-all.sh accum negation   # only the named axes
#
# Env:    GRID_RUNS=3  (honoured by run-axis.sh — a verdict needs EVERY run to agree, else
#                       :unresolved; see run-axis.sh)
#
# Emits every `#grid/Verdict` line from every axis, then a summary tally on stderr.
set -euo pipefail

GRID_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ── THE LADDER ───────────────────────────────────────────────────────────────────────────────
# axis        : dial swept (the other params are held fixed)   : the three rungs
declare -A LADDER=(
  [min-finding]="500 3|1000 3|2000 3"        # dial: stations   (threshold fixed at 3)
  [negation]="250|500|1000"                  # dial: items
  [leading-exists]="200|500|1000"            # dial: distinct locs (each Wind asserted twice).
                                              # THE REGRESSION AXIS for the 2026-08-24 defect: a
                                              # LEADING :exists read through a QUERY across a
                                              # multi-round fixpoint returned ROUNDS x locs rows.
                                              # Both references were right (Clara: 1 activation;
                                              # $oracle: immune, it rebuilds memories from empty),
                                              # so this axis mismatches on BOTH :accuracy and
                                              # :port-accuracy if it ever regresses. Verified to
                                              # fail against the pre-fix engine before landing.
  [neg-consumer]="250|500|1000"              # dial: items. THE THREE-WAY AXIS — a positive
                                              # consumer downstream of a negation gate, the one
                                              # shape no other axis crosses. Emits :oracle-derived
                                              # so the verdict carries :oracle-accuracy and
                                              # :port-accuracy. Task #94 was FOUND AND CLOSED
                                              # by this axis (ff581b6f); per neg-consumer.wat:38-43
                                              # any MISMATCH here is that regression returning.
  [asym-join]="500|1000|2000"                # dial: items
  [strat-neg]="6 500|6 1000|6 2000"          # dial: items      (strata fixed at 6 — chain depth)
  [user-reduce]="10 25|20 50|40 100"         # dial: locs × reads together
  [node-share]="10 200|25 200|50 200"        # dial: rules      (items fixed at 200)
  [accum]="50 200|100 200|200 200"           # dial: groups     (readings fixed at 200)
  [deep-cascade]="10 100|30 100|50 100"      # dial: depth      (width fixed at 100 — width 5
                                              # was 300 derived facts / 4 ms, an order of
                                              # magnitude under every other axis, where the
                                              # ratio measured jitter, not the engine)
  [fanout]="10000|20000|40000"               # dial: items (target derived-pair count; fanout
                                              # fixed at 20 internally) — 40000 is R4's Clara-win
                                              # size (REALIZATIONS.md:201), deliberately
)

# Deterministic order — the table reads the same every run.
ORDER=(min-finding negation leading-exists neg-consumer asym-join strat-neg user-reduce node-share accum deep-cascade fanout)

# ── DISCOVERY, because a LIST cannot notice what was never added to it ────────────────────────
#
# 2026-08-06: four axes (min-finding, node-share, strat-neg, user-reduce) sat DEAD for days — they
# died at rule-compile the hour law A armed (#57/#83) and every gate stayed green, because the
# loader gate only PARSES and nothing ran this script. All four were already in ORDER, so the list
# was not the bug. But the list is the bug WAITING: a NEW axis added to the directory without a
# LADDER entry is swept by nobody and says nothing, which is the identical silence one file over.
#
# So the ladder stays HAND-AUTHORED (the sizes are the artifact — see the header), and the axis SET
# is DISCOVERED from disk and reconciled against it. An axis on disk with no rung is an ERROR that
# names itself, never a silent skip. `check-where-shapes.sh` already works this way for the
# where-* corpus (it globs, and fails loudly on a .wat with no .clj twin); this brings the perf
# half to the same standard. [[feedback_a_gate_that_discovers_beats_one_that_lists]]
#
# A perf axis is exactly `<axis>.wat` WITH a `gen-<axis>.sh` twin — that pairing is what run-axis.sh
# requires, and it is what distinguishes a perf axis from the where-* expressivity corpus (which has
# static .clj twins and its own runner, deliberately: different question, different instrument).
DISCOVERED=()
for wat in "$GRID_DIR"/*.wat; do
  [ -e "$wat" ] || continue
  stem="$(basename "$wat" .wat)"
  [ -f "$GRID_DIR/gen-$stem.sh" ] || continue   # not a perf axis (where-* has no gen-, by design)
  DISCOVERED+=("$stem")
done

MISSING=()
for stem in "${DISCOVERED[@]}"; do
  [ -n "${LADDER[$stem]:-}" ] || MISSING+=("$stem")
done
if [ ${#MISSING[@]} -gt 0 ]; then
  echo "run-all: ${#MISSING[@]} axis/axes on disk have NO LADDER entry and would be SWEPT BY NOBODY:" >&2
  for m in "${MISSING[@]}"; do echo "    $m   (has $m.wat + gen-$m.sh, but no rung)" >&2; done
  echo "  Add a deliberate size ladder above — the sizes ARE the artifact, so choose them, do not" >&2
  echo "  copy a neighbour's. A grid run at different sizes is not comparable to the one before it." >&2
  exit 2
fi

# The reverse direction: a rung for an axis whose files are gone is a ladder pointing at nothing.
STALE=()
for stem in "${!LADDER[@]}"; do
  [ -f "$GRID_DIR/$stem.wat" ] && [ -f "$GRID_DIR/gen-$stem.sh" ] || STALE+=("$stem")
done
if [ ${#STALE[@]} -gt 0 ]; then
  echo "run-all: ${#STALE[@]} LADDER entry/entries name an axis with no .wat + gen-.sh pair:" >&2
  for s in "${STALE[@]}"; do echo "    $s" >&2; done
  exit 2
fi

# ORDER is the deliberate reading order and must cover everything discovered — otherwise a new
# axis has a rung, passes the check above, and is STILL never swept by a bare `run-all.sh`.
for stem in "${DISCOVERED[@]}"; do
  case " ${ORDER[*]} " in
    *" $stem "*) ;;
    *) echo "run-all: axis '$stem' has a LADDER rung but is absent from ORDER — a bare run would skip it" >&2
       exit 2 ;;
  esac
done

AXES=("$@")
if [ ${#AXES[@]} -eq 0 ]; then AXES=("${ORDER[@]}"); fi

# ── INSTRUMENT HEADER — every future capture is self-describing (`3W1`) ───────────────────────
#
# 29 `GRID-native-vs-clara-*.txt` captures through 2026-09-06 name no JDK and no Clojure CLI
# version anywhere in the file — a manual `bash run-all.sh > GRID-....txt` never recorded what it
# ran under, so `check-grid-speed.sh`'s FLOOR comment cites a grid whose instrument cannot be
# re-derived, defended, or refuted. This line is the fix, not a new capture wrapper: this script
# IS what a human redirects into a `GRID-*.txt` file, so printing the instrument to STDOUT, once,
# before the sweep, means the very next `> GRID-....txt` capture is self-describing for free.
#
# Consumers of these files match only `#grid/Verdict` lines (`check-grid-speed.sh`'s
# `case "$line" in \#grid/Verdict*)`, `compare-grids.sh`'s `/#grid\/Verdict/` awk pattern) and
# silently skip every other line, so this header is inert to both — verified by reading each.
# EDN-escape (backslash, then double-quote) — `java -version`'s own output embeds literal `"`
# around the version number, which would otherwise terminate the :java-version string early and
# leave the rest of the line as un-parseable trailing junk.
edn_escape() { sed 's/\\/\\\\/g; s/"/\\"/g'; }
JAVA_VERSION_TXT="$(java -version 2>&1 | tr '\n' ' ' | sed 's/  */ /g; s/ $//' | edn_escape || true)"
[ -n "$JAVA_VERSION_TXT" ] || JAVA_VERSION_TXT="java: not found on PATH"
CLOJURE_VERSION_TXT="$(clojure --version 2>&1 | edn_escape || true)"
[ -n "$CLOJURE_VERSION_TXT" ] || CLOJURE_VERSION_TXT="clojure: not found on PATH"
# Read the Clara pin from run-axis.sh's own CLARA_DEP rather than a second hardcoded copy — a
# duplicate literal here is exactly the kind of second copy that drifts undetected.
CLARA_VERSION_TXT="$(sed -n 's/.*com\.cerner\/clara-rules {:mvn\/version "\([^"]*\)"}.*/\1/p' "$GRID_DIR/run-axis.sh" | head -1)"
[ -n "$CLARA_VERSION_TXT" ] || CLARA_VERSION_TXT="unknown (could not read CLARA_DEP from run-axis.sh)"
printf '#grid/Capture {:captured-at "%s" :java-version "%s" :clojure-cli-version "%s" :clara-version "%s" :host "%s"}\n' \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$JAVA_VERSION_TXT" "$CLOJURE_VERSION_TXT" "$CLARA_VERSION_TXT" "$(hostname -s 2>/dev/null || echo unknown)"

rc=0
COMPLETED=0
# Captures every #grid/Verdict line from every axis, so the tally below can see them, WITHOUT
# changing what reaches our own stdout: `tee` still streams the exact same lines through, in the
# exact same order, that a bare `bash run-axis.sh` would have written directly.
ALL_OUT="$(mktemp)"; trap 'rm -f "$ALL_OUT"' EXIT
for axis in "${AXES[@]}"; do
  spec="${LADDER[$axis]:-}"
  if [ -z "$spec" ]; then
    echo "run-all: unknown axis '$axis' — known: ${ORDER[*]}" >&2
    exit 2
  fi
  # Split the |-separated rungs into positional SIZE args for run-axis.sh.
  IFS='|' read -r -a rungs <<< "$spec"
  echo "── $axis ──────────────────────────────────────────────" >&2
  # A failing axis must not abort the sweep — the whole point is the WHOLE grid, and a broken
  # axis is itself a finding. Record it and carry on.
  if bash "$GRID_DIR/run-axis.sh" "$axis" "${rungs[@]}" | tee -a "$ALL_OUT"; then
    COMPLETED=$(( COMPLETED + 1 ))
  else
    axis_rc=$?
    echo "run-all: axis '$axis' FAILED (rc=$axis_rc) — see its stderr above" >&2
    rc=1
  fi
done

# ── THE SUMMARY TALLY, PROMISED AT THE TOP OF THIS FILE AND NEVER BUILT ────────────────────────
# `ACCURACY`/`ORACLE_ACCURACY`/`PORT_ACCURACY` (run-axis.sh) never reach THIS script's exit
# code — that wiring is `3T1`, a separate row, deliberately untouched here. This tally is where
# that signal becomes visible at all: on stderr, without touching the exit code.
VERDICTS=$(grep -c '^#grid/Verdict' "$ALL_OUT" || true)
MATCHES=$(grep -c ':match' "$ALL_OUT" || true)
MISMATCHES=$(grep -c ':MISMATCH' "$ALL_OUT" || true)
echo "run-all: TALLY — axes $COMPLETED/${#AXES[@]} completed, $VERDICTS verdict(s) emitted," \
     "$MATCHES :match, $MISMATCHES :MISMATCH (:accuracy + :oracle-accuracy + :port-accuracy" \
     "combined)" >&2

exit $rc
