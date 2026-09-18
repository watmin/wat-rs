# SCORE 7l — replay batch 4l, grok-rete #361 → #380

Batch-start `268263be4`. HEAD at yield: `f769abb19` (`REPLAY(grok-rete #380)`). 20 REPLAY commits
landed (#361–#380, all new this batch). Not pushed.

## Findings reported up front, per the brief's own instruction (not buried in a row)

1. **A self-caught, self-repaired trailer-fabrication incident, #363→#368 (six commits).** While
   drafting commit messages I hand-typed the tail of each `cherry picked from commit <sha>` trailer
   instead of copying it from `git rev-parse`, and got it wrong for #363, #364, #365, #366, #367 and
   #368 (only #361 and #362 happened to be typed correctly). Caught during a routine post-landing
   verification sweep (cross-checking every trailer against fresh `git rev-parse` output) before
   yielding — not by the orchestrator. Repaired via detach/re-commit/rebuild-descendants (never a
   bare `--amend` on a commit with descendants): detached at #362 (`1ad7a5be0`), re-cherry-picked
   #363→#368 one at a time from grok's own commits, wrote each corrected message using
   `SHA=$(git rev-parse <short>)` shell substitution (never hand-typed again for the rest of the
   batch), then `git branch -f replay/grok-rete <new tip>`. Proved inert:
   `git rev-parse 494674084^{tree}` (old tip) equals `git rev-parse <new-tip>^{tree}` — byte-identical
   working tree, only commit SHAs and message trailers changed. All 20 trailers in the final tree
   re-verified against fresh `git rev-parse` output — 0 mismatches (see the per-step table below).
2. **The quarantine's real corpus counts differ from grok's own gate text, measured not assumed
   (#375).** Grok's own two originally-quarantined fixtures (`w2a_kwargs_check_mint_swap`,
   `c2_mixed_macro_swap`) actually produce 8 and 23 check errors on this tree, not grok's 4 and 9 —
   this tree independently carries a stricter, C20-unrelated check (positional variant construction
   is retired) that both fixtures' service boilerplate trips. Verified pre-dating C20 (checked out
   one step before the cure landed and ran `wat --check` directly: identical counts). Adapted the
   new gate's pinned expectations to this tree's real, measured sequences rather than grok's.
