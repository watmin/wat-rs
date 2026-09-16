# SCORE 7e — replay batch 4e: grok-rete #226 → #240

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched. `~/work/holon/` untouched.
Parent brief: `BRIEF-7e-replay-batch-4e.md`.
Start: `8cd884e9a` (batch 4d's tip, 225 REPLAY commits, tree clean). HEAD: `97987f74f` (#233).

**INCOMPLETE — STOPPED mid-#234.** #226–#233 committed and green. #234's cherry-pick and hand-fix
are staged, uncommitted, left as evidence. #234–#240 NOT replayed.

## The steps reached

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
| 234 | `057f9d494` | **NOT COMMITTED** | code, 2 `.rs` | **STOP-11 — unrelated `kind(lib)` red** |
| 235–240 | | | | **NOT REACHED** |

`scripts/replay/verify-step-record.sh 8cd884e9a HEAD 226 233`:

```
step-range: #226..#233 each present exactly once, sources match
step-record: complete
```
(exit 0 — the range actually landed. The brief's own closing command,
`verify-step-record.sh 8cd884e9a HEAD 226 240`, is **not run**: #234–#240 do not exist as commits and
that invocation would legitimately report `MISSING-STEP` for all seven — expected, not a defect.)

## EXPECTATIONS (EXPECTATIONS-7e's E1–E16), scored against the completed #226–#233

| # | result |
|---|---|
| E1 | **UNMET for the full range.** `verify-step-record.sh 8cd884e9a HEAD 226 233` → exit 0, range check passes for the 8 steps actually landed. The brief's own closing invocation (`226 240`) was not run because #234–#240 do not exist. |
| E2 | **PASS** for the docs-only steps reached: `git show --name-only` on #227 `77f568bcd`, #228 `a8266de5a`, #229 `29c8b5ced`, #231 `24887777c`, #232 `3bf9a1064` — every path under `docs/` and ending `.md`. #235–#237, #239, #240 not reached. |
| E3 | **PASS for #233** — `census`/`nested-program-gate` present, `lint-subset`/`kind(lib)`/`doctest` absent from its body (verified by `verify-step-record.sh`'s own derivation, which requires exactly this and passed). |
| E4 | **PASS.** #226's body names `wat/rete/syntax.wat` and the `cargo build --release` rebuild between phases; no `UNREGISTERABLE wat/` anywhere in either convert.sh log. |
| E5 | **PASS.** convert.sh's output for `wat/rete/syntax.wat` at both `C^` and `C` was read in full: no divergent-macro report fired for this step (the G1 class exists for the file's `defquery` macro per `future-macro-changes.txt`, but C's diff does not touch `defquery`'s body — only `with-network`/`with-overlay`'s comments and let-body move — so the class did not fire here; confirmed by reading the log, not assumed from the file's history). |
| E6 | **PASS.** Every non-`wat/` `.wat` this batch produced (`tests/rete/probe_arc278_export.wat` at #233) → `--check` rc=0. |
| E7 | **PASS, disproved not faked.** `wat/rete/syntax.wat` pre-step vs post-step, identical path shape (`/tmp/check226-{pre,post}/syntax.wat`): both rc=1, both name the SAME 5 `ReservedPrefix` defns (`query-read`, `make-rule`, `make-query`, `with-network`, `with-overlay`) — delta 0, matching the diff (an existing defn's body changed; 0 new top-level `:wat::` defns added). |
| E8 | **PASS for #226/#230.** Named tests: #226 3/3, #230 4/4 (N > 0 both). #233 has no `.rs`, so no named test applies (correctly noted, not skipped silently). #234's 7 named tests also ran and passed (7/7) before the unrelated `kind(lib)` red stopped the step short of commit. |
| E9 | **Self-measured, not the orchestrator's checkpoint run** (`floor.sh`/clippy explicitly reserved and NOT run by me, per the brief's ⛔). `kind(lib)` and `lint-subset`/`doctest` WERE run per-step (required walls, not reserved) — see E10. |
| E10 | **PARTIALLY MET, and this is where the batch stopped.** Predicted-then-measured deltas held for #226 (+3 `#[test]` in `kind(lib)`, 1478→1481, after fixing a REAL registry-completeness gap the diff itself created — see below) and #230 (+0 in `kind(lib)`, the new tests are an integration binary). At #234, `kind(lib)` went RED on `rete::kernel::tests::gather_probe_cost::probe_extend_cost_split` — a test #234's diff does not touch, in a file (`gather_probe_cost.rs`) introduced by grok and replayed at #183/#184, well outside this batch's blast radius. **Not dismissed as pre-existing/unrelated** (both are explicitly barred dispositions) — captured verbatim, named exactly, and the step was left uncommitted. See "STOP" below and REPLAY-LOG's #234 section for the full block. |
| E11 | **Not reached** — requires the orchestrator's own spot re-run at a code step past where this batch stopped; #226/#230's own numbers are self-measured and reported above (E10). |
| E12 | **PASS for #226–#233.** `git diff --name-only 8cd884e9a..HEAD` touches only the 8 committed steps' own claimed files plus this directory's docs; no `wat-scripts/fixes/` edit; `wat/rete/syntax.wat` IS expected (#226, per the brief); no `absent-on-main.tsv` row in `226..233`. |
| E13 | **PASS, vacuously — no repair commit exists.** Nothing to check; #234 was never committed, so there is no possibility of a knowingly-red commit past it. |
| E14 | **PASS.** Every `.census/…txt` file cited in #226/#230/#233's bodies exists on disk: `2026-09-16T06-44-34Z.txt`, `2026-09-16T06-53-48Z.txt`, `2026-09-16T06-58-21Z.txt` (all 2107 files), plus the pre-#226 baseline `2026-09-16T06-13-01Z.txt` cited as the diff predecessor. |
| E15 | **PASS.** `git replace -l` → 0 entries. No overlay of any kind was used anywhere in this batch. |
| E16 | **PASS.** `git log --format=%b` over `8cd884e9a..HEAD`, each verdict pattern checked: every `census:`/`nested-program-gate:`/`lint-subset:`/`kind(lib):`/`doctest:` line in #226/#230/#233's bodies matches on ONE line (verified directly, shown in REPLAY-LOG's checkpoint). |

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

## #227–#232 — docs-only

Five steps, every one `git cherry-pick -x --no-commit` + hand-composed
`REPLAY(grok-rete #N): <C's subject>` + trailer (finding 26/31's rule, never a bare `-x`). #228 and
#232 auto-merged one hunk each against local drift, conflict-free.

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

## STOP at #234 — an in-crate wall red, unrelated to the step, not dismissed

`fix(rete): an argument with no parameter is refused, not placed`. Both files (`eval.rs`,
`probe_arc278_export.rs`) auto-merged clean, matching C's diff exactly. One wat-in-`.rs`-string
hand-fix logged: `synthetic_user_fence`'s `rete_op_index(":wat::rete::core::i64::<")` lookup used a
retired name (corpus already renamed to `:wat::rete::i64::<`); fixed at
`tests/rete/probe_arc278_export.rs:910`. All 7 named tests then passed (confirming the fix — a wrong
name would have panicked the lookup helper itself, not merely mis-asserted): 7/7. census (2107 files,
unchanged) and nested-program-gate (3/3) both clean.

`cargo nextest run --release -E 'kind(lib)'` then failed:

```
FAIL [   0.171s] ( 636/1481) wat rete::kernel::tests::gather_probe_cost::probe_extend_cost_split
thread 'rete::kernel::tests::gather_probe_cost::probe_extend_cost_split' (500797) panicked at
src/rete/kernel/tests/gather_probe_cost.rs:887:5:
combined (34 ns) is far below its parts b+m+e (82 ns) — the combined closure is no longer doing
the work the parts describe
```

**Exact assertion:** `h >= (b + m + e) * 0.5` at `gather_probe_cost.rs:887`, a nanosecond wall-clock
apportionment check the test's own comment already calls a "loose bound (0.5x–2x) because these are
wall clocks." This is finding 28's exact CLASS — the SAME shape as the `accum_alpha_class_lookup_split`
ratio floor grok itself struck at #202 (folded into #190, two batches ago) — but a DIFFERENT test, in
a file `#234` does not touch. `gather_probe_cost.rs` was introduced by grok and replayed at
#183/#184, entirely outside this batch's (`#226`–`#240`) blast radius. The identical `kind(lib)`
invocation (1481 tests) passed 1481/1481 twice already in this session — at #226 (after the registry
fix) and at #230 — circumstantial evidence this is nextest's own internal parallelism producing
contention on a nanosecond-scale timing gate, not a defect #234 introduced.

**Disposition: not dismissed, not re-run, not committed.** The doctrine is explicit that
"pre-existing"/"unrelated to my change" are NOT dispositions, so this is reported as a live finding
rather than waved through. Per the anti-re-run rule, the test was captured on its one and only run
(verbatim above), the exact arm named, and #234 was **not committed** — it remains staged, uncommitted,
in the working tree as the preserved failing evidence (finding 28's own precedent: "on the preserved
failing tree"). No fold applies: no step in `#226`–`#240` touches `gather_probe_cost.rs`, so there is
nothing in this batch's blast radius to fold the defect into.

## Fold rule

No fold performed. #226's `REGISTRY_MEMBERSHIP_GAP_A` fix was landed INSIDE #226 itself (the step
whose own diff created the gap), not folded in after the fact — this is the correct order per
finding 20's own lesson, not a fold. The `kind(lib)` red at #234 has no step in this batch to fold
into (the failing file is outside the blast radius) — see "STOP" above.

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
- **The #234 `kind(lib)` red's scope.** Determined it was outside this batch's blast radius by (a)
  checking `git blame`/`git log -S` on the failing test to confirm it predates `#226` by many steps,
  and (b) confirming #234's own diff does not touch the failing file — both checks are static (no
  re-running the failing gate) and support NOT folding this into #234.

