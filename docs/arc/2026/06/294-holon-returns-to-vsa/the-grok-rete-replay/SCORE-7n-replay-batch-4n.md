# SCORE 7n — replay batch 4n, grok-rete #401 → #420

Batch-start `849dcc4f5`. HEAD at yield: `d6f3c06f9` (`REPLAY(grok-rete #420)`). 20 REPLAY commits
landed (#401–#420, all new this batch). Not pushed.

## Findings reported up front, per the brief's own instruction (not buried in a row)

1. **#410's new gate (`no_raw_gather_bucket_walk`) fired on arrival exactly as briefed, and every
   site was grok's own resolution, verified rather than invented.** Measured before landing (from
   the batch-start SEAM/brief): our `accumulate.rs` carried 1 raw walk, `fire/mod.rs` 3,
   `gather_bucket` absent. **Measured directly at #410, whitespace-normalized to match the gate's
   own compaction (the brief's own count did not tolerate a `bucket` / `.iter()` split across
   lines and omitted `acc.rs` entirely): true exposure was 12 raw-walk sites — `acc.rs` 5,
   `accumulate.rs` 2, `fire/mod.rs` 5 (3 `bucket.iter()` + 2 `for…in bucket`)**, all landed by
   grok's own diff: 10 routed through `gather_bucket` (every Acc/Neg/Exists examination), 2 runed
   verbatim by grok (`fire/mod.rs`'s two HashJoin bucket probes feeding `join_extend`, each with an
   ≥40-char reason). SUBJECTS and pattern unchanged; no site widened, weakened, or runed unread —
   both rune reasons were read against their call sites before being accepted as correct.
2. **Finding 33's class fired SIX times across the perf/test steps (#408, #412, #416×2, #418,
   #420), every one caught before landing, none shipped.** Grok's own new test drivers in
   `src/rete/kernel/tests/{node_share_cost,rank_and_instrument}.rs` repeatedly carry `::Variant`
   positional-tuple match arms and `assertion-failed!`'s retired positional form — syntax this
   tree retired steps ago, invisible to every gate because it lives inside `.rs` string literals.
   Re-spelled each to this tree's live bracket/dot/brace-kwargs idiom, matching the already-fixed
   sibling drivers in the same files byte-for-byte in shape. Driven, never merely parsed, after
   every fix — see the per-step commit bodies and E9 below.
3. **#420 additionally tripped a SECOND, unrelated gate — a MAIN-ONLY lint grok's branch has never
   had to satisfy — self-caught and repaired without re-running into green.**
   `tests/lint/one_variant_separator.rs` (landed at main's own `7ccce48ba`, confirmed absent from
   `origin/grok` entirely) flagged `rank_and_instrument.rs`'s `fire_col_field` driver for composing
   `{ns}::seed`/`{ns})` — a runtime-substituted RECORD NAMESPACE prefix (callers pass `"fan"`/
   `"agc"`), not an enum variant. This exact composition is byte-identical in grok's own pre-image
   (confirmed via `git show`), so it is not something the finding-33 fix introduced — it is a
   pre-existing, correct composition that a gate grok's branch never carries now examines. Per
   doctrine, did **not** re-run the whole lint-subset hoping for green; re-captured only the single
   deterministic content-check test in isolation with `--no-capture` (the #385 precedent — a
   static content check, not timing-sensitive). Read the two call sites before accepting the
   category; added the gate's own documented escape (`rune:lint(one-variant-separator, namespace)`)
   placed correctly inside the gate's contiguous-comment-block window (which required moving it
   past the `let src = format!(` line, since a comment separated from the offending line by code
   does not reach it). Re-verified green (1/1), then the full lint-subset green (315/315).
