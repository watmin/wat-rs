#!/usr/bin/env bash
# wat-scripts/perf/grid/run-axis.sh AXIS SIZE...
#
# THE shared Clara-grid runner (docs/arc/2026/06/278-rules-engine/DESIGN-clara-grid.md) — every
# axis (A0..A8) mirrors the SAME three artifacts and gets swept by this ONE script:
#   grid/<axis>.wat        — the wat workload; fires native `:wat::rete::fire-rules`; stdin =
#                             an i64 vector [size...]; stdout = one #grid/Result EDN line.
#   grid/gen-<axis>.sh     — emits the Clara translation (.clj) of the SAME workload; args =
#                             the size numbers (word-split, positional); stdout (of the emitted
#                             program's -main) = one #grid/Result EDN line with :clara-ns.
#   (this script)          — runs both sides per SIZE, canonicalizes + compares :derived
#                             (ACCURACY) and :native-ns vs :clara-ns (SPEED), emits a #grid/Verdict.
#
# SIZE is a space-separated size-tuple, e.g. "3 10" (strat-neg: strata items) or "20 10"
# (deep-cascade: depth width) — axis-specific shape, opaque to this runner. Pass one SIZE per
# sweep point: `run-axis.sh strat-neg "3 10" "6 2000"` runs two points.
#
# CANONICALIZATION (STOP trigger #2 in the DESIGN — tag-form must be reconciled, not faked away):
# wat's EDN printer tags every PersistentVector as `#wat.core/PersistentVector [...]` (a real,
# load-bearing round-trip-identity decision, wat/src/edn_shim.rs) while Clojure's `pr-str` of a
# plain vector is bare `[...]`. The VALUES are identical either way — only the wire wrapper
# differs — so this runner strips the wat-side tag before the string-compare. That is exactly
# the canonicalization the DESIGN calls for, not a fudge: the accuracy check still fails loudly
# on any real difference in the derived-fact SET (missing/extra/reordered elements).
#
# TWO VERDICTS PER SIZE, and the distinction is load-bearing:
#   :ratio / :winner            — the ENGINE. clara-ns / native-ns, both sides timing the FIRE only.
#                                 This is what the differential and the whole grid history mean.
#   :wall-ratio / :wall-winner  — the PROGRAM. clara-wall / wat-wall, `time` around each process:
#                                 our freeze + seed + fire + derive + print, against the JVM's cold
#                                 boot + Clojure load + its own everything. Same convention (>1 ⇒ us).
#   :fire-share-pct             — how much of OUR wall clock the timed region actually covers.
# They are NOT merged. An engine can win the fire and lose the program, and collapsing them hides
# exactly that. Added 2026-08-01 after measuring fanout [40000]: fire 0.046s of a 5.33s run —
# :fire-share-pct 0.9. The record had flagged this ("the grid timed `fire` and declared superiority
# on 2% of the runtime") and we kept rediscovering it because the runner never captured the whole.
#
# Emits, per SIZE, one line:
#   #grid/Verdict {:axis "<axis>" :size [<size>] :accuracy :match|:MISMATCH :ratio <f64> :winner :us|:clara|:tie
#                  :wat-wall-ms <n> :clara-wall-ms <n> :wall-ratio <f64> :wall-winner <..> :fire-share-pct <f64>}
# :ratio is clara-ns / native-ns (>1 ⇒ native faster ⇒ :us; <1 ⇒ Clara faster ⇒ :clara; within
# ±5% ⇒ :tie). A :MISMATCH also dumps both :derived sets to stderr (never hidden — DESIGN STOP #4).
set -euo pipefail

GRID_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLARA_DEP='{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}} :paths ["."]}'

