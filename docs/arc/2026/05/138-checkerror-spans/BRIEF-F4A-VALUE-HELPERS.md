# Arc 138 F4a — Sonnet Brief: thread span through Value-shaped helpers

**Goal:** add `span: Span` parameter to ~11 Value-shaped helper functions across src/spawn.rs (4), src/string_ops.rs (1), src/io.rs (5+). Update each call site to pass the appropriate span. Each helper currently takes `(op: &str, v: Value)` or similar and emits `RuntimeError::TypeMismatch { ..., span: Span::unknown() }` because it has no AST context. After F4a, helpers take `(op: &str, v: Value, span: Span)` and propagate the span into emitted errors.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Driver:** the user invoked the no-deferrals rule (`docs/arc/2026/05/138-checkerror-spans/CRACKS-AUDIT.md`). F4a is the first sub-slice of F4 — the Value-shaped API gap.

## Read in order

1. `docs/arc/2026/05/138-checkerror-spans/CRACKS-AUDIT.md` — F4 decomposition; F4a is sub-item 1 of 3.
2. `docs/arc/2026/05/138-checkerror-spans/SCORE-F3-IOTRAIT.md` — just-shipped F3; explicitly named the "Value-only helpers (expect_reader/writer/i64/string/vec_u8) — F4 territory" leftover.
3. `docs/arc/2026/05/138-checkerror-spans/SCORE-F2-SCHEMECTX.md` — trait-expansion + caller-update pattern (F4a is similar but for free functions).

## Helpers to update

### src/spawn.rs (4 helpers)

- `arity_2(op: &str, args: &[WatAST])` — line ~223. Already takes args; broaden to take `list_span: &Span` for the arity error.
- `expect_string(op: &str, v: Value)` — line ~236. Add `span: Span`.
- `expect_option_string(op: &str, v: Value)` — line ~249. Add `span: Span`.
- `expect_vec_ast(op: &str, v: Value)` — line ~272. Add `span: Span`.

### src/string_ops.rs (1 helper)

- `two_strings(op: &str, ...)` — line ~267. Likely already takes args (check signature); add `span: Span` if not derivable.

### src/io.rs (5 helpers leftover from F3)

- `expect_reader(op: &str, v: Value)` — line ~716. Add `span: Span`.
- `expect_writer(op: &str, v: Value)` — line ~729. Add `span: Span`.
- `expect_i64(op: &str, v: Value)` — line ~742. Add `span: Span`.
- `expect_string(op: &str, v: Value)` — line ~755. Add `span: Span`.
- `expect_vec_u8(op: &str, v: Value)` — line ~768. Add `span: Span`.

## What to do

For each helper:
1. Add `span: Span` (or `list_span: &Span` for whole-form errors) to signature.
2. Inside the helper body, replace `Span::unknown()` in error construction with the threaded span.
3. DELETE the `// arc 138: no span — <reason>` rationale comments at those sites (no longer applicable).
4. Update every caller to pass the appropriate span:
   - **Pattern A:** when the helper is checking a specific arg's value, caller passes `args[i].span().clone()`.
   - **Pattern B:** when the helper is checking the call form as a whole (arity_2), caller passes `list_span.clone()`.

## Constraints

- Files modified: src/spawn.rs + src/string_ops.rs + src/io.rs. NO others.
- NO new variants. NO Display string changes.
- NO commits, NO pushes.
- All 6 existing arc138 canaries continue to pass.
- Workspace tests pass: `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` returns empty.
- Span::unknown() count in these 3 files should drop substantially. Any leftover (e.g., the helpers themselves used in test code) carries `// arc 138: no span — <reason>` rationale.

## Out of scope

- src/edn_shim.rs Pattern E sites — correct architecture (EDN parser layer); leave as-is.
- src/rust_deps/marshal.rs — F4b (separate slice — FromWat trait expansion).
- src/time.rs / fork.rs / sandbox.rs OS errors — genuine Pattern E.
- ThreadOwnedCell::with_mut — F4c (separate slice).

## Reporting back

Comprehensive (~400 words):

1. **Diff stat:** files modified (target: 3).
2. **Helpers updated:** list per file with new signatures.
3. **Caller distribution:** count of call sites per file, pattern split (A/B).
4. **Pre/post Span::unknown() counts** in the 3 files.
5. **Verification:** all 6 canaries pass; workspace tests pass.
6. **Honest deltas:** any helper that has tests calling it directly (test-code call sites would need span: Span::unknown() — fine in test context).
7. **Four questions** applied.

## Why this is medium-small

Value-shaped helpers are well-bounded — each helper is small (5-15 lines), each has 2-10 callers in the same file. ~11 helpers × ~3 callers each = ~33 caller updates plus 11 signature changes. Estimated 15-25 min sonnet runtime, comparable to F2 (6 min for trait expansion + 16 callers).
