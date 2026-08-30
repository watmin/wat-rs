#!/usr/bin/env bash
# Doc coverage for Rust sources: which functions of >= N lines carry a doc comment.
#
# WHY THIS IS COMMITTED. A doc-coverage number was recorded in an arc breadcrumb with no
# instrument beside it, and the number was WRONG in a way nobody could see: it read only the line
# directly above `fn`, so every function whose doc sits above an attribute — `#[allow(...)]`,
# `#[inline]`, `#[cfg(test)]` — counted as undocumented. That mis-measurement named
# `fire/pass/hash_join.rs` as the worst file in `src/rete` when in fact all four of its functions
# are documented, three of them behind `#[allow(clippy::too_many_arguments)]`.
#
# A metric with no committed instrument is unfalsifiable: it cannot be re-derived, so it rots
# silently and is quoted for weeks. If you record a doc-coverage figure anywhere, cite this
# script and the flags you ran it with, so the next reader can reproduce or refute it.
#
# DEFINITION (the thing that was wrong before). Walking UPWARD from the `fn` line, skipping
# attributes and plain `//` comments, a function is DOCUMENTED if the first line reached is `///`
# or `//!`. A blank line stops the walk — a doc separated from its item by a blank line is not
# attached to it.
#
# Usage:
#   scripts/doc-coverage.sh src/rete                # summary + per-file undocumented counts
#   scripts/doc-coverage.sh src/rete --min 15       # only functions >= 15 lines (default 15)
#   scripts/doc-coverage.sh src/rete --list         # every undocumented function, file:line
#   scripts/doc-coverage.sh src/rete --exclude /tests/    # skip any PATH containing this
#
# NOTE on comparing directories. The summary also reports total lines and comment density, so
# the whole "is this dir an exemplar" table is derivable from ONE committed instrument. When
# comparing a dir against a sibling, say what you excluded: `src/rete` carries a 10k-line
# `kernel/tests/` module that its siblings have no equivalent of, so an unqualified line count
# compares test bulk rather than code. An earlier recorded table used that exclusion silently,
# and the number could not be reproduced until someone guessed it.
set -euo pipefail

root="${1:?usage: doc-coverage.sh <dir-or-file> [--min N] [--list]}"; shift || true
min=15; list=0; exclude=""
while [ $# -gt 0 ]; do
  case "$1" in
    --min) min="$2"; shift 2 ;;
    --list) list=1; shift ;;
    --exclude) exclude="$2"; shift 2 ;;
    *) echo "unknown flag: $1" >&2; exit 2 ;;
  esac
done

prog='
{ raw[NR] = $0 }
END {
  for (i = 1; i <= NR; i++) {
    line = raw[i]
    if (line !~ /^[[:space:]]*(pub(\([a-z]+\))?[[:space:]]+)?(const[[:space:]]+)?(async[[:space:]]+)?(unsafe[[:space:]]+)?(extern[[:space:]]+"[^"]*"[[:space:]]+)?fn[[:space:]]+[A-Za-z_]/) continue
    nm = "?"
    if (match(line, /fn[[:space:]]+[A-Za-z_][A-Za-z0-9_]*/)) {
      nm = substr(line, RSTART + 2, RLENGTH - 2); gsub(/^[[:space:]]+/, "", nm)
    }
    documented = 0; j = i - 1
    while (j >= 1) {
      p = raw[j]; gsub(/^[[:space:]]+|[[:space:]]+$/, "", p)
      if (p ~ /^(\/\/\/|\/\/!)/)     { documented = 1; break }
      if (p ~ /^#\[/ || p ~ /^#!\[/) { j--; continue }
      if (p ~ /^\/\//)               { j--; continue }
      if (p == "")                   { break }
      # tail line of a multi-line attribute
      if (p ~ /^[])}][,)]?$/ || p ~ /^[A-Za-z_:<>&, ]+[,)]$/) { j--; continue }
      break
    }
    d = 0; started = 0; endl = i
    for (k = i; k <= NR; k++) {
      s = raw[k]; sub(/\/\/.*$/, "", s)
      no = gsub(/\{/, "", s); nc = gsub(/\}/, "", s)
      d += no - nc
      if (no > 0) started = 1
      if (started && d <= 0) { endl = k; break }
    }
    is_test = 0
    for (t = i - 1; t >= 1 && t >= i - 8; t--) {
      q = raw[t]; gsub(/^[[:space:]]+|[[:space:]]+$/, "", q)
      if (q ~ /^#\[(test|wat_test)/) { is_test = 1; break }
      if (q !~ /^#\[/ && q !~ /^\/\// && q != "") break
    }
    printf "%s\t%s\t%d\t%d\t%s\t%s\n", (documented ? "DOC" : "UNDOC"), FILE, i, endl - i + 1, nm, (is_test ? "TEST" : "FN")
  }
}'

tmp="$(mktemp)"; trap 'rm -f "$tmp"' EXIT
files="$(mktemp)"; trap 'rm -f "$tmp" "$files"' EXIT
if [ -d "$root" ]; then
  if [ -n "$exclude" ]; then
    # PATH-fragment match, deliberately not `find -name`. A basename filter silently stops
    # matching the moment the target becomes a directory: `--exclude tests.rs` was written when
    # `kernel/tests.rs` was one file, and when it became `kernel/tests/*.rs` (2026-08-30) the
    # flag matched NOTHING and the reported figure moved with no error and no warning.
    find "$root" -name '*.rs' | grep -v -- "$exclude" | sort > "$files"
  else
    find "$root" -name '*.rs' | sort > "$files"
  fi
else
  echo "$root" > "$files"
fi
while read -r f; do awk -v FILE="$f" "$prog" "$f"; done < "$files" > "$tmp"

if [ "$list" = 1 ]; then
  awk -F'\t' -v m="$min" '$1=="UNDOC" && $4>=m {printf "%s:%s\t%s ln\t%-6s %s\n",$2,$3,$4,$6,$5}' "$tmp"
  exit 0
fi

total=$(awk -F'\t' -v m="$min" '$4>=m && $6=="FN"' "$tmp" | wc -l)
undoc=$(awk -F'\t' -v m="$min" '$1=="UNDOC" && $4>=m && $6=="FN"' "$tmp" | wc -l)
undoc_t=$(awk -F'\t' -v m="$min" '$1=="UNDOC" && $4>=m && $6=="TEST"' "$tmp" | wc -l)
echo "$root — non-#[test] functions >= $min lines: $total, undocumented: $undoc ($(( total ? undoc*100/total : 0 ))%)"
# Counted apart on purpose: a #[test] whose name is a sentence already states its contract, and
# folding those into one figure inflates it. Report both; never quote only the sum.
# NOTE the split is exactly "carries #[test]" — a helper living inside `mod tests` counts as FN,
# because that is what the walk can actually see. Do not read FN as "production".
echo "  (plus $undoc_t undocumented #[test] fns, excluded from the figure above)"
lines=$(xargs cat < "$files" | wc -l)
com=$(xargs cat < "$files" | grep -c '^[[:space:]]*//')
echo "  lines: $lines, comment lines: $com ($(( lines ? com*100/lines : 0 ))%)${exclude:+  [excluding $exclude]}"
echo
awk -F'\t' -v m="$min" '$1=="UNDOC" && $4>=m && $6=="FN" {print $2}' "$tmp" | sort | uniq -c | sort -rn
