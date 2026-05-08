#!/usr/bin/env bash
# cargo-test-summary.sh — run cargo test on the workspace, output a single-line
# summary that's safe for agents to read (no awk-pipe hallucination triggers).
#
# Usage:
#   ./scripts/cargo-test-summary.sh                 # full workspace
#   ./scripts/cargo-test-summary.sh --test wat_arc167_fn_flat_signature
#
# Output (always one line, ALWAYS first/only line of stdout):
#   passed: N failed: M
#
# Exit code: 0 if M=0, 1 otherwise. Lets shell scripts gate on success.
#
# Why this script exists: agents (sonnet especially) hallucinate "permission
# denied" when asked to pipe `cargo test` output through `awk '{p+=$N;...}'`
# patterns. Wrapping the awk inside this script removes the trigger; agents
# call this and read its single-line output cleanly.

set -euo pipefail

# Run cargo test through this wrapper; pass any extra args to cargo.
LOG=$(mktemp -t wat-cargo-summary.XXXXXX)
trap 'rm -f "$LOG"' EXIT

# --release is the project default; --no-fail-fast keeps all failures visible.
cargo test --release --workspace --no-fail-fast "$@" >"$LOG" 2>&1 || true

# Sum up "test result: ok. P passed; F failed" lines across all targets.
# Multiple test binaries each emit one summary line; we add them all up.
TOTAL_PASSED=0
TOTAL_FAILED=0
while IFS= read -r line; do
  # Lines look like: "test result: ok. 793 passed; 0 failed; ..."
  #              or: "test result: FAILED. 12 passed; 5 failed; ..."
  passed=$(echo "$line" | sed -nE 's/.*\. ([0-9]+) passed; .*/\1/p')
  failed=$(echo "$line" | sed -nE 's/.* ([0-9]+) failed.*/\1/p')
  if [[ -n "$passed" ]]; then
    TOTAL_PASSED=$((TOTAL_PASSED + passed))
  fi
  if [[ -n "$failed" ]]; then
    TOTAL_FAILED=$((TOTAL_FAILED + failed))
  fi
done < <(grep '^test result' "$LOG" || true)

echo "passed: $TOTAL_PASSED failed: $TOTAL_FAILED"

[[ $TOTAL_FAILED -eq 0 ]]
