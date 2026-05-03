# Arc 138 Slice 4b — Sonnet Brief: EdnReadError + LowerError gain spans

**Goal:** add `span: Span` to every `EdnReadError` variant (src/edn_shim.rs) and `LowerError` variant (src/lower.rs); update Display arms to prefix coordinates via `span_prefix`; thread real spans into emission sites; add canaries.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Scope:** primary changes in `src/edn_shim.rs` (6 variants, ~31 emission sites) and `src/lower.rs` (12 variants, ~42 emission sites). NO cross-file consumers — both error types are only re-exported via `src/lib.rs`; pattern-match destructuring lives only in their own files (and possibly tests). NO other files modified.

NO substrate-design changes beyond span-on-variant. NO new helpers beyond a `span_prefix` helper local to each file. NO commits.

## Read in order — your contract

1. `docs/arc/2026/05/138-checkerror-spans/SCORE-SLICE-4A.md` — the just-shipped predecessor; same shape as this slice (variant restructure + Display + emissions + canary).
2. `docs/arc/2026/05/138-checkerror-spans/SCORE-SLICE-2.md` — the original worked example for "small variant restructure + Display + emission threading + canary in one slice."
3. **`src/types.rs`** or **`src/macros.rs`** (just-modified by 4a) — read the `span_prefix` helper and how Display arms use it. Mirror the pattern.

## Variant restructure plan

**EdnReadError (src/edn_shim.rs ~line 214):** all 6 variants gain `span: Span`. Convert unit variant.
- `UnknownTag { ns, name, body_shape }` → `UnknownTag { ns, name, body_shape, span }`
- `UnsupportedTag(String)` → `UnsupportedTag(String, Span)`
- `NoTypeRegistry` → `NoTypeRegistry(Span)`
- `UnknownStructField { type_path, key }` → `UnknownStructField { type_path, key, span }`
- `EnumVariantNotFound { type_path, variant }` → `EnumVariantNotFound { type_path, variant, span }`
- `Other(String)` → `Other(String, Span)`

**LowerError (src/lower.rs ~line 43):** all 12 variants gain `span: Span`. Many unit variants — convert to single-element tuple.
- `AtomArity(usize)` → `AtomArity(usize, Span)`
- `AtomNonLiteral` → `AtomNonLiteral(Span)`
- `BindArity(usize)` → `BindArity(usize, Span)`
- `BundleShape` → `BundleShape(Span)`
- `PermuteArity(usize)` → `PermuteArity(usize, Span)`
- `PermuteStepNotInt` → `PermuteStepNotInt(Span)`
- `PermuteStepOverflow(i64)` → `PermuteStepOverflow(i64, Span)`
- `ThermometerShape` → `ThermometerShape(Span)`
- `BlendShape` → `BlendShape(Span)`
- `UnsupportedUpperCall(String)` → `UnsupportedUpperCall(String, Span)`
- `UnsupportedForm(String)` → `UnsupportedForm(String, Span)`
- `MalformedCall` → `MalformedCall(Span)`

## Display arm updates

Add a file-local helper `fn span_prefix(span: &Span) -> String` near the top of `src/edn_shim.rs` (mirror src/macros.rs / src/types.rs). Same for `src/lower.rs`.

Each Display arm prefixes `{span_prefix(span)}` when non-unknown.

## Emission threading patterns

Same vocabulary as prior slices:

- **Pattern A — args[i].span()**: when an arg is in scope from `for arg in items` or similar, use it.
- **Pattern B — list_span / form span**: when the form's outermost span is in scope (e.g., from `WatAST::List(items, span)` destructure), use that span.
- **Pattern D — head keyword**: when emitting via `WatAST::Keyword(k, head_span)`, use head_span.
- **Pattern E — no span available**: rare; rationale comment `// arc 138: no span — <reason>`. Examples likely: parsing raw EDN strings (no AST yet), `Other(String)` catch-all from upstream parser errors.
- **Pattern F — broaden helper sig**: if a private helper lacks span access, add `span: Span` parameter and propagate from callers within the same file.

For src/edn_shim.rs: `read_edn(s: &str, ...)` and friends parse EDN strings — there's no WatAST yet. Most EdnReadError sites likely have NO AST span available (Pattern E dominates). The `edn_to_value(edn: &OwnedValue, ...)` walks an OwnedValue tree, also no WatAST. Acceptable; document Pattern E with rationale.

For src/lower.rs: `lower(ast: &WatAST)` and `lower_call(items: &[WatAST])` — the AST is in scope at every site. Most emissions can use Pattern A or B.

## Constraints

- ONLY src/edn_shim.rs + src/lower.rs modified. NO other files.
- NO test changes outside the canaries (existing test pattern updates are OK if they're mandatory compile fixes for new variant shapes; document them in the report).
- NO commits, NO pushes.
- NO new variants beyond the existing ones.
- NO Display string content changes (besides the `span_prefix` prefix).
- NO trait expansion.
- `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` returns empty.
- All existing canaries (4 from prior slices) continue to pass.
- Two NEW canaries added: one for EdnReadError (e.g., trigger UnknownTag via parsing `#unknown/Type {}`), one for LowerError (e.g., trigger MalformedCall via `lower(&parse(":not-a-list"))`).

## What success looks like

1. 6 EdnReadError + 12 LowerError variants all carry `span: Span`.
2. Display arms prefix coordinates via local `span_prefix`.
3. Emission sites use real spans where in-scope; Pattern E documented per-site for sites without AST context.
4. Two new canaries pass; existing canaries pass; workspace tests pass.
5. NO commits.

## Reporting back

Target ~400 words. Per slice 4a's example — be COMPREHENSIVE and SELF-CONTAINED. Include per-file counts, line numbers, pattern distribution, canary names, honest deltas.

1. **Counts**: BEFORE Span::unknown() in src/edn_shim.rs + src/lower.rs (likely 0 each) → AFTER. Per-file pattern distribution.
2. **Variant restructure**: list of 6 + 12 variants confirmed.
3. **Display arm count**: 6 + 12 = 18 arms updated.
4. **Pattern distribution per file**: A/B/D/E/F.
5. **Canaries**: names + line numbers + what they trigger.
6. **Verification**: `cargo test --release --workspace` totals; existing canaries pass; new canaries pass.
7. **`git diff --stat`** — should be 2 files (src/edn_shim.rs, src/lower.rs).
8. **Honest deltas** — any helper sigs broadened (Pattern F); any Pattern E rationales (especially edn_shim.rs where AST may genuinely be unavailable); any existing test pattern updates needed.
9. **Four questions applied** to your output.

## What this slice tests (meta)

The hypothesis: with slice 4a's pattern proven (variant + Display + emission + canary in one engagement, ~10 min for 50 emissions), this 73-emission slice ships in ~15-25 min. EdnReadError likely has high Pattern E ratio (no AST in scope when parsing raw EDN); LowerError likely has high Pattern A/B ratio (AST in scope throughout).

Begin by reading slice 4a's just-shipped span_prefix helper in src/macros.rs and src/form_match.rs. Mirror in src/edn_shim.rs and src/lower.rs. Sweep emissions. Add canaries. Verify. Report comprehensively.
