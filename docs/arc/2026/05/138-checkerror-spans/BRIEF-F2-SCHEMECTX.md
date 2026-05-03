# Arc 138 F2 — Sonnet Brief: close SchemeCtx trait gap

**Goal:** expand the `SchemeCtx` trait (src/rust_deps/mod.rs) — three `push_*` methods gain `span: Span` parameter. Update the implementor `CheckSchemeCtx` (src/check.rs) to use the threaded span. Update all ~16 callers across 4 external crates to pass real spans.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Driver:** the user invoked the no-deferrals rule (`docs/arc/2026/05/138-checkerror-spans/CRACKS-AUDIT.md`). SchemeCtx is item 2 of 4 cracks; F2 closes it.

## Read in order

1. `docs/arc/2026/05/138-checkerror-spans/CRACKS-AUDIT.md` — F2 charter.
2. `docs/arc/2026/05/138-checkerror-spans/SCORE-SLICE-1-FINISH.md` — original observation; specifically the "SchemeCtx trait gap" substrate observation section.
3. `src/rust_deps/mod.rs` lines 88–115 (trait def).
4. `src/check.rs` lines 8375–8429 (CheckSchemeCtx impl).
5. The 4 caller files for context (cursor.rs, auto.rs, codegen.rs, shim.rs).

## What to do

### 1. Trait expansion (src/rust_deps/mod.rs)

Modify the SchemeCtx trait — three methods gain `span: Span` parameter:

```rust
fn push_type_mismatch(
    &mut self,
    callee: &str,
    param: &str,
    expected: String,
    got: String,
    span: crate::span::Span,
);

fn push_arity_mismatch(
    &mut self,
    callee: &str,
    expected: usize,
    got: usize,
    span: crate::span::Span,
);

fn push_malformed(
    &mut self,
    head: &str,
    reason: String,
    span: crate::span::Span,
);
```

Add `use crate::span::Span;` if needed.

### 2. CheckSchemeCtx impl (src/check.rs)

Update the three method impls to use the new `span` parameter:

```rust
fn push_type_mismatch(&mut self, callee: &str, param: &str, expected: String, got: String, span: Span) {
    self.errors.push(CheckError::TypeMismatch {
        callee: callee.into(),
        param: param.into(),
        expected,
        got,
        span,  // arc 138 F2: real span threaded through
    });
}
// similar for push_arity_mismatch, push_malformed
```

DELETE the 3 `// arc 138: no span — SchemeCtx trait...` rationale comments.

### 3. Caller updates — 16 sites across 4 files

Each caller is inside a RustScheme function with signature `fn(args: &[WatAST], ctx: &mut dyn SchemeCtx) -> Option<TypeExpr>`. The `args` slice IS in scope. Threading patterns:

- **For arity mismatches:** the call-form span ISN'T in args directly. Use `args.first().map(|a| a.span().clone()).unwrap_or_else(Span::unknown)` OR if the scheme function is wrapped in a way that has the call-form span, use that. Simpler: use `args[0].span()` if any arg exists; `Span::unknown()` for arity-zero calls (rare; e.g., `:rust::telemetry::uuid::v4`).
- **For type mismatches:** use `args[i].span().clone()` for the offending argument's span.
- **For malformed:** use the offending node's span.

Update sites:
- `crates/wat-telemetry-sqlite/src/cursor.rs`: 4 sites (lines ~612, 619, 634, 647)
- `crates/wat-telemetry-sqlite/src/auto.rs`: 8 sites (lines ~122, 128, 198, 204, 215, 260, 266, 277)
- `crates/wat-macros/src/codegen.rs`: 3 sites (lines ~434, 461, 488) — **proc-macro emit; the emitted code needs to include the span argument; since `args` is in scope of the emitted function, the emitted code can use `args[i].span().clone()` etc.**
- `crates/wat-telemetry/src/shim.rs`: 1 site (line 35) — `:rust::telemetry::uuid::v4` is arity-0; `args.first().map(...).unwrap_or_else(Span::unknown)` is the cleanest pattern.

### 4. wat-macros codegen.rs special handling

The proc-macro emits code that calls push_*. After F2, the emitted code must include the new `span` argument. Look at lines ~434, 461, 488 — these are inside a `quote! { ... }` block. The emitted code receives `args: &[WatAST]` at runtime. Use the appropriate `args[i].span().clone()` pattern in the emit.

### 5. Pattern preservation

Use the established slice 1/2 patterns for span source choice:
- **Pattern A** (args[i].span()) for type mismatches on a specific arg
- **Pattern B** (collective list/call-form span) for whole-form errors when available
- **Pattern E** (Span::unknown() with rationale) ONLY when arity-zero calls genuinely have no arg span available

## Constraints

- ALL FILES that consume SchemeCtx are in scope. Modify trait def + 1 impl + ~16 callers across 4 external files.
- NO new variants. NO Display string changes.
- NO commits, NO pushes.
- All 6 existing arc138 canaries continue to pass.
- Workspace tests pass: `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` returns empty.
- The proc-macro change (codegen.rs) must produce code that compiles AND runs correctly — the emitted runtime code is exercised by tests across the workspace.

## Reporting back

Comprehensive (~400 words):

1. **Diff stat:** files modified (target: 6 — src/rust_deps/mod.rs, src/check.rs, cursor.rs, auto.rs, codegen.rs, shim.rs).
2. **Trait expansion confirmed:** 3 methods gained `span: Span`.
3. **CheckSchemeCtx impl updated:** 3 methods use the threaded span; 3 rationale comments deleted.
4. **Caller distribution:** sites updated per file; pattern distribution (A/B/E).
5. **Proc-macro emit:** the codegen.rs `quote! { ... }` blocks now emit code that passes span. Confirm the emit is syntactically + semantically correct.
6. **Verification:** all 6 canaries pass; workspace tests pass.
7. **Honest deltas:** anything unexpected (test pattern updates, helper sigs broadened in callers, anything that surprised you).
8. **Four questions** applied.

## Why this is medium-complexity

Trait expansion touches all implementors AND all callers. The implementor count is small (1 — CheckSchemeCtx) but caller count is moderate (~16). The proc-macro special case (codegen.rs) requires generating different emitted code, not just modifying call sites. Estimated 30-45 min.
