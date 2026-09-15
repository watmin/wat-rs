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

Stamp: written at HEAD `74f52aed1` (= origin). In flight with grok: nothing — #153 is a boundary: the
codemod-source policy (`bootstrap/pending/POLICY-153-codemod-source.md`) awaits the builder's ruling.

## WHERE THE WORK IS (verify each against `git log`)

```
main              a3218644d   FROZEN · PUSHED                                     do not touch
origin/grok-rete  37528f6e0   FROZEN — read with `git show`, never check it out
replay/grok-rete  (this)      main + stone 0 + pilot #1–#10 + 2b + 2a1/2a1b/2a2/2a4/2a4b/2a4c (tooling, CLOSED)
                              + batch 1 #11–#60 (CLOSED; floor 5495/5495, clippy 0, pushed)
                              + stone 3, the nested-program gate (CLOSED; floor 5502/5502, pushed)
                              + batch 2 #61–#125 (CLOSED; folded per 5b; floor 5540/5540, pushed)
                              + batch 3 #126–#152 (CLOSED; floor 5546/5546, pushed)
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
- **Next: #153, a BOUNDARY** — the first of 12 steps that change `wat-scripts/fixes/`. The draft policy
  (`bootstrap/pending/POLICY-153-codemod-source.md`): P1 port grok's 7 new codemods (convert like any
  `.wat`, a fixture from their own step's converted corpus file); Q1 re-express grok's edits to main's
  tools on main's version. Awaits the builder's ruling.
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
  - #212 #278 edit files main DELETED for cause (`absent-on-main.tsv`, finding 21; 4-YES candidate: keep
    main's deletion, drop grok's edit, log it — the builder's ruling is owed at #212);
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

- **Never commit on a tree grok is working in.** Park tracked edits in `bootstrap/pending/`.
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

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.** Run the FIRST block. Trust `git log` over every line here.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `DERIVAMVS NE MENTIAMVR.` ·
> `HAERESIS EST ITERVM ROGARE.`
