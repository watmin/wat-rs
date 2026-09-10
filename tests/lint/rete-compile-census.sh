#!/usr/bin/env bash
# Census — WHICH CORPUS FILES DECLARE RETE RULES THAT CANNOT COMPILE?
#
# `every_wat_scripts_file_loads` (tests/lint/wat_scripts_fixes_load.rs) LOADS every .wat under
# wat-scripts/. Loading runs validate; it never runs rete COMPILE, where the four-axis fence/:then
# gate lives (`wat/rete/compile.wat:463` — pure AND det AND total AND rete-primitive). A rule that
# violates any axis loads green forever and dies the moment anything compiles it.
#
# This walks every file that DECLARES a rule, neutralises its own `:user::main` (so no codemod
# side-effects run), appends a driver that calls compile-all on each declared namespace, and
# classifies the result.
#
# ⛔ TWO TRAPS THIS SCRIPT WAS BUILT AROUND, both of which produced a WRONG number first:
#   1. `rc` IS USELESS HERE. A rete compile AssertionFailure still exits 0. Classify on OUTPUT.
#   2. NAMESPACE EXTRACTION MUST NOT BE GREEDY. `sed 's/.*: *://'` on "rete::defrule :fix::g3"
#      yields "g3", so the driver collects ZERO rules and "compiles" vacuously — the first run of
#      this census reported 136/136 OK for exactly that reason. Anchor on a known positive.
#
# Usage: tests/lint/rete-compile-census.sh [<binary>]   (run from the repo root)
set -u
BIN="${1:-./target/release/wat}"
WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT
# ⛔ THE ANCHOR IS SYNTHESIZED, NOT FOUND IN THE CORPUS — AND THAT IS THE WHOLE LESSON.
# This pinned `wat-scripts/fixes/to-faithful-clojure-net.wat` as its known-positive, on the
# reasoning that an instrument which cannot find a defect you already know about is not an
# instrument. Correct discipline, pinned to a MUTABLE fact: `strike-no-rule-that-cannot-compile`
# repaired that file on 2026-09-10 and this script began refusing to run at all, permanently,
# because its anchor asserted the very defect the strike existed to cure. An anchor must be
# something you CONSTRUCT, so curing the corpus can never disarm the instrument that measured it.
anchor_file() {
  cat > "$WORK/__anchor.wat" <<'ANCHORWAT'
(:wat::core::defrecord :anchor::N [k <- :wat::core::i64])
;; `:wat::core::=` is a GENERIC core op, not a `:wat::rete::` primitive, so the four-axis fence
;; (wat/rete/compile.wat:463) must refuse it on the is-rete axis. Load-time validate does NOT
;; refuse it — a CoreGeneric head inside a fence is parked expressivity — so this file reaches
;; compile, which is exactly the property this census exists to measure.
(:wat::rete::defrule :anchor::must-not-compile
  :when [(:anchor::N (?k <- :k))
         (:anchor::N (?j <- :k))
         (:wat::rete::where (:wat::core::= ?k ?j))]
  :then [])
ANCHORWAT
  echo "$WORK/__anchor.wat"
}

run_one() {
  local f="$1" base out nss
  base="$(echo "$f" | tr '/' '_')"
  sed 's/:user::main/:user::orig-main-off/g' "$f" > "$WORK/$base"
  # NOT greedy: strip through the `defrule `/`defquery ` token, then drop the final segment.
  nss=$(grep -oh 'rete::def\(rule\|query\) :[a-zA-Z0-9_:-]*' "$f" \
        | sed 's/^.*def[a-z]* ://' | sed 's/::[^:]*$//' | sort -u)
  if [ -z "$nss" ]; then echo "NO-RULES"; return; fi
  { echo; echo '(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::core::do'
    for ns in $nss; do
      echo "  (:wat::core::match (:wat::rete::compile-all (:wat::rete::collect-rules :$ns) (:wat::core::PersistentVector))"
      echo "    ((:wat::rete::CompileOutcome::Compiled __s) (:wat::kernel::println \"CENSUS-OK\"))"
      echo "    ((:wat::rete::CompileOutcome::MayNotTerminate __r __f) (:wat::kernel::println \"CENSUS-MNT\")))"
    done
    echo '))'; } >> "$WORK/$base"
  out=$(timeout 60 "$BIN" "$WORK/$base" </dev/null 2>&1)
  if   echo "$out" | grep -q 'AssertionFailure\|Error\|error'; then printf 'FAIL\t%s\n' "$(echo "$out" | grep -o ':message "[^"]*"' | head -1)"
  elif echo "$out" | grep -q 'CENSUS-OK\|CENSUS-MNT'; then echo "OK"
  else echo "EMPTY"; fi
}

# ── anchor: an instrument that cannot find the known defect is not an instrument ──
if [ "$(run_one "$(anchor_file)" | cut -f1)" != "FAIL" ]; then
  echo "⛔ ANCHOR FAILED: the synthesized fence (a generic :wat::core::= head) must not compile." >&2
  echo "   The census is not trustworthy; fix the instrument before reading any total." >&2
  exit 2
fi

ok=0; fail=0; norules=0
for f in $(grep -rl 'rete::defrule\|rete::defquery' wat-scripts --include='*.wat' | sort); do
  r="$(run_one "$f")"
  case "${r%%$'\t'*}" in
    OK)       ok=$((ok+1)) ;;
    NO-RULES) norules=$((norules+1)) ;;
    *)        fail=$((fail+1)); printf 'FAIL  %s\n      %s\n' "$f" "${r#*$'\t'}" ;;
  esac
done
printf '\ncompiles: %s   cannot compile: %s   declares no rules of its own: %s\n' "$ok" "$fail" "$norules"
