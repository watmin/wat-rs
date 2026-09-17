#!/usr/bin/env bash
# check-grid-three-way.sh — THE THIRD PAIRING, at LOW VOLUME. Every sized grid axis, three ways:
#
#   Clara 0.24.0  |  wat-oracle (fire-rules$oracle)  |  wat-native (fire-rules)
#
# and the three pairings diagnose three DIFFERENT faults (run-axis.sh:291-296):
#
#   oracle != clara   =>  the SPEC is wrong        (the pairing NOTHING has ever run)
#   native != clara   =>  the fast path is wrong
#   oracle != native  =>  a PORT bug               (also gated, JVM-free, by
#                                                   tests/rete/wat_scripts_grid_port_check.rs)
#
# ── ⛔ WHY THIS RUNS AT CORRECTNESS SIZES AND NOT ON THE PERF LADDER ────────────────────────────
#
# The `oracle` is the interpreted spec; on `run-all.sh`'s LADDER it takes hours, which is why the
# third pairing sat unrun for the whole arc and why every one of the 47 recorded `GRID-*.txt` was
# produced under `GRID_SKIP_ORACLE=1` (0 of 47 carry `:oracle-accuracy`). Builder's ruling,
# 2026-09-03: *"clara vs wat native is the typical measurement — wat oracle vs wat native needs to
# use low volume tests so we don't waste hours"*. So this script uses the CORRECTNESS sizes — the
# same table `wat_scripts_grid_port_check.rs` uses — and the whole three-way costs seconds.
#
# ⛔ These sizes are NOT the perf ladder and must never drift toward it, nor it toward them. This
# script compares SETS. It reports no ratio, no winner and no nanoseconds: `:clara-ns` and the
# speed verdict belong to `run-all.sh`, which is a different instrument answering a different
# question.
#
# ── ONE JVM, ALL AXES ──────────────────────────────────────────────────────────────────────────
#
# `check-where-shapes.sh:18-23` measured the tax: a JVM cold boot + Clojure load + Clara compile is
# ~3 s and the fire is microseconds, so 38 families through a per-row runner cost ~67 s for six of
# them and 3.7 s for all thirty-eight in one JVM. The same applies here: every axis's Clara program
# is staged into ONE temp directory and `require`d into a SINGLE JVM, which is driven once.
#
# ── DISCOVERY, NOT A LIST ──────────────────────────────────────────────────────────────────────
#
# The directory is WALKED for sized (non-`where-*`) axes and the walk is held against the size
# table by EXACT SET EQUALITY in both directions. A new axis cannot land without a deliberate size,
# and a deleted one cannot vanish quietly. An axis whose Clara program is missing, ambiguous, or
# unreadable is a HARD FAILURE — never a skip. A silently skipped axis is how a corpus goes dark
# while the light stays green, and it is the failure this arc keeps re-finding.
#
# ── EQUALITY IS SATISFIED BY ABSENCE ───────────────────────────────────────────────────────────
#
# An empty set compares equal to an empty set and reports agreement while proving nothing. Each
# axis therefore clears its guards IN THIS ORDER, which is the order `wat_scripts_grid_port_check`
# learned under mutation (an anti-vacuity instrument must never pre-empt a correctness verdict):
#
#   0. the axis's source still CALLS the oracle verb, and each vector field matches EXACTLY ONCE
#      (two `:derived`-shaped fields on one line would make the comparison ambiguous)
#   1. the echoed `:size` equals the size we sent, on BOTH the wat and the Clara line
#   2. none of the three sets is empty
#   3. THE THREE PAIRINGS, each named separately, printing both sets and their symmetric difference
#
#   check-grid-three-way.sh                    # every sized axis
#   check-grid-three-way.sh parametric-erasure # one
#
# JDK: PATH, else JAVA_HOME, else $HOME/opt/jdk-*/bin/java.
set -euo pipefail

GRID_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$GRID_DIR/../../.." && pwd)"
ONLY="${1:-}"

