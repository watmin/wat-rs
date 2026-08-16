# BRIEF — 296 Wave B5: `tests/macros` (7)

> `CAMPAIGN-the-recapture-cascade.md` governs. Read `SCORE-296-WaveB2-wat_lang.md` **including its
> appended `★ ADJUDICATION OF THE 9 FINDINGS`** — the worked example, and it names the sub-classes.

## Baseline — re-verify, do not trust

```
HEAD = 30b5a830, tree CLEAN, floor GREEN, clippy 0
  Summary [ 221.909s] 4656 tests run: 4656 passed (3 slow), 56 skipped
296-pending: 17.  This brief takes 7.
```

## The cohort — 7 items across FOUR files

```
probe_arc209_macro_param_type_enforced.rs   1   lying_macro_param_type_is_rejected_at_macro_def
probe_arc249_threading.rs                   2   witness_thread_first_empty_step_panics_at_expansion
                                                witness_thread_last_empty_step_desugars_to_call_on_acc
probe_arc258_stone2b_macro_error.rs         2   contract_02_non_exhaustive_cond_names_else
                                                contract_03_macro_error_surfaces_its_message
probe_arc279_format.rs                      2   format_strict_missing_kwarg_is_macro_error
                                                format_strict_unused_kwarg_is_macro_error
```

Re-verify: `grep -c '296-recapture-pending' tests/macros/*.rs`. (My first extraction of these names swept
up helper `fn` signatures and had to be redone — count *tests*, not `fn` lines.)

## ⚠ THIS DIRECTORY IS COMPLETELY UNCONVERTED

`tests/macros` has **0** `.edn` goldens and **0 of 44** files using `assert_edn_matches_file!`. `function`
had 1; `diagnostics` had 115. You are establishing the convention from nothing. Worked references live
elsewhere: `tests/types/enums.rs`, `tests/function/*.edn` (Wave B4, yesterday's shape),
`tests/diagnostics/*.edn`.

Expect mechanism changes beyond a straight assert swap, and **report each as a delta**. Two known
classes, both already hit: `Debug` of an `Option<StartupError>` is not valid EDN (use `.expect_err(...)`
first); and a `format!("{}\n---\n{:?}", e, e)` helper glues the same EDN blob to itself around a bare
`---` since Stone B made `Display` emit EDN (drop to `Debug` only).

## ⛔ THE DOCTRINE FOR THIS COHORT — MACROEXPAND FIRST

Project law (`CLAUDE.md`), and this is the cohort it was written for:

> **Debugging a MACRO? READ THE EXPANDED FORM FIRST — `macroexpand`.** A confusing error from a
> `defservice`/`defrecord`/`deftest`/kwargs-`defn` form is almost never a mystery in the macro's
> *logic*; it is something the macro **emitted** that you have not looked at. Dump the expansion and
> read the actual generated names/forms before theorizing, greping, or briefing.

**Two of these tests are literally about expansion** — `witness_thread_first_empty_step_panics_at_expansion`
and `witness_thread_last_empty_step_desugars_to_call_on_acc`. For those, the expanded form **is** the
subject; adjudicating them from the error text alone is guessing at second-hand evidence.

Pairs with it: `target/release/wat --check <f.wat>` is ~0.2s. **Expand to see WHAT was emitted;
`--check` to see whether it type-checks.** Note a `defservice` cannot be runtime-macroexpanded (a
`:wat::core::Record` evaluates to its constructor fn) — expand at the form level.

**The recurring class, if a generic form misbehaves:** suspect a **string comparison with one side
normalized and the other not** before suspecting the type system. Arc 278 hit it three times — a
companion-name suffix appended past `<T>`, a type-arg list flat-`split(',')` tearing `State<K,V>` into
`State<K`+`V>`, and a `:messages` membership check comparing a base name against a declared `Name<K>`.
The type system is usually fine; a `format!`/`split`/`==` on names is the culprit.

## ⛔ TIMELY HAZARD — `cond` internals moved an hour ago

Commit `59ee1f06` (merged today) states in its own message: *"the cond golden slid 1246→1296."* It
updated `tests/wat_lang/wat_core_cond__cond_refuses_missing_else.edn`.

**This cohort contains `contract_02_non_exhaustive_cond_names_else` — the same subject.** If its
expectation carries a `wat/core.wat` line for the cond machinery, that line has almost certainly moved
by the same ~50, and that is **STALENESS from a change that landed today**, not from arc 296. Say so
explicitly if you find it; do not report it as a mystery.

## THE LAW — one test at a time

`UPDATE_EDN=1` writes whatever the code currently emits. The inline literal **is** the old expectation.
**Per test: un-ignore → run WITHOUT `UPDATE_EDN` → adjudicate → capture only expected-staleness.**

**STALENESS → capture:** same error count and order · every span in the user's `.wat` identical (EDN
spells `end_line`/`end_col` as `:end #wat.core.Option/Some [#wat.core/Pos {…}]` — same numbers,
different spelling) · kinds map 1:1 · every payload field present with the same value · EDN adds
`:message`/`:causes`/`:location` — additive.

**FINDING → report, do NOT capture:** an error disappeared · a non-additive error appeared · **a span
moved in the user's `.wat`** · a payload field lost its value · `field-0`/`field-N` anywhere · a
populated `:remedies` collapsed to `[]`.

