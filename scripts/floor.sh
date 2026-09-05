#!/usr/bin/env bash
# floor.sh — run the release floor and CAPTURE IT, always, before anyone reads it.
#
# WHY THIS EXISTS (arc 278, 2026-08-05). The injected CLAUDE.md used to bless four
# tests by name as "timing flakes … NOT release failures" and to call a green→red
# flip "a mode/timing signal first, not a regression". Riders read that as a licence
# to not report a red. The cost, on the record: an intermittent floor failure whose
# ARM WAS NEVER CAPTURED — the first look truncated the log, the re-run went green,
# and every mechanism proposed afterwards was a guess at which of five outcomes had
# fired. A dismissal does not merely tolerate a bug; it destroys the only evidence
# that could name one.
#
#   ⛔ THERE IS NO SUCH THING AS A KNOWN FLAKE. A RED IS A RED.
#   ⛔ DO NOT RE-RUN ON A RED. The re-run is what erases the finding.
#
# This script exists so that capturing is not a discipline anyone has to remember:
# the log is on disk before the summary is even printed. It is the extirpare rung
# above a convention — you cannot forget to capture, because capture is the default
# and reading is what happens after it.
#
# USAGE
#   scripts/floor.sh                       # the whole floor
#   scripts/floor.sh -E 'test(foo)'        # any extra args go straight to nextest
#
# OUTPUT (gitignored; the script is tracked, its output is not)
#   .floor/<utc-stamp>/raw.log       nextest's output, byte-for-byte, untruncated
#   .floor/<utc-stamp>/clean.log     the same, ANSI-stripped — quote from THIS one
#   .floor/<utc-stamp>/ARM.txt       on failure: each failing test's WHOLE block
#   .floor/<utc-stamp>/doctest.log   the doctest run, byte-for-byte
#   .floor/latest -> <utc-stamp>     symlink to the most recent run
#
# EXIT CODE is nextest's own. Never pipe this script into head/tail to decide
# pass/fail — a pipe returns the PAGER's exit, and a window makes an absence
# unfalsifiable.

set -uo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 2
ROOT="$PWD"

STAMP="$(date -u +%Y-%m-%dT%H-%M-%SZ)"
OUT="$ROOT/.floor/$STAMP"
mkdir -p "$OUT"
ln -sfn "$STAMP" "$ROOT/.floor/latest"

RAW="$OUT/raw.log"
CLEAN="$OUT/clean.log"
ARM="$OUT/ARM.txt"

strip_ansi() { sed 's/\x1b\[[0-9;]*m//g'; }

echo "[floor] capturing to .floor/$STAMP/ (symlinked as .floor/latest)"
echo "[floor] cargo test --doc --release  THEN  nice -n 19 cargo nextest run --release $*"
echo

# Capture EVERYTHING. The tee is the point: the disk gets the whole run even if
# this shell is killed, the terminal scrolls, or the reader only looks at the end.
#
# `nice -n 19` — the lowest scheduling priority. Builder, 2026-08-16: "i'm tired of
# being keyboard dos'd while cargo runs."
#
# THE REASON IS PARITY, not a benchmark. Builder's ruling: *"cargo test nice's theirs,
# nextest doesn't... so nice nextest."* `cargo test` already de-prioritises the test
# threads it spawns; nextest forks a process per test and does not. So this restores
# the behaviour the runner swap silently dropped — an argument from what the tools DO,
# which is why it does not need a benchmark to stand up.
#
# Wall-clock cost, measured: none (216.5s without, 217.4s with — noise).
#
# ⛔ TWO ALTERNATIVES WERE TRIED AND ARE NOT HERE. Recorded so nobody re-adds them:
#
#   1. PARALLELISM CAPS — cargo `jobs = -2` + nextest `test-threads = -2`. Looked
#      obvious, MEASURABLY DID NOTHING: floor load 17.56 uncapped vs 18.03 capped.
#      They bound rustc INVOCATIONS and test PROCESSES, and each of those spawns
#      threads of its own, so "leave 2 cores free" was never what they did. (A
#      negative value IS machine-relative — "logical CPUs minus N", confirmed by
#      direct concurrency observation with positive controls. It just doesn't help.)
#
#   2. A LATENCY BENCHMARK to justify this `nice`. Three instruments failed to
#      produce a defensible number: load average is blind to priority by design;
#      wake latency stays ~1ms even under full load because Linux favours just-woken
#      tasks; and a work-burst probe had run-to-run variance LARGER than the effect
#      (nice-19 max came out 9.04ms and 93.47ms on consecutive runs). An earlier
#      version of that probe recalibrated its work unit inside each measurement,
#      which normalised away the very starvation it was measuring and produced a
#      flattering 9x that was pure artifact. **There is no measured benefit claim
#      here, and there should not be one until an instrument earns it.**
#
# ── THE DOCTEST GATE ──────────────────────────────────────────────────────────
# ARMED 2026-09-04 AT ZERO, and it had never run before that day.
#
# nextest cannot run doctests — it is a documented limitation of the runner, not a
# choice anyone here made. So for as long as the floor has been `cargo nextest`, every
# ``` block in a Rust doc comment has been unexecuted. That was not an EXCLUSION (the
# floor's 17 skipped are 5 `default-filter` names + 12 `#[ignore]`s, every one carrying
# a written reason); it was an ABSENCE nobody had decided on. See
# `docs/arc/2026/06/255-builtin-registry/the-walls-must-not-be-muted/`.
#
# What it caught the moment it was first run, on a tree whose floor was 5139/5139 green:
#   src/edn/contract.rs         a PUBLIC example constructing `RuntimeError` by struct
#                               literal — a shape no external caller can use, because
#                               both fields are private. Stale through two API changes.
#   src/function/parse.rs       wat source in a BARE fence (rustdoc reads bare as Rust)
#   src/rete/kernel/fire/…      an ASCII diagram in an INDENTED block — also doctested
#
# ⛔ IT RUNS FIRST, and unconditionally. First because it is seconds against the floor's
# two minutes, so a stale doc fails fast. Unconditionally — even under a scoped `-E` —
# because a conditional is a door, and this gate exists precisely because a door nobody
# chose stayed open for months.
#
# `cargo test` de-prioritises its own threads (the parity argument below), so no `nice`.
echo "[floor] cargo test --doc --release"
DOC="$OUT/doctest.log"
cargo test --doc --release 2>&1 | tee "$DOC"
doc_status=${PIPESTATUS[0]}

