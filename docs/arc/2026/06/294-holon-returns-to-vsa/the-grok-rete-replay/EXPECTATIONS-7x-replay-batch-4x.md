# EXPECTATIONS 7x — replay batch 4x, grok-rete #601 → #620 (written BEFORE the strike)

## ⚠ Which rows were PRE-FLIGHTED, and which are predictions

**Executed and ALL PASS:** origin an ancestor, `refs/original/` empty, 0 replace refs, tree clean, 600
REPLAY steps, 13 docs-only, seven code steps, the range is 18 `.md` + 11 `.rs` + 10 `.sh` + 7 `.wat` + 5
`.clj`, zero hazard rows, zero new gates, the `#[test]` delta is **+6** with no `#[ignore]` moving, the
divergence table (**#619 4/8, #607 3/5**, rest 0–1), **all five new `.wat` failing `--check` here** with
42 positional `assertion-failed!` / 25 `PersistentVector/conj` / 51 `:wat::core::i64::*`, and **#610's
`wat/` edit being 45 CODE lines, not a comment sweep**.

**Predictions, not verified rows:** the conversions, the compositions, the gate verdicts, the floor.

⛔ **What I could NOT measure.** Whether `convert.sh` cures all five cleanly — this is a larger and more
varied retired-form set than #537's nine (three distinct families at once). **E3 scores running it and
reporting what it did, including a refusal.**

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh <batch-start> HEAD 601 620` | exit 0; `sources match` |
| E1b | ⛔ **every subject is GROK'S, verbatim** | per step vs `git log -1 --format=%s <C>` | 20 of 20, kind included |
| E1c | ⛔ **trailers PIPED, not typed** | per step vs `commits.tsv`, two-sided | 20 of 20 |
| E2 | docs-only steps are docs-only | the **13** | only `docs/`/`.md` |
| E2b | **and the inverse** | the 7 code steps | each carries ≥1 non-docs file |
| E3 | ⛔ **the five new `.wat` were CONVERTED BY THE CHAIN** | `git show` #616/#619; the SCORE | every one through `scripts/replay/convert.sh`; each named with what changed; **no hand-edited `.wat`**; all five `--check` rc=0 after |
| E4 | ⛔ **#610's stdlib change composed, and its gates ran** | `git show` #610; the SCORE | the 45-line `rule-negates` change landed; loader gates green with N > 0; the oracle's own tests run and quoted |
| E5 | **the deltas landed, not the blobs** (finding 36) | the SCORE | delta-vs-delta for every conflicted file, with the **step-relative** pre-image where an earlier step in this batch touched it |
| E6 | finding 33's class swept per code step | the SCORE | explicit for all seven; **10 `.sh` this batch** |
| E7 | **each `.rs`-touching step carries its FULL record line** | `git show -s` per step | `census:` + `nested-program-gate:` + `lint-subset` + `kind(lib)` + `doctest`, each on ONE line; `.sh`-only steps say why they need none |
| E8 | **NO PUBLISHED HISTORY REWRITTEN** | `merge-base --is-ancestor`; `for-each-ref refs/original/` | ancestor YES; empty |
| E9 | repairs visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | 0 refs; exit 0 either way |
| E10 | the checkpoint | `scripts/floor.sh` + clippy `-D warnings --all-targets` | green; clippy 0 — **the orchestrator's row** |
| E11 | **test-count delta ACCOUNTED FOR** | predict from the diff BEFORE the floor | **5883 run, 22 skipped** (5877 + 6) |
| E12 | **every `census:` line is TRUE** | `census.sh --diff` at the tip | `no STOP-8` |
| E13 | **the SCORE discloses what BOUGHT each green** | read the SCORE | every conversion, composition and gate verdict |
| E14 | **every deviation from the brief is REPORTED** | the SCORE | honest disagreement passes; **and if this brief contradicts grok's diff, the diff wins** |
| E15 | **NO COUNTERPART ACTIVITY**; no unfiltered run | `.floor/`, `git status`, `.pulsare/` mtimes | none; frozen root untouched |
| E16 | no knowingly-red commit; messages survived their heredoc | per-step landing; `git log --format=%B` | none red; no eaten backtick spans |

## ⛔ E3 and E4 are the rows that catch a lazy 4x

Five broken `.wat` can be hand-patched to green in minutes, bypassing the recorded chain that exists so a
migration is reproducible. And a 45-line stdlib change can be pasted in without ever asking the loader
whether the stdlib still loads. **The SCORE shows the chain's own output per file and the gates' own
verdicts, or those rows fail.**

**Runtime prediction:** ~110–160 min. The weight is #619 (13 files, four conversions), #610 (stdlib),
and #607.

**What would make me reject the batch:** a hand-edited `.wat`; a stdlib change landed without running the
loader gates; a blob comparison passed off as a delta check; a batch-relative pre-image where a
step-relative one was needed; a typed trailer or paraphrased subject; any `refs/original/` entry; any
`pulsare_yield`.
