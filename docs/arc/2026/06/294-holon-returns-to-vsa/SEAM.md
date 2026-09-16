# SEAM — the ONE live breadcrumb. 2026-09-14. ⛔ **YOU ARE ON `replay/grok-rete`, NOT MAIN.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice, which is why it will feel like *continuing* rather than *waking*, and **that feeling is
> the failure.** Run the datamancy bootstrap (the grimoire + the 4 primers from the **SIGNED MCP**,
> never a disk copy), read `docs/COMPACTION-AMNESIA-RECOVERY.md`, then run the commands below before
> you touch anything.

> `251/SEAM.md` · `255/SEAM.md` · `278/SEAM.md` and `278/CURRENT-STATE-annihilate-interpretation.md`
> are PARKED and point HERE. ⛔ **PARKED IS NOT DEAD.**

## FIRST — RUN THESE. DO NOT READ THE NUMBERS BELOW.

```bash
git rev-parse --abbrev-ref HEAD        # expect replay/grok-rete (merge/grok-rete is only the REFERENCE)
git status --porcelain                 # dirty = grok is mid-strike: commit NOTHING until it yields
git log --oneline -12; git log origin/replay/grok-rete --oneline -1
cat /home/john/work/holon/.pulsare/to-claude          # has grok scored? which file? (compare its mtime to the last brief)
ls bootstrap/pending/                   # parked drafts / curare
git log --oneline | grep -c 'REPLAY(grok-rete #'       # how far the replay has come
readlink .census/latest                 # the census baseline the next step diffs against
```

Stamp: written at HEAD `351b24fbc` (4g's SCORE commit; the records commit lands on top of it). In flight:
nothing — **batch 4g (#261–#280) is CLOSED and PUSHED**; **280 of 651 replayed**. Next is **batch 4h
(#281–#300)**: censused two-sided — **15 docs-only, 5 code (#283 #288 #294 #298 #300)**, **ZERO hazard
rows of any kind** (no stdlib-touch, no absent-on-main, no future-macro, no main-deleted) and **not one
step touches `wat/`**; the next two-phase stdlib step is still #377. One M-status-absent path (#294
modifies `tests/lint/rete_citation_resolves.rs`) is created at #283 earlier in the same batch.
⚠ **#283 IS THE TRAP AND IT WILL LAND RED — the THIRD new grok lint gate to meet main's divergent
corpus.** `tests/lint/rete_citation_resolves.rs` (913 lines) demands every backticked identifier and every
bare `*.rs`/`*.wat` filename cited in a comment under `src/rete/` either RESOLVE or carry
`// rune:lint(cited-name-absent) <name> — <reason>` (40-char reason floor). The other 27 files in #283 are
grok repairing ITS OWN citations; ours diverged by the same renames/module splits that forced #270's
ledger reseed, so our unresolved set is different and probably larger. Repair **AT #283** (#184
precedent). ⛔ **Never reword a CORRECT citation to dodge a red** — the gate's own header warns that too
narrow a universe manufactures findings, and six rete-comment names are legitimately attested only
outside `src/`.
📊 **The pattern is now three for three**: every new grok lint gate lands red here (#274 → 9 undeclared +
1 hollow; #278 → 63 rune declarations across 11 files; #283 → predicted). Budget for it.
⛔ **The executor is a spawned AGENT, not pulsare** (2026-09-15: the builder's grok credits are exhausted
for ~2 days): Opus for substrate/judgment stones, Sonnet for mechanical replay batches.

## WHERE THE WORK IS (verify each against `git log`)

```
main              a3218644d   FROZEN · PUSHED                                     do not touch
origin/grok-rete  37528f6e0   FROZEN — read with `git show`, never check it out
replay/grok-rete  (this)      main + stone 0 + pilot #1–#10 + 2b + 2a1/2a1b/2a2/2a4/2a4b/2a4c (tooling, CLOSED)
                              + batch 1 #11–#60 (CLOSED; floor 5495/5495, clippy 0, pushed)
                              + stone 3, the nested-program gate (CLOSED; floor 5502/5502, pushed)
                              + batch 2 #61–#125 (CLOSED; folded per 5b; floor 5540/5540, pushed)
                              + batch 3 #126–#152 (CLOSED; floor 5546/5546, pushed)
                              + batch 4a #153–#159 (CLOSED; floor 5551/5551, pushed; P1+Q1's first use)
                              + 2a4d, the per-SET stdlib world (CLOSED; floor 5552/5552, pushed)
                              + batch 4b #160–#211 (CLOSED; floor 5569/5569, clippy 0, pushed)
                              + batch 4c #212–#220 (CLOSED; floor 5576/5576, clippy 0, pushed)
                              + batch 4d #221–#225 (CLOSED; floor 5578/5578, clippy 0, pushed)
                              + batch 4e #226–#240 (CLOSED; floor 5593/5593, clippy 0, pushed)
                              + batch 4f #241–#260 (CLOSED; floor 5615/5615, clippy 0, pushed)
                              + batch 4g #261–#280 (CLOSED; floor 5657/5657, clippy 0, pushed)
                              ⇒ 280 of 651 replayed. NEXT: batch 4h #281–#300.
merge/grok-rete   REFERENCE   the first (rejected) whole merge; a crib and the end cross-check only
```

- **Batch 1 is CLOSED** (SCORE-4, SCORE-4b, REPLAY-LOG; finding 18). 50 steps; the #60 checkpoint went
  red because two walls met the other side's content (#50's rete wall vs main-only fmt rules; main's
  separator lint vs #60's `config.rs`). The repairs were FOLDED into #50/#60 (#51–#60 rebuilt; proof:
  the old tip and the new differ by exactly the repairs). The TDD close held: families A/B/C live.
- **Verified after 4b (finding 19):** 2a4b's stdlib-list gate read `stdlib.rs`'s TEXT (a commented-out
  row passed) — rewritten to ask the runtime, `33f3ebcfb`. Absolute paths had blinded the codemods'
  two path rules; run5 now hands repo-relative paths. **G1: the stdlib door refuses a divergent MACRO**
  (and expands every `defn` body) — batch 1 unaffected; 6 later steps from #155 need it.
- **2a4c is CLOSED** (`0bdac1ccd`; the orchestrator's floor 5498/5498 and run5 reproduced grok's: MA 0 ·
  PC 2 · VS 0 losing, chain 1370; the only `wat/` refusals left are the era `format`/`defservice` bodies).
- **Stone 3 is CLOSED** (`91ed5fc54`; SCORE-3): `tests/lint/nested_program_starts.rs` starts every
  nested program in a derived program-carrying position on the child's real path (123 checked; RED on
  the pre-2b erase child; a pin must name a live test); the seven arc-170 children start, via three
  recorded `wrap-nested-forms-*` migrations; `arc112_scheme_probe` shipped its worker. Verified by the
  orchestrator (floor 5502/5502), plus two probe headers and a dead parent worker fixed.
  `cf-norevoke`'s raise is HISTORY: the committed revoke test was fixed long ago (`Outcome.Bounced/Served`).
