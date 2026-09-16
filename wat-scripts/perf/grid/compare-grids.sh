#!/usr/bin/env bash
# compare-grids.sh OLD-GRID.txt NEW-GRID.txt — diff two recorded grids, cell by cell.
#
# ── WHY THIS EXISTS ───────────────────────────────────────────────────────────────────────────
#
# Thirteen `GRID-native-vs-clara-*.txt` files sit in this directory and NOTHING read them against
# each other. Every "we got faster / we regressed" claim this arc has made about the grid was made
# by eye, from two files on a screen — which is a number with no instrument, and a number with no
# instrument cannot be re-derived by the next hand. The comparison is mechanical; this is it.
#
# ⛔ IT PRINTS TWO DELTAS PER CELL, AND CONFLATING THEM IS THE MISTAKE IT EXISTS TO PREVENT.
#
#   Δwat   — OUR engine's fire time (`:wat-ns`). Attributable to us. A change here is a change
#            we made, modulo box noise.
#   Δratio — the VERDICT (`:ratio` = clara-ns / wat-ns). This is what the grid history means and
#            what `check-grid-speed.sh` gates, but it is a QUOTIENT: it moves when the JVM moves,
#            when the box moves, or when we move. A ratio that improved while `:wat-ns` got worse
#            means CLARA got slower, not that we got faster.
#
# When the two disagree in sign, the cell is flagged `⚠DIVERGE` — that is the interesting cell,
# and reading only the ratio column is how it would be missed.
#
# Cells are matched on (axis, size). A cell in one file and not the other is reported as ONLY-OLD
# or ONLY-NEW rather than silently dropped: a ladder change makes two grids incomparable, and the
# runner's own header says so ("a grid run at different sizes is not comparable to the one before
# it"). An :accuracy that is not :match, on either side, is reported whatever its timings did —
# a fast wrong answer is not a result.
#
# Usage:  bash compare-grids.sh GRID-...-2026-08-31T00-40-36Z.txt GRID-...-2026-09-02T....txt
set -euo pipefail

