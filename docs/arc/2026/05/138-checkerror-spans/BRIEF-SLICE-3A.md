# Arc 138 Slice 3a — Sonnet Brief: RuntimeError variants + src/runtime.rs sweep

**Goal:** retrofit `Span` onto every USER-FACING `RuntimeError` variant in `src/runtime.rs`. Update Display arms to prefix `<file>:<line>:<col>:` on non-unknown spans. Thread real spans through every emission site IN `src/runtime.rs`. Add a canary test that asserts the rendering contains source coordinates.

**This is slice 3a. Slice 3b sweeps ~100 emission sites in OTHER src/ files (io.rs, time.rs, marshal.rs, fork.rs, string_ops.rs, spawn.rs, assertion.rs, edn_shim.rs, hologram.rs, freeze.rs) — that's a separate sonnet engagement. Slice 3a stays inside `src/runtime.rs`.**

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Scope:** primarily `src/runtime.rs`. The shape:
1. Add `span: Span` (or `Span` positional for tuple variants) to ~22 user-facing variants.
2. Update Display arms.
3. Thread real spans through ~489 emission sites in src/runtime.rs.
4. Add a canary test.
5. Other src/ files compile but their emission sites are sweep-deferred to slice 3b — those sites pass `Span::unknown()` for now (orchestrator commits this as a transient gap; slice 3b sweeps them).

External files MUST compile after slice 3a. The fastest path: each external file gains `Span::unknown()` at every changed emission site (one mechanical pass per external file). The orchestrator views these as transient — slice 3b replaces them with real spans.

NO commits.

## Read in order — your contract

1. `docs/arc/2026/05/138-checkerror-spans/BRIEF-SLICE-3A.md` — this file.
2. `docs/arc/2026/05/138-checkerror-spans/SCORE-SLICE-2.md` — slice 2's clean ship (8/8+4/4); the `_with_span` sibling-API pattern surfaced there. Same pattern likely applies here at any public RuntimeError-emitting function with external callers.
3. `docs/arc/2026/05/138-checkerror-spans/DESIGN.md` — the arc framing. The "Slice 3" section lists the user-facing variant inventory.
4. `src/runtime.rs::RuntimeError` enum at lines ~845-1030 — the variant inventory.
5. `src/check.rs::span_prefix` (line ~362) and `src/types.rs::span_prefix` (slice 2) — the canonical helper shape; mirror in `src/runtime.rs`.
6. `src/runtime.rs::TailCall.call_span` (line ~995) — already has a span field; the IN-FILE worked example.
7. `src/runtime.rs::SandboxScopeLeak` (line ~1010+) — arc 140 already shipped this with `call_span` + `outer_define_span`. Don't touch; it's the second worked example.

## What to produce

### Step A — Variant categorization

Apply spans to the **22 user-facing variants** below. Use the DESIGN's preliminary categorization. **NO span on internal signals** (TryPropagate, OptionPropagate, UserMainMissing) — they're caught before reaching user diagnostics. **Don't touch** TailCall or SandboxScopeLeak (already shipped).

Variants to add `span` to:

| Variant | Current shape | Proposed shape |
|---|---|---|
| `UnboundSymbol(String)` | tuple | `UnboundSymbol(String, Span)` |
| `UnknownFunction(String)` | tuple | `UnknownFunction(String, Span)` |
| `NotCallable { got }` | struct | `NotCallable { got, span: Span }` |
| `TypeMismatch { op, expected, got }` | struct | `TypeMismatch { op, expected, got, span: Span }` |
| `ArityMismatch { op, expected, got }` | struct | `ArityMismatch { op, expected, got, span: Span }` |
| `BadCondition { got }` | struct | `BadCondition { got, span: Span }` |
| `MalformedForm { head, reason }` | struct | `MalformedForm { head, reason, span: Span }` |
| `ParamShadowsBuiltin(String)` | tuple | `ParamShadowsBuiltin(String, Span)` |
| `DivisionByZero` | unit | `DivisionByZero(Span)` |
| `DuplicateDefine(String)` | tuple | `DuplicateDefine(String, Span)` |
| `ReservedPrefix(String)` | tuple | `ReservedPrefix(String, Span)` |
| `DefineInExpressionPosition` | unit | `DefineInExpressionPosition(Span)` |
| `EvalForbidsMutationForm { head }` | struct | `EvalForbidsMutationForm { head, span: Span }` |
| `ChannelDisconnected { op }` | struct | `ChannelDisconnected { op, span: Span }` |
| `NoEncodingCtx { op }` | struct | `NoEncodingCtx { op, span: Span }` |
| `NoSourceLoader { op }` | struct | `NoSourceLoader { op, span: Span }` |
| `NoMacroRegistry { op }` | struct | `NoMacroRegistry { op, span: Span }` |
| `MacroExpansionFailed { op, reason }` | struct | `MacroExpansionFailed { op, reason, span: Span }` |
| `PatternMatchFailed { value_type }` | struct | `PatternMatchFailed { value_type, span: Span }` |
| `EffectfulInStep { op }` | struct | `EffectfulInStep { op, span: Span }` |
| `NoStepRule { op }` | struct | `NoStepRule { op, span: Span }` |
| `AssertionFailed { message, actual, expected }` | struct | `AssertionFailed { message, actual, expected, span: Span }` |