# THE BINARY UNDER MEASUREMENT — the LOCAL build, never `cargo wat`.
#
# `cargo wat` resolves to whatever sits in ~/.cargo/bin, which is a DIFFERENT binary than the
# one in this tree: on 2026-07-30 the install was 7 hours older than target/release/wat, across
# a kernel purge and a compiler bump. A benchmark that reads one build while you reason about
# another is an instrument supplying its own result — and this runner had that bug on every
# axis, which is reason enough to distrust any number it produced before this line existed.
# Set WAT_BIN explicitly if you genuinely mean to measure an install.
REPO_ROOT="$(cd "$GRID_DIR/../../.." && pwd)"
WAT_BIN_PINNED="${WAT_BIN:+yes}"          # did the CALLER name a binary, or are we defaulting?
WAT_BIN="${WAT_BIN:-$REPO_ROOT/target/release/wat}"
[ -x "$WAT_BIN" ] || {
  echo "run-axis: no wat binary at $WAT_BIN — build it first (cargo build --release)" >&2
  exit 1
}

# ── THE FRESHNESS WALL — the default binary must be NEWER than every input that builds it ──
#
# The block above solved HALF of its own stated principle. It stops you measuring the WRONG
# binary (`cargo wat`), and its argument — "a benchmark that reads one build while you reason
# about another is an instrument supplying its own result" — applies word for word to a STALE
# local build, which it did not check. `[ -x ]` asks whether the file exists.
#
# THE COST IS ON THE RECORD, and it was read as an engine change. Between the grids of
# 2026-08-26T23-43-12Z and 2026-08-27T00-04-33Z — 21 minutes apart, `git log` EMPTY between
# them — `leading-exists` wat-ns fell 3.2x / 4.0x / 3.8x across its three rungs and stayed
# down in every later run. No code changed; the binary did. Both grids are in this directory
# and both read as measurements of the engine. (Found 2026-08-30, while building a baseline
# to read a new grid against.)
#
# ⛔ IT REFUSES; IT DOES NOT BUILD. An auto-build here would cure this defect by causing the
# OTHER one the record already paid for: `run-all.sh` immediately after a compile measures a
# hot, loaded box, which is exactly how a phantom +46.9% on `deep-cascade [10 100]` was
# manufactured on 2026-08-24. Building and letting the machine settle are the caller's job and
# must stay two separate acts.
#
# ⛔ IT ASKS CARGO; IT DOES NOT ENUMERATE THE INPUTS. The first version of this wall listed
# them by hand — src, crates, wat, build.rs, Cargo.{toml,lock} — and was wrong twice within a
# minute of being written, which is why it is not what shipped:
#
#   1. IT OMITTED `tests/`. `build.rs` emits `cargo:rerun-if-changed=tests/<group>` for every
#      group dir, so cargo considers those inputs and the hand list did not. An input set is a
#      SECOND copy of cargo's dependency graph, and it rots exactly the way any second copy does.
#   2. IT WAS UNSATISFIABLE. A `#[cfg(test)]` source can be newer than the binary while being
#      unable to affect it — cargo rebuilds, correctly relinks nothing, and the mtime does not
#      move. The wall then refuses, tells you to build, and refuses again forever. A guard whose
#      instruction cannot clear it is worse than no guard: it teaches you to bypass it.
#
# Cargo already owns this question. Run it and watch the ARTIFACT: unchanged means the binary
# already reflected the tree (a true no-op costs 0.08s, measured); changed means it did not, and
# every number this run would have printed would have described a build that is not in the tree.
if [ -z "$WAT_BIN_PINNED" ]; then
  BIN_BEFORE="$(stat -c '%Y %s' "$WAT_BIN" 2>/dev/null || echo absent)"
  BUILD_ERR="$(mktemp)"
  if ! ( cd "$REPO_ROOT" && cargo build --release --bin wat ) >/dev/null 2>"$BUILD_ERR"; then
    echo "run-axis: REFUSING — \`cargo build --release --bin wat\` failed; cannot vouch for the binary:" >&2
    sed 's/^/    /' "$BUILD_ERR" >&2
    rm -f "$BUILD_ERR"
    exit 1
  fi
  rm -f "$BUILD_ERR"
  BIN_AFTER="$(stat -c '%Y %s' "$WAT_BIN" 2>/dev/null || echo absent)"
  if [ "$BIN_BEFORE" != "$BIN_AFTER" ]; then
    {
      echo "run-axis: REFUSING — $WAT_BIN did not match the tree; it has just been rebuilt."
      echo "  Artifact (mtime size):  $BIN_BEFORE  ->  $BIN_AFTER"
      echo "  Nothing was measured. The binary is now current, but this box is now HOT from the"
      echo "  compile, and a grid started in that window measures the weather — that is how a"
      echo "  phantom +46.9% on deep-cascade [10 100] was manufactured on 2026-08-24."
      echo "  Wait for the 1-min load average to settle (uptime), then run the grid again."
      echo "  To measure a specific build on purpose, name it: WAT_BIN=/path/to/wat $0 ..."
    } >&2
    exit 1
  fi
