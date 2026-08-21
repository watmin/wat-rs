#!/usr/bin/env bash
# PROBE for DESIGN-STONE-255.1c-io. Answers ONE question behaviourally, with both
# controls, without a build: is each :wat::io:: verb enforced by the checker, or does
# it fall through `check.rs`'s blanket-accept the way `peer-pid` does (#110)?
#
# Method: hand every verb SEVEN arguments. A scheme-enforced verb rejects with
# ArityMismatch. A blanket-accepted verb check-PASSES (exit 0).
#
# ⚠ The generated programs are deliberately ILLEGAL, so they must NOT live in
# `wat-scripts/scratch-pad/` — `every_wat_scripts_file_loads` type-checks that tree
# and a program that cannot check would take it RED. They are written to a temp dir.
#
# RESULT, 2026-08-20, HEAD 4160b12a:
#   NEGATIVE CONTROL  peer-pid + 5 args ....... EXIT 0, no ArityMismatch  (blanket-accept)
#   28 of 29 io verbs ......................... ArityMismatch             (plain TypeScheme)
#    1 of 29  IOReader/read-frame ............. MalformedForm, "expected 1 or 2 args"
#                                               (bespoke `infer_ioreader_read_frame`,
#                                                check.rs:2969, intercepts BEFORE the
#                                                scheme registered at check.rs:15794)
set -u
WAT=${WAT:-./target/release/wat}
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
probe() { # $1 = verb, $2 = argcount
  local args=""; for i in $(seq 1 "$2"); do args="$args $i"; done
  printf '(:wat::core::defn :user::main [] -> :wat::core::nil\n  (:wat::core::let [x (%s%s)] nil))\n' "$1" "$args" > "$TMP/p.wat"
  timeout 30 "$WAT" --check "$TMP/p.wat" 2>&1
}

echo "=== NEGATIVE CONTROL — :wat::kernel::peer-pid has no scheme (#110) ==="
out=$(probe ':wat::kernel::peer-pid' 5)
if echo "$out" | grep -q ArityMismatch; then echo "  ⛔ CONTROL BROKE: peer-pid now rejects — #110 may have closed. Re-read this probe."
else echo "  ✓ check PASSES 5 args — the blanket-accept, demonstrated."; fi

echo; echo "=== every :wat::io:: verb, 7 args ==="
enforced=0; fellthrough=0; bespoke=0
# ⚠ THE POPULATION MUST NOT SHRINK AS THE CARVE PROCEEDS. An earlier version of this
# probe enumerated ONLY `runtime.rs` dispatch arms — the exact thing each stone DELETES.
# After 255.1c-io-reader it silently measured 19 verbs instead of 29 and still printed a
# reassuring "0 fell through". A probe whose population is drained by the work it audits
# reports success by measuring nothing. Enumerate the UNION of both homes: arms still in
# `runtime.rs` PLUS names already registered via `#[wat_intrinsic]`.
VERBS=$( { grep -ohE '":wat::io::[^"]+" *=>' src/runtime.rs | sed 's/" *=>//; s/^"//'
           grep -rohE '#\[wat_intrinsic\("(:wat::io::[^"]+)"' src/intrinsic/ | sed 's/.*("//; s/"$//'
         } | sort -u )
count=$(echo "$VERBS" | grep -c .)
echo "  population: $count verbs (arms in runtime.rs + registered intrinsics)"
if [ "$count" -lt 29 ]; then echo "  ⛔ POPULATION SHRANK below 29 — the enumerator is losing verbs, not the corpus."; fi
for v in $VERBS; do
  out=$(probe "$v" 7)
  if   echo "$out" | grep -q ArityMismatch;  then enforced=$((enforced+1))
  elif echo "$out" | grep -q MalformedForm;  then bespoke=$((bespoke+1));  echo "  ⚠ bespoke arm: $v"
  else fellthrough=$((fellthrough+1));            echo "  ⛔ FELL THROUGH: $v"; fi
done
echo "---"
echo "  scheme-enforced (ArityMismatch) : $enforced"
echo "  bespoke infer arm (Malformed)   : $bespoke"
echo "  blanket-accepted (fell through) : $fellthrough   ← must be 0"
