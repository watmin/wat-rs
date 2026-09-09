# SCORE — the grid records its instrument

Executing strike per `DESIGN.md`. Written as-I-go per house rule.

## Re-derivation, before any edit

Read `DESIGN.md` in full, then `wat-rs/CLAUDE.md` in full (floor discipline, no-known-flakes,
scratch-`.wat` location, wat-fix codemod rule — none of it reaches a rider any other way).

Checked DESIGN's own claims independently rather than trusting them:

- `cd wat-scripts/perf/grid && ls GRID-*.txt | wc -l` → **29**.
  `grep -ilE 'jdk|temurin|openjdk|java ' GRID-*.txt` → **0 matches**. Confirmed: 29 of 29 recorded
  grids name no JDK, no distribution, no Clojure CLI version.
- `echo $JAVA_HOME` → `/home/john/opt/jdk-21`; `readlink -f $(which java)` →
  `/home/john/opt/jdk-21.0.12+8/bin/java`; `java -version` → `Temurin-21.0.12+8`. Confirmed
  independently: this box runs the same distribution and major version CI pins, reached through
  the `$HOME/opt/jdk-*` path DESIGN names.
- `.github/workflows/ci.yml:177-178` and `:226-227` — `distribution: temurin`,
  `java-version: "21"`, both parity-job `setup-java` steps. Confirmed.
- **Found nothing that overturns DESIGN's correction.** No grid names a JDK (checked all 29, not
  a sample); no evidence turned up that the 2026-08-27 capture ran under a *different* JDK than
  this box's current one. DESIGN's framing stands as written: plausible, not provable.

## What "whatever emits GRID-*.txt" turned out to be