WAT_BIN="${WAT_BIN:-$REPO_ROOT/target/release/wat}"
[ -x "$WAT_BIN" ] || {
  echo "check-grid-three-way: no wat binary at $WAT_BIN — cargo build --release" >&2
  exit 1
}

# ── THE CORRECTNESS SIZES ──────────────────────────────────────────────────────────────────────
# The same table as `tests/rete/wat_scripts_grid_port_check.rs::CORRECTNESS_SIZES`. Each is the
# smallest size that makes that axis's derived set structurally interesting; the derivation of the
# element count each one predicts is documented there, beside the shape formula it comes from.
declare -A SIZES=(
  [accum]="10 20"
  [asym-join]="100"
  [deep-cascade]="5 20"
  [fanout]="500"
  [leading-exists]="20"
  [min-finding]="100 3"
  [negation]="50"
  [neg-consumer]="50"
  [node-share]="10 20"
  [parametric-erasure]="200"
  [strat-neg]="3 50"
  [user-reduce]="5 20"
)

# The oracle verb every sized axis must still call. A source that no longer calls it cannot produce
# an oracle column at all, and a rewrite that redirects it makes that column a mirror of native —
# the `X == X` shape this whole family of gates exists to refuse.
ORACLE_VERB=':wat::rete::fire-rules$oracle'

find_java() {
  if command -v java >/dev/null 2>&1; then return 0; fi
  if [ -n "${JAVA_HOME:-}" ] && [ -x "$JAVA_HOME/bin/java" ]; then
    export PATH="$JAVA_HOME/bin:$PATH"; return 0
  fi
  local j
  for j in "$HOME"/opt/jdk-*/bin/java; do
    [ -x "$j" ] || continue
    JAVA_HOME="$(cd "$(dirname "$j")/.." && pwd)"; export JAVA_HOME
    export PATH="$JAVA_HOME/bin:$PATH"; return 0
  done
  echo "check-grid-three-way: no java (PATH, JAVA_HOME, or \$HOME/opt/jdk-*)" >&2
  return 1
}
find_java || exit 1

# `:paths ["."]` so the staged axis programs are `require`-able by namespace from the temp dir.
CLARA_DEP='{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}} :paths ["."]}'
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

T0=$(date +%s)