**Variants to LEAVE unchanged** (no span):
- `TryPropagate(Value)` — internal control-flow signal; never user-facing
- `OptionPropagate` — internal control-flow signal
- `TailCall { ... call_span: Span }` — already has call_span (don't dual-add)
- `UserMainMissing` — fires before any user source executes; no span source
- `EvalVerificationFailed { err }` — wraps HashError on content-addressed payload; no source span exists
- `SandboxScopeLeak { ... }` — already shipped via arc 140 with two spans

### Step B — Display arms

Add `fn span_prefix(span: &Span) -> String` near the top of `src/runtime.rs` (or near the impl Display block). Mirror `src/check.rs::span_prefix` exactly.

For each of the 22 variants, prefix the Display rendering with `{span_prefix(span)}` when non-unknown. Keep the message body unchanged otherwise. Internal signals' Display arms (TryPropagate, OptionPropagate, etc.) stay AS-IS.

### Step C — Emission sites in src/runtime.rs

~489 sites in src/runtime.rs. Each site needs a span argument:

| Variant pattern | Best-source heuristic |
|---|---|
| `Err(RuntimeError::UnboundSymbol(name))` | the symbol's AST node span |
| `Err(RuntimeError::UnknownFunction(name))` | the call site span (typically `list_span.clone()` from `eval_call`) |
| `Err(RuntimeError::TypeMismatch { ... })` | the offending arg's span (or the call form span if no arg context) |
| `Err(RuntimeError::ArityMismatch { ... })` | the call form span (typically `list_span` or the head keyword span) |
| `Err(RuntimeError::BadCondition { ... })` | the condition expression's span |
| `Err(RuntimeError::MalformedForm { ... })` | the malformed form's span |
| `Err(RuntimeError::DivisionByZero)` | the arithmetic op's span |
| `Err(RuntimeError::ChannelDisconnected { ... })` | the kernel-comm form's span |
| `Err(RuntimeError::NoEncodingCtx { ... })` etc. | the substrate-primitive call form's span |

Most sites are inside `eval_*` functions that already take a `node: &WatAST` or have access to a list_span. Existing `apply_function` carries `caller_span`. Use what's local.

**Helper signature broadening is allowed.** Same `_with_span` sibling pattern from slice 2 if you need to preserve a public API. Name added parameters in honest deltas.

**For external files (io.rs, time.rs, etc.)** — add `Span::unknown()` at every emission site to keep the workspace compiling. This is a TRANSIENT gap; slice 3b sweeps them. Mark each transient site with `// arc 138 slice 3b: span TBD` so slice 3b's sonnet finds them mechanically.

### Step D — Canary test

Add ONE unit test that asserts a RuntimeError surfaced from a real evaluation carries `<test>:` (file:line:col) in its rendered Display output. Place in `src/runtime.rs::tests`. Trigger any user-facing variant (e.g., `(:wat::core::i64::div 1 0)` → DivisionByZero; or `(:nonexistent-func 1)` → UnknownFunction).

Verification command: `cargo test --release -p wat --lib runtime::tests::arc138_runtime_error_message_carries_span` (or similar — your name choice).

## Constraints

- ONLY `src/runtime.rs` modified plus the 10 external files (transient `Span::unknown()` additions for compile).
- NO new variants beyond span fields. NO Display string changes beyond the prefix.
- TailCall, SandboxScopeLeak, TryPropagate, OptionPropagate, UserMainMissing, EvalVerificationFailed — DO NOT TOUCH.
- NO commits.
- `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading` returns empty.
- The canary passes.
- Each external file's transient `Span::unknown()` site has a `// arc 138 slice 3b: span TBD` comment.

## What success looks like

1. 22 variants gain `span: Span` field (or positional Span for tuple variants).
2. 22 Display arms prefix coords.
3. ~480+ emission sites in src/runtime.rs use real spans (≥ 90% real-spanned; the rest documented).
4. ~100 emission sites in external files use `Span::unknown()` with `// arc 138 slice 3b: span TBD` comments.
5. Canary test passes.
6. Workspace test green excluding lab.
7. NO commits.

## Reporting back

Target ~500 words:

1. **Counts**: `Span::unknown()` in src/runtime.rs (BEFORE = 54 from synthetic-AST-construction sites; AFTER = whatever you transient-mark) and in each external file. Note the synthetic-AST sites (struct-new etc.) DON'T count toward the sweep — they're pre-existing legitimate Span::unknown for synthetic AST construction.
2. **Variant changes**: 22 variants gained span; list confirmed.
3. **Display arms**: 22 arms updated; list of LEFT-AS-IS variants.
4. **Emission distribution in src/runtime.rs**: rough bucket counts per variant.
5. **External file transient additions**: count per file; pattern of marker comments.
6. **Helper-fn signatures broadened**: list any helper that gained span params; any `_with_span` siblings added.
7. **Canary**: name + location + verification.
8. **Verification**: `cargo test --release -p wat --lib`; `cargo test --release --workspace 2>&1 | grep FAILED | grep -v trading | head -5` (empty).
9. **`git diff --stat`** for all touched files.
10. **Honest deltas** — anything beyond the brief; `_with_span` sibling additions; sites where the span source wasn't obvious.
11. **Four questions applied**.

**Do NOT commit.** Orchestrator will independently score against `docs/arc/2026/05/138-checkerror-spans/EXPECTATIONS-SLICE-3A.md` and verify your `git diff --stat` against your reported counts before any commit.

Begin by reading slice 2 SCORE for calibration, then DESIGN slice 3 spec, then `src/runtime.rs::RuntimeError` enum, then TailCall + SandboxScopeLeak as worked-IN-FILE examples. Plan variant + Display + helper-signature changes BEFORE editing. Sweep variant defs first, Display arms second, src/runtime.rs emissions third, external transient stubs fourth, canary fifth. Run cargo test after each batch. Report.
