#!/usr/bin/env bash
# integration-run.sh — per-binary, leak-contained integration runner for the
# wat crate.
#
# Usage:
#   ./scripts/integration-run.sh [--all] [--timeout SECS] [--out FILE]
#
# Flags:
#   --all            Include the leaky-signal tier (arc-170 process class).
#                    Default: excluded (see tier heuristic below).
#   --timeout SECS   Per-binary timeout. Default: 60.
#   --out FILE       Inventory output path. Default: target/integration-inventory.tsv
#
# What it does (one paragraph):
#   Enumerates the wat crate's test binaries (tests/*.rs basenames + [[test]]
#   module-group names from Cargo.toml), optionally filters the leaky-signal
#   tier, builds the tests once, then runs each binary in its own session
#   (setsid + timeout) — capturing output to a temp file — and reaps the
#   entire session with pkill -s after EVERY binary regardless of outcome.
#   Leaked child processes structurally cannot escape; a hang costs one
#   binary's timeout, not the run. Writes a per-binary TSV inventory plus
#   a footer with totals and an error-class histogram.
#
# Why this script exists (arc 245.7 — the integration-runner stone):
#   green-gate.sh deliberately excludes the integration RUN because a
#   bare `cargo test --workspace` leaks processes: the arc-170 process class
#   (ambient-stdio / fork / lifeline integration tests) spawns children that
#   outlive their harness. The user has reaped leaked procs twice this week.
#   This script provides the CONTAINED run so the ~190-failure integration
#   tier can be inventoried without poisoning the dev machine.
#
# Tier heuristic (HEURISTIC — containment backstops misclassification):
#   Default tier EXCLUDES any binary whose source file contains at least one
#   match for the regex: spawn_process|run_hermetic|fork|pidfd|lifeline|ambient
#   This is a heuristic for the arc-170 process class (~67 files at the time
#   of writing). It is NOT a guaranteed partition — containment (setsid +
#   pkill -s) is the true safety net; the heuristic only narrows the run to
#   the portion most likely to be green-able. --all includes all binaries
#   (still contained).
#
# Inventory format (TSV, one line per binary):
#   name<TAB>status<TAB>counts<TAB>failing-test-names
#   status: pass | fail | timeout
#   counts: Np/Mf/Ki (e.g. 3p/0f/1i) — blank for timeout rows
#   failing-test-names: comma-joined; empty if none
#
# Footer (after a leading # per line):
#   Totals: binaries run/passed/failed/timed-out; tests passed/failed/ignored
#   Error-class histogram: NoMatchingClause | UnresolvedReference |
#     MalformedForm | TypeMismatch | UnboundSymbol
#
# Containment loop (proven mechanic — BRIEF stone 245.7):
#   setsid bash -c 'sleep 300 & exit' &  →  orphan survives in session <leader-pid>
#   pkill -s <leader-pid>                →  session empty
#
# Exit code: 0 iff every run binary passed; 1 otherwise.
# (It will exit 1 today — the tier is red; the deliverable is the inventory.)
#
# Comparable: scripts/green-gate.sh (arc 239) — style precedent.
set -euo pipefail

cd "$(dirname "$0")/.."

# ── argument parsing ──────────────────────────────────────────────────────────
INCLUDE_ALL=0
TIMEOUT=60
OUT="target/integration-inventory.tsv"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --all)          INCLUDE_ALL=1; shift ;;
        --timeout)      TIMEOUT="$2"; shift 2 ;;
        --out)          OUT="$2"; shift 2 ;;
        *) echo "Unknown flag: $1" >&2; exit 1 ;;
    esac
done

# ── enumerate test binaries ───────────────────────────────────────────────────
# Source 1: flat tests/*.rs → basename without extension
# Source 2: [[test]] name = "..." entries in Cargo.toml
mapfile -t FLAT_NAMES < <(
    ls tests/*.rs 2>/dev/null \
    | xargs -n1 basename \
    | sed 's/\.rs$//' \
    | sort
)

mapfile -t MODULE_GROUP_NAMES < <(
    awk '/^\[\[test\]\]/{in_block=1} in_block && /^name =/{
        gsub(/.*= "|".*/, "", $0); print; in_block=0
    }' Cargo.toml \
    | sort
)

