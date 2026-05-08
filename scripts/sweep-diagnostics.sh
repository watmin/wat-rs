#!/usr/bin/env bash
# sweep-diagnostics.sh — run cargo test and filter output to the lines a
# substrate-as-teacher migration sweep cares about: walker error variant
# names + the surrounding source-location lines.
#
# Usage:
#   ./scripts/sweep-diagnostics.sh BareLegacyFnSignature
#   ./scripts/sweep-diagnostics.sh BareLegacyFnSignature --test wat_arc166_defn
#
# Output: each block is a walker firing with file/line context. Format:
#   ---
#   <walker-variant-name>
#   <source-location lines that follow in cargo's output>
#
# Exit code: 0 always (this is a diagnostic dump, not a pass/fail check).
# Use cargo-test-summary.sh or cargo-test-failures.sh for status.
#
# Why this script exists: substrate-as-teacher loops need to read walker
# diagnostics, but raw cargo output is verbose. This filters to JUST the
# walker firings + their context, removing the awk-pipe trigger from the
# agent's surface.

set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <walker-variant-name> [extra cargo args...]" >&2
  exit 2
fi

PATTERN="$1"
shift

LOG=$(mktemp -t wat-sweep-diag.XXXXXX)
trap 'rm -f "$LOG"' EXIT

cargo test --release --workspace --no-fail-fast "$@" >"$LOG" 2>&1 || true

# Print blocks that mention the pattern. Cargo prints diagnostic stacks
# that include the variant name + a "span:" or "in <file>:<line>:<col>"
# location-bearing line nearby. Use grep -B/-A to capture context.

# -B 2 -A 4 captures 2 lines before + 4 after each match — usually enough
# for the file:line context that follows the variant in error rendering.
grep -B 2 -A 4 "$PATTERN" "$LOG" || {
  echo "no '$PATTERN' diagnostics in cargo output" >&2
  exit 0
}