- **Batch 2 is CLOSED** (SCORE-5, SCORE-5b; findings 20–21). 65 steps; the #125 checkpoint's two reds
  (#95's keyword rows vs main's registry ratchet + `RETE_MODULES`; #108's doc vs main's doctest floor)
  were repaired AFTER #125 — FOLDED into #95/#108 by 5b (proof: the rebuilt tip's tree is identical;
  subjects unchanged). The step gate now also runs the library's unit tests + doctests (STOP-11).
- **Batch 3 is CLOSED** (SCORE-6; finding 22). 27 steps; #126 re-expressed on main's moved homes; the
  #139 checkpoint's golden red folded into #126 by grok itself. The record carried the non-census walls at
  only 3 of 11 steps — the orchestrator re-ran them at the 8 unproven commits: all clean.
  `scripts/replay/verify-step-record.sh` now makes the record checkable (five verbatim lines from #153).
- **Batch 4a is CLOSED** (SCORE-7a; finding 23). 7 steps, the policy's first use: 5 ported tools with
  fixtures, the Q1 re-expressions, `convert.sh` refusing a chain member. Verified: floor 5551/5551, clippy 0,
  run5 unchanged (chain vs main 1370), and the record gate's FIRST PASS. Two METHOD flaws routed to 2a4d —
  the stdlib door reads one file at a time (4a's enums live in `wat/rete.wat`, their users in fmt/grep/
  query/oracle: grok KEY-FIRST'd the leftovers BY HAND), and #155 hand-wrapped `(overlay records)` in 11
  CHAIN members (necessary, no tool could, R21).
- **2a4d is CLOSED** (`816cb99a0` + `b6a8b0f38`; findings 24–25). The door is asked once per SET and unions
  the KEPT rows per member (the raw-source union stripped the shared world — 17 `wat/` refusals vs 1);
  `wrap-overlay-in-fireoutcome` records #155's hand wrap (11/11 byte-identical, idempotent); 4a's hand
  KEY-FIRSTs are proven by re-converting #155 with the fixed door. Verified: floor 5552/5552, clippy 0,
  run5 identity unchanged (chain vs main 1370, VS report back to 22).
- **Batch 4b, FIRST HALF #160–#185 — the mid-batch checkpoint only.** ⚠ **SUPERSEDED by the full-batch row
  directly below; kept for its trap-door detail, NOT a current-state claim** (`67267bfd7`; findings
  26–27). 26 steps. Verified at the checkpoint: floor **5554/5554, 24 skipped** — which is the
  test-count delta PREDICTED from the diff (+4 = #184's two new gate files, −2 = #183's deliberate
  `#[ignore]`s) — clippy 0, and **E7 re-run at HEAD reproducing #184's verdict lines exactly** (kind(lib)
  1475+4, doctest 8, lint-subset 148, stone-3 gate 3, `kernel::tests` 87+2 = #177's 89). E1/E2/E4/E8/E9
  re-checked by the orchestrator, not credited; all 17 `.census/` files named in bodies exist. Three trap
  doors cleared: #176 (19 files; ours is 7 lines short of C's, the deliberate convergent-duplicate drop),
  #177 (`kernel/tests.rs` 10,189 → 14 files, landing GROK'S OWN decomposition — file names identical),
  #184 (two new lint gates; its `no_stale_path_in_doc` RED repaired IN #184, not folded into #168, because
  #168 was green under every gate existing when it landed and C's own body fixes its six the same way).
- **Batch 4b #160–#211 is CLOSED and PUSHED** (`7a2b7d45f`; SCORE-7b, REPLAY-LOG; findings 26–28). 52 steps
  (51 real + #202 empty). Orchestrator-verified: floor **5569/5569, 24 skipped** — again the count PREDICTED
  from the diff before running — clippy 0, E7 reproducing #207's verdict lines exactly (kind(lib) 1478+4,
  doctest 8, lint-subset 149, stone-3 3), all 7 new `.wat` `--check` rc=0, and
  `verify-step-record.sh 060199f7f HEAD 160 211` green on BOTH the range and the record. E1/E2/E8/E9
  re-checked by the orchestrator; 7/7 `.rs.txt` harness files byte-identical to grok's; 8/8 census files
  named in bodies exist. **#190 carries the folded #202 strike** (finding 28).
- **Batch 4g #261–#280 is CLOSED and PUSHED** (steps at `351b24fbc`; SCORE-7g, REPLAY-LOG; finding 35).
  20 steps, 15 docs-only. Verified by the orchestrator: floor **5657/5657, 24 skipped** — the count
  PREDICTED from the diff for the **ninth consecutive batch** — clippy 0, and every wall re-run at HEAD
  reproducing #278's own numbers (lint-subset 192, kind(lib) 1490, doctest 8, nested-gate 3/3, census 2122
  files). All five gates green in ONE run: #274's meta-gate, #278's resolver, and its three controls.
  **Structural integrity after TWO `git filter-branch --msg-filter` passes, verified independently**: all
  five landmark SHAs (`7b58b6cbd` `400165612` `fcc5febcf` `4f9276699` `c3c824a34`) still exist AND are
  ancestors of HEAD; origin still an ancestor, exactly 21 commits behind; **0 `refs/original/`**, 0 replace
  refs. No published history was rewritten — see finding 35 on why the mechanism is still a loaded gun.
  **#274's predicted trap landed exactly as briefed** — 9 undeclared + 1 hollow — and was repaired AT the
  step (all 30 `tests/lint/*.rs` ride in #274's own commit), with one gate RECLASSIFIED from my brief's
  own guess after the executor read it. The brief told it not to trust my split; it didn't, and was right.
  **#278** dropped the dead-path hunk, added no inert rune under `tests/resolve/`, and then hit an
  UNPREDICTED landing-time burden: **63 rune declarations across 11 files** (44 distinct names), every one
  naming a mechanism with a file:line citation. Verified honest: **no rename-table row was re-pointed
  anywhere** — the only non-rune change under `wat-scripts/fixes/` is grok's own map/filter row deletion.
  **#270's ledger verified SHRINK-ONLY**: grok's 35 tuples → our 27, 8 removed, **ZERO added**, every
  removal a stale path main had renamed independently.
  The `attested()` exclusion at #278 is verified **verdict-neutral and LEGITIMATE** — finding 35 says why.
  ⛔ **Do not "fix" it**: retiring the orphan would NOT restore the control.
- **Batch 4f #241–#260 is CLOSED and PUSHED** (steps at `400165612`; SCORE-7f, REPLAY-LOG; finding 34).
  20 steps, 14 docs-only. Verified by the orchestrator: floor **5615/5615, 24 skipped** — the count
  PREDICTED from the diff for the **eighth consecutive batch** — clippy 0, and all four walls re-run at
  HEAD reproducing #258's verdict lines exactly (kind(lib) 1490, lint-subset 155, doctest 8, stone-3 gate
  3/3, census files=2117). The range gate passes IDENTICALLY with and without `GIT_NO_REPLACE_OBJECTS=1`;
  0 replace refs. E2 re-checked two-sided, E4's 11 `.wat` re-run (8 rc=0, 3 deliberate rc=1 refusal
  fixtures whose named tests assert `!ok`), E13's 6 census artifacts all present.
  **#254's deliberate divergence HELD**: the struck apportionment assertion (finding 32) is absent; the
  single `grep` hit is the strike's own explanatory comment, and the executor proved it PREDATES #254 by
  checking the pre-#254 blob rather than by explaining it away.
  **The executor found two defects in its OWN commit bodies before yielding** and folded both in place,
  never as a post-batch commit: #242/#254 omitted `census`/`nested-program-gate` (it had misread the
  record gate's rule as `.wat`-triggered when it is PATH-based — `^src/` also qualifies), and #242
  overstated a test count. Repaired by detach + re-commit + rebuild; **proven inert independently by the
  orchestrator** — `git diff 20bf647f3 fcc5febcf` = 0 lines, `git diff 7cfc1d93f fcc5febcf` = 0 lines,
  260 steps at all three tips. Two rebuilds, not one.
