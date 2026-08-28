# STONE P3 — the arc's own ignore ledger is re-diagnosed

> Row P3 of `WORKLIST-open-stones.md`. Finding **3** of
> `NOTE-an-absence-recorded-as-an-answer-…md` named three of these; **the real population is seven**,
> measured below. The 255 seam calls them *"the worklist, written by a prior self as the unlock
> condition"*.

## The work

Seven `#[ignore]`s name arc 255 as their blocker. **Every one of them carries an unlock condition
that has already fired** — *"unlock when we circle back to arc 255"* — and we circled back weeks ago.
One of them is hiding a test that **passes**. Another fails for a reason its text does not describe.

Measured this session with `cargo nextest run --release --run-ignored all`:

| # | test | state | the reason it CARRIES | the reason that is TRUE |
|---|---|---|---|---|
| 1 | `probe_arc255_reflection_parity::metadata_of_answers_for_a_rust_builtin` | **PASSES** | *"metadata-of reflection not yet built"* | it IS built — `(metadata-of :wat::i64::+)` answers `Some` |
| 2 | `probe_arc255_reflection_parity::user_form_carries_guaranteed_baseline` | fails | same string | a bare user `defn` returns `None`; the guaranteed baseline was never built for the **user** branch |
| 3 | `probe_arc255_ivc_metadata_plain_values::metadata_of_emits_plain_values_and_enums_not_holon_ast` | fails | same string | **the feature shipped to a DIFFERENT contract** — `metadata map missing key :pure`, because purity/determinism are emitted as `:purity`/`:determinism` **enums**, not `:pure`/`:deterministic` bools |
| 4 | `probe_arc255_ivb2b_verify_examples::verify_examples_reports_no_failures` | fails | *"RED, KNOWN, COUNTED: 5 failures / 1 cause"* | **stale**: it now panics before collecting anything — see the NOTE, three refuted fixes |
| 5 | `probe_diag_typealias_leniency::probe_undeclared_field_type_keyword_rejected_or_lenient` | fails | *"arc 255 banked gate … un-ignore when 255 makes them check errors"* | *(you determine it)* |
| 6 | `probe_undefined_builtin_resolves::wrong_operator_leaf_is_a_check_error` | fails | *"checker rejection of undefined builtins not …"* | **`walk.rs:268`, the arc's ENDGAME** — 2,539 tests fail if default-denied |
| 7 | `probe_undefined_builtin_resolves::bogus_leaf_under_known_namespace_is_a_check_error` | fails | same | same |

⚠ **Re-run all seven yourself before acting.** These numbers are one session old and rows 5's true
cause is deliberately left for you to measure — I did not, and I will not accept a disposition for it
that I cannot check.

## The three dispositions — the arc's own ruling, applied

The 255 seam already ruled this shape: *"THREE dispositions, not two — staleness (capture) · finding
(report) · SUPERSEDED (a later arc replaced the design; retire or rewrite)."* Give each of the seven
exactly one:

- **UN-IGNORE** — it passes. Delete the attribute; the floor gains a test. *(Row 1 at minimum.)*
- **REWRITE** — the feature shipped, to a different contract. The test asserts the OLD one and is
  now the stale artifact. Rewrite it to the shipped contract and un-ignore. *(Row 3: the enum
  contract is the ruled one — `src/intrinsic/mod.rs`'s own header carries a "CORRECTED 2026-08-25"
  note about it. Read that before rewriting.)*
- **RE-POINT** — genuinely blocked, but the STATED unlock is false. Rewrite the reason to name the
  **actual, checkable** blocker. *(Rows 2, 4, 6, 7, and probably 5.)*

⛔ **No row may keep the words "unlock when we circle back to arc 255".** That is a promise, not a
condition, and it has already come true on all seven. A reason must name something a reader can
CHECK: a line, a design ruling, a named stone, a measured count.

## ★ THE WALL — `tests/lint/ignore_reason_justified.rs`

Re-diagnosing seven reasons fixes seven instances. **The class is that a copy-pasted unlock condition
outlives the thing it was waiting for**, and nothing notices — this file's own
`probe_arc255_reflection_parity.rs:94-121` records that two SIBLING tests were deleted in August for
exactly this staleness, while the survivors kept the identical string, unrechecked, for three weeks.

The house already has this lint shape three times: `tests/lint/unused_span_justified.rs`,
`retired_name_justified.rs`, `span_substitution_justified.rs`. **Copy one.** The new lint asserts
every `#[ignore = "…"]` reason in the tree names a checkable condition — at minimum, that none
contains a "circle back / come back to / when we get to arc N" promise.

⚠ **FROZEN ALLOWLIST, identity = file + test fn name, NEVER a line number.** There are **14** real
`#[ignore]` attributes in the tree (anchored count — a naive grep says 68 because it catches the
phrase in doc comments). Seven are arc 255's and yours; the other seven belong to other arcs and are
**out of this stone's scope**. Allowlist them BY NAME so the residue is visible and countable, exactly
as `tests/lint/no_bare_is_err.rs` does. Do not widen the lint to fix them.

