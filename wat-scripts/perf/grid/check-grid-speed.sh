#!/usr/bin/env bash
# check-grid-speed.sh — the grid's SPEED half, as a CI gate.
#
# ── WHY THIS EXISTS, AND WHY THE OLD REASON FOR ITS ABSENCE IS GONE ───────────────────────────
#
# `circumspicere` found the speed half runs in no CI job. The recorded reason was "it needs Clara
# and a JDK the runner lacks" — which EXPIRED on 2026-08-27 when the `parity` job added Temurin 21
# and a pinned Clojure CLI. A second reason was then offered and never written down: a shared CI
# runner is a noisy instrument, so a wall-clock regression gate there would flap.
#
# ⛔ THAT SECOND REASON DID NOT SURVIVE MEASUREMENT EITHER, and this is the point of the gate's
# shape. From the recorded 33-cell grid (`GRID-native-vs-clara-2026-08-27T07-15-56Z.txt`):
#
#     tightest cell  8.50x     median  22.09x     widest  59.11x        33/33 :us, 33/33 :match
#
# We are not near parity anywhere. Runner noise would have to be ~8x to flip a `:winner`, which
# is also exactly why a `:winner` gate would be NEARLY VACUOUS — it fires only on catastrophe, and
# would have missed the real 4x regression this arc already found and fixed.
#
# So the gate is a per-axis RATIO FLOOR at ~50% of the recorded minimum. Two properties earn it:
#   · A RATIO cancels runner speed. Both engines run in the same job on the same box; a slow
#     runner slows the numerator and denominator together. That is the whole reason the row's own
#     note said "a ratio against Clara measured in the same job", and it is why a raw ms threshold
#     was the wrong instrument, not why the gate was.
#   · The margin is 2x, not 5%. A gate whose trip point sits 2x from the measurement cannot flap
#     on a noisy box; it fires when something halves, which is the regression class worth catching.
#
# GRID_RUNS=1 in CI IS DELIBERATE, not a corner cut. The 3-run convention exists so a NEAR-PARITY
# verdict (`:winner`, a +-5% band) is not read off one sample. This gate's margin is 2x, so one
# sample settles it, and the run costs ~2 minutes instead of ~7. The 3-run artifact stays the
# reference measurement; this is a regression tripwire, not a benchmark.
#
# ⚠ THE FLOORS ARE THE ARTIFACT. Derived from the recorded grid above, one per axis because the
# ratios legitimately span 8.5x-59x. Raise one only with a new recorded grid to cite, and never
# lower one to make a red go away — a ratio that fell by half is the finding this exists to
# report.
#
# ── THE CAPTURE'S JDK — KNOWN, EVIDENCE, AND WHAT SUPERSEDES IT (`3W1`) ────────────────────────
#
# KNOWN: `GRID-native-vs-clara-2026-08-27T07-15-56Z.txt` — the grid the FLOOR array below derives
# from — names no JDK and no Clojure CLI version anywhere in the file. Neither does any other
# recorded `GRID-*.txt` in this directory (29 of 29, checked 2026-09-09; `grep -il
# 'jdk|temurin|openjdk|java '` over all of them returns nothing). CI pins `distribution: temurin`,
# `java-version: "21"`. Whether the 2026-08-27 capture ran under that JDK, a different major
# version, or a different distribution entirely was never recorded, so it cannot be re-derived,
# defended, or refuted from the file cited above.
#
# EVIDENCE, NOT PROOF: the box this capture was measured on runs, TODAY (checked 2026-09-09, not
# at capture time), `Temurin-21.0.12+8` via `JAVA_HOME=$HOME/opt/jdk-21` — the same distribution
# and major version CI pins, reached through the ordinary `$HOME/opt/jdk-*` discovery this box
# still uses. That makes it PLAUSIBLE the capture ran under the JDK CI now pins. It is NOT proof:
# a box's JDK today is not evidence of that same box's JDK on 2026-08-27, and nothing ties the two
# together. Read this floor as "measured on an unrecorded JDK, on a box that currently happens to
# run the one CI pins" — never as "measured on Temurin 21."
#
# WHAT SUPERSEDES THIS: `run-all.sh` now emits a `#grid/Capture` header (java -version, Clojure
# CLI version, the pinned Clara version, capture timestamp) as the first line of its stdout, so
# the next full grid recorded through the ordinary `bash run-all.sh > GRID-....txt` capture is
# self-describing by construction. THAT grid — not this comment, not a fresh read of the box's
# CURRENT `java -version` — becomes the provenance-bearing baseline once it exists. Re-deriving
# these floor numbers under it is deliberately NOT done here: it is a performance act needing six
# samples, a quiet box, and its own scorecard, not a recording-mechanism fix.
set -euo pipefail

