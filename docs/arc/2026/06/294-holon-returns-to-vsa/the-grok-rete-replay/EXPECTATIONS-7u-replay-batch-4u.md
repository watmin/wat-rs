# EXPECTATIONS 7u — replay batch 4u, grok-rete #541 → #560 (written BEFORE the strike)

## ⚠ Which rows were PRE-FLIGHTED, and which are predictions

**Executed and ALL PASS:** origin an ancestor, `refs/original/` empty, 0 replace refs, tree clean, 540
REPLAY steps, **all 20 steps docs-only**, the range is **52 `.md` and nothing else**, **zero**
`#[test]`/`#[ignore]` delta, `CURRENT-STATE-annihilate-interpretation.md` diverges from grok's pre-image
by **12 lines** and is touched by **nine** steps, and #558's `wat/rete.wat:547+` is out of range on
**both** trees while living in a `.md` that `no_stale_path_in_doc` does not scan.

**Predictions, not verified rows:** the conflict resolutions, and everything downstream of them.

⛔ **What I could NOT measure.** How many of the nine touches actually conflict, and whether any drops
content. **E3 scores resolving each one with both sides intact and verifying after each.**

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh <batch-start> HEAD 541 560` | exit 0; `sources match` |
| E1b | ⛔ **every subject is GROK'S, verbatim** | per step vs `git log -1 --format=%s <C>` | 20 of 20 |
| E1c | ⛔ **trailers PIPED, not typed** | per step vs `commits.tsv`, two-sided | 20 of 20 |
| E2 | **all 20 steps are docs-only** | `git diff --name-only` over the range | only `.md`; zero `.rs`/`.wat`/`.sh` |
| E3 | ⛔ **the diverged shared doc kept BOTH sides** | `git diff` the file; grep for markers | main's 12-line annotation still present at the tip; grok's new text landed at each of the nine steps; **zero conflict markers**; each resolution named in its own commit body |
| E4 | **grok's prose landed unedited** | hash each touched `.md` vs grok's blob | every file byte-identical to grok's, except where a conflict resolution is recorded and explained |
| E5 | **#558's citation was NOT "fixed"** | `git show` #558 | `wat/rete.wat:547+` lands as grok wrote it; the observation is in the SCORE |
| E6 | **the docs gates are GREEN and their verdicts READ** | `-E 'test(no_stale_path_in_doc)'`, the docs-wat gate | green, N > 0; verdicts quoted |
| E7 | **NO PUBLISHED HISTORY REWRITTEN** | `merge-base --is-ancestor`; `for-each-ref refs/original/` | ancestor YES; empty |
| E8 | repairs visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | 0 refs; exit 0 either way |
| E9 | the checkpoint | `scripts/floor.sh` + clippy `-D warnings --all-targets` | green; clippy 0 — **the orchestrator's row** |
| E10 | **the SCORE discloses what BOUGHT each green** | read the SCORE | every conflict resolution, both sides named |
| E11 | **test-count delta is ZERO** | predict from the diff BEFORE the floor | **5872 run, 22 skipped — unchanged**; structurally impossible to move |
| E12 | **every deviation from the brief is REPORTED** | the SCORE | honest disagreement passes; agreement does not earn it |
| E13 | **NO COUNTERPART ACTIVITY**; no unfiltered run | `.floor/`, `git status`, `.pulsare/` mtimes | none; frozen root untouched |
| E14 | no knowingly-red commit; messages survived their heredoc | per-step landing; `git log --format=%B` | none red; no eaten backtick spans |

## ⛔ E3 and E4 are the rows that catch a lazy 4u

Nine conflicts in one file resolve fastest by taking one side — and taking grok's would silently delete
main's annotation, while taking ours would silently drop the history being replayed. **Neither is
acceptable.** The SCORE names each resolution, and E4's hash check is what proves nothing else drifted.

**Runtime prediction:** ~40–70 min — twenty documentation steps, with conflicts concentrated in one file.

**What would make me reject the batch:** main's annotation gone; grok's text dropped; a conflict marker
left in a file; an edited grok citation or figure; a typed trailer or paraphrased subject; any
`refs/original/` entry; any `pulsare_yield`.
