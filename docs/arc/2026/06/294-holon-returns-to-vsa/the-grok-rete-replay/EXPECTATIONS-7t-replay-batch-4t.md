# EXPECTATIONS 7t — replay batch 4t, grok-rete #521 → #540 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended.

## ⚠ Which rows were PRE-FLIGHTED, and which are predictions

**Executed against HEAD `5f476ab4d` and ALL PASS:** E8 (origin an ancestor; `refs/original/` empty), E9
(0 replace refs), tree clean, 520 REPLAY steps, E2 (18 docs-only), E2b (#537 and #540 the only code
steps), E4 (the range is 39 `.md` + 9 `.wat` + 1 `.sh`; zero `src/`, zero hazard rows, zero new gates,
zero `wat-scripts/fixes/`, zero `wat/`), the **zero** `#[test]`/`#[ignore]` delta, and **#537's nine new
`.wat` all failing our checker on the retired positional `assertion-failed!`, with 9 occurrences of the
retired `:wat::core::i64::+`**.

**Predictions, not verified rows:** E3, E5–E7, E10–E16.

⛔ **What I could NOT measure and did not pretend to.** Whether `scripts/replay/convert.sh` cures all
nine cleanly — grok's era predates two migrations at once here, and the chain has only been exercised on
one or two files at a time in earlier batches. **E3 scores running it and reporting what it did,
including a refusal.**

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh 5f476ab4d HEAD 521 540` | exit 0; `sources match` |
| E1b | ⛔ **every subject is GROK'S, verbatim** (finding 39) | per step vs `git log -1 --format=%s <C>` | 20 of 20, kind included |
| E1c | ⛔ **trailers PIPED, not typed** | per step vs `commits.tsv`, two-sided | 20 of 20 — six were fabricated across the last three batches |
| E2 | docs-only steps are docs-only | the **18** | only `docs/`/`.md` |
| E2b | **and the inverse** | #537, #540 | each carries ≥1 non-docs file |
| E3 | ⛔ **#537's NINE were CONVERTED BY THE CHAIN, not by hand** | `git show` #537; the SCORE | every one of the nine went through `scripts/replay/convert.sh 628e6371d`; the SCORE names each file and what changed; **no hand-edited `.wat`** |
| E4 | ZERO `src/`, zero hazard paths, zero new gates | `git diff --name-only` over the range | none |
| E5 | ⛔ **both `wat-scripts/` gates are GREEN and their verdicts READ** | `-E 'test(every_wat_scripts_file_loads_on_the_current_runtime)'` and `-E 'test(every_rete_name_in_wat_scripts_code_resolves)'` | green, N > 0 each, at #537 and at the tip; verdicts quoted |
| E6 | **finding 33 swept on #540's `.sh`** | the SCORE | explicit; "not applicable" is an answer, silence is not |
| E7 | **grok's measurements were NOT edited** | `git diff` the landed docs vs grok's blobs | the vigilia prose lands as written; divergences noted in the SCORE only |
| E8 | **NO PUBLISHED HISTORY REWRITTEN** | `merge-base --is-ancestor`; `for-each-ref refs/original/` | ancestor YES; empty |
| E9 | repairs visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | 0 refs; exit 0 either way |
| E10 | the checkpoint | `scripts/floor.sh` + clippy `-D warnings --all-targets` | green; clippy 0 — **the orchestrator's row** |
| E11 | **the SCORE discloses what BOUGHT each green** | read the SCORE | every converted file, every gate verdict |
| E12 | **test-count delta is ZERO** | predict from the diff BEFORE the floor | **5872 run, 22 skipped — unchanged.** A different number is a result to report |
| E13 | **every verdict line is TRUE, and on ONE line** | grep each | no wrapped line |
| E14 | **every deviation from the brief is REPORTED** | the SCORE | honest disagreement passes; agreement does not earn it |
| E15 | **NO COUNTERPART ACTIVITY**; no unfiltered run | `.floor/`, `git status`, `.pulsare/` mtimes | none; frozen root untouched |
| E16 | no knowingly-red commit; messages survived their heredoc | per-step landing; `git log --format=%B` | none red; no eaten backtick spans |

## ⛔ E3 and E5 are the rows that catch a lazy #537

Nine broken scratch files can be hand-patched in a few minutes and the gates will go green — and the
recorded chain, which exists so a migration is reproducible, will have been bypassed. **The SCORE shows
`convert.sh`'s own output per file, or E3 fails.**

**Runtime prediction:** ~50–80 min. Eighteen docs steps, one conversion step, one `.sh`.

**What would make me reject the batch:** a hand-edited `.wat`; a gate verdict assumed rather than run; an
edited grok measurement; a typed trailer or paraphrased subject; a wrapped verdict line; any
`refs/original/` entry; any `pulsare_yield`.
