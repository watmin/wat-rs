# BRIEF — 296 Wave B4: `tests/function` (8)

> `CAMPAIGN-the-recapture-cascade.md` governs. Read `SCORE-296-WaveB2-wat_lang.md` **including its
> appended `★ ADJUDICATION OF THE 9 FINDINGS`** — it is the worked example, and it names the
> sub-classes you will meet here.

## Baseline — re-verify, do not trust

```
HEAD = 59ee1f06, tree CLEAN, floor GREEN, clippy 0
  Summary [ 202.768s] 4648 tests run: 4648 passed (3 slow), 64 skipped
296-pending: 25.  This brief takes 8.
```

## The cohort — 8 items across FOUR files

```
4  fn_rename.rs            ← ⛔ THE HAZARD FILE, see below
2  fn_signature.rs         fn_body_type_mismatch_surfaces · malformed_args_vector_clear_error
1  defn.rs                 defn_body_type_mismatch_surfaces
1  recursive_patterns.rs   nonexhaustive_partial_pattern_rejected
```

Re-verify: `grep -c '296-recapture-pending' tests/function/*.rs`.

## ⚠ THIS DIRECTORY IS NOT YET CONVERTED — you are introducing the idiom

Unlike `tests/diagnostics` (115 goldens, 16 files already using `assert_edn_matches_file!`),
`tests/function` has **1** golden and **1** file using it. You are establishing the convention here,
not joining it. Worked references live elsewhere: `tests/types/enums.rs`,
`tests/process/probe_supervisor_select_lost.rs`, and Wave B3's `tests/diagnostics/*.edn`.

Expect to need small mechanism changes beyond a straight assert swap. B2 hit `Debug` of an
`Option<StartupError>` (not valid EDN — `.expect_err(...)` first) and a redundant
`Display + "---" + Debug` concatenation. **Report each one as a delta.**

## ⛔ THE HAZARD — `fn_rename.rs` (4 of the 8) reeks of SUPERSESSION

Every one of its four asserts that a **retired form silently aliases**:

```
lambda_post_retirement_silently_aliases_to_fn
bare_fn_type_post_retirement_walker_silent
multiple_lambda_sites_post_retirement_silently_alias
both_legacy_walkers_retired_silently_alias
```

Arc 155 retired `:wat::core::lambda`. Arc 241's stones 11–16 then performed **HARD CUTS** on a family
of retired forms (`define`, `defalias`, `Option/expect`…) — a hard cut *rejects*, it does not silently
alias. **If the retirement policy moved from silent-alias to hard-cut for `lambda` or the bare `fn`
type, these four are SUPERSEDED, not stale** — and recapturing them would freeze a policy we
deliberately replaced.

**Do the one-command check before adjudicating any of the four:**

```
grep -rl "lambda" docs/arc/ | xargs grep -l "retire\|hard cut\|hard-cut" | sort
```

Read any arc later than 155 that names the subject. This is the move that costs one command, and
skipping it is how a deliberate supersession got labelled a security hole in this campaign — and how
Wave B3 nearly recaptured a test whose design arc 298 had annihilated.

**A test asserting a retired policy is ORPHANED, not stale.** Its disposition is retire-or-rewrite
against the superseding stone, never recapture.

## THE LAW — one test at a time

`UPDATE_EDN=1` writes whatever the code currently emits. The inline literal **is** the old
expectation. **Per test: un-ignore → run WITHOUT `UPDATE_EDN` → adjudicate → capture only
expected-staleness.**

**STALENESS → capture:** same error count and order · every span in the user's `.wat` identical (EDN
spells `end_line`/`end_col` as `:end #wat.core.Option/Some [#wat.core/Pos {…}]` — same numbers,
different spelling) · kinds map 1:1 · every payload field present with the same value · EDN adds
`:message`/`:causes`/`:location` — additive.

**FINDING → report, do NOT capture:** an error disappeared · a non-additive error appeared · **a span
moved in the user's `.wat`** · a payload field lost its value · `field-0`/`field-N` anywhere ·
a populated `:remedies` collapsed to `[]`.

**SUPERSEDED → retire or rewrite, do NOT capture.** See the hazard above.

Prose drift inside a `:reason`/`:message` is **not** a finding. Judge structure and values.

**An internal `src/*.rs` span that moved is STALENESS — recapture it and KEEP PINNING IT.** Builder
ruling: a pinned line updated when the line moves is in a constant state of correctness, and the span
discriminates *which call site* raised the error. **Seven goldens in the tree pin such a span today;
five needed updating in the last migration alone** — so this is routine, not alarming.

### The sub-class B2 had to mint

