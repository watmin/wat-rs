# Arc 138 F1 — Pre-handoff expectations

**Written:** 2026-05-03, AFTER drafting brief, BEFORE spawning sonnet.

**Brief:** `BRIEF-F1-MACRODEF.md`
**Target:** src/macros.rs only.

## Setup — workspace state pre-spawn

- Baseline (commit `1df4af5`): cracks audit committed; arc 138 slices 1–4b shipped.
- 3 Pattern E sites in macros.rs `register`/`register_stdlib` use `Span::unknown()` with rationale comments at lines 98–99, 106, 126.
- 1 constructor of MacroDef at line 335 (parse_defmacro_form). `list_span` already in scope as the defmacro form's outer span.
- 6/6 arc138 canaries pass; 771/771 lib tests pass.

## Hard scorecard (6 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File scope | Only src/macros.rs modified. No other files. |
| 2 | MacroDef gains span field | `pub span: Span` field added. Doc comment present. |
| 3 | Constructor sets span | `parse_defmacro_form` at line 335 sets `span: list_span.clone()` (or equivalent). |
| 4 | 3 Pattern E sites resolved | `Span::unknown()` → `def.span.clone()` at register lines 100, 107 and register_stdlib line 127. Rationale comments deleted. |
| 5 | Workspace tests pass | `cargo test --release --workspace 2>&1 \| grep FAILED \| grep -v trading` returns empty. All 6 arc138 canaries PASS. 771/771 lib tests. |
| 6 | No commits | Working tree shows uncommitted modifications only. |

## Soft scorecard (3 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 7 | Byte-equivalence preserved | `macro_byte_equivalent` unchanged; span does NOT enter the comparison. |
| 8 | Test pattern updates minimal | Any MacroDef literal in tests gets `span: Span::unknown()` added; no test logic changes. |
| 9 | Honest report | Compact ~250 words; per-row evidence; honest delta on anything unexpected. |

## Independent prediction

Smallest possible followup. Single file, one struct, one constructor, 3 emission sites.

- **Most likely (~85%):** 6/6 hard + 3/3 soft. Sonnet ships in 5-10 min.
- **Test pattern updates surface (~10%):** sonnet finds 1-3 MacroDef literals in tests that need span field added. Mechanical; no scope expansion.
- **Sonnet adds the optional DuplicateMacro canary (~5%):** strengthens coverage; documents in report.

## Methodology

After sonnet reports back:

1. Read this file FIRST.
2. `git diff --stat` → 1 file, src/macros.rs only.
3. `grep -c "Span::unknown()" src/macros.rs` → should drop by 3 (from 7 down to 4 — 4 leftover are pre-existing in test code or other Pattern E categories outside this slice's scope).
4. `grep -c "// arc 138: no span" src/macros.rs` → should drop by 3 (from 4 down to 1; the 1 leftover is the parse_defmacro non-list arm).
5. `cargo test --release -p wat --lib arc138` → 6/6 PASS.
6. `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` → empty.
7. Score; commit + push; queue F2 (SchemeCtx).

## What this slice tells us

- Smallest crack closes cleanly → followup pattern validated. F2/F3/F4 dispatch with confidence.
- Sonnet handles trivial slices cleanly → calibration data for sub-15-min engagements.