# Combine and deduplicate (module group names shadow any same-named flat file)
mapfile -t ALL_NAMES < <(
    printf '%s\n' "${FLAT_NAMES[@]}" "${MODULE_GROUP_NAMES[@]}" \
    | sort -u
)

echo "Enumerated ${#ALL_NAMES[@]} test binaries (${#FLAT_NAMES[@]} flat + ${#MODULE_GROUP_NAMES[@]} module-group)"

# ── tier filter ───────────────────────────────────────────────────────────────
# HEURISTIC: exclude binaries matching arc-170 process-class signals.
# For flat tests/*.rs: grep the source file directly.
# For [[test]] module groups: grep the group's directory.
# containment backstops any misclassification.

LEAKY_SIGNAL='spawn_process|run_hermetic|fork|pidfd|lifeline|ambient'

declare -a TIER_NAMES=()
declare -a EXCLUDED_NAMES=()

for name in "${ALL_NAMES[@]}"; do
    if [[ "$INCLUDE_ALL" == "1" ]]; then
        TIER_NAMES+=("$name")
        continue
    fi

    # Determine the source path to check
    src_file="tests/${name}.rs"
    src_dir="tests/${name}"

    leaky=0
    if [[ -f "$src_file" ]]; then
        if grep -qE "$LEAKY_SIGNAL" "$src_file" 2>/dev/null; then
            leaky=1
        fi
    elif [[ -d "$src_dir" ]]; then
        if grep -rlE "$LEAKY_SIGNAL" "$src_dir" 2>/dev/null | grep -q .; then
            leaky=1
        fi
    fi
    # If neither file nor dir found: include (STOP-3 will catch missing binaries)

    if [[ "$leaky" == "1" ]]; then
        EXCLUDED_NAMES+=("$name")
    else
        TIER_NAMES+=("$name")
    fi
done

echo "Default tier: ${#TIER_NAMES[@]} binaries (excluded ${#EXCLUDED_NAMES[@]} leaky-signal matches)"

if [[ "${#TIER_NAMES[@]}" -eq 0 ]]; then
    echo "ERROR: no binaries in tier — nothing to run." >&2
    exit 1
fi

# ── build once ────────────────────────────────────────────────────────────────
echo "== integration-run: building tests (cargo build --release --tests -p wat) =="
if ! cargo build --release --tests -p wat 2>&1; then
    echo "ERROR: build failed — every binary would show 'fail' misleadingly; stopping." >&2
    exit 1
fi
echo "== build OK =="

# ── run loop ──────────────────────────────────────────────────────────────────
# Proven containment mechanic (BRIEF stone 245.7):
#   setsid timeout $T cargo test ... & → test binary runs in new session
#   sid=$!; wait "$sid"               → wait for session leader to exit
#   pkill -s "$sid" 2>/dev/null       → reap the ENTIRE session — ALWAYS

TMPDIR_CAP="$(mktemp -d)"
trap 'rm -rf "$TMPDIR_CAP"' EXIT

mkdir -p "$(dirname "$OUT")"
: > "$OUT"   # truncate / create

total_bins=0
total_passed_bins=0
total_failed_bins=0
total_timeout_bins=0
total_p=0
total_f=0
total_i=0

declare -A error_class_counts
for cls in NoMatchingClause UnresolvedReference MalformedForm TypeMismatch UnboundSymbol; do
    error_class_counts[$cls]=0
done

timeout_names=()

echo "== integration-run: running ${#TIER_NAMES[@]} binaries (timeout ${TIMEOUT}s each) =="