## Rooms — verified against `4577955d8`

```
tests/reflection/probe_arc255_reflection_parity.rs:70,:82     rows 1 and 2
tests/reflection/probe_arc255_reflection_parity.rs:94-121     the record of the two deleted siblings — READ IT
tests/reflection/probe_arc255_ivc_metadata_plain_values.rs:67 row 3
tests/reflection/probe_arc255_ivb2b_verify_examples.rs:65     row 4
tests/types/probe_diag_typealias_leniency.rs:16               row 5
tests/wat_lang/probe_undefined_builtin_resolves.rs:17,:40     rows 6 and 7 — the ENDGAME gate
src/intrinsic/mod.rs (header)                                 the "CORRECTED 2026-08-25" note row 3 needs
tests/lint/unused_span_justified.rs                           ★ the lint shape to copy
tests/lint/no_bare_is_err.rs                                  ★ the frozen-allowlist shape to copy
docs/arc/2026/06/296-diagnostics-fully-edn/IGNORE-LEDGER.md   the prior ledger, retired in place
docs/arc/2026/06/296-diagnostics-fully-edn/NOTE-the-doctest-runner-masks-every-failure-behind-one-raise.md
                                                              ⛔ row 4's THREE REFUTED FIXES — read
                                                              before proposing anything for it
```

## Blast radius

`tests/` only — seven ignore attributes/reasons, one rewritten test body, one new lint. **No `src/`
change.** If a disposition seems to require one, that is STOP-3.

## STOP triggers — each REJECTS. Ship nothing and report the gap.

1. **A row's measured state differs from the table above.** Report it; the table is one session old
   and a disagreement is a finding, not a licence to adjust.
2. **You cannot name a checkable blocker for a RE-POINT row.** Say so. *"Still blocked, cause
   unknown"* is an honest report and a fine outcome for one row; inventing a plausible cause is not.
3. **A disposition needs a `src/` change.** Then it is not a re-diagnosis, it is a stone. STOP and
   name what it would take.
4. **Row 4 tempts you to fix the doctest runner.** THREE fixes have been attempted and each was
   refuted by measurement. Re-point its reason at that NOTE; do not attempt a fourth.
5. **Rows 6/7 tempt you to touch `walk.rs:268`.** That is the arc's endgame, sized at 2,539 failing
   tests. Re-point the reason; change nothing.
6. **The lint goes red on an arc-255 row after your dispositions.** Then a reason you wrote is still
   a promise. Fix the reason, not the lint.

## Acceptance — run each, report the actual output

```
 0. ★ THE LEDGER, RE-MEASURED. `cargo nextest run --release --run-ignored all -E '<the seven>'`.
    Per row: pass/fail, and for each failure the ASSERTION TEXT verbatim — not a summary. Compare
    against the brief's table and report every disagreement.

 1. ★ THE FLOOR GAINS A TEST PER UN-IGNORE. Report the floor count before and after — accounted
    BY NAME (which test), never by arithmetic.

 2. ★ THE REWRITTEN TEST ASSERTS THE SHIPPED CONTRACT. For row 3, show the metadata map's ACTUAL
    keys/values (a scratch .wat under wat-scripts/scratch-pad/, `--check` clean) and show the new
    assertions match them. A rewrite that passes because it asserts less is not a rewrite.

 3. ★ EVERY SURVIVING REASON NAMES A CHECKABLE BLOCKER. Paste all seven final reason strings.
    None may contain "circle back". For each, one sentence on how a reader would check it.

 4. ★ THE LINT CAN GO RED. Add a throwaway `#[ignore = "unlock when we circle back to arc 255"]`
    to any test, show the lint FAILS and NAMES the offending file + test fn, remove it, show green.
    `NISI FRANGAS, NIHIL PROBAS.` Confirm the edit LANDED before reading its output.

 5. ★ THE ALLOWLIST IS BY NAME AND ITS SIZE IS REPORTED. How many non-arc-255 ignores are frozen,
    and by what identity. Never a line number.

 6. cargo build --release --all-targets — clean.

 7. cargo nextest run --release -E 'binary_id(wat::lint) + binary_id(wat::reflection)'
    Summary lines verbatim.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. Verify with `pwd` first. Any path containing
  `.claude/worktrees/` is harness state — never operate on it.
- Everything FOREGROUND. Ending your turn ENDS you; nothing wakes you. Land the numbers.
- You may run `cargo build`, `cargo nextest run --release -E '<filter>'` (including
  `--run-ignored all`), `./target/release/wat --check <file>` and `./target/release/wat <file>`.
  The orchestrator runs the full floor and clippy centrally — leave those two alone.
- You may not spawn sub-agents.
- **No `git stash`, in any form.** `git show HEAD:<path>` for a pre-image.
- Do not commit, push, revert, or create a worktree. Leave the tree dirty.

## Report back with

Row-by-row: the command, its actual output, PASS/FAIL. The seven dispositions with their reasoning.
Then the honest deltas. Every rider on this chain has caught a real defect in an orchestrator brief;
this one's table is one session old and row 5 is deliberately unmeasured — if it is wrong, say so.
