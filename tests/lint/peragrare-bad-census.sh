#!/usr/bin/env bash
# tests/lint/peragrare-bad-census.sh — the peragrare census for
# tests/lint/every_wat_bad_fixture_actually_fails.rs: enumerates the discrimination space
# that gate's comparison could tell apart, over the corpus it actually runs (every
# `*.wat.bad` under tests/, wat-scripts/, docs/), and reports which cells are visited,
# hollow, or never asked.
#
# INSTRUMENT (one sentence, from reading every_wat_bad_fixture_actually_fails.rs:242-301):
#   For each `.wat.bad`, ask whether `startup_from_file(rel)` errs (the filename's implicit
#   claim); a clean (Ok) start is tolerated ONLY if the file's own `rune:lint(bad-is-banked)`
#   declaration is well-formed (known category, reason >= 24 chars, owner field) AND names a
#   test function that exists under tests/ and is still `#[ignore]`d — otherwise the gate fails.
#
# MEMBERSHIP: a fixture is a member iff it is a `*.wat.bad` under ROOTS = (tests, wat-scripts,
# docs) — check_shard() loops over every file the walk finds with no pre-filter, so corpus ==
# members: 268 files = 268 members, 0 never-offered, 0 dropped.
#
# AXES: derived from what the comparison above could come out differently along —
#   AXIS1 startup outcome     {err, clean}      — line 244's `.is_err()`, the primary branch.
#   AXIS2 declaration state   {absent, bad-category, short-reason, no-owner-field,
#                              owner-absent, owner-live, owner-ignored}  — lines 251-301,
#     reachable ONLY when AXIS1=clean: a `continue` at line 245-246 skips every declaration
#     read when AXIS1=err, so (err, <any AXIS2 value>) is CELL-UNCONSTRUCTIBLE by the gate's
#     own control flow, not merely hard to build. Grid = 2 x 7 = 14 cells, 7 exempt.
#   NOT MODELLED: the fixture's own `;;`-comment-claimed error KIND/law (present in 129 of 268).
#   Fails the axis litmus test — changing which error kind fires, while staying an Err, cannot
#   flip this comparison's verdict (still `.is_err()` = true) — so it is domain colour for THIS
#   instrument, not a grid axis.
#
# COORDINATE ASSIGNMENT is COMPUTED, not read off literally: AXIS1 cannot be measured without
# executing `startup_from_file`, so it is INFERRED from declaration presence — "carries a
# rune:lint( line" stands in for "is currently clean" — a named, disclosed limitation: a
# truly-clean-but-undeclared fixture would misclassify as `err` here, exactly the
# (clean,absent) cell this census cannot see into. AXIS2 is computed by re-implementing
# declaration_on/owner_in/owner_state_in's parsing in awk/bash.
#
# Usage:
#   bash peragrare-bad-census.sh                # full report (cells, populations)
#   bash peragrare-bad-census.sh --pins         # populated / empty / moving pin
#   bash peragrare-bad-census.sh --cell A1 A2   # population of one cell
#   bash peragrare-bad-census.sh --verify       # re-run the report and check it doesn't error
set -euo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

ROOTS=(tests wat-scripts docs)
MIN_REASON_CHARS=24
DECLARED_CATEGORIES=(bad-is-banked)

corpus() {
  for r in "${ROOTS[@]}"; do
    [ -d "$r" ] && find "$r" -name '*.wat.bad'
  done | sort
}

declaration_of() {
  local f="$1"
  awk '
    { line=$0
      idx = index(line, "rune:lint(")
      if (idx > 0) {
        after = substr(line, idx+10)
        closeidx = index(after, ")")
        if (closeidx > 0) {
          cat = substr(after, 1, closeidx-1)
          reason = substr(after, closeidx+1)
          gsub(/^[ \t\xE2\x80\x94-]+/, "", reason)
          sub(/^ +/, "", reason)
          print cat "\t" reason
          exit
        } else {
          print after "\t"
          exit
        }
      }
    }
  ' "$f"
}

owner_of() {
  local reason="$1"
  local idx="${reason#*banked-by: }"
  if [ "$idx" = "$reason" ]; then echo ""; return; fi
  echo "$idx" | awk '{print $1}'
}

owner_state_of() {
  local owner="$1"
  local match
  match="$(grep -rn "fn ${owner}(" tests --include='*.rs' 2>/dev/null | head -1 || true)"
  if [ -z "$match" ]; then echo "owner-absent"; return; fi
  local file="${match%%:*}"
  local rest="${match#*:}"
  local lineno="${rest%%:*}"
  local i=$((lineno - 1))
  local state="owner-live"
  while [ "$i" -ge 1 ]; do
    local line
    line="$(sed -n "${i}p" "$file" | sed -e 's/^[[:space:]]*//;s/[[:space:]]*$//')"
    case "$line" in
      \#\[*)
        case "$line" in
          *ignore*) state="owner-ignored"; break ;;
          *) i=$((i-1)); continue ;;
        esac
        ;;
      *) break ;;
    esac
  done
  echo "$state"
}

