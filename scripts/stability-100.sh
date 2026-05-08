#!/usr/bin/env bash
# 100-round stability test. Captures workspace pass/fail counts per run.
# Output: /tmp/stability-100.log (per-run lines) + summary at end.
set -u

LOG=/tmp/stability-100.log
SUMMARY=/tmp/stability-100-summary.log
ROUNDS=${1:-100}

: > "$LOG"
: > "$SUMMARY"

clean_runs=0
flake_runs=0
build_failures=0

for i in $(seq 1 "$ROUNDS"); do
  start=$(date +%s)
  out=$(cargo test --release --workspace --no-fail-fast 2>&1 || true)
  end=$(date +%s)
  elapsed=$((end - start))

  # Sum across all "test result" lines
  passed=$(printf '%s\n' "$out" | grep -E '^test result' | awk '{ s += $4 } END { print s+0 }')
  failed=$(printf '%s\n' "$out" | grep -E '^test result' | awk '{ s += $6 } END { print s+0 }')

  # Detect build failure (no test result lines at all)
  result_lines=$(printf '%s\n' "$out" | grep -cE '^test result' || true)

  if [ "$result_lines" -eq 0 ]; then
    build_failures=$((build_failures + 1))
    printf 'run %3d: BUILD FAILURE (%ds)\n' "$i" "$elapsed" | tee -a "$LOG"
    # Capture compile error excerpt
    printf '%s\n' "$out" | grep -E '^error' | head -5 >> "$LOG"
    continue
  fi

  if [ "$failed" -eq 0 ]; then
    clean_runs=$((clean_runs + 1))
    printf 'run %3d: passed=%s failed=%s (%ds) CLEAN\n' "$i" "$passed" "$failed" "$elapsed" | tee -a "$LOG"
  else
    flake_runs=$((flake_runs + 1))
    printf 'run %3d: passed=%s failed=%s (%ds) FLAKE\n' "$i" "$passed" "$failed" "$elapsed" | tee -a "$LOG"
    # Capture which tests failed
    printf '  --- failing tests run %d ---\n' "$i" >> "$LOG"
    printf '%s\n' "$out" | grep -E '^test .* FAILED|^failures:$|^    [a-zA-Z_:]+' | head -50 >> "$LOG"
  fi
done

{
  echo "=== STABILITY-100 SUMMARY ==="
  echo "rounds: $ROUNDS"
  echo "clean: $clean_runs"
  echo "flake: $flake_runs"
  echo "build_failures: $build_failures"
  if [ "$ROUNDS" -gt 0 ]; then
    rate=$(awk "BEGIN { printf \"%.1f\", ($clean_runs * 100.0) / $ROUNDS }")
    echo "clean_rate: ${rate}%"
  fi
} | tee "$SUMMARY"