[ $# -eq 2 ] || { echo "usage: compare-grids.sh OLD-GRID.txt NEW-GRID.txt" >&2; exit 2; }
OLD="$1"; NEW="$2"
[ -r "$OLD" ] || { echo "compare-grids: cannot read $OLD" >&2; exit 2; }
[ -r "$NEW" ] || { echo "compare-grids: cannot read $NEW" >&2; exit 2; }

awk -v oldf="$OLD" -v newf="$NEW" '
# POSIX awk only — mawk is the awk on this box, so no gawk 3-arg match()/RS extensions.
function field(line, name,   re, rest, v) {
  re = ":" name " "
  if (!match(line, re)) return ""
  rest = substr(line, RSTART + RLENGTH)
  v = rest
  sub(/[ }].*$/, "", v)
  return v
}
function sizeof(line,   rest, v) {
  if (!match(line, /:size \[/)) return "?"
  rest = substr(line, RSTART + RLENGTH)
  v = rest
  sub(/\].*$/, "", v)
  return v
}
function axisof(line,   rest, v) {
  if (!match(line, /:axis "/)) return "?"
  rest = substr(line, RSTART + RLENGTH)
  v = rest
  sub(/".*$/, "", v)
  return v
}
function pct(new, old) { return (old == 0) ? 0 : (new - old) / old * 100.0 }

/#grid\/Verdict/ {
  key = axisof($0) "|" sizeof($0)
  if (FILENAME == oldf) {
    seen_old[key] = 1; o_ratio[key] = field($0,"ratio"); o_wat[key] = field($0,"wat-ns")
    o_acc[key] = field($0,"accuracy"); o_win[key] = field($0,"winner"); order[++n] = key
    o_runs[field($0,"runs")] = 1
    o_lo[key] = field($0,"wat-ns-min"); o_hi[key] = field($0,"wat-ns-max")
  } else {
    seen_new[key] = 1; n_ratio[key] = field($0,"ratio"); n_wat[key] = field($0,"wat-ns")
    n_acc[key] = field($0,"accuracy"); n_win[key] = field($0,"winner")
    n_runs[field($0,"runs")] = 1
    n_lo[key] = field($0,"wat-ns-min"); n_hi[key] = field($0,"wat-ns-max")
    if (!seen_old[key]) order[++n] = key
  }
}

END {
  # ⛔ SAMPLE COUNT IS PART OF THE MEASUREMENT. Two grids taken at different GRID_RUNS are not
  # sample-comparable: run-axis.sh reports the MEAN of N runs, so a 3-run cell and a 5-run cell
  # carry different variance and a 10% delta between them may be nothing at all. Found the hour
  # this script was written — the 2026-08-27 grid is :runs 3, the 2026-08-31 grid is :runs 5, and
  # they had already been compared by eye. The check is here so nobody has to remember it.
  ro = ""; for (r in o_runs) ro = ro (ro == "" ? "" : ",") r
  rn = ""; for (r in n_runs) rn = rn (rn == "" ? "" : ",") r
  printf "OLD %s  (:runs %s)\nNEW %s  (:runs %s)\n", oldf, ro, newf, rn
  if (ro != rn) {
    printf "\n⛔ SAMPLE-COUNT MISMATCH — :runs %s vs :runs %s. These grids are NOT sample-comparable;\n", ro, rn
    printf "   a per-cell delta below mixes a variance change with an engine change. Re-run the\n"
    printf "   older side at GRID_RUNS=%s before reasoning from any number here.\n\n", rn
    runs_mismatch = 1
  } else { printf "\n" }
  printf "%-16s %-10s %12s %12s %8s   %9s %9s %8s  %s\n", \
         "AXIS","SIZE","wat-ns OLD","wat-ns NEW","Δwat%","ratio OLD","ratio NEW","Δratio%","FLAGS"
  printf "%s\n", "────────────────────────────────────────────────────────────────────────────────────────────────────────"
  worse = 0; better = 0; diverge = 0; onlyone = 0; badacc = 0
  for (i = 1; i <= n; i++) {
    key = order[i]
    split(key, k, "|"); axis = k[1]; size = k[2]
    if (!seen_old[key]) { printf "%-16s %-10s %12s %12s %8s   %9s %9s %8s  ONLY-NEW\n", axis, size, "-", n_wat[key], "-", "-", n_ratio[key], "-"; onlyone++; continue }
    if (!seen_new[key]) { printf "%-16s %-10s %12s %12s %8s   %9s %9s %8s  ONLY-OLD\n", axis, size, o_wat[key], "-", "-", o_ratio[key], "-", "-"; onlyone++; continue }

    dw = pct(n_wat[key] + 0, o_wat[key] + 0)
    dr = pct(n_ratio[key] + 0, o_ratio[key] + 0)
    flags = ""
    if (o_acc[key] != ":match" || n_acc[key] != ":match") { flags = flags "⛔ACCURACY(" o_acc[key] "→" n_acc[key] ") "; badacc++ }
    if (o_win[key] != n_win[key]) flags = flags "⛔WINNER(" o_win[key] "→" n_win[key] ") "
    # Δwat < 0 is us getting FASTER; Δratio > 0 is the verdict improving. Same direction of good.
    if ((dw < -5 && dr < -5) || (dw > 5 && dr > 5)) { flags = flags "⚠DIVERGE "; diverge++ }
    # ⛔ DISJOINT INTERVALS ARE THE ONLY SIGNAL. Not "is the delta bigger than a spread" --
    # that test FALSE-POSITIVED 3 of 33 cells on two same-build sweeps (2026-09-02), because a
    # cell whose own 5 runs happened to land tight does not thereby bound run-to-run drift. The
    # honest test is whether the two observed [min,max] ranges OVERLAP: if they do, the cell
    # cannot separate the builds, whatever the means did.
    #
    # ⛔ AND A ONE-SIDED SPREAD IS NOT A BOUND. If either side predates :wat-ns-min/:wat-ns-max
    # the cell is NO-SPREAD -- unfalsifiable -- never "clean". Every grid before 2026-09-02 is
    # that case, which is the true state of the recorded perf history in this arc.
    have_both = (o_lo[key] != "" && o_hi[key] != "" && n_lo[key] != "" && n_hi[key] != "")
    if (!have_both) { flags = flags "NO-SPREAD "; nospread++ }
    else {
      olo = o_lo[key] + 0; ohi = o_hi[key] + 0; nlo = n_lo[key] + 0; nhi = n_hi[key] + 0
      overlap = (olo <= nhi && nlo <= ohi)
      spread = (ohi - olo) / (o_wat[key] + 0) * 100.0
      nsp    = (nhi - nlo) / (n_wat[key] + 0) * 100.0
      if (nsp > spread) spread = nsp
      if (overlap) { flags = flags "≈noise(ranges overlap, ±" sprintf("%.0f", spread) "%) "; quiet++ }
      else if (dw > 0) { flags = flags "⛔SLOWER(disjoint) "; worse++ }
      else { flags = flags "⭐faster(disjoint) "; better++ }
    }
    printf "%-16s %-10s %12d %12d %+7.1f%%   %9.2f %9.2f %+7.1f%%  %s\n", \
           axis, size, o_wat[key], n_wat[key], dw, o_ratio[key], n_ratio[key], dr, flags
  }
  printf "\n"
  printf "cells: %d compared · %d only-in-one · slower BEYOND noise: %d · faster BEYOND noise: %d · within noise: %d · no spread recorded: %d · diverging: %d · accuracy flags: %d\n", \
         n - onlyone, onlyone, worse, better, quiet, nospread, diverge, badacc
  if (nospread > 0) printf "\n⚠ %d cell(s) predate :wat-ns-min/:wat-ns-max. Their Δwat cannot be told from noise by this artifact — re-run that side to make it falsifiable.\n", nospread
  if (runs_mismatch) printf "\n⛔ Re-read the SAMPLE-COUNT MISMATCH above before quoting any delta from this table.\n"
  if (badacc > 0) { printf "\n⛔ AN ACCURACY FLAG IS NOT A PERFORMANCE RESULT. Stop and read the mismatch.\n"; exit 1 }
  if (runs_mismatch) exit 3
}
' "$OLD" "$NEW"
