# BRIEF — 296 Wave B2 CAPTURE: the adjudication is done; convert and capture

> **The judgment already happened. This brief is mechanism, not judgment.**
> Governing record: `SCORE-296-WaveB2-wat_lang.md` — the recovered 40-row table **plus** the
> `★ ADJUDICATION OF THE 9 FINDINGS` section appended to it. Read both parts.

## State you are walking into

```
HEAD = 19aab903 (== origin/main), and main is GREEN (4584 run / 4584 passed / 122 skipped)
The CHECKOUT at /home/watmin/work/holon/wat-rs has 40 uncommitted #[ignore] deletions across 19 files.
Those 40 tests therefore RUN, and all 40 currently FAIL. That is expected and it is your worklist.
Floor with those un-ignores in place: 4624 run / 4584 passed / 40 failed / 82 skipped.
```

Those 40 line-deletions are **the parked Wave B2 work — do not revert them, do not re-add any `#[ignore]`.**

## The work

**39 tests: convert the inline expectation to an `.edn` golden and capture it.**
**1 test: rewrite it against the superseding stone.** That is the whole job.

### The 39 — capture

Every one is adjudicated **STALENESS** in the SCORE: 31 from the original run, plus 8 that were reported as
findings and dispositioned to staleness in the appended adjudication (findings 1–8). Causes, for context
only — you do **not** need to re-derive them:

- **2–7** — `72a1ac3d` armed the namespacing gate and migrated the corpus by codemod; names grew, spans and
  `:name` payloads moved by exactly that growth.
- **1, 8** — the old golden pinned a **wrong** value (a mid-token span; a stdlib span with error and
  `original_def_span` byte-identical). The correct value is what the code emits now.

The conversion:

```rust
assert_eq!(err, r#"Check(CheckErrors([…]))"#);                                   // before
wat::assert_edn_matches_file!(err, "<stem>__<fn_name>.edn", "<what it pins>");   // after
```

Golden co-located with its `.rs`, named `<source_file_stem>__<test_fn_name>.edn`. Capture with
`UPDATE_EDN=1`, then **re-run without it** to prove the golden matches what the code emits.
Worked reference: `tests/types/enums.rs` and `tests/process/probe_supervisor_select_lost.rs`.

**An internal `src/*.rs` span in a golden is KEPT PINNED** — builder ruling from batch 1: a pinned line
updated when the line moves is in a constant state of correctness, and the span discriminates *which call
site* raised the error. Do not strip them.

### The 1 — rewrite, do NOT capture

`tests/wat_lang/wat_not_eq.rs :: not_eq_f64_cross_numeric_coerce` is **SUPERSEDED**. It asserts arc
**237.8a** (cross-numeric coercion DELETED → mixed-numeric `not=` is a type error). Arc **300 Stone C5**
deliberately reversed that, and Stone **C5b** (`1f1873e1`, today) rebuilt the same path. Capturing a golden
here would freeze a design we abandoned.

**Rewrite it to assert C5's contract.** Ground truth, measured live this session:

```clojure
(:wat::core::not= 3 3.0)   ⇒ true     ; type-checks (C5) and evaluates category-aware (C4)
(:wat::core::= 3 3.0)      ⇒ false    ; different numeric categories
```

So the test should assert that mixed-numeric `not=`/`=` **type-check** (an `Ok`, not a `CheckError`) and
evaluate to `true`/`false` respectively. **Rename the test function** — `not_eq_f64_cross_numeric_coerce`
names the retired behaviour; something like `not_eq_f64_cross_numeric_is_category_aware` states the current
contract. Leave a short comment naming C5 as the superseding stone and 237.8a as what it replaced.

If the file's fixture asserts the old rejection, update it the same way — this is one test's own fixture,
not a corpus migration, so no codemod is required.

## Blast radius

