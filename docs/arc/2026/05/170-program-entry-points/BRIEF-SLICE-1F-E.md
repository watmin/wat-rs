# Arc 170 slice 1f-ε — BRIEF (`:user::main` migration sweep)

**Sonnet uniform composition.** Closes ~202 baseline failures + chain-unblocks an unknown fraction of the ~399 heterogeneous tail (tests that fail at startup-time can't even reach their actual assertion bugs). Pattern is per `feedback_simple_is_uniform_composition` — N identical signature swaps is simple, not complex.

**Single root cause** (verified by sampling 3 distinct files): `:user::main` definitions in test files declare retired non-canonical signatures `(_stdin _stdout _stderr -> :nil)` (3-arg shape) instead of the canonical `[] -> :wat::core::nil` (no-arg shape per arc 170 slice 1e REALIZATIONS pass 7 + pass 10). The substrate rejects them at startup-time with `StartupError::Check`.

## Slice surface

> *"Migrate every retired `:user::main` signature to canonical no-arg shape."*

This is the closing phase of arc 170 slice 1e's retirement. Slice 1e flipped the contract; this slice migrates the consumers. Per user direction 2026-05-10: arc 170's slices grow as necessary; this is not a new arc.

## Sample evidence (3 distinct test files)

```
;; wat-tests/core/struct-to-form.wat
(:wat::core::define
  (:user::main
    (_stdin :wat::io::IOReader)
    (_stdout :wat::io::IOWriter)
    (_stderr :wat::io::IOWriter)
    -> :wat::core::nil)
  ...body that doesn't use _stdin/_stdout/_stderr...)
```

```
;; wat-tests/core/result-expect.wat (same shape, params NOT underscore-prefixed but unused in body)
;; wat-tests/core/option-expect.wat (same shape)
```

Pattern: 3-arg signature; params unused in body (underscore-prefixed) OR named but ceremonial (body uses match/let, no stdio calls). Migration is mechanical for these cases.

## Scope

### Uniform transformation (the dominant case)

For each `:user::main` definition that matches the retired-signature pattern:

```diff
- (:wat::core::define
-   (:user::main
-     (_stdin :wat::io::IOReader)
-     (_stdout :wat::io::IOWriter)
-     (_stderr :wat::io::IOWriter)
-     -> :wat::core::nil)
-   <body>)
+ (:wat::core::define
+   (:user::main -> :wat::core::nil)
+   <body>)
```

Same edit applied to each site. ~88 unique files; ~202 deftest sites (some files have multiple).

### Non-uniform sub-cases (surface as honest-delta)

Some test bodies may USE the stdio params (without `_` prefix; with actual reads/writes). These need substantive rewrite:
- `(:wat::io::IOReader/read-line stdin)` → `(:wat::kernel::readln)` (ambient via orchestrator services)
- `(:wat::io::IOWriter/write-string stdout ...)` → `(:wat::kernel::println ...)` (same)
- `(:wat::io::IOWriter/write-string stderr ...)` → `(:wat::kernel::eprintln ...)` (same)

Sonnet should detect these by grepping body for the param names (without `_` prefix means it's used). Apply the substantive rewrite where needed. Surface count of mechanical vs substantive cases.

### Out of scope

- **No test-body assertion fixes** — tests that fail AFTER startup with actual assertion errors are heterogeneous-tail work; not this slice.
- **No deftest macro changes** — `deftest`/`deftest-hermetic` macros stay as-is.
- **No bridge-migration** — moving `run-sandboxed-*` body to Layer 1 is a separate later slice.
- **No Console retirement** — separate slice.

## Pre-flight verification

```
:wat::kernel::println    — src/thread_io.rs:182 ✓ (slice 1f-α shipped this)
:wat::kernel::eprintln   — src/thread_io.rs:211 ✓
:wat::kernel::readln     — src/thread_io.rs:240 ✓
```

These are the ambient stdio primitives that substantive-rewrite cases migrate TO. All present.

## Discovery method

```bash
# All test files with retired :user::main signature
grep -rln ":user::main" wat-tests/ tests/ examples/ crates/ 2>/dev/null \
  | xargs grep -l "stdin\|stdout\|stderr"
```

The orchestrator's pre-flight counted 88 unique files. Verify at slice time and sweep.

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A | Sweep all retired-signature `:user::main` sites in `wat-tests/`, `tests/`, `examples/`, `crates/` | grep finds 0 remaining matches of the `(stdin/stdout/stderr` 3-arg shape |
| B | `cargo check --release` green | clean |
| C | `cargo check --release` produces no new warnings | clean |
| D | Workspace failure count drops by ≥ 150 (target: 200+ including chain-unblock) | cargo test count |
| E | Workspace pass count rises by ≥ 150 | cargo test count |
| F | No new regression of pre-existing passing tests | re-baseline |
| G | Mechanical vs substantive case counts surfaced | inline report |
| H | Honest deltas surfaced | per FM 5 |

**8 rows.** No "≥ 200 drop" hard requirement — FM 9 discipline: don't over-predict; let actual data inform.

## Predicted runtime

**90-180 min sonnet.** Bulk grep-and-edit across ~88 files. Most uniform; some substantive sub-cases. Mechanical phase fast; substantive phase paced by each test body's complexity.

**Hard cap:** 360 min (2× upper).

## Honest-delta categories (anticipated)

1. **Mechanical vs substantive split** — pre-flight saw all-mechanical in 3 samples; sonnet may find substantive cases requiring println/eprintln/readln migration. Surface count.
2. **Tests in `tests/` (Rust integration tests with embedded wat strings)** — these have wat source literals inside Rust `r#"..."#` strings. The migration applies; the edit is in the Rust file. May need different grep pattern.
3. **Tests in `examples/` or `crates/`** — same migration applies but in different file trees. Verify the sweep covers these.
4. **deftest-hermetic vs deftest** — both should target canonical `:user::main`; both currently broken with the same root cause; both fix with the same edit. Verify both types of test recover.
5. **Some tests may BREAK with the migration** — if a test was depending on the retired param to work, removing it surfaces an actual test-body bug. These are the heterogeneous-tail kind. Surface count if material.
6. **Workspace count uncertainty** — pre-flight predicted "~202 close + unknown chain-unblock". Actual may be ≥ 202 if chain-unblocks materialize, or possibly slightly less if some sites have additional issues. Surface actual count.

## What to NOT do

- No substrate Rust edits (this is pure wat-source migration)
- No new dependencies; no Mutex/RwLock/CondVar
- No test-body assertion fixes (heterogeneous-tail work)
- No bridge migration (separate slice)
- No Console retirement (separate slice)
- Don't commit yourself — orchestrator atomic-commits with SCORE

## Verification

```
cargo check --release 2>&1 | tail -3
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
grep -rln "_stdin.*_stdout.*_stderr\|(stdin.*:IOReader)\|stdin\\s*:wat::io" wat-tests/ tests/ examples/ crates/ 2>/dev/null | wc -l
```

Post-1f-δ′ baseline: **1577 passed / 636 failed**. Target: failure count drops by ≥ 150 toward ~1730/486 or better.

## Reference

- Predecessor slices: 1f-δ (`316a94e`), 1f-δ′ (`72e051c`) — the bridge restores; both unblocked the test infrastructure from substrate side
- Slice 1e (commit `206bdd1`) — the retirement this migration closes
- `feedback_simple_is_uniform_composition.md` — the discipline this slice executes
- The 202 sample failure message: *"`:user::main` declared with a non-canonical signature is retired (arc 170 slice 1e — REALIZATIONS pass 7 + pass 10); canonical shape is `[] -> :wat::core::nil`"*

## Path forward post-slice-1f-ε

1. Orchestrator scores; atomic-commits deliverable + SCORE
2. **Re-sample remaining failures** — the 399 heterogeneous tail may shrink dramatically; classify what's left
3. **Bridge-migration slice** — move `run-sandboxed-*` body from kernel-namespace to Layer 1 (`:wat::test::run-ast`); retire kernel verbs
4. **Slice 1f-? Console retirement** — independent; bundleable
5. **Arc 170 INSCRIPTION** — once baseline is acceptable