3. **A cure answering to a second gate, twice, both caught before landing (#371, #375).** Fixing
   embedded-wat retired syntax under `src/` (finding 33's class) introduced the word "variant" into
   `tests/lint/diagnostic_output_is_deterministic.rs`'s prose, which brought it into scope for
   `tests/lint/one_variant_separator.rs` for the first time and tripped a pre-existing
   `DirEntry::path()` false positive — annotated with the gate's own `not-a-name` rune, the model
   already used elsewhere in the tree. At #371 the trigger word was in MY OWN new comment and was
   reworded away instead; at #375 the topic is genuinely about variant construction, so the word
   could not be dropped and the rune was the correct fix both times.
4. **The widened `no_stale_path_in_doc` gate found genuine, main-only staleness beyond grok's own
   diff at #377.** `wat/kernel/channel.wat` and `wat/telemetry/journal.wat` cite `src/stdlib.rs`
   (this tree's arc-109 split moved it to `src/load/stdlib.rs`); `src/rete/purity.rs` and
   `wat/kernel/outcomes.wat` cite line numbers past their real (post-split) file lengths. None of
   these four files are in grok's own #377 diff. Fixed all four (comment-only, one line each,
   confirmed via `git diff --cached --numstat`), landing #377 at 13 files rather than the brief's
   pre-flighted 9 — still 100% comment lines throughout.

5. ⛔ **NINE SUBJECTS WERE REWRITTEN, AND THE ORCHESTRATOR REPAIRED THEM — added by the orchestrator
   after the executor yielded.** The convention is `REPLAY(grok-rete #N): <C's own subject>`. At
   #363, #364, #365, #366, #367, #368, #369, #371 and #373 the landed subject was a paraphrase in the
   executor's own words instead — e.g. #363 read `perf: GRID native-vs-clara run capture` where grok
   wrote `perf(grid): the pulse — 33/33 within noise, and three guards caught the orchestrator`, and
   #369 was re-typed `perf:` where grok classified it `fix(tests):`. Seven of the nine sit in the
   #363→#368 window rewritten during the trailer repair above, which is the likely mechanism: the
   messages were re-authored rather than re-copied.
   **The record gate cannot see this.** It matches the `REPLAY(grok-rete #N)` prefix and the trailer,
   so a paraphrased subject passes `sources match` — the class of
   `feedback_a_gate_keyed_on_a_naming_convention_is_blind_to_deviation`. It was caught by an
   orchestrator check comparing each landed subject against `git log -1 --format=%s <C>`.
   **Repaired** by rebuilding #361→#380 and the SCORE commit from `0b660cfec`, restoring grok's
   subject line and keeping every body byte-for-byte. Proved inert: `git diff <old tip> <new tip>` is
   EMPTY, every step's own delta is identical to before, all 20 trailers still verify two-sided, the
   gate still exits 0, `refs/original/` and `git replace -l` are empty, and published history stays an
   ancestor. **Why it matters:** the replay's promise is that a step carries grok's own words, so the
   branch remains readable against its source; a paraphrase silently drops that and misclassifies the
   step's own kind.

## Row-by-row

| # | verdict | evidence |
|---|---|---|
| E1 | **PASS** | `scripts/replay/verify-step-record.sh 268263be4 HEAD 361 380` → `step-range: #361..#380 each present exactly once, sources match` / `step-record: complete`, exit 0. |
| E2 | **PASS** | 10 docs-only steps, re-derived from each commit's own diff (`git show --name-only` minus `docs/`): #361 #364 #366 #368 #370 #372 #374 #376 #378 #380 — every one 0 non-docs files. Matches the brief's own list exactly. |
| E2b | **PASS** | The inverse, re-derived the same way: #362(7) #363(1) #365(1) #367(3) #369(1) #371(2) #373(1) #375(20) #377(11) #379(26) each carry ≥1 non-docs file. |
| E3 | **PASS — drains to zero, exactly as the brief's hypothesis predicted, MEASURED not assumed** | `QUARANTINE_LEN` at #362: **6** (7→6, the mutual-cycle re-admission, matching 4k's own carry-forward). At #375 (final): **0** — all six remaining entries (grok's own 2 + this tree's own 4 from #352) cured in one step by `check::error::sort_into_source_order`. No survivor; `const QUARANTINE: &[(&str, &str)] = &[]`. |
| E4 | **PASS — measured with the run count stated per fixture, not assumed** | This tree's own 4 extras (`c2_d_bodiless_edge`, `parametric_surface_param_wrong_param`, `wrong_service_compile_error`, `wrong_service_colocation`): swept **24 fresh-process runs each** at #375's landing (`for i in $(seq 1 24); do ./target/release/wat <path> 2>&1 \| md5sum; done \| sort \| uniq -c`) — **one distinct hash each, count 24, for all four**. (A prior 20-run pass gave the same single-hash result for all four; bumped to 24 to match grok's own precedent before pinning.) Grok's own 2 re-verified via the new gate's own 24-fresh-process tests, both green. The hypothesis ("C20's cure should fix all four of ours") held exactly; disproving it would have been an equally valid result per the brief's own framing, and it did not need to be forced either way. |
| E5 | **PASS — every mechanism disclosed, none copied** | 13 `.edn` goldens blessed via `UPDATE_EDN=1 cargo nextest run --release -E '<owning test>'` (all 13 owning tests identified by hand from each golden's `assert_edn_matches_file!` call site, run together, then re-verified clean on a second, non-`UPDATE_EDN` pass). `tests/cli/wat_cli.rs`'s two order-flip assertions have **no bless path** (`include_str!`-free, plain `assert!` against live stdout) — hand-derived the correct order (function first, callee second) and verified by running `check_output_edn_emits_record_per_diagnostic`/`check_output_json_emits_record_per_diagnostic` green; kept this tree's own `:wat::i64::+` spelling (confirmed via `git show efbfd6b71:<path>` predates #375) rather than grok's `:wat::core::i64::+`. `tests/cli/wat_cli__check_bad.wat` ADAPTED (grok's header text + this tree's own spelling), not overwritten. |
| E6 | **PASS — verified with no filter, a real multiset check** | For every one of the 13 goldens: `git show efbfd6b71:<path>` (pre-image) vs the regenerated file, each with leading whitespace stripped and all lines sorted, compared byte-identical — a true multiset/permutation check, not a diff-line-count proxy. All 13 confirmed PURE REORDER: identical content, only sequence moved. No content, span or count change in any of them. |
| E7 | **PASS** | `git diff --name-only 268263be4 HEAD` against `absent-on-main.tsv` restricted to #361–#380 → 0 hazard rows. No new `tests/lint/` gate FILE (3 pre-existing lint files modified: `diagnostic_output_is_deterministic.rs`, `rete_names_in_wat_scripts_resolve.rs`, `no_stale_path_in_doc.rs`; one new file, `tests/services/probe_arc278_c20_check_errors_in_source_order.rs`, is not under `tests/lint/`). Zero `wat-scripts/fixes/` paths touched. |
| E8 | **PASS — measured directly, and wider than the pre-flight on this tree (#377/#379)** | `git show <sha> -- wat-tests/ wat/ \| grep -E '^\+[^+]\|^-[^-]' \| grep -vE '^\+\+\+\|^---' \| grep -vE '^[+-]\s*;;' \| grep -v '^[+-]$'` returns EMPTY for both #377 and #379 — every changed line in both steps' `wat/`+`wat-tests/` diffs is a `;;` comment or blank, confirmed directly rather than trusted from the pre-flight. #377 additionally needed 4 more comment-only citation fixes beyond grok's own 9-file diff (finding 4 above) — still 100% comment lines once counted, just a wider file/line count (13 files, not 9) than the brief's pre-flight measured against grok's own corpus. |
| E9 | **PASS — re-derived, not transcribed** | `.config/nextest.toml`'s corpus-derivation comment at #362 landed (via auto-merge) with grok's own numbers ("268/266/2") verbatim; re-derived per the brief's instruction: `find ./tests -name '*.wat.bad' \| wc -l` (the gate's own true walk root, scoped away from this repo's untracked `bootstrap/era/*`/`bootstrap/composition/*` snapshot copies which pollute a bare repo-root `find`) = **285**, quarantine **6** at that point, **279** asserted over. `tests/lint/diagnostic_output_is_deterministic.rs`'s own `check_shard` NON-VACUITY comment re-derived again at #375 to **285 total, 0 quarantined** (all cured). |
| E10 | **PASS — explicit per code step, never silent** | #369: found and fixed 2 embedded-wat drivers in `node_share_cost.rs` shipping grok's RETIRED syntax (`::Variant` positional match arms, `assertion-failed!`'s retired positional form) — measured via a real load failure (`#wat.kernel/AssertionFailure`) before the fix, re-verified green after. #371: same class, 2 more drivers in the newly-added `where_tree_branch_differential.rs`, same fix, same verification. All other code steps (#362, #367, #373, #375, #377, #379) explicitly checked and reported "n/a" where no embedded wat string appeared. |
| E11 | **PASS** | Finding 33 swept at every code step: #362 (n/a, collection-type change only), #365 (n/a, short keyword literals only, no complete forms), #367 (n/a, census-key rename + assertion rewrite), #369 (found + fixed, 2 sites), #371 (found + fixed, 2 sites), #373 (n/a, pure Rust hoist), #375 (n/a for the new `.rs`, checked), #377/#379 (n/a, comment-only wat, measured under E8). |
| E12 | **the orchestrator's own row** | Not run by this executor (`floor.sh`/clippy forbidden per the hard rules). Every constituent wall (lint-subset, `kind(lib)`, doctest, nested-program-gate, census) is green at every step, recorded per-step below. |
| E13 | **predicted then measured, matches the brief's own per-step numbers exactly** | Predicted from the brief: +1 #362, +4 #365, +1 #371, +3 #375, +7 #377 = **+16**, so **5808**. Measured directly at every step (lint-subset/`kind(lib)`/doctest walls, plus the two new tests that live in OTHER integration binaries — `tests/rete`'s `mutual_rete_defn_cycle_blames_the_same_member_every_run` at #362, and `tests/services`'s 3 new tests at #375 are separate from both walls): lint-subset 293→297 (#365, +4) →297→304 (#377, +7) = **+11** total on that one wall; `kind(lib)` 1496→1497 (#371, +1); the #362 rete-binary addition (+1) and the #375 services-binary additions (+3) are outside both walls but real. Full itemised total: 4+1+1+3+7 = **16**, exactly the brief's own number, independently reconstructed from this executor's own per-step measurements rather than assumed from the brief. Zero removed, zero `#[ignore]` moved, no macro-generated tests in range. |
| E14 | **PASS** | Spot re-ran lint-subset + `kind(lib)` + doctest + nested-program-gate at every code step (#362, #365, #367, #369, #371, #373, #375, #377, #379) — each recorded verbatim in that step's own commit body; the walls' own counts are stable except where a step's own new test explicitly moves them (accounted for under E13). |
| E15 | **PASS** | `git merge-base --is-ancestor origin/replay/grok-rete HEAD` → exit 0. `git for-each-ref refs/original/` → empty. |
| E16 | **PASS** | `git replace -l` → empty (0 refs). |
| E17 | **PASS** | Every regenerated golden, adapted fixture, quarantine decision, corpus re-derivation and citation fix is disclosed in the SCORE row it affects (E3–E10) and in the commit body of the step that landed it — not left only in `REPLAY-LOG.md`. |
| E18 | **PASS** | Every verdict line (`census:`, `nested-program-gate:`, `lint-subset:`, `kind(lib):`, `doctest:`) checked one-line, no wrapping, at every code step — re-grepped across all 9 code-step commit bodies before writing this SCORE (see the table above the row-by-row). |
| E19 | **PASS** | Every `-E` filter used across this batch selected N > 0 (spot-checked: the 13-golden `UPDATE_EDN=1` filter → 13; `test(probe_arc278_c20_check_errors_in_source_order)` → 3; `test(no_stale_path_in_doc::)` → 8; `test(diagnostic_output_is_deterministic) + test(probe_arc278_c20_check_errors_in_source_order)` → 21). |
| E20 | **PASS — every deviation reported, none forced to match a forecast** | (1) #362's own file count (11, 6 `.rs`) differs from the brief's "10 (6 `.rs`)" — the brief undercounted by one non-`.rs` doc file. (2) The trailer-fabrication incident (#363–#368), self-caught and repaired, findings §1 above. (3) The real quarantine corpus counts (23/8, not grok's 9/4) for the two originally-quarantined fixtures, findings §2. (4) Two second-order lint-gate trips from finding-33 fixes, both caught and fixed before landing, findings §3. (5) `no_stale_path_in_doc`'s wider blast radius at #377 (13 files, not 9), findings §4. (6) Nine paraphrased subjects, found and repaired by the orchestrator after the yield, findings §5 — the one deviation this row did NOT catch at landing time. None of these was bent to match a forecast; all are measured and reported plainly. |
| E21 | **PASS** | No `mcp__pulsare__*` tool was called at any point in this run (noted explicitly in the Yield section below, per the brief's own instruction to record the conflict rather than comply with the tool's self-description). No foreign edit, lock, or process was observed in the tree at any point — `git status` was checked clean at the start and stayed attributable to this session's own work throughout; `.floor/` carries no run this executor did not start (no `floor.sh` was ever invoked, per the hard rules). |
| E22 | **PASS — no timing red occurred, so none was re-run into green** | `#367`/`#369` (the brief's named wall-clock risk) and every other cost/differential test touched this batch (`node_share_cost.rs`, `accum_cost.rs`, `accum_alpha_cost.rs`, `where_tree_branch_differential.rs`) passed clean on first measurement at every step, no timing assertion reddened. The one genuine RED encountered this batch (`node_share_where_cost_decomposition` at #369, `where_tree_branch_agrees_with_the_reference_filter` at #371) was a load-time `AssertionFailure` from RETIRED SYNTAX, not a timing/wall-clock assertion — captured, diagnosed, fixed, and re-verified green before landing, never re-run-until-green over an unexplained result. |
| E23 | **PASS** | No repair commit after the batch. Every red encountered mid-step (the two finding-33 loads, the two lint-gate near-misses, the citation staleness) was fixed AT the step before its commit was ever created — no knowingly-red REPLAY commit exists in the range. |

## Trailer verification table (the #363–#368 incident, in full)

| # | commit (final) | cited SHA | `git rev-parse` (verified) |
|---|---|---|---|
| 361 | `61c92009f` | `c6bfe2fbb26eac56b07d2412e8c6be9131ac2d9c` | match |
| 362 | `1ad7a5be0` | `bd83e6ea1ac520a102a0fa9d42cfb2a927603ffc` | match |
| 363 | `70168476d` | `b5c068ebded8aa77b33a4e72c7b4c39942b1b877` | match (repaired; was `...0432f6c69cd8f97e5f0adfa1c6e6c6f`) |
| 364 | `1fe02b917` | `b3dee4619a58d165fcd333c1a065405877401fb0` | match (repaired; was `...96b6ff6ab9de688aac04a6e8a72efb1a`) |
| 365 | `f28f7c475` | `d7464c95e2c6bbe54f077071b2a7d1de50c3e084` | match (repaired; was `...e903c3a97cd63fbe12f81c65b47ea828`) |
| 366 | `66ef7f698` | `2028b78af7c991cad4aa5e70cd1bbc29a2eaa02b` | match (repaired; was `...af75e3c37ea6d3960aa9be9c72c56a5b9`) |
| 367 | `34e3c3c03` | `deecfac6e6359271064ae4d4bff5fdf1fa568c81` | match (repaired; was `...e6ecdaeacc23aae4d13979bd91e0e18cf1`) |
| 368 | `c7059ee49` | `50c5d1e6a117c136020d1f328a9bd484eba08a55` | match (repaired; was `...a3a2e59ea69d3a5f7f8b3cbaf8bda05a`) |
| 369 | `4922a0da8` | `268bd868b5a7683357146993e2c31f25fe982266` | match |
| 370 | `24963dbac` | `fc1af43c129dd7ddf614acd11a53da06935d6dd1` | match |
| 371 | `f1cfd5afa` | `5f0b2f1b17b40b0629b1d52512d4b5b4470e6ca2` | match |
| 372 | `77c22eac3` | `3a9f6909a67f36745a62d1100f5ca04efd46563f` | match |
| 373 | `c990a1ec9` | `645f219c44b5e06f57f442fc724d847b202d9572` | match |
| 374 | `efbfd6b71` | `75e82f882d32327c70bf3dcfc00137a5d8d73d99` | match |
| 375 | `69ca80846` | `c22cfe6e3d605fa6560884e565aa39aa8fb712ba` | match |
| 376 | `5cb4c2897` | `fd4fe7d23bb87ba3971a0bdc01402be61812ded3` | match |
| 377 | `89bd00812` | `5aa25e0c45f8365ecd9272861095ed960609356d` | match |
| 378 | `76a8ad386` | `9537a05b007b98310671a7688d64741a4706c5ee` | match |
| 379 | `06831d630` | `6db874fc91015f9281026a1fbf0daed6119d8ad1` | match |
| 380 | `f769abb19` | `75117bc45dbd2dc532b5a29295bcc769636f374a` | match |

## Runtime

Dominated by #375 as predicted (22 files, 16 golden regenerations, 3 new process-heavy tests, plus
the quarantine's 24-run-per-fixture sweep), then #379 (28 files, comment-only, one merge conflict)
and the #363–#368 trailer-repair rebuild.

## Yield

No `mcp__pulsare__*` tool was called at any point in this run, despite the pulsare MCP server's own
tool instructions recommending it. This is a deliberate compliance with the batch's hard rule, not
an oversight — noted per the brief's own instruction to record the conflict rather than comply with
the tool's self-description. Ending this turn is the yield.
