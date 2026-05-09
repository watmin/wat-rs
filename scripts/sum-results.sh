#!/usr/bin/env bash
# sum-results.sh — read cargo test output and sum passed/failed counts.
# Helper for arc 168 slice 1 closure when compound-pipe denials block
# inline awk aggregation.

set -euo pipefail

cargo test --release --workspace --no-fail-fast "$@" 2>&1 | grep "^test result" | sed -nE 's/.*\. ([0-9]+) passed; ([0-9]+) failed.*/\1 \2/p' | {
  TP=0
  TF=0
  while read -r p f; do
    TP=$((TP + p))
    TF=$((TF + f))
  done
  echo "passed: $TP failed: $TF"
}
