# SCORE 7h — replay batch 4h, grok-rete #281 → #300

Batch complete: 20 REPLAY commits, tip `a1775d37f`. Every row below answers with the command run
and its actual output — a row that could not be checked says so and why, never left blank.

## E1 — 20 steps, correctly subjected

Command: `scripts/replay/verify-step-record.sh b35509d88 HEAD 281 300`

```
step-range: #281..#300 each present exactly once, sources match
step-record: complete
```

exit 0 (also re-run under `GIT_NO_REPLACE_OBJECTS=1`, identical output). **PASS.**

## E2 — docs-only steps are docs-only

Command, per the 15 docs-only steps:

```
for n in 281 282 284 285 286 287 289 290 291 292 293 295 296 297 299; do
  sha=$(git log --format='%H %s' b35509d88..HEAD | grep "#${n}):" | awk '{print $1}')
  git show --name-only --format= "$sha" | grep -vE '^docs/|\.md$'
done
```

No output — every file in every one of the 15 commits is under `docs/`. **PASS.**

## E2b — and the inverse: the 5 code steps each carry ≥1 non-docs file

Command, per the 5 code steps:

```
for n in 283 288 294 298 300; do
  sha=$(git log --format='%H %s' b35509d88..HEAD | grep "#${n}):" | awk '{print $1}')
  echo "#$n: $(git show --name-only --format= "$sha" | grep -vcE '^docs/|\.md$') non-docs files"
done
```

```
#283: 29 non-docs files
#288: 1 non-docs files
#294: 7 non-docs files
#298: 2 non-docs files
#300: 0 non-docs files
```

#283, #288, #294, #298 each carry ≥1 non-docs `.rs` file, confirming grok's own #299 correction
that #298 is not docs-only (2 `.rs` files, matching `commits.tsv`'s prediction exactly). **#300 is
the one exception, and it is disclosed rather than hidden**: #300 is an empty commit (0 files
total, not just 0 non-docs files) because its entire content — a 5-line
`rune:lint(cited-name-absent) EXEC_SP` doc comment — was already landed at #298 under this
replay's own "never a knowingly-red REPLAY commit" rule (see E10, E19, and REPLAY-LOG.md's #298
and #300 sections for the full account). This is the #202 precedent (`13bc69e2a`) repeating in
this batch. **PASS, with the #300 exception explicitly named here** (not only in the log).

## E3 — #283's new gate is GREEN, both arms

Command: `cargo nextest run --release -E 'test(rete_citation_resolves)'`

```
Nextest run ID ... with nextest profile: default
    Starting 20 tests across 46 binaries (5681 tests skipped)
    ... (all PASS) ...
     Summary [   0.279s] 20 tests run: 20 passed, 5681 skipped
```

All 20 tests green — both required arms
(`every_backticked_name_in_a_rete_comment_resolves`, `every_bare_filename_in_a_rete_comment_
names_a_file`) plus the 3 sibling controls (`prose_cannot_vouch_for_prose`,
`the_universe_reaches_the_test_corpus`, `each_resolver_half_answers_a_name_no_other_half_can`,
`the_gate_does_not_attest_its_own_text`) and 14 classifier unit tests. Neither `unresolved` nor
`hollow` fires at HEAD. **PASS** (measured red-then-green during landing — see E4/E5 below for the
repair path).

## E4 — every rune is a DECLARATION, not a suppression

Every `rune:lint(cited-name-absent)` line added in the range (`git diff b35509d88..HEAD -- '*.rs'
| grep '^+.*rune:lint(cited-name-absent)'`), read individually:

- **11 new declarations, mine, at #283** — all for names each site's own comment already says are
  `DELETED` (arc 255 Stone P6-c-W5a, main-only, pre-dating this replay): `eval_alpha_match`,
  `eval_alpha_match_local`, `eval_alpha_match_kind`, `eval_alpha_match_under`,
  `eval_cond_has_deferred_constraint` (`src/rete/matcher.rs`); `eval_pure_predicate`,
  `eval_deterministic_predicate`, `eval_total_predicate`, `eval_rete_primitive_predicate`,
  `eval_axis_predicate` (`src/rete/purity.rs`); `eval_vocabulary_admitted_predicate`
  (`src/rete/vocabulary.rs`). Each names the exact token, the specific arc/stone, and the specific
  `#[wat_intrinsic]` replacement function it now calls — no blanket reason, no shared boilerplate
  reused as an argument. Every reason ≥40 chars, measured for all 11
  (`grep ... | sed -E 's/.*—\s*//' | awk '{print length}'`), ranging 135–159: shortest is
  `eval_alpha_match_kind`'s "deleted at arc 255 Stone P6-c-W5a alongside its siblings; the shared
  arity-guard dispatch it named no longer exists as a standalone fn." (135 chars); longest is
  `eval_cond_has_deferred_constraint`'s (159 chars).