Two of B2's "findings" were neither stale nor superseded: **the golden pinned a WRONG value, so the
fix looks like a regression.** These carry a heavier burden — you must *prove the old value was
wrong* (B2's were a span landing mid-token, and an error span identical to its own
`original_def_span` in a file that didn't contain the symbol). If you meet one, prove it or report it;
do not wave it through as staleness.

## The conversion

```rust
assert_eq!(err, r#"Check(CheckErrors([…]))"#);                                   // before
wat::assert_edn_matches_file!(err, "<stem>__<fn_name>.edn", "<what it pins>");   // after
```

Golden co-located, `<source_file_stem>__<test_fn_name>.edn`. Capture with `UPDATE_EDN=1` **after**
adjudicating, then re-run **without** it to prove the golden matches.

⚠ **A golden must not embed an absolute host path.** B3 caught one: `rust_caller_span!()` produced
`/home/watmin/work/holon/wat-rs/...`, the only absolute path among that directory's goldens, and
therefore non-portable. If a value carries one, that is a finding — say so rather than capturing it.

## Capture-once

```
cargo nextest run --release -E 'binary(function)' --run-ignored all --no-capture > /tmp/wb4.log 2>&1
```

Run once, grep the file. `--run-ignored all` sweeps **every** ignored test in that binary — batch 1
hit an arc-255 test that is **RED BY DESIGN**. Check a stranger's ignore reason before adopting it.

## STOP triggers — REJECTION criteria, never permission to ship less

- **STOP-1 — an adjudication you cannot place** in any column. Capture verbatim, report; do not
  capture the golden to move on.
- **STOP-2 — more than ~6 findings**, or a fixture that exercises nothing. `3cd00fbb` hollowed nine
  fixtures by deleting the `main` that drove them: **a positive fixture fails by passing** — a `.wat`
  that lost its driver loads clean and its `is_ok()` passes. Report and stop.
- **STOP-3 — the work needs a `src/` or `.wat` corpus change.** Findings, not licence. A corpus
  migration is a wat-fix codemod (R21), never a hand edit.
- **STOP-4 — you are tempted to re-`#[ignore]` something to reach green.** Never.
- **STOP-5 — a red you did not intend. Do NOT re-run** — a re-run that goes green destroys the only
  evidence. `scripts/floor.sh` keeps the untruncated log: copy the failing test's whole stdout+stderr
  block **verbatim**, name the exact assertion, report. **There is no such thing as a known flake.**

## Blast radius

`tests/function/*.rs` + new `tests/function/*.edn`. **No `src/`. No `.wat` corpus changes.** Do not
touch the existing golden, other waves' captures, or any non-296 ignored test.

⚠ **Scope your `UPDATE_EDN` filter by FUNCTION, not by file.** B3 used a file-wide filter and swept a
pre-existing non-cohort golden into a reformat — a blast-radius violation it caught by diffing
`git status`. Check `git status` before you finish and account for every changed file.

## Verify — in this order; read the Summary line, never a piped exit code

```
cargo build --release --tests
cargo clippy --workspace --all-targets --release -- -D warnings      # must be 0
scripts/floor.sh
```

| | before | after (if all 8 capture) |
|---|---|---|
| tests run | 4648 | **4656** |
| passed | 4648 | **4656** |
| failed | 0 | **0** |
| skipped | 64 | **56** |
| 296-pending | 25 | **17** |

A superseded test that gets rewritten still nets one fewer ignore and one more passing test, so the
table holds either way. If findings block some, leave them **failing** and name them.

## Negative controls

Standing rule (`docs/DUNGEON-CRAWL.md` Phase 3): for each control, **is it keepable?** If yes, bank it
as a test. If it needs an `src/` mutation, report it with the reason. Discarding is a declared
exception with a stated reason, never the default.

## How to work

You are a **rider, not the orchestrator. Ending your turn ENDS you** — nothing wakes you, no
notification is coming. ⛔ **Run every build and test in the FOREGROUND and block on it.** No
`run_in_background`, no Monitor, no polling-then-stopping. Five riders on these arcs have now died
exactly that way, the most recent one today.

Anchor at `/home/watmin/work/holon/wat-rs`; `pwd` first. **Leave the work uncommitted.** Never
`git commit`, `push`, `stash`, `revert`, or `checkout --`.

## Report

- the 8-row adjudication table, one row per test, with its column
- **for each of `fn_rename.rs`'s four: the arc search you ran and what it returned.** If they are
  superseded, name the stone; if they are not, say what you read that rules it out
- every finding **verbatim**, with the exact field that moved
- any hollow fixture, and what its test was failing to exercise
- negative controls kept or not, and why
- `git status` accounted for, file by file
- clippy count and the floor **Summary line verbatim**, with the arithmetic
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.** Re-measure what you
  act on and say plainly where I was wrong.
