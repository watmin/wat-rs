# EXPECTATIONS 7y — replay batch 4y, grok-rete #621 → #640 (written BEFORE the strike)

## ⚠ Which rows were PRE-FLIGHTED, and which are predictions

**Executed and ALL PASS:** origin an ancestor, `refs/original/` empty, 0 replace refs, tree clean, 620
REPLAY steps, 11 docs-only, nine code steps, the range is 35 `.wat` + 25 `.md` + 8 `.rs` + 4 `.sh` + 3
`.wat.bad` + 1 `.edn`, zero hazard rows, zero `wat/` paths, the test delta is **+11 with net 0 ignores**
(counted excluding comments and string literals), **our corpus carries 157 files / 1008
`:wat::rete::where` sites / 715 `defrule` sites against grok's "42 in-scope"**, **2 of grok's 18 #630
paths are absent here**, and the codemod's own header gates it to joins only.

**Predictions, not verified rows:** the migration, the compositions, and every gate verdict.

⛔⛔ **WHAT I COULD NOT MEASURE, AND TRIED TO.** Our exposure to #638's zero-exemption compile gate.
Grok's own census instrument reports `compiles: 0 · cannot compile: 176` here — a uniform failure
including files that provably load clean, i.e. **the instrument is broken on our syntax, not the
corpus.** **E4 scores measuring it correctly, with the instrument proven on a known positive and a known
negative first.**

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh <batch-start> HEAD 621 640` | exit 0; `sources match` |
| E1b | ⛔ **every subject is GROK'S, verbatim** | per step vs `git log -1 --format=%s <C>` | 20 of 20, kind included |
| E1c | ⛔ **trailers PIPED, not typed** | per step vs `commits.tsv`, two-sided | 20 of 20 |
| E2 | docs-only steps are docs-only | the **11** | only `docs/`/`.md` |
| E2b | **and the inverse** | the 9 code steps | each carries ≥1 non-docs file |
| E3 | ⛔ **R21 OBEYED at #630** | `git show` #630; the SCORE | the codemod dry-run, diffed, applied to a list **derived here**; a second pass changing **0 files**; **no hand-edited `.wat`**; our count stated and compared to grok's 18 |
| E4 | ⛔⛔ **#638's gate verdict is REAL, and its instrument was PROVEN** | `-E 'test(rete_compile_gate)'`; the SCORE | green, N > 0 — **or** a red reported as a FINDING with the file list and failing axis per file. **Either way the SCORE shows the instrument passing a known positive and catching a known negative.** **Zero runes** (the gate forbids them); **no rule deleted to buy green** |
| E5 | **#628's reversal landed as grok wrote it** | `git show` #628 | the refinement is applied and reverted; the "helpful" keep is refused |
| E6 | **the deltas landed, not the blobs** (finding 36) | the SCORE | delta-vs-delta, with **step-relative** pre-images where an earlier step in this batch touched the file |
| E7 | finding 33's class swept per code step | the SCORE | explicit for all nine |
| E8 | **the loader gates ran after every `.wat` change** | `-E` on both `wat-scripts/` gates | green, N > 0, verdicts quoted |
| E9 | **NO PUBLISHED HISTORY REWRITTEN** | `merge-base --is-ancestor`; `for-each-ref refs/original/` | ancestor YES; empty |
| E10 | the checkpoint | `scripts/floor.sh` + clippy `-D warnings --all-targets` | green; clippy 0 — **the orchestrator's row** |
| E11 | **test-count delta ACCOUNTED FOR** | predict from the diff BEFORE the floor | **5893 run, 22 skipped** (5882 + 11) |
| E12 | **every `census:` line is TRUE** | `census.sh --diff` at the tip | `no STOP-8` |
| E13 | **the SCORE discloses what BOUGHT each green** | read the SCORE | path list, dry-run, idempotence, every verdict |
| E14 | **every deviation from the brief is REPORTED** | the SCORE | honest disagreement passes; **and if the brief contradicts grok's diff, the diff wins** |
| E15 | **NO COUNTERPART ACTIVITY**; no unfiltered run | `.floor/`, `git status`, `.pulsare/` mtimes | none; frozen root untouched |
| E16 | no knowingly-red commit; messages survived their heredoc | per-step landing; `git log --format=%B` | none red; no eaten backtick spans |

## ⛔ E3 and E4 are the rows that decide this batch

A corpus migration can be hand-patched and a zero-exemption gate can be greened by deleting the rule
that offends it. Both look identical to success from outside. **E3 wants the tool's own output and a
zero-change second pass; E4 wants the instrument proven before its number is believed — because I
already produced a false 176 with grok's own script.**

**Runtime prediction:** ~150–220 min — the heaviest batch since 4o. The weight is #630 (the migration),
#638 (the gate and its three `fixes/` edits), and #635.

**What would make me reject the batch:** a hand-edited `.wat`; grok's path list transcribed instead of
ours derived; a codemod whose second pass still changes files; a rune added to #638's gate; a rule
deleted to buy green; a census number believed without the instrument being proven; a typed trailer or
paraphrased subject; any `refs/original/` entry; any `pulsare_yield`.