Grepped for any script that names or generates a `GRID-*.txt` filename — found none.
`wat-scripts/perf/grid/run-all.sh` is the actual instrument: every recorded grid in this directory
was a human running `bash run-all.sh > GRID-native-vs-clara-$(date ...).txt` by hand and committing
the redirect target (confirmed against `bbeb1997d`'s commit message, which describes exactly this).
There is no separate "capture" wrapper to instrument — `run-all.sh`'s own stdout **is** the file
once redirected, so that is where the header had to land.

## Deliverable 1 — the grid records its instrument

Added a header block to `wat-scripts/perf/grid/run-all.sh` (before the axis sweep loop, after
`AXES` is resolved) that prints one `#grid/Capture {...}` EDN line to **stdout**, once, before any
`#grid/Verdict` line:

```
#grid/Capture {:captured-at "2026-09-09T09:19:43Z" :java-version "openjdk version \"21.0.12\" 2026-07-21 LTS OpenJDK Runtime Environment Temurin-21.0.12+8 (build 21.0.12+8-LTS) OpenJDK 64-Bit Server VM Temurin-21.0.12+8 (build 21.0.12+8-LTS, mixed mode, sharing)" :clojure-cli-version "Clojure CLI version 1.12.4.1582" :clara-version "0.24.0" :host "reason"}
```

Fields:
- `:captured-at` — UTC timestamp of the run.
- `:java-version` — full `java -version` output (all three lines, joined; embedded `"` chars from
  `java -version`'s own output are EDN-escaped so the whole line stays parseable, not truncated at
  the first embedded quote — this was wrong in the first draft and fixed before landing, see
  below).
- `:clojure-cli-version` — `clojure --version` output.
- `:clara-version` — **not** a second hardcoded literal. Read directly out of `run-axis.sh`'s own
  `CLARA_DEP` (`sed` extraction of the `:mvn/version` string), so the Clara pin has exactly one
  source of truth instead of two copies that can drift apart — the same class of defect this
  strike exists to fix, so I did not want to re-create a smaller instance of it in the cure.
- `:host` — short hostname, for a multi-box future.

Design choice: this is emitted by `run-all.sh` itself, not a new wrapper script, because
`run-all.sh` is what a human already redirects into `GRID-*.txt` — the next `bash run-all.sh >
GRID-....txt` is self-describing for free, no new convention to remember or forget.

**Verified both existing consumers are inert to the new line, by reading their parsers and then
driving them:**
- `check-grid-speed.sh`'s sweep loop: `case "$line" in \#grid/Verdict*) ;; *) continue ;; esac` —
  skips any non-`#grid/Verdict` line. Extracted the loop logic and ran it by hand against a real
  captured file (below): `seen=3` (the 3 real verdicts), header correctly not counted.
- `compare-grids.sh`'s awk: matches only `/#grid\/Verdict/`. Ran it against a real captured file;
  the header line does not appear as a row, `ONLY-OLD`/`ONLY-NEW`, or in any tally.

**Drove the whole thing end-to-end**, not just the extracted snippet — `target/release/wat` was
already fresh (Sep 9 build), no other cargo/nextest process running (`pgrep -af 'cargo|nextest'`
checked clear first):

```
$ GRID_SKIP_ORACLE=1 GRID_RUNS=1 timeout 180 bash wat-scripts/perf/grid/run-all.sh negation
#grid/Capture {:captured-at "2026-09-09T09:19:43Z" :java-version "openjdk version \"21.0.12\" 2026-07-21 LTS OpenJDK Runtime Environment Temurin-21.0.12+8 (build 21.0.12+8-LTS) OpenJDK 64-Bit Server VM Temurin-21.0.12+8 (build 21.0.12+8-LTS, mixed mode, sharing)" :clojure-cli-version "Clojure CLI version 1.12.4.1582" :clara-version "0.24.0" :host "reason"}
#grid/Verdict {:axis "negation" :size [250] :accuracy :match :runs 1 :ratio 49.9911 ...}
#grid/Verdict {:axis "negation" :size [500] :accuracy :match :runs 1 :ratio 34.9876 ...}
#grid/Verdict {:axis "negation" :size [1000] :accuracy :match :runs 1 :ratio 30.2467 ...}
exit=0
```

(First attempt at this drive hit `run-axis.sh`'s pre-existing freshness-wall refusal — an
unrelated rebuild had just made the binary current and hot; waited, re-ran, got the clean pass
above. Not a defect in this change; documented so the exit-code history isn't mysterious.)

A bug was caught and fixed here before landing: the first draft did not EDN-escape
`java -version`'s own embedded `"21.0.12"` quoting, which produced a `#grid/Capture` line whose
`:java-version` string terminated at the first embedded `"`, leaving the rest of the line as
unparseable trailing text. Fixed with an `edn_escape` helper (`sed 's/\\/\\\\/g; s/"/\\"/g'`)
applied to both `:java-version` and `:clojure-cli-version` before they're interpolated. Verified
fixed by re-running the extraction, not by re-reading the code — the corrected output above shows
`\"21.0.12\"` properly escaped inside the string.

## Deliverable 2 — the FLOOR comment states what is and is not known

Rewrote the block above the `FLOOR` array in `wat-scripts/perf/grid/check-grid-speed.sh`. Exact
text added (a new section, kept the pre-existing "THE FLOORS ARE THE ARTIFACT" paragraph above it
unchanged):

```
# ── THE CAPTURE'S JDK — KNOWN, EVIDENCE, AND WHAT SUPERSEDES IT (`3W1`) ────────────────────────
#
# KNOWN: `GRID-native-vs-clara-2026-08-27T07-15-56Z.txt` — the grid the FLOOR array below derives
# from — names no JDK and no Clojure CLI version anywhere in the file. Neither does any other
# recorded `GRID-*.txt` in this directory (29 of 29, checked 2026-09-09; `grep -il
# 'jdk|temurin|openjdk|java '` over all of them returns nothing). CI pins `distribution: temurin`,
# `java-version: "21"`. Whether the 2026-08-27 capture ran under that JDK, a different major
# version, or a different distribution entirely was never recorded, so it cannot be re-derived,
# defended, or refuted from the file cited above.
#
# EVIDENCE, NOT PROOF: the box this capture was measured on runs, TODAY (checked 2026-09-09, not
# at capture time), `Temurin-21.0.12+8` via `JAVA_HOME=$HOME/opt/jdk-21` — the same distribution
# and major version CI pins, reached through the ordinary `$HOME/opt/jdk-*` discovery this box
# still uses. That makes it PLAUSIBLE the capture ran under the JDK CI now pins. It is NOT proof:
# a box's JDK today is not evidence of that same box's JDK on 2026-08-27, and nothing ties the two
# together. Read this floor as "measured on an unrecorded JDK, on a box that currently happens to
# run the one CI pins" — never as "measured on Temurin 21."
#
# WHAT SUPERSEDES THIS: `run-all.sh` now emits a `#grid/Capture` header (java -version, Clojure
# CLI version, the pinned Clara version, capture timestamp) as the first line of its stdout, so
# the next full grid recorded through the ordinary `bash run-all.sh > GRID-....txt` capture is
# self-describing by construction. THAT grid — not this comment, not a fresh read of the box's
# CURRENT `java -version` — becomes the provenance-bearing baseline once it exists. Re-deriving
# these floor numbers under it is deliberately NOT done here: it is a performance act needing six
# samples, a quiet box, and its own scorecard, not a recording-mechanism fix.
```

Also updated the one-line comment directly above the `FLOOR` declaration itself:
`# axis : floor (~50% of that axis's minimum ratio in the 2026-08-27 grid — a grid whose own JDK
was never recorded; see "THE CAPTURE'S JDK" above)`.

**Checked against the DESIGN's explicit prohibition**: the new text never states or implies the
floors were measured on Temurin 21 — every sentence that touches the box's current JDK is
qualified as "today," "plausible," "not proof," or explicitly contrasted with what would be proof
(a `#grid/Capture`-bearing grid).

## New GRID-*.txt — generated for the drive above, NOT committed

The end-to-end drive in Deliverable 1 produced one real capture (`negation` axis, `GRID_RUNS=1`,
3 verdicts) to prove the header mechanism works against the real script, not just an extracted
snippet. **Decision: does not belong in the commit.** Reasons:
- It is a partial, single-axis sweep at `GRID_RUNS=1` — not comparable to any of the 29 full
  33-cell, `GRID_RUNS=3` captures (the file's own header says a different-ladder grid is
  incomparable to the ones before it; this is also a different run-count, which `compare-grids.sh`
  itself refuses to compare).
- Nothing references it. Adding a 30th `GRID-*.txt` that no script, test, or comment cites is
  exactly the `4G1` shape named in the house rules — a golden nothing regenerates or reads.
- It served its purpose (proving the mechanism) and is left in the session scratchpad, not this
  repo:
  `/tmp/claude-1000/-home-john-work-holon/fd01e281-0457-4e4a-a481-acd7beca46ad/scratchpad/test-grid-header2.txt`.

`git status --short` after all edits shows exactly the two intended files modified, nothing else:

```
 M wat-scripts/perf/grid/check-grid-speed.sh
 M wat-scripts/perf/grid/run-all.sh
```

## Re-derivation of the floor numbers themselves

**Not done, per the pin.** DESIGN and the task both mark this explicitly out of scope — re-measuring
needs six samples, a quiet box, and its own scorecard, and doing it here would produce a second set
of numbers with recorded provenance while leaving unfixed exactly the mechanism (deliverable 1)
that lost provenance the first time. I did not find, during this session, any reason to override
that pin — nothing surfaced suggesting the current floors are wrong, only that their instrument was
unrecorded. A future capture under the now-self-describing `run-all.sh`, immediately compared
against the existing floors via `compare-grids.sh`, is the natural next strike; it is left undone
here on purpose.

## Gates run

Targeted, before the full floor, to catch a cheap red first (`pgrep -af 'cargo|nextest'` checked
clear before each):

```
$ cargo nextest run --release every_parity_script_is_invoked
PASS wat::lint every_parity_script_is_invoked::every_parity_script_is_invoked_by_ci_or_a_test
Summary [   0.047s] 1 test run: 1 passed, 5508 skipped
```

```
$ cargo nextest run --release -E 'test(every_wat_scripts_file_loads_on_the_current_runtime) or test(every_rete_name_in_wat_scripts_code_resolves)'
PASS wat::lint rete_names_in_wat_scripts_resolve::every_rete_name_in_wat_scripts_code_resolves
PASS wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime
Summary [ 135.970s] 2 tests run: 2 passed, 5507 skipped
```

(Neither of these tests reads `.sh` files, and this strike touched no `.wat` file — both are pure
sanity checks confirming the shell edits didn't somehow red the wat-scripts corpus gates the house
rules flag as walking this directory.)

## Floor

Ran `./scripts/floor.sh` in the FOREGROUND (not backgrounded — `pgrep -af 'cargo|nextest'` checked
clear immediately before, and the command was run without `run_in_background`). Clean on the first
pass — no red to capture at any point in this strike. Read from `.floor/latest/clean.log`:

```
     Summary [ 450.943s] 5490 tests run: 5490 passed, 19 skipped
```

**5490 — exactly the pinned number.** `.floor/2026-09-09T09-23-29Z/` (symlinked as `.floor/latest`)
holds the untruncated log.

## What I did NOT do

- Did **not** re-derive or touch any `FLOOR` array value. Only the comment above it changed.
- Did **not** claim, anywhere in the new comment, that the floors were measured on Temurin 21 —
  every claim about this box's current JDK is explicitly marked as evidence, not proof, per the
  DESIGN's own pin.
- Did **not** back-fill JDK provenance into any of the 29 existing `GRID-*.txt` files — DESIGN
  rejects this explicitly ("unknowable; inventing it would be worse than the gap").
- Did **not** add the speed gate to CI — out of scope per DESIGN.
- Did **not** commit the demonstration `GRID-*.txt` capture — see "New GRID-*.txt" above.
- Did **not** create a new wrapper/capture script — the header lives in `run-all.sh` itself, the
  actual instrument every existing `GRID-*.txt` came from.
- Did **not** re-run any test after a red — there was no red anywhere in this session.
- Did **not** find any evidence overturning DESIGN's correction (that CI's JDK pin was "never
  checked against" the floors is not established). Checked independently; found nothing new.

---

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
Claude-Session: https://claude.ai/code/session_018ntHDMRNCKDKNr2gVfzXmP
