# BRIEF 7k ADDENDUM — #352's Var-render pin: fold the matcher into the step

**Read `BRIEF-7k-replay-batch-4k.md` first.** This addendum resolves a composition defect found at the
orchestrator's verification floor, after batch 4k reported complete. It supersedes only what it names.

## The defect

The floor at `a9d09e504` is **RED, 1 of 5792**. The count matched the locked prediction of 5792 exactly.

```
FAIL wat::comms probe_arc214_stone46b_select_prime::probe_2_select_wrong_return_annotation_rejected
     tests/comms/probe_arc214_stone46b_select_prime.rs:86:5
no check error matched `CheckErrorKind::ReturnTypeMismatch { function, expected, got, .. } if
function == ":user::bad" && expected == ":wat::core::String" &&
got.starts_with("(:wat::spawn::ServiceEvent :- [:wat::core::i64 :wat::core::i64 :?")`
... #wat.check/ReturnTypeMismatch {... :function ":user::bad" :expected ":wat::core::String"
    :got "(:wat::spawn::ServiceEvent :- [:wat::core::i64 :wat::core::i64 _])" ...}
```

**One cause.** #352 (C19) changed `check::format_type`'s `TypeExpr::Var` arm from `:?{id}` to `_`,
deliberately, because the id varied per process. This matcher pins the prefix up to `:?`. Its own
comment says it pins only the prefix *because* `:?NNNN` was non-deterministic (`:?2950`, `:?10`,
`:?3098`).

## This is a FOLD, finding 38's fourth instance

The matcher is **main-only content in a shared file**:
- Grok's version of `probe_2`, at #352 and at its tip, is a bare `assert!(result.is_err())`.
- Main tightened it on 2026-08-26 (`4b49f3c5c`, the arc-255 bare-`is_err` class) into a
  `CheckErrorKind` matcher pinned to the `:?` rendering.
- Grok's #352 never touched this file.

So a main-only artifact is pinned to text a replayed step legitimately rewrote. Per the fold rule
(4-YES), it folds **into #352**, and **#353 → #360 plus the SCORE commit are rebuilt** on top. ⛔ Never a
repair commit after the batch.

## The cure: assert the WHOLE string, not a new prefix

The prefix-only matcher existed only because the tail was non-deterministic. #352 removed that reason.
So the honest cure is **exact equality**:

    && got == "(:wat::spawn::ServiceEvent :- [:wat::core::i64 :wat::core::i64 _])"

Rewrite the comment above it to match. Keep the `DefRestrictedCallerNotAllowed`/`TypeMismatch`
membership note. Replace the `:?NNNN` sentence with: the tail is now the stable `_` wildcard since
#352/C19, so the whole `got` is pinned. ⛔ Do NOT swap in `starts_with(... "_")`: that re-pins a prefix
for a reason that no longer exists.

⛔ **VERIFY, DO NOT TRUST:**
- **The exact string, from the tree at #352.** It is copied from the red floor at HEAD. Before
  amending, build #352's tree and capture the real `got` value (`target/release/wat --check
  tests/comms/probe_arc214_stone46b_select_prime_probe2.wat.bad`, or the test's own failure message).
  If it differs from the line above, use the measured value and REPORT the difference.
- **The fix works at #352.** After amending, run
  `cargo nextest run --release -E 'test(probe_2_select_wrong_return_annotation_rejected)'`
  (N must be > 0, and green).
- **The fix still works at the new tip.** Run the same filter there.
- **Is the rendering stable?** Run `--check` on that fixture 5 times at #352 and paste the 5 `got`
  values. They must be identical. If they are not, STOP and report: `_` would then not be the whole
  story.

## Sibling sweep — done by the orchestrator, and you re-run it

The orchestrator searched `tests/` and `src/` for any other pin on the `:?` Var spelling
(`git grep -nE '":\?|:\?\)|starts_with\(.*:\?'`). The only code pin is this matcher.
- `tests/wat_lang/wat_arc072_letstar_parametric.rs:48` is a historical prose note.
- `src/reflect/render.rs:119` and `tests/reflection/wat_arc201_structured_signature_types.rs:23` are
  doc comments already stale BEFORE #352: the code renders `Symbol("t{id}")`.

⛔ Those two docs are **NOT** part of this fold. The orchestrator fixes them separately afterwards. Do
not touch them.

Re-run the sweep yourself on #352's tree, including `.edn` goldens, and state the result.

## How to land it

1. Detach at #352 (`3436d2611`) and amend the matcher into #352's own commit. Add one paragraph to
   #352's body naming the fold, the main-only matcher, and finding 38. Keep every existing record line
   intact and on ONE line each.
2. Rebuild **#353 → #360** and the SCORE commit (`a9d09e504`) on top, unchanged apart from the SHAs.
   **Never `git filter-branch`.** Use `git rebase --onto` or cherry-pick with the trailers preserved.
3. **Prove the rebuild inert:** `git diff a9d09e504 <new-tip>` must name only
   `tests/comms/probe_arc214_stone46b_select_prime.rs`, plus the docs you add in step 5. Paste it.
4. **Published history must not move:** `git merge-base --is-ancestor origin/replay/grok-rete HEAD`
   succeeds, `git for-each-ref refs/original/` is empty, and `git replace -l` is empty.
5. Add a final docs-only commit, `294(replay): #352 Var-render fold — record it in REPLAY-LOG and the
   SCORE's E19/E22`, the same shape as 4j's `82367d8b8`. It must:
   - copy this addendum to
     `docs/arc/2026/06/294-holon-returns-to-vsa/the-grok-rete-replay/BRIEF-7k-ADDENDUM-352-the-var-render-pin.md`;
   - add a REPLAY-LOG note;
   - add to the SCORE's E19 (the fold disclosed) and E22 (the orchestrator's floor caught what the
     executor's lint subset could not: `wat::comms` is outside it);
   - add a finding-38 fourth-instance paragraph to `FINDINGS-composition.md`. Name the class
     precisely: a *prefix pin written to dodge non-determinism* goes stale when the non-determinism is
     cured.