## Blast radius

The replayed commits' own files (`wat/rete/syntax.wat`, `src/check.rs`, `src/intrinsic/mod.rs`,
`src/rete/kernel/arm.rs`, `src/rete/kernel/tests/arm_lease.rs`, `src/rete/purity.rs`,
`src/runtime.rs`, `src/rete/export.rs`, `tests/rete/probe_arc278_export.{rs,wat}`,
`docs/arc/2026/06/278-rules-engine/**`), plus this directory's SCORE and REPLAY-LOG. No
`wat-scripts/fixes/` edit. No ref other than `replay/grok-rete` moved; no `refs/replace/` entries.
`#234`'s staged-but-uncommitted `src/rete/expr_ir/eval.rs` and `tests/rete/probe_arc278_export.rs`
are also in the working tree, preserved as evidence — not committed, not part of the blast radius
claim above.

## STOP

**STOP-11 at #234**: `cargo nextest run --release -E 'kind(lib)'` red on
`rete::kernel::tests::gather_probe_cost::probe_extend_cost_split`, a test unrelated to #234's own
diff, in a file outside this batch's blast radius — full verbatim block above and in REPLAY-LOG.
Do not push. Main untouched. `~/work/holon/` untouched. No subagents spawned. No worktrees used.
**Tree is NOT clean at yield** — `git status --porcelain` shows #234's cherry-picked + hand-fixed,
uncommitted changes (`M src/rete/expr_ir/eval.rs`, `M tests/rete/probe_arc278_export.rs`), left in
place deliberately as the reproducible failing evidence.
