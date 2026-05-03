# Arc 138 Slice 4a — Sonnet Brief: MacroError + ClauseGrammarError gain spans

**Goal:** add `span: Span` to every `MacroError` variant (src/macros.rs) and `ClauseGrammarError` variant (src/form_match.rs); update Display arms to prefix coordinates via `span_prefix`; thread real spans into emission sites; add canaries; update cross-file consumer pattern matches.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Scope:** primary changes in `src/macros.rs` (9 variants, ~37 emission sites) and `src/form_match.rs` (7 variants, ~13 emission sites). Cross-file consumer pattern updates in `src/check.rs::grammar_error_to_check_error`. NO changes to `src/freeze.rs` (only wraps MacroError, doesn't destructure variants) or `src/runtime.rs` (only doc reference). NO changes elsewhere.

NO substrate-design changes beyond span-on-variant. NO new helpers beyond a `span_prefix` helper local to each file. NO commits.

## Read in order — your contract

1. `docs/arc/2026/05/138-checkerror-spans/DESIGN.md` — arc framing.
2. `docs/arc/2026/05/138-checkerror-spans/SCORE-SLICE-2.md` — the worked example for "small variant restructure + Display + emission threading + canary in one slice." Slice 2 did 10 TypeError variants in `src/types.rs`; you're doing essentially "two slice 2's."
3. `docs/arc/2026/05/138-checkerror-spans/SCORE-SLICE-3A-FINISH.md` — the helper-sig broadening pattern (`list_span: &Span`).
4. **`src/types.rs`** — read the `span_prefix` helper and how `TypeError::*` Display arms use it. Mirror the pattern in src/macros.rs and src/form_match.rs.
5. **`src/check.rs::grammar_error_to_check_error`** at line ~6906 — the cross-file consumer to update.

## Variant restructure plan

**MacroError (src/macros.rs ~line 142):** all 9 variants gain `span: Span` field.
- `DuplicateMacro(String)` → `DuplicateMacro(String, Span)`
- `ReservedPrefix(String)` → `ReservedPrefix(String, Span)`
- `MalformedDefmacro { reason }` → `MalformedDefmacro { reason, span }`
- `UnsupportedBody { name, reason }` → `UnsupportedBody { name, reason, span }`
- `ArityMismatch { name, expected, got }` → `ArityMismatch { name, expected, got, span }`
- `UnboundMacroParam { name }` → `UnboundMacroParam { name, span }`
- `SpliceNotList { name, got }` → `SpliceNotList { name, got, span }`
- `ExpansionDepthExceeded { limit }` → `ExpansionDepthExceeded { limit, span }`
- `MalformedTemplate { reason }` → `MalformedTemplate { reason, span }`

**ClauseGrammarError (src/form_match.rs ~line 82):** all 7 variants gain `span: Span`. Convert unit variants to tuple form.
- `NotAList` → `NotAList(Span)`
- `EmptyList` → `EmptyList(Span)`
- `NonKeywordHead` → `NonKeywordHead(Span)`
- `UnknownHead(String)` → `UnknownHead(String, Span)`
- `NotArity { got }` → `NotArity { got, span }`
- `WhereArity { got }` → `WhereArity { got, span }`
- `BinaryArity { op, got }` → `BinaryArity { op, got, span }`

## Display arm updates

Add a file-local helper `fn span_prefix(span: &Span) -> String` near the top of `src/macros.rs` (mirror src/types.rs's helper). Same for `src/form_match.rs`.

Each Display arm prefixes `{span_prefix(span)}` when non-unknown. Mirror src/types.rs slice 2 pattern.

## Emission threading patterns

Same vocabulary as prior slices:

- **Pattern A — args[i].span()**: when an arg is in scope from `for arg in items` or similar, use it.
- **Pattern B — list_span / form span**: when the form's outermost span is in scope (e.g., from `WatAST::List(items, span)` destructure), use that span.
- **Pattern D — head keyword**: when emitting via `WatAST::Keyword(k, head_span)`, use head_span.
- **Pattern E — no span available**: rare; rationale comment `// arc 138: no span — <reason>`.
- **Pattern F — broaden helper sig**: if a helper function lacks span access, add `span: Span` parameter and propagate from callers within the same file. Cross-file broadening is OUT OF SCOPE.

For src/macros.rs: most emission sites are inside `register`, `register_stdlib`, `expand_once`, `expand_form` and helpers. The `WatAST` being processed is in scope at most sites — pluck its `.span()`.

For src/form_match.rs: `classify_clause(ast: &WatAST)` and helpers. The `ast` parameter span (or `items[0].span()` for sub-clauses) is the right source.

## Cross-file consumer updates — `src/check.rs::grammar_error_to_check_error`

Function at line ~6906 destructures all 7 ClauseGrammarError variants:
```rust
let reason = match e {
    G::NotAList => "...",
    G::EmptyList => "...",
    G::NonKeywordHead => "...",
    G::UnknownHead(h) => format!(...),
    G::NotArity { got } => format!(...),
    G::WhereArity { got } => format!(...),
    G::BinaryArity { op, got } => format!(...),
};
```

After variant restructure, update patterns:
- Unit-converted-to-tuple: `G::NotAList(_)`, `G::EmptyList(_)`, `G::NonKeywordHead(_)`
- Tuple-extended: `G::UnknownHead(h, _)`
- Struct-extended: `G::NotArity { got, .. }`, `G::WhereArity { got, .. }`, `G::BinaryArity { op, got, .. }`

The function's own `span: Span` parameter is preserved; the new variant span is ignored at this call site (caller already has the right span context). This is correct — don't try to use the variant's span.

## Constraints

- ONLY src/macros.rs + src/form_match.rs + src/check.rs modified. NO other files.
- NO test changes outside the canaries.
- NO commits, NO pushes.
- NO new variants beyond the existing ones.
- NO Display string content changes (besides the `span_prefix` prefix).
- NO trait expansion.
- `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` returns empty.
- Existing canaries continue to pass.
- Two NEW canaries added: one for MacroError (e.g., trigger UnknownHead via macroexpand in a wat snippet), one for ClauseGrammarError (e.g., trigger via `:wat::form::matches?` with malformed clause).

## What success looks like

1. 9 MacroError + 7 ClauseGrammarError variants all carry `span: Span`.
2. Display arms prefix coordinates via local `span_prefix`.
3. Emission sites use real spans where in-scope; Pattern E/F documented per-site if needed.
4. `grammar_error_to_check_error` patterns updated; check.rs builds.
5. Two new canaries pass; existing canaries pass; workspace tests pass.
6. NO commits.

## Reporting back

Target ~400 words:

1. **Counts**: BEFORE Span::unknown() in src/macros.rs + src/form_match.rs (likely 0 each) → AFTER. Per-file pattern distribution.
2. **Variant restructure**: list of 9 + 7 variants confirmed.
3. **Display arm count**: 9 + 7 = 16 arms updated.
4. **Pattern distribution per file**: A/B/D/E/F.
5. **Cross-file consumer**: `grammar_error_to_check_error` patterns updated; line range.
6. **Canaries**: names + line numbers + what they trigger.
7. **Verification**: `cargo test --release --workspace` totals; existing canaries pass; new canaries pass.
8. **`git diff --stat`** — should be 3 files (src/macros.rs, src/form_match.rs, src/check.rs).
9. **Honest deltas** — any helper sigs broadened (Pattern F); any Pattern E rationales; any cross-file pattern issues.
10. **Four questions applied** to your output.

## What this slice tests (meta)

The hypothesis: with slice 2's TypeError pattern proven, the same shape applies to MacroError + ClauseGrammarError in one engagement (~50 emission sites + 16 variants + cross-file consumer). Expected runtime: 25-40 min.

Begin by reading slice 2's worked example (src/types.rs `span_prefix` helper + Display arms). Mirror in src/macros.rs and src/form_match.rs. Then sweep emission sites. Then update check.rs pattern matches. Then add canaries. Then verify. Report.
