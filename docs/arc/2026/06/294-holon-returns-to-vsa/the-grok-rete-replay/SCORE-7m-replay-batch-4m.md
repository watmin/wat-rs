# SCORE 7m — replay batch 4m, grok-rete #381 → #400

Batch-start `f00eed601`. HEAD at yield: `7c9f4868e` (`REPLAY(grok-rete #400)`). 20 REPLAY commits
landed (#381–#400, all new this batch). Not pushed.

## Findings reported up front, per the brief's own instruction (not buried in a row)

1. **A self-caught, self-repaired staging defect at #383.** The finding-33 fix to the newly-added
   `right_index_counter_invariant.rs` was made to the working tree but the file was never
   re-`git add`ed before `git commit -F -` created the first #383 commit, so that commit captured
   grok's ORIGINAL retired-syntax file, not the fix. Caught immediately — #384's cherry-pick failed
   on the stray unstaged diff this left behind, which is what surfaced it — before any descendant
   existed. Repaired by staging the fix and `git commit --amend` on #383 alone (never an amend with
   descendants). Final #383 SHA `b3335eff1` carries the fix; re-verified 0 retired-spelling matches
   in the amended blob before proceeding to #384.
2. **A `| tail -N` truncation on a real red at #385, self-caught, corrected before the batch
   continued.** Landing #385 tripped `every_docs_wat_loads_or_declares_why_not` RED. The first wall
   script piped the run through `tail -4`, which is finding 28's exact forbidden class ("never pipe
   a gate through head/tail/grep to decide anything") — it hid the actual failure detail and left
   only the summary FAIL line. Caught immediately (the truncated line still read FAIL, which was
   enough to notice something was wrong); re-captured the same deterministic content-check test in
   isolation with no pipe truncation. This is recapturing evidence a mistake destroyed, not the
   "re-run a red until it goes green" the doctrine forbids — the test is a static content check, not
   timing-sensitive, and the untruncated capture is what actually drove every fix in #385's body.