`tests/wat_lang/*.rs` + new `tests/wat_lang/*.edn`, and `tests/types/probe_arc293_holder_bound.rs` + its
golden. **No `src/`. No `.wat` corpus migration. No new `#[ignore]`s. Do not touch the 32 goldens captured
by earlier waves, and do not touch any non-296 ignored test.**

## Capture-once

```
cargo nextest run --release -E 'binary(wat_lang)' --run-ignored all --no-capture > /tmp/wb2cap.log 2>&1
```

Run once, grep the file. `--run-ignored all` sweeps **every** ignored test in that binary, not just this
cohort — batch 1 hit an arc-255 test that is **RED BY DESIGN**. Check a stranger's ignore reason before
treating it as yours.

## STOP triggers — these REJECT; none is permission to ship less

- **STOP-1 — a test's actual output does not match its SCORE row.** The adjudication says what each one
  should look like. If what you see contradicts it, **stop and report** — do not capture to move on, and do
  not re-adjudicate it yourself. The record was settled with evidence; a contradiction means either the
  record or the tree moved, and that is mine to resolve.
- **STOP-2 — a fixture that exercises nothing.** `3cd00fbb` hollowed nine fixtures by deleting the `main`
  that drove them, and a `.wat` that lost its driver still loads clean and its `is_ok()` passes. **A
  positive fixture fails by passing.** If a test's fixture only declares and never calls, report it.
- **STOP-3 — the work seems to need a `src/` or `.wat` corpus change.** It does not. That is a finding.
- **STOP-4 — you are tempted to re-`#[ignore]` something to reach green.** Never. A test you cannot make
  pass is reported still-failing. The orchestrator did exactly this once and the builder cut it:
  *"uhhhhh just fix it?..... why did you ignore it with a note?"*
- **STOP-5 — a red you did not intend. Do NOT re-run** — a re-run that goes green destroys the only
  evidence. `scripts/floor.sh` keeps the untruncated log: copy the failing test's whole stdout+stderr block
  **verbatim** (never `| head`, never a summary), name the exact assertion that fired, and report. **There
  is no such thing as a known flake.**

## Verify — in this order, and read the Summary line, never a piped exit code

```
cargo build --release --tests
cargo clippy --workspace --all-targets --release -- -D warnings      # must be 0
scripts/floor.sh
```

| | before | after |
|---|---|---|
| tests run | 4624 | **4624** |
| passed | 4584 | **4624** |
| failed | **40** | **0** |
| skipped | 82 | 82 |

If `failed` is not 0, name every one that remains. If `run` or `skipped` moved, an ignore was added or a
test was lost — say so.

## Negative controls — keep the keepable ones

Standing rule (`docs/DUNGEON-CRAWL.md` Phase 3): for each control you build, **is it keepable?** If yes,
bank it as a test. If it needs an `src/` mutation, report it with the reason. Discarding is a declared
exception with a stated reason, never the default.

Specifically: after capturing, prove the goldens are not vacuous — a golden that matches anything proves
nothing. Show at least one case where a deliberately wrong expectation makes the assertion fail.

## How to work

You are a **rider, not the orchestrator. Ending your turn ENDS you** — nothing wakes you, no notification is
coming. ⛔ **Run every build and test in the FOREGROUND and block on it.** No `run_in_background`, no
Monitor, no polling-then-stopping. Four riders on these arcs died exactly that way. Your turn ends when the
numbers are in your hands.

Anchor at `/home/watmin/work/holon/wat-rs`; `pwd` first. **Leave the work uncommitted.** Never `git commit`,
`push`, `stash`, `revert`, or `checkout --` — `stash@{0}` holds unrelated work and the checkout holds the
parked un-ignores.

## Report

- how many goldens you captured, and the full list
- the rewritten supersession: its new name, what it now asserts, and why that matches C5
- every STOP that fired
- the non-vacuity demonstration
- clippy count and the floor **Summary line, verbatim**, with the arithmetic
- **the honest deltas — especially anywhere this brief did not match the disk.** Every count in this arc's
  briefs has been wrong at least once. Re-measure what you act on and say plainly where I was wrong.
