#!/usr/bin/env bash
# The conversion DELTA gate — and the RECOVERY column that was printed by hand
# for three consecutive stones (255.9, 255.10, 255.11) and recommended for
# institutionalisation by each of them.
#
#   scripts/replay/delta.sh [--list FILE] [--binary PATH] [--out DIR] [--codemod PATH]
#
# Copies every path in the list TWICE under $OUT (`orig/` and `conv/`), runs the
# faithful-Clojure codemod on the `conv/` copy ONLY, then `--check`s BOTH sides
# **from copies** — the symmetry that matters: checking originals in the live
# tree while checking converted files from a copy manufactures phantom
# regressions on files that resolve siblings by relative path (the asymmetry
# that produced a false regression in the 255.9 weigh).
#
# Columns, all four printed every run:
#
#   ORIG-CLEAN   files whose ORIGINAL spelling checks rc 0
#   CONV-CLEAN   files whose CONVERTED spelling checks rc 0
#   NEW          orig rc 0 -> conv rc != 0   (the conversion BROKE it)  = the delta
#   RECOVERY     orig rc != 0 -> conv rc 0   (the conversion FIXED it)
#
# ⛔ RECOVERY IS A STOP, NOT A CELEBRATION, AND IT IS THE ONLY COLUMN THAT CAN
# SEE A GREEN-FORGING CONVERSION. A cure that makes a checker stop refusing shows
# up here and NOWHERE else: NEW cannot fall below zero, so a wall that was
# disarmed reads as "delta unchanged". 255.9 caught exactly that with this column
# computed by hand. Non-zero RECOVERY exits 9 and must be explained file by file
# before the stone lands.
#
# Exit: 0 = ran, RECOVERY 0. 9 = RECOVERY non-zero (STOP). 1 = usage/setup error.
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
cd "$ROOT"

LIST="$ROOT/docs/arc/2026/06/251-types-as-forms/delta-sample-179.txt"
WAT="$ROOT/target/release/wat"
CODEMOD="$ROOT/wat-scripts/fixes/to-faithful-clojure.wat"
OUT=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --list)    LIST=$2; shift 2 ;;
    --binary)  WAT=$2; shift 2 ;;
    --out)     OUT=$2; shift 2 ;;
    --codemod) CODEMOD=$2; shift 2 ;;
    -h|--help) sed -n '2,28p' "$0"; exit 0 ;;
    *) echo "delta.sh: unknown argument '$1'" >&2; exit 1 ;;
  esac
done

[[ -x "$WAT"      ]] || { echo "delta.sh: no wat binary at $WAT (cargo build --release)" >&2; exit 1; }
[[ -f "$LIST"     ]] || { echo "delta.sh: no list at $LIST" >&2; exit 1; }
[[ -f "$CODEMOD"  ]] || { echo "delta.sh: no codemod at $CODEMOD" >&2; exit 1; }

STAMP=$(date -u +%Y-%m-%dT%H-%M-%SZ)
[[ -n "$OUT" ]] || OUT="$ROOT/.delta/$STAMP"
mkdir -p "$OUT/orig" "$OUT/conv"

echo "[delta] list     $LIST"
echo "[delta] list-sha $(sha256sum "$LIST" | awk '{print $1}')"
echo "[delta] binary   $WAT"
echo "[delta] out      $OUT"

# ── copy both sides identically ────────────────────────────────────────────
missing=0
n=0
while read -r p; do
  [[ -n "$p" ]] || continue
  n=$((n + 1))
  if [[ ! -f "$ROOT/$p" ]]; then
    echo "[delta] MISSING $p"
    missing=$((missing + 1))
    continue
  fi
  mkdir -p "$OUT/orig/$(dirname "$p")" "$OUT/conv/$(dirname "$p")"
  cp "$ROOT/$p" "$OUT/orig/$p"
  cp "$ROOT/$p" "$OUT/conv/$p"
done < "$LIST"
echo "[delta] paths=$n missing=$missing"
if [[ $missing -ne 0 ]]; then
  echo "[delta] ⛔ the committed list does not match the tree — fix the list or the tree, do not paper over it" >&2
  exit 1
