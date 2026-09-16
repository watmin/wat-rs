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

Stamp: written at HEAD `628e26a00`. In flight: nothing — **batch 4c (#212–#220) is CLOSED**; **220 of 651
replayed**. Next is **#221**, the first MULTI-stdlib-file step since 2a4d.
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
                              + batch 4c #212–#220 (CLOSED; floor 5576/5576, clippy 0)
                              ⇒ 220 of 651 replayed. NEXT: #221, first MULTI-stdlib step since 2a4d.
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
- **Batch 4c #212–#220 is CLOSED** (`628e26a00`; SCORE-7c, REPLAY-LOG; findings 29–30). 9 steps.
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