fi

# ── THE BLAST DOOR — a benchmark may not be able to take the workstation down ──
#
# 2026-07-30: `run-axis.sh node-share "10 2000" "50 2000"` CRASHED THE BUILDER'S MACHINE. The
# N=50 point consumed the box's 43 GiB of available RAM and the OOM killer took the desktop
# with it; the axis produced empty stdout AND empty stderr, which is the signature of a process
# being killed rather than failing. A benchmark that can do that is not a measurement tool.
#
# The cap is not caution, it is a wall: each side runs in its OWN cgroup scope with a hard
# MemoryMax and swap disabled, so a runaway dies alone and reports WHY. MemorySwapMax=0 matters
# more than the cap — swap thrash is what actually wedges a desktop before the OOM killer acts.
#
# Raise GRID_MEM_MAX deliberately for a big run; do not remove the guard. And grow a size ladder
# UPWARD from small (R24: these harnesses are O(n^2) interpreted — a point that returns fast
# says nothing about the next one).
GRID_MEM_MAX="${GRID_MEM_MAX:-6G}"
GRID_TIMEOUT="${GRID_TIMEOUT:-300}"

# ── REPEATS — one lucky sample may not produce a verdict ──────────────────────
#
# 2026-07-31: this runner reported `accum [100 200] ratio 1.0604 :winner :us` — a FLIP from a
# previously-recorded 0.653 :clara. Four more runs of the identical point read 0.6996 / 0.5911 /
# 0.8984 / 0.6134. The 1.06 was one sample at the top of an ~80%-wide distribution and the flip was
# not real. At the same time `[200 200]` read 0.664 against a prior 0.665 — two samples of a
# ~40%-wide distribution landing on top of each other, which looked like "unchanged" and was not.
#
# Single-run verdicts are therefore unreadable wherever the ratio is near parity. Each point now
# runs GRID_RUNS times and the verdict reports mean AND min/max — the SPREAD stays visible, because
# a bare mean would have concealed exactly the thing that produced the false flip.
GRID_RUNS="${GRID_RUNS:-3}"

if command -v systemd-run >/dev/null 2>&1 && systemd-run --user --scope --quiet true 2>/dev/null; then
  guard() { systemd-run --user --scope --quiet \
              -p MemoryMax="$GRID_MEM_MAX" -p MemorySwapMax=0 \
              -- timeout "$GRID_TIMEOUT" "$@"; }
  GUARD_KIND="cgroup MemoryMax=$GRID_MEM_MAX, no swap, ${GRID_TIMEOUT}s"
else
  # Fallback: address-space cap. Blunter than a cgroup (Rust reserves generously, so a false
  # trip is possible) but it still beats an unbounded run against the machine.
  GUARD_KB=$(( $(numfmt --from=iec "$GRID_MEM_MAX") / 1024 ))
  guard() { ( ulimit -v "$GUARD_KB"; exec timeout "$GRID_TIMEOUT" "$@" ); }
  GUARD_KIND="ulimit -v $GRID_MEM_MAX, ${GRID_TIMEOUT}s (no systemd-run)"
fi
echo "run-axis: guard = $GUARD_KIND" >&2