fi

# ── convert the conv/ side ONLY ────────────────────────────────────────────
{
  printf '['
  first=1
  while read -r p; do
    [[ -n "$p" ]] || continue
    [[ $first -eq 1 ]] || printf ' '
    first=0
    printf '"%s"' "$OUT/conv/$p"
  done < "$LIST"
  printf ']\n'
} > "$OUT/paths.edn"

"$WAT" "$CODEMOD" < "$OUT/paths.edn" > "$OUT/codemod.log" 2>&1 || {
  echo "[delta] ⛔ codemod exited non-zero — see $OUT/codemod.log" >&2
  exit 1
}

# ── check BOTH sides, from copies, same way ────────────────────────────────
# ⚠ Each check runs with stdin on /dev/null. `wat` READS STDIN, so a plain
# `while read … done < "$LIST"` loop has its list eaten by the first file that
# blocks on a read — the first draft of this script silently measured 10 of 179
# and reported a clean delta. xargs already hands each child /dev/null; the
# explicit redirect inside the worker is belt-and-braces for anyone who
# reshapes this loop.
PARTS="$OUT/parts"
rm -rf "$PARTS"; mkdir -p "$PARTS"
sed '/^$/d' "$LIST" | xargs -P16 -I{} bash -c '
  p="$1"; out="$2"; wat="$3"; parts="$4"
  "$wat" --check "$out/orig/$p" >/dev/null 2>&1 </dev/null; a=$?
  "$wat" --check "$out/conv/$p" >/dev/null 2>&1 </dev/null; b=$?
  h=$(printf "%s" "$p" | sha256sum | awk "{print \$1}")
  printf "%s\t%s\t%s\n" "$p" "$a" "$b" > "$parts/$h"
' _ {} "$OUT" "$WAT" "$PARTS"
cat "$PARTS"/* > "$OUT/results.tsv.unsorted"
# Re-emit in LIST order so two runs diff line-for-line.
: > "$OUT/results.tsv"
while read -r p; do
  [[ -n "$p" ]] || continue
  grep -F -m1 -- "$(printf '%s\t' "$p")" "$OUT/results.tsv.unsorted" >> "$OUT/results.tsv"
done < "$LIST"
rm -f "$OUT/results.tsv.unsorted"
got=$(wc -l < "$OUT/results.tsv")
[[ "$got" -eq "$n" ]] || { echo "[delta] ⛔ measured $got of $n files — refusing to report a partial delta" >&2; exit 1; }

ORIG_CLEAN=$(awk -F'\t' '$2==0' "$OUT/results.tsv" | wc -l)
CONV_CLEAN=$(awk -F'\t' '$3==0' "$OUT/results.tsv" | wc -l)
awk -F'\t' '$2==0 && $3!=0 {print $1}' "$OUT/results.tsv" > "$OUT/new.txt"
awk -F'\t' '$2!=0 && $3==0 {print $1}' "$OUT/results.tsv" > "$OUT/recovery.txt"
NEW=$(wc -l < "$OUT/new.txt")
RECOVERY=$(wc -l < "$OUT/recovery.txt")

echo
printf '  ORIG-CLEAN  %s/%s\n' "$ORIG_CLEAN" "$n"
printf '  CONV-CLEAN  %s/%s\n' "$CONV_CLEAN" "$n"
printf '  NEW         %s   (orig clean -> converted broken)\n' "$NEW"
printf '  RECOVERY    %s   (orig broken -> converted clean)\n' "$RECOVERY"
echo
[[ $NEW -eq 0 ]] || { echo "NEW files:"; sed 's/^/  /' "$OUT/new.txt"; }
if [[ $RECOVERY -ne 0 ]]; then
  echo "⛔ RECOVERY files — STOP until each is explained:"
  sed 's/^/  /' "$OUT/recovery.txt"
  echo "[delta] exit=9 (RECOVERY non-zero). Results kept at $OUT/"
  exit 9
fi
echo "[delta] exit=0. RECOVERY 0. Results kept at $OUT/ regardless — a clean run is evidence too."