GRID_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# axis : floor (~50% of that axis's minimum ratio in the 2026-08-27 grid — a grid whose own JDK
# was never recorded; see "THE CAPTURE'S JDK" above)
declare -A FLOOR=(
  [accum]=6.3          [asym-join]=8.2      [deep-cascade]=6.6
  [fanout]=4.2         [leading-exists]=14.9 [min-finding]=26.6
  [negation]=9.1       [neg-consumer]=8.0   [node-share]=11.8
  [strat-neg]=17.8     [user-reduce]=4.7
)

OUT="$(mktemp)"; trap 'rm -f "$OUT"' EXIT
echo "check-grid-speed: sweeping every axis x its ladder (GRID_RUNS=1, oracle skipped)" >&2
GRID_SKIP_ORACLE=1 GRID_RUNS="${GRID_RUNS:-1}" bash "$GRID_DIR/run-all.sh" | tee "$OUT"

fail=0; seen=0
while IFS= read -r line; do
  case "$line" in \#grid/Verdict*) ;; *) continue ;; esac
  seen=$((seen + 1))
  axis=$(sed -E 's/.*:axis "([a-z-]+)".*/\1/' <<<"$line")
  size=$(sed -E 's/.*:size \[([^]]*)\].*/\1/' <<<"$line")
  acc=$(sed -E 's/.*:accuracy :([A-Za-z]+).*/\1/' <<<"$line")
  ratio=$(sed -E 's/.*:ratio ([0-9.]+).*/\1/' <<<"$line")

  # CORRECTNESS first. This corpus is NOT the where-family the parity job already diffs — a
  # native-vs-Clara mismatch on a perf axis is a wrong answer nothing else here would see.
  if [ "$acc" != "match" ]; then
    echo "GRID MISMATCH  $axis [$size] :accuracy :$acc — native and Clara disagree on this axis" >&2
    fail=1; continue
  fi

  floor="${FLOOR[$axis]:-}"
  if [ -z "$floor" ]; then
    echo "check-grid-speed: axis '$axis' has NO floor — add one (with a recorded grid to cite) or" \
         "remove the axis; an unfloored axis is a cell this gate silently cannot fail" >&2
    fail=1; continue
  fi
  if awk -v r="$ratio" -v f="$floor" 'BEGIN{exit !(r < f)}'; then
    echo "GRID SPEED REGRESSION  $axis [$size] ratio $ratio < floor $floor" \
         "(we are still $(awk -v r=$ratio 'BEGIN{printf "%.1f", r}')x Clara, but that is HALF what" \
         "this axis measured on 2026-08-27 — something got materially slower)" >&2
    fail=1
  fi
done < "$OUT"

# NON-VACUITY. Without this the gate passes just as happily on an empty sweep — a broken runner,
# a renamed axis, a `run-all.sh` that emitted nothing. The count is the floor's own floor.
if [ "$seen" -lt 30 ]; then
  echo "check-grid-speed: only $seen verdict(s) — the 2026-08-27 grid had 33. A short sweep is a" \
       "gate that cannot fail, not a green one" >&2
  exit 2
fi

if [ "$fail" -ne 0 ]; then
  echo "check-grid-speed: FAILED — see the lines above ($seen cells swept)" >&2
  exit 1
fi
echo "check-grid-speed: OK — $seen cells, every one :match and above its floor" >&2
