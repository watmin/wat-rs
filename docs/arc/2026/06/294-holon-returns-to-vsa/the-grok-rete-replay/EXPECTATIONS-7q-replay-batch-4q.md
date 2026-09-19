# EXPECTATIONS 7q — replay batch 4q, grok-rete #461 → #480 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended. Fixed now so the result cannot move
the goalposts.

## ⚠ Which rows were PRE-FLIGHTED, and which are predictions

**Executed against HEAD `bca6203b9` and ALL PASS:** E10 (origin an ancestor; `refs/original/` empty), E11
(0 replace refs), tree clean, 460 REPLAY steps, E2 (12 docs-only), E2b (8 code), E5 (zero hazard rows, no
`wat/` path, no `wat-scripts/fixes/` edit, #478's `.wat` is new), the `#[test]` delta (**+9**: 6 at #465,
1 at #474, 2 at #478), **our exposure to #465's gate** (exactly two `census_count("filter:test-reuse")`
sites in `node_share_cost.rs`, both converted by grok's own step), and the #472 collision: our
`token_bindings_representation_dominance` carries a NON-VACUITY check and ONE ordering assertion with a
**4.53–10.15x** margin, is NOT `#[ignore]`d, and has been green in twenty consecutive floors.

**Predictions, not verified rows:** E3–E4, E6–E9, E12–E18.

⛔ **One thing I could NOT measure and did not pretend to.** Whether `no_unknown_ward_rune` (grok's
vocabulary gate, landing at #472) is satisfied by OUR file — which will carry two runed ignores rather
than three — is unknown until it runs. **E4 scores running it and reporting what it says.**

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh bca6203b9 HEAD 461 480` | exit 0; `sources match` |
| E1b | ⛔ **every subject is GROK'S, verbatim** (finding 39) | per step vs `git log -1 --format=%s <C>` | 20 of 20, kind included |
| E1c | **trailers COPIED, not typed** | per step vs `commits.tsv`, two-sided | 20 of 20 |
| E2 | docs-only steps are docs-only | the **12** (#461 #462 #464 #466 #468 #469 #471 #473 #475 #476 #479 #480) | only `docs/`/`.md` |
| E2b | **and the inverse** | the 8 code steps | each carries ≥1 non-docs file |
| E3 | ⛔ **THE #472 RULING LANDED** | `git show` #472 and #477 | the excusare vocabulary and the two OTHER re-wordings land; **`token_bindings_representation_dominance` is NOT `#[ignore]`d**; #477's matching edit to that removed string is skipped; both bodies record why |
| E4 | ⛔ **the vocabulary gate is GREEN on OUR file** | `-E 'test(no_unknown_ward_rune)'` | green, N > 0 — or a reported STOP if it demands a rune on an ignore we do not carry |
| E5 | ZERO hazard paths; no `wat/` | `git diff --name-only` over the range | none |
| E6 | **#465's gate is GREEN, and its verdict was READ** | `-E 'test(kernel_tests_census_count_is_bench_scoped)'` | green, N > 0; the SCORE quotes what it reported, not my forecast; **zero runes** (its exemption list is empty) |
| E7 | **the deltas landed, not the blobs** (finding 36) | the SCORE | delta-vs-delta for every conflicted step |
| E8 | finding 33's class swept per code step | the SCORE | explicit per step, #478's new `.wat` included |
| E9 | the checkpoint | `scripts/floor.sh` + clippy `-D warnings` | green; clippy 0 — **the orchestrator's row** |
| E10 | **NO PUBLISHED HISTORY REWRITTEN** | `merge-base --is-ancestor`; `for-each-ref refs/original/` | ancestor YES; empty |
| E11 | repairs visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | 0 refs; exit 0 either way |
| E12 | **test-count delta ACCOUNTED FOR** | predict from the diff BEFORE the floor | **5865 run, 24 skipped** — +9 tests, and the ignore count unchanged because the one new ignore is not landed |
| E13 | **the SCORE discloses what BOUGHT each green** | read the SCORE | the #472 divergence, every rune, every re-composition |
| E14 | **every `census:` line is TRUE** | `scripts/replay/census.sh --diff` at the tip | `no STOP-8` |
| E15 | every `-E` filter selected N > 0 | each run's own `N tests run` | N > 0 |
| E16 | **every deviation from the brief is REPORTED** | the SCORE | honest disagreement passes; agreement does not earn it |
| E17 | **NO COUNTERPART ACTIVITY**, and no unfiltered test run | `.floor/`, `git status`, `.pulsare/` mtimes | none; frozen root untouched; no `mcp__pulsare__*`; no bare `cargo nextest run` |
| E18 | **a timing red was never re-run into green** | the SCORE | any red carries its verbatim block and arm, and the batch STOPPED |
| E19 | no knowingly-red commit | every step green at its own landing | none |
| E20 | **commit messages survived their heredoc** | `git log --format=%B` per step | no eaten backtick spans |

## ⛔ E3 and E4 are the rows that catch a lazy #472

The cheap way through #472 is to land grok's hunk whole — the ignore applies cleanly, the floor stays
green, and a live gate goes silent with a reason that is false in this tree. **The SCORE shows the test
still running, the two assertions it still makes, and #477's skipped edit — or E3 fails.**

## ⛔ Every `-E` filter must select a NON-ZERO count

A filterset matching nothing runs zero tests and **exits 0**.

**Runtime prediction:** ~80–120 min. The weight is #470 (9 files) and the #472/#477 pair.

**Trap doors named in advance:**
- **#472 — do not silence the test our strike cured**; skip #477's matching edit.
- **#465 — a new gate with an empty exemption list**; predicted green, read its verdict.
- **The count moves both ways** — +9 tests, ignores unchanged under the ruling.
- **No unfiltered `cargo nextest run`.** Filtered runs only.

**What would make me reject the batch:** `token_bindings_representation_dominance` landed `#[ignore]`d; a
rune added to #465's gate; the vocabulary gate's verdict assumed rather than run; a paraphrased subject or
typed trailer; any `refs/original/` entry; any `pulsare_yield`.
