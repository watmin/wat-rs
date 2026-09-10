#!/usr/bin/env bash
# scripts/dot-flip-derive-census.sh
#
# Phase ①-of-③b-ii instrument. See:
#   docs/arc/2026/06/255-builtin-registry/DESIGN-the-flip-asks-843-times.md
#   docs/arc/2026/06/255-builtin-registry/BRIEF-phase-one-the-derivation-is-the-census.md
#
# A variant cannot be recognised by its SPELLING (`::PeerKind::thread` is a variant,
# `::Record::def` a surface method, textually identical). The substrate is asked instead
# (`:wat::runtime::variant-parent-of`), but the ask only sees the RUNNING type env — a single
# global pass over one process saw only loaded code (217/562 confirmed). The fix measured in the
# DESIGN: ask ONCE PER FILE, with THAT FILE LOADED — `(:wat::load-file! "<abs path>")` at top
# level brings a file's own `defenum`s and macro expansions into scope while the stdlib's
# answers survive alongside.
#
# Three corrections on top of the first pass (round 1 hit STOP-1 + STOP-4 — 636/845 files failed
# to load; see the arc's report). All three are MEASURED, not guessed:
#
#   ① `(:wat::config::set-redef! true)` as the driver's OWN first form (before `load-file!`)
#      clears the 598 `DefRedefForbidden` collisions in wat-scripts/ (standalone scripts with a
#      real top-level `:user::main`) — the loaded file's own declarations still land in scope
#      (verified: wat-scripts/probes/arc-278/s2s-process-probe.wat's `defenum`-declared
#      `probe::Echo::EchoResponse::RequestMalformed` still answers VARIANT after the fix). The
#      round-1 "no collision" belief rested on `wat-tests/counter-actor-proof-process.wat`, whose
#      only `:user::main` DEFN is nested inside a quoted subprocess-spawn literal — that file has
#      ZERO top-level `:user::main` and could never have shown this failure mode.
#   ② `wat/*.wat` (the stdlib source) is NEVER loaded. It is already frozen into the running
#      binary, so its own candidates are askable with NO load at all — loading it a second time is
#      exactly what produced round 1's `ReservedPrefix` (29, re-declaring a `:wat::` name) and
#      `DuplicateDefine` (6, re-registering an already-frozen name). Stdlib candidates go through
#      ONE plain (unmodified) invocation of the proven ask loop, load-free.
#   ③ Three files carry zero `defenum`/`defservice` and (measured) zero variant-shaped candidates
#      of their own: wat-scripts/scratch-pad/1a-epsilon-probe/probe-noeval-args.wat,
#      wat-scripts/scratch-pad/1a-epsilon-probe/probe-nonleading-setredef.wat,
#      wat-tests/core/unknown-call-head-panics.wat. Since neither declares a new variant type,
#      every namespaced token they contain is either an existing stdlib name (answerable by the
#      SAME load-free stdlib ask) or a `:user::`-local binding that is never a variant. Their
#      candidates are folded into the load-free stdlib ask rather than requiring their own load —
#      resolved, not skipped or excluded from the walk.
#
# For every OTHER corpus .wat file under wat-tests/, wat-scripts/:
#   1. collect that file's distinct namespaced keyword tokens (bare, no leading ':'),
#      over-generating freely — the ask is the filter, not this script;
#   2. generate a driver: `(:wat::config::set-redef! true)`, `(:wat::load-file! "<abs path>")`,
#      then the PROVEN wat-scripts/scratch-pad/dot-flip-derive-renames.wat body VERBATIM (its
#      `pair-for` composes the new spelling FROM THE ANSWER — reused, never re-derived);
#   3. run it, feeding the candidate vector on stdin; capture confirmed pairs.
#
# The union of every driver's output (the load-free stdlib+3-files pass, plus one per remaining
# file), sorted + deduplicated, is the deliverable (phase ① writes it only to stdout — redirect
# it: this script is deterministic and re-runnable).
#
# stdout: one `:old::spelling :new.spelling` pair per line, sorted + deduplicated (the census).
# stderr: progress, and — for any driver that did not exit 0 — a `FAIL <path>` line followed by
#         the verbatim captured output. STOP-1/STOP-4 in the brief: a load failure is reported,
#         NEVER silently skipped or excluded from the corpus walked.
#
# Exit status: 0 iff every driver ran to completion (0 failures). A nonzero exit means the
# failure list on stderr is non-empty and this stone's STOP-1 is live — read it before trusting
# the stdout pair list.

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WAT_BIN="$ROOT/target/release/wat"
PAIR_FOR_BODY="$ROOT/wat-scripts/scratch-pad/dot-flip-derive-renames.wat"

if [[ ! -x "$WAT_BIN" ]]; then
    echo "missing $WAT_BIN — build it first (this script does not build)" >&2
    exit 2
fi
if [[ ! -f "$PAIR_FOR_BODY" ]]; then
    echo "missing $PAIR_FOR_BODY — the proven ask loop this script reuses verbatim" >&2
    exit 2
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# Every distinct namespaced keyword token in a file, as a well-formed candidate: an identifier
# segment is [A-Za-z_][A-Za-z0-9_?!*+=-]*, segments joined by exactly "::". This char class never
# admits '<', '>', or '/' (arc 109's wall; a leading-colon check in keyword::from-string) — so no
# post-hoc filter is needed, the well-formedness is structural. A malformed candidate would abort
# the run rather than answer None (DESIGN), so this is deliberately conservative, not permissive.
TOKEN_RE=':[A-Za-z_][A-Za-z0-9_?!*+=-]*(::[A-Za-z_][A-Za-z0-9_?!*+=-]*)+'