- **Batch 4e #226–#240 is CLOSED and PUSHED** (`c3c824a34`; SCORE-7e, REPLAY-LOG; findings 32–33). 15 steps.
  Orchestrator-verified: floor **5593/5593, 24 skipped** — the predicted count for the SEVENTH consecutive
  time — clippy 0, E7 reproducing #238's verdict lines exactly (kind(lib) 1482, doctest 8, lint-subset 153,
  stone-3 3, its named test 1/1), and the range gate green **identically with and without
  `GIT_NO_REPLACE_OBJECTS=1`**. E2/E15/E16 re-checked by me; E16 (finding 31's new row) passed its first
  real exercise. `--check` on `wat/rete/syntax.wat` DISPROVED with a **delta of 0** — #226 modifies a macro
  rather than adding top-level `:wat::` defns, where 4d's `fire.wat` gave +3 for three new ones: the count
  tracks DEFINITIONS, not edit volume, confirmed in both directions.
  ⛔ **#234 STOPPED on a red that was NOT its own** (finding 32) — the executor refused every barred
  disposition, and the controlled measurement exonerated it. #238 then caught a SECOND instance of the
  embedded-wat class (finding 33) before committing.
- **Batch 4d #221–#225 is CLOSED and PUSHED** (`b036c5fd8`; SCORE-7d, REPLAY-LOG; finding 31). 5 steps.
  Orchestrator-verified: floor **5578/5578, 24 skipped** — the predicted count for the FIFTH consecutive
  time — clippy 0, E7 reproducing #221's verdict lines exactly (kind(lib) 1478, doctest 8, lint-subset 153,
  stone-3 3, its named test 2/2), and the range gate green **identically with and without
  `GIT_NO_REPLACE_OBJECTS=1`** (finding 29's rule: zero `refs/replace/`). E2/E11/E13 re-checked by me.
  ★ **2a4d IS PROVEN FOR A REAL 2-FILE STDLIB SET** — the reason this boundary existed. Each member was
  converted SOLO and diffed byte-for-byte against the 2-file union output, both revisions: **all four
  comparisons identical**. No sibling converts differently inside the union than alone.
  ⚠ **`--check` on a `wat/` stdlib file is UNSATISFIABLE by construction** and was DISPROVED, not skipped —
  I re-derived it myself from identical path shapes: main's own untouched `fire.wat` gives **16**
  `ReservedPrefix`, ours **19** — exactly +3 for the three new top-level defns. Generalises finding 25's
  `wat/fix.wat` precedent to all of `wat/`.
- **Batch 4c #212–#220 is CLOSED and PUSHED** (steps at `628e26a00`, records at `154672b4a`; SCORE-7c,
  REPLAY-LOG; findings 29–30). 9 steps.

