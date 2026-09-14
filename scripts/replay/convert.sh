#!/usr/bin/env bash
# Write each <path> at <rev>, converted through the recorded-migration chain
# by this tree's ./target/release/wat, to <out-dir>/<path>. Each codemod runs
# ONCE over that set's in-scope paths. Work happens in a temp dir plus <out-dir>;
# the tree is never written.
#
# 2a4 TWO-PHASE (the RECIPE — this script does not cargo-build):
#   A step that touches wat/*.wat (stdlib source):
#     (a) convert.sh C <out-std> wat/*.wat     # enum-fields uses declared-stdlib-types
#     (b) copy those files into the tree, cargo build --release
#     (c) convert.sh C <out-rest> <other .wat>  # rebuilt binary; type-of sees the new world
#   The C^ set uses HEAD's binary (HEAD already carries C^'s stdlib).
#
# usage: scripts/replay/convert.sh <rev> <out-dir> <path>…
set -euo pipefail

if [[ $# -lt 3 ]]; then
  echo "usage: $0 <rev> <out-dir> <path>…" >&2
  exit 2
fi
REV=$1
OUT=$2
shift 2
PATHS=("$@")

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
cd "$ROOT"
WAT="$ROOT/target/release/wat"
CHAIN="$ROOT/scripts/replay/chain-order.sh"
ORDER_BASE=de827fb4c
ORDER_MAIN=a3218644d

if [[ ! -x "$WAT" ]]; then
  echo "convert.sh: missing $WAT — cargo build --release first" >&2
  exit 2
fi

in_scope() {
  local codemod=$1
  local path=$2
  python3 - "$codemod" "$path" <<'PY'
import sys, fnmatch
codemod, path = sys.argv[1], sys.argv[2]
scope = None
with open(codemod) as f:
    for line in f:
        t = line.strip()
        if t.startswith(";;") and "SCOPE:" in t:
            rest = t.split("SCOPE:", 1)[1].strip()
            scope = rest
            break
if not scope:
    sys.exit(1)
for ent in scope.split():
    if ent == "corpus":
        if path.endswith(".wat") and not path.startswith("wat-scripts/fixes/"):
            sys.exit(0)
        continue
    if ent.endswith("/"):
        if path.startswith(ent):
            sys.exit(0)
        continue
    if fnmatch.fnmatch(path, ent):
        sys.exit(0)
sys.exit(1)
PY
}

edn_vec() {
  python3 -c 'import json,sys; print("[" + " ".join(json.dumps(p) for p in sys.argv[1:]) + "]")' "$@"
}

mkdir -p "$OUT"
ABS_OUT=$(cd "$OUT" && pwd)
declare -a WORK_PATHS=()
for p in "${PATHS[@]}"; do
  mkdir -p "$ABS_OUT/$(dirname "$p")"
  if ! git show "$REV:$p" > "$ABS_OUT/$p" 2>"$ABS_OUT/.show.err"; then
    echo "convert.sh: git show $REV:$p failed" >&2
    cat "$ABS_OUT/.show.err" >&2
    exit 2
  fi
  WORK_PATHS+=("$ABS_OUT/$p")
done

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
LOG="$TMP/log"
: > "$LOG"

mapfile -t STEPS < <("$CHAIN" "$ORDER_BASE" "$ORDER_MAIN")

# one-param-spec CONTEXT: every .wat at <rev> outside wat-scripts/fixes/,
# minus files today's reader cannot lex, each REPORTED.
CTX_LIST="$TMP/ctx.list"
: > "$CTX_LIST"
git ls-tree -r --name-only "$REV" | grep '\.wat$' | grep -v '^wat-scripts/fixes/' | sort > "$TMP/all.wat" || true
NEED_CTX=0
for cm in "${STEPS[@]}"; do
  if [[ "$(basename "$cm")" == "one-param-spec.wat" ]]; then
    for p in "${PATHS[@]}"; do
      if in_scope "$cm" "$p"; then NEED_CTX=1; break; fi
    done
  fi
done
if [[ $NEED_CTX -eq 1 ]]; then
  CTX_TREE="$TMP/ctx-tree"
  mkdir -p "$CTX_TREE"
  git archive "$REV" | tar -x -C "$CTX_TREE"
  mapfile -t CTX_CANDS < "$TMP/all.wat"
  declare -a CTX_ABS=()
  for rel in "${CTX_CANDS[@]}"; do
    CTX_ABS+=("$CTX_TREE/$rel")
  done
  cat > "$TMP/readable.wat" <<'WAT'
(:wat::core::defn :user::check-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [p (:wat::core::first paths)]
      (:wat::core::do
        (:wat::core::match (:wat::core::read-string (:wat::io::read-file p))
          [:wat::core::ReadOutcome.Forms {:forms _}
            (:wat::kernel::println (:wat::string::concat "OK " p))]
          [:wat::core::ReadOutcome.Malformed {:cause c}
            (:wat::kernel::println
              (:wat::string::concat
                "[convert.sh] UNREADABLE "
                (:wat::string::concat p (:wat::string::concat " " (:wat::core::Error/message c)))))])
        (:user::check-each (:wat::core::rest paths))))))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::check-each
    (:wat::core::match (:wat::kernel::readln)
      [:wat::kernel::ReadlnOutcome.Datum {:v v} v]
      [:wat::kernel::ReadlnOutcome.Eof {}
        (:wat::kernel::assertion-failed! :message "readln: end of input")]
      [:wat::kernel::ReadlnOutcome.Stopped {}
        (:wat::kernel::assertion-failed! :message "readln: stop requested")])))
