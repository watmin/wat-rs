# Arc 170 slice 1f-ζ — BRIEF (`:user::main` migration continuation)

**Sonnet uniform composition.** Continuation of slice 1f-ε. Sonnet's 1f-ε discovery was too narrow — found 27 files / ~180 sites, missed an unknown larger number of:

1. **Wat source embedded in Rust string literals** (single-line and multi-line)
2. **Inner `:user::main` definitions inside `(:wat::core::forms ...)` blocks** passed to `spawn-program-ast` / `fork-program-ast`
3. **Standalone wat files** not in the original grep path

Re-classification shows **438 occurrences of `BareLegacyMainSignature` diagnostic** still in test output post-1f-ε, with confirmed missed sites in:
- `tests/wat_arc104_fork_program.rs:67, 90, 112` (inline wat source as `r#"..."#`)
- `wat-tests/console.wat:185, 243` (standalone wat file)
- `tests/wat_harness.rs:18, 57` (3-arg sigs in test body)
- `tests/wat_arc144_special_forms.rs:75, 304` (multiple sites per file)
- `tests/wat_arc113_cross_fork_cascade.rs:57` (inner form)
- `tests/wat_arc144_uniform_reflection.rs:91, 129, 164` (multiple)

## Slice surface

> *"Finish the `:user::main` migration sonnet started in 1f-ε."*

Same uniform-composition pattern. The only new value-add is broader discovery — same transformation rule applies.

## Discovery method (broader than 1f-ε)

```bash
# Standalone wat files — all paths
grep -rn ":user::main" --include="*.wat" wat-tests/ tests/ examples/ crates/

# Wat source inside Rust string literals — both single-line and multi-line r#"..."#
grep -rnE '":user::main|\(:wat::core::define [^)]*\(:user::main' tests/ crates/ examples/ --include="*.rs"

# Three-arg signature anywhere (the actual pattern)
grep -rnE '\(stdin\s+:wat::io::IOReader\)|\(_stdin\s+:wat::io::IOReader\)' wat-tests/ tests/ examples/ crates/

# Multi-line definitions: detect by paren-context (use a different approach if grep alone insufficient)
```

Sonnet should run all three greps, deduplicate file paths, sweep each file. Expect tens of additional files beyond 1f-ε's 27.

## Transformation rule (unchanged from 1f-ε)

For each retired `:user::main` site (top-level OR inner-form OR string-literal):

```diff
- (:user::main (_stdin :IOReader) (_stdout :IOWriter) (_stderr :IOWriter) -> :nil)
+ (:user::main -> :nil)
```

If body uses the stdio params (without `_` prefix), substantive migration to ambient `:wat::kernel::println` / `eprintln` / `readln`.

For wat source inside Rust string literals: edit the string literal contents. Watch for escape sequences (`\"` etc.).

## What to NOT do

- Don't fix the 7 heterogeneous-tail files from 1f-ε SCORE (negative-test cases like `wat_arc170_slice_1e_user_main_nil.rs` — those DEPEND on the substrate rejecting wrong shapes)
- Don't restore `spawn-program` / `fork-program-ast` retired verbs (separate sibling slice)
- Don't touch test assertion logic (heterogeneous tail)
- Don't commit yourself — orchestrator atomic-commits with SCORE

## Pre-flight expected outcome

Post-1f-ε baseline: **1752 passed / 461 failed**. Expected post-1f-ζ: failure count drops by ≥ 200 (chain-unblock from completing the migration). If actual drop is much smaller, sample multiple failure modes (FM 9) before generalizing.

## Ship criteria

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `BareLegacyMainSignature` occurrences in cargo test output drops below 20 | grep test stderr |
| B | `(stdin\s*:wat::io::IOReader)` 3-arg pattern: 0 matches in test trees | grep |
| C | `cargo check --release` green | clean |
| D | No regression of pre-existing passing tests | re-baseline |
| E | Failure count drops ≥ 200 | cargo test count |
| F | Pass count rises ≥ 200 | cargo test count |
| G | Honest deltas surfaced | per FM 5 |

**7 rows.** No "0 BareLegacyMainSignature" hard requirement — some tests DEPEND on the diagnostic firing (negative tests).

## Predicted runtime

**60-120 min sonnet.** Same uniform-composition pattern as 1f-ε; broader discovery, same edits. Discovery phase ~10-20 min; sweep phase ~50-100 min.

**Hard cap:** 240 min.

## Honest-delta categories (anticipated)

1. **More substantive cases** — pre-flight 1f-ε saw 3.5× more substantive than 3-sample suggested. Same pattern likely holds — broader discovery surfaces more substantive bodies.
2. **String-literal escape sequences in Rust** — wat source inside Rust raw strings (`r#"..."#`) vs escaped strings (`"\"..."` with `\\"`) need different sed-like patterns. Surface friction.
3. **Negative-test cases** — `freeze_err` test strings INTENTIONALLY contain 3-arg signatures to verify the substrate rejects them. Sonnet must distinguish these (do NOT migrate) from genuinely-needs-migration sites. The diagnostic context is the tell (`freeze_err(...)` test calls).
4. **Workspace count uncertainty** — broader sweep may surface tests that fail for OTHER reasons after signature fix (heterogeneous tail). Surface honest count.

## Reference

- Slice 1f-ε SCORE (`7b19cef`) — incomplete sweep; this slice continues
- `feedback_simple_is_uniform_composition.md` — discipline
- Recovery doc FM 9 — multi-sample discipline (orchestrator-side now broader)

## Path forward post-slice-1f-ζ

1. Orchestrator scores; atomic-commits deliverable + SCORE
2. **Re-sample remaining failures** — should be substantially reduced
3. **Sibling slice — restore retired `spawn-program` / `fork-program-ast`** (BareLegacy* diagnostics for those)
4. **Heterogeneous-tail triage** — substantive test-body issues case-by-case
5. **Bridge-migration slice** — Layer 1 absorbs `run-sandboxed-*` body
6. **Arc 170 INSCRIPTION** — once baseline stabilizes
