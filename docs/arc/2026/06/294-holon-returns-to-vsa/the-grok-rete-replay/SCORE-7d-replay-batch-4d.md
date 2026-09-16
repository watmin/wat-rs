# SCORE 7d — replay batch 4d: grok-rete #221 → #225

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched. `~/work/holon/` untouched.
Parent brief: `BRIEF-7d-replay-batch-4d.md`.
Start: `2922feecd` (batch 4c's tip, 220 REPLAY commits, tree clean). HEAD: `efa6e7035` (#225).
Ended before #226 per the brief.

## The five

| N | C | replayed | kind |
|---|---|---|---|
| 221 | `16f504e14` | `a484e2078` | **shared — the batch's whole risk.** First two-stdlib-file step since stone 2a4d; `wat/rete/oracle/fire.wat` + `wat/rete/oracle/stratify.wat`, a new 192-line probe `.wat`, a new 106-line `.rs` test |
| 222 | `a49b68608` | `8ffb7c23a` | docs |
| 223 | `cd2ab4b37` | `72154d99d` | docs |
| 224 | `85043bbab` | `ad532d614` | docs |
| 225 | `c8f1f7839` | `efa6e7035` | docs |

`scripts/replay/verify-step-record.sh 2922feecd HEAD 221 225`:

```
step-range: #221..#225 each present exactly once, sources match
step-record: complete
```
(exit 0 — see "Record repair" below for how #221 reached this state.)

## EXPECTATIONS (EXPECTATIONS-7d's E1–E14)

| # | result |
|---|---|
| E1 | **PASS.** `verify-step-record.sh 2922feecd HEAD 221 225` → exit 0, `step-range: #221..#225 each present exactly once, sources match`. |
| E2 | **PASS.** `git show --name-only` on #222 `8ffb7c23a`, #223 `72154d99d`, #224 `ad532d614`, #225 `efa6e7035`: every path is under `docs/` and ends `.md`. |
| E3 | **DISPROVED for the two stdlib files, PASS for the fixture — not worked around (finding 25's precedent).** `./target/release/wat --check tests/rete/probe_arc278_oracle_accumulate_supersedes.wat` → rc=0. `--check` on `wat/rete/oracle/fire.wat` and `wat/rete/oracle/stratify.wat` both → rc=1, `ReservedPrefix` (19 and 9 errors respectively) because both define into the `:wat::` prefix, which the plain `FsLoader` load `--check` uses refuses for ANY `wat/` stdlib file by construction. Verified this is not a defect: `git show 2922feecd:wat/rete/oracle/fire.wat` (main's own unmodified copy at the batch's start tip, pre-step) checked the identical way gives rc=1 with 16 `ReservedPrefix` errors — the same class, pre-existing. `every_ungated_wat_checks.rs` explicitly excludes `wat/` from its scope ("already covered... by the binary's own stdlib load"); `tracked_wat_dir_is_stdlib_sources.rs` confirms `wat/**/*.wat` **is** the baked `(:wat::stdlib::sources)` list. The real gates for these two files are `cargo build --release` succeeding (it did, and re-embeds the changed `include_str!` content — confirmed by binary sha256 moving `d5071893e…`→`8f917acde…` on the rebuild that followed the merge, and staying fixed at `8f917acde…` across a further no-op rebuild with zero recompile lines, so the move is real content, not build noise) plus every test that boots the runtime (which every_wat_scripts test and the named probe do). |
| E4 | **PASS.** `convert.sh`'s own log for both phases (`93ea0c618`→before, `16f504e14`→after) names BOTH `wat/rete/oracle/fire.wat` and `wat/rete/oracle/stratify.wat` at every one of the 20 codemod stages; no `UNREGISTERABLE` anywhere. Rebuild between phase (a) and phase (c) is in the commit body verbatim. |
| E5 | **PASS, measured, not assumed.** Each stdlib member was ALSO converted SOLO (before and after) and `diff`ed byte-for-byte against its 2-file-union output: `fire.wat` and `stratify.wat` are **IDENTICAL solo vs union**, both revisions. 2a4d's per-member-KEPT-rows fix holds for this 2-file set — no sibling converted differently inside the union than alone. |
| E6 | **PASS.** `cargo nextest run --release -E 'test(probe_arc278_oracle_accumulate_supersedes)'` → 2 tests run, 2 passed. |
| E7 | **PASS, N > 0.** `-E 'test(no_inlined_edn) + test(no_loose_string_assert) + test(no_inlined_wat)'` → 30 tests run, 30 passed. |
| E8 | **NOT RUN — by design.** `scripts/floor.sh` and `clippy` are explicitly reserved for the orchestrator (brief's ⛔); I did not run either. |
| E9 | **Not scored by me** — requires the orchestrator's floor run to compare against. Predicted delta from the diffs: #221 adds 2 `#[test]` fns (the new probe's own two tests) and 0 removed; #222–#225 add 0 (no `.rs`). No `#[ignore]` delta introduced by this batch. |
| E10 | **Not run by me** — orchestrator's spot re-run, reserved. Self-measured numbers for the record: lint-subset 153 passed, `kind(lib)` 1478 passed, doctest 8 passed, nested-program-gate (`test(nested_program_starts)`) 3 passed — all held identical across #221 (the only code step; #222–225 are docs-only and re-run nothing). |
| E11 | **PASS.** `git diff --name-only 2922feecd..HEAD` touches only `wat/rete/oracle/{fire,stratify}.wat`, `tests/rete/probe_arc278_oracle_accumulate_supersedes.{rs,wat}`, and four `docs/` paths — no `wat-scripts/fixes/` edit (its codemods were RUN, never edited), no `absent-on-main.tsv` row in this range (confirmed empty per the brief). |
| E12 | **PASS.** No repair commit after #225. The one record repair (#221's verdict-line wrap) was applied via `git rebase -i` to REWORD #221 in place, before #222–#225 existed to be rebuilt on top of anything — see below. |
| E13 | **PASS.** Both `.census/*.txt` files #221's body cites exist on disk: `.census/2026-09-16T05-33-01Z.txt` (pre) and `.census/2026-09-16T06-13-01Z.txt` (post, 2107 files). |
| E14 | **PASS.** `git replace -l` → 0 entries. The repair used a real `git rebase -i` (reword), which produces ordinary reachable commits with new SHAs — nothing lives in `refs/replace/` or any local-only overlay. `git diff <old-tip> <new-tip>` = 0 lines (the rebase changed only #221's commit message, not one byte of tree content), so the repair is visible to `push` by construction; no `GIT_NO_REPLACE_OBJECTS=1` re-run was needed because no replace mechanism was used. |

## #221 — the whole batch's risk, driven and disproved where the bar could not be met

The two-phase stdlib convert ran exactly as prescribed: `convert.sh 93ea0c618 … wat/rete/oracle/fire.wat wat/rete/oracle/stratify.wat` and the `16f504e14` counterpart, `git merge-file` per file (0 conflicts on both — grok's change and main's syntax-migrated content threaded cleanly), the merged files installed into the tree, `cargo build --release`, THEN `convert.sh 16f504e14 … tests/rete/probe_arc278_oracle_accumulate_supersedes.wat` (new in C) against the rebuilt binary. `fire.wat`'s diff against main's prior content matches the commit message's own description exactly: `fire-fixpoint` split into `fire-grow-fixpoint` (the old monotone half, renamed) + a new `fire-support-fixpoint` (the shrinking half) + `retain-supported`; `stratify.wat`'s change is comment-only, as claimed.

**2a4d's first real 2-file exercise measured, not assumed.** Per finding 25's exact method, both stdlib members were converted solo and compared byte-for-byte against the 2-file union, both revisions: all four comparisons (`fire`/`stratify` × before/after) are IDENTICAL. No sibling misbehaved inside the union.

**The `--check` row on the stdlib pair could not be satisfied, and was disproved rather than worked around** (finding 25's own precedent for `wat/fix.wat`): `wat --check` on a `wat/` stdlib file always fails `ReservedPrefix` because it defines into `:wat::`, which the plain-user-program loader `--check` uses refuses categorically — verified true of main's own UNMODIFIED `fire.wat` before this step touched it (16 `ReservedPrefix` errors there vs. 19 after, exactly +3 for the three added top-level defns `retain-supported`/`fire-grow-fixpoint`/`fire-support-fixpoint` replacing the old single `fire-fixpoint` entry). The real gates for `wat/` content are the binary's own stdlib load (`cargo build --release`, which re-embeds the `include_str!`'d files — confirmed via a binary-sha256 move that a follow-up no-op rebuild proved was real content, not build churn) and any test that boots the runtime — which the named probe test and the finding-24 hygiene walls both do.

The new `.rs` test file was ALREADY in the finding-24 house shape (a co-located `.wat` fixture read via `call_beside_value`, `assert_eq!` throughout, no inlined EDN/wat literals) — the hygiene walls (`no_inlined_edn`, `no_loose_string_assert`, `no_inlined_wat`) ran anyway per the brief's instruction and came back 30/30 green, confirming rather than merely assuming the shape.

## Record repair — a `git rebase -i` reword, no `git replace`, no new commit

`verify-step-record.sh 2922feecd HEAD 221 225`'s first run reported `MISSING #221 … census: .*--diff no STOP-8`. The underlying check had genuinely run and passed (census diff was clean, `no STOP-8`, both files on disk) — the commit BODY's census line was simply wrapped across two lines by hand-formatting, and the verifier's regex matches within a single line only, so `--diff no STOP-8` on the second line never joined `census:` on the first. Repaired by rewording #221 in place with `git rebase -i 2922feecd` (`GIT_SEQUENCE_EDITOR` marked the first pick as `reword`, `GIT_EDITOR` substituted the corrected single-line message) — this was done BEFORE #222–#225 existed as commits reachable only through the old #221, so "rebuild the descendants" reduced to the rebase carrying them forward automatically. Proven: `git diff <old-tip 34ac9b123> <new-tip efa6e7035>` = **0 lines** — no byte of tree content moved, only the one commit message. `git replace -l` = 0 entries throughout; no overlay of any kind was used, so the fix is visible to `push` unconditionally.

## Fold rule

No fold. The one defect found (#221's wrapped verdict line) was a same-step record correction applied via reword before any later step depended on the broken message, not a composition defect discovered downstream — there was nothing to rebuild "onto" corrected content, since the rebase mechanism itself carries #222–#225 forward unchanged (proven 0-line diff above).

## Judgement calls the brief did not cover

- **Docs-only steps still get the corrected subject, never a bare `git cherry-pick -x`.** The brief's own item 1 ("Docs-only: `git cherry-pick -x C`") read literally would reproduce finding 26's exact trap (grok's own subject surviving verbatim). Used `git cherry-pick -x --no-commit` for #222–#225 too, then hand-composed the `REPLAY(grok-rete #N): <C's subject>` commit with the `(cherry picked from commit …)` trailer preserved, matching the minimal format #220 (also docs-only) already used in this branch's own history.
- **`--check` on `wat/` stdlib files is categorically unsatisfiable and was disproved, not skipped or faked.** See E3 and the #221 section above; this generalizes finding 25's `wat/fix.wat` precedent to `wat/rete/oracle/*.wat`.
- **Binary-sha256 as build evidence needed its own control.** A rebuild on a genuinely unchanged tree in this environment does NOT recompile or change the hash (measured directly: two consecutive `cargo build --release` runs with no edits between them produced 0 "Compiling wat" lines and an identical hash the second time), so the observed sha256 move immediately after installing the merged stdlib files is real content, not environmental noise. Recorded as an explicit control rather than asserted.
- **nested-program-gate's concrete filter**, not named verbatim in this brief, taken from BRIEF-7c's precedent: `test(nested_program_starts)`, selecting 3 tests (3/3 passed here, matching every prior batch's number).

## Blast radius

The replayed commits' own files (`wat/rete/oracle/fire.wat`, `wat/rete/oracle/stratify.wat`,
`tests/rete/probe_arc278_oracle_accumulate_supersedes.{rs,wat}`,
`docs/arc/2026/06/278-rules-engine/**`), plus this directory's SCORE and REPLAY-LOG. No
`wat-scripts/fixes/` edit (codemods RUN, never edited). No ref other than `replay/grok-rete` moved;
no `refs/replace/` entries.

## STOP

None. Do not push. Main untouched. `~/work/holon/` untouched. No subagents spawned. No worktrees
used. Tree clean at yield (`git status --porcelain` empty).