WAT
  if [[ ${#CTX_ABS[@]} -gt 0 ]]; then
    edn_vec "${CTX_ABS[@]}" | "$WAT" "$TMP/readable.wat" > "$TMP/readable.out" 2>&1 || {
      echo "convert.sh: readable-probe failed" >&2
      cat "$TMP/readable.out" >&2
      exit 1
    }
    # wat println renders strings with surrounding quotes.
    grep -E 'UNREADABLE ' "$TMP/readable.out" | sed -e 's/^"//' -e 's/"$//' || true
    grep -E 'OK ' "$TMP/readable.out" | sed -e 's/^"OK //' -e 's/"$//' > "$CTX_LIST" || true
  fi
fi

for cm in "${STEPS[@]}"; do
  declare -a SCOPED=()
  declare -a SCOPED_REL=()
  i=0
  for p in "${PATHS[@]}"; do
    if in_scope "$cm" "$p"; then
      SCOPED+=("${WORK_PATHS[$i]}")
      SCOPED_REL+=("$p")
    fi
    i=$((i + 1))
  done
  if [[ ${#SCOPED[@]} -eq 0 ]]; then
    continue
  fi
  {
    echo "=== $cm ==="
    # 2a4b: run from the out-dir so TARGET paths are repo-relative (`wat/gen.wat`).
    # Codemod path stays absolute. one-param-spec CONTEXT may stay absolute (tmp ctx-tree).
    if [[ "$(basename "$cm")" == "one-param-spec.wat" ]]; then
      mapfile -t CTX_OK < "$CTX_LIST"
      if [[ ${#CTX_OK[@]} -eq 0 ]]; then
        (cd "$ABS_OUT" && { edn_vec "${SCOPED_REL[@]}"; edn_vec "${SCOPED_REL[@]}"; } | "$WAT" "$ROOT/$cm")
      else
        (cd "$ABS_OUT" && { edn_vec "${CTX_OK[@]}"; edn_vec "${SCOPED_REL[@]}"; } | "$WAT" "$ROOT/$cm")
      fi
    else
      (cd "$ABS_OUT" && edn_vec "${SCOPED_REL[@]}" | "$WAT" "$ROOT/$cm")
    fi
  } >> "$LOG" 2>&1 || {
    echo "convert.sh: $cm failed on $REV [${SCOPED_REL[*]}]" >&2
    cat "$LOG" >&2
    exit 1
  }
done

# Surface per-codemod println so a step log can carry UNREGISTERABLE /
# UNRESOLVED / SPLICE (UNREADABLE is already printed above).
if [[ -s "$LOG" ]]; then
  cat "$LOG"
fi
