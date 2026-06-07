#!/usr/bin/env bash
# gen-test-mods.sh — auto-maintain the `mod` lists of grouped [[test]] dirs.
#
# Each grouped test dir (tests/<group>/mod.rs) that OPTS IN with the marker pair
#   // BEGIN GENERATED MODS ...   /   // END GENERATED MODS
# has its module list generated from the dir's *.rs files. This makes silently
# ignoring a test file IMPOSSIBLE: drop a .rs in the dir, run the generator, and
# it is declared → compiled → run → coverage-counted. The --check gate (wired
# into green-gate.sh) fails LOUD if anyone forgets, so a file can never sit in a
# grouped dir un-compiled. Self-enforcing completeness — the project's own idiom
# (cf. vigilatum, green-gate, the .scopes()-reader gate, grimoire --check).
#
# Usage:
#   scripts/gen-test-mods.sh           # rewrite the generated blocks in place
#   scripts/gen-test-mods.sh --check   # exit 1 if any block is stale (gate mode)
#
# Standard enforced: declared mods == *.rs files (set equality), sorted.
set -euo pipefail
cd "$(dirname "$0")/.."

B='// BEGIN GENERATED MODS'
E='// END GENERATED MODS'
check=0; [[ "${1:-}" == "--check" ]] && check=1
drift=0; any=0

for modrs in tests/*/mod.rs; do
  grep -qF "$B" "$modrs" || continue          # opt-in only
  any=1
  dir=$(dirname "$modrs")

  # the .rs files that SHOULD be declared (everything but mod.rs), sorted
  actual=$(ls "$dir"/*.rs 2>/dev/null | grep -v '/mod\.rs$' | xargs -n1 basename | sed 's/\.rs$//' | sort)

  if [[ "$check" == "1" ]]; then
    declared=$(awk -v b="$B" -v e="$E" 'index($0,b)==1{f=1;next} index($0,e)==1{f=0} f' "$modrs" \
               | sed -n 's/^mod \([A-Za-z0-9_]*\);.*$/\1/p' | sort)
    if [[ "$declared" != "$actual" ]]; then
      echo "DRIFT: $modrs (declared mods != *.rs files):" >&2
      echo "  '<' = declared-but-missing-file   '>' = file-but-undeclared (would be IGNORED)" >&2
      comm -3 <(printf '%s\n' "$declared") <(printf '%s\n' "$actual") >&2 || true
      drift=1
    fi
  else
    { awk -v b="$B" '{print} index($0,b)==1{exit}' "$modrs"
      printf '%s\n' "$actual" | sed '/^$/d; s/^/mod /; s/$/;/'
      awk -v e="$E" 'f{print} index($0,e)==1{f=1;print}' "$modrs"
    } > "$modrs.tmp" && mv "$modrs.tmp" "$modrs"
    echo "wrote $modrs ($(printf '%s\n' "$actual" | sed '/^$/d' | wc -l) modules)"
  fi
done

[[ "$any" == "0" ]] && { echo "no opt-in grouped test dirs (marker '$B' not found)" >&2; }

if [[ "$check" == "1" ]]; then
  [[ "$drift" == "1" ]] && { echo "STALE test-group mod.rs — run: scripts/gen-test-mods.sh" >&2; exit 1; }
  echo "test-group mod.rs all current ✓"
fi