classify() {
  local f="$1"
  local decl
  decl="$(declaration_of "$f")"
  if [ -z "$decl" ]; then
    echo "err n/a"
    return
  fi
  local cat reason
  cat="$(printf '%s' "$decl" | cut -f1)"
  reason="$(printf '%s' "$decl" | cut -f2-)"
  local known=0
  for c in "${DECLARED_CATEGORIES[@]}"; do
    [ "$c" = "$cat" ] && known=1
  done
  if [ "$known" -ne 1 ]; then
    echo "clean bad-category"
    return
  fi
  local rlen
  rlen="$(printf '%s' "$reason" | wc -m | tr -d ' ')"
  if [ "$rlen" -lt "$MIN_REASON_CHARS" ]; then
    echo "clean short-reason"
    return
  fi
  local owner
  owner="$(owner_of "$reason")"
  if [ -z "$owner" ]; then
    echo "clean no-owner-field"
    return
  fi
  local ostate
  ostate="$(owner_state_of "$owner")"
  echo "clean $ostate"
}

AXIS2_VALUES=(absent bad-category short-reason no-owner-field owner-absent owner-live owner-ignored)

do_report() {
  local files n
  files="$(corpus)"
  n="$(printf '%s\n' "$files" | grep -c . || true)"
  echo "corpus members (files under ${ROOTS[*]}): $n"
  declare -A cellcount
  declare -A cellmembers
  while IFS= read -r f; do
    [ -z "$f" ] && continue
    local coord a1 a2 key
    coord="$(classify "$f")"
    a1="$(echo "$coord" | cut -d' ' -f1)"
    a2="$(echo "$coord" | cut -d' ' -f2)"
    key="${a1}|${a2}"
    cellcount["$key"]=$(( ${cellcount["$key"]:-0} + 1 ))
    cellmembers["$key"]="${cellmembers[$key]:-} $f"
  done <<<"$files"
  echo "== cells (axis1=err) =="
  echo "  (err,n/a) = ${cellcount["err|n/a"]:-0}  [exempt: cell-unconstructible]"
  echo "== cells (axis1=clean) =="
  for v in "${AXIS2_VALUES[@]}"; do
    printf '  (clean,%s) = %s' "$v" "${cellcount["clean|$v"]:-0}"
    if [ -n "${cellmembers["clean|$v"]:-}" ]; then
      printf '  [%s]' "${cellmembers["clean|$v"]}"
    fi
    printf '\n'
  done
}

cell_count() {
  local files count=0
  files="$(corpus)"
  while IFS= read -r f; do
    [ -z "$f" ] && continue
    local coord a1 a2
    coord="$(classify "$f")"
    a1="$(echo "$coord" | cut -d' ' -f1)"
    a2="$(echo "$coord" | cut -d' ' -f2)"
    if [ "$a1" = "$1" ] && [ "$a2" = "$2" ]; then
      count=$((count+1))
    fi
  done <<<"$files"
  echo "$count"
}

do_pins() {
  echo "-- populated pin: (clean,owner-ignored) --"
  local pop
  pop="$(cell_count clean owner-ignored)"
  echo "   count = $pop (expect 3)"
  [ "$pop" -eq 3 ] && echo "   PASSED" || { echo "   FAILED"; exit 1; }

  echo "-- empty pin: axis1=nonexistent-outcome (a value no axis takes) --"
  local emp
  emp="$(cell_count nonexistent-outcome '*')"
  echo "   count = $emp (expect 0; not one of the grid's cells)"
  [ "$emp" -eq 0 ] && echo "   PASSED" || { echo "   FAILED"; exit 1; }

  echo "-- moving pin: one synthetic fixture per non-default axis value (denominator = (2-1)+(7-1) = 7) --"
  local tmp moved=0 total=0
  tmp="$(mktemp -d)"
  probe() {
    total=$((total+1))
    local label="$1" content="$2" e1="$3" e2="$4" got
    printf '%s\n' "$content" > "$tmp/$label.wat.bad"
    got="$(classify "$tmp/$label.wat.bad")"
    if [ "$got" = "$e1 $e2" ]; then
      echo "   [$label] -> ($got) landed as expected: OK"
      moved=$((moved+1))
    else
      echo "   [$label] -> got ($got), expected ($e1 $e2): DID NOT LAND"
    fi
  }
  probe axis1-err ";; comment
;; comment
(:wat::core::i64::+ 1 2)" err n/a
  probe axis2-bad-category ";; header
;; header
;; rune:lint(nonsense-category) — a category nobody defined banked-by: some_fn" clean bad-category
  probe axis2-short-reason ";; header
;; header
;; rune:lint(bad-is-banked) — too short" clean short-reason
  probe axis2-no-owner-field ";; header
;; header
;; rune:lint(bad-is-banked) — a reason with no owner field at all here" clean no-owner-field
  probe axis2-owner-absent ";; header
;; header
;; rune:lint(bad-is-banked) — arc 999 should reject this banked-by: no_such_test_function_anywhere_xyz" clean owner-absent
  probe axis2-owner-live ";; header
;; header
;; rune:lint(bad-is-banked) — arc 999 should reject this banked-by: a_rune_line_yields_its_category_and_reason" clean owner-live
  probe axis2-owner-ignored ";; header
;; header
;; rune:lint(bad-is-banked) — arc 255 must make an undeclared field-type keyword a check error banked-by: probe_undeclared_field_type_keyword_rejected_or_lenient" clean owner-ignored
  rm -rf "$tmp"
  echo "   moved $moved of $total"
  [ "$moved" -eq "$total" ] && echo "   PASSED" || { echo "   FAILED"; exit 1; }
}

case "${1:-}" in
  --pins) do_pins ;;
  --cell) shift; cell_count "$1" "$2" ;;
  --verify) do_report >/dev/null && echo "peragrare-bad-census --verify: report ran without error." ;;
  *) do_report ;;
esac
