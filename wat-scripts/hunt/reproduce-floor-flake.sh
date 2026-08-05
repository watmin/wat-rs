#!/usr/bin/env bash
# reproduce-floor-flake.sh — hunt a LOAD-DEPENDENT floor failure and capture its ARM.
#
# WHY THIS EXISTS (arc 278 task #79). An intermittent failure
# (`test_run_string_entry_path`) fired once under a full-release floor run and then
# survived ~15 green re-runs. Two facts shaped this instrument:
#
#   1. It does NOT reproduce in isolation, nor when its own binary is run alone
#      (16 solo runs, clean). It needs the full 4356-test PARALLEL load.
#   2. When it DID fire, the failure text was truncated by a pager and lost. We
#      have a failure and no evidence about it.
#
# So re-running is not the problem — re-running WITHOUT CAPTURE is. This script
# loops the whole floor, oversubscribes the box to widen the race window, stops on
# the FIRST failure, and keeps the complete untruncated output.
#
# ⛔ "not reproducible" is a statement about your SEARCH, not about the bug. A
# flake is closed by a PROVEN MECHANISM or it stays open. Two prior dispositions
# of this exact passes-isolated/fails-under-load shape were both wrong; the third
# look root-caused it as a genuine ordering bug.
#
# USAGE
#   ./wat-scripts/hunt/reproduce-floor-flake.sh [RUNS] [THREADS]
#     RUNS     how many full-floor runs before giving up   (default 12)
#     THREADS  nextest test-threads; OVERSUBSCRIBE to widen
#              the window                                  (default 2x nproc)
#
# OUTPUT
#   wat-scripts/hunt/out/run-NNN.log   full untruncated output of every run
#   wat-scripts/hunt/out/SUMMARY.txt   one Summary line per run + the verdict
#
# ⛔ ORPHANS: this runs cargo repeatedly. It is BOUNDED (RUNS) and traps INT/TERM
# so a Ctrl-C does not leave a nextest holding the target/ lock. If you kill it any
# other way, `pkill -f cargo-nextest` before your next weigh.

set -uo pipefail

cd "$(dirname "$0")/../.." || exit 1

RUNS="${1:-12}"
THREADS="${2:-$(( $(nproc) * 2 ))}"
OUT="wat-scripts/hunt/out"
mkdir -p "$OUT"
: > "$OUT/SUMMARY.txt"

cleanup() {
  echo "[hunt] interrupted — reaping any live nextest so the target/ lock is free" | tee -a "$OUT/SUMMARY.txt"
  pkill -f cargo-nextest 2>/dev/null
  exit 130
}
trap cleanup INT TERM

echo "[hunt] $RUNS full-floor runs at --test-threads=$THREADS (nproc=$(nproc))" | tee -a "$OUT/SUMMARY.txt"
echo "[hunt] stopping on the FIRST failure, with the output kept whole" | tee -a "$OUT/SUMMARY.txt"

# Build ONCE up front so a compile does not eat into the first run's timing and so
# every iteration races the same binary.
cargo build --release --all-targets 2>&1 | tail -2

for i in $(seq 1 "$RUNS"); do
  log=$(printf '%s/run-%03d.log' "$OUT" "$i")

  # NO PIPE on the command itself — a piped exit code is the pager's, not
  # nextest's, and this arc has walked into that trap while quoting the rule
  # against it. Redirect whole, then read the file.
  cargo nextest run --release --test-threads="$THREADS" > "$log" 2>&1
  code=$?

  # Strip ANSI before matching: the Summary line is coloured, so a naive
  # `grep '^ *Summary'` never matches and silently reports nothing.
  summary=$(sed 's/\x1b\[[0-9;]*m//g' "$log" | grep -E '^ *Summary' | tail -1)
  printf 'run %03d  exit=%-3s %s\n' "$i" "$code" "${summary:-<no Summary line — read $log>}" \
    | tee -a "$OUT/SUMMARY.txt"

  if [ "$code" -ne 0 ]; then
    {
      echo
      echo "=============================================================="
      echo "[hunt] REPRODUCED on run $i. The arm is below — read it before"
      echo "[hunt] re-running anything. Full log: $log"
      echo "=============================================================="
      # Every FAIL line, then the captured stdout/stderr of the failing tests.
      sed 's/\x1b\[[0-9;]*m//g' "$log" | grep -E '^ *(FAIL|TRY|SIGSEGV)' || true
      echo "--- failure output (untruncated) ---"
      sed 's/\x1b\[[0-9;]*m//g' "$log" | sed -n '/^--- STDOUT:/,$p'
    } | tee -a "$OUT/SUMMARY.txt"
    exit 1
  fi
done

echo "[hunt] $RUNS/$RUNS clean at --test-threads=$THREADS." | tee -a "$OUT/SUMMARY.txt"
echo "[hunt] THIS IS NOT A CLEAN BILL OF HEALTH — it BOUNDS the frequency below" \
     "~1/$RUNS at this load, nothing more. Do not close the stone on it." \
     | tee -a "$OUT/SUMMARY.txt"
exit 0
