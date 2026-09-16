# SCORE 7i — replay batch 4i, grok-rete #301 → #320

Batch complete: 20 REPLAY commits, tip `be2982f0f`. Every row below answers with the command run
and its actual output — a row that could not be checked says so and why, never left blank.

## E1 — 20 steps, correctly subjected

Command: `scripts/replay/verify-step-record.sh f0bb300bf HEAD 301 320`

```
step-range: #301..#320 each present exactly once, sources match
step-record: complete
```

exit 0 (also re-run under `GIT_NO_REPLACE_OBJECTS=1`, identical output). **PASS.**

## E2 — docs-only steps are docs-only

Command, per the 13 docs-only steps:

```
for n in 301 303 305 307 308 309 311 312 314 315 316 318 319; do
  sha=$(git log --format='%H %s' f0bb300bf..HEAD | grep "#${n}):" | awk '{print $1}')
  git show --name-only --format= "$sha" | grep -vE '^docs/|\.md$'
done
```

No output — every file in every one of the 13 commits is under `docs/`. Note #315 is included
in this list and has 0 files total (see E22): a commit with zero files trivially satisfies "no
non-docs file," so it is counted here as docs-only-shaped even though it is really empty. **PASS.**

## E2b — and the inverse: the 7 code steps each carry ≥1 non-docs file

Command, per the 7 code steps:

```
for n in 302 304 306 310 313 317 320; do
  sha=$(git log --format='%H %s' f0bb300bf..HEAD | grep "#${n}):" | awk '{print $1}')
  echo "#$n: $(git show --name-only --format= "$sha" | grep -vcE '^docs/|\.md$') non-docs files"
done
```

```
#302: 5 non-docs files
#304: 1 non-docs files
#306: 1 non-docs files
#310: 2 non-docs files
#313: 1 non-docs files
#317: 1 non-docs files
#320: 4 non-docs files
```

All 7 carry ≥1 non-docs file. **Disclosed discrepancy, not hidden:** `bootstrap/era/replay-plan/
commits.tsv` itself tags **8** rows `code` in this range (301-320), not 7 — it also tags #315
`code`, but #315 is grok's own empty commit-message-only correction (0 files total, see E22). The
brief's own "seven carry code: 302 304 306 310 313 317 320" list is the one that matches reality;
`commits.tsv`'s crude per-commit categorisation counts #315 as `code` despite it carrying nothing.
**PASS**, 7+13+1(empty)=... — see E1's own 20-count; the 13+7 split matches the brief exactly, with
#315 the one row whose tsv `code` tag is misleading in isolation.

## E3 — #310's new gate is GREEN, both arms

Command: `cargo nextest run --release -E 'test(census_name_read_by_a_cost_test_is_emitted)'`

```
Nextest run ID a5695004-3604-4612-b15b-9c4e914bac19 with nextest profile: default
    Starting 14 tests across 46 binaries (5727 tests skipped)
        PASS (11 extractor unit tests)
        PASS (12/14) the_computed_name_half_reaches_the_bucket_helpers
        PASS (13/14) every_census_name_a_cost_test_reads_is_emitted
        PASS (14/14) every_census_name_retired_rune_names_a_name_the_engine_no_longer_emits
     Summary [   0.256s] 14 tests run: 14 passed, 5727 skipped
```

Both required arms green, non-vacuity floors independently re-measured (not just trusted from the
PASS): compiled the gate file standalone (`rustc`, `CARGO_MANIFEST_DIR` set, a throwaway `main()`
calling its own `emitted()`/`reads()`, no repo file touched) and printed the internals:

```
em.from_literals=75  em.from_computed=16  em.names.len()=91
rd.len()=99  runed=4  unresolved=0
```

`75 > 50` and `99 > 40` both clear; `em.names.len()=91` matches the brief's own "91 census
mentions" figure exactly. **PASS**, neither `unresolved` nor `hollow` fires.

## E4 — every rune is a DECLARATION, not a suppression