for name in "${TIER_NAMES[@]}"; do
    cap="$TMPDIR_CAP/${name}.cap"
    : > "$cap"

    # ── THE CONTAINMENT LOOP (do not reorder; pkill-s runs on ALL paths) ──
    setsid timeout "$TIMEOUT" cargo test --release -p wat --test "$name" \
        > "$cap" 2>&1 &
    sid=$!
    # `|| code=$?` keeps errexit from aborting on a red binary (proven: bare
    # `wait; code=$?` under set -e dies at the FIRST failure, skipping the reap).
    code=0; wait "$sid" || code=$?
    pkill -s "$sid" 2>/dev/null || true     # ALWAYS — success, fail, or timeout
    # ── end containment loop ──────────────────────────────────────────────

    total_bins=$(( total_bins + 1 ))

    if [[ "$code" -eq 124 ]]; then
        # timeout
        status="timeout"
        counts=""
        failing_names=""
        total_timeout_bins=$(( total_timeout_bins + 1 ))
        timeout_names+=("$name")
    else
        # Parse: "test result: ok. N passed; M failed; K ignored"
        result_line=$(grep -oE 'test result: [^;]+; [0-9]+ failed; [0-9]+ ignored' "$cap" 2>/dev/null || true)

        n_p=0; n_f=0; n_i=0
        if [[ -n "$result_line" ]]; then
            n_p=$(echo "$result_line" | grep -oP '\d+(?= passed)')  || n_p=0
            n_f=$(echo "$result_line" | grep -oP '\d+(?= failed)')  || n_f=0
            n_i=$(echo "$result_line" | grep -oP '\d+(?= ignored)') || n_i=0
        fi

        # Extract failing test names from the 'failures:' block
        # Format in cargo output:
        #   failures:
        #     test_name_1
        #     test_name_2
        failing_block=$(awk '/^failures:$/{found=1; next} found && /^$/{exit} found{print $1}' "$cap" 2>/dev/null || true)
        failing_names=$(echo "$failing_block" | tr '\n' ',' | sed 's/,$//' | sed 's/^,//')

        if [[ "$n_f" -gt 0 || "$code" -ne 0 ]]; then
            status="fail"
            total_failed_bins=$(( total_failed_bins + 1 ))
        else
            status="pass"
            total_passed_bins=$(( total_passed_bins + 1 ))
        fi

        counts="${n_p}p/${n_f}f/${n_i}i"
        total_p=$(( total_p + n_p ))
        total_f=$(( total_f + n_f ))
        total_i=$(( total_i + n_i ))

        # Error-class histogram
        for cls in NoMatchingClause UnresolvedReference MalformedForm TypeMismatch UnboundSymbol; do
            n=$(grep -c "$cls" "$cap" 2>/dev/null || true)
            error_class_counts[$cls]=$(( error_class_counts[$cls] + n ))
        done
    fi

    printf '%s\t%s\t%s\t%s\n' "$name" "$status" "$counts" "$failing_names" >> "$OUT"
    printf '  %-65s %s\n' "$name" "$status"
done

echo "== integration-run: run complete =="

# ── STOP-2 check ──────────────────────────────────────────────────────────────
timeout_threshold=$(( (total_bins + 3) / 4 ))   # 25% ceiling
if [[ "${#timeout_names[@]}" -gt "$timeout_threshold" ]]; then
    echo "" >&2
    echo "STOP-2: >${timeout_threshold} of ${total_bins} default-tier binaries timed out." >&2
    echo "Timeout list:" >&2
    printf '  %s\n' "${timeout_names[@]}" >&2
    echo "The tier heuristic may be wrong — leaky binaries inside the default tier." >&2
fi

# ── footer ────────────────────────────────────────────────────────────────────
{
    echo "# ─── TOTALS ────────────────────────────────────────────────────────"
    echo "# binaries_run=${total_bins}  passed=${total_passed_bins}  failed=${total_failed_bins}  timed_out=${total_timeout_bins}"
    echo "# tests_passed=${total_p}  tests_failed=${total_f}  tests_ignored=${total_i}"
    echo "# ─── ERROR-CLASS HISTOGRAM ─────────────────────────────────────────"
    for cls in NoMatchingClause UnresolvedReference MalformedForm TypeMismatch UnboundSymbol; do
        echo "# ${cls}=${error_class_counts[$cls]}"
    done
    echo "# ─── EXCLUDED (leaky-signal heuristic) ─────────────────────────────"
    echo "# excluded_count=${#EXCLUDED_NAMES[@]}"
    echo "# inventory_path=$(realpath "$OUT" 2>/dev/null || echo "$OUT")"
} >> "$OUT"

# Print footer to stdout too
echo ""
echo "=== INVENTORY FOOTER ==="
tail -n 10 "$OUT"
echo "========================"
echo "Inventory: $OUT  ($(wc -l < "$OUT") lines)"

# ── exit code ─────────────────────────────────────────────────────────────────
if [[ "$total_failed_bins" -gt 0 || "$total_timeout_bins" -gt 0 ]]; then
    exit 1
fi
exit 0
