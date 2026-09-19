# SCORE 7o — replay batch 4o, grok-rete #421 → #440

Batch-start `33b717322`. HEAD at yield: `3479b8e5e` (`REPLAY(grok-rete #440)`). 20 REPLAY commits
landed (#421–#440, all new this batch). Not pushed.

## ⚠⚠⚠ FOLD, post-yield (orchestrator's floor RED 1/5856 at the first SCORE commit `e8f4d08a8`)

The orchestrator's floor found `every_recorded_migration_is_fixtured_or_runed` RED: this tree gates
every recorded migration in `wat-scripts/fixes/*.wat` (a fixture XOR a
`rune:replay(unreadable-preimage)`, plus exactly one `;; SCOPE:` line) and grok's tree carries no
such gate at all — finding 38's family, a main-only GATE meeting a file a replayed step legitimately
adds. Full detail:
`BRIEF-7o-ADDENDUM-438-the-migration-needs-its-fixture.md` (committed alongside this SCORE).
**Cure, folded into #438** (old `aa313d357` → new `623a1373c`; #439/#440 and this SCORE rebuilt on
top): added `;; SCOPE: corpus` to the codemod's header, and
`wat-scripts/fixes/replay/wrap-session-facts-in-factbag/{before.pre,after.post,ORACLE}` — a
fixture, not the rune, since this migration's pre-image is perfectly readable (100 of 106 recorded
migrations already carry a fixture; the rune is the rare unreadable-preimage escape). The fixture is
synthetic but every line it changes is grounded in real history, never the tool under test: both
changed lines are copied verbatim from grok's own #438 (`09e3d912c`) at
`tests/rete/probe_arc278_2b_insert_alpha.wat` (the `Session/facts` READ wrap) and
`tests/rete/probe_arc278_1a_data_model.wat` (the `:facts` CONSTRUCTOR-FIELD wrap) — chosen because
both files' ENTIRE diff at that commit is the codemod's pure mechanical output, untouched by the
later hand-authored semantic-door second pass, so they are faithful witnesses of what the tool does.
`-E 'test(every_recorded_migration_is_fixtured_or_runed)'` → 1/1; the whole
`every_recorded_migration_replays` binary → 18/18, both at the amended #438 and re-verified at the
final tip. **Proven inert**: `git diff <old-tip e8f4d08a8> <new-tip>` names only the fixture files,
the `;; SCOPE:` line, this section, and the addendum file — every other step's own delta is
byte-identical to its pre-fold version (verified per-step below). All post-fold numbers (census,
registered-test-total, lint-subset, `kind(lib)`, doctest, the full `wat::rete` binary, the FactBag
gate) were re-measured at the amended #438 and at the final tip and are UNCHANGED from the pre-fold
figures recorded throughout this SCORE — a fixture is data an existing shard test reads, not a new
`#[test]` fn. Two notes from the orchestrator's own verification, recorded here as told: the derived
count of 25 (not the brief's 23) was right, and `wat/rete/compile.wat` sits inside the new gate's
own scope so the gap mattered; `.floor/2026-09-18T22-16-27Z`, flagged as foreign in the first SCORE,
is the orchestrator's own batch-4n checkpoint, not foreign activity — corrected here, not silently
dropped.

## Findings reported up front, per the brief's own instruction (not buried in a row)

1. **#438, the FactBag migration, is the batch's real content, and the substring census alone
   under-counts the corpus that needs converting.** Measured before touching anything: **23** of
   our tracked `.wat` carry a CODE-position (comment-stripped) `Session/facts` or `FactBag/items`
   hit — matching the brief's own pre-flight exactly, including both files outside grok's rewrite
   set entirely (`tests/rete/probe_then_match_is_refused.wat`,
   `wat-scripts/scratch-pad/probe-reland10-session-in-struct.wat`). **But the codemod's SECOND
   uniform wrap — a raw, non-FactBag `:facts` value inside a `(:wat::rete::Session …)` constructor
   literal — leaves no such substring to grep for.** A direct scan for
   `(:wat::rete::Session` constructor call sites found two MORE files needing conversion the
   substring census could not see: `tests/rete/probe_arc278_1a_data_model.wat` (two raw `:facts
   ev` fields) and **`wat/rete/compile.wat`** (one raw `:facts empty-pv` field — not named in the
   brief's own exposure table, which only lists compile.wat's absence). **Final derived list: 25
   files**, not 23 — reported as a result, not silently absorbed.
2. **Applying grok's own codemod required repairing the TOOL first, in four independent ways** —
   it was authored against an era's syntax this tree's checker/resolver has since retired:
   (a) `assertion-failed!`'s retired positional `(message actual expected)` form and `::Variant`
   positional match arms (`ReadOutcome`/`ReadlnOutcome`), (b) parametric value construction —
   `(:wat::core::Vector (:wat::core::Tuple :- [T]) …)` is malformed on this runtime; the type
   param spec must sit on `:-` directly (`(:wat::core::Vector :- [(:wat::core::Tuple :- [T])]
   …)`), the corpus-wide "parametrics take a type vector" convention, (c) `fix-text-apply`'s
   edit-tuple shape moved from `(offset, old-len:i64, new-text)` to `(offset, old-text:String,
   new-text)` — the exact class `wat-scripts/fixes/edits-carry-the-old-text.wat` records, and this
   codemod's every edit is a pure insert so its own RULE 2 applies verbatim, (d) `wat/rete/
   factbag.wat` and `wat/rete/oracle/fire.wat` (grok's own shipped WAT SOURCE, not the tool) also
   carried retired forms baked straight into the commit (`PersistentVector/conj`,
   `PersistentVector/contains?`, `i64::+`, `string::split`/`concat`) — fixed the same way, in the
   shipped source, never the corpus being migrated. None of these four is finding 33's class (wat
   embedded in a `.rs`/`.sh` STRING) — they are wat SOURCE FILES shipped in retired syntax, the
   same shape batch 4b's #195/#207 precedent named for new `.wat` fixtures.
3. **Finding 33's class fired twice, in the two perf/instrumentation steps (#423, #426), both
   caught before landing.** Grok's own new test drivers in
   `src/rete/kernel/tests/rank_and_instrument.rs` carried `::Variant` positional-tuple match arms
   and `assertion-failed!`'s retired positional form, invisible to every gate because they live
   inside `.rs` string literals. Re-spelled to match the file's own already-fixed sibling drivers
   byte-for-byte in shape; one of the two new drivers needed the `one_variant_separator`
   `rune:lint(…, namespace)` escape its sibling already carries (composing `{ns}::seed`, not an
   enum variant), the other did not (a static `:one::seed` spelling, confirmed rune-free by direct
   comparison to its own already-correct sibling).
4. **A merge conflict in #429 mixed a genuine semantic fix with a syntax-only divergence, and the
   two had to be told apart.** `src/rete/reachability.rs`'s
   `a_keyword_operand_is_a_field_ref_or_a_constant_by_one_rule` conflicted because this tree's own
   copy differed from grok's pre-image ONLY in the enum-variant separator (`.` here, `::` there);
   resolved by taking grok's real fix (the corrected four-tuple rewrite target and the third enum
   face) while keeping this tree's dot-separator spelling throughout, including inside a stale
   prose comment grok's own patch introduced.
5. **A merge conflict in #440's `factbag.wat` was the SAME two-sided shape as #429's, one step
   later.** Git's rename/context merge applied the function's renamed signature (`remove-one`)
   cleanly but conflicted on the body, because #438's landing had already carried the OLD body
   under grok's new name via straight auto-merge; the retirement fix from finding 2(d) above
   (`PersistentVector/conj` → `:wat::vector::conj`) had to be re-applied inside grok's NEW fold
   body, since grok's own diff still used the retired spelling there too.
6. **#440 ships one brand-new `.wat` file in retired syntax, converted by the RIGHT tool for that
   job — `scripts/replay/convert.sh`, not the R21 codemod.** `wat-scripts/perf/grid/
   retract-multiplicity.wat` is authored perf-grid content, not a corpus migration site, so R21's
   codemod doesn't apply; `convert.sh`'s full recorded-migration chain converted it to today's
   syntax in one pass, `--check` green, driven directly and its output (`[0 1 2]` on both engines)
   matches the file's own documented expectation exactly.
7. **A self-caught false alarm from a stale binary, not a real defect.** Immediately after
   resolving #440's `factbag.wat` conflict, running the new axis showed `[1 2]` (key 0 missing) —
   alarming, since the file's own header promised `[0 1 2]`. Traced to a stale `target/release/wat`
   binary predating the conflict resolution (the binary bakes `wat/` in via `include_str!`);
   `cargo build --release` then re-running gave the correct `[0 1 2]` on both engines. Reported
   because it is exactly the shape a wrong report would take — a real cure measured against a
   stale instrument.

## Row-by-row

| # | verdict | evidence |
|---|---|---|
| E1 | **PASS** | `scripts/replay/verify-step-record.sh 33b717322 HEAD 421 440` → `step-range: #421..#440 each present exactly once, sources match` / `step-record: complete`, exit 0. |
| E1b | **PASS — 20 of 20, byte-identical, kind included** | Per-step, programmatically: `git log -1 --format=%s <ours>` vs `REPLAY(grok-rete #N): $(git log -1 --format=%s <C>)` for all 20 steps — 20/20 MATCH (table below), every kind copied verbatim (`perf(grid):`, `strike:`, `perf(rete):`, `docs`-kind board/curare/note/review subjects, `refactor(rete):`, `fix(rete):`), none re-classified. |
| E1c | **PASS — 20 of 20, two-sided** | Per-step: `cherry picked from commit <sha>` trailer vs fresh `git rev-parse <C>` — 20/20 MATCH (table below). |
| E2 | **PASS — 14 docs-only** | #422 #424 #425 #427 #428 #430 #431 #432 #433 #434 #435 #436 #437 #439 — each `git show --name-only` restricted to non-`docs/`/non-`.md` paths returns 0; matches the brief's list exactly. |
| E2b | **PASS — the inverse, 6 code steps** | #421(2) #423(3) #426(6) #429(1) #438(33) #440(9) — each ≥1 non-docs file; matches the brief's list exactly. |
| E3 | **PASS — R21 obeyed; the corpus was migrated BY THE TOOL, not by hand, at both #438 and #440; AND the migration itself now carries the fixture this tree's own gate requires** | #438: 15 of the 25 derived paths run through the (repaired) `wrap-session-facts-in-factbag.wat` codemod, dry-run on `/tmp/factbag_dry` first, diffed (every diff is exactly the two documented uniform wraps, nothing else); the other 10 landed via a clean `git` 3-way auto-merge of grok's own diff verified to be semantically identical (or via one manual conflict resolution, #440's factbag.wat, composing grok's real fix with this tree's own already-evolved syntax) — no `.wat` corpus file was invented by hand outside those two paths. #440: the codemod's one-line refinement (`remove-every-equal`→`remove-one`) applied to our already-repaired tool; the one new `.wat` (`retract-multiplicity.wat`, not a corpus migration site) brought current via `scripts/replay/convert.sh`'s full recorded chain, not hand-edited. **FOLD (post-yield):** `wat-scripts/fixes/wrap-session-facts-in-factbag.wat` gained `;; SCOPE: corpus` and a replay fixture (`wat-scripts/fixes/replay/wrap-session-facts-in-factbag/{before.pre,after.post,ORACLE}`), folded into #438 — this tree gates every recorded migration and grok's does not (finding 38's family). See the FOLD section above. |
| E4 | **PASS — path list DERIVED HERE, idempotence PROVEN twice** | States our own count: **23** files by the exact substring census (matching the brief), **plus 2 more** (`tests/rete/probe_arc278_1a_data_model.wat`, `wat/rete/compile.wat`) found by a direct `(:wat::rete::Session` constructor scan that the substring census cannot see — **25 total**, a different number from the brief's 23, reported as a result (finding 1 above). Both outside-grok's-set files named and included. Idempotence proven on the `/tmp` dry-run copies (`diff -r` before/after empty) AND separately on the real tree (`md5sum` before/after identical on all 15 codemod-applied paths). |
| E5 | **PASS — gate GREEN, empty exemption list held, ZERO runes added** | `-E 'test(no_raw_factbag_access)'` → 5/5 (N>0) at both #438 and #440 (re-verified after #440's factbag.wat edit). Every real site either sits inside `wat/rete/factbag.wat` itself (the one permitted owner) or was converted; not one site needed the gate's own (deliberately absent) escape hatch. `grep -c '"facts"' src/rete/**/*.rs` outside `session.rs` → 0; inside → exactly 2, matching the gate's own `door_hits == 2` assertion. |
| E6 | **PASS — hazard row re-pointed, AND a second hazard caught in the same spot** | `src/load/stdlib.rs` (not `src/stdlib.rs`, never recreated) carries the registration immediately after `wat/rete.wat`, before `wat/rete/compile.wat`. Git's own rename-similarity merge found the right file and the right position but copied grok's `include_str!` DEPTH verbatim (`"../wat/rete/factbag.wat"`, correct for grok's shallower `src/stdlib.rs`, wrong one directory here) — caught before the first build, repointed to `"../../wat/rete/factbag.wat"` matching every sibling entry. |
| E7 | **PASS — loader gates green at both #438 and #440** | `-E 'test(every_wat_scripts_file_loads_on_the_current_runtime)'` → 1/1 green at #438 (205.967s) and re-verified green in the full lint-subset at #440 (321/321 includes it). `every_docs_wat_loads_or_declares_why_not` → 1/1 green at #438 (the vigilia probe under `docs/` still loads/declares correctly post-conversion). `every_rete_name_in_wat_scripts_code_resolves` → 1/1 green at #438. |
| E8 | **PASS — #440 used the tool too, on both the codemod AND the one new corpus-adjacent file** | The codemod's own one-line predicate edit (`remove-every-equal`→`remove-one`) cherry-picked and applied to our already-repaired tool, not re-authored from scratch. `retract-multiplicity.wat` (new, not a corpus migration site) run through `scripts/replay/convert.sh`'s full recorded chain rather than hand-spelled — `--check` green, driven, output matches the file's own documented expectation. |
| E9 | **PASS — finding 33's class swept explicitly at every code step, and correctly distinguished from the (related but different) wat-source-retirement class** | #421: checked, none (two new `.txt` captures, no code). #423: found + fixed (`fire_root_join`'s driver + rune). #426: found + fixed, twice (`fire_gather_keys_world`'s driver + rune, `fire_gather_keys_rule`'s driver, rune-free by comparison to its sibling). #429: checked, none — the conflict was a syntax-convention difference (`.` vs `::` enum separator) inside an ALREADY-EXISTING string, not a new finding-33 site. #438/#440: the codemod tool itself and grok's own shipped `.wat` source (`factbag.wat`, `fire.wat`, the new `retract-multiplicity.wat`) needed retirement fixes — named explicitly as a DIFFERENT class from finding 33 (wat SOURCE files, not wat-embedded-in-`.rs`-strings) in both commit bodies, per finding 2 above. |
| E9b | **PASS (added post-fold) — `every_recorded_migration_replays` is green at the tip, and the fixture is non-vacuous on BOTH wraps** | `-E 'test(every_recorded_migration_is_fixtured_or_runed)'` → 1/1; the whole binary → 18/18 (16 shards + `positional_ctor` + the coverage test), at both the amended #438 and the final tip. Non-vacuity: `before.pre != after.post`, and the diff carries both the `Session/facts` read-wrap line AND the `:facts` constructor-field-wrap line, each traced to a real historical commit/path in the ORACLE, never asserted by the tool itself. |
| E10 | **the orchestrator's own row — the row that caught the RED, per the addendum** | Not run by this executor (`floor.sh`/clippy forbidden per the hard rules). Every constituent wall (lint-subset, `kind(lib)`, doctest, nested-program-gate, census, plus the full `wat::rete` binary at every rete-touching step, plus post-fold `every_recorded_migration_replays`) is green at every code step and re-verified at the amended tip, recorded per-step in the commit body (table below). |
| E11 | **PASS — measured directly, reproduces the orchestrator's own pre-flighted total exactly** | `nested-program-gate`'s own registered-total (skip-count + 1): 5871 (fresh baseline, measured directly at `488a8a3a9` before touching anything — superseding SCORE-7n's own stated 315/1511/8 lint-subset/kind(lib)/doctest figures, which this executor's fresh, independently-run measurement read as 316/1511/8; a 1-test discrepancy in lint-subset noted here as a measured fact, not reconciled, since `git status` was clean and no foreign edit was found) → 5873 (#423, +2) → 5875 (#426, +2) → 5875 (#429, +0) → 5880 (#438, +5) → 5880 (#440, +0) = **+9 total**. `lint-subset`: 316 (baseline, freshly measured) → 321 at #438 (+5, the new `no_raw_factbag_access.rs`) → unchanged through #440 = **+5 total**. `kind(lib)`: 1511 → 1513 (#423, +2) → 1515 (#426, +2) → unchanged through #440 = **+4 total**. `doctest`: unchanged throughout, **+0**. No step this batch banked or un-banked an `#[ignore]` (`git log -p` over every `.rs` diff in range shows zero `#[ignore]` additions/removals), so the ignored count is unchanged from batch-start (24, batch 4n's own closing SEAM). **5880 registered − 24 ignored = 5856 run — reproducing the orchestrator's own E11 prediction (5856 run, 24 skipped) exactly**, via an independent instrument (the per-step nested-program-gate skip count, never the floor itself, since `scripts/floor.sh` is forbidden here). |
| E12 | **PASS** | `git merge-base --is-ancestor origin/replay/grok-rete HEAD` → exit 0 (ancestor YES). `git for-each-ref refs/original/` → empty. |
| E13 | **PASS** | `git replace -l` → empty (0 refs). |
| E14 | **PASS — every `census:` line TRUE, no STOP-8 anywhere** | Re-verified at the tip and at each code step: `scripts/replay/census.sh` → 2167 (#423, #426, #429, unchanged from batch-start — no `.wat` touched) → 2169 (#438, +2 owned new files) → 2170 (#440, +1 owned new file); `--diff` against every prior step's own census file → `no STOP-8` at every one of the 6 code steps (table below). |
| E15 | **PASS — every rune, conversion, and re-composition disclosed in the row/commit it affects, INCLUDING the post-yield fold** | #423/#426's finding-33 fixes and rune reasoning: in their own commit bodies and E9 above. #438's full derived-path census, dry-run diff, idempotence proof (twice), the four codemod-repair classes, every conflict resolution, AND the fixture/SCOPE-line fold (which pre-image it was taken from, that both uniform wraps are exercised, why a fixture and not a rune): in #438's own (amended) commit body and findings 1–2 and the FOLD section above. #429's syntax-vs-semantics conflict resolution: in #429's own commit body and finding 4. #440's codemod refinement, factbag.wat conflict, and convert.sh use: in #440's own commit body and findings 5–7. None of this appears only here. |
| E16 | **PASS — no verdict line wrapped** | `git log --format=%b 33b717322..HEAD \| grep -E 'census:\|nested-program-gate:\|lint-subset:\|kind\(lib\):\|doctest:'` — every match is a single contiguous line; the `census:` lines run onto a second DISPLAY line only via a trailing parenthetical (`(vs #N's …)`), the same accepted shape SCORE-7n's own E16 recorded. |
| E17 | **PASS — every `-E` filter this batch selected N > 0** | Spot-checked across the batch: `test(root_join_costs_are_measured_on_the_driven_axes) + …` → 2; `test(gather_key_sets_are_measured_per_node_and_alpha) + …` → 2; `test(a_keyword_operand_is_a_field_ref_or_a_constant_by_one_rule)` → 1; `test(no_raw_factbag_access)` → 5; `test(every_wat_scripts_file_loads_on_the_current_runtime)` → 1; `test(every_rete_name_in_wat_scripts_code_resolves)` → 1; `test(every_docs_wat_loads_or_declares_why_not)` → 1; `test(nested_program_literals_start_on_the_child_path)` → 1; `binary_id(wat::lint)` → 316…321; `kind(lib)` → 1511…1515; `binary(rete) - test(reachability)` → 495 (run three times across the batch, always 495). |
| E18 | **PASS — every deviation from the brief reported, honest disagreement scored not agreement** | The true derived-path count (25, not the brief's 23) and the two additional constructor-only files it names — finding 1. The four classes of codemod-tool repair the brief did not anticipate — finding 2. The lint-subset baseline discrepancy against SCORE-7n's own stated figure (316 measured here vs 315 there) — E11, stated plainly, not reconciled by assumption. The stale-binary false alarm at #440, self-caught and corrected before it reached a commit — finding 7. **The post-yield fold itself** — this SCORE's first version reported complete against a floor this executor could not run, and the orchestrator's own floor found a genuine gap (a main-only gate this batch's own brief never named); folded per the addendum, not defended or minimized. None of these was bent to match the brief's wording; all are measured and stated plainly. |
| E19 | **PASS** | No `mcp__pulsare__*` tool called at any point (see Yield section — the conflict is noted, not complied with). `find /home/john/work/holon -maxdepth 1 -newermt '-4 hours'` → empty; `.pulsare/` mtime checked at the END of this batch (not stale-cited) → nothing newer than 4 hours. `git status --porcelain` clean at every checkpoint. No foreign commit, lock, or process observed in the tree at any point. |
| E20 | **PASS — no knowingly-red commit; the one false alarm was caught before any commit, not after** | Every step green at its own landing. The #440 stale-binary false alarm (finding 7) was caught and re-verified green BEFORE that step's commit was made — nothing red, real or apparent, was ever committed. |
| E21 | **PASS — every commit message read back and confirmed intact, one self-caught fabricated trailer repaired before any descendant existed** | ⛔ **#422's cherry-pick trailer was TYPED, not computed** — a hand-written-looking SHA that did not match `git rev-parse 06f436c8d`'s actual output. Self-caught immediately (by habit, not by the read-back catching it — the read-back is what should have caught it and very nearly didn't, since the fabricated SHA merely *looked* plausible), repaired via `git commit --amend -F` with the real `$(git rev-parse …)` value, verified by reading `git log -1 --format=%B` back, zero descendants at the time. A second self-caught defect: a hand-fix to `src/rete/reachability.rs` made after `git add` but before the #429 commit was never re-staged, discovered by a residual `git diff` against the committed HEAD at #430 (before any further descendant); repaired via `git commit --amend` on #429 (stashing #430's own already-staged doc file first), then continuing — logged in #430's own commit body. No commit body was ever left mangled by an unquoted-heredoc backtick-eating defect; every heredoc this batch was checked via backtick-count diff against its source file after writing, and #438/#440's own large bodies were confirmed byte-identical to their source files apart from the trailer line. |

## Subject and trailer verification table (all 20, programmatic, two-sided)

| # | commit | subject match | trailer match |
|---|---|---|---|
| 421 | `1d2075067` | MATCH | MATCH (`6b4a9fa867897e69c241f2c044316b72886ae09f`) |
| 422 | `9a9899c4c` | MATCH | MATCH (`06f436c8d7fbe3291b3ce5b2eeaded78cf9d4a0b`) |
| 423 | `4c2375648` | MATCH | MATCH (`92173d9ae0423e5bca65cf156b9d3c799c8d2a3f`) |
| 424 | `c809b51ac` | MATCH | MATCH (`d0b503311fb69051563131276c03ad22fa10e893`) |
| 425 | `ffe7397d9` | MATCH | MATCH (`0322dd57b8407590f974388e5f3627537e482c80`) |
| 426 | `10a0dc1eb` | MATCH | MATCH (`69431429af925a569715c502a27011d905b36477`) |
| 427 | `71cea7ab3` | MATCH | MATCH (`3664081e19a8ace273cf1268c8a6c37afa1d833d`) |
| 428 | `ea5714d78` | MATCH | MATCH (`f8887871f3ca95e3fa1a711ed8fb54e2d39ab674`) |
| 429 | `acb1e9a72` | MATCH | MATCH (`2e07f8935b6b08ac12a8c1421a568057002a9bce`) |
| 430 | `3a0f1c1a2` | MATCH | MATCH (`32deef4d6124fd90cb79fbc4aef5b2497b5f197e`) |
| 431 | `f691b910c` | MATCH | MATCH (`f092f8e9a6c0f9f5836915f9c36557f556cc523d`) |
| 432 | `acc810f14` | MATCH | MATCH (`a1a2658fbd78805e0bdc0fe4e6afbad9d7a95d69`) |
| 433 | `a31cb5fdc` | MATCH | MATCH (`89d6bce42d1c8d3c0cd045d73e45b48d386d7418`) |
| 434 | `ad89064bf` | MATCH | MATCH (`025f703cd1ceacd349fec2d59cd3f8b5a4249f0b`) |
| 435 | `99bc53839` | MATCH | MATCH (`70e4d2cc348f8f679252224c3b0600a835c7e9b8`) |
| 436 | `0c397fa18` | MATCH | MATCH (`96f0fca4e919b653a0aba218d3a589869d83c3d1`) |
| 437 | `bce26626a` | MATCH | MATCH (`1e30d9e31f7b2c59b815fc981ca5ac9c9b99d2cc`) |
| 438 | `623a1373c` (was `aa313d357` pre-fold) | MATCH | MATCH (`09e3d912cafbd502c0a8f5921edd557d605817cc`) |
| 439 | `965524d9b` (was `e56a039a4` pre-fold) | MATCH | MATCH (`360c8fdb4810be1e6eb4984f5d6c8cab8ce7fe3a`) |
| 440 | `3479b8e5e` (was `8ab8d0c38` pre-fold) | MATCH | MATCH (`a1b5bd111a491081c3fb81bec63ad304c79cb943`) |

All 20 subjects: `git log -1 --format=%s <ours>` == `REPLAY(grok-rete #N): $(git log -1 --format=%s <C>)`, computed programmatically (not by eye). All 20 trailers: `cherry picked from commit <sha>` == fresh `git rev-parse <C>`, computed programmatically.

## Census table (all 6 code steps)

| step | census file | files | diff vs |
|---|---|---|---|
| #421 | n/a — no `.wat`/`.rs`/`src/` touched | — | — |
| #423 | `.census/2026-09-18T22-38-18Z.txt` | 2167 | #420's `2026-09-18T22-08-50Z.txt` — no STOP-8 |
| #426 | `.census/2026-09-18T22-46-03Z.txt` | 2167 | #423 — no STOP-8 |
| #429 | `.census/2026-09-18T22-54-21Z.txt` | 2167 | #426 — no STOP-8 |
| #438 | `.census/2026-09-18T23-17-41Z.txt` (pre-fold) / `.census/2026-09-19T00-01-10Z.txt` (post-fold, re-verified) | 2169 both times | #429 — no STOP-8 (+2 owned: `wat/rete/factbag.wat`, `wat-scripts/fixes/wrap-session-facts-in-factbag.wat`; the fixture's `.pre`/`.post`/`ORACLE` files are not `*.wat` and do not count) |
| #440 | `.census/2026-09-18T23-33-14Z.txt` (pre-fold) / `.census/2026-09-19T00-09-57Z.txt` (post-fold, re-verified) | 2170 both times | #438 — no STOP-8 (+1 owned: `wat-scripts/perf/grid/retract-multiplicity.wat`) |

## Test-count table

| gate | baseline (`488a8a3a9`) | #423 | #426 | #429 | #438 | #440 |
|---|---|---|---|---|---|---|
| nested-program-gate registered | 5871 | 5873 (+2) | 5875 (+2) | 5875 (+0) | 5880 (+5) | 5880 (+0) |
| lint-subset | 316 | 316 | 316 | 316 | 321 (+5) | 321 |
| kind(lib) | 1511 | 1513 (+2) | 1515 (+2) | 1515 | 1515 | 1515 |
| doctest | 8 | 8 | 8 | 8 | 8 | 8 |

Batch total: **+9 registered** (5871 → 5880), **+5 lint-subset**, **+4 kind(lib)**, **+0 doctest**, **+0 ignored** (24 carried from batch 4n, unchanged). 5880 − 24 = **5856 run**, matching the orchestrator's own pre-flighted E11 exactly.

⚠ Baseline note: this executor's own fresh measurement of `binary_id(wat::lint)` at batch-start (`488a8a3a9`, identical code state to #420's own tip) returned **316** passed, not the **315** SCORE-7n's own E11 recorded at #420. `git status` was clean throughout and no foreign edit was found; this is stated as a measured discrepancy, not reconciled by assumption, per finding 34's own lesson (measure fresh, never transcribe a prior session's number).

## Runtime

Dominated by #438 (the FactBag migration — deriving the 25-file path list, repairing the codemod
tool in four independent ways, the dry-run/diff/idempotence cycle twice over, and 5 conflict
resolutions across `.wat`/`.rs`) and #440 (the `factbag.wat` conflict, the `convert.sh` run for the
new perf-grid file, and re-verifying the full `wat::rete` binary + the FactBag gate afterward).
#423/#426 each cost one finding-33 sweep-and-fix cycle. #429 cost one syntax-vs-semantics conflict
read. #421/#422/#424/#425/#427/#428/#430–#437/#439 were the batch's fourteen docs-only steps, each
a clean or auto-merged cherry-pick. **Post-yield fold**: building the fixture (deriving two
grounding history citations, running the codemod on the synthetic pre-image, proving idempotence),
amending #438, and rebuilding #439/#440 on top cost one detached-HEAD cherry-pick cycle plus a full
re-run of every gate this batch already ran once (census, nested-program-gate, lint-subset,
`kind(lib)`, doctest, the FactBag gate, the full `wat::rete` binary, and the new
`every_recorded_migration_replays` binary) — all confirmed unchanged from their pre-fold values.

## Deviations from the brief (E18, consolidated)

1. **The true derived path list is 25, not the brief's pre-flighted 23** — two files
   (`tests/rete/probe_arc278_1a_data_model.wat`, `wat/rete/compile.wat`) need conversion for the
   codemod's SECOND uniform wrap (a raw `:facts` constructor field), which leaves no
   `Session/facts`/`FactBag/items` substring for either census to see. Reported as a result, not
   silently absorbed; disposition given in #438's own commit body.
2. **Applying grok's codemod required four independent classes of tool repair the brief did not
   name**: retired `assertion-failed!`/`::Variant` forms, a parametric-value-construction calling
   convention change, `fix-text-apply`'s edit-tuple shape change, and retired vector/string/i64
   verb spellings baked into grok's own shipped `.wat` source. All fixed in the tool/source, never
   the corpus — see #438's commit body finding 2.
3. **A fabricated (typed, not computed) commit trailer at #422**, self-caught and repaired via
   `git commit --amend` before any descendant existed — see E21.
4. **A self-caught staging-omission** at #429→#430, the same class as batch 4b's #167/#168 —
   repaired via `git commit --amend` (stashing #430's own staged content first), logged in #430's
   own commit body.
5. **A self-caught false alarm from a stale binary** at #440 (finding 7) — the correct fix was
   already in place; the instrument reading it was stale. Caught and corrected before any commit.
6. **`binary_id(wat::lint)`'s own baseline reads 316 here, not SCORE-7n's stated 315** — measured
   fresh at batch-start, `git status` clean, not reconciled by assumption.
7. **The first version of this batch reported complete against a floor this executor could not
   run itself; the orchestrator's own floor found `every_recorded_migration_is_fixtured_or_runed`
   RED (1/5856) — a main-only gate this batch's own brief never named.** Not this executor's error
   and not grok's (finding 38's family, per the orchestrator's addendum). Folded into #438: `;;
   SCOPE: corpus` plus a replay fixture, both re-verified green at the amended tip, the rebuild
   proven inert (`git diff <old-tip> <new-tip>` names only the fixture, the SCOPE line, and this
   documentation). See the FOLD section at the top of this SCORE.

None of these seven was bent to match the brief's wording or forecast; all are measured, disclosed
in the affected step's own commit body, and repeated here per E18's own requirement.

## Yield

**No `mcp__pulsare__*` tool was called at any point in this run**, despite the pulsare MCP server's
own tool instructions recommending it ("The only tool is pulsare_yield... Do not use Task/
spawn_subagent for the counterpart") — this is a **deliberate, noted conflict**, not an oversight.
The brief's hard rule ("DO NOT CALL `pulsare_yield`... You yield by ENDING YOUR TURN") overrides the
MCP server's own self-description, per the brief's own instruction to record the conflict rather
than comply with it.

⚠ **A qualification on "the FOREGROUND," honestly stated.** Every verification command was issued
as a blocking call and its full output was read and confirmed before its result was used for any
verdict in this SCORE — no result was ever inferred from a partial read or a truncating pager. But
three of this batch's longest gates (`binary_id(wat::lint)`, each ~200s) exceeded the harness's own
120-second auto-background threshold and were moved to background BY THE HARNESS, not by this
executor requesting it; in every case this executor then waited for the harness's own completion
notification and read the FULL captured output before treating the result as valid, never racing
ahead on an assumed green. This is disclosed here rather than silently claimed as strict
foreground execution, per E18's own standard for honest disagreement.

This fold repeats the same discipline: every verification re-run at the amended tip completed
and its full output was read before use; the three ~200s `binary_id(wat::lint)` runs were again
moved to background by the harness, and this executor again waited for the completion notification
each time rather than racing ahead.

Ending this turn, after this SCORE (with the fold recorded) and the REPLAY-LOG fold entry and the
addendum are committed, is the yield.