AXIS="${1:?usage: run-axis.sh AXIS SIZE...}"; shift
WAT_FILE="$GRID_DIR/$AXIS.wat"
GEN_FILE="$GRID_DIR/gen-$AXIS.sh"
[ -f "$WAT_FILE" ] || { echo "run-axis: missing $WAT_FILE" >&2; exit 1; }
[ -f "$GEN_FILE" ] || { echo "run-axis: missing $GEN_FILE" >&2; exit 1; }
[ "$#" -ge 1 ] || { echo "run-axis: need at least one SIZE" >&2; exit 1; }

# GRID_SKIP_ORACLE=1 — native vs Clara only. Axes fire `$oracle` after the timed
# native fire so :oracle-derived can be emitted; that is not the engine ratio
# (:native-ns is fire-only) but it is the bulk of wat-wall on the slow axes.
# Replace the oracle call with the already-fired native session.
# `$` is a perl metacharacter — escape it. The 2026-08-20 skip was a no-op
# because it still matched `fire-rules-spec` after the `$oracle` rename.
WAT_SRC="$WAT_FILE"
ORACLE_TMP=""
if [ -n "${GRID_SKIP_ORACLE:-}" ]; then
  ORACLE_TMP="$(mktemp --suffix=.wat)"
  # ⛔ SUBSTITUTES THE `Fired` ARM, NOT A BARE SESSION — arc 278, the fire-outcome wall.
  # The axes now face `(:wat::rete::FireOutcome :- [Session])`, so this call is the SCRUTINEE of a
  # `match`. Swapping in `fired` (a Session) left the match applied to a Session — a type error, and
  # it broke this runner from the moment the wall landed. `(FireOutcome::Fired fired)` keeps the
  # match well-typed and makes it take that arm at once, which is exactly what "skip the oracle
  # fire, reuse the native result" means. It also stays independent of the arms' wording.
  #
  # ⚠ THIS REWRITE EXISTS TWICE. `tests/rete/wat_scripts_grid_axes_live.rs`'s `skip_oracle_fire`
  # does the same substitution for the liveness gate, and ONLY THAT ONE IS ON THE FLOOR — which is
  # why this side stayed broken silently. If you change one, change both; better, give them one
  # home. Two places holding one truth is the defect this arc pulls out most often.
  perl -pe 's/\(:wat::rete::fire-rules\$oracle\s+staged\)/(:wat::rete::FireOutcome::Fired fired)/g' "$WAT_FILE" > "$ORACLE_TMP"
  # Match the CALL form, not a comment that names `$oracle` (strat-neg's header does).
  if grep -F -q '(:wat::rete::fire-rules$oracle' "$ORACLE_TMP"; then
    echo "run-axis: GRID_SKIP_ORACLE rewrite left a fire-rules\$oracle token — skip is a no-op" >&2
    exit 2
  fi
  WAT_SRC="$ORACLE_TMP"
  echo "run-axis: GRID_SKIP_ORACLE=1 — fire-rules\$oracle not invoked (native vs Clara only)" >&2
fi
trap 'rm -f "$ORACLE_TMP"' EXIT

