# EXPECTATIONS 7h — replay batch 4h, grok-rete #281 → #300 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended. Fixed now so the result cannot move
the goalposts.

⚠ **Which rows were PRE-FLIGHTED, and which could not be.** A row a CORRECT tree cannot satisfy is as
broken as one a defect can satisfy (finding 34 §2 — EXPECTATIONS-7f's E5 demanded a `grep -c` of 0 where a
correct tree returns 1, because the struck assertion's own comment quotes it). The cure is to RUN a row
before demanding it. So, stated honestly rather than claimed wholesale:

**Run against HEAD `351b24fbc` before this document was committed, all PASS** — E16 (origin is an ancestor;
`refs/original/` empty), E17 (0 replace refs), E9 (0 `.wat` across grok's #281–#300), E8 (0 `wat/` or
`wat-scripts/fixes/` paths across the range), E2 (15 docs-only) and E2b (5 code steps, #298 carrying
exactly 2 non-docs files — independently confirming grok's own #299 correction).

**Cannot be run until #283 lands, and are therefore predictions, not verified rows** — E3 (the gate does
not exist yet), E4 (no runes exist yet), E5 (nothing to compare yet), E6 (the step is not replayed yet),
E7 (those tests are not present yet). E11–E13 depend on a floor the executor is forbidden to run.

**An earlier draft of this file claimed every row had been run. It had not.** That claim was written
before any row was executed — the same defect finding 34 records, committed while recording its cure.
Corrected at source before release rather than after the result.

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh <start> HEAD 281 300` | exit 0; `step-range: #281..#300 each present exactly once, sources match` |
| E2 | docs-only steps are docs-only | `git show --name-only` on all **15** (#281 #282 #284 #285 #286 #287 #289 #290 #291 #292 #293 #295 #296 #297 #299) | only `docs/`/`.md` |
| E2b | **and the inverse** | the 5 code steps (#283 #288 #294 #298 #300) | each carries ≥1 non-docs file — **#298 especially**, per grok's own #299 correction |
| E3 | **#283's new gate is GREEN, both arms** | `-E 'test(rete_citation_resolves)'` | all tests run, PASS — neither `unresolved` nor `hollow` fires |
| E4 | **every rune is a DECLARATION, not a suppression** | each `rune:lint(cited-name-absent)` line added in range | names an exact token AND a reason ≥ 40 chars naming a mechanism; **zero blanket reasons** |
| E5 | ⛔ **no CORRECT citation was reworded to dodge a red** | comment-only changes under `src/rete/` outside grok's own 27 files | every one is a rune addition or grok's own edit — no silent rewording |
| E6 | **#283's repair landed AT #283** | `git show --name-only` on the #283 REPLAY commit | it carries the repaired files itself, not a later step |
| E7 | named tests at the 5 code steps | `-E` per step (#283 #288 #294 #298 #300) | green, **N > 0 selected** |
| E8 | **ZERO hazard paths in range** | `git diff --name-only` vs `wat/`, `wat-scripts/fixes/`, `absent-on-main.tsv` | **none at all** — this range has no hazard row of any kind |
| E9 | **no `.wat` anywhere in range** | `git diff --name-only <start>..HEAD \| grep -c '\.wat$'` | **0** — no `--check` and no codemod work is expected here |
| E10 | finding 33's class actively looked for | the SCORE / REPLAY-LOG | each code step records whether the `.rs`/`.sh` side was grepped — "not applicable" is an answer, silence is not |
| E11 | the checkpoint | `scripts/floor.sh` + `cargo clippy --release --all-targets -- -D warnings` | green; clippy 0 — **the orchestrator's row, not the executor's** |
| E12 | **test-count delta ACCOUNTED FOR** | predict from the diff BEFORE the floor runs | predicted == actual, exactly |
| E13 | spot re-run of the walls | lint-subset + `kind(lib)` + doctests + stone-3 at HEAD vs the last code step's verdict lines | identical numbers |
| E14 | no knowingly-red commit | no repair commit after #300 | none |
| E15 | every artifact a body names exists | each `.census/…txt` cited | all present on disk |
| E16 | ⛔ **NO PUBLISHED HISTORY WAS REWRITTEN** | `git merge-base --is-ancestor origin/replay/grok-rete HEAD`; `git for-each-ref refs/original/` | ancestor **YES**; `refs/original/` **EMPTY** |
| E17 | every repair visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | **0 replace refs**; gate exit 0 either way |
| E18 | **no verdict line is WRAPPED** | grep each pattern across the range | every required line matches on ONE line |
| E19 | ⛔ **the SCORE discloses what BOUGHT each green** | read the SCORE | every landing-time action that changed a gate, its inputs, or its ledger appears in the SCORE row that gate satisfies |
| E20 | every `-E` filter selected N > 0 | each run's own `N tests run` line | N > 0 everywhere |

## ⛔ E16 is new, and a tree-hash comparison does NOT satisfy it

Batch 4g repaired four wrapped verdict lines with `git filter-branch --msg-filter`, a **whole-history**
rewriter, on a 280-commit branch. It was safe — verified — but the entire margin was one rev-range
argument. A tree hash speaks to CONTENT and says nothing about the commit graph, so it cannot establish
that published commits survived. **The ancestor check and the empty `refs/original/` are the proof.**
Prefer the detach / re-commit / rebuild-descendants pattern (batch 4f's), which cannot reach below its own
start point.

## ⛔ E19 is new, and it exists because 4g's SCORE read green without saying what it cost

4g's E8 reported #278's gate green; the green was produced by **63 rune declarations added at landing**.
#270's 8 ledger deletions and #278's `attested()` exclusion appeared only in the REPLAY-LOG. All three
were disclosed honestly — in the wrong document. The SCORE is what gets scored.

## ⛔ Every `-E` filter must be confirmed to SELECT A NON-ZERO COUNT

A nextest filterset matching nothing runs zero tests and **exits 0** — a mis-aimed probe is
indistinguishable from a working gate.

**Runtime prediction:** ~90–140 min. Fifteen docs cherry-picks are quick; the weight is entirely #283
(28 files, a 913-line gate, plus an unresolved-citation repair whose size is unknown and expected to
exceed #274's ten), then #294 (7 `.rs`).

**Trap doors named in advance:**
- **#283 — the third new grok lint gate; it WILL land red.** Repair at the step (#184 precedent). Never
  reword a correct citation to dodge it: the gate's own header warns a too-narrow universe manufactures
  findings, and six rete-comment names are legitimately attested only outside `src/`.
- **#298 is NOT docs-only** despite its `curare:` subject — grok's own #299 says so, and our census agrees.
- **#294 modifies a file #283 creates.** Missing ⇒ #283 is at fault; STOP rather than create it.
- **The `census:` verdict line** must keep `--diff` and `no STOP-8` contiguous — put any `(vs #N's …)`
  detail AFTER `no STOP-8`. This is what cost 4g a history rewrite.

**What would make me reject the batch:** the #283 gate weakened, allowlisted, or its red deferred; a
correct citation reworded to dodge it; a rune with a blanket reason; any `refs/original/` entry or the
pushed tip no longer an ancestor; any `refs/replace/` entry; or a SCORE green whose landing-time cost is
disclosed only in the log.
