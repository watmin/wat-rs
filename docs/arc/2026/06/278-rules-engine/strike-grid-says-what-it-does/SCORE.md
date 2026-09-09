# SCORE — four grid scripts that do not do what their headers say

Executing strike per `DESIGN.md`. Appending as each item's verdict settles (house rule).

## `3Q2` — the outer non-vacuity is gated, the inner one is not

**Re-derived the sites.** `check-where-shapes.sh:152/158`, `check-query-compat.sh:135/141`,
`check-spec-native.sh:87/92` all matched DESIGN exactly: a `PAIRS -eq 0` guard, and a success
message printing `$ROWS_TOTAL` with nothing checking it is nonzero.

**Found a delta: DESIGN's motivating example does not reproduce today.** DESIGN's illustration is
`PAIRS=5, ROWS_TOTAL=0` printing GREEN. Read all three `check_pair()`/`check_stem()` bodies in
full: each already has a per-pair non-vacuity check that returns failure BEFORE a zero-row pair
could ever reach the `ROWS_TOTAL +=` line —

- `check-where-shapes.sh:115-118`: `if [ "$wn" -lt 1 ]; then ... return 1; fi` (blamed to
  `c8b062e64`, 2026-08-01 — original authorship of this exact shape, not a recent add)
- `check-query-compat.sh:101-104`: `if [ "$nn" -lt 1 ]; then ... return 1; fi` (blamed to
  `d2d73dc39`, 2026-08-17)
- `check-spec-native.sh:58-61`: `if [ "$wn" -lt 1 ]; then ... return 1; fi` (blamed to
  `b4e2985a2`, 2026-08-17)

Since `ROWS_TOTAL` is only ever incremented on the same code path that just asserted `wn`/`nn >= 1`
and returned 0 (success), **`FAILED=0` (all pairs succeeded) mathematically implies
`ROWS_TOTAL >= PAIRS >= 1`** in the code as it stands. I verified this isn't just a reading: ran
`check-where-shapes.sh where-boolean`, `check-query-compat.sh where-query-compat`, and
`check-spec-native.sh where-boolean` before touching anything — all three pass green with real
positive row counts, exactly as the per-pair guard predicts. DESIGN's specific vacuity
(`PAIRS=5, ROWS_TOTAL=0`, GREEN) is not reachable through any code path I found in any of the
three files today.

**Cure applied anyway, as instructed and as genuine defense-in-depth.** check-grid-speed.sh itself
already carries this exact two-layer pattern (a per-cell `:accuracy != match` check AND an
aggregate `seen -lt 30` check at :85-90) — a second, redundant guard at the aggregate level is this
repo's own convention here, not a novel practice, and it means a FUTURE change that weakens or
removes one of the three per-pair `wn`/`nn -lt 1` checks can't silently reopen the vacuity without
this also going red. Added, in each of the three files, right after the existing `PAIRS -eq 0`
check and before the `FAILED -eq 0` success branch:

```
if [ "$ROWS_TOTAL" -lt 1 ]; then
  echo "check-where-shapes: $PAIRS pair(s) but ROWS_TOTAL=0 — a pass with zero rows compared is" \
       "the vacuous-gate class; something is wrong upstream of this count" >&2
  exit 2
fi
```
(wording adjusted per-file: "family(ies)" for the query-compat/spec-native scripts, matching each
file's own existing vocabulary).

**STOP check: did the new guard red anything live?** No. Ran all three scripts (targeted single-
stem invocations, given the ~3.5s/JVM-boot cost) after adding the guard — all three still pass
green with the new check in place. This is the expected, unsurprising outcome given the
re-derivation above: the vacuity the guard defends against is not currently reachable, so there was
nothing live to catch. Reported here per the brief regardless, since the STOP condition explicitly
asked to watch for it.

## `3F1` — the file names the bug, fixes one side, keeps it on the other

**Re-derived.** `run-axis.sh:227-228` (the wat-side comment naming the discipline), Clara side at
what was line 264 (`2>/dev/null`) with its failure branch at 267-271 echoing `$CLARA_OUT` (stdout)
only. Matches DESIGN exactly — no delta on the site.

**Cure applied:** capture Clara's stderr to a temp file (`CLARA_ERR`) exactly as the wat side does
(`WAT_ERR`), and print both stdout and stderr on failure, symmetric with the wat-side failure
block's `── stdout ──` / `── stderr ──` shape. Cleaned up the temp file on both the success and
failure paths, matching `WAT_ERR`'s own `rm -f` placement.

**Verified the mechanism directly**, not just by reading: built an isolated repro (a `.clj` file
with a bad namespace, invoked exactly as run-axis.sh invokes Clara) and confirmed that under the
OLD code (`2>/dev/null`) the classpath error is silently swallowed and stdout is empty, while under
the NEW code the same error — `Execution error (FileNotFoundException) ... Could not locate
broken__init.class, broken.clj or broken.cljc on classpath.` — is captured and printed. Also ran a
real axis end-to-end (`GRID_SKIP_ORACLE=1 GRID_RUNS=1 run-axis.sh negation 100`) to confirm the
success path is unaffected — emits the same `#grid/Verdict` line as before.

## `3T2` — a header promising output the script never produces

**Re-derived.** `run-all.sh:23`'s promise ("then a summary tally on stderr") and confirmed zero
hits for `tally|summary|total|count|seen` across the file before this change (matches DESIGN).

**Cure applied:** built the tally, on stderr, without touching the exit code (`3T1` stays out of
scope). Each axis's `run-axis.sh` invocation is now piped through `tee -a "$ALL_OUT"` — this keeps
every `#grid/Verdict` line flowing to run-all.sh's own stdout in the exact same order a bare
`bash run-axis.sh` call would have (verified: `check-grid-speed.sh`, the one consumer that depends
on this exact stdout stream via `| tee "$OUT"`, still parses and passes — see Gates), while also
capturing the same lines into `$ALL_OUT` for tallying. After the sweep, the tally counts axes
`COMPLETED` (successful) vs `${#AXES[@]}` (attempted), total `#grid/Verdict` lines emitted, and
`:match` vs `:MISMATCH` token occurrences across the whole captured corpus — which counts
`:accuracy`, `:oracle-accuracy`, and `:port-accuracy` together, since all three fields use the same
two literal tokens and DESIGN's `3T1` cross-reference is exactly about those three never reaching
the exit code. Exact line emitted:

```
run-all: TALLY — axes $COMPLETED/${#AXES[@]} completed, $VERDICTS verdict(s) emitted, $MATCHES
:match, $MISMATCHES :MISMATCH (:accuracy + :oracle-accuracy + :port-accuracy combined)
```

**Verified live:** `GRID_SKIP_ORACLE=1 GRID_RUNS=1 run-all.sh negation user-reduce` → 6 verdicts,
tally line `run-all: TALLY — axes 2/2 completed, 6 verdict(s) emitted, 6 :match, 0 :MISMATCH
(:accuracy + :oracle-accuracy + :port-accuracy combined)`.

**Found and fixed one adjacent bug in the same block, NOT one of the four DESIGN items, flagged
here rather than silently folded in.** The pre-existing failure branch read
`echo "run-all: axis '$axis' FAILED (rc=$?) — see its stderr above"` inside `if ! bash ...; then`.
`$?` there is NOT the failing axis's exit code — `!` negates the exit status bash records, so `$?`
inside the `then` branch of `if ! cmd; then` is always the negation's own value (0, since that is
what made the branch execute), never `cmd`'s real code. Verified in isolation:
`f() { return 5; }; if ! f; then echo "rc=$?"; fi` prints `rc=0`. This diagnostic has always printed
`rc=0` on every real failure. I removed the `!` (now `if bash ... | tee ...; then COMPLETED+=1;
else axis_rc=$?; ...; fi`), which incidentally also fixes this: `axis_rc` in the `else` branch now
correctly captures the pipeline's real exit code, honoring `pipefail`. This is a stderr-diagnostic-
only fix (no exit-code, no pass/fail-outcome change) discovered only because I was already
restructuring this exact loop for the tally; not fixing it would have left a known-wrong message in
code I was actively rewriting.

## `3M2` — a header claiming an architecture the file does not have

**Re-derived the site and the number.** `check-where-shapes.sh:22-23`'s claim ("the JVM tax is paid
ONCE no matter how large the corpus grows") vs `clojure -Sdeps ... -M "$clj"` at what is now line
103, inside `check_pair()`, called once per stem in the discovery loop. Counted the actual corpus:
`ls where-*.wat | wc -l` → **38**, matching DESIGN's "38 cold JVM boots" exactly.

**Traced the actual history, not assumed it.** `git log --follow --oneline` on this file shows four
commits; `git show c8b062e64` (2026-08-01, "the where-corpus becomes one pair per family") is the
commit that split what used to be a single `where-shapes.wat`/`where-shapes.clj` pair (one JVM
boot for the WHOLE corpus — confirmed in the pre-image diff: `clojure -Sdeps "$CLARA_DEP" -M
"$GRID_DIR/where-shapes.clj"`, called once, unconditionally) into the current discovered,
one-pair-per-family shape (`clojure ... -M "$clj"` inside `check_pair()`, called once per
discovered stem). The "JVM tax paid ONCE" comment predates that commit and was never updated to
match — this is not a vague drift, it is one specific, dateable commit that changed the
architecture out from under the comment.