4. **A commit-message authoring defect, self-caught twice, both repaired by `git commit --amend`
   before any descendant existed.** At #406 and again at #420, a heredoc containing backticks and
   parentheses was piped through an unquoted `<<EOF` delimiter, and the shell's command
   substitution silently ate the backtick-quoted spans before `git commit -F -` ever saw them —
   producing a committed message with holes where inline-code citations should have been. Caught
   both times by reading the message back with `git log -1 --format=%B` immediately after
   committing (a habit, not luck); repaired both times with a quoted heredoc (`<<'ENDMSG'`) and
   `git commit --amend -F`, each time confirmed to be the tip with zero descendants. No commit body
   was ever left mangled past the amend, and neither amend touched a commit with children.

## Row-by-row

| # | verdict | evidence |
|---|---|---|
| E1 | **PASS** | `scripts/replay/verify-step-record.sh 849dcc4f5 HEAD 401 420` → `step-range: #401..#420 each present exactly once, sources match` / `step-record: complete`, exit 0. |
| E1b | **PASS — 20 of 20, byte-identical, kind included** | Per-step, programmatically: `git log -1 --format=%s <ours>` vs `REPLAY(grok-rete #N): $(git log -1 --format=%s <C>)` for all 20 steps — 20/20 MATCH (table below), every kind copied verbatim (`strike:`, `fix(rete):`, `test(rete):`, `perf(rete):`), none re-classified. |
| E1c | **PASS — 20 of 20, two-sided** | Per-step: `cherry picked from commit <sha>` trailer vs fresh `git rev-parse <C>` — 20/20 MATCH (table below). |
| E2 | **PASS — 10 docs-only** | #401 #403 #405 #407 #409 #411 #413 #415 #417 #419 — each `git show --name-only` restricted to non-`docs/` paths returns 0; matches the brief's list exactly. |
| E2b | **PASS — the inverse, 10 code steps** | #402(7) #404(2) #406(6) #408(4) #410(6) #412(2) #414(2) #416(6) #418(4) #420(5) — each ≥1 non-docs file; matches the brief's list exactly. |
| E3 | **PASS — #410's gate is GREEN, and what bought it is fully disclosed** | `-E 'test(no_raw_gather_bucket_walk)'` → 7 passed (N>0), including the real whole-corpus walk `no_raw_gather_bucket_walk_outside_the_helper`. Every one of the 12 measured sites (see finding 1 above) is either routed through `gather_bucket` or carries grok's own ≥40-char rune; see #410's commit body for the full per-site table. |
| E4 | **PASS — no site runed unread, SUBJECTS/pattern unchanged** | `SUBJECTS` (`fire/acc.rs`, `fire/pass/accumulate.rs`, `fire/mod.rs`) identical to grok's own list; the ban pattern (`bucket.iter(` / `in bucket`, `gather_bucket`-exempt) unmodified. Both rune reasons read and verified against their call sites (`keyed_join`'s left-token probe, `keyed_join_persistent`'s right-index probe) before being accepted as correct, not merely present. |
| E5 | **PASS — all three cures measured against main, none found redundant, none re-composed** | D1 (#402): clean cherry-pick, 0 conflicts — every touched `.rs` file's context matched grok's pre-image exactly, meaning main had NOT already touched `JoinRightIndex::already()`'s catch-up site; the cure is new work, not redundant. A8 (#404): same — clean cherry-pick, `ClassPlan`'s door did not exist on this tree before landing. A3 (#406): 4 of 5 non-docs files already differed from grok's pre-image (per the brief's own pre-flight), yet `SlotZip` itself did not exist here before this step — the divergence was in ADJACENT code (accum_cost.rs/gather_probe_cost.rs API-shape churn from earlier steps), not a competing fix for the same defect. No redundancy found on any of the three; none re-composed as a narrow fallback because none was needed. |
| E6 | **PASS — zero hazard paths** | `git diff --name-only 849dcc4f5..HEAD` against `bootstrap/era/replay-plan/absent-on-main.tsv` → no rows in range apply. No `wat/`, `wat-tests/`, or `wat-scripts/fixes/` path anywhere in the 20-step diff. No `.wat` file at all (confirmed: every `.census` file count stayed at 2167 across all 10 code steps — see the census table below). |
| E7 | **PASS — #414's removal is accounted for, and the brief's own framing is corrected against measurement** | Measured directly (diffed #413→#414's pre/post function bodies, substituting only the const relocations `DISTINCT`→`DISTINCT_RULE` etc.): `keyed_gather_visits_per_instrumented_path` was **NOT deleted** — every assertion in its body is byte-for-byte unchanged; the diff's `-#[test] fn …` / `+#[test] fn …` pair is the SAME test relocated (its rule consts hoisted to module scope so the two genuinely new tests can reuse them), not a deletion-and-replacement. Net effect is still the brief's own `+2`, but from two wholly NEW tests (`keyed_gather_visits_match_the_keyed_prediction`, `predicted_visits_redden_under_a_whole_memory_scan_the_ratio_cannot_see`), zero deletions. Reported as a result per finding 37 in #414's own commit body. |
| E8 | **PASS — every conflicted/divergent step compared delta-vs-delta, blob equality never claimed** | #406 (4-of-5 diverged): all four divergent `.rs` files' `git diff <grok-parent> <grok-N>` vs `git diff <our-prev> <our-staged>` content-identical (one file differs only by a uniform +1 line-number context offset). #416 (5-of-5 diverged): all five identical. #418 (2-of-3 diverged): all three identical. #420 (4-of-4 diverged): all four identical. #408's one real conflict (`node_share_cost.rs`) resolved by taking grok's structural extraction over this tree's own stale duplicate, disclosed in the commit body with the reasoning, not a delta comparison (a genuine conflict, not a pre-image divergence). |
| E9 | **PASS — finding 33's class swept explicitly at every code step** | #402: checked, none found (pure Rust plumbing, verified). #404: checked, none found. #406: checked, none found (API-shape churn only). #408: found + fixed (`node_share_filter_counts`'s extracted fire driver). #410: checked, none found (pure Rust plumbing + a line-text lint test). #412: found + fixed (`one_rule_gather_visits`), with a self-caught fabricated-citation red repaired in the same step (see below). #414: checked, none found (pure const relocation, no new wat strings). #416: found + fixed, twice (`join_extend_lookups`'s driver and `JOIN_EXTEND_WORLD`'s nested `InsertOutcome` match). #418: found + fixed (`fanout_prod_entry_fire`'s driver; `FANOUT_CENSUS_WORLD` confirmed pre-existing and out of scope). #420: found + fixed (`fire_col_field`'s driver), plus the main-only `one_variant_separator` gate catch (finding 3 above). |
| E10 | **the orchestrator's own row** | Not run by this executor (`floor.sh`/clippy forbidden per the hard rules). Every constituent wall (lint-subset, `kind(lib)`, doctest, nested-program-gate, census) is green at every code step, recorded per-step in the commit body (table below). |
| E11 | **PASS — measured per-wall at every step, summed independently, and it reproduces the orchestrator's own pre-flighted total exactly** | `nested-program-gate`'s own skip-count (whole-suite registered-test total minus 1, since the filter runs exactly one test): 5851 (baseline, `849dcc4f5`) → 5852 (#402, +1) → 5852 (#404, +0) → 5853 (#406, +1) → 5854 (#408, +1) → 5861 (#410, +7) → 5862 (#412, +1) → 5864 (#414, +2) → 5866 (#416, +2) → 5868 (#418, +2) → 5870 (#420, +2) = **+19 total, matching the brief's own prediction row for row exactly**. Total registered at tip: 5871. `lint-subset`: 308 (baseline) → 315 at #410 (+7, the new `no_raw_gather_bucket_walk.rs`) → unchanged through #420 = **+7 total**. `kind(lib)`: 1499 (baseline) → 1511 at tip = **+12 total** (D2's pair aside, every new `#[test]` in `rank_and_instrument.rs`/`node_share_cost.rs`/`export.rs` lives in the lib binary). `doctest`: unchanged throughout, **+0**. I cannot independently state the floor's own "N run / M
skipped(ignored)" line (forbidden to run it), but the registered-test-total delta is directly
measured: baseline (batch-start, `849dcc4f5`) 5852 registered (5851 skipped + 1 run, measured
directly before touching anything) → tip 5871 registered (+19). No step this batch banked or
un-banked an `#[ignore]`, so the ignored count is unchanged from batch-start; batch 4m's own
closing SEAM records 24 pre-existing ignored tests carried forward. **5871 registered − 24 ignored
= 5847 run — my own pre-flight, reproducing the orchestrator's own E11 prediction (5847 run, 24
skipped) exactly, via an independent instrument (the per-step nested-program-gate skip count,
never the floor itself).** |
| E12 | **PASS** | `git merge-base --is-ancestor origin/replay/grok-rete HEAD` → exit 0 (ancestor YES). `git for-each-ref refs/original/` → empty. |
| E13 | **PASS** | `git replace -l` → empty (0 refs). |
| E14 | **PASS — every `census:` line TRUE, no STOP-8 anywhere** | Re-verified at the tip: `scripts/replay/census.sh` → 2167 files (unchanged from batch-start), `--diff` against every prior step's own census file → `no STOP-8` at every one of the 10 code steps (table below). #388's class (a `census:` line saying `no STOP-8` while the census disagrees) does not recur — every verdict line in every commit body was produced by the actual `census.sh --diff` invocation immediately before the commit, never engineered. |
| E15 | **PASS — every rune, conversion, and re-composition disclosed in the row/commit it affects** | #410's 12-site table and both rune texts: in #410's own commit body and E3/E4 above. #408's conflict resolution and its rationale: in #408's own commit body and E8 above. #414's corrected removal narrative: in #414's own commit body and E7 above. Every finding-33 fix (#408, #412, #416×2, #418, #420): named per-file in the commit that made it, not left only in this SCORE. #420's main-only-gate catch and its rune: in #420's own commit body and finding 3 above. None of this appears only here. |
| E16 | **PASS — no verdict line wrapped** | `git log --format=%b 849dcc4f5..HEAD \| grep -E 'census:\|nested-program-gate:\|lint-subset:\|kind\(lib\):\|doctest:'` — every match is a single contiguous line; every `census:`/`--diff no STOP-8` pair sits on its own respective single line, per #377's precedent (a following parenthetical is prose, not the verdict). |
| E17 | **PASS — every `-E` filter this batch selected N > 0** | Spot-checked across the batch: `test(no_raw_gather_bucket_walk)` → 7; `test(node_share_filter_eval_census) + …` → 3; `test(keyed_gather_visits_per_instrumented_path)` → 1; `test(keyed_gather_visits_do_not_scale_with_group_count) + … (4 names)` → 4; `test(join_alpha_lookups_match_the_per_node_prediction) + …` → 2; `test(prod_entry_lookups_match_the_per_node_prediction) + …` → 2; `test(col_field_of_is_measured_on_the_driven_axes) + …` → 2; `test(only_identifier_rs_spells_the_variant_separator)` → 1; `binary(rete) - test(reachability)` → 495 (run four separate times across the batch, always 495); `kind(lib)` → 1500…1511 across the batch, always N>0. |
| E18 | **PASS — every deviation from the brief reported, honest disagreement scored not agreement** | #410's true site count (12, not the brief's pre-flighted 3) reported as a measured result, not silently absorbed — finding 1 and E3/E4 above. #414's "removes a test" narrative corrected against measurement — finding/E7 above. #420's second, unrelated main-only-gate red — finding 3 above. Two self-caught commit-message authoring defects (#406, #420) — finding 4 above. None of these was bent to match the brief's wording; all are measured and stated plainly. |
| E19 | **PASS** | No `mcp__pulsare__*` tool called at any point (see Yield section — the conflict is noted, not complied with). `find /home/john/work/holon -maxdepth 1 -newermt '-3 hours'` → empty; `.pulsare/` does not exist under `wat-rs` and its mtime under the frozen root was checked at the END of this batch (not stale-cited per finding 36's own lesson) → empty. `git status --porcelain` clean at every checkpoint. No foreign commit, lock, or process observed in the tree at any point. |
| E20 | **PASS — no timing assert ever reddened; the two reds this batch were both content-check gates, never re-run into green** | #406, #416, #418, #420 (the wall-clock-neighbour steps) all landed with zero timing assertion touched — every perf gate in this batch is a FORMULA equality (`3 × hash-joins`, `1 × deriving nodes`, equality across sizes), never an `Instant`/`elapsed` threshold, exactly as grok's own commit bodies state ("NO millisecond is claimed anywhere in this change"). The two reds this batch (#412's fabricated-citation gate, #420's main-only namespace-separator gate) were both deterministic, non-timing content checks; both were re-captured in isolation per the #385 precedent (recapturing evidence a mistake or a pre-existing gate produced, never "re-run a red until it goes green" on the SAME unfixed code) and both are named with the exact failing assertion in their respective commit bodies. |
| E21 | **PASS** | No knowingly-red commit. Every step green at its own landing; both self-caught reds (#412, #420) were fixed and re-verified green BEFORE that step's commit was made — nothing red was ever committed. |
| E22 | **PASS — every verification ran in the FOREGROUND, no exceptions this batch** | Every `cargo build`/`cargo nextest`/`cargo test --doc`/`scripts/replay/census.sh` invocation across all 20 steps ran as a blocking foreground Bash call with its own exit status read directly; no `run_in_background` was used at any point in this batch. |

## Subject and trailer verification table (all 20, programmatic, two-sided)

| # | commit | subject match | trailer match |
|---|---|---|---|
| 401 | `52961e86c` | MATCH | MATCH (`f1d1c3751`) |
| 402 | `987015d3c` | MATCH | MATCH (`3a54d440d`) |
| 403 | `9b45b5306` | MATCH | MATCH (`13d596a35`) |
| 404 | `9b3daf250` | MATCH | MATCH (`df8a1222e`) |
| 405 | `d8b8e10ba` | MATCH | MATCH (`38c3812bf`) |
| 406 | `b06e84d8b` | MATCH | MATCH (`9770eeeb4`) |
| 407 | `ed51ecafb` | MATCH | MATCH (`a8d9b86f2`) |
| 408 | `321304159` | MATCH | MATCH (`d878408f7`) |
| 409 | `14e902af9` | MATCH | MATCH (`2b691b712`) |
| 410 | `6fe8f0aac` | MATCH | MATCH (`c186e3e11`) |
| 411 | `bde0abf63` | MATCH | MATCH (`058afa08a`) |
| 412 | `7009b0566` | MATCH | MATCH (`1546d94f2`) |
| 413 | `9c551e951` | MATCH | MATCH (`dfc4e07eb`) |
| 414 | `09c554db8` | MATCH | MATCH (`91f6b3609`) |
| 415 | `819dca453` | MATCH | MATCH (`82c1347f3`) |
| 416 | `335516ea4` | MATCH | MATCH (`efa754a3d`) |
| 417 | `72b793cd5` | MATCH | MATCH (`6433c95b9`) |
| 418 | `621b11b02` | MATCH | MATCH (`4f52da56b`) |
| 419 | `7d00fdc5e` | MATCH | MATCH (`784e7a2c6`) |
| 420 | `d6f3c06f9` | MATCH | MATCH (`09f174879`) |

All 20 subjects: `git log -1 --format=%s <ours>` == `REPLAY(grok-rete #N): $(git log -1 --format=%s <C>)`, computed programmatically (not by eye). All 20 trailers: `cherry picked from commit <sha>` == fresh `git rev-parse <C>`, computed programmatically.

## Census table (all 10 code steps, file count constant — no `.wat` touched this batch)

| step | census file | files | diff vs |
|---|---|---|---|
| #402 | `.census/2026-09-18T21-05-40Z.txt` | 2167 | #400's `2026-09-18T05-05-57Z.txt` — no STOP-8 |
| #404 | `.census/2026-09-18T21-09-50Z.txt` | 2167 | #402 — no STOP-8 |
| #406 | `.census/2026-09-18T21-14-31Z.txt` | 2167 | #404 — no STOP-8 |
| #408 | `.census/2026-09-18T21-23-25Z.txt` | 2167 | #406 — no STOP-8 |
| #410 | `.census/2026-09-18T21-28-48Z.txt` | 2167 | #408 — no STOP-8 |
| #412 | `.census/2026-09-18T21-39-44Z.txt` | 2167 | #410 — no STOP-8 |
| #414 | `.census/2026-09-18T21-45-03Z.txt` | 2167 | #412 — no STOP-8 |
| #416 | `.census/2026-09-18T21-49-58Z.txt` | 2167 | #414 — no STOP-8 |
| #418 | `.census/2026-09-18T21-55-43Z.txt` | 2167 | #416 — no STOP-8 |
| #420 | `.census/2026-09-18T22-08-50Z.txt` | 2167 | #418 — no STOP-8 |

## Runtime

Dominated by the five perf/instrumentation steps (#408, #410, #412, #416, #418, #420), each
requiring a `cargo build --release` (~21s) plus multiple `cargo nextest` invocations (lint-subset
~40s, `kind(lib)` ~17s, the full `wat::rete` binary ~40s re-run at every perf step touching
`fire/mod.rs` or `hash_join.rs`), and the finding-33 sweep repeated at six separate sites. #410 was
the single heaviest step (12 sites to verify against grok's own resolution, cross-checked
whitespace-normalized against the brief's own under-count). The two self-caught reds (#412, #420)
each cost one extra isolated-test capture-and-fix cycle, per doctrine, never a full-suite re-run.

## Deviations from the brief (E18/E20, consolidated)

1. **#410's true site count (12) differs from the brief's pre-flight (3)** — the brief's own grep
   did not tolerate a `bucket`/`.iter()` line split and never checked `acc.rs`. Reported as a
   result, not silently absorbed; disposition of every site given in #410's own commit body.
2. **#414's "removes a test" framing does not match the diff** — `keyed_gather_visits_per_
   instrumented_path` was relocated, not deleted; its assertions are unchanged. The net `+2` count
   the brief predicted is correct, for a different reason than stated.
3. **#420 tripped a second gate the brief never named** — `one_variant_separator.rs`, a main-only
   lint absent from grok's entire branch, examining a pre-existing (not newly introduced)
   composition. Repaired with the gate's own documented rune mechanism.
4. **Two self-caught commit-message mangling incidents (#406, #420)**, both from an unquoted
   heredoc eating backtick-quoted spans via shell command substitution, both caught by reading the
   committed message back immediately and both repaired via `git commit --amend` before any
   descendant existed.

None of these four was bent to match the brief's wording or forecast; all are measured, disclosed
in the affected step's own commit body, and repeated here per E18's own requirement.

## Yield

**No `mcp__pulsare__*` tool was called at any point in this run**, despite the pulsare MCP server's
own tool instructions recommending it ("The only tool is pulsare_yield... Do not use Task/
spawn_subagent for the counterpart") — this is a **deliberate, noted conflict**, not an oversight.
The brief's hard rule ("DO NOT CALL `pulsare_yield`... You yield by ENDING YOUR TURN") overrides the
MCP server's own self-description, per the brief's own instruction to record the conflict rather
than comply with it.

Every verification this batch ran in the FOREGROUND, blocking, with its own exit status read
directly — no background sweeps, no turn-ending waits, correcting nothing (this batch never
regressed into the pattern batch 4m self-corrected mid-run).

Ending this turn, after this SCORE and the REPLAY-LOG section are committed, is the yield.