**SUPERSEDED → retire or rewrite, do NOT capture.** Before calling an absent error a defect,
`grep -rl` its subject across `docs/arc/` and read any later arc naming it. **Three waves running, this
column decided real rows** — B2's `not_eq` (237.8a reversed by C5), B3's `stone7c` (`Span::unknown`
annihilated by 298.2), and B4's whole `fn_rename` file (which turned out NOT superseded, but only a
read of the arc record could establish that).

**⚠ B4's LESSON: JUDGE THE BODY, NOT THE NAME.** All four of B4's `fn_rename` tests are named
`..._silently_aliases...`; every one of their bodies asserts a hard-cut *rejection*. Judging by name
would have produced four confident wrong dispositions. **`lying_macro_param_type_is_rejected_at_macro_def`
and the two `witness_*` names are descriptive claims, not evidence — read what each asserts.**

Prose drift inside a `:reason`/`:message` is **not** a finding. Judge structure and values.

**An internal `src/*.rs` or `wat/*.wat` span that moved is STALENESS — recapture and KEEP PINNING IT.**
Seven goldens pin such spans today; five needed updating in one recent migration, and grok's commit
moved another. Routine, not alarming.

### The sub-class B2 minted

Two of B2's "findings" were neither stale nor superseded: **the golden pinned a WRONG value, so the fix
looks like a regression.** Heavier burden — you must *prove the old value was wrong* (B2's were a span
landing mid-token, and an error span byte-identical to its own `original_def_span` in a file that did
not contain the symbol). Prove it or report it; never wave it through as staleness.

## The conversion

```rust
assert_eq!(err, r#"Check(CheckErrors([…]))"#);                                   // before
wat::assert_edn_matches_file!(err, "<stem>__<fn_name>.edn", "<what it pins>");   // after
```

Golden co-located, `<source_file_stem>__<test_fn_name>.edn`. Capture with `UPDATE_EDN=1` **after**
adjudicating, then re-run **without** it to prove the golden matches.

⚠ **No absolute host path in a golden.** B3 caught `rust_caller_span!()` producing
`/home/watmin/work/holon/wat-rs/...` — non-portable, the only absolute path in its directory. If a
value carries one, that is a finding; say so rather than capturing it.

⚠ **Scope your `UPDATE_EDN` filter by TEST NAME, not by file or binary.** B3 used a file-wide filter and
swept a pre-existing non-cohort golden into a reformat. B4 filtered by the eight specific test names and
had zero collateral. Check `git status` before finishing and account for every changed file.

## Capture-once

```
cargo nextest run --release -E 'binary(macros)' --run-ignored all --no-capture > /tmp/wb5.log 2>&1
```

Run once, grep the file. `--run-ignored all` sweeps **every** ignored test in that binary — batch 1 hit
an arc-255 test that is **RED BY DESIGN**. Check a stranger's ignore reason before adopting it.

## STOP triggers — REJECTION criteria, never permission to ship less

- **STOP-1 — an adjudication you cannot place.** Capture verbatim, report; do not capture the golden.
- **STOP-2 — more than ~6 findings**, or a fixture that exercises nothing. `3cd00fbb` hollowed nine
  fixtures by deleting the `main` that drove them: **a positive fixture fails by passing.**
- **STOP-3 — the work needs a `src/` or `.wat` corpus change.** Findings, not licence. A corpus
  migration is a wat-fix codemod (R21), never a hand edit.
- **STOP-4 — tempted to re-`#[ignore]` to reach green.** Never.
- **STOP-5 — a red you did not intend. Do NOT re-run** — a re-run that goes green destroys the only
  evidence. `scripts/floor.sh` keeps the untruncated log: copy the whole stdout+stderr block
  **verbatim**, name the exact assertion, report. **There is no such thing as a known flake.**

## Blast radius

`tests/macros/*.rs` + new `tests/macros/*.edn`. **No `src/`. No `.wat` corpus changes.** Do not touch
other waves' captures or any non-296 ignored test.

## Verify — in this order; read the Summary line, never a piped exit code

```
cargo build --release --tests
cargo clippy --workspace --all-targets --release -- -D warnings      # must be 0
scripts/floor.sh
```

| | before | after (if all 7 land) |
|---|---|---|
| tests run | 4656 | **4663** |
| passed | 4656 | **4663** |
| failed | 0 | **0** |
| skipped | 56 | **49** |
| 296-pending | 17 | **10** |

If findings block some, leave them **failing** and name them.

## Negative controls

Standing rule (`docs/DUNGEON-CRAWL.md` Phase 3): for each control, **is it keepable?** If yes, bank it
as a test. If it needs an `src/` mutation, report it with the reason. Discarding is a declared exception
with a stated reason, never the default.

## Report

- the 7-row adjudication table, one row per test, with its column
- **for the two `witness_*` expansion tests: the expanded form you read**, and what it showed
- whether `contract_02_non_exhaustive_cond_names_else` was touched by the `cond` slide, and by how much
- every finding **verbatim**, with the exact field that moved
- any test whose NAME disagrees with what its body asserts (B4 found four; say if you find more)
- any hollow fixture, and what its test failed to exercise
- negative controls kept or not, and why
- `git status` accounted for, file by file
- clippy count and the floor **Summary line verbatim**, with the arithmetic
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.** Say plainly where I
  was wrong.