> ⚠ **THE BATCH BULLETS ARE NOT IN ONE CONSISTENT ORDER.** They run 4a → 4b → **4e → 4d → 4c**: ascending
> to 4b, then REVERSE-chronological, because each new row was anchored on the previous newest and landed
> above it. Every row is individually dated with its own SHA, but **do not read position as recency.**
> **The STAMP (top) and the WHERE block are the authority on what is current.** Left unshuffled
> deliberately: moving multi-line bullets has mangled a splice in this file before, and the cost of the
> disorder is legibility, not truth.
  Orchestrator-verified: floor **5576/5576, 24 skipped** — the predicted count for the fourth consecutive
  time — clippy 0, E7 reproducing #218's verdict lines exactly (kind(lib) 1478, doctest 8, lint-subset 153,
  stone-3 3), E3 over all 8 touched `.wat` giving **5 rc=0 / 3 rc=1** exactly as predicted, and
  `surface-field-dispatch.wat` **printing 142** — grok's own bar ("Not 'it loads'"). E2/E13/E15 re-checked
  by the orchestrator. **#212 carries two orchestrator folds:** the `git replace` record repair rebuilt for
  real (finding 29), and the `Signal::User1` rune/rot migration (finding 30).
  ⚠ **#212's own gate was DRIVEN, not assumed** — it found two rots the ruling's three planning docs never
  predicted, and the executor migrated them via recorded codemods rather than runing them.
