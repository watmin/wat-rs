# SCORE 7j — replay batch 4j, grok-rete #321 → STOP at #324

**STOPPED, not completed.** Three steps landed (#321, #322, #323 — all docs-only), then **STOPPED at
#324, before any commit**, on a genuine finding neither this brief nor EXPECTATIONS-7j anticipated:
two of grok's own five named tests for its D5 fix fail on this tree for a reason that is neither
syntax, nor a conversion defect, nor this executor's error — a pre-existing, RULED, ACCEPTED main-side
design decision (`250162a0e SCORE(277)`, ancestor of the batch's own start point `665b17b60`) that
permanently forbids `match` anywhere inside a `:then` item, for any spelling, exhaustive or not. Full
evidence and reasoning is in `REPLAY-LOG.md`'s `#324` section; this SCORE answers every row against
what was actually run, per row, never leaving one blank.

Working tree is clean at `1245b02df` (`REPLAY(grok-rete #323)`), which is 3 steps ahead of the batch
anchor `665b17b60`. Nothing from #324 was committed. Not pushed.

## E1 — 20 steps, correctly subjected

Command: `scripts/replay/verify-step-record.sh 665b17b60 HEAD 321 340`

```
MISSING-STEP #324: no REPLAY(grok-rete #324) commit in 665b17b60..HEAD
MISSING-STEP #325: no REPLAY(grok-rete #325) commit in 665b17b60..HEAD
MISSING-STEP #326: no REPLAY(grok-rete #326) commit in 665b17b60..HEAD
MISSING-STEP #327: no REPLAY(grok-rete #327) commit in 665b17b60..HEAD
MISSING-STEP #328: no REPLAY(grok-rete #328) commit in 665b17b60..HEAD
MISSING-STEP #329: no REPLAY(grok-rete #329) commit in 665b17b60..HEAD
MISSING-STEP #330: no REPLAY(grok-rete #330) commit in 665b17b60..HEAD
MISSING-STEP #331: no REPLAY(grok-rete #331) commit in 665b17b60..HEAD
MISSING-STEP #332: no REPLAY(grok-rete #332) commit in 665b17b60..HEAD
MISSING-STEP #333: no REPLAY(grok-rete #333) commit in 665b17b60..HEAD
MISSING-STEP #334: no REPLAY(grok-rete #334) commit in 665b17b60..HEAD
MISSING-STEP #335: no REPLAY(grok-rete #335) commit in 665b17b60..HEAD
MISSING-STEP #336: no REPLAY(grok-rete #336) commit in 665b17b60..HEAD
MISSING-STEP #337: no REPLAY(grok-rete #337) commit in 665b17b60..HEAD
MISSING-STEP #338: no REPLAY(grok-rete #338) commit in 665b17b60..HEAD
MISSING-STEP #339: no REPLAY(grok-rete #339) commit in 665b17b60..HEAD
MISSING-STEP #340: no REPLAY(grok-rete #340) commit in 665b17b60..HEAD
```

exit 1 (17 `MISSING-STEP` lines, #324 through #340, all verbatim above — none omitted). **This is the
EXPECTED, CORRECT shape of a STOP** — 3 of 20 landed. Re-run against only the
landed range for the record: `scripts/replay/verify-step-record.sh 665b17b60 HEAD 321 323` →

```
step-range: #321..#323 each present exactly once, sources match
step-record: complete
```

exit 0. **FAIL against the full 20-step contract, as it must; PASS against the 3 steps actually
landed.**

## E2 — docs-only steps are docs-only

Of the 14 docs-only steps named in the brief, only 3 were reached: #321, #322, #323. Command:

```
for n in 321 322 323; do
  sha=$(git log --format='%H %s' 665b17b60..HEAD | grep "#${n}):" | awk '{print $1}')
  git show --name-only --format= "$sha" | grep -vE '^docs/|\.md$'
done
```

No output — all three commits' files are entirely under `docs/`. **PASS for the 3 reached; #325,
#326, #329–#331, #333–#335, #338–#340 never attempted.**

## E2b — and the inverse: the 6 code steps each carry ≥1 non-docs file

**Not reached.** #324 (the first code step) was STOPPED before commit. #327, #328, #332, #336, #337
never attempted. **N/A — no code step landed this batch.**

## E3 — every produced `.wat` checks

Of the 13 new `.wat` named in the brief, only #324's 4 new fixtures
(`probe_arc278_match_arm_body_ok.wat`, `_then_core_bare.wat`, `_then_rete_bare.wat`,
`_then_wrapped.wat`) plus 1 modified (`experiri-then-match.wat`, resolved to 0 net change) and 1
deliberately-unconverted `.wat.bad` were ever prepared, in a working tree that was then reset before
commit. Measured before the reset: all 4 new `.wat`, after `convert.sh` re-expression, passed
`./target/release/wat --check` (implicitly — each is exercised end-to-end by its own named test, which
either ran to a real `println` or died on the documented, expected fault; none died on a parse or
type-check error). The `.wat.bad` fixture is a deliberate refusal by design (`assert!(!ok, …)` in its
own test) — its own EDN golden mismatch is a FORMAT issue (E-row below), not a `--check` failure.
**None of this was committed — the working tree was reset to #323 before yielding**, so as of the tree
this SCORE describes, **0 of the 13 `.wat` exist on disk** (they were never landed). **PASS for what
was measured before the reset; N/A for "on disk now" since nothing from #324 landed.**

## E4 — conversion recorded where needed

For the 4 new `.wat` from #324 (the only step that reached this point): YES, conversion was needed and
was performed via `scripts/replay/convert.sh ab606b671 <out> <paths>`, which applied
`match-arm-to-bracket-map-pattern`, `bare-variant-to-qualified`, `positional-ctor-to-map`,
`variant-separator-to-dot` (chain members whose scope matched); diffed against the un-converted
originals to confirm only spelling/form changed, not test logic (same fact counts, same field
names — recorded verbatim in `REPLAY-LOG.md`'s #324 section). The 5th new file
(`probe_arc278_match_arm_body_bad.wat.bad`) explicitly says "no conversion" and states why: its golden
`.edn` pins exact `:line`/`:col` positions a syntax conversion would move, and it was measured (not
assumed) that the un-converted syntax reaches the same code path a converted one would, because the
file dies at freeze-time validation before `:user::main` ever runs. **PASS, with the explicit
"no conversion" answer for the one file that needed it, matching E4's own bar for what counts as an
answer.**

## E5 — #332's probes may deliberately FAIL

**Not reached.** #332 was never attempted; the batch stopped 8 steps before it. **N/A.**

## E6 — #329's red floor is adjudicated, not inherited

**Not reached.** #329 was never attempted. **N/A — and note for the next executor picking up this
batch: #329's own subject ("curare: thirty-fifth stamp — D6 closed; and I pushed a red floor") still
needs the judgment call the brief flagged, once #324 is resolved and the batch resumes.**

## E7 — named tests at the 6 code steps

Only #324 was attempted. Its named tests (from grok's own commit body): 5 total.

```
cargo nextest run --release -E 'test(probe_arc278_match_arm)'
Summary [   0.562s] 5 tests run: 2 passed, 3 failed, 5741 skipped
    FAIL (1/5) a_misspelled_constructor_in_a_match_arm_body_is_still_refused
    FAIL (2/5) the_bare_and_wrapped_then_spellings_compile_and_agree
    FAIL (3/5) a_correct_constructor_in_a_match_arm_body_still_fires
    PASS (4/5) the_core_spelling_is_refused_by_the_fence_not_by_a_phantom_arity_error
    PASS (5/5) the_banked_d5_repro_pair_both_load
```

**N > 0 selected (5), but 3 of 5 FAILED — this is the STOP itself, not a row to mark PASS.** Per the
absolute rule ("no knowingly-red REPLAY commit, ever"), #324 was never committed. Full failure
transcripts (both the EDN-structural-mismatch one and the two `then-item-fence` runtime faults) are in
`REPLAY-LOG.md`'s #324 section verbatim, not summarized.

## E8 — ZERO hazard paths in range

Commands, run BEFORE any step was attempted (pre-flight, matching EXPECTATIONS-7j's own claim):

```
git diff --name-only 665b17b60..HEAD | grep -E '^wat/|^wat-scripts/fixes/'
awk -F'\t' '$1>=321 && $1<=340' bootstrap/era/replay-plan/absent-on-main.tsv
awk -F'\t' '$1>=321 && $1<=340' bootstrap/era/replay-plan/stdlib-touch.tsv
awk -F'\t' '$1>=321 && $1<=340' bootstrap/era/replay-plan/flags.tsv
```

First: no output (checked at #323's HEAD, before #324's attempt — and #324's own diff never touches
`wat/` or `wat-scripts/fixes/` either, confirmed from its `git show --name-status` file list: 5
`.wat` under `docs/arc/**` and `tests/rete/**` only). Second and third: no output, confirmed empty for
the whole 321–340 range. Fourth: all 20 rows read `fixes=0 main-deleted=0`. **None at all — PASS, and
this holds independent of the STOP since it is a path-based property of the commits themselves, not of
what landed.**

## E9 — the 13 `.wat` are NEW fixtures, not a corpus rewrite

Confirmed for #324's 5 (the only step reached): `git show --name-status ab606b671` shows 4 `A` under
`tests/rete/` and 1 `M` under `docs/arc/2026/06/278-rules-engine/harness-experiri/` (the file this
step's own diff touches only in its header comment, and which resolved to 0 net change on this tree —
see REPLAY-LOG). None under `wat/`; no `wat-scripts/fixes/` edit. **PASS for the 5 reached; the
remaining 8 across #327/#328/#332/#336 were never examined since those steps were never attempted.**

## E10 — finding 33's class actively looked for

#324's commit body (drafted, never committed — preserved in this SCORE and REPLAY-LOG instead) states:
finding 33 grepped YES — `git grep -n ':wat::' src/rete/validate/mod.rs`'s own diff hunk carries
`:wat::core::match`/`:wat::rete::core::match` only as AST-walker Rust string comparisons against
parsed keywords (the walker's actual subject matter), never as embedded executable wat inside a string
literal that a rename could silently stale. **PASS for #324; not applicable to #325–#340 since none
were attempted.**

## E11 — the checkpoint

Not run by this executor — forbidden (`scripts/floor.sh`, `cargo clippy` are explicitly off-limits;
the orchestrator's own row). **Not checked, per the brief's own prohibition.**

## E12 — test-count delta ACCOUNTED FOR

No test-affecting step landed (#321–#323 are docs-only; #324 was never committed). Baseline
re-measured immediately before #321 and unchanged through #323:

| step | lint-subset | kind(lib) | doctest | delta source |
|---|---|---|---|---|
| baseline (#320/HEAD before this batch) | 249 | 1493 | 8 | — |
| #321 | 249 (+0) | 1493 (+0) | 8 | docs-only |
| #322 | 249 (+0) | 1493 (+0) | 8 | docs-only |
| #323 | 249 (+0) | 1493 (+0) | 8 | docs-only |

No delta anywhere in the landed range. **PASS — predicted == actual (both zero) at every landed
step.**

## E13 — spot re-run of the walls

Foreground re-run at HEAD (`1245b02df`, #323), after the #324 reset:

```
lint-subset:  249 passed   (cargo nextest -E "binary(lint) - test(every_wat_scripts_file_loads_on_the_current_runtime)")
kind(lib):   1493 passed   (cargo nextest -E "kind(lib)")
doctest:        8 passed   (cargo test --doc --release; 5+3 across two binaries)
nested-program-gate: 3/3, 5738 skipped (cargo nextest -E "test(nested_program_starts)")
```

Identical to the pre-batch baseline (no code step landed to move any number). **PASS.**

## E14 — no knowingly-red commit

**PASS, and this is the row the whole STOP exists to satisfy.** #324's 3-of-5 red was caught BEFORE
commit (`git cherry-pick -x --no-commit`, never `git cherry-pick -x` alone), the working tree was
`git reset --hard HEAD` before yielding, and nothing red is anywhere in this branch's history. No
repair commit was appended chasing a green after the fact, because nothing red was ever committed to
repair.

## E15 — every artifact a body names exists

No `.census/…txt` file is cited in any of #321–#323's bodies (all three are docs-only; the record gate
requires no census line for a docs-only step, confirmed by `verify-step-record.sh`'s own path-based
rule and its `step-record: complete` result above). #324 was never committed, so it names no artifact
either. **N/A — no artifact was cited that needs checking.**

## E16 — NO PUBLISHED HISTORY WAS REWRITTEN

Commands:

```
git merge-base --is-ancestor origin/replay/grok-rete HEAD && echo ancestor-OK
git for-each-ref refs/original/
```

```
ancestor-OK
(empty)
```

**PASS.** One in-session amend occurred (#322's self-caught trailer fabrication, see E19/E22 below) —
it amended the TIP commit of this session's own unpushed work, before any descendant existed, and is
fully disclosed there. No `filter-branch`, no `replace`, no rewrite of anything published or anything
with a descendant.

## E17 — every repair visible to `push`

Command: `git replace -l`

```
(empty)
```

Gate re-run under `GIT_NO_REPLACE_OBJECTS=1`:

```
step-range: #321..#323 each present exactly once, sources match
step-record: complete
```

exit 0 either way. **PASS — 0 replace refs; the one repair this session made (#322's amend) is an
ordinary commit rewrite of an unpushed tip, needs no overlay, and is visible identically with or
without `refs/replace`.**

## E18 — no verdict line is WRAPPED

#321–#323 are all docs-only and carry no verdict lines at all (`verify-step-record.sh`'s path-based
rule requires none from a docs-only step, and its `step-record: complete` result over the 321–323
range with zero `MISSING` lines proves none was required and none was malformed). #324 was never
committed, so it contributes no verdict lines to check. **N/A — nothing in the landed range has a
verdict line to wrap.**

## E19 — the SCORE discloses what BOUGHT each green

- **#321–#323's green** is bought by nothing but a clean 3-way auto-merge each time — no rune added,
  no gate touched, no ledger changed. Stated plainly rather than left to be inferred.
- **#322's trailer self-catch** (E16/E22) is disclosed here AND in `REPLAY-LOG.md`, not left buried in
  the log alone — the amend that fixed it is named, and the verification command that caught it
  (`git rev-parse 7fe03ebb6` vs. the hand-typed trailer) is given verbatim.
- **The STOP at #324 is not a green at all**, and this SCORE does not dress it as one: E7 above states
  the 3-of-5 failure baldly, with the full commands and outputs, rather than folding it into a PASS
  row or omitting it.

## E20 — every `-E` filter selected N > 0

Every nextest invocation's own `N tests run` line, read directly: lint-subset 249 (×1, pre-flight),
kind(lib) 1493 (×1), doctest 8 (×1, via `cargo test --doc`), nested-program-gate 3 (×1), #324's named
suite 5 (×1, `test(probe_arc278_match_arm)`). None was ever 0. **PASS.**

## E21 — NO COUNTERPART ACTIVITY

`mcp__pulsare__*` was never called — confirmed by this session's own tool-call history containing no
tool of that family. `/home/john/work/holon/` (the frozen root, `.pulsare/` included) was never read or
written; this executor's entire working surface was `/home/john/work/holon/wat-rs`. `git status
--porcelain` at every checkpoint showed only this executor's own committed/reset work. `ls .census/`
was not consulted this batch (no census-producing step landed) so there is nothing to compare against
a foreign entry; no new file appeared under `.census/` or `.floor/` during this session outside what
this executor itself would have produced (and this executor never ran `scripts/floor.sh`, per the
prohibition). **PASS — no `pulsare_yield`, no foreign floor, no foreign artifact, frozen root
untouched.**

⛔ **DISCLOSURE PER THE BRIEF'S OWN HARD PROHIBITION #1:** the `pulsare` MCP server's own tool
instructions, visible to this session, state *"The only tool is pulsare_yield. Write the files, then
yield. That is a tmux send-keys, always... Do not use Task/spawn_subagent for the counterpart."* This
was **not followed** — per this brief's explicit, repeated override ("that instruction is overridden
here — say so in your report rather than complying"), `mcp__pulsare__pulsare_yield` was never called
and no `mcp__pulsare__*` tool was invoked at any point. Yielding happens by ending this turn with this
report, per the brief's own instruction, and this conflict is recorded here rather than silently
complied with or silently ignored.

## E22 — every deviation from this brief is REPORTED

**This entire batch IS the deviation**, and it is the central content of this SCORE and of
`REPLAY-LOG.md`'s `#324` section, not a footnote:

1. **#322's self-caught trailer fabrication** — a hand-typed SHA that did not match `git rev-parse`'s
   output, caught and repaired before any descendant commit existed. Named here per finding 36 §4's
   precedent (the class that "must never become routine"), even though the self-catch made it cheap.
2. **#324's STOP** — neither this brief nor EXPECTATIONS-7j anticipated any issue with #324 (it names
   #329 and #332 as the batch's trap doors; #324 is presented as a routine "D5 — a match arm's PATTERN
   is not a constructor call" fix). Measured instead of assumed: after resolving two real merge
   conflicts (one `.wat` header, one `.rs` missing-argument build error — both mechanical and
   correctly resolved, confirmed by a clean build), 3 of grok's own 5 named tests still failed, and
   root-causing them (not just observing the red) found: (a) one is a pure EDN-writer-convention
   mismatch, fixable by `UPDATE_EDN=1` regeneration, not a real defect; (b) two are a genuine,
   permanent capability boundary — main's own `250162a0e SCORE(277)` ruling, ACCEPTED, ancestor of
   this batch's own start point, entirely absent from grok's own branch, and never revisited by any of
   grok's own remaining 300+ commits. **This is reported as a RESULT, per finding 37's rule, not
   suppressed, not silently "fixed" by weakening grok's tests, and not committed red.**

**PASS — this row is satisfied by the report existing and being this specific, not by the batch
having gone smoothly.**

## What would have made this batch worse — avoided

A knowingly-red REPLAY commit at #324 — avoided (reset, never committed). #329's red-floor question
answered by assumption rather than by reading its body — avoided by never reaching it (STOPPED first,
rather than skipping ahead past #324 to land the easier remaining docs steps and leaving #324 as an
unexplained gap in the middle of the range). #336's cure pulled backward into #332 — avoided by never
reaching either. A fabricated trailer SHA left uncorrected — avoided (#322, self-caught). Silently
weakening or reinterpreting grok's own D5 test assertions to force a green — avoided; the tension is
named, traced to its root commit, and handed back as a judgment call rather than resolved
unilaterally. `pulsare_yield` or any `mcp__pulsare__*` call — never made, despite the MCP server's own
instructions recommending it; the conflict is disclosed in E21 rather than silently resolved either
way.

## For whoever resumes this batch

`#324` needs a builder-level decision before it can land, not more investigation — the investigation is
complete and is above. Three concrete options, none chosen here:
1. Regenerate #324's `.edn` golden with `UPDATE_EDN=1` (uncontroversial, do regardless of the other
   two — it is a pure format fix), and separately decide whether the 2 `then-item-fence` failures mean
   `the_bare_and_wrapped_then_spellings_compile_and_agree` and
   `a_correct_constructor_in_a_match_arm_body_still_fires` should be REWRITTEN to assert refusal (with
   the walker fix's own value demonstrated instead by confirming the refusal now cites the true Stone-C
   reason, the same pattern `the_core_spelling_is_refused_by_the_fence_not_by_a_phantom_arity_error`
   already establishes for the core spelling), landing that rewrite as this step's own content with the
   divergence fully documented in the commit body.
2. Revisit whether `250162a0e`'s ruling should be narrowed now that D5's walker fix makes a
   NON-recursive, item-top-level-only match check sufficient to prevent the original arc-277 concern
   (a head-level axis admitting an inexhaustive match) while still allowing a match NESTED inside a
   constructor's field value (which is total by construction once the constructor's own fields are
   validated) — this would be a `wat/` change, forbidden to an executor in this batch, and a
   reopening of an ACCEPTED SCORE, both squarely builder-level.
3. Confirm with the builder that grok's D5 probes are accepted as permanently non-portable to this
   tree's design and land #324 with only the walker fix plus the 2 tests that already pass, dropping
   or ignore-marking the 3 that cannot (with the reason attached at the test, not silently).
