#!/usr/bin/env bash
# scripts/query-db.sh — run a wat interrogation script against a sqlite .db.
#
# Usage:
#   ./scripts/query-db.sh <db-path> <script.wat>
#
# Pipes <db-path> as a single line to the wat-cli binary's stdin, which
# the script's :user::main reads via `(:wat::io::IOReader/read-line stdin)`
# to know which database to open.
#
# Builds the wat-cli binary on first run if not present (release mode for
# the speed-of-script-iteration UX). Subsequent runs use the cached
# binary; `cargo build` keeps it fresh on substrate changes.
#
# Exit code: passes through whatever the wat-cli returns.
#
#   ./scripts/query-db.sh /tmp/some-run.db ./wat-scripts/count-logs.wat
#   ./scripts/query-db.sh ../holon-lab-trading/runs/pulse-X.db \
#     ./wat-scripts/metrics-summary.wat

set -euo pipefail

if [ "$#" -ne 2 ]; then
    echo "usage: $0 <db-path> <script.wat>" >&2
    exit 64
fi

DB_PATH="$1"
SCRIPT_PATH="$2"

if [ ! -f "$DB_PATH" ]; then
    echo "$0: db not found: $DB_PATH" >&2
    exit 66
fi

if [ ! -f "$SCRIPT_PATH" ]; then
    echo "$0: script not found: $SCRIPT_PATH" >&2
    exit 66
fi

# Locate the wat-cli binary. Prefer release; fall back to debug if
# only debug exists. Build release if neither exists.
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WAT_RELEASE="${REPO_ROOT}/target/release/wat"
WAT_DEBUG="${REPO_ROOT}/target/debug/wat"

if [ -x "$WAT_RELEASE" ]; then
    WAT_BIN="$WAT_RELEASE"
elif [ -x "$WAT_DEBUG" ]; then
    WAT_BIN="$WAT_DEBUG"
else
    echo "$0: building wat-cli (release) — first invocation" >&2
    (cd "$REPO_ROOT" && cargo build --release -p wat-cli)
    WAT_BIN="$WAT_RELEASE"
fi

# Resolve to absolute paths so the wat program sees real paths
# regardless of cwd.
DB_PATH="$(cd "$(dirname "$DB_PATH")" && pwd)/$(basename "$DB_PATH")"
SCRIPT_PATH="$(cd "$(dirname "$SCRIPT_PATH")" && pwd)/$(basename "$SCRIPT_PATH")"

echo "$DB_PATH" | "$WAT_BIN" "$SCRIPT_PATH"
