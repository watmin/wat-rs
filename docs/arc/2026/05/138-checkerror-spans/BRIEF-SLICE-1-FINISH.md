# Arc 138 Slice 1 (Finish) — Sonnet Brief: thread real spans into CheckError emission sites

**Goal:** replace `Span::unknown()` with the most relevant local span at every `CheckError` emission site in `src/check.rs`. The variant fields, Display arms, `diagnostic()` arms, and 6 worked-example sites are already in place. The remaining ~208 emission sites need real spans threaded.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Scope:** ONE file — `src/check.rs`. Six `CheckError` variants gained `span: Span` field at the top of slice 1: `ArityMismatch`, `TypeMismatch`, `ReturnTypeMismatch`, `UnknownCallee`, `MalformedForm`, `CommCallOutOfPosition`. The struct-literal constructors at every emission site currently fill that field with `Span::unknown()`. Replace each with the most relevant local span from the AST node being checked.

NO substrate-design changes. NO new variants. NO new helpers. NO Display / diagnostic changes. NO commits. ONLY thread spans.

## Read in order — your contract

1. `docs/arc/2026/05/138-checkerror-spans/DESIGN.md` — the arc's framing. Read the "Reference instance" and "Why this matters — agents need to navigate" sections; the first commit's INSCRIPTION-style rationale is the why.
2. **The 6 worked-example sites** in `src/check.rs` (currently using real spans, not `Span::unknown()`):
   - Line ~2632 — `ReturnTypeMismatch` in `check_user_define`. Uses `func.body.span().clone()`.
   - Line ~3174 — `TypeMismatch` in scheme call-site arg loop. Uses `arg.span().clone()`.
   - Line ~5786 — `TypeMismatch` in spawn-args loop. Uses `arg.span().clone()`.
   - Line ~7838 — `TypeMismatch` in `:wat::core::vec`. Uses `arg.span().clone()`.
   - Line ~7886 — `ReturnTypeMismatch` in `infer_lambda`. Uses `body.span().clone()`.
   - Line ~7966 — `TypeMismatch` in `:wat::core::and/or`. Uses `arg.span().clone()`.
3. The variant defs at lines ~73-115 in `src/check.rs` to confirm the field names.
4. The Span shape: `src/span.rs` — `Span::unknown()` is the sentinel; `Display` is `file:line:col`; `is_unknown()` distinguishes synthetic from real; `WatAST::span() -> &Span`.

## What to produce

Every `errors.push(CheckError::{TypeMismatch,ArityMismatch,ReturnTypeMismatch,UnknownCallee,MalformedForm,CommCallOutOfPosition} { ... })` in `src/check.rs` gets its `span:` field updated:

- **Pattern A — arg-iteration loop**: when an `arg` (or `arg_ty`/`expected` paired with an arg) is in scope from a `for (i, arg) in args.iter()` or similar, use `arg.span().clone()`. The offending argument is the user-clickable site.
- **Pattern B — items[0] / head keyword**: when the diagnostic is about the call as a whole (arity, callee resolution), bind the head keyword's span at the destructure (e.g., `if let WatAST::Keyword(k, head_span) = head` — there are MANY existing `WatAST::Keyword(k, _)` patterns where `_` is the span; rename the binding to use it). Use `head_span.clone()` for the call.
- **Pattern C — form list span**: when neither arg nor head is the right level (whole let* binding, whole match form), look up the AST node parameter to the enclosing function (`form: &WatAST`, `node: &WatAST`, `items: &[WatAST]`'s parent, etc.) and use its `.span().clone()`.
- **Pattern D — synthetic poison / retired keyword**: emission sites that fire on `WatAST::Keyword(k, _) if k == "..."` patterns. Bind the span and use it. The keyword token's span IS the right span — that's what the user typed.
- **Pattern E — genuinely no span available**: synthetic check rules with no originating AST node (rare). Leave `Span::unknown()` AND add a one-line `// arc 138: no span — <reason>` comment justifying it.

For each emission site:
1. Read 5-15 lines of context above to find the most-relevant span source.
2. If `arg`, `node`, `form`, `binding`, or similar is in scope as a `&WatAST` → use `<that>.span().clone()`.
3. If the head keyword pattern uses `_` → rename to a span binding.
4. If a parent function takes `node: &WatAST` and the emission is in a deep call, propagate the span as a parameter or use a closer node already in scope.
5. Document the rare leftover Span::unknown() cases.

## Constraints

- ONLY `src/check.rs` modified.
- NO substrate-design changes (don't add CheckError variants; don't change Display strings; don't change diagnostic arms; don't add helpers).
- NO test changes UNLESS a test's exact-string-match breaks because of a Display change you didn't make — investigate before changing.
- NO commits, NO pushes.
- `cargo test --release --workspace` exit=0; the canary `check::tests::type_mismatch_message_carries_span` MUST still pass.
- The `Span::unknown()` count drops substantially. Target: ≤30 leftover sites, each with a `// arc 138: no span — ...` rationale comment.

## What success looks like

1. Every emission site that has an obvious local `&WatAST` source uses that span.
2. Workspace tests stay green; canary passes.
3. Leftover `Span::unknown()` instances each have a justifying comment.
4. The diff is single-file (`src/check.rs`) and span-only (no other changes).
5. NO commits.

## Reporting back

Target ~400 words:

1. **Counts**: BEFORE `Span::unknown()` count → AFTER count. Run `grep -c "span: Span::unknown()" src/check.rs` and report both.
2. **Pattern distribution**: how many sites used each of patterns A, B, C, D, E. Confirm no E-class sites slipped through unjustified.
3. **Touched-line range** in `src/check.rs` (e.g., "L1330-L11000, ~210 line edits, single file").
4. **Verification**: `cargo test --release --workspace` totals. Canary test result.
5. **`git diff --stat`** output for `src/check.rs`.
6. **Honest deltas** — anything the worked-example patterns didn't anticipate. Specifically: any emission site where you had to thread a span through additional fn signatures (added a parameter), or any site where the AST shape didn't have an obvious span source.
7. **Four questions applied** to your output.

## What this slice tests (meta)

The hypothesis: arc 138's two-pattern infrastructure (variant + Display + diagnostic + 6 worked sites) is sufficient teaching for a sonnet sweep across the remaining 208 sites. Patterns are mechanical-with-local-judgment; same shape as arc 109's symbol-migration sweeps.

If you ship clean — the foundation work is durable; arc 138 slices 2-6 (TypeError, RuntimeError, MacroError group, ConfigError, doctrine) can dispatch with confidence.
If you encounter ambiguity, name it; the next slice's BRIEF will incorporate.

Begin by reading the worked sites. Plan the pattern split before editing. Sweep top-to-bottom in `src/check.rs`. Run `cargo test` after each ~50-site batch to catch regressions early. Report.
