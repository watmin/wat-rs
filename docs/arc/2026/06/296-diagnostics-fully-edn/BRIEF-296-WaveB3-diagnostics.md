# BRIEF — 296 Wave B3: `tests/diagnostics` (18)

> Read `CAMPAIGN-the-recapture-cascade.md` — **its LAW governs.** Then read
> `SCORE-296-WaveB2-wat_lang.md`, including its appended `★ ADJUDICATION OF THE 9 FINDINGS` — it is the
> worked example of how a batch gets dispositioned, and it names the sub-classes you may meet.

## Baseline — re-verify it, do not trust it

```
HEAD = d714fc98 (== origin/main), tree CLEAN, floor GREEN
  Summary [ 191.885s] 4624 tests run: 4624 passed (2 slow), 82 skipped
clippy 0.   296-pending ignores: 43.   This brief takes 18 of them.
```

## The cohort — 18 items across SIX files, not eighteen

```
7  probe_diagnostic_value_snapshot_in_errors.rs      ← the hazard file, see ⛔ below
6  probe_arc241_stone10_remedy.rs                    ← the other hazard file, see ⛔ below
2  probe_arc243_stone6_checkerror_pattern_a.rs
1  probe_arc296_raise_gate.rs
1  probe_arc243_stone7c_runtimeerror_pattern_a.rs
1  probe_arc242_stone2_value_position_doctrine.rs
```

**Re-verify against the disk**: `grep -c '296-recapture-pending' tests/diagnostics/*.rs`. Every count in
this arc's briefs has been wrong at least once — including one where files were counted as items.

**Good news on the ground:** `tests/diagnostics` already holds **115** `.edn` goldens and **16** files
using `assert_edn_matches_file!`. The idiom is native here; you are joining a convention, not inventing one.

## THE LAW — one test at a time

`UPDATE_EDN=1` writes whatever the code currently emits. The inline literal **is** the old expectation, so
converting-and-capturing without reading freezes whatever arrives.

**Per test: un-ignore → run WITHOUT `UPDATE_EDN` → adjudicate → capture only expected-staleness.**

### The adjudication vocabulary

**EXPECTED STALENESS → convert + capture:** same error count and order · every span in the user's `.wat`
identical (EDN spells `end_line`/`end_col` as `:end #wat.core.Option/Some [#wat.core/Pos {…}]` — same
numbers, different spelling) · kinds map 1:1 · every payload field present with the same value · EDN adds
`:message`/`:causes`/`:location` — additive.

**FINDING → report, do NOT capture:** an error disappeared · an error appeared that is not additive ·
**a span moved in the user's `.wat`** · a payload field lost its value · `field-0`/`field-N` anywhere ·
a populated `:remedies` collapsed to `[]`.

**SUPERSEDED → retire or rewrite, do NOT capture:** the assertion pins a design a later arc deliberately
replaced. **Before calling an absent error a defect, `grep -rl` its subject across `docs/arc/` and read any
later arc that names it.** That is one command. Skipping it is how a deliberate supersession got labelled a
security hole in this campaign.

Prose drift inside a `:reason`/`:message` is **not** a finding. Judge structure and values.

**An internal `src/*.rs` span that moved is STALENESS — recapture it and KEEP PINNING IT.** Builder ruling:
a pinned line updated when the line moves is in a constant state of correctness, while a dropped field is
permanently blind — and the span discriminates *which call site* raised the error.

## ⛔ TWO HAZARDS, and they sit in the two biggest files

### 1. `probe_diagnostic_value_snapshot_in_errors.rs` (7) — `field-N` must NOT be captured

`src/edn_shim.rs:3572` still reads:

```rust
pub fn value_to_edn_with(v: &Value, types: Option<&crate::types::TypeEnv>) -> OwnedValue
```

**A caller that passes `None` renders a record POSITIONALLY as `{:field-0 … :field-1 …}` instead of by its
real field names.** That is a live, documented defect — `NOTE-value-to-edn-renders-fields-positionally.md`.
This file captures **value snapshots embedded in errors**, which is exactly the path that reaches that door.

**Measured this session: there are currently ZERO `field-0`/`field-N` occurrences in any
`tests/diagnostics/*.edn`. Your job includes keeping that number at zero.**

If any snapshot renders `field-N`, that is a **FINDING** — report it with the verbatim golden fragment and
which test produced it. **Do not capture it.** Capturing would freeze a known defect into a golden and make
it read as ground truth, which is precisely the trap Wave B2's findings 1 and 8 turned out to be.

### 2. `probe_arc241_stone10_remedy.rs` (6) — remedies collapsing to `[]`