if [ "$doc_status" -ne 0 ]; then
  cat <<EOF

[floor] ⛔ DOCTEST RED — exit=$doc_status
[floor]
[floor]   FULL LOG:  .floor/$STAMP/doctest.log
[floor]
[floor]   A doc example that does not compile is a lie the repository tells its
[floor]   readers, and it is shipped. The failing block names its own file and
[floor]   line; fix the example, or TAG the fence honestly (\`\`\`text / \`\`\`wat
[floor]   / \`\`\`edn) if it was never Rust. Do not delete the block to get green.
[floor]
[floor]   DO NOT RE-RUN to see if it passes. It will not.
[floor]
EOF
  exit "$doc_status"
fi
echo "[floor] doctests exit=0"
echo

# ⚠ `nice` must stay INSIDE the pipeline's first stage — `${PIPESTATUS[0]}` below is
# nextest's own exit code, and it stays correct because `nice` exec's and returns the
# child's status. Verified both ways: green propagates 0, red propagates 100 with
# ARM.txt captured. Do NOT move it to wrap the whole pipe.
#
# ⚠ `nice` must stay INSIDE the pipeline's first stage — `${PIPESTATUS[0]}` below is
# nextest's own exit code, and it stays correct because `nice` exec's and returns the
# child's status. Do NOT move it to wrap the whole pipe.
nice -n 19 cargo nextest run --release "$@" 2>&1 | tee "$RAW"
status=${PIPESTATUS[0]}

strip_ansi < "$RAW" > "$CLEAN"

summary="$(grep -E '^ *Summary' "$CLEAN" | tail -1)"

echo
if [ -n "$summary" ]; then
  echo "[floor]$summary"
else
  echo "[floor] ⚠ NO SUMMARY LINE — the run did not complete (build failure, crash, or kill)."
  echo "[floor]   That is itself a finding. Full output: .floor/$STAMP/clean.log"
fi

if [ "$status" -eq 0 ]; then
  echo "[floor] exit=0. Log kept at .floor/$STAMP/ regardless — a green run is evidence too."
  exit 0
fi

# ── RED ───────────────────────────────────────────────────────────────────────
# Keep the ARM. Each failing test's whole block, verbatim: the arm/assertion text
# is what predicts the mechanism, and it is exactly what a truncating pager eats.
{
  echo "FLOOR RED — $STAMP"
  echo "exit=$status"
  echo "${summary:-  (no Summary line — run did not complete)}"
  echo
  echo "=============================================================="
  echo "FAILING TESTS"
  echo "=============================================================="
  grep -E '^ *(FAIL|TRY|SIGSEGV|ABORT|TIMEOUT|SLOW)' "$CLEAN" | sort -u
  echo
  echo "=============================================================="
  echo "THE ARM — each failing test's WHOLE stdout+stderr block, verbatim."
  echo "Report THIS. Not a summary of it. Not a window into it."
  echo "=============================================================="
  echo
  # nextest prints, per failure, the test header then its stdout/stderr sections.
  # Take from the first FAIL to end-of-file: cheap, and it cannot cut the arm off.
  awk '/^ *(FAIL|TRY|SIGSEGV|ABORT|TIMEOUT)/{seen=1} seen' "$CLEAN"
} > "$ARM"

cat <<EOF

[floor] ⛔ RED — exit=$status
[floor]
[floor]   THE ARM IS CAPTURED:  .floor/$STAMP/ARM.txt
[floor]   FULL LOG:             .floor/$STAMP/clean.log   (raw.log keeps the colour)
[floor]
[floor]   DO NOT RE-RUN. A re-run that goes green destroys the only evidence
[floor]   this failure will ever produce. There is no such thing as a known
[floor]   flake — "timing" / "pre-existing" / "unrelated" / "passes in
[floor]   isolation" are descriptions of your SEARCH, not dispositions.
[floor]
[floor]   REPORT, in this order:
[floor]     1. the Summary line, verbatim
[floor]     2. the failing test names
[floor]     3. each one's WHOLE block from ARM.txt — never a summary of it
[floor]     4. the exact arm/assertion that fired (each arm = a different mechanism)
[floor]
EOF

exit "$status"
