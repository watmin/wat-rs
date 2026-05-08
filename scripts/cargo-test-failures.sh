#!/usr/bin/env bash
# cargo-test-failures.sh — run cargo test on the workspace, output the list
# of failing test names grouped by test target.
#
# Usage:
#   ./scripts/cargo-test-failures.sh                 # full workspace
#   ./scripts/cargo-test-failures.sh --test foo      # specific binary
#
# Output:
#   target: <crate>::<binary>
#     test_name_1
#     test_name_2
#   target: <crate>::<binary2>
#     test_name_3
#   ---
#   total failures: N
#
# Exit code: 0 if no failures, 1 otherwise.
#
# Why this script exists: same reason as cargo-test-summary.sh — wraps the
# awk/sed parse inside a single-purpose script so agents don't hit the
# awk-pipe-denial hallucination. Output is structured for direct consumption
# by sonnet briefs that drive substrate-as-teacher migration loops.

set -euo pipefail

LOG=$(mktemp -t wat-cargo-failures.XXXXXX)
trap 'rm -f "$LOG"' EXIT

cargo test --release --workspace --no-fail-fast "$@" >"$LOG" 2>&1 || true

# Cargo emits "failures:" sections in each failing test binary. Each section
# lists failing test names. We extract them and group by the running-binary
# header that precedes the section.

awk '
  /^     Running / {
    # New test binary; capture its identifier.
    current_target = $0
    sub(/.* \(/, "", current_target)
    sub(/\)$/, "", current_target)
    # Strip the hash suffix from the binary path so the target is stable.
    sub(/-[a-f0-9]+$/, "", current_target)
    sub(/.*\//, "", current_target)
    failures_section = 0
    next
  }
  /^failures:$/ {
    failures_section = 1
    next
  }
  /^test result:/ {
    failures_section = 0
    next
  }
  failures_section == 1 && /^    [a-zA-Z0-9_:]+$/ {
    # Collect failure under current_target. Trim leading spaces.
    name = $0
    sub(/^    /, "", name)
    targets[current_target] = (targets[current_target] ? targets[current_target] "\n  " name : "  " name)
    count++
  }
  END {
    for (t in targets) {
      print "target: " t
      print targets[t]
    }
    print "---"
    print "total failures: " (count + 0)
  }
' "$LOG"

# Exit non-zero if there were failures.
if grep -q '^failures:$' "$LOG"; then
  exit 1
fi
exit 0