**Read `DESIGN-296-remediation-collapse.md` BEFORE touching this file.** A populated `:remedies` that has
become `[]` is a **FINDING**, not staleness — a remedy is user-facing repair guidance, and losing it is a
silent capability regression that a green test would happily bless.

## The conversion

```rust
assert_eq!(err, r#"Check(CheckErrors([…]))"#);                                   // before
wat::assert_edn_matches_file!(err, "<stem>__<fn_name>.edn", "<what it pins>");   // after
```

Golden co-located, named `<source_file_stem>__<test_fn_name>.edn`. Capture with `UPDATE_EDN=1` **after**
adjudicating, then re-run **without** it to prove the golden matches. Worked references in this very
directory: any of the 16 files already using the idiom, plus Wave B2's `tests/wat_lang/*.edn`.

A test may need small mechanism changes beyond a straight assert swap — B2 hit `Debug` of an
`Option<StartupError>` (not valid EDN — use `.expect_err(...)` first) and a redundant
`Display + "---" + Debug` concatenation. Those are fine; **report them as deltas.**

## Capture-once

```
cargo nextest run --release -E 'binary(diagnostics)' --run-ignored all --no-capture > /tmp/wb3.log 2>&1
```

Run once, grep the file. `--run-ignored all` sweeps **every** ignored test in that binary, not just this
cohort — batch 1 hit an arc-255 test that is **RED BY DESIGN**. Check a stranger's ignore reason before
treating it as yours.

## STOP triggers — these REJECT; none is permission to ship less

- **STOP-1 — an adjudication you cannot place** in any column. Capture the output verbatim and report; do
  not capture the golden to move on.
- **STOP-2 — more than ~6 findings**, or a fixture that exercises nothing. `3cd00fbb` hollowed nine
  fixtures by deleting the `main` that drove them: **a positive fixture fails by passing** — a `.wat` that
  lost its driver still loads clean and its `is_ok()` passes. Report and stop; the orchestrator re-plans.
- **STOP-3 — the work needs a `src/` or `.wat` corpus change.** Findings, not licence. A `.wat` corpus
  migration is a wat-fix codemod (R21), never a hand edit.
- **STOP-4 — you are tempted to re-`#[ignore]` something to reach green.** Never. A test you cannot
  adjudicate is reported still-failing.
- **STOP-5 — a red you did not intend. Do NOT re-run** — a re-run that goes green destroys the only
  evidence. `scripts/floor.sh` keeps the untruncated log: copy the failing test's whole stdout+stderr block
  **verbatim** (never `| head`, never a summary), name the exact assertion that fired, and report.
  **There is no such thing as a known flake.**

## Blast radius

`tests/diagnostics/*.rs` + new `tests/diagnostics/*.edn`. **No `src/`. No `.wat` corpus changes.** Do not
touch the 115 existing goldens, Wave B2's captures, or any non-296 ignored test.

## Verify — in this order; read the Summary line, never a piped exit code

```
cargo build --release --tests
cargo clippy --workspace --all-targets --release -- -D warnings      # must be 0
scripts/floor.sh
```

| | before | after (if all 18 are staleness) |
|---|---|---|
| tests run | 4624 | **4642** (+18) |
| passed | 4624 | **4642** |
| failed | 0 | **0** |
| skipped | 82 | **64** (−18) |
| 296-pending | 43 | **25** |

If findings block some, say exactly which and leave them failing — **do not re-ignore them.**

## Negative controls — keep the keepable ones

Standing rule (`docs/DUNGEON-CRAWL.md` Phase 3): for each control, **is it keepable?** If yes, bank it as a
test. If it needs an `src/` mutation, report it with the reason. Discarding is a declared exception with a
stated reason, never the default.

## How to work

You are a **rider, not the orchestrator. Ending your turn ENDS you** — nothing wakes you, no notification
is coming. ⛔ **Run every build and test in the FOREGROUND and block on it.** No `run_in_background`, no
Monitor, no polling-then-stopping. Four riders on these arcs died exactly that way.

Anchor at `/home/watmin/work/holon/wat-rs`; `pwd` first. **Leave the work uncommitted.** Never `git commit`,
`push`, `stash`, `revert`, or `checkout --` — `stash@{0}` holds unrelated work.

## Report

- the full adjudication table for the 18, one row per test, with its column
- every finding **verbatim**, with the exact field that moved
- **the `field-N` count in `tests/diagnostics/*.edn` after your work** — it must still be zero
- **the `:remedies` status of all 6 remedy tests** — populated or collapsed
- any hollow fixture found, and what its test was failing to exercise
- negative controls: which you kept as tests, which you did not and why
- clippy count and the floor **Summary line, verbatim**, with the arithmetic
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.** Re-measure what you act
  on and say plainly where I was wrong.
