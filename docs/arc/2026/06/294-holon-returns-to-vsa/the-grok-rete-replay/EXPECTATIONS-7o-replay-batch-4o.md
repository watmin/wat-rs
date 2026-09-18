# EXPECTATIONS 7o — replay batch 4o, grok-rete #421 → #440 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended. Fixed now so the result cannot move
the goalposts.

## ⚠ Which rows were PRE-FLIGHTED, and which are predictions

A row is RUN before it is demanded (finding 34 §2). Stated honestly:

**Executed against HEAD `33b717322` and ALL PASS:** E12 (origin an ancestor; `refs/original/` empty), E13
(0 replace refs), tree clean, 420 REPLAY steps, E2 (14 docs-only), E2b (6 code), both M-status-absent docs
created earlier in the range, and the FactBag census: **23 of our `.wat` carry a CODE-position
`Session/facts`/`FactBag/items`** against grok's 26 rewritten (24 exist here, 2 do not); **2 of ours sit
outside grok's set**; our `wat/` exposure is **`oracle/explain.wat` 1, `oracle/fire.wat` 7,
`oracle/insert.wat` 2**; our `src/rete/` exposure is **`fire/rules.rs` 4, `insert.rs` 2**; **both
`session.rs` doors already exist here**; and the one hazard row is `src/stdlib.rs` → `src/load/stdlib.rs`.

**Predictions, not verified rows:** E3–E11, E14–E20.

⛔ **One thing I could NOT measure and did not pretend to.** Whether grok's codemod applies cleanly to
**our** two extra files, and whether its idempotence holds over our corpus, is unknown until it is run —
grok never saw those files. **E4 scores running it and reporting what it did, including a refusal.**

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh 33b717322 HEAD 421 440` | exit 0; `sources match` |
| E1b | ⛔ **every subject is GROK'S, verbatim** (finding 39) | per step vs `git log -1 --format=%s <C>` | 20 of 20, kind included |
| E1c | **trailers COPIED, not typed** | per step vs `commits.tsv`, two-sided | 20 of 20 |
| E2 | docs-only steps are docs-only | the **14** (#422 #424 #425 #427 #428 #430–#437 #439) | only `docs/`/`.md` |
| E2b | **and the inverse** | the 6 code steps (#421 #423 #426 #429 #438 #440) | each carries ≥1 non-docs file |
| E3 | ⛔ **R21 WAS OBEYED — the corpus was migrated BY THE TOOL** | the SCORE + `git show` #438 | the codemod was dry-run on a copy, diffed, then applied to a derived path list; no hand-edited `.wat` except sites the SCORE names with the reason the tool could not reach them |
| E4 | ⛔ **the path list was DERIVED HERE, and idempotence PROVEN** | the SCORE | states our own count (mine says 23; a different number is a result), names the 2 files outside grok's set, and reports a second run changing **0 files** |
| E5 | ⛔ **the new gate is GREEN with an EMPTY exemption list** | `-E 'test(no_raw_factbag_access)'` | green, N > 0; **zero runes added** — the gate's own text forbids them; any unconvertible site was a STOP, not a rune |
| E6 | **the hazard row was re-pointed** | `git show` #438 | the registration lands in `src/load/stdlib.rs`, after `wat/rete.wat`; `src/stdlib.rs` is not recreated |
| E7 | **the loader gates ran after the migration** | `-E 'test(every_wat_scripts_file_loads_on_the_current_runtime)'` and the docs-wat gate | green at #438 and at #440 |
| E8 | **#440's second pass used the tool too** | `git show` #440 | same discipline; any corpus rewrite goes through the codemod |
| E9 | finding 33's class swept per code step | the SCORE | explicit per step; it recurred six times last batch |
| E10 | the checkpoint | `scripts/floor.sh` + clippy `-D warnings` | green; clippy 0 — **the orchestrator's row** |
| E11 | **test-count delta ACCOUNTED FOR** | predict from the diff BEFORE the floor | predicted == actual. My pre-flight, off the diff: **+2** #423, **+2** #426, **+5** #438 = **+9**, so **5856 run, 24 skipped** |
| E12 | **NO PUBLISHED HISTORY REWRITTEN** | `merge-base --is-ancestor`; `for-each-ref refs/original/` | ancestor YES; empty |
| E13 | repairs visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | 0 refs; exit 0 either way |
| E14 | **every `census:` line is TRUE** | `scripts/replay/census.sh --diff` at the tip | `no STOP-8` |
| E15 | **the SCORE discloses what BOUGHT each green** | read the SCORE | the derived path list, the dry-run diff, the idempotence re-run, every hand-converted site |
| E16 | no verdict line WRAPPED | grep each pattern | one-line matches |
| E17 | every `-E` filter selected N > 0 | each run's own `N tests run` | N > 0 |
| E18 | **every deviation from the brief is REPORTED** | the SCORE | honest disagreement passes; agreement does not earn it |
| E19 | **NO COUNTERPART ACTIVITY** | `.floor/`, `git status`, `.pulsare/` mtimes | none; frozen root untouched; no `mcp__pulsare__*` call |
| E20 | no knowingly-red commit | every step green at its own landing | none |
| E21 | **commit messages survived their heredoc** | `git log --format=%B` per step | no eaten backtick spans; last batch this happened twice |

## ⛔ E3, E4 and E5 are the rows that catch a lazy #438

The cheap ways through a corpus migration are to hand-edit the 26 files, to copy grok's path list without
deriving ours, or to rune a site the gate says cannot be runed. All three can look green. **The SCORE
shows the tool's own output, our derived list, and a zero-change second pass — or those rows fail.**

## ⛔ Every `-E` filter must select a NON-ZERO count

A filterset matching nothing runs zero tests and **exits 0**.

**Runtime prediction:** ~120–180 min, nearly all of it #438 and #440. The other four steps are small.

**Trap doors named in advance:**
- **#438 — R21. Run the codemod; never hand-edit the corpus.** Derive our path list; prove idempotence.
- **#438's gate has an empty exemption list** — an unconvertible site is a STOP, not a rune.
- **#438's hazard row** — `src/stdlib.rs` is `src/load/stdlib.rs` here, and load order matters.
- **Two of our corpus files are outside grok's rewrite set** and grok never saw them.
- **Quote the heredoc; read the message back.**

**What would make me reject the batch:** 26 hand-edited `.wat`; grok's path list transcribed instead of
ours derived; a rune added to the FactBag gate; the stdlib registration landed at the old path; a
codemod whose second pass still changes files; a paraphrased subject or typed trailer; any
`refs/original/` entry; any `pulsare_yield`.