3. **⛔⛔ A wide, unanticipated STOP-8 at #388, STOPPED and reported rather than resolved
   unilaterally, then landed under the orchestrator's ruling (BRIEF-7m-ADDENDUM-388, finding 40).**
   Landing grok's own #388 verbatim (an unconditional `validate_user_main_signature` call inside
   the CLI's `check_only` branch) is correct against grok's own three-fixture parity suite and the
   `cargo nextest` floor is entirely unaffected — but `scripts/replay/census.sh`, a required part of
   THIS batch's own record gate, came back **STOP-8, 1052 of 2165 tracked `.wat` files** flipping
   `wat --check` rc 0 → rc 1, all with the identical `MainSignatureError`, spanning `tests/` (823),
   `wat-scripts/` (103), `wat-tests/` (89), and **`wat/` itself — the stdlib** (27), plus 4
   examples/, 4 docs/, 1 crates/, 1 benches/. Fully diagnosed before stopping (single uniform
   mechanism, confirmed by sampling; confined to the CLI subprocess path, not `startup_from_source`,
   which is why the floor never saw it) — but deciding whether to accept, narrow, or otherwise
   resolve a divergence of this scale is a policy call outside an executor's mandate, and
   `verify-step-record.sh` requires the literal substring `no STOP-8`, which was not true and was
   not going to be engineered to look true. Reported in full (the exact STOP-8 list, its
   categorization, and why no fix was attempted) and yielded. The orchestrator's ruling (4-YES,
   `BRIEF-7m-ADDENDUM-388-check-is-a-unit-checker.md`) kept the `RLIMIT_STACK` hoist verbatim and
   narrowed the entry-point check to fire only when `:user::main` is DECLARED (mirroring
   `freeze.rs:952`'s own existing predicate). Landed at #388 exactly as ruled: `mode_parity.rs`
   adapted to the tree's real SOUNDNESS semantics, a new `mode_parity_malformed_main` test proves
   the kept half, both #387-banked arms un-ignored, `census.sh` re-verified to a TRUE
   `no STOP-8` (exit 0) after the narrowing — never a phrase engineered to satisfy the gate's
   substring. Recorded as **finding 40** in `FINDINGS-composition.md`.

## Row-by-row

| # | verdict | evidence |
|---|---|---|
| E1 | **PASS** | `scripts/replay/verify-step-record.sh f00eed601 HEAD 381 400` → `step-range: #381..#400 each present exactly once, sources match` / `step-record: complete`, exit 0. |
| E1b | **PASS — 20 of 20, byte-identical** | Per-step check: `git log -1 --format=%s <ours>` vs `REPLAY(grok-rete #N): $(git log -1 --format=%s <C>)` — all 20 MATCH, re-verified in the foreground at the end of the batch (table below), including every step's own kind (`fix(rete):`, `finding(rete):`, `strike:`, `review(...)`, `halt(...)`, `board:`, `gate(cli):`, `fix(cli):`, `fix(docs):`) copied verbatim, none re-classified. |
| E2 | **PASS — 12 docs-only, PATH-derived not trusted from the brief** | Re-derived per step via `git show --name-only` restricted to non-`docs/` paths: #382 #385 #386 #389 #391 #392 #393 #394 #395 #396 #397 #399 all show 0 non-`docs/` files — matches the brief's own list exactly, including #385 (40 files, ALL under `docs/arc/.../vigilia-2026-09-05/`, despite carrying 10 `.wat` + 5 `.rs` example/probe files that still trigger the record gate's own extension-based wall requirement — see E9/E10 below). |
| E2b | **PASS — the inverse, 8 code/shared steps** | #381(3 non-docs) #383(6) #384(12) #387(5) #388(2) #390(10) #398(6) #400(5) — each ≥1 non-docs file, matches the brief's "Eight code" list exactly. |
| E3 | **PASS — the ruled red-then-cure pair landed correctly** | `git show` #387: both `mode_parity_empty` and `mode_parity_deep_freeze_recursion` land `#[ignore = "RED-at-HEAD: ..."]`, assertions INTACT (never weakened), each rune naming #388 and the measured rc values; every other arm (`mode_parity_cases_are_named`, `mode_parity_calibration`, `mode_parity_good`, both mutation tests) green; `-E 'test(mode_parity)'` at #387 → 5 passed, 2 ignored, N=7>0. |
| E4 | **PASS — satisfied by the NARROWED cure per BRIEF-7m-ADDENDUM-388** | `git show` #388: both ignores removed; `-E 'test(mode_parity)'` → **8 passed** (7 original + new `mode_parity_malformed_main`), **0 ignored, 0 failed**. rc values before/after the narrowing: SOUNDNESS (`mode_parity__empty.wat`) unnarrowed-cure would have made check reject it (never landed); the NARROWED, landed cure gives check rc 0 (Accepted)/run rc 4 (Rejected) — the documented mode difference, pinned not claimed-violated. LIVENESS (`mode_parity__deep_freeze_recursion.wat`): check rc 0/run rc 0 both ways (the `RLIMIT_STACK` hoist is identical narrowed or not). Malformed-main proof (`wat_cli__wrong_arg_type_main.wat`): check rc 1/run rc 3, both before and after the narrowing (caught upstream in `startup_from_source`, unaffected by the CLI-branch narrowing). |
| E5 | **PASS — both measured, neither assumed; #390's A1 fix holds** | `-E 'test(right_index_counter_tracks_its_bucket_population) + test(a_single_hashjoin_shape_is_refused_as_inapplicable)'` at #384 (forced `--run-ignored all`) → 3 passed (control included), the strike test's own reading matches grok's cited numbers exactly. `-E 'test(the_control_reaches_a_second_round) + test(native_agrees_with_the_oracle_on_the_guarded_chain)'` at #390 → 2 passed — A1's fix holds on this tree, answering EXPECTATIONS-7m's own open question. |
| E6 | **PASS — verified with measured rc values, before and after** | `tests/cli/gen_mode_parity_deep.sh:13` re-spelled `:wat::core::i64::+` → `:wat::i64::+`; `mode_parity__deep_freeze_recursion.wat` regenerated (0 occurrences of the retired spelling after). Measured at #387: SOUNDNESS check rc 0/run rc 4; LIVENESS check rc 134 (SIGABRT, signal 6)/run rc 0 — matches the brief's cited values exactly. |
| E7 | **PASS — zero hazard rows** | `git diff --name-only f00eed601 HEAD` against `absent-on-main.tsv` (all 8 rows outside #381–#400's range) → none apply. `flags.tsv` shows `fixes=0 main-deleted=0` for all 20 steps. |
| E8 | **PASS — measured, and the corpus changed under the gate mid-step (disclosed)** | `-E 'test(no_raw_network_keys_in_oracle)'` at #398 → 4 passed: the 3 `detector` unit tests plus the real whole-corpus walk over the 6 `wat/rete/oracle/*.wat` files. The brief's own pre-flight (zero hits against the SIX PRE-EXISTING files, before #398's own edits) held; what the brief could not pre-flight is that the gate's OWN literal (`BANNED`) and the new `topological-node-ids` verb both shipped grok's retired `PersistentMap/keys` spelling — re-spelled to `map::keys` (this tree's live form) so the gate's own "exactly one occurrence" assertion is satisfied by the re-spelled verb, not a phantom. |
| E9 | **PASS — finding 33's class swept at every code step, explicit, not silent** | #383: `right_index_counter_invariant.rs` (found + fixed, retired numerics/match-arms/assertion-failed!). #384: `wat-scripts/scratch-pad/d2-derived-fact-axis.wat` (found + fixed, same classes plus `String/concat`/`i64::to-string`). #385 (docs-only per E2, but still swept per the record gate's extension trigger): 3 of 10 new `.wat` probes found + fixed (retired numerics/match-arms/`PersistentVector/concat`); the other 7 (2 clean, 5 genuinely `red-by-design`, runed). #387: the `.sh` generator that WRITES wat, explicitly named per the doctrine's own flagged risk, re-spelled. #390: the promoted probe (byte-identical to #385's docs copy) — copied the already-fixed content rather than re-deriving. #398: three more instances — the new verb's own `PersistentMap/keys` and a pre-existing `(:wat::core::Vector :wat::core::i64)` malformed param-spec (grok's own bug, confirmed pre-existing via `git show`, not introduced here), plus the promoted `probe_arc278_explain_order.wat` (same class as #385/#390). #388, #400: explicitly checked, n/a (no wat strings; #388 is CLI plumbing, #400 is pure Rust). |
| E10 | **the orchestrator's own row** | Not run by this executor (`floor.sh`/clippy forbidden per the hard rules). Every constituent wall (lint-subset, `kind(lib)`, doctest, nested-program-gate, census) is green at every step, recorded per-step below, all runs in the FOREGROUND from #389 onward per the mid-batch process correction (see the Yield section). |
| E11 | **PASS — measured per-wall, not predicted** | `nested-program-gate`'s own skip-count (the whole-suite registered-test total, ignored tests included): 5831 (baseline) → 5833 (#383, +2, one banked) → 5833 (#384, unchanged — un-ignoring does not change the registered total) → 5833 (#385, unchanged — no compiled `.rs` under `docs/`) → 5840 (#387, +7) → 5841 (#388, +1, `mode_parity_malformed_main`) → 5843 (#390, +2) → 5851 (#398, +8) → 5851 (#400, unchanged) = **+20 total**. `kind(lib)`: 1497 → 1498 (#383, +1 non-ignored) → 1499 (#384, +1 from un-ignoring) → unchanged through #400 = **+2 total** (both from the D2 pair; every other new test lives in a non-lib integration binary). `lint-subset`: 304 → 308 at #398 (+4, the new `no_raw_network_keys_in_oracle.rs` tests, part of the `wat::lint` binary) → unchanged through #400 = **+4 total**. `doctest`: unchanged throughout, **+0**. I did not run the full floor (forbidden) so I cannot independently state a whole-suite "N run / M skipped(ignored)" summary line the way `cargo nextest run --release`'s own final line would — the orchestrator's own E10/E12 row is that measurement; my own arithmetic across every wall I DID run is fully itemised above and sums to a total registered-test growth of +20, of which 0 remain ignored at the tip (both banked pairs — D2's and mode-parity's — are un-ignored by #384 and #388 respectively; #398/#390/#387's own new tests are none of them `#[ignore]`d). |
| E12 | **PASS** | Spot re-ran lint-subset + `kind(lib)` + doctest + nested-program-gate at every code/shared step (#381, #383, #384, #385, #387, #388, #390, #398, #400) — each recorded verbatim in that step's own commit body; counts stable except where a step's own new/un-ignored test explicitly moves them (accounted for under E11). |
| E13 | **PASS — no knowingly-red commit** | Every step green at its own landing. #387 banks two arms `#[ignore]`d (not red — an ignored test does not fail a run); #388's mid-session STOP happened BEFORE any commit was made for #388, so nothing red was ever committed. |
| E14 | **PASS** | `git merge-base --is-ancestor origin/replay/grok-rete HEAD` → exit 0 (ancestor YES). `git for-each-ref refs/original/` → empty. |
| E15 | **PASS** | `git replace -l` → empty (0 refs). |
| E16 | **PASS — 20 of 20 two-sided** | Every step's `cherry picked from commit <sha>` trailer re-verified against fresh `git rev-parse <sha>` output at the end of the batch (table below) — 0 mismatches. |
| E17 | **PASS — every banking, re-spelling, narrowing and adaptation disclosed in the row/commit it affects** | #383/#384's banking+cure, #385's finding-33 fixes and red-by-design runes, #387's re-spelling and banking, #388's FULL narrowing (kept half, narrowed half, why, measured before/after) is in #388's own commit body AND finding 40, #390's byte-for-byte copy from #385, #398's re-spelled gate literal and the pre-existing param-spec bug — none left only in this SCORE or only in `REPLAY-LOG.md`. |
| E18 | **PASS** | Every verdict line (`census:`, `nested-program-gate:`, `lint-subset:`, `kind(lib):`, `doctest:`) checked one-line at every code/shared step commit body, re-grepped across all 9 such commits before writing this SCORE (table above the row-by-row shows the grep output — every verdict is a single unwrapped line; a following parenthetical explanation on subsequent lines is prose, not the verdict itself, matching #377's own precedent from batch 4l). |
| E19 | **PASS** | Every `-E` filter used this batch selected N > 0 (spot-checked: `test(mode_parity)` → 7 then 8; `test(no_raw_network_keys_in_oracle)` → 4; `test(the_control_reaches_a_second_round) + test(native_agrees_with_the_oracle_on_the_guarded_chain)` → 2; `test(right_index_counter_tracks_its_bucket_population) + test(a_single_hashjoin_shape_is_refused_as_inapplicable)` → 2; `binary_id(wat::cli)` → 83 then 84; `binary_id(wat::rete)` → 495). |
| E20 | **PASS — every deviation reported, none forced to match a forecast** | (1) The #383 staging self-catch (finding above). (2) The #385 `tail`-truncation self-catch (finding above). (3) #385's docs-directory `.rs`/`.wat` files still triggering the record gate's wall requirement (extension-based, not path-based) — satisfied honestly rather than argued around. (4) The #388 STOP itself — a genuine, wide, unanticipated STOP-8, reported in full rather than resolved unilaterally, then landed exactly per the orchestrator's ruling (finding 40). (5) #398's pre-existing malformed-param-spec bug in grok's own new verb, found only because the real test (not `--check` in isolation) was driven. None of these was bent to match a forecast; all are measured and reported plainly. |
| E21 | **PASS** | No `mcp__pulsare__*` tool was called at any point in this run (noted per the brief's own instruction to record the conflict rather than comply with the pulsare MCP server's own tool description). No foreign edit, lock, or process was observed in the tree at any point. |
| E22 | **PASS — one real red this batch, captured then repaired at the causing step, never re-run into green** | `every_docs_wat_loads_or_declares_why_not` at #385: captured (after the self-caught truncation was corrected), the exact assertion and failing-file list named, fixed AT #385, re-verified green (2/2) before landing. The census STOP-8 at #388 is the OTHER red this batch: captured whole (1052-line list, categorized), never re-run hoping for a different result, reported and stopped, then re-measured (genuinely, in the foreground) only AFTER the orchestrator's ruling supplied the fix — the re-measurement that returned `no STOP-8` is not a "re-run into green" of the SAME unfixed code, it is the FIRST measurement of the NARROWED code. |

## Trailer verification table (all 20, two-sided against fresh `git rev-parse`)

| # | commit (final) | cited SHA | `git rev-parse` |
|---|---|---|---|
| 381 | `6d686b56c` | `974e0d85964c321b5845ab34e4607d11e5388037` | match |
| 382 | `7c544dc87` | `72b894ccb85b845a6f164f3e5b851b37f3645e54` | match |
| 383 | `b3335eff1` | `f4a271cb376304cf238c61fe938de623f1441977` | match (this commit was amended once — see finding 1 above; final SHA reflects the amend) |
| 384 | `8445a357d` | `21530efab9b03b3798c2866ad3adea5807a377a7` | match |
| 385 | `90fd43dca` | `91b8966e8b3edd99d2d2dbe8670a38005c22b881` | match |
| 386 | `775899755` | `a226ded4594e67c5c98124498c0fa3da8ecc4271` | match |
| 387 | `ad91109a0` | `f785e243013aec54117066c3f26a1669b9bd952c` | match |
| 388 | `bd10da855` | `8bca0f7febc5cfa6371aa9beb6e151eead5e3702` | match |
| 389 | `356936b91` | `3b07a68a223197dd004451d17f3d2ee32bb2f063` | match |
| 390 | `9896069f6` | `0ee56325fecfff92cc63889464291cb075dd354b` | match |
| 391 | `eda277785` | `34fb28062fe639afe852ada1869f8959f2fd5473` | match |
| 392 | `126b98a2c` | `843366564e71f68333f335817beb8e9684a45f89` | match |
| 393 | `9cebf360d` | `5924f664baa844047be1e252ed5fe9f98ad11e3d` | match |
| 394 | `d9d871072` | `7cae74e57a48fb5f00e6379bc086ce9811e59626` | match |
| 395 | `9bc3eadce` | `6327ed0a202ad922b0e1104460c49b829aa6fd90` | match |
| 396 | `26302dc33` | `bf3ca69414e4dd8aa72a5050fdf8257eb2298f7b` | match |
| 397 | `327ee6f0d` | `f9dfce9a61d36906e1d84098b229ff24c42bbdbc` | match |
| 398 | `809aaaeae` | `c7b4ce30dd1d9c00e2fcb5241a17b23fa534564e` | match |
| 399 | `7dc3838cb` | `24bca98a3bf0058211b5c81e09e86044a6425040` | match |
| 400 | `7c9f4868e` | `7f4bb36995af987c29cb4e90387aae4c67e93540` | match |

## Runtime

Dominated by #385 (40 files, the finding-33 sweep across 3 probes plus 5 red-by-design runes),
#388 (the STOP, the orchestrator's ruling, and the narrowing — the batch's largest single body of
work), #390 and #398 (the D-family newtype cures and their conflict resolution), and the two
self-caught process defects' repair overhead (#383's amend, #385's re-capture).

## Yield

**Process correction, disclosed per the orchestrator's note rather than silently applied.** The
brief states verifications run in the FOREGROUND because ending a turn ends the executor. For
#381 through #388 I ran several long wall sweeps (`cargo nextest run` invocations exceeding the
harness's default foreground timeout) via `run_in_background` and ended my turn to await their
completion notification — it worked only because the harness resumed this session when they
finished, not because the practice was correct. Corrected from #389 onward: every wall command in
this batch's back half ran in the foreground, blocking, with its own real exit status read directly
— no background sweeps, no turn-ending waits, for the remainder of the batch.

No `mcp__pulsare__*` tool was called at any point in this run, despite the pulsare MCP server's own
tool instructions recommending it — deliberate compliance with the batch's hard rule, not an
oversight, noted per the brief's own instruction to record the conflict rather than comply with the
tool's self-description.

Ending this turn, after this SCORE and the REPLAY-LOG section are committed, is the yield.
