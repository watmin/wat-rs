#!/usr/bin/env bash
# wat-scripts/perf/grid/peragrare-census.sh — the peragrare census for
# check-grid-three-way.sh: enumerates the discrimination space that instrument's
# oracle/native/Clara three-way SET comparison could tell apart, over the corpus it
# actually runs (the 16 sized, non-`where-*` axes it discovers — see MEMBERSHIP below),
# and reports which cells of that space are visited, hollow, or never asked.
#
# INSTRUMENT (one sentence, from reading check-grid-three-way.sh itself):
#   For each sized non-`where-*` grid axis, fire its ruleset through Clara 0.24.0,
#   wat-oracle (fire-rules$oracle) and wat-native (fire-rules) at its documented
#   correctness size, and declare the axis AGREED iff the three pairwise SET
#   comparisons of :derived/:oracle-derived (oracle-vs-clara="spec wrong",
#   native-vs-clara="fast path wrong", oracle-vs-native="port bug") all hold, given
#   non-empty results and an echoed :size matching what was sent.
#
# MEMBERSHIP: a fixture is a corpus member of THIS instrument iff
#   `check-grid-three-way.sh`'s own DISCOVERED loop would pick it up — every
#   `wat-scripts/perf/grid/*.wat` EXCLUDING the `where-*` family (that family is
#   handed to `check-where-shapes.sh` / `check-query-compat.sh` instead: a two-way
#   byte diff and a query-binding differential, both DIFFERENT questions, deliberately
#   not cast here per the ward's assumption 1 — "two comparisons are two instruments").
#   The `where-*` files are therefore a NEVER-OFFERED exclusion, not a silent drop:
#   check-grid-three-way.sh's own `case "$stem" in where-*) continue ;; esac` is the
#   line that excludes them, and two NAMED sibling instruments pick them up for their
#   own (different) questions.
#
# AXES: derived from what could make the instrument's three-way SET comparison come
# out differently — specifically, from the three defects that this exact corpus was
# built to close (the ward's own worked case) plus two more the corpus's own header
# comments name as open seams:
#   H  head-kind             {record, userfn}                — stratum-by-produced-type
#      vs stratum-by-fn-name (the userfn-head.wat cure, closed 2026-09-0x).
#   R  retract-duplicate     {absent, present}                — remove-one vs remove-all
#      (the retract-multiplicity.wat cure).
#   A  accumulate-source     {none, base, derived}            — an AccumulateNode's :from
#      type itself derived within the same ruleset, needing supersession across rounds
#      (the accum-over-derived.wat cure).
#   L  leading-nonmonotonic  {none, leading, nonleading}      — a parentless :exists /
#      accumulate re-emitting per fixpoint round into cumulative beta memory (the
#      2026-08-24 defect leading-exists.wat exists to catch, and the "family A" fuzzer
#      defect accum-lead-rule-cascade.wat's own header names).
#   N  negation-consumed     {na, not-consumed, consumed}     — a POSITIVE rule
#      consuming a fact gated by a negation elsewhere (task #94, neg-consumer.wat's
#      own header: "the grid tests two PURE regimes ... and never the INTERLEAVING").
#
# COORDINATE ASSIGNMENT is a HAND-DERIVED, CITED judgment, not a generic parser output.
# ⛔ NAMED LIMITATION (ward step: "the rule when one breaks: name the break, say what
# you did instead, and say what it moved"): this corpus mixes at least THREE distinct
# LHS/RHS authoring idioms — inline quoted forms (accum.wat), named let-bound variables
# referenced by bare symbol in the :lhs/:rhs vector (fanout.wat, min-finding.wat), and
# bracket-style `defrule :when [...] :then [...]` (accum.wat's rules, userfn-head.wat).
# A first-pass line-window text classifier was written and tested this session; it
# correctly re-derived axes R (retract) and A (accumulate-source) mechanically against
# all 16 real fixtures with no exceptions, but produced FALSE positives/negatives on H
# and L against the let-bound-variable idiom (e.g. fanout.wat's `:rhs (PV rhs)` pulled
# in an unrelated neighbouring `(:fan::all-facts ...)` call as a false "userfn" head)
# and on N's "gate" extraction. Rather than ship a classifier known to misclassify,
# H/A/L/N below are asserted by direct reading (cited inline) and cross-checked against
# a handful of MECHANICAL, comment-stripped presence/absence facts that a corpus edit
# would have to break loudly (see verify_fixture below) — this is what moved: coordinate
# TRUTH is a hand judgment; coordinate DRIFT DETECTION is mechanical.
#
# THE THREE PINS test the one part of this census that IS a script end to end: the
# cross-product AGGREGATION (bucketing a fixture's 5-tuple into one of the 108 cells).
# See --pins.
#
# Usage:
#   bash peragrare-census.sh                 # full report (cells, findings-eligible gaps)
#   bash peragrare-census.sh --verify        # re-run the mechanical presence/absence
#                                             # facts against the live corpus; nonzero
#                                             # exit + a loud message on any drift
#   bash peragrare-census.sh --pins          # populated / empty / moving pin
#   bash peragrare-census.sh --cell H R A L N  # population of one cell (or 0)
set -euo pipefail
GRID_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$GRID_DIR"