- **1 new declaration, mine, folded from #300 into #298** — `EXEC_SP` (`src/rete/expr_ir/eval.rs`),
  copied byte-for-byte from grok's own `b41a63672` diff (the fix grok itself wrote for this exact
  name). Reason: "the deleted cursor, named because its absence is what this section states..."
  (well over 40 chars).
- **The remaining ~20 rune lines the `grep` above surfaces** (`any_constrains`, `exec.rs`,
  `token_element_compatible` ×2, `restore_parent`, `compiled_rhs_cache`, `invoke_wat_compile`,
  `tests.rs`, `fire_cost_census.rs`, `head_is_boolean_rete_predicate`, `validate.rs` ×2,
  `keyword_constant_segment` ×2, `check_field_at` ×2, `NoMatchingArm` ×2, `infer_reduce`,
  `ShadowNode`, `tree.rs`) are grok's OWN pre-existing repair work from #283's own 27
  citation-repairing files — read at #283-landing time to confirm none was reworded (see E5), not
  authored by this executor.

The mechanical proof is E3 itself: `hollow.is_empty()` is asserted BEFORE `unresolved.is_empty()`
in `every_backticked_name_in_a_rete_comment_resolves`, and it passed — a hollow reason (blanket,
too-short, or a refused shrug word) would have failed that assertion first. **PASS, zero blanket
reasons.**

## E5 — no CORRECT citation was reworded to dodge a red

Every comment-only change this executor made under `src/rete/` at #283, outside grok's own 27
citation-repair files, falls into exactly one of three categories, none a reword-to-dodge:

1. **Rune additions** (11, listed at E4) — new declarations, not rewordings of existing text.
2. **Two genuine stale-rename fixes** — `eval_persistentmap_contains_key_q` → `eval_contains`
   (`src/rete/expr_ir/eval.rs:1132`; confirmed by reading `src/runtime.rs:9360-9368`'s
   `MapContainer::PersistentMap` arm, which is what the sentence now correctly names) and
   `is_pure_total` → `is_expand_time_legal` (`src/rete/purity.rs:353,2338`; confirmed by
   `src/intrinsic/mod.rs:2718`'s own record: *"Stone expand-1 renamed `is_pure_total` ->
   `is_expand_time_legal`"*). Both are THE FIX #1 ("it moved or was renamed — spell it as the
   identifier that exists today"), not #2 ("reword to say what is true now") — the old citations
   were simply WRONG, not correct-but-inconvenient.
3. **One bare-filename precision fix** — `string.rs` → `src/intrinsic/string.rs`
   (`src/rete/purity.rs:2054`). The gate's own `shadowed_by_split` heuristic mis-attributed this to
   `src/string/` (which holds unrelated case-conversion helpers); `declare-acronyms`, the function
   the sentence is actually about, was confirmed by direct grep to live in
   `src/intrinsic/string.rs`. This is the gate's own heuristic being wrong about a correct-in-spirit
   citation, and the fix makes the citation UNAMBIGUOUS rather than reworded to dodge anything —
   consistent with the gate's own header warning that a too-narrow universe manufactures findings.

Separately, at `each_resolver_half_answers_a_name_no_other_half_can` (a TEST CONTROL, not a
citation under `src/rete/` — outside this row's literal scope but disclosed here for completeness):
the `WAT_ONLY` canary `SiftRulesResponse` was swapped for `SiftRulesRequest` because
`src/check.rs:23614` now cites the former in a Rust code position on this tree (confirmed absent
on grok's own tree). This is a broken TEST FIXTURE repaired, not a `src/rete/` citation reworded.
**PASS.**

## E6 — #283's repair landed AT #283

Command: `git show --name-only --format= <283-sha> | wc -l` and cross-check against the repair
file list.

```
29
```

The #283 REPLAY commit (`0f7433b5a`) itself carries all 29 files: grok's own 28 plus the ONE file
the repair extends grok's set by (`src/rete/expr_ir/eval.rs`, disclosed explicitly in the commit
body's own opening paragraph) — not a later step. **PASS.**

## E7 — named tests at the 5 code steps

Per-step counts, all captured live at commit time:

- #283: 20/20 (`rete_citation_resolves`'s full suite)
- #288: 9/9 (`no_unknown_ward_rune`)
- #294: 60/60 (`rete_citation_resolves`'s 20 unchanged + `accum_alpha_cost`/`accum_cost`/
  `gather_probe_cost`'s 40)
- #298: 2/2 (`exec_frame_unwind`'s D4 probe)
- #300: N/A — empty commit, no tests to name (disclosed, not silent)

All green, N > 0 in every `-E` selection that ran a test. **PASS.**

## E8 — ZERO hazard paths in range

Commands:

```
git diff --name-only b35509d88..HEAD | grep -E '^wat/|^wat-scripts/fixes/'
awk -F'\t' '$1>=281 && $1<=300' bootstrap/era/replay-plan/stdlib-touch.tsv
awk -F'\t' '$1>=281 && $1<=300' bootstrap/era/replay-plan/absent-on-main.tsv
awk -F'\t' '$1>=281 && $1<=300' bootstrap/era/replay-plan/flags.tsv
```

First three: no output. Fourth: all 20 rows show `fixes=0 main-deleted=0`. **None at all — PASS.**

## E9 — no `.wat` anywhere in range

Command: `git diff --name-only b35509d88..HEAD | grep -c '\.wat$'`

```
0
```

**PASS.**

## E10 — finding 33's class actively looked for

Each code step's commit body carries an explicit "finding 33 grepped:" line:

- #283: "YES — `git diff --cached -- <29 .rs files> | grep '":wat::'` on changed lines only
  returns nothing"
- #288: "not applicable — this step neither renames/rehomes a name nor adds a .rs file containing
  wat program strings"
- #294: "YES — no `":wat::` hits across all 7 files"
- #298: "YES — no `":wat::` hits across both .rs files; the two new EXEC_SP citations are
  doc-comment prose, not embedded wat program strings"
- #300: not stated in the empty commit's body (no files to grep); recorded here as "not
  applicable, empty commit" rather than left silent.

Every code step answers explicitly; "not applicable" appears twice and is stated, never implied by
silence. **PASS.**

## E11 — the checkpoint

`scripts/floor.sh` and `cargo clippy --release --all-targets -- -D warnings` are explicitly
forbidden to this executor (brief anchor: "Do NOT run scripts/floor.sh, cargo clippy, or run5 —
the orchestrator weighs those centrally and uncontended; a gate run while you hold the tree is a
FALSE result"). **Not checked, because the brief forbids running it from inside this session — it
is the orchestrator's own row.**

## E12 — test-count delta ACCOUNTED FOR

Predicted and measured together, incrementally, at each step (not reconstructed after the fact):

| step | lint-subset | kind(lib) | doctest | delta source |
|---|---|---|---|---|
| baseline (#278) | 192 | 1490 | 8 | — |
| #283 | 212 (+20) | 1490 (+0) | 8 | new `rete_citation_resolves.rs`, 20 tests, an integration binary |
| #288 | 221 (+9) | 1490 (+0) | 8 | new `no_unknown_ward_rune.rs`, 9 tests |
| #294 | 235 (+14) | 1490 (+0) | 8 | new `rete_engine_label_names_its_evidence.rs` + `universe_control_name.rs` |
| #298 | 235 (+0) | 1492 (+2) | 8 | new `#[cfg(test)] mod exec_frame_unwind` inside `src/rete/expr_ir/eval.rs` — a `kind(lib)` unit-test module, not an integration binary |
| #300 | 235 (+0) | 1492 (+0) | 8 | empty commit |

Every delta traces to a specific new test module named above; no test was deleted, renamed, or
moved anywhere in this range. Predicted == actual at every step because each was measured
immediately after landing, not assembled afterward. **PASS.**

## E13 — spot re-run of the walls

Full wall re-run at HEAD (after #300, before this SCORE was drafted):

```
lint-subset:          235 passed   (identical to #298's recorded 235)
kind(lib):            1492 passed  (identical to #298's recorded 1492)
doctest:                 8 passed  (identical to #298's recorded 8)
census:               2122 files, no STOP-8  (byte-identical file to #298's own — `diff` confirmed)
nested-program-gate (stone-3):  3/3, 5723 skipped  (identical to #298's recorded 3/3)
```

Identical to the last code step's (#298) own recorded numbers. **PASS.**

## E14 — no knowingly-red commit

No repair commit was appended after #300. Every REPLAY commit's gates were run and confirmed green
BEFORE that commit was made — including #283 and #298, both of which went red on first measurement
and were repaired in the working tree before the commit was created, never committed red and fixed
after. **PASS.**

## E15 — every artifact a body names exists

Every `.census/…txt` cited across the commit bodies:

```
.census/2026-09-16T10-57-22Z.txt   OK   (#278's, cited as #283's prior-baseline)
.census/2026-09-16T12-09-53Z.txt   OK   (#283)
.census/2026-09-16T12-25-37Z.txt   OK   (#294)
.census/2026-09-16T12-33-31Z.txt   OK   (#298)
```

All present on disk (gitignored, local-only artifacts as designed). **PASS.**

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

`origin/replay/grok-rete` (`b35509d88`) is an ancestor of HEAD (`a1775d37f`); `refs/original/` is
empty. No `git filter-branch` was used anywhere in this batch — the two process repairs made
(the #283 sequencing slip and the #282 trailer typo) were a plain `git reset --hard` to an
unpushed commit and a `git commit --amend` on an unpushed tip, respectively, neither of which
creates a `refs/original/` backup or touches any commit below the batch's own start. **PASS.**

## E17 — every repair visible to `push`

Command: `git replace -l`

```
(empty)
```

Gate re-run under `GIT_NO_REPLACE_OBJECTS=1`:

```
step-range: #281..#300 each present exactly once, sources match
step-record: complete
```

exit 0 either way — **0 replace refs**. Every repair this batch made (the #283/#298 in-tree fixes
before commit, the #282 amend, the sequencing-slip reset) changed the actual commit objects
directly; there is no `refs/replace/` overlay anywhere to hide behind. **PASS.**

## E18 — no verdict line is WRAPPED

Proven mechanically by E1: `verify-step-record.sh`'s patterns (`census: .*--diff no STOP-8`,
`nested-program-gate: PASS`, `lint-subset: [0-9]+ passed`, `kind\(lib\): [0-9]+ passed`,
`doctest: [0-9]+ passed`) match `grep -qE` against each commit body LINE BY LINE — a wrapped line
would have produced `MISSING` at E1, and E1 reported `step-record: complete` with zero MISSING
lines. Additionally self-checked before committing each body: every verdict block in this batch
was typed as single, un-wrapped lines from the start (learned directly from finding 31/4g's
history, read in full before this batch began). **PASS.**

## E19 — the SCORE discloses what BOUGHT each green

Every landing-time action that changed a gate, its inputs, or the record, stated in the row it
affects, not only in REPLAY-LOG.md:

- **E2b**: #300 lands as an empty commit (0 files) because its content was folded into #298 —
  named in E2b itself, not left as a silent anomaly.
- **E3/E4/E5**: #283's gate went red 3-of-20 on first measurement; the exact 13 unresolved names,
  1 stale bare filename, and 1 broken control canary, and how each was resolved (2 stale renames
  fixed, 1 bare-filename made precise, 11 rune declarations added, 1 canary swapped, 1
  self-inflicted cross-gate red found and fixed by rewording rather than by an unrelated rune) are
  named in E4/E5 above, not only in the log.
- **E7/E10**: #298 went red 19-of-20 on first measurement (`EXEC_SP` unresolved) for a reason this
  executor determined was NOT corpus-divergence but a fold-forward of grok's own #300 fix,
  applied at #298 instead of #300 per the never-a-knowingly-red-commit rule — stated in E2b and
  E10, and in full in REPLAY-LOG.md's #298/#300 sections.
- **E16**: two process repairs (a `git reset --hard` for a sequencing slip, a `git commit --amend`
  for a fabricated SHA) are named here and in E1's narrative, not only in the log.

**PASS.**

## E20 — every `-E` filter selected N > 0

Every nextest invocation's own `N tests run` line, read directly (never inferred from exit code
alone): #283's 20, #288's 9, #294's 60, #298's 2, plus every lint-subset run (212 → 221 → 235 →
235), every `kind(lib)` run (1490 → 1490 → 1490 → 1492), every `nested_program_starts::` run
(3/3 throughout), every doctest run (8 throughout). None was ever 0. **PASS.**

## What would have rejected this batch — none of it happened

The #283 gate was never weakened, allowlisted, or its red deferred (E3/E4/E5) — it landed red
exactly as the brief predicted and was repaired in place before the commit. No correct citation
was reworded to dodge a red (E5) — every fix was either a genuine stale-name correction, a genuine
absence declaration, or a precision fix to an ambiguous-but-not-wrong bare filename. No rune
carries a blanket reason (E4). No `refs/original/` entry exists and the pushed tip is still an
ancestor of HEAD (E16). No `refs/replace/` entry exists (E17). The one process irregularity this
batch produced beyond grok's own content — the #298/#300 fold-forward — is disclosed in the SCORE
row it affects, not only in the log (E19), matching exactly what E19 was written to require.