- **Superseded release note (kept for its pointers):** batch 4c (`BRIEF-7c…` + `EXPECTATIONS-7c…`, now TRACKED — the
  7b pair is tracked too, since `SCORE-7b` cited a brief that lived only in gitignored `bootstrap/pending/`),
  ending BEFORE **#221,
  the first MULTI-stdlib-file step since 2a4d** — the first real exercise of the per-SET world, deliberately
  isolated (an orchestrator scheduling call inside the batch rule, not a new boundary; the builder may fold
  it in). #226 changes a stdlib MACRO — the G1 class 2a4c closed. Step notes:
  `docs/arc/2026/06/294-holon-returns-to-vsa/the-grok-rete-replay/STEP-NOTES-212-and-226.md`. ⚠ #212's gate walks **8** docs `.wat` here, not grok's 10
  (the arc-130 pair is `.wat.bad`); grok's own DESIGN/EXPECTATIONS for that gate land at #211 and
  **disagree with the shipped #212** in two places — follow the commit.
- **The doctrine:** `[[project_merge_doctrine_syntax_vs_subsystems]]`. Main owns syntax; the branch owns
  its subsystem; replay ONE COMMIT AT A TIME; correct over fast; seconds are not worth a stone (RULED).

## THE REPLAY — how a step goes (`BRIEF-1-pilot-first-ten-commits.md` § "One step")

- docs-only: `git cherry-pick -x C`. Otherwise cherry-pick `--no-commit`, `convert.sh C^ <out> <paths>`
  and `convert.sh C <out> <paths>`, new `.wat` = the C output, modified `.wat` = `git merge-file` of
  main's copy with the two converted versions, `.rs` re-expressed on main's homes; gate: build,
  `--check` every produced `.wat`, C's own tests by name; commit `REPLAY(grok-rete #N)`.