# ── MEMBERSHIP: reconcile against check-grid-three-way.sh's own discovery ──────────────────────
EXPECTED_FIXTURES=(accum accum-lead-rule-cascade accum-over-derived asym-join deep-cascade
  fanout leading-exists min-finding negation neg-consumer node-share parametric-erasure
  retract-multiplicity strat-neg userfn-head user-reduce)

live_discovered() {
  for wat in *.wat; do
    stem="$(basename "$wat" .wat)"
    case "$stem" in where-*) continue ;; esac
    echo "$stem"
  done | sort
}

check_membership() {
  local live expected diff
  live="$(live_discovered)"
  expected="$(printf '%s\n' "${EXPECTED_FIXTURES[@]}" | sort)"
  if [ "$live" != "$expected" ]; then
    echo "peragrare-census: MEMBERSHIP DRIFT — the live *.wat (non-where-*) population no" >&2
    echo "  longer matches EXPECTED_FIXTURES. The coordinate table below is keyed to the" >&2
    echo "  OLD population and must be re-derived (new/removed axis) before this census" >&2
    echo "  can be trusted. diff (live vs expected):" >&2
    diff <(echo "$live") <(echo "$expected") >&2 || true
    return 1
  fi
  return 0
}

# ── THE COORDINATE TABLE — hand-derived, cited (see file header) ───────────────────────────────
# name H R A L N
read -r -d '' TABLE <<'EOF' || true
accum record absent base nonleading na
accum-lead-rule-cascade record absent base leading na
accum-over-derived record absent derived nonleading na
asym-join record absent none none na
deep-cascade record absent none none na
fanout record absent none none na
leading-exists record absent none leading na
min-finding record absent base nonleading na
negation record absent none none not-consumed
neg-consumer record absent none none consumed
node-share record absent none none na
parametric-erasure record absent none none na
retract-multiplicity record present none none na
strat-neg record absent none none not-consumed
userfn-head userfn absent none none consumed
user-reduce record absent base nonleading na
EOF

# ── MECHANICAL DRIFT DETECTION — comment-stripped presence/absence facts per fixture ────────────
# `;;` full-line comments are stripped first (comments are exempt from code-position
# analysis — the same doctrine this repo's own rete-name lint applies to `.wat` code).
stripped() { grep -v '^\s*;;' "$1.wat"; }

verify_fixture() {  # name H R A L N
  local name="$1" H="$2" R="$3" A="$4" L="$5" N="$6" s ok=0
  s="$(stripped "$name")"
  # R: retract present/absent
  local retract_n; retract_n="$(grep -c 'wat::rete::retract\b' <<<"$s" || true)"
  if [ "$R" = present ] && [ "${retract_n:-0}" -eq 0 ]; then
    echo "  [$name] table says R=present but file has NO wat::rete::retract call" >&2; ok=1; fi
  if [ "$R" = absent ] && [ "${retract_n:-0}" -gt 0 ]; then
    echo "  [$name] table says R=absent but file DOES call wat::rete::retract" >&2; ok=1; fi
  # A: :from present iff A != none
  local from_n; from_n="$(grep -c ':from' <<<"$s" || true)"
  if [ "$A" = none ] && [ "${from_n:-0}" -gt 0 ]; then
    echo "  [$name] table says A=none but file HAS an accumulate :from clause" >&2; ok=1; fi
  if [ "$A" != none ] && [ "${from_n:-0}" -eq 0 ]; then
    echo "  [$name] table says A=$A but file has NO accumulate :from clause" >&2; ok=1; fi
  # L: an accumulate (builtin acc:: OR a custom-fold :from clause, e.g. user-reduce.wat's
  # `sum-of-squares :from Reading`) or a leading :exists present iff L != none
  local nm_n; nm_n="$(grep -c 'acc::\|rete::exists\|:from' <<<"$s" || true)"
  if [ "$L" = none ] && [ "${nm_n:-0}" -gt 0 ]; then
    echo "  [$name] table says L=none but file mentions acc::/exists" >&2; ok=1; fi
  if [ "$L" != none ] && [ "${nm_n:-0}" -eq 0 ]; then
    echo "  [$name] table says L=$L but file has NO acc::/exists mention" >&2; ok=1; fi
  # N: wat::rete::not present iff N != na
  local not_n; not_n="$(grep -c 'wat::rete::not\b' <<<"$s" || true)"
  if [ "$N" = na ] && [ "${not_n:-0}" -gt 0 ]; then
    echo "  [$name] table says N=na but file DOES call wat::rete::not" >&2; ok=1; fi
  if [ "$N" != na ] && [ "${not_n:-0}" -eq 0 ]; then
    echo "  [$name] table says N=$N but file has NO wat::rete::not call" >&2; ok=1; fi
  # H=userfn: only userfn-head.wat: the exact cited construction must still be present.
  if [ "$H" = userfn ]; then
    grep -qF ':then [(:ufh::mk-rate ?k)]' <<<"$s" || {
      echo "  [$name] table says H=userfn but the cited '(:ufh::mk-rate ?k)' :then is gone" >&2; ok=1; }
  fi
  return $ok
}