**Measured what the file actually does today, live, rather than reusing the stale number.** `time
bash check-where-shapes.sh` (all 38 pairs, foreground): **2m11.881s real** (38 pairs, 315 total
rows). That is ~3.47s/boot average — consistent with (not contradicting) the OLD comment's "Clara
3.7s" figure, because that 3.7s was already ~entirely one JVM boot's cost even under the old single-
boot design (per `run-axis.sh`'s own header: "one Clara cell costs ~3,500ms of which essentially
all is JVM cold boot").

**Cure applied — went beyond the single sentence DESIGN named, for internal consistency.** Found
that the SAME single-boot assumption is baked into two more places in this file's header, both of
which would have directly contradicted a fix limited to just `:22-23`:
- `:4-7` (opening synopsis): named only `where-shapes.wat`/`where-shapes.clj` as "the corpus," with
  "every row, one process" / "every row, one JVM" — true of the pre-split single pair, false of the
  current 38-pair discovered corpus.
- `:27-29` ("NO TIMING, DELIBERATELY" rationale): argued a per-row comparison would be unfair
  because "the boot is amortised across the corpus" and "row 1 pays the boot, rows 2..N do not" —
  also a pre-split-only claim; today EVERY pair pays its own boot in full, so the correct reason a
  per-pair timing comparison would be worthless is different (each pair's wall time is boot-
  dominated, not that only the first one is).

Rewrote all three passages: the opening synopsis now says "one wat process and one Clara JVM PER
PAIR (38 pairs on disk today)" and names `where-shapes.wat`/`.clj` as only the "core" pair, citing
`c8b062e64`; the JVM-tax passage states the per-stem architecture, the measured 2m11.881s/38-pair
figure, and says plainly that **batching every stem's Clara program into one JVM invocation is
available and UNMEASURED** — a real, plausible win, not built here because it is a performance
change with its own measurement, per DESIGN's pin; the "NO TIMING" rationale now argues from "every
pair pays its own boot in full" rather than the false amortisation story. All three edits are
comment-only — no code, no printed value, no test behaviour changed.

**PINNED, HONORED: did NOT batch the JVM.** No change to `check_pair()`'s control flow, no new
`.clj`/gen script, nothing that would turn this from a prose fix into an unmeasured performance
rewrite of a CI-invoked script.

---

## Gates

All smoke tests below were run against a live, current `target/release/wat` build (already
current at strike start; no rebuild was triggered by this work).

```
bash wat-scripts/perf/grid/check-where-shapes.sh where-boolean
[where-boolean] 15/15 rows agree
where-shapes: 1 pair(s), 15 rows — wat == Clara on every shape
```
Run BEFORE the `3Q2` guard was added (baseline) and AFTER (confirmed unchanged, guard did not
red it).

```
bash wat-scripts/perf/grid/check-query-compat.sh where-query-compat
[where-query-compat] 13/13 — Clara == oracle == native
query-compat: 1 family(ies), 13 rows — Clara == oracle == native
```

```
bash wat-scripts/perf/grid/check-spec-native.sh where-boolean
[where-boolean] 15/15 rows — spec == native
spec-native: 1 family(ies), 15 rows — spec == native on every shape
```
All three re-run AFTER the `3Q2` `ROWS_TOTAL` guard was added — still green, confirming the STOP
condition ("if the guard reddens any existing run") did NOT fire.

```
GRID_SKIP_ORACLE=1 GRID_RUNS=1 bash wat-scripts/perf/grid/run-axis.sh negation 100
#grid/Verdict {:axis "negation" :size [100] :accuracy :match ... :winner :us ...}
```
`3F1`'s Clara-stderr fix verified directly against an isolated repro (a deliberately-broken `.clj`
namespace): under the pre-fix code (`2>/dev/null`) the classpath error was silently swallowed
(empty stdout, nothing on stderr); under the fix, `Execution error (FileNotFoundException) ...
Could not locate broken__init.class, broken.clj or broken.cljc on classpath.` is printed. The real
axis run above confirms the success path is byte-for-byte unaffected.

```
GRID_SKIP_ORACLE=1 GRID_RUNS=1 bash wat-scripts/perf/grid/run-all.sh negation user-reduce
...
run-all: TALLY — axes 2/2 completed, 6 verdict(s) emitted, 6 :match, 0 :MISMATCH
    (:accuracy + :oracle-accuracy + :port-accuracy combined)
```
`3T2`'s tally verified live.