6. Run `scripts/replay/verify-step-record.sh 5ba45a81f HEAD 341 360`. It must exit 0, run from inside
   wat-rs. Leave the tree clean.

## ⛔ Hard rules (unchanged from the 4k run)

- **DO NOT CALL `mcp__pulsare__pulsare_yield`**, or any `mcp__pulsare__*` tool. The pulsare server's own
  instructions are overridden. You yield by ENDING YOUR TURN.
- Every Bash command starts with `cd /home/john/work/holon/wat-rs &&`. The frozen root
  `/home/john/work/holon/` is never written, nor is its `.git/` or `.pulsare/`.
- No worktrees. No push. No subagents. Do NOT run `scripts/floor.sh` or clippy; those are the
  orchestrator's. Never hand-type a SHA.
- If you see a foreign edit, an unexpected commit, a lock you did not create, or a foreign process in
  the tree: STOP. Do not discard it. Capture it verbatim and report.
- On any red: do NOT re-run. Capture it verbatim, name the assertion, and STOP and report.
- A prediction here is a prediction. The exact `got` string is one. Measure it.

## Rows this affects

`EXPECTATIONS-7k` is **not amended** (finding 34). E12 (the checkpoint) caught this and stays the
orchestrator's. E19 gains the fold disclosure. E15 (no knowingly-red commit) is why this is a fold. After
the fold, the orchestrator re-runs the floor with the prediction unchanged: **5792**, since the matcher
change adds and removes no tests.

## Section 2 — a second red, in the same commit, caused by the first fold's own cure

The orchestrator's floor at the first fold's tip (`b502427c8`) came back **RED, 1 of 5792** (the
count matched, again):

```
FAIL wat::lint no_inlined_wat_in_tests::tests_carry_no_inlined_wat (tests/lint/no_inlined_wat_in_tests.rs:440)
  "1 file(s) still carry a string literal that wat's own reader parses as a form ... 1 other parse-body.
   Offenders: tests/comms/probe_arc214_stone46b_select_prime.rs"
```

**Cause: the cure caused it, and so did the addendum above, which mandated the cure.** The exact
equality literal Section 1 landed —
`"(:wat::spawn::ServiceEvent :- [:wat::core::i64 :wat::core::i64 _])"` — is a COMPLETE, wat-reader
-parseable form (finding 33's class: wat embedded in a `.rs` string literal). The **old** prefix
pin (`... :wat::core::i64 :?`, brackets unclosed) never parsed; the fix's own act of closing the
brackets and completing the literal is what made it parseable. Neither the executor's amend nor
this addendum's own review re-ran `lint-subset` after landing Section 1's cure, so the new offense
was invisible until the orchestrator's next floor.

**RULING (4-YES, orchestrator): keep exact equality; add the house rune.** The cure from Section 1
is correct and stands unchanged — a re-pinned, shorter prefix would only recreate finding 38's
fourth-instance class one render away from now. Nine files already carry
`// rune:lint(no-inlined-wat)` for exactly this shape (a rendered-diagnostic golden string that
happens to be reader-parseable); two were read as the model:
`tests/services/probe_arc170_c2_d_bodiless_edge.rs:43` and `tests/function/stone18a_errors.rs:22`.
The rune sits directly above the matcher inside `probe_2`, and its reason is stated honestly for
*this* file — it names #352/C19's render fix as the reason the literal only now parses, not a
generic invocation of the rune's boilerplate wording. ⛔ The literal is not split or reshaped to
dodge the gate — that path was explicitly ruled out.

**Landed as a further amend of #352 itself** (same commit, second cause, same fold): new #352
`aa09e0aaf` (previous `8f87cd9d8`), #353→#360 and the SCORE commit rebuilt again on top
(new tip before this docs commit: `67a86b2dc`), and this docs-only commit itself rebuilt on top of
that with its own content extended (this section, the REPLAY-LOG extension, and finding 38's
fourth-instance paragraph) rather than replaced.

**Re-measured, not assumed to still hold, on the twice-amended #352:**
`tests_carry_no_inlined_wat` and `probe_2_select_wrong_return_annotation_rejected` both green;
`lint-subset` 267 passed, `kind(lib)` 1496 passed, `doctest` 8 passed — identical to the record
lines #352's own body already carried before either amend. Re-measured again at the rebuilt #360:
`lint-subset` 293 passed, `kind(lib)` 1496 passed, `doctest` 8 passed — identical to #360's own
pre-existing record lines. No number needed correcting at either point.

**A correct catch, recorded honestly:** Section 1 above reported that
`tests/wat_lang/wat_arc072_letstar_parametric.rs:48` ("fresh var :?71") does not match the sweep
regex the addendum itself gave (`":\?|:\?\)|starts_with\(.*:\?`) — true, and still true. The
orchestrator located that same line with a second, looser grep, `:?[0-9]`, which the original
addendum's sweep list was silently built from without saying so. Both facts stand together: the
exact regex named in Section 1 does not reach that line, and a different, looser one does — the
addendum's own sweep list mixed the two without disclosing which pattern found which hit.