do_verify() {
  local fail=0
  check_membership || fail=1
  while read -r name H R A L N; do
    [ -z "$name" ] && continue
    verify_fixture "$name" "$H" "$R" "$A" "$L" "$N" || fail=1
  done <<<"$TABLE"
  if [ "$fail" -ne 0 ]; then
    echo "peragrare-census --verify: DRIFT DETECTED (see above) — the table needs re-deriving." >&2
    return 1
  fi
  echo "peragrare-census --verify: all ${#EXPECTED_FIXTURES[@]} fixtures' mechanical facts agree with the table."
}

# ── AGGREGATION — the one genuinely mechanical, pin-tested part ────────────────────────────────
H_VALUES=(record userfn)
R_VALUES=(absent present)
A_VALUES=(none base derived)
L_VALUES=(none leading nonleading)
N_VALUES=(na not-consumed consumed)

cell_count() {  # H R A L N -> population of that cell (0 if none / invalid)
  local qH="$1" qR="$2" qA="$3" qL="$4" qN="$5" n=0
  while read -r name H R A L N; do
    [ -z "$name" ] && continue
    if [ "$H" = "$qH" ] && [ "$R" = "$qR" ] && [ "$A" = "$qA" ] && [ "$L" = "$qL" ] && [ "$N" = "$qN" ]; then
      n=$((n+1))
    fi
  done <<<"$TABLE"
  echo "$n"
}

do_report() {
  check_membership
  echo "== VISITED CELLS =="
  local total_cells=0 visited=0 members=0
  declare -A seen
  while read -r name H R A L N; do
    [ -z "$name" ] && continue
    key="$H|$R|$A|$L|$N"
    if [ -n "${seen[$key]:-}" ]; then seen["$key"]="${seen[$key]} $name"; else seen["$key"]="$name"; fi
    members=$((members+1))
  done <<<"$TABLE"
  for k in "${!seen[@]}"; do
    IFS='|' read -r H R A L N <<<"$k"
    names="${seen[$k]}"
    cnt=$(wc -w <<<"$names")
    printf '  (%s,%s,%s,%s,%s) = %d  [%s]\n' "$H" "$R" "$A" "$L" "$N" "$cnt" "$names"
    visited=$((visited+1))
  done
  for _h in "${H_VALUES[@]}"; do for _r in "${R_VALUES[@]}"; do for _a in "${A_VALUES[@]}"; do
    for _l in "${L_VALUES[@]}"; do for _n in "${N_VALUES[@]}"; do total_cells=$((total_cells+1)); done
  done; done; done; done
  echo "total cells = $total_cells ; visited = $visited ; empty = $((total_cells-visited))"
  echo "members = $members"
}

do_pins() {
  echo "-- populated pin: (record,absent,none,none,na) --"
  local pop; pop="$(cell_count record absent none none na)"
  echo "   count = $pop (expect 5: asym-join deep-cascade fanout node-share parametric-erasure)"
  [ "$pop" -eq 5 ] && echo "   PASSED" || { echo "   FAILED"; exit 1; }

  echo "-- empty pin: H=lambda (a value no axis takes) --"
  local emp; emp="$(cell_count lambda absent none none na)"
  echo "   count = $emp (expect 0, and not one of the 108 grid cells since H has no such value)"
  [ "$emp" -eq 0 ] && echo "   PASSED" || { echo "   FAILED"; exit 1; }

  echo "-- moving pin: one synthetic row per non-default axis value (8 = (2-1)+(2-1)+(3-1)+(3-1)+(3-1)) --"
  local default="record absent none none na"
  local moved=0 total=0
  test_move() {  # label H R A L N
    total=$((total+1))
    local label="$1" H="$2" R="$3" A="$4" L="$5" N="$6"
    TABLE_SAVE="$TABLE"
    TABLE="$TABLE
synthetic-$label $H $R $A $L $N"
    local c; c="$(cell_count "$H" "$R" "$A" "$L" "$N")"
    TABLE="$TABLE_SAVE"
    if [ "$c" -ge 1 ]; then
      echo "   [$label] -> ($H,$R,$A,$L,$N) landed, count>=1 : OK"
      moved=$((moved+1))
    else
      echo "   [$label] -> ($H,$R,$A,$L,$N) DID NOT LAND"
    fi
  }
  test_move H-userfn        userfn absent none        none    na
  test_move R-present       record present none       none    na
  test_move A-base          record absent base        none    na
  test_move A-derived       record absent derived     none    na
  test_move L-leading       record absent none        leading na
  test_move L-nonleading    record absent none        nonleading na
  test_move N-not-consumed  record absent none        none    not-consumed
  test_move N-consumed      record absent none        none    consumed
  echo "   moved $moved of $total"
  [ "$moved" -eq "$total" ] && echo "   PASSED" || { echo "   FAILED"; exit 1; }
}

case "${1:-}" in
  --verify) do_verify ;;
  --pins) do_pins ;;
  --cell) shift; cell_count "$1" "$2" "$3" "$4" "$5" ;;
  *) do_report ;;
esac