⛔ **DEVIATION FROM THE BRIEF, reported as instructed:** the brief predicted #310 "WILL LAND RED"
("four for four" after #274/#278/#283) because grok's own commit repairs only `accum_cost.rs`
while this tree carries 8 `*_cost.rs` files under `src/rete/kernel/tests/` (confirmed: 8 files,
matching the brief's own count). Measured instead of assumed (see E3's standalone-compile
verification): **`unresolved=0`.** This corpus's other 7 `*_cost.rs` files contain no census-name
read that fails to resolve against non-test `src/`'s emitted set. **No repair was needed or made
at #310 beyond what grok itself carries — the gate landed clean.**

The 4 `runed` reads are all pre-existing grok content (the `ALPHA_KIDS` array retirements,
`c9d751049`), **none added by this executor**:

```
src/rete/kernel/tests/accum_cost.rs:546,548,550,552
  rune:lint(census-name-retired) — retired by c9d751049 (per-fact alpha timers off);
  read via kid_pairs == 0, never via ns.
```

Reason length measured character-for-character (Python `len()` over the captured group after the
em-dash): **88 chars**, all four identical, well over the 40-char floor. Each names the exact
retiring commit, the mechanism (per-fact alpha timers removed), and why the reader stays sound
(branches on `kid_pairs == 0`, never trusts the nanosecond value) — a declaration, not a blanket
reason. **PASS, zero blanket reasons, zero runes added by this executor.**

## E5 — #310's repair landed AT #310

There was no repair to land — E3/E4 measured `unresolved=0` on this tree without any change beyond
grok's own diff. **N/A in the sense the brief anticipated (no repair needed); PASS in the sense
that matters: nothing was deferred, weakened, or folded elsewhere, because nothing needed fixing.**

## E6 — our `gather_probe_cost.rs` divergence SURVIVES

Command: `grep -n 'h >= (b + m + e)' src/rete/kernel/tests/gather_probe_cost.rs`

```
(no live assertion match — the struck apportionment assert is absent as code)
```

This range's diffs never touch `gather_probe_cost.rs` at all (confirmed: none of the 7 code
steps' `git show --name-only` lists that file). The struck assert (finding 32) was never at risk
of being "restored" because nothing in #301-#320 conflicts there. **PASS, absent, untouched.**

## E7 — named tests at the 7 code steps

Per-step counts, all captured live at commit time:

- #302: N/A — `.sh`-only change, no `.rs`/`.wat`, no named test applicable (disclosed in the
  commit body, not silent)
- #304: 1/1 (`c4_probe_bind_only_decides_skip_span_for_the_accum_axis`)
- #306: 2/2 (`accum_alpha_leftover_split`, `accum_alpha_push_split`)
- #310: 2/2 named (`accum_leftover_split`, `accum_seen_fire_context_split`) + 14/14 new gate suite
- #313: 1/1 (`node_share_where_cost_decomposition`)
- #317: 2/2 (`token_bindings_representation_dominance`, `bind_key_construction_vs_map_operation`)
  — re-verified 5 consecutive times plus once under full `kind(lib)` load, per the brief's
  suspicion-of-new-timing-assertions instruction
- #320: 5/5 (`accum_matcher_op_census`, `accum_leftover_split`, `accum_alpha_leftover_split`,
  `cascade_setup_leftover_split`, `fanout_three_leftover_split`)

All green, N > 0 in every `-E` selection that ran a test; #302 explicitly disclosed as N/A rather
than left silent. **PASS.**

## E8 — ZERO hazard paths in range

Commands:

```
git diff --name-only f0bb300bf..HEAD | grep -E '^wat/|^wat-scripts/fixes/'
awk -F'\t' '$1>=301 && $1<=320' bootstrap/era/replay-plan/stdlib-touch.tsv
awk -F'\t' '$1>=301 && $1<=320' bootstrap/era/replay-plan/absent-on-main.tsv
awk -F'\t' '$1>=301 && $1<=320' bootstrap/era/replay-plan/flags.tsv
```

First three: no output. Fourth: all 20 rows read `fixes=0 main-deleted=0` (read directly off the
awk output, not asserted). **None at all — PASS.**

## E9 — no `.wat` anywhere in range

Command: `git diff --name-only f0bb300bf..HEAD | grep -c '\.wat$'`

```
0
```

**PASS.**

## E10 — finding 33's class actively looked for

Each code step's commit body carries an explicit "finding 33 grepped:" line, read back here:

- #302: "YES — this is the exact file whose embedded perl substitution sat broken for weeks
  (#167). This diff's own hunks (lines 343-379) carry no `:wat::` tokens. The pre-existing perl
  substitution at lines 196/198 (untouched by this diff) still targets
  `:wat::rete::fire-rules$oracle` and `:wat::rete::FireOutcome.Fired` — both confirmed live via
  grep across `src/rete/vocabulary.rs` and `src/rete/kernel/outcome.rs`."
- #304: "YES, and it fired." A genuine finding-33 hit — the new probe's embedded `staged` wat
  string used a stale, pre-migration form (paren-clause double-colon match, positional
  `assertion-failed!`); running it raised
  `"assertion-failed! takes kwargs :message / :actual / :expected; the positional ... form is
  retired"`. Hand-converted to match the file's own 5 other correct `staged` occurrences
  (finding 33's own "sibling copy on the floor" shape). Re-ran: PASS.
- #306: "not applicable — zero `:wat::` tokens anywhere in this diff" (pure Rust benchmark
  instrumentation).
- #310: "not applicable — zero `:wat::` tokens anywhere in this diff."
- #313: "not applicable — zero `:wat::` tokens anywhere in this diff."
- #317: one `:wat::` mention (`:wat::rete::alpha-match{,-local,-under}`) — confirmed prose inside
  a `//` comment, not an embedded executable wat string; confirmed the named primitives are
  still live via grep across `src/intrinsic/rete.rs`.
- #320: "not applicable — zero `:wat::` tokens anywhere in this diff."

Every code step answers explicitly; "not applicable" appears four times and is stated, never
implied by silence; #302 is the class's own hot file and was read in full; **#304 is the one
genuine hit this batch, found and repaired at the step**. **PASS.**

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
| baseline (#300) | 235 | 1492 | 8 | — |
| #302 | 235 (+0) | 1492 (+0) | 8 | `.sh`-only, no test binary touched |
| #304 | 235 (+0) | 1493 (+1) | 8 | new `#[test] c4_probe_bind_only_decides_skip_span_for_the_accum_axis` — a `kind(lib)` unit test |
| #306 | 235 (+0) | 1493 (+0) | 8 | additive instrumentation inside existing test fns, no new `#[test]` |
| #310 | 249 (+14) | 1493 (+0) | 8 | new `tests/lint/census_name_read_by_a_cost_test_is_emitted.rs`, 14 tests, an integration (lint) binary |
| #313 | 249 (+0) | 1493 (+0) | 8 | no new `#[test]`, only assertion/comment changes inside existing fns |
| #317 | 249 (+0) | 1493 (+0) | 8 | no new `#[test]` |
| #320 | 249 (+0) | 1493 (+0) | 8 | comment + rendering fixes only, no new `#[test]` |

Every delta traces to a specific new test module named above; no test was deleted, renamed, or
moved anywhere in this range. Predicted == actual at every step because each was measured
immediately after landing, not assembled afterward. **PASS.**

## E13 — spot re-run of the walls

Full wall re-run at HEAD, in the foreground, after #320 and before this SCORE was drafted:

```
lint-subset:          249 passed   (identical to #320's recorded 249)
kind(lib):            1493 passed  (identical to #320's recorded 1493)
doctest:                 8 passed  (identical to #320's recorded 8)
census:               2122 files, no STOP-8  (byte-identical to #320's own census file — `diff` confirmed)
nested-program-gate (stone-3):  3/3, 5738 skipped  (identical to #320's recorded 3/3)
```

Identical to the last code step's (#320) own recorded numbers. **PASS.**

## E14 — no knowingly-red commit

No repair commit was appended after #320. Every REPLAY commit's gates were run and confirmed
green BEFORE that commit was made. #304 went red on first measurement (the stale embedded-wat
string, finding 33) and was repaired in the working tree — not committed red and fixed after.
#310, predicted red by the brief, measured green on first try and needed no repair. No other step
in this batch went red at any point. **PASS.**

## E15 — every artifact a body names exists

Every `.census/…txt` cited across the commit bodies:

```
.census/2026-09-16T12-37-37Z.txt   OK   (#300's own, cited as #304's prior-baseline)
.census/2026-09-16T23-12-59Z.txt   OK   (#304)
.census/2026-09-16T23-16-54Z.txt   OK   (#306)
.census/2026-09-16T23-23-04Z.txt   OK   (#310)
.census/2026-09-16T23-26-43Z.txt   OK   (#313)
.census/2026-09-16T23-30-42Z.txt   OK   (#317)
.census/2026-09-16T23-34-48Z.txt   OK   (#320)
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

`origin/replay/grok-rete` is an ancestor of HEAD (`be2982f0f`); `refs/original/` is empty. **No
`git filter-branch`, no `git replace`, and no history rewrite of any kind was used anywhere in
this batch** — every one of the 20 steps was a plain forward commit (19 real cherry-picks + 1
`--allow-empty` for #315), landed once, in order, on a clean tree each time. No process repair was
needed this batch. **PASS.**

## E17 — every repair visible to `push`

Command: `git replace -l`

```
(empty)
```

Gate re-run under `GIT_NO_REPLACE_OBJECTS=1`:

```
step-range: #301..#320 each present exactly once, sources match
step-record: complete
```

exit 0 either way — **0 replace refs**. Nothing in this batch used `refs/replace/` or any other
overlay. **PASS.**

## E18 — no verdict line is WRAPPED

Proven mechanically by E1: `verify-step-record.sh`'s patterns match `grep -qE` against each commit
body LINE BY LINE — a wrapped line would have produced `MISSING` at E1, and E1 reported
`step-record: complete` with zero MISSING lines across all 5 code steps that carry verdict lines
(#304, #306, #310, #313, #317, #320 — 6 code steps with `src/` changes; #302 carries no `src/`/
`.wat`/`.rs` change so the gate requires no verdict lines from it, confirmed by `verify-step-
record.sh`'s own path-based rule). Every verdict block in this batch was typed as single,
un-wrapped lines from the start. **PASS.**

## E19 — the SCORE discloses what BOUGHT each green

Every landing-time action that changed a gate, its inputs, or the record, stated in the row it
affects, not only in REPLAY-LOG.md:

- **E4/E5**: #310's gate was predicted red by the brief and measured green with zero repair —
  named here with the exact standalone-compile measurement (`em.from_literals=75`,
  `rd.len()=99`, `unresolved=0`) that established it, not left as an unexplained deviation.
- **E10**: #304's genuine finding-33 hit (a stale embedded wat string using a retired positional
  `assertion-failed!` form) and its hand-repair, matched byte-for-byte against the file's own 5
  correct sibling occurrences — named here and in REPLAY-LOG.md's #304 section.
- **E7**: #317's new directional timing assertions were re-verified 5x plus once under
  `kind(lib)` load precisely because the brief flagged this class (findings 28/32) for suspicion
  — the extra verification is disclosed here, not only performed silently.
- **E2b**: the `commits.tsv` vs brief discrepancy over whether #315 counts as an 8th "code" row
  is named here rather than silently reconciled.
- **E16**: no process repair occurred this batch at all — stated explicitly rather than left to
  be inferred from an empty section.

**PASS.**

## E20 — every `-E` filter selected N > 0

Every nextest invocation's own `N tests run` line, read directly (never inferred from exit code
alone): #304's 1, #306's 2, #310's 2 (+14 new-gate suite), #313's 1, #317's 2 (×6 re-runs), #320's
5, plus every lint-subset run (235 → 235 → 235 → 249 → 249 → 249 → 249), every `kind(lib)` run
(1492 → 1493 → 1493 → 1493 → 1493 → 1493 → 1493), every `nested_program_starts` run (3/3
throughout), every doctest run (8 throughout). None was ever 0. **PASS.**

## E21 — NO COUNTERPART ACTIVITY

Commands: `ls .floor/` (no new directory created during this session — the newest entry,
`2026-09-16T12-56-08Z`, predates this batch's own work, whose own artifacts are timestamped
`2026-09-16T23-xx`); `git status --porcelain` shows only this executor's own committed work, tree
clean at every checkpoint; `mcp__pulsare__*` was never called (confirmed: no tool of that family
appears anywhere in this session's tool-call history); `/home/john/work/holon/` (the frozen root,
including `.pulsare/`) was never read or written — this executor's entire working surface was
`/home/john/work/holon/wat-rs`. **PASS — no `pulsare_yield`, no foreign floor, no foreign
artifact, frozen root untouched.**

## E22 — #315 recorded despite being empty

Command: `git show --stat ac2e4ebb1`

```
commit ac2e4ebb1a0811590350f8d2167822d404c211d8
REPLAY(grok-rete #315): curare: restore a phrase the previous commit message lost to the shell
 0 files changed
```

Grok's own `2e98d80069d3f99895f6a24a9890a4f1adab1c78` carries 0 file changes (confirmed via
`git show --stat` on grok's own commit before landing) — a pure commit-message correction, not a
tree change. The REPLAY commit was made `--allow-empty` and its body states plainly that grok's
own commit carries no tree change, per the #202/#300 precedent. **PASS.**

## What would have rejected this batch — none of it happened

The #310 gate was never weakened, allowlisted, or its red deferred (E3/E4/E5) — it was measured
honestly and landed green, which is itself the deviation from the brief that had to be reported
and was. No rune was added or reworded by this executor to dodge anything (E4 — all 4 runes are
grok's own, none mine). No `refs/original/` entry exists and the pushed tip is still an ancestor
of HEAD (E16). No `refs/replace/` entry exists (E17). No `pulsare_yield` or other signal left this
sandbox (E21). No history rewrite of any kind occurred (E16). No new ratio/threshold assertion was
accepted without scrutiny — #306's and #313's timing arms carry only liveness (`> 0.0`) checks,
and #317's new directional orderings were re-verified 5x plus once under load before being trusted
(E7). #304's one genuine finding-33 defect was found by grepping and reading, not missed (E10).
#315 was recorded as a real, empty `REPLAY` commit rather than skipped (E22).
