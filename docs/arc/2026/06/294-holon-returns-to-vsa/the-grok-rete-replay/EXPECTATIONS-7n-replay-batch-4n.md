# EXPECTATIONS 7n — replay batch 4n, grok-rete #401 → #420 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended. Fixed now so the result cannot move
the goalposts.

## ⚠ Which rows were PRE-FLIGHTED, and which are predictions

A row is RUN before it is demanded (finding 34 §2). Stated honestly:

**Executed against HEAD `849dcc4f5` and ALL PASS:** E12 (origin an ancestor; `refs/original/` empty), E13
(0 replace refs), tree clean, 400 REPLAY steps, E2 (10 docs-only), E2b (10 code), E6 (zero hazard rows; no
`wat/`, `wat-tests/` or `wat-scripts/fixes/` path in the range; no `.wat` file at all; no M-status-absent
path), **our exposure to #410's gate** (`accumulate.rs` 1 raw walk, `fire/mod.rs` 3, `gather_bucket`
absent), **the wall-clock census** (`gather_probe_cost.rs` 24, `accum_cost.rs` 23, `census.rs` 4), and
**the divergence table** (#416 5-of-5, #420 4-of-4, #406 4-of-5, #410 3-of-5, #418 2-of-3 touched `.rs`
differ from grok's pre-image).

**Predictions, not verified rows:** E3–E5, E7–E11, E14–E19.

⛔ **One thing I could NOT measure and did not pretend to.** Whether grok's three cures (D1 #402, A8 #404,
A3 #406) are already covered by main's own rete work — as D10/D11 turned out to be at 4k — is unknown
until each is composed. **E5 scores measuring it and reporting redundancy as a result, not landing a
second mechanism silently.**

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh 849dcc4f5 HEAD 401 420` | exit 0; `sources match` |
| E1b | ⛔ **every subject is GROK'S, verbatim** (finding 39) | per step, `git log -1 --format=%s <ours>` vs `REPLAY(grok-rete #N): $(git log -1 --format=%s <C>)` | 20 of 20 identical, kind included |
| E1c | **trailers COPIED, not typed** | per step, trailer vs `commits.tsv`, two-sided | 20 of 20 |
| E2 | docs-only steps are docs-only | the **10** (#401 #403 #405 #407 #409 #411 #413 #415 #417 #419) | only `docs/`/`.md` |
| E2b | **and the inverse** | the 10 code steps | each carries ≥1 non-docs file |
| E3 | ⛔ **#410's gate is GREEN, and what bought it is disclosed** | `-E 'test(no_raw_gather_bucket_walk)'`; `git show` #410 | green, N > 0; every site in our `accumulate.rs` and `fire/mod.rs` either routed through `gather_bucket` or runed with a ≥40-char reason naming why it is not an examination |
| E4 | ⛔ **no site was runed unread, and the gate was not weakened** | the SCORE + diffs | `SUBJECTS` unchanged, pattern unchanged; each rune quotes the code it covers and says what that code does |
| E5 | **redundancy against main's own rete was MEASURED** | the SCORE, per cure (#402 #404 #406) | each says whether main already covered it, with the evidence; a re-composition is disclosed, never silent |
| E6 | ZERO hazard paths in range | `git diff --name-only` vs `absent-on-main.tsv`, `wat/`, `wat-scripts/fixes/` | none |
| E7 | **#414's removal is accounted for** | the SCORE | states what `keyed_gather_visits_per_instrumented_path` asserted and which of the three new formula tests covers it |
| E8 | **the deltas landed, not the blobs** (finding 36) | the SCORE | for each conflicted step, the comparison is delta-vs-delta; blob equality is never claimed |
| E9 | finding 33's class swept per code step | the SCORE | each code step answers explicitly; "not applicable" is an answer, silence is not |
| E10 | the checkpoint | `scripts/floor.sh` + clippy `-D warnings` | green; clippy 0 — **the orchestrator's row** |
| E11 | **test-count delta ACCOUNTED FOR** | predict from the diff BEFORE the floor | predicted == actual. My pre-flight, read off the diff: **+1** #402, **+1** #406, **+1** #408, **+7** #410, **+1** #412, **+3−1** #414, **+2** #416, **+2** #418, **+2** #420 = **+19**, so **5847 run, 24 skipped** |
| E12 | **NO PUBLISHED HISTORY REWRITTEN** | `merge-base --is-ancestor origin/replay/grok-rete HEAD`; `for-each-ref refs/original/` | ancestor YES; empty |
| E13 | repairs visible to `push` | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | 0 refs; exit 0 either way |
| E14 | **every `census:` line is TRUE** | `scripts/replay/census.sh --diff` at the tip | `no STOP-8`; #388's class does not recur |
| E15 | **the SCORE discloses what BOUGHT each green** | read the SCORE | every rune, conversion, re-composition and adapted assertion appears in the row it affects |
| E16 | no verdict line WRAPPED | grep each pattern | one-line matches; `--diff`/`no STOP-8` contiguous |
| E17 | every `-E` filter selected N > 0 | each run's own `N tests run` | N > 0 |
| E18 | **every deviation from the brief is REPORTED** | the SCORE | honest disagreement passes this row; agreement does not earn it |
| E19 | **NO COUNTERPART ACTIVITY** | `.floor/` for foreign runs; `git status` for foreign artifacts; `.pulsare/` mtimes | none; frozen root untouched; no `mcp__pulsare__*` call |
| E20 | **a timing red was never re-run into green** | the SCORE | if a cost test reds at #406/#416/#418/#420, the SCORE carries the verbatim block and the arm, and the batch STOPPED (the 4i precedent) |
| E21 | no knowingly-red commit | every step green at its own landing | none |
| E22 | **verifications ran in the FOREGROUND** | the SCORE | stated plainly; last batch's background sweeps were a deviation |

## ⛔ E3 and E4 are the rows that catch a lazy #410

The cheap ways through a new gate are to rune every site with a boilerplate reason, to trim `SUBJECTS`, or
to loosen the pattern until nothing matches. All three look green. **The SCORE shows, per site, what the
code does and why it is an examination or a probe — or those rows fail.**

## ⛔ Every `-E` filter must select a NON-ZERO count

A filterset matching nothing runs zero tests and **exits 0**.

**Runtime prediction:** ~100–150 min. The weight is #410 (the gate plus our own site conversions), then
#402 (7 files) and the three perf steps, all of which conflict.

**Trap doors named in advance:**
- **#410 — the gate fires here.** Repair at the step; read every site; never rune blind.
- **#406/#416/#418/#420 — wall-clock neighbours.** A timing red is the 4i class: stop, do not re-run.
- **Conflicts everywhere** — compose, and compare deltas not blobs.
- **#414 deletes a test.** Account for what it asserted.
- **Subjects and trailers are copied from git, never typed.**

**What would make me reject the batch:** a runed site nobody read; `SUBJECTS` or the pattern weakened; a
cure landed twice because redundancy went unmeasured; a timing red re-run instead of reported; a
paraphrased subject or a hand-typed trailer; a `census:` line that says `no STOP-8` while the census says
otherwise; any `refs/original/` entry; any `pulsare_yield`.