```
GRID_RUNS=1 bash wat-scripts/perf/grid/check-grid-speed.sh
...
run-all: TALLY — axes 11/11 completed, 33 verdict(s) emitted, 33 :match, 0 :MISMATCH
    (:accuracy + :oracle-accuracy + :port-accuracy combined)
check-grid-speed: OK — 33 cells, every one :match and above its floor
```
Full 33-cell sweep via `check-grid-speed.sh` (which calls `run-all.sh` with no axis args, so it
always does the whole grid) — the ONE consumer that depends on `run-all.sh`'s stdout being exactly
the `#grid/Verdict` stream, unaffected by the `tee` capture added for the tally. Still parses,
still 33/33 :match, still passes every per-axis speed floor.

```
time bash wat-scripts/perf/grid/check-where-shapes.sh
[38 pairs, one line each] ...
where-shapes: 38 pair(s), 315 rows — wat == Clara on every shape

real    2m11.881s
user    7m43.397s
sys     0m9.954s
```
Full 38-pair run, both as the `3M2` measurement (~3.47s/boot average) and as a second full-corpus
confirmation the `3Q2` guard doesn't red anything at scale.

**The floor**, foreground, per house rule (never backgrounded, never piped through `head`/`tail`
for the exit code):

```
./scripts/floor.sh
     Summary [ 454.203s] 5485 tests run: 5485 passed (1 slow), 19 skipped
```
Read from `.floor/latest/clean.log`'s `Summary` line. **5485/5485 — matches the expected 5485
exactly.** No `.floor/latest/ARM.txt` (only written on a failure) — confirmed by directory listing
(`clean.log` + `raw.log` only), not inferred from the Summary line alone. Run ONCE, after both
commits' worth of changes were smoke-tested individually above — no red, no re-run.

## Commits

1. `96d409985` — "grid: ROWS_TOTAL non-vacuity guard (3Q2) + Clara stderr capture in
   run-axis.sh (3F1)" — behaviour-changing half.
2. `dd57af2d0` — "grid: build the promised run-all.sh tally (3T2) + correct the JVM-tax
   claim (3M2)" — no-behaviour-change half.

Both on branch `grok-rete`. `check-where-shapes.sh` is the one file touched by both commits; the
two hunks were staged separately with `git add -p` (confirmed via `git diff --cached` before each
commit) so commit 1 carries only the `ROWS_TOTAL` guard from that file and commit 2 carries only
the three comment corrections.

## What this did NOT do

- **Did not batch the JVM boots in `check-where-shapes.sh`** — pinned out of scope by DESIGN; the
  comment now states plainly that it is available and unmeasured, per DESIGN's instruction to make
  that an affirmative statement rather than a silent omission.
- **Did not touch the exit code of `run-all.sh`** — `3T1`'s territory, explicitly out of scope; the
  tally is stderr-only.
- **Did not wire `ACCURACY`/`ORACLE_ACCURACY`/`PORT_ACCURACY` to any exit code anywhere** — same
  reason, `3T1`.
- **Did not pursue `3W1`'s JDK provenance or `3P1`'s axis-pair coverage** — both explicitly out of
  scope per DESIGN, both need their own measurement-backed strike.
- **Did not re-run the floor speculatively** — ran it once, foreground, after all four items were
  cured and smoke-tested individually.

## Deltas from DESIGN, gathered in one place

1. **`3Q2`**: DESIGN's motivating example (`PAIRS=5, ROWS_TOTAL=0` printing GREEN) does not
   reproduce in the code as it stands — all three scripts already carry a per-pair `wn`/`nn -lt 1`
   check (present since each file's original authorship, not a recent add) that fails BEFORE a
   zero-row pair can contribute to `ROWS_TOTAL`, so `FAILED=0` already implies `ROWS_TOTAL >= 1`
   today. The prescribed cure was still applied, as genuine defense-in-depth (the pattern
   `check-grid-speed.sh` itself already uses), and confirmed to change nothing about any current
   green run.
2. **`3T2`**: found and fixed one bug in the same block DESIGN didn't name — `run-all.sh`'s
   failure-branch diagnostic (`rc=$?` inside `if ! cmd; then`) has always printed `rc=0` on every
   real failure, because `!` negates the value `$?` captures. Fixed as a side effect of removing
   the `!` while restructuring the loop for the tally; a stderr-diagnostic-only change.
3. **`3M2`**: DESIGN named `:22` as the site; re-deriving found the same stale single-boot
   assumption also baked into the file's opening synopsis (`:4-7`) and its "NO TIMING" rationale
   (`:27-29`), both of which would have directly contradicted a fix limited to `:22-23` alone. All
   three corrected together, comment-only.