# ── DISCOVERY, reconciled against the table in BOTH directions ─────────────────────────────────
DISCOVERED=()
for wat in "$GRID_DIR"/*.wat; do
  [ -e "$wat" ] || continue
  stem="$(basename "$wat" .wat)"
  case "$stem" in where-*) continue ;; esac
  DISCOVERED+=("$stem")
done
if [ "${#DISCOVERED[@]}" -eq 0 ]; then
  echo "check-grid-three-way: the walk of $GRID_DIR found NO sized axes — the glob went blind, and" >&2
  echo "  every check below would then pass over nothing. Fix the walk, not this assertion." >&2
  exit 1
fi

MISSING=()
for stem in "${DISCOVERED[@]}"; do
  [ -n "${SIZES[$stem]:-}" ] || MISSING+=("$stem")
done
if [ "${#MISSING[@]}" -gt 0 ]; then
  echo "check-grid-three-way: ${#MISSING[@]} sized axis/axes on disk have NO correctness size here:" >&2
  for m in "${MISSING[@]}"; do echo "    $m   ($m.wat exists, SIZES has no row)" >&2; done
  echo "  Add a deliberate size — and add it to CORRECTNESS_SIZES in" >&2
  echo "  tests/rete/wat_scripts_grid_port_check.rs too, which walks the same population." >&2
  exit 1
fi
STALE=()
for stem in "${!SIZES[@]}"; do
  [ -f "$GRID_DIR/$stem.wat" ] || STALE+=("$stem")
done
if [ "${#STALE[@]}" -gt 0 ]; then
  echo "check-grid-three-way: ${#STALE[@]} SIZES row(s) name an axis with no .wat on disk:" >&2
  for s in "${STALE[@]}"; do echo "    $s" >&2; done
  exit 1
fi

# ── STAGE every axis's Clara program into ONE directory ────────────────────────────────────────
# Two legitimate provenances, and EXACTLY ONE must apply per axis:
#   * `gen-<axis>.sh SIZE` — the eleven perf axes, whose Clara side is generated per size.
#   * `<axis>.clj`         — a STATIC twin, for a correctness-only axis that must not become a
#                            perf axis (a `gen-` script is what `run-all.sh:81-87` discovers, and
#                            `:89-99` then exits 2 for a rung-less one).
# Neither is a hard failure; BOTH is a hard failure, because then it is undefined which one ran.
AXES=()
for stem in "${DISCOVERED[@]}"; do
  if [ -n "$ONLY" ] && [ "$stem" != "$ONLY" ]; then continue; fi
  size="${SIZES[$stem]}"
  gen="$GRID_DIR/gen-$stem.sh"
  clj="$GRID_DIR/$stem.clj"
  file="$(tr '-' '_' <<<"$stem").clj"

  if [ -f "$gen" ] && [ -f "$clj" ]; then
    echo "[$stem] has BOTH gen-$stem.sh AND $stem.clj — it is undefined which is the twin" >&2
    exit 1
  elif [ -f "$gen" ]; then
    # shellcheck disable=SC2086
    if ! bash "$gen" $size > "$STAGE/$file" 2> "$STAGE/$stem.gen.err"; then
      echo "[$stem] gen-$stem.sh $size FAILED:" >&2
      cat "$STAGE/$stem.gen.err" >&2
      exit 1
    fi
  elif [ -f "$clj" ]; then
    cp "$clj" "$STAGE/$file"
  else
    echo "[$stem] has NO Clara twin — neither gen-$stem.sh nor $stem.clj. A three-way with two" >&2
    echo "  engines is not a three-way; author the twin or delete the axis. Skipping it here is" >&2
    echo "  how an axis goes dark while the verdict stays green." >&2
    exit 1
  fi
  printf '%s %s\n' "$stem" "$size" >> "$STAGE/axes.txt"
  AXES+=("$stem")
done

if [ "${#AXES[@]}" -eq 0 ]; then
  echo "check-grid-three-way: matched NO axes${ONLY:+ for '$ONLY'}" >&2
  exit 1
fi

# ── DRIVE ALL OF THEM IN ONE JVM ───────────────────────────────────────────────────────────────
# Each staged program prints one `#grid/Result` line carrying its own `:axis`, so rows are
# attributed by NAME and the driver needs no ordering contract. A throw is caught per axis and
# reported as a `#grid/Error` row rather than taking the other eleven down with it.
cat > "$STAGE/drive.clj" <<'DRIVER'
(doseq [line (remove clojure.string/blank? (clojure.string/split-lines (slurp "axes.txt")))]
  (let [parts (clojure.string/split (clojure.string/trim line) #"\s+")
        ax    (first parts)
        args  (rest parts)]
    (try
      (require (symbol ax))
      (apply (resolve (symbol ax "-main")) args)
      (catch Throwable t
        (binding [*out* *err*]
          (println (str "--- " ax " ---"))
          (.printStackTrace t))
        (println (str "#grid/Error {:axis \"" ax "\" :msg " (pr-str (str t)) "}"))))))
(flush)
DRIVER

if ! (cd "$STAGE" && clojure -Sdeps "$CLARA_DEP" -M drive.clj) \
      > "$STAGE/clara.out" 2> "$STAGE/clara.err"; then
  echo "check-grid-three-way: the Clara JVM itself failed (no axis ran):" >&2
  cat "$STAGE/clara.err" >&2
  exit 1
fi

# ── COMPARE ────────────────────────────────────────────────────────────────────────────────────

# Extract one `<key> [#wat.core/PersistentVector] [...]` bracket. Prints "<n-matches>\t<contents>".
#
# The lookbehind on `[ {]` is what keeps a LATENT hazard latent: `:oracle-derived` does NOT contain
# `:derived` (the colon is part of the needle — driven, and corrected in
# wat_scripts_grid_port_check.rs against a doc-comment in this tree that claims otherwise), but a
# future `:spec-derived` would, and the match count below is what refuses an ambiguous line.
extract() {
  local out n first
  out="$(grep -oP "(?<=[ {])\Q$2\E\s+(?:#wat\.core/PersistentVector\s+)?\K\[[^]]*\]" <<<"$1" || true)"
  if [ -z "$out" ]; then printf '0\t\n'; return 0; fi
  n="$(printf '%s\n' "$out" | wc -l)"
  first="$(printf '%s\n' "$out" | sed -n '1p')"
  first="${first#[}"; first="${first%]}"
  printf '%s\t%s\n' "$n" "$first"
}
norm() { local s; s="$(tr -s '[:space:]' ' ' <<<"$1")"; s="${s# }"; s="${s% }"; printf '%s' "$s"; }

report_pair() {  # name-a set-a name-b set-b
  local A B
  A="$STAGE/.a"; B="$STAGE/.b"
  tr ' ' '\n' <<<"$2" | grep -v '^$' | sort -u > "$A"
  tr ' ' '\n' <<<"$4" | grep -v '^$' | sort -u > "$B"
  echo "      $1 ($(wc -w <<<"$2") elems): $2" >&2
  echo "      $3 ($(wc -w <<<"$4") elems): $4" >&2
  echo "      only in $1: $(comm -23 "$A" "$B" | tr '\n' ' ')" >&2
  echo "      only in $3: $(comm -13 "$A" "$B" | tr '\n' ' ')" >&2
}

FAILED=0
AGREED=0
for stem in "${AXES[@]}"; do
  size="${SIZES[$stem]}"
  fail=0

  # ── guard 0a: the axis still fires the oracle ────────────────────────────────────────────────
  if ! grep -qF "$ORACLE_VERB" "$GRID_DIR/$stem.wat"; then
    echo "[$stem] source does NOT call $ORACLE_VERB — there is no oracle answer to compare" >&2
    FAILED=1; continue
  fi

  if ! printf '[%s]\n' "$size" | "$WAT_BIN" "$GRID_DIR/$stem.wat" \
        > "$STAGE/$stem.wat.out" 2> "$STAGE/$stem.wat.err"; then
    echo "[$stem] wat FAILED (size [$size]):" >&2
    cat "$STAGE/$stem.wat.err" >&2
    FAILED=1; continue
  fi
  WLINE="$(grep -m1 '^\s*#grid/Result' "$STAGE/$stem.wat.out" || true)"
  if [ -z "$WLINE" ]; then
    echo "[$stem] wat exited 0 but emitted NO #grid/Result line. stdout:" >&2
    cat "$STAGE/$stem.wat.out" >&2
    FAILED=1; continue
  fi
  CLINE="$(grep -m1 "^#grid/Result .*:axis \"$stem\"" "$STAGE/clara.out" || true)"
  if [ -z "$CLINE" ]; then
    echo "[$stem] Clara emitted NO #grid/Result row for this axis:" >&2
    grep -m1 ":axis \"$stem\"" "$STAGE/clara.out" >&2 || echo "      (no line mentions it at all)" >&2
    sed -n "/--- $stem ---/,/^--- /p" "$STAGE/clara.err" >&2 || true
    FAILED=1; continue
  fi

  # ── guard 0b: each field matches EXACTLY ONCE, on both lines ────────────────────────────────
  IFS=$'\t' read -r n_n NAT   < <(extract "$WLINE" ':derived')
  IFS=$'\t' read -r n_o ORA   < <(extract "$WLINE" ':oracle-derived')
  IFS=$'\t' read -r n_c CLA   < <(extract "$CLINE" ':derived')
  IFS=$'\t' read -r n_ws WSZ  < <(extract "$WLINE" ':size')
  IFS=$'\t' read -r n_cs CSZ  < <(extract "$CLINE" ':size')
  for pair in "wat:derived=$n_n" "wat:oracle-derived=$n_o" "clara:derived=$n_c" \
              "wat:size=$n_ws" "clara:size=$n_cs"; do
    if [ "${pair#*=}" != "1" ]; then
      echo "[$stem] field ${pair%=*} matched ${pair#*=} time(s), expected exactly 1 — the line is" >&2
      echo "      missing that column or carries two of it, and the comparison would be undefined." >&2
      echo "      wat  : $WLINE" >&2
      echo "      clara: $CLINE" >&2
      fail=1
    fi
  done
  if [ "$fail" -ne 0 ]; then FAILED=1; continue; fi

  NAT="$(norm "$NAT")"; ORA="$(norm "$ORA")"; CLA="$(norm "$CLA")"
  WSZ="$(norm "$WSZ")"; CSZ="$(norm "$CSZ")"; SENT="$(norm "$size")"

  # ── guard 1: the echoed :size is the size we SENT, on both sides ─────────────────────────────
  # Catches an arity mistake AT ITS CAUSE. `fanout [20 5]` echoes `:size [20]` — it read one
  # element and ignored the other — then derives nothing and "agrees" with everybody.
  if [ "$WSZ" != "$SENT" ]; then
    echo "[$stem] SIZE ARITY MISMATCH (wat): sent [$SENT], axis echoed [$WSZ]. The axis silently" >&2
    echo "      ignored part of the size; whatever it derived is not the workload asked for." >&2
    FAILED=1; continue
  fi
  if [ "$CSZ" != "$SENT" ]; then
    echo "[$stem] SIZE MISMATCH (clara): sent [$SENT], the Clara program reports [$CSZ] — the two" >&2
    echo "      engines did not run the same workload, so the comparison below is meaningless." >&2
    FAILED=1; continue
  fi

  # ── guard 2: non-vacuity ─────────────────────────────────────────────────────────────────────
  n_nat=$(wc -w <<<"$NAT"); n_ora=$(wc -w <<<"$ORA"); n_cla=$(wc -w <<<"$CLA")
  if [ "$n_nat" -eq 0 ] || [ "$n_ora" -eq 0 ] || [ "$n_cla" -eq 0 ]; then
    printf '[%s] VACUOUS — clara=%s native=%s oracle=%s. An empty set compares EQUAL to an empty\n' \
      "$stem" "$n_cla" "$n_nat" "$n_ora" >&2
    echo "      set and reports agreement while proving nothing." >&2
    FAILED=1; continue
  fi

  # ── guard 3: THE THREE PAIRINGS, each named ──────────────────────────────────────────────────
  # NOT a count: D7 produced a right-sized WRONG answer, which a cardinality check passes.
  if [ "$ORA" != "$CLA" ]; then
    echo "[$stem] ⛔ oracle != clara  =>  THE SPEC IS WRONG (size [$SENT])" >&2
    report_pair oracle "$ORA" clara "$CLA"
    fail=1
  fi
  if [ "$NAT" != "$CLA" ]; then
    echo "[$stem] ⛔ native != clara  =>  THE FAST PATH IS WRONG (size [$SENT])" >&2
    report_pair native "$NAT" clara "$CLA"
    fail=1
  fi
  if [ "$ORA" != "$NAT" ]; then
    echo "[$stem] ⛔ oracle != native  =>  A PORT BUG (size [$SENT])" >&2
    report_pair oracle "$ORA" native "$NAT"
    fail=1
  fi

  if [ "$fail" -ne 0 ]; then
    FAILED=1
  else
    printf '%-20s clara=%-4s native=%-4s oracle=%-4s  ALL THREE MATCH\n' \
      "$stem" "$n_cla" "$n_nat" "$n_ora"
    AGREED=$((AGREED + 1))
  fi
done

ELAPSED=$(( $(date +%s) - T0 ))
if [ "$FAILED" -eq 0 ]; then
  echo "grid-three-way: ${#AXES[@]} axis/axes, all AGREED — Clara == oracle == native (${ELAPSED}s)"
else
  echo "grid-three-way: FAILURES above ($AGREED of ${#AXES[@]} axes agreed, ${ELAPSED}s)" >&2
  exit 1
fi
