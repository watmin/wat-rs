# EXPECTATIONS 7z — replay batch 4z, grok-rete #641 → #651 — the final batch (written BEFORE the strike)

## ⚠ Which rows were PRE-FLIGHTED, and which are predictions

**Executed and ALL PASS:** origin an ancestor, `refs/original/` empty, 0 replace refs, tree clean, 640
REPLAY steps, 7 docs-only, four code steps, the range is 19 `.md` + 7 `.rs` + 6 `.wat` + 4 `.wat.bad`,
zero hazard rows, zero `wat/` paths, zero new gates, **zero `wat-scripts/fixes/` edits** (so the #438
class cannot recur), the test delta is **+10 with no macro expansion** (checked for
`macro_rules!`/`shards!`/`paste!`), the divergence table, and the two `wat-scripts/` `.wat`:
**`probe-acc-count-unused-bind.wat` `--check` 101 and declaring 2 `defrule` + 2 `defquery`**;
**`census-fence-binders.wat` `--check` 1 and declaring none.**

**Predictions, not verified rows:** the conversions, the compositions, every gate verdict, the floor.

⛔ **What I could NOT measure.** Whether `probe-acc-count-unused-bind.wat`, once its syntax is cured,
**compiles** under #638's zero-exemption gate. **E4 scores measuring it and reporting a failure as a
finding — never a rune, never a deletion.**

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 11 steps, correctly subjected | `verify-step-record.sh <batch-start> HEAD 641 651` | exit 0; `sources match` |
| E1b | ⛔ **every subject is GROK'S, verbatim** | per step vs `git log -1 --format=%s <C>` | 11 of 11, kind included |
| E1c | ⛔ **trailers PIPED, not typed** | per step vs `commits.tsv`, two-sided | 11 of 11 |
| E2 | docs-only steps are docs-only | the **7** (#641 #642 #643 #645 #648 #649 #650) | only `docs/`/`.md` |
| E2b | **and the inverse** | #644 #646 #647 #651 | each carries ≥1 non-docs file |
| E3 | ⛔ **both `wat-scripts/` `.wat` were CONVERTED BY THE CHAIN** | `git show` #646/#651; the SCORE | via `scripts/replay/convert.sh`; **no hand-edited `.wat`**; loader gates green with N > 0 and quoted |
| E4 | ⛔⛔ **the compile gate's verdict on #646's probe is REAL** | `-E 'test(rete_compile_gate)'` | green, N > 0 — **or** a red reported as a FINDING with the failing axis. **Zero runes; no rule deleted to buy green** |
| E5 | ⛔ **the finding/withdrawal pair landed AS WRITTEN** | `git show` #646, #647, #648 | #646's finding lands unsoftened; #648's withdrawal lands after it; #647's self-correction lands. None skipped for being superseded |
| E6 | **the deltas landed, not the blobs** | the SCORE | delta-vs-delta with `index` AND `@@` stripped; **step-relative** pre-image for the file #644 and #647 share |
| E7 | finding 33's class swept per code step | the SCORE | explicit for all four; 7 `.rs` this batch |
| E8 | **NO PUBLISHED HISTORY REWRITTEN** | `merge-base --is-ancestor`; `for-each-ref refs/original/` | ancestor YES; empty |
| E9 | repairs visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | 0 refs; exit 0 either way |
| E10 | the checkpoint | `scripts/floor.sh` + clippy `-D warnings --all-targets` | green; clippy 0 — **the orchestrator's row** |
| E11 | **test-count delta ACCOUNTED FOR, FROM THE RUNNER** | `cargo nextest list` before the floor | **5918 run, 22 skipped** (5908 + 10). ⛔ Derive it from the runner, not from grepping `#[test]` |
| E12 | **every `census:` line is TRUE** | `census.sh --diff` at the tip | `no STOP-8` |
| E13 | **the SCORE discloses what BOUGHT each green** | read the SCORE | every conversion and gate verdict |
| E14 | **every deviation is REPORTED** | the SCORE | honest disagreement passes; **the diff and the runner outrank this brief** |
| E15 | **NO COUNTERPART ACTIVITY**; no unfiltered run | `.floor/`, `git status`, `.pulsare/` mtimes | none; frozen root untouched |
| E16 | no knowingly-red commit; messages survived their heredoc | per-step landing; `git log --format=%B` | none red; no eaten backtick spans |
| **E17** | ⭐ **THE REPLAY IS COMPLETE** | `git log \| grep -c 'REPLAY(grok-rete #'` | **651**, and #651's trailer cites grok's tip `37528f6e0` |

## ⛔ E4 and E5 are the rows that decide the last batch

#646's probe meets a gate that landed one batch ago and forbids every escape hatch. And #646/#648 are a
finding and its withdrawal — the temptation is to skip the finding because its retraction is already in
hand. **Replaying a record means landing what it said, in the order it said it.**

**Runtime prediction:** ~60–100 min — eleven steps, four with code, two conversions.

**What would make me reject the batch:** a hand-edited `.wat`; a rune on the compile gate; a rule deleted
to buy green; #646's finding skipped or softened because #648 withdraws it; a test count grepped rather
than asked of the runner; a typed trailer or paraphrased subject; any `refs/original/` entry; any
`pulsare_yield`.
