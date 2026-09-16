# SCORE 7e — replay batch 4e: grok-rete #226 → #240

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched. `~/work/holon/` untouched.
Parent brief: `BRIEF-7e-replay-batch-4e.md`.
Start: `8cd884e9a` (batch 4d's tip, 225 REPLAY commits, tree clean). HEAD: `3347c6732` (#240).

**COMPLETE.** All 15 steps (#226–#240) committed and green. One mid-batch STOP at #234, resolved not
by anything in this batch but by an orchestrator strike (`0fa6948da`) landed on top of #233 while
this executor was paused — see "#234 — STOP then EXONERATED" below.

## The steps

| N | C | replayed | kind | note |
|---|---|---|---|---|
| 226 | `7319c1ea4` | `27e5f9c5c` | shared, 6 files (+1: `src/intrinsic/mod.rs`) | **THE TRAP — driven, see below** |
| 227 | `f67024851` | `77f568bcd` | docs | |
| 228 | `a685a9d8d` | `a8266de5a` | docs | 1-hunk auto-merge |
| 229 | `4a0bbf793` | `29c8b5ced` | docs | |
| 230 | `bb0256e38` | `5715e2d44` | shared, 2 `.rs` | |
| 231 | `f7237cd8a` | `24887777c` | docs | |
| 232 | `4a77fa915` | `3bf9a1064` | docs | 1-hunk auto-merge |
| 233 | `99120fc8b` | `97987f74f` | shared, 4 files (1 `.wat`, 0 `.rs`) | **path-based trap — driven correctly** |
| — | — | `0fa6948da` | **orchestrator, not a REPLAY step** | strikes `probe_extend_cost_split`'s nanosecond ratio gate |
| — | — | `9ef711fbd` | **orchestrator, not a REPLAY step** | finding 32 + SEAM |
| 234 | `057f9d494` | `e8eceb7e8` | code, 2 `.rs` | **STOP-11, then EXONERATED — see below** |
| 235 | `d7c2949e1` | `f82abd59f` | docs | |
| 236 | `72d8d2c42` | `e4987f441` | docs | 1-hunk auto-merge |
| 237 | `e38a5de20` | `c1216a60e` | docs | |
| 238 | `17fc5fb3e` | `af49c0bba` | shared, 3 `.rs` | **hand-fixed wat-in-string, see below** |
| 239 | `61097cc1e` | `65bc434f1` | docs | |
| 240 | `09b973d2c` | `3347c6732` | docs | 1-hunk auto-merge |

`scripts/replay/verify-step-record.sh 8cd884e9a HEAD 226 240`:

```
step-range: #226..#240 each present exactly once, sources match
step-record: complete
```
(exit 0, run in the foreground on the final tree.)

## EXPECTATIONS (EXPECTATIONS-7e's E1–E16)

| # | result |
|---|---|
| E1 | **PASS.** `verify-step-record.sh 8cd884e9a HEAD 226 240` → exit 0, `step-range: #226..#240 each present exactly once, sources match`, `step-record: complete`. |
| E2 | **PASS.** `git show --name-only` on every docs-only step (#227–#229, #231–#232, #235–#237, #239–#240) — every path under `docs/` and ending `.md`. |
| E3 | **PASS.** #233's body carries `census`/`nested-program-gate`, and — confirmed directly, not just by the gate's silence — no `lint-subset`/`kind(lib)`/`doctest` line anywhere in it (it has no `.rs`). |
| E4 | **PASS.** #226's body names `wat/rete/syntax.wat` and the `cargo build --release` rebuild between phases; no `UNREGISTERABLE wat/` anywhere in either convert.sh log for this batch. |
| E5 | **PASS.** convert.sh's output for `wat/rete/syntax.wat` at both `C^` and `C` was read in full: no divergent-macro report fired (the G1 class exists for the file's `defquery` macro per `future-macro-changes.txt`, but C's diff does not touch `defquery`'s body — only `with-network`/`with-overlay`'s comments and let-body move). |
| E6 | **PASS.** Every non-`wat/` `.wat` this batch produced or modified (`tests/rete/probe_arc278_export.wat` at #233) → `--check` rc=0. |
| E7 | **PASS, disproved not faked.** `wat/rete/syntax.wat` pre-step vs post-step, identical path shape: both rc=1, both name the SAME 5 `ReservedPrefix` defns — delta 0, matching the diff (an existing defn's body changed; 0 new top-level `:wat::` defns added). |
| E8 | **PASS, N > 0 at every code/shared step.** #226 3/3, #230 4/4, #234 7/7, #238 1/1 (`every_acc_head_shaped_row_runs_as_an_acc_head`, non-vacuous by its own internal assert). #233 has no `.rs`, correctly no named test. |
| E9 | **Self-measured, not run by me.** `scripts/floor.sh`/clippy explicitly reserved for the orchestrator and NOT run — per the brief's ⛔. |
| E10 | **PASS, test-count delta accounted for at every code/shared step.** `kind(lib)`: 1478 (batch start) → 1481 (#226, +3 new tests, AFTER fixing a real `REGISTRY_MEMBERSHIP_GAP_A` gap the diff itself created) → 1481 (#230, +0 — the new tests are an integration binary, not `kind(lib)`) → 1481 (#233, no `.rs`, not run) → **RED at #234** (unrelated pre-existing test, resolved by orchestrator strike `0fa6948da`, not by anything in #234's own diff) → 1481 (#234, re-run clean after the strike) → 1482 (#238, +1 — its own new test). `lint-subset` held at 153 throughout (no new lint-binary test file in this batch); `doctest` held at 8 throughout. |
| E11 | **Not run by me** — the orchestrator's own spot re-run at HEAD is reserved. Self-measured numbers for the record, above (E10), plus `nested-program-gate` (`test(nested_program_starts)`) holding at 3/3 across every code/shared step in this batch. |
| E12 | **PASS.** `git diff --name-only 8cd884e9a..HEAD` touches only this batch's own claimed files (`wat/rete/syntax.wat` IS expected — #226) plus this directory's docs, plus the two orchestrator commits' own files (`src/rete/kernel/tests/gather_probe_cost.rs`, `docs/.../SEAM.md`, `FINDINGS-composition.md` — not from any REPLAY step). No `wat-scripts/fixes/` edit anywhere. No `absent-on-main.tsv` row in `226..240`. |
| E13 | **PASS.** No repair commit after #240. The one mid-batch stop (#234) was never committed red — it was left uncommitted, then committed green only after the orchestrator's independent strike, in its normal position in the sequence. |
| E14 | **PASS.** Every `.census/…txt` file cited in #226/#230/#233/#234/#238's bodies exists on disk (2107 files throughout — no step in this batch adds or removes a tracked `.wat`). |
| E15 | **PASS.** `git replace -l` → 0 entries. No overlay of any kind was used anywhere in this batch. |
| E16 | **PASS.** Every `census:`/`nested-program-gate:`/`lint-subset:`/`kind(lib):`/`doctest:` line in every code/shared step's body (#226, #230, #233 partial, #234, #238) matches on ONE line — verified directly with `grep -cE` per commit, shown below. |

## #226 — the trap, driven exactly as warned, plus one gate the diff itself required

TWO-PHASE stdlib convert on `wat/rete/syntax.wat`: `convert.sh` at `C^`(`c8f1f7839…`)/`C`(`7319c1ea4…`),
`git merge-file` (0 conflicts), merged content installed, `cargo build --release`
(`8f917acde…`→`74c86db96…`, confirmed genuine by an immediate zero-op control rebuild reproducing the
same hash exactly). No `UNREGISTERABLE` anywhere — no STOP-9. The G1 divergent-macro class
(`:wat::rete::defquery`, `future-macro-changes.txt`) did not fire: read in full, convert.sh's log
shows no divergent-macro report, and C's diff does not touch `defquery`'s body. `--check` on the
stdlib file disproved rather than faked: pre/post both rc=1, same 5 `ReservedPrefix` names, delta 0
(matches the diff — an existing body changed, no new top-level defn).

`src/rete/purity.rs` and `src/runtime.rs` conflicted exactly as predicted (main's most-diverged
files): main had already homed/classified `arm-session`/`release-session` and deleted the whole
ledger block grok's diff assumed present. Re-expressed by hand (a `.rs` conflict, not R21 — a corpus
rewrite rule): grok's one new ledger entry and one new dispatch arm landed at the equivalent live
location; grok's own "beside its two siblings" comment was corrected rather than copied, since the
siblings are no longer in that ledger on this tree. Grok's redundant re-addition of a
`release-session` match arm was dropped (already intercepted by main's registry-first door). Two
wat-in-`.rs`-string hand-fixes, logged in the commit body: `assertion-failed!` positional→kwargs,
and a HARD-CUT retirement (`:wat::core::i64::/`→`:wat::i64::/`).

**A real gate the diff itself created, fixed within the step.** `kind(lib)` first ran RED on
`registry_membership_gap_a_is_named_and_frozen` — check.rs's cherry-picked `TypeScheme` for
`adopt-session-lease` has no `registry()` row (the fn is `#[restricted_to]`, not `#[wat_intrinsic]`,
deliberately). This is finding 20's exact class (#95's identical gate), whose own ruling sanctions
adding a NEW name to `REGISTRY_MEMBERSHIP_GAP_A`. Fixed in `src/intrinsic/mod.rs`, inside this same
commit — not folded in later, learning finding 20's own lesson that landing the fix after the batch
left intervening steps knowingly red. Re-ran: 1481/1481.

## #227–#232, #235–#237, #239–#240 — docs-only

Ten steps, every one `git cherry-pick -x --no-commit` + hand-composed
`REPLAY(grok-rete #N): <C's subject>` + trailer (finding 26/31's rule, never a bare `-x`). #228, #232,
#236, #240 each auto-merged one hunk against local drift in
`CURRENT-STATE-annihilate-interpretation.md`, conflict-free.

## #230 — shared, 2 `.rs`

`fix(rete): wall 5 — the import door bounds its own recursion`. Clean auto-merge on both files
(`src/rete/export.rs`, `tests/rete/probe_arc278_export.rs` — the latter pre-existing, C's diff a pure
append). No wat-in-string literals. 4 named tests, 4 passed. `kind(lib)` unchanged (1481 — the new
tests are an integration binary).

## #233 — the path-based trap, driven correctly

`strike: draw D3`. 3 docs + 1 `.wat` (`tests/rete/probe_arc278_export.wat`, pre-existing, modified),
0 `.rs` — the record gate correctly required only `census`/`nested-program-gate`. The 3 docs
auto-merged clean. The `.wat` file's own raw cherry-pick auto-merged too, but in grok's OWN syntax
(un-converted) — discarded and redone by the modified-`.wat` recipe: `convert.sh` at `C^`/`C` (both
clean), `git merge-file` (0 conflicts), result matches C's diff exactly on main's syntax, `--check`
rc=0. No `.rs`, so no named test — correctly noted as such rather than silently skipped.

## #234 — STOP, then EXONERATED by an orchestrator strike, not by this step's content

`fix(rete): an argument with no parameter is refused, not placed`. Both files (`eval.rs`,
`probe_arc278_export.rs`) auto-merged clean, matching C's diff exactly. One wat-in-`.rs`-string
hand-fix logged: `synthetic_user_fence`'s `rete_op_index(":wat::rete::core::i64::<")` lookup used a
retired name (corpus already renamed to `:wat::rete::i64::<`); fixed at
`tests/rete/probe_arc278_export.rs:910`. All 7 named tests passed (confirming the fix — a wrong name
would have panicked the lookup helper itself, not merely mis-asserted). census (2107 files, unchanged)
and nested-program-gate (3/3) both clean.

`cargo nextest run --release -E 'kind(lib)'` then failed:

```
FAIL [   0.171s] ( 636/1481) wat rete::kernel::tests::gather_probe_cost::probe_extend_cost_split
thread 'rete::kernel::tests::gather_probe_cost::probe_extend_cost_split' (500797) panicked at
src/rete/kernel/tests/gather_probe_cost.rs:887:5:
combined (34 ns) is far below its parts b+m+e (82 ns) — the combined closure is no longer doing
the work the parts describe
```

**Exact assertion:** `h >= (b + m + e) * 0.5` at `gather_probe_cost.rs:887`, a nanosecond wall-clock
apportionment check. `gather_probe_cost.rs` was introduced by grok and replayed at #183/#184, entirely
outside this batch's blast radius, and #234's own diff does not touch it. **Not dismissed** (doctrine
bars "pre-existing"/"unrelated to my change" as dispositions): captured verbatim, the exact assertion
named, NOT re-run, and #234 was left **uncommitted**, its cherry-picked + hand-fixed content preserved
in the working tree, and the finding surfaced to the orchestrator.

**The orchestrator ran the control this executor could not** (no floor/clippy/run5, and no repeated
`kind(lib)` runs, per the brief's own constraints) and confirmed the diagnosis: the SAME `kind(lib)`
invocation on the clean #233 tree WITHOUT #234's diff failed 1 of 10 (33 ns vs 79 ns); WITH #234's
diff, 1 of 5 (35 ns vs 87 ns) — indistinguishable populations, combined 2 of 15 (~13%). **#234 is
exonerated** — the red was noise from a nanosecond-scale wall-clock ratio gate under nextest's own
parallelism, not content #234 introduced. Struck in a separate orchestrator commit (`0fa6948da`, plus
`9ef711fbd` recording finding 32 and the SEAM), deliberately **not** folded into #234 or any REPLAY
step: grok's own tree carries `probe_extend_cost_split`'s `* 0.5` bound unchanged to its tip (same
constant at all seven later revisions touching the file, and two of them — #416, #470 — remove work
from `h`, making the gate MORE fragile downstream, never less), so this strike is main's own decision
against a gate grok keeps, and folding it into a replayed step would have stopped that step's diff
from matching grok's. Proven struck: 2-of-15 → 0-of-15 over 15 further runs, floor 5585/5585, clippy 0.
A census found a fourth gate of the same shape (`harvest_cost.rs:338`, millisecond scale, two-sided,
non-vacuity-guarded) and it was correctly **kept** — same shape is not the same defect.

Resumed on the coordinator's word: re-read `HEAD` (now `9ef711fbd`, moved while paused), verified the
two preserved files were byte-for-byte unchanged (`git diff --stat` reproduced the exact same
321+/8- this executor left), rebuilt, re-ran every wall — `kind(lib)` now 1481/1481, all else
unchanged — and committed #234 green, with the STOP-then-exoneration narrated in its own body rather
than silently absorbed.

## #238 — shared, 3 `.rs`, a second wat-in-string re-expression

`fix(rete): the fence and the executor share one head-space`. All 3 files (`src/rete/expr_ir/mod.rs`,
`src/rete/kernel/arm.rs`, `src/rete/reachability.rs`) auto-merged clean, 331+/5- matching C's diff
exactly. Two hand-fixes in the new `reachability.rs` code, both R21's one exception (embedded wat the
codemods cannot reach), both logged in the commit body:

- `probe_eq_for`'s three numeric/string match arms carried a stale `core::` segment
  (`:wat::rete::core::{i64,string,f64}::=`); the numerics-to-their-homes rename already moved these to
  `:wat::rete::{i64,string,f64}::=` on this tree — confirmed against `vocabulary.rs`'s own `rete_name`
  rows AND this same file's own pre-existing driver table a few hundred lines above (which already used
  the correct spellings). `bool`/`keyword` correctly KEEP `core::` and were left untouched. The
  function's own doc comment explains why this had to be exact: the name is checked against
  `rete_op_index` by the caller specifically so a rename cannot rot silently — three of five type arms
  would have hit that designed-in red on every run had this gone uncorrected.
- The embedded wat program template inside `synth_acc`'s `format!` carried grok's old positional match
  arms and positional `assertion-failed!`. Re-expressed to main's bracket dot-variant syntax and kwargs
  `assertion-failed!`, with every field name cross-checked against the pre-existing, already-correct
  `tests/rete/probe_fence_names_the_head_core_op.wat` (identical `CompileOutcome`/`InsertOutcome`/
  `FireOutcome` shapes). `:wat::rete::core::defn` for `:probe::wrapped` was confirmed CORRECT as
  written, not stale — a live, current, widely-used form required for a fn used as a rete acc/then
  head, not a rename target.

Named test `every_acc_head_shaped_row_runs_as_an_acc_head` — 1/1, and its pass is itself the
confirmation the two hand-fixes are correct: the test drives EVERY eligible `RETE_OPS` row, including
the three numeric/string types the fix touched, through an internal assert that would have named the
row and pasted the failing program on any residual mismatch. `kind(lib)` 1482 (was 1481 — the test's
own +1).

## Fold rule

No fold performed anywhere in this batch. #226's `REGISTRY_MEMBERSHIP_GAP_A` fix was landed INSIDE
#226 itself (the step whose own diff created the gap), not folded in after the fact — correct order
per finding 20's own lesson, not a fold. The `kind(lib)` red at #234 had no step in this batch to fold
into (the failing file, `gather_probe_cost.rs`, is outside the blast radius) and was struck by the
orchestrator in its own separate commit, deliberately not folded into #234 or any REPLAY step (grok's
own tree keeps the struck assertion to its tip — see #234 above).

## Judgement calls the brief did not cover

- **#226's `REGISTRY_MEMBERSHIP_GAP_A` gap.** The brief warned #226 would need re-expression on
  `src/check.rs`/`src/runtime.rs` but did not anticipate this specific registry-completeness gate
  (`src/intrinsic/mod.rs`, arc 255's ratchet). Resolved by following the gate's own error message and
  finding 20's precedent exactly, landed inside the step rather than folded later.
- **`arm-session`/`release-session`'s comment in `purity.rs` was adapted, not copied.** Grok's own
  rationale named those two as still-unruled "siblings"; on main they are already classified. Copying
  the stale claim verbatim would have shipped a comment that is false on this tree the moment it
  lands — corrected instead, logged as a judgement call, not silently changed.
- **Two pre-existing (not new) files at #230 and #233** (`tests/rete/probe_arc278_export.{rs,wat}`)
  showed as `M`, not `A`, under cherry-pick — confirmed each time by checking the file's actual diff
  shape (a pure append) rather than assuming "new in C" from the brief's file-count table alone.
- **The #234 `kind(lib)` red's scope**, at STOP time: determined it was outside this batch's blast
  radius by (a) `git log -S` on the failing test confirming it predates `#226` by many steps, and (b)
  confirming #234's own diff does not touch the failing file — both static checks (no re-running the
  failing gate), which is what made the STOP a report rather than a guess.
- **Resuming after the orchestrator's strike**, the byte-identity of the preserved evidence was
  verified independently (`git diff --stat` against the pre-pause numbers) before rebuilding or
  re-running anything, rather than trusting the coordinator's message alone.
- **#238's two wat-in-string fixes were cross-checked against LIVE pre-existing content in the same
  files/corpus** (this file's own driver table; `probe_fence_names_the_head_core_op.wat`) rather than
  reconstructed from memory of the naming convention, the same discipline used at #226/#234.

## Blast radius

The replayed commits' own files (`wat/rete/syntax.wat`, `src/check.rs`, `src/intrinsic/mod.rs`,
`src/rete/kernel/arm.rs`, `src/rete/kernel/tests/arm_lease.rs`, `src/rete/purity.rs`,
`src/runtime.rs`, `src/rete/export.rs`, `src/rete/expr_ir/{eval,mod}.rs`, `src/rete/reachability.rs`,
`tests/rete/probe_arc278_export.{rs,wat}`, `docs/arc/2026/06/278-rules-engine/**`), plus this
directory's SCORE and REPLAY-LOG. No `wat-scripts/fixes/` edit. No ref other than `replay/grok-rete`
moved; no `refs/replace/` entries. `src/rete/kernel/tests/gather_probe_cost.rs` and
`docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md`/`FINDINGS-composition.md` moved too, but as the
ORCHESTRATOR's own two commits (`0fa6948da`, `9ef711fbd`), not as part of any REPLAY step or this
executor's blast radius.

## STOP

None outstanding. The mid-batch STOP-11 at #234 is resolved (see above) — the orchestrator's control
measurement and strike commit exonerate the step and remove the red; #234 committed green as part of
this batch's normal sequence. Do not push. Main untouched. `~/work/holon/` untouched. No subagents
spawned. No worktrees used. Tree clean at yield (`git status --porcelain` empty).