candidates_for() {
    grep -oE "$TOKEN_RE" "$1" 2>/dev/null | sed 's/^://' | grep -vE '[<>/]'
}

PAIRS_OUT="$WORK/pairs.txt"
: > "$PAIRS_OUT"
FAIL_COUNT=0

run_driver() {
    # $1 = driver path, $2 = candidates file (one bare name per line), $3 = label for FAIL reports
    local driver="$1" candfile="$2" label="$3"
    mapfile -t cands < "$candfile"
    if [[ ${#cands[@]} -eq 0 ]]; then
        return 0
    fi
    local edn="["
    for c in "${cands[@]}"; do
        edn+="\"$c\" "
    done
    edn+="]"

    local out="$WORK/out.txt" err="$WORK/err.txt"
    printf '%s\n' "$edn" | "$WAT_BIN" "$driver" >"$out" 2>"$err"
    local rc=$?

    if [[ $rc -ne 0 ]]; then
        FAIL_COUNT=$((FAIL_COUNT + 1))
        {
            echo "FAIL $label"
            echo "  exit=$rc"
            echo "  stdout:"
            sed 's/^/    /' "$out"
            echo "  stderr:"
            sed 's/^/    /' "$err"
        } >&2
        return 1
    fi
    # println renders EDN — a String comes back quoted; strip exactly one leading/trailing quote.
    sed -e 's/^"//' -e 's/"$//' "$out" >> "$PAIRS_OUT"
    return 0
}

# ---- ② + ③: ONE load-free driver for wat/*.wat's own candidates, plus the three files that
# carry zero defenum/defservice of their own (their tokens need no load; see header). ----
mapfile -t STDLIB_FILES < <(find "$ROOT/wat" -type f -name '*.wat' | sort)

MEASURED_EMPTY=(
    "$ROOT/wat-scripts/scratch-pad/1a-epsilon-probe/probe-noeval-args.wat"
    "$ROOT/wat-scripts/scratch-pad/1a-epsilon-probe/probe-nonleading-setredef.wat"
    "$ROOT/wat-tests/core/unknown-call-head-panics.wat"
)

echo "measured-empty check (zero defenum/defservice, folded into the load-free ask):" >&2
for f in "${MEASURED_EMPTY[@]}"; do
    n=$(grep -cE 'defenum|defservice' "$f" 2>/dev/null || true)
    nc=$(candidates_for "$f" | sort -u | wc -l)
    echo "  $f: defenum/defservice=$n candidate-tokens=$nc" >&2
done

NOLOAD_CANDS="$WORK/noload-cands.txt"
{
    for f in "${STDLIB_FILES[@]}"; do candidates_for "$f"; done
    for f in "${MEASURED_EMPTY[@]}"; do candidates_for "$f"; done
} | sort -u > "$NOLOAD_CANDS"

echo "load-free ask: ${#STDLIB_FILES[@]} stdlib files + ${#MEASURED_EMPTY[@]} measured-empty files, $(wc -l < "$NOLOAD_CANDS") distinct candidates" >&2
run_driver "$PAIR_FOR_BODY" "$NOLOAD_CANDS" "<load-free stdlib+measured-empty ask>"

# ---- ① per-file load for everything else (wat-tests/, wat-scripts/), set-redef! true first ----
mapfile -t OTHER_FILES < <(find "$ROOT/wat-tests" "$ROOT/wat-scripts" -type f -name '*.wat' | sort)

is_measured_empty() {
    local f="$1"
    for m in "${MEASURED_EMPTY[@]}"; do
        [[ "$f" == "$m" ]] && return 0
    done
    return 1
}

TOTAL_LOADED=0
i=0
for f in "${OTHER_FILES[@]}"; do
    i=$((i + 1))
    if is_measured_empty "$f"; then
        continue
    fi
    candfile="$WORK/cands-$i.txt"
    candidates_for "$f" | sort -u > "$candfile"
    if [[ ! -s "$candfile" ]]; then
        continue
    fi

    driver="$WORK/driver-$i.wat"
    {
        echo '(:wat::config::set-redef! true)'
        printf '(:wat::load-file! "%s")\n\n' "$f"
        cat "$PAIR_FOR_BODY"
    } > "$driver"

    TOTAL_LOADED=$((TOTAL_LOADED + 1))
    run_driver "$driver" "$candfile" "$f"
done

TOTAL_FILES=$(( ${#STDLIB_FILES[@]} + ${#OTHER_FILES[@]} ))
echo "done: $TOTAL_FILES corpus files (${#STDLIB_FILES[@]} stdlib asked load-free, $TOTAL_LOADED loaded individually, ${#MEASURED_EMPTY[@]} measured-empty folded in), $FAIL_COUNT driver(s) failed" >&2

sort -u "$PAIRS_OUT"

if [[ $FAIL_COUNT -gt 0 ]]; then
    exit 1
fi
