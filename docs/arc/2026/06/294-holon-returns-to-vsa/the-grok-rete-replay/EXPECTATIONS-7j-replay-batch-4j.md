# EXPECTATIONS 7j — replay batch 4j, grok-rete #321 → #340 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended. Fixed now so the result cannot move
the goalposts.

## ⚠ Which rows were PRE-FLIGHTED, and which are predictions

A row a CORRECT tree cannot satisfy is as broken as one a defect can satisfy (finding 34 §2), so a row is
RUN before it is demanded. Stated honestly rather than claimed wholesale:

**Executed against HEAD `665b17b60` before this file was written — ALL PASS:** E16 (origin an ancestor;
`refs/original/` empty), E17 (0 replace refs), tree clean, 320 REPLAY steps, E2 (14 docs-only), E2b (6
code), E8 (0 `wat/` or `wat-scripts/fixes/` paths), E9b (13 `.wat`, none under `wat/`), **no new
`tests/lint/` gate in range**, and **0 M-status-absent paths** — every file a step modifies already
exists here.

**Cannot be run until the batch lands — predictions, not verified rows:** E3–E7, E10–E15, E18–E20.

⛔ **And per finding 37, a prediction in this document is a prediction.** At 4i I wrote that a gate "WILL
land red" and it landed green; the executor measured instead of complying and reported the contradiction,
which was the correct act. **Disproving a row's forecast is a RESULT.** What is scored is whether the
measurement was taken and reported, never whether it matched my guess.

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh 665b17b60 HEAD 321 340` | exit 0; `step-range: #321..#340 each present exactly once, sources match` |
| E2 | docs-only steps are docs-only | `git show --name-only` on all **14** (#321 #322 #323 #325 #326 #329 #330 #331 #333 #334 #335 #338 #339 #340) | only `docs/`/`.md` |
| E2b | **and the inverse** | the 6 code steps (#324 #327 #328 #332 #336 #337) | each carries ≥1 non-docs file |
| E3 | every produced `.wat` checks | `--check` each of the 13 | rc 0, **or a deliberate refusal with its reason stated** — see E5 |
| E4 | conversion recorded where needed | the REPLAY-LOG | each new `.wat` says whether `convert.sh` was needed and what changed; "no conversion needed" is an answer |
| E5 | ⚠ **#332's probes may deliberately FAIL** | its body | #332 is a FINDING (D7 is LIVE). If a probe demonstrates the drop, its non-zero rc is the point and the body must say so |
| E6 | ⚠ **#329's red floor is adjudicated, not inherited** | its body + the SCORE | grok's own subject says it pushed a red. The SCORE states whether the fold rule applied, or STOPs — **never a knowingly-red REPLAY commit** |
| E7 | named tests at the 6 code steps | `-E` per step | green, **N > 0 selected** |
| E8 | **ZERO hazard paths in range** | `git diff --name-only` vs `wat/`, `wat-scripts/fixes/`, `absent-on-main.tsv` | **none at all** |
| E9 | the 13 `.wat` are NEW fixtures, not a corpus rewrite | `git diff --name-status` | all `A` under `tests/rete/`, `wat-scripts/scratch-pad/`, `docs/arc/**`; **none under `wat/`**; no `wat-scripts/fixes/` edit |
| E10 | finding 33's class actively looked for | the SCORE / REPLAY-LOG | each code step records whether the `.rs`/`.sh` side was grepped — "not applicable" is an answer, silence is not |
| E11 | the checkpoint | `scripts/floor.sh` + `cargo clippy --release --all-targets -- -D warnings` | green; clippy 0 — **the orchestrator's row** |
| E12 | **test-count delta ACCOUNTED FOR** | predict from the diff BEFORE the floor runs | predicted == actual, exactly |
| E13 | spot re-run of the walls | lint-subset + `kind(lib)` + doctests + stone-3 at HEAD vs the last code step's verdict lines | identical numbers |
| E14 | no knowingly-red commit | no repair commit after #340 | none |
| E15 | every artifact a body names exists | each `.census/…txt` cited | all present on disk |
| E16 | **NO PUBLISHED HISTORY REWRITTEN** | `git merge-base --is-ancestor origin/replay/grok-rete HEAD`; `git for-each-ref refs/original/` | ancestor **YES**; `refs/original/` **EMPTY** |
| E17 | every repair visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | **0 replace refs**; exit 0 either way |
| E18 | **no verdict line is WRAPPED** | grep each pattern across the range | every required line on ONE line; `--diff` and `no STOP-8` contiguous |
| E19 | **the SCORE discloses what BOUGHT each green** | read the SCORE | every landing-time action that changed a gate, its inputs, or its ledger appears in the row that gate satisfies |
| E20 | every `-E` filter selected N > 0 | each run's own `N tests run` line | N > 0 everywhere |
| E21 | ⛔ **NO COUNTERPART ACTIVITY** | `ls .floor/` for runs the orchestrator did not start; `git status` for foreign artifacts; `.pulsare/` mtimes | no `pulsare_yield`, no foreign floor, no foreign artifact, frozen root untouched |
| E22 | **every deviation from this brief is REPORTED** | the SCORE | where the tree disagreed with a forecast here, the SCORE names it as a measured result — **this row passes on honest disagreement, not on agreement** |

## ⛔ E22 is new, and it exists because my last brief was wrong

BRIEF-7i predicted a red that did not happen. The executor's refusal to assume is what stopped a
manufactured "repair". **A brief is a hypothesis with measurements attached; the SCORE is where the tree
answers back.** An executor that reports "the brief said X, I measured not-X, here is the command and its
output" has done the job exactly.

## ⛔ Every `-E` filter must be confirmed to SELECT A NON-ZERO COUNT

A filterset matching nothing runs zero tests and **exits 0** — a mis-aimed probe is indistinguishable from
a working gate.

**Runtime prediction:** ~70–110 min. Fourteen docs cherry-picks are quick; the weight is #328 (12 files),
#324 (10), #336 (7), plus `--check`/conversion on 13 new `.wat`.

**Trap doors named in advance:**
- **#329 — grok's own subject says it pushed a red floor.** The judgment call of the batch.
- **#332 — a finding, not a fix.** Its probes may deliberately fail; #336 is the cure. Do not pull #336
  backward into #332.
- **13 new `.wat` return** after two batches with none — `--check` and `convert.sh` work reappears. Not a
  corpus rewrite; R21's codemod path is not triggered unless a conversion proves structural across files.
- **#333 restores a table grok's own earlier edit deleted** — it will look redundant and is not.

**What would make me reject the batch:** a knowingly-red REPLAY commit at #329; #336's fix folded backward
into #332; a `.wat` left failing `--check` without a stated, checkable reason; any `refs/original/` entry
or the pushed tip no longer an ancestor; any `refs/replace/` entry; a fabricated trailer SHA; any
`pulsare_yield`; or a SCORE that silently agrees with a forecast it never measured.