for SIZE in "$@"; do
  SIZE_JSON="[$(echo "$SIZE" | tr -s ' ' ',')]"

  # The Clara program is generated ONCE per size and re-run; generation is not under measurement.
  CLJ_TMP="$(mktemp -d)"
  AXIS_FILE="${AXIS//-/_}"
  bash "$GEN_FILE" $SIZE > "$CLJ_TMP/$AXIS_FILE.clj"

  RATIOS=""
  WAT_NSS=""
  CLARA_NSS=""
  WAT_WALLS=""
  CLARA_WALLS=""
  ACCURACY=":match"
  ORACLE_ACCURACY=":match"
  PORT_ACCURACY=":match"
  HAS_ORACLE=0

  for RUN in $(seq 1 "$GRID_RUNS"); do
    # ── wat side: native fire-rules, timed inside the script itself ──────────
    # stderr is CAPTURED, not discarded: `2>/dev/null` made a wat-side failure loud but
    # REASONLESS — you learned the axis produced nothing and never why.
    WAT_ERR="$(mktemp)"
    set +e
    # OUTER WALL CLOCK. `:native-ns` times the FIRE ONLY, and that is honest for comparing
    # engines — but at fanout [40000] the fire is ~0.9% of the program, so a ratio built on it
    # says nothing about what a user waits for. Measured 2026-08-01; the record had already
    # flagged it ("the grid timed `fire` and declared superiority on 2% of the runtime") and we
    # kept rediscovering it because the runner never captured the whole. Now it does, both sides.
    WAT_W0=$(date +%s%N)
    WAT_OUT="$(echo "$SIZE_JSON" | guard "$WAT_BIN" "$WAT_SRC" 2>"$WAT_ERR")"
    WAT_RC=$?
    WAT_W1=$(date +%s%N)
    set -e
    WAT_LINE="$(echo "$WAT_OUT" | grep -o '#grid/Result.*' || true)"
    if [ -z "$WAT_LINE" ]; then
      echo "run-axis: wat side produced no #grid/Result for axis=$AXIS size=[$SIZE] run=$RUN (rc=$WAT_RC)" >&2
      echo "  binary: $WAT_BIN" >&2
      # rc is the diagnosis, and empty-output-with-no-error is the tell of a KILL, not a failure.
      case "$WAT_RC" in
        124) echo "  ⇒ TIMED OUT at ${GRID_TIMEOUT}s. Raise GRID_TIMEOUT, or the size is too big." >&2 ;;
        137) echo "  ⇒ KILLED (SIGKILL) — almost certainly the memory cap ($GRID_MEM_MAX)." >&2
             echo "    That is the guard working. Do NOT just raise it; a blowup at this size IS the finding." >&2 ;;
        *)   echo "  ⇒ exit $WAT_RC" >&2 ;;
      esac
      echo "  ── stdout ──" >&2; echo "$WAT_OUT" >&2
      echo "  ── stderr ──" >&2; cat "$WAT_ERR" >&2
      rm -f "$WAT_ERR"; rm -rf "$CLJ_TMP"
      exit 1
    fi
    rm -f "$WAT_ERR"

    # ── Clara side ────────────────────────────────────────────────────────────
    # The Clara side's wall clock INCLUDES JVM cold boot + Clojure load, which is the point:
    # that is what a user of the peer actually waits for, exactly as our wall includes freeze,
    # seeding, derive and print. Neither number is flattered; both are the whole program.
    CLARA_W0=$(date +%s%N)
    CLARA_OUT="$(cd "$CLJ_TMP" && clojure -Sdeps "$CLARA_DEP" -M -m "$AXIS" 2>/dev/null || true)"
    CLARA_W1=$(date +%s%N)
    CLARA_LINE="$(echo "$CLARA_OUT" | grep -o '#grid/Result.*' || true)"
    if [ -z "$CLARA_LINE" ]; then
      echo "run-axis: Clara side produced no #grid/Result for axis=$AXIS size=[$SIZE] run=$RUN:" >&2
      echo "$CLARA_OUT" >&2
      rm -rf "$CLJ_TMP"
      exit 1
    fi

    # ── canonicalize :derived (strip wat's PersistentVector tag) + compare ────
    # Checked on EVERY run, not just the first: an accuracy divergence that only appears
    # sometimes is a worse finding than one that always does, and must not be sampled away.
    WAT_DERIVED="$(echo "$WAT_LINE" | grep -oP ':derived\s+(?:#wat\.core/PersistentVector\s+)?\K\[[^]]*\]')"
    CLARA_DERIVED="$(echo "$CLARA_LINE" | grep -oP ':derived\s+\K\[[^]]*\]')"
    WAT_NS="$(echo "$WAT_LINE" | grep -oP ':native-ns\s+\K[0-9]+')"
    CLARA_NS="$(echo "$CLARA_LINE" | grep -oP ':clara-ns\s+\K[0-9]+')"

    if [ -z "$WAT_DERIVED" ] || [ "$WAT_DERIVED" != "$CLARA_DERIVED" ]; then
      ACCURACY=":MISMATCH"
      echo "run-axis: MISMATCH axis=$AXIS size=[$SIZE] run=$RUN" >&2
      echo "  wat   :derived $WAT_DERIVED" >&2
      echo "  clara :derived $CLARA_DERIVED" >&2
    fi

    # ── THREE-WAY (opt-in): an axis may ALSO emit :oracle-derived, the wat SPEC's answer ──
    # Every other axis runs NATIVE only, so a flaw the oracle and its faithful Rust port SHARE
    # is invisible: `oracle == native` passes and Clara is never asked about the oracle. Where
    # the axis supplies the column we render it, and the THREE pairings diagnose differently:
    #   oracle vs clara  MISMATCH  =>  the SPEC is wrong
    #   native vs clara  MISMATCH  =>  the fast path is wrong
    #   oracle vs native MISMATCH  =>  a PORT bug
    # Absent the field this block is inert, so the 2-way axes are byte-for-byte unaffected.
    # GRID_SKIP_ORACLE rewrites spec→native so :oracle-derived would be a lie; do not score it.
    ORACLE_DERIVED=""
    if [ -z "${GRID_SKIP_ORACLE:-}" ]; then
      ORACLE_DERIVED="$(echo "$WAT_LINE" | grep -oP ':oracle-derived\s+(?:#wat\.core/PersistentVector\s+)?\K\[[^]]*\]' || true)"
    fi
    if [ -n "$ORACLE_DERIVED" ]; then
      HAS_ORACLE=1
      if [ "$ORACLE_DERIVED" != "$CLARA_DERIVED" ]; then
        ORACLE_ACCURACY=":MISMATCH"
        echo "run-axis: ORACLE MISMATCH axis=$AXIS size=[$SIZE] run=$RUN" >&2
        echo "  oracle :derived $ORACLE_DERIVED" >&2
        echo "  clara  :derived $CLARA_DERIVED" >&2
      fi
      if [ "$ORACLE_DERIVED" != "$WAT_DERIVED" ]; then
        PORT_ACCURACY=":MISMATCH"
        echo "run-axis: PORT MISMATCH (oracle != native) axis=$AXIS size=[$SIZE] run=$RUN" >&2
        echo "  oracle :derived $ORACLE_DERIVED" >&2
        echo "  native :derived $WAT_DERIVED" >&2
      fi
    fi

    RATIOS="$RATIOS $(awk -v n="$WAT_NS" -v c="$CLARA_NS" 'BEGIN { printf "%.4f", (n>0)? c/n : -1 }')"
    WAT_NSS="$WAT_NSS $WAT_NS"
    CLARA_NSS="$CLARA_NSS $CLARA_NS"
    WAT_WALLS="$WAT_WALLS $(( (WAT_W1   - WAT_W0)   / 1000000 ))"
    CLARA_WALLS="$CLARA_WALLS $(( (CLARA_W1 - CLARA_W0) / 1000000 ))"
  done

  rm -rf "$CLJ_TMP"

  # ── mean + spread, and a verdict only where every run agrees ───────────────
  # :us / :clara require EVERY run to fall on the same side of the ±5% band. If the spread
  # straddles parity the honest answer is :unresolved — not the mean's coin flip. A matrix that
  # reports :unresolved for two points and stands behind nineteen is worth more than one that
  # reports twenty-one verdicts, two of which are samples.
  STATS="$(echo "$RATIOS" | awk '{ n=0; s=0; mn=1e18; mx=-1e18;
      for (i=1; i<=NF; i++) { v=$i+0; n++; s+=v; if (v<mn) mn=v; if (v>mx) mx=v }
      printf "%.4f %.4f %.4f", s/n, mn, mx }')"
  MEAN="$(echo "$STATS" | cut -d' ' -f1)"
  MIN="$(echo "$STATS"  | cut -d' ' -f2)"
  MAX="$(echo "$STATS"  | cut -d' ' -f3)"
  WINNER="$(awk -v mn="$MIN" -v mx="$MAX" 'BEGIN {
      if      (mn > 1.05) print ":us"
      else if (mx < 0.95) print ":clara"
      else                print ":unresolved" }')"

  WAT_MEAN="$(echo "$WAT_NSS"   | awk '{ s=0; for(i=1;i<=NF;i++) s+=$i; printf "%d", s/NF }')"
  CLARA_MEAN="$(echo "$CLARA_NSS" | awk '{ s=0; for(i=1;i<=NF;i++) s+=$i; printf "%d", s/NF }')"

  # ── OUR OWN DISPERSION, because a mean cannot be compared to a mean without one ───────────
  #
  # The verdict carried :min/:max for the RATIO from the first day and NOTHING for :wat-ns, so
  # every "we got N% faster/slower" claim ever read off this grid was un-falsifiable: the reader
  # had no way to ask whether N exceeded the run-to-run spread. Measured 2026-09-02 on the cell
  # that looked like the one real signal — fanout [40000], the biggest and steadiest axis, three
  # consecutive 5-run verdicts on the SAME binary:
  #
  #     22,964,924  →  23,695,524  →  25,348,294 ns        a 10.4% spread, nothing changed
  #
  # The grid delta being investigated was +11.4%. It was noise, and the artifact could not say so.
  # These two fields are what let the next hand answer that WITHOUT re-running the sweep.
  WAT_NS_MIN="$(echo "$WAT_NSS" | awk '{ m=$1; for(i=1;i<=NF;i++) if($i<m) m=$i; printf "%d", m }')"
  WAT_NS_MAX="$(echo "$WAT_NSS" | awk '{ m=$1; for(i=1;i<=NF;i++) if($i>m) m=$i; printf "%d", m }')"

  # ── the WHOLE-PROGRAM verdict, reported beside the fire-only one ───────────
  # :wall-ratio is clara-wall / wat-wall on the SAME convention as :ratio (>1 ⇒ we finish
  # sooner). It is deliberately NOT folded into :winner — :winner remains the ENGINE verdict,
  # because that is what the differential and the whole grid history mean. Two numbers, two
  # claims, neither standing in for the other: an engine can win the fire and lose the program.
  WAT_WALL="$(echo "$WAT_WALLS"   | awk '{ s=0; for(i=1;i<=NF;i++) s+=$i; printf "%d", s/NF }')"
  CLARA_WALL="$(echo "$CLARA_WALLS" | awk '{ s=0; for(i=1;i<=NF;i++) s+=$i; printf "%d", s/NF }')"
  WALL_RATIO="$(awk -v w="$WAT_WALL" -v c="$CLARA_WALL" 'BEGIN { printf "%.4f", (w>0)? c/w : -1 }')"
  WALL_WINNER="$(awk -v r="$WALL_RATIO" 'BEGIN {
      if      (r > 1.05) print ":us"
      else if (r < 0.95) print ":clara"
      else               print ":tie" }')"
  # The share of our wall clock the timed region actually covers. If this is 1%, any claim
  # resting on :ratio alone is a claim about 1% of the program — say so in the artifact, not
  # in someone's memory.
  FIRE_SHARE="$(awk -v ns="$WAT_MEAN" -v wall="$WAT_WALL" 'BEGIN { printf "%.2f", (wall>0)? (ns/1000000.0)*100.0/wall : -1 }')"

  ORACLE_FIELDS=""
  if [ "$HAS_ORACLE" = "1" ]; then
    ORACLE_FIELDS=" :oracle-accuracy $ORACLE_ACCURACY :port-accuracy $PORT_ACCURACY"
  fi
  echo "#grid/Verdict {:axis \"$AXIS\" :size [$(echo "$SIZE" | tr -s ' ' ' ')] :accuracy $ACCURACY$ORACLE_FIELDS :runs $GRID_RUNS :ratio $MEAN :min $MIN :max $MAX :wat-ns $WAT_MEAN :wat-ns-min $WAT_NS_MIN :wat-ns-max $WAT_NS_MAX :clara-ns $CLARA_MEAN :winner $WINNER :wat-wall-ms $WAT_WALL :clara-wall-ms $CLARA_WALL :wall-ratio $WALL_RATIO :wall-winner $WALL_WINNER :fire-share-pct $FIRE_SHARE}"
done