- **From #61 the gate adds (BRIEF-4b D):** the fast lint subset on every `.rs` step, and
  `scripts/replay/census.sh` + `--diff` whenever the binary or any `.wat` changed. **STOP-8:** a file
  rc 0 → non-zero that the step did not produce — a wall meeting the other side's content. Found at its
  step, the repair is part of that step's commit. **STOP-10** (stone 3): the nested-program gate runs with
  the census and must stay green. **STOP-11** (5b, from #126): every `.rs` step also runs the library's
  unit tests (`kind(lib)`) and the doctests.
- **LATENT (RULED):** a `:wat::*` call head that exists on neither side is re-expressed with main's
  registered verb of the same meaning at that one site, logged `LATENT (c3fefc5ab)`; STOP-7 if none.
- **Stdlib steps (2a4):** a step touching `wat/*.wat` converts in two phases (stdlib files with the
  door's stdlib mode, rebuild, then consumers). The door's stdlib mode keys on a repo-relative `wat/`
  prefix; tracked `wat/**/*.wat` ≡ the baked list is gated by asking the runtime. Until 2a4c lands it
  REFUSES a changed stdlib macro (G1); after it, `convert.sh` reporting `UNREGISTERABLE` for a `wat/`
  file is STOP-9. Every path handed to a codemod is REPO-RELATIVE (the codemods' two path rules —
  `stdlib-source-path?`, positional-ctor's `skip-path?` — assume it).
- **A composition defect found late FOLDS into the step that needs it** (this branch becomes main; no
  knowingly-red REPLAY commit). Rebuild the later steps; prove it with `git diff <old-tip> <new-tip>`.
- **Batches** (census `bootstrap/era/replay-plan/commits.tsv`): #153 next, after its policy; batches END BEFORE
  each commit needing a new policy —
  - #153 #155 #157 #159 #278 #438 #440 #627 #628 #630 #635 #638 change grok's own codemods (a
    conversion policy is still owed);
  - #212 #278 are NOT boundaries (finding 21's correction): with rename detection every "deleted" file was
    MOVED — #212's two `130-…` `.wat` R100 → `.wat.bad` (both sides preserve them; the suffix IS grok's
    rune, whose gate scans `.wat` only → drop the rune, merge the README prose, log it), #278's probe R055 →
    a `tests/resolve/` fixture outside the `wat-scripts/` lint grok's 4 lines served → drop, log;
  - #377 #379 touch `wat/core.wat`/`wat/service.wat`, whose era `format`/`defservice` bodies STOP-9
    will refuse; #379 also changes `wat/service.wat`, which positional-ctor skips (arc 296 M2 RESIDUE 1).
  A step that edits a file main MOVED (`src/stdlib.rs`, `src/edn_shim.rs`, `src/string_ops.rs`) is the
  standing `.rs` rule, not a boundary.
- **End:** cross-check against `merge/grok-rete` (its 8 tree-wide-missing grok tests,
  `bootstrap/merge-audit/`), then floor + clippy, then main. The ignored-test NAMES in grok-rete's
  files must equal grok-rete's own at its tip.

## KNOWN FLAWS — routed, not left (FINDINGS-composition.md is the detail)

- **The builtin set lives in two stores** — `BUILTIN_PRIMITIVES` (`src/runtime.rs`) and
  `builtin_names` (`src/types.rs`): 14 only in the first, 12 only in the second (`SCORE-2a1b.md`).
- **One ruling's table lives in THREE codemods:** `alias-enum` in match-arm and positional-ctor, and
  `bare-variant-to-qualified`'s `rename-five`. One home in `wat/fix.wat`.
- **`variant-parent-of`'s own `@example` is false** (`src/reflect/verbs.rs`); the doctest verifier is
  `#[ignore]` (its NOTE awaits a ruling).
- **2b's three:** unreachable refusal arms (`match_arm.rs`, `check.rs`); ~21 doc lines teach `::`; c03's
  `variadic-wrap` hardcodes `:wat::core::i64`.
- `docs/…/278-rules-engine/probes/surface-field-dispatch.wat`: fixed when its grok-rete commit replays.
- The chain vs main's 28 CHAIN-FAILS: classify with `bootstrap/era/probe-K/classify.sh`.
- **#190's struck ratio floors failed HARDER here than on grok's branch, and why is OPEN.** grok's #202
  records its gate *"passed 8 for 8 with 70% headroom"* in isolation; ours failed **4 of 10 at idle**, whole
  idle range below grok's whole calibrated range (S/L 2.00–4.38 vs 4.88–5.58; F/L 1.12–2.31 vs 2.38–2.84).
  Same box, same pinned rustc 1.97.0, fixture semantically identical. Remaining candidate: this binary's
  codegen around a tight two-element loop. Harmless now the floors are struck — but NOT understood.
- **ONE TRUTH IN TWO PLACES, and only one copy is on the floor** (found at #167): the oracle-fire rewrite
  lives in BOTH `wat-scripts/perf/grid/run-axis.sh` (a `perl` substitution) and
  `tests/rete/wat_scripts_grid_axes_live.rs`'s `skip_oracle_fire` — and only the latter is gated, which is
  why the script side sat broken from the moment the fire-outcome wall landed (it swapped in a bare
  `Session` where the axes expect `(:wat::rete::FireOutcome :- [Session])`). One home, or a gate proving the
  two agree. Kin: "One ruling's table lives in THREE codemods".

## INSTRUMENTS — where they live

| what | where |
|---|---|
| frozen binaries | `bootstrap/wat-main-a3218644d` (main), `bootstrap/wat-replay-base-8a5b7eb20` (replay base) |
| **the codemod bar** | `bootstrap/era/probe-S/run5.sh` (each tool vs its predecessor; ~10 min; clean tree; repo-relative paths since 2026-09-14 — baseline MA 0 · PC 2 · VS 0 losing, chain 1370) |
| the stdlib door on one file | `bootstrap/era/probe-R/door-stdlib.wat` (+ `stdlib-div/` divergent copies) |
| future steps changing a stdlib macro | `bootstrap/era/replay-plan/future-macro-changes.txt` |
| **the whole-tree census** | `scripts/replay/census.sh` (tracked; captures in ignored `.census/`) |
| the #60 probe, sites, old-tip census | `bootstrap/era/probe-T/` |
| main-binary residual classifier | `bootstrap/era/probe-K/classify.sh` |
| the door over the corpus / the one-step identity | `bootstrap/era/probe-R/corpus.sh`, `verify-refute2.sh` |
| the replay census | `bootstrap/era/replay-plan/{commits,flags,stdlib-touch}.tsv`; **`absent-on-main.tsv`** (the honest modify/delete census — `main-deleted.txt` missed MOVED files, finding 21) |
| nested-program census | `bootstrap/era/probe-N/extract-forms2.wat` |

A temporary Rust probe goes in `src/check.rs`'s tests module before `    fn decls(src: &str)` by SCRIPT,
on a stashed tree, then `git checkout -- src/check.rs`.

## OPERATIONAL RULES — each one was paid for

- **Never commit on a tree the EXECUTOR is working in.** Park tracked edits in `bootstrap/pending/`.
- **The executor gets the acceptance criteria and CHEAP TARGETED checks** (`cargo build --release`,
  `-E 'test(<name>)'`, `--check`, a named probe) — never the floor, clippy, run5 or a push; the orchestrator
  weighs centrally, uncontended. With a spawned agent the prompt must also CARRY the doctrine
  (`wat-rs/CLAUDE.md` does not reach it): R21 codemods, no known flakes, capture a red verbatim and never
  re-run, no worktrees, no subagents, the anchor path, and "ending your turn ENDS you — verify in the
  FOREGROUND" (an agent backgrounded its gate run and lost it).
- **An acceptance row nothing can satisfy teaches an executor to fake it** (finding 25: `--check` on baked
  stdlib). Retire the row; never waive it.
- **After ANY commit or `--amend`: assert `git status --porcelain` is EMPTY and that the commit's own diff
  names every path its body claims** (finding 27). An Edit-tool fix to a file `cherry-pick --no-commit`
  already staged is NOT in the commit unless re-`git add`-ed, and a dirty tree right after a commit is that
  defect's signature. THREE occurrences in one session, across BOTH agents — it is the tool's shape, not a
  habit. And read every COUNT off `git show --stat`/the diff before writing it into a body: a count in a
  commit body is a measurement (#162 logged 4 of 7 sites; #184 logged 2 of 1 added files).
- **Pass `verify-step-record.sh` the batch's STEP RANGE** (`<from> <to> <first-N> <last-N>`), or it cannot
  see a mis-subjected or skipped step — finding 26. Every step, docs-only included, commits as
  `REPLAY(grok-rete #N): <C's subject>` with the `-x` trailer kept in the body.
- ⛔ **WAT EMBEDDED IN `.rs`/`.sh` STRINGS IS THE REPLAY'S MOST PERSISTENT DEFECT SOURCE** (finding 33) —
  #162, #167, #238 so far, each invisible to every gate because no `.wat` file is involved. When a step's
  subject mentions a rename or rehome, **grep the `.rs`/`.sh` side too**. Rung: CONVENTION — the instrument
  that would catch it (a wat parser aimed at Rust string literals) does not exist.
- ⛔ **A RENAME CENSUS MUST BE BUILT FROM THE RECORDED MIGRATIONS, NOT THE SHAPE OF THE NAME** (finding 33).
  `rete::core::{i64,f64,string}` were rehomed; `rete::core::keyword::=` was deliberately NOT (it is a live
  `#[wat_special_form]`). A pattern lumping them flagged correct work as stale.
- ⛔ **A WALL-CLOCK RATIO IS NOT A GATE — but measure before striking one** (finding 32). Three such gates
  have now been struck here (two at #190/#202, one at `gather_probe_cost.rs`), each after a CONTROLLED
  measurement, never on resemblance. A census found a FOURTH (`harvest_cost.rs:338`) that **is kept**: it
  reports in milliseconds, is two-sided, and never fired in 15 runs. Same shape ≠ same defect.
  **Before any disposition, check whether the branch fixes it later** — it did at #202, and did not here.
  And prove the fix: the struck gate went 2-of-15 → **0-of-15**.
- ⛔ **Each verdict line in a commit body must be ON ONE LINE** (finding 31). `verify-step-record.sh`'s
  patterns match within a single line, so a hand-WRAPPED `census: … --diff no STOP-8` fails a wall that
  genuinely ran, and the gate then looks wrong to an author whose record was true. Do not reword the claim
  to satisfy it — unwrap the line.
- ⛔ **Fix a mis-teaching instrument AT THE SOURCE, not in the brief that hit it** (finding 31). BRIEF-1's
  item 1 (`Docs-only: git cherry-pick -x C.`) read as a complete instruction and contradicted item 4; it
  misled THREE executors in a row, the last two escaping only because their briefs patched over it. Item 1
  is now amended. Patching each brief while the source keeps mis-teaching fixes the case and leaves the
  class.
- ⛔ **When an executor reports a REPAIR, ask what OBJECT GRAPH proves it** (finding 29). A `git replace`
  overlay made the record gate read `complete` here and `MISSING #215` under `GIT_NO_REPLACE_OBJECTS=1` —
  `refs/replace/` is local and **`push` does not carry it**, so the DR site and every clone would have
  disagreed with this box. Re-run the gate the way the PUSHED state will be read.
- ⛔ **A mutation proof must falsify THE PROPOSITION YOU RELY ON** (finding 30). Stripping a rune proves the
  GATE notices a missing rune — NOT that the rune's stated reason is the real cause. To audit a
  declaration, run its own sentence and read the ERROR TEXT; `rc=1` says nothing about why.
- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything — redirect to a file and read the
  file** (finding 28). "Capture the red verbatim" is UNACTIONABLE if the RUN was truncated: by the time you
  know it is red, the block is gone. **BOTH SIDES of this merge hit it independently** (our #190; grok's own
  #198, *"a trap door that is mine"*), so it is the affordance, not one agent. Rung: CONVENTION — no gate
  can see how a command was invoked, so it is re-stated in every executor prompt.
  `[[feedback_a_truncating_pager_makes_absence_unfalsifiable]]`
- **An isolated re-run is the WEAKEST evidence against a failure seen under load**, and on a timing test it
  answers a different question (1 test alone ≠ 13-of-87 under contention).
- **A FULLY-folded step still gets an EMPTY commit** (`git commit --allow-empty`) carrying its
  `REPLAY(grok-rete #N)` subject and `-x` trailer, with a body naming where its content went — the range
  gate requires one commit per N. #202 is the first; a cherry-pick whose changes are already present STOPS
  rather than committing, so verify the outcome (`git show --stat` = 0 files), not a flag name.
- **A floor runs uncontended, and nothing tracked is edited while it runs** (lint tests read source
  text at runtime).
- **Never decide on `pgrep -f`.** Wait on a DONE line the job writes.
- **A probe's input is what the real stage receives; a probe guards that its edits actually ran**
  (the #60 probe's first run pointed at a missing site file and dropped nothing).
- **A gate's sabotage must falsify the CLAIM, not the proxy** (2a4b's text-scan gate passed with
  `wat/gen.wat` un-baked). `[[feedback_a_green_test_can_prove_nothing]]`
- **A step gate sees only what the step produced.** `[[feedback_a_step_gate_sees_only_what_the_step_produced]]`
- **A rider's STOP can be right while its cause is wrong** (#11 was latent; #60's arm 1 was #60, not #59).

## ⚠ RULINGS — do not re-litigate

- **Character case carries NO meaning.** **A variant IS a tagged record; `Variant <: Enum`.**
- 2026-09-12: replay one commit at a time; all migrations replay on a fixture.
- 2026-09-13: build the declaration door; C (one membership door); stone 3 between batches;
  batch 1 = #11–#60; the LATENT rule; the `convert.sh` speedup DROPPED (seconds are fine).
- 2026-09-14: 2a4 (the door's stdlib mode + two-phase stdlib steps); 4b (fold the #60 repairs into
  #50/#60; a wall-driven recorded migration; the per-step census). All four 4-YES.
- 2026-09-15: **no other branch is approached until grok-rete is proven a success** (the builder: "we are
  not approaching another branch until we know grok-rete was a success"). "Success" is § THE REPLAY's
  End. After it lands, the replay recipe becomes ONE playbook so the next branch starts at "batch 1".
- 2026-09-15: **the codemod-source policy — P1 and Q1** (`the-grok-rete-replay/POLICY-codemod-source.md`):
  grok-rete's own new codemods are PORTED (converted like any `.wat`; a fixture from their own step's
  converted corpus file); its edits to main's tools are RE-EXPRESSED on main's version (the tool's
  fixture still replays). A codemod step is no longer a boundary; #278 stays one for a deleted file.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.** Run the FIRST block. Trust `git log` over every line here.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `DERIVAMVS NE MENTIAMVR.` ·
> `HAERESIS EST ITERVM ROGARE.`
