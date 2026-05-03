# Arc 138 F1 — Sonnet Brief: close MacroDef structural span gap

**Goal:** add `span: Span` field to `MacroDef` struct in src/macros.rs, thread it at the one constructor site, replace the 3 Pattern E `Span::unknown()` placeholders in `register`/`register_stdlib` with the threaded span. Remove the now-obsolete `// arc 138: no span` rationale comments at those sites.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Driver:** the user invoked the no-deferrals rule (`docs/arc/2026/05/138-checkerror-spans/CRACKS-AUDIT.md`). MacroDef is the smallest of four real cracks; F1 closes it.

## Read in order

1. `docs/arc/2026/05/138-checkerror-spans/CRACKS-AUDIT.md` — the no-deferrals charter; F1 is item one.
2. `docs/arc/2026/05/138-checkerror-spans/SCORE-SLICE-4A.md` — context on the original "MacroDef structural span gap" observation that this followup closes.
3. `src/macros.rs` lines 50–142 (struct + register/register_stdlib).
4. `src/macros.rs` lines 309–341 (`parse_defmacro_form` — the only constructor of MacroDef).

## What to do

1. **Struct change:** Add `pub span: Span` field to `MacroDef` (src/macros.rs ~line 54). Document the field briefly: "Source span of the `(:wat::core::defmacro ...)` form that registered this macro. Used by register/register_stdlib to attribute MacroError emissions back to the user's source position."

2. **Constructor update:** at src/macros.rs ~line 335, the single `MacroDef { ... }` literal in `parse_defmacro_form`. Add `span: list_span.clone()` (or move list_span if it isn't otherwise used; check the function flow). The `list_span` is already in scope as the outer defmacro form's span — that's the right span source.

3. **Update register (line 96–111):**
   - Replace `Span::unknown()` at line 100 (ReservedPrefix) with `def.span.clone()`.
   - Replace `Span::unknown()` at line 107 (DuplicateMacro) with `def.span.clone()`.
   - DELETE the 2 rationale comments at lines 98–99 and 106 (they're no longer applicable).

4. **Update register_stdlib (line 121–131):**
   - Replace `Span::unknown()` at line 127 (DuplicateMacro) with `def.span.clone()`.
   - DELETE the rationale comment at line 126.

5. **Test updates:** if any test constructs `MacroDef` directly via struct literal, update it to include `span: Span::unknown()` (test code legitimately uses synthetic spans). Search for `MacroDef\s*{` in tests/ and src/macros.rs's tests module.

6. **Byte-equivalence preservation:** `macro_byte_equivalent` (line 140) compares params + rest_param + body but NOT name. Span should similarly NOT be compared (two equivalent macros declared from different sources should still be byte-equivalent). Leave macro_byte_equivalent unchanged — it already excludes name and span doesn't enter the comparison.

## Constraints

- ONLY src/macros.rs modified. No other files.
- NO new variants. No Display string changes. No trait expansion.
- NO commits, NO pushes.
- All 6 existing arc138 canaries continue to pass.
- Workspace tests pass: `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` returns empty.
- The existing `macros::tests::arc138_macro_error_message_carries_span` canary continues to pass (it triggers ArityMismatch which already has real span; this followup is about register/register_stdlib paths, not arity).

## Optional additional canary

If you want to strengthen coverage, add a test that triggers `DuplicateMacro` or `ReservedPrefix` via two `defmacro` forms in a wat snippet, asserts the rendered Display contains `<test>:` substring (now possible because the span propagates through MacroDef). NOT required — the fix itself is what matters.

## Reporting back

Compact (~250 words):

1. **Diff stat:** should be 1 file (src/macros.rs).
2. **Struct field added:** confirm.
3. **Constructor update:** confirm `parse_defmacro_form` sets `span: list_span.clone()`.
4. **3 Pattern E sites resolved:** confirm `Span::unknown()` → `def.span.clone()` at all 3 sites; rationale comments deleted.
5. **Test pattern updates:** list any tests where MacroDef literal needed `span:` field added.
6. **Verification:** all 6 canaries pass; workspace tests pass.
7. **Honest deltas:** anything unexpected.
8. **Four questions** applied briefly.

## Why this is small

One struct + one constructor + 3 site updates. Most of the work is the boilerplate (struct field doc, test compile fixes). Estimated 10-15 min sonnet runtime.
