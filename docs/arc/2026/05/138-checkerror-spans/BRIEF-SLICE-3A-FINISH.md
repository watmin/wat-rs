# Arc 138 Slice 3a-finish — Sonnet Brief: thread real spans into RuntimeError emission stubs

**Goal:** replace `Span::unknown()` with the most relevant local span at every emission site marked `// arc 138 slice 3a-finish: span TBD` in `src/runtime.rs`. The variant fields, Display arms, and helper signatures are already in place (slice 3a sonnet portion + orchestrator recovery). The remaining 300 stub sites need real spans threaded.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

**Scope:** ONE file — `src/runtime.rs`. Six RuntimeError variants gained `span: Span` field (or `(_, Span)` tuple element) at slice 3a: `ArityMismatch`, `TypeMismatch`, `MalformedForm`, `NoMacroRegistry`, `NoStepRule`, `NoEncodingCtx`, `NoSourceLoader`, `EvalForbidsMutationForm`, `ChannelDisconnected`, `BadCondition`, `NotCallable`, `MacroExpansionFailed`, `PatternMatchFailed`, `EffectfulInStep`, `AssertionFailed`, plus tuple variants `UnboundSymbol(String, Span)`, `UnknownFunction(String, Span)`, `MalformedDelimiter(String, Span)`, `ReservedPrefix(String, Span)`, `DuplicateDefine(String, Span)`, `DivisionByZero(Span)`, `DefineInExpressionPosition(Span)`. The struct-literal constructors at every emission site currently fill that field with `Span::unknown()` and a `// arc 138 slice 3a-finish: span TBD` marker. Replace each with the most relevant local span.

NO substrate-design changes. NO new variants. NO new helpers. NO Display changes. NO commits. ONLY thread spans.

## Read in order — your contract

1. `docs/arc/2026/05/138-checkerror-spans/DESIGN.md` — the arc's framing.
2. `docs/arc/2026/05/138-checkerror-spans/SCORE-SLICE-3A.md` — the partial-ship that produced the 300 stub queue.
3. **The helper-signature pattern already established by slice 3a:** ~10+ helpers take `list_span: &Span` (e.g., lines 2035, 2098, 2150, 2203, 2341, 2394, 3178, 3242, 3452, 3527 in `src/runtime.rs`). When a builtin-impl function has `list_span` in scope, that's the call-form span — use it for whole-form errors. For arg-specific errors, use `args[i].span().clone()`.
4. **Worked-example sites** — the ~140 already-real-spanned RuntimeError emissions from slice 3a sonnet portion (NOT marked with `arc 138 slice 3a-finish`). Search `src/runtime.rs` for `RuntimeError::TypeMismatch {` followed by NOT-`arc 138 slice 3a-finish` to see real-span usage.
5. The variant defs at lines ~845-1030 in `src/runtime.rs` to confirm the field names.
6. The Span shape: `src/span.rs` — `Span::unknown()` is the sentinel; `WatAST::span() -> &Span`.

## What to produce

Every line marked `// arc 138 slice 3a-finish: span TBD` immediately above `span: Span::unknown()` (or as part of a tuple constructor) gets its span replaced with the most relevant local span. Then DELETE the marker comment.

**Pattern A — args[i] in scope from arity-checked builtin:** when the function does `if args.len() != N { ... }` then proceeds with `args[0]`, `args[1]`, etc., the offending arg span is `args[i].span().clone()`. For type errors on a specific arg, use that arg's span.

**Pattern B — list_span: &Span parameter in scope:** when the enclosing function takes `list_span: &Span`, that's the call-form's outermost span. Use `list_span.clone()` for whole-form errors (arity mismatches, "first argument must be X" malformed, etc.).

**Pattern C — match arm receiving an evaluated value:** when the marker is in a `match eval(&args[i], ...)?` arm or `_ => Err(...)` branch, the OFFENDING value came from `eval(&args[i], ...)` — use `args[i].span().clone()`.

**Pattern D — head keyword binding:** if you see `WatAST::Keyword(k, _) if k == "..."` patterns in scope, the `_` is the keyword's span. Rename to a binding (`WatAST::Keyword(k, head_span)`) and use `head_span.clone()`.

**Pattern E — synthetic / no real span available:** the variant is being constructed in a `runtime_error_to_eval_error_value` re-wrap path or a synthetic dispatcher with no originating AST node. Leave `Span::unknown()` AND replace the marker with `// arc 138: no span — <reason>` (e.g., "synthetic re-wrap from EvalError", "internal dispatcher without AST source").

**Pattern F — needs threading:** the function neither has `list_span` nor `args` nor an enclosing AST node. Two options: (a) add `list_span: &Span` to the function signature and propagate from callers (this is the slice 3a established pattern; broadening is acceptable — same shape as the 23 helpers slice 1 broadened). (b) If broadening crosses a public-API boundary, leave `Span::unknown()` with `// arc 138: no span — <reason>` and call it out as substrate observation.

## Constraints

- ONLY `src/runtime.rs` modified (helper-sig broadening within `src/runtime.rs` is in-scope; cross-file changes are NOT).
- NO substrate-design changes (don't add RuntimeError variants; don't change Display strings; don't add new helpers other than minor span-threading).
- NO test changes UNLESS a test breaks because of a Display change you didn't make — investigate before changing.
- NO commits, NO pushes.
- `cargo test --release --workspace` exit=0 (excluding lab); the canary `runtime::tests::arc138_runtime_error_message_carries_span` MUST still pass.
- Marker count `// arc 138 slice 3a-finish: span TBD` drops substantially. Target: ≤ 30 leftover, each with a `// arc 138: no span — ...` rationale comment.
- DON'T touch the 156 `// arc 138 slice 3b: span TBD` markers in external files — those are slice 3b's queue.
- DON'T touch the 54 pre-existing `Span::unknown()` baseline in synthetic-AST construction (struct-new dispatchers, enum constructors, lambda body wrappers). They're not error emission; they're synthetic AST.

## Size warning + fallback

300 sites is at the upper edge of single-engagement context budget. Slice 3a sonnet portion ran ~36 min + hit context limits with ~300 sites unfilled. Calibration says 300 mechanical-only sites = ~45 min if pattern is uniform.

**If you start running long (~250+ sites done):** stop, report progress honestly, list remaining marker count + line ranges of unfilled sites. Orchestrator re-spawns for the gap. Partial progress is acceptable; INCOMPLETE delivery is honest.

Sweep top-to-bottom in `src/runtime.rs`. Run `cargo test --release -p wat --lib` after each ~75-site batch to catch regressions early.

## What success looks like

1. Every marker site that has a `list_span` or `args[i]` in scope uses the appropriate real span.
2. Workspace tests stay green; canary passes.
3. Leftover `Span::unknown()` instances each have `// arc 138: no span — <reason>` rationale.
4. The diff is single-file (`src/runtime.rs`).
5. NO commits.

## Reporting back

Target ~400 words:

1. **Counts**: BEFORE marker count (300) → AFTER count. Run `grep -c "arc 138 slice 3a-finish" src/runtime.rs` and report both. Run `grep -c "Span::unknown()" src/runtime.rs` and report both (baseline 54 synthetic-AST + initial 380 stubs → final).
2. **Pattern distribution**: how many sites used each of patterns A, B, C, D, E, F. Confirm no F-class sites added cross-file dependencies.
3. **Touched-line range** in `src/runtime.rs` (e.g., "L2000-L20000, single file").
4. **Verification**: `cargo test --release --workspace` totals. Canary test result.
5. **`git diff --stat`** output for `src/runtime.rs`.
6. **Honest deltas** — any helper sigs broadened (Pattern F); any Pattern E rationales; any place where the shape didn't fit the patterns above.
7. **Four questions applied** to your output.

## What this slice tests (meta)

The hypothesis: with 140 worked sites already in place + helper sig pattern established + variant defs settled, sonnet can sweep 300 stub sites in one engagement OR honestly report partial progress for re-spawn.

Begin by reading the worked sites + helper sigs. Plan the pattern split before editing. Sweep top-to-bottom. Run `cargo test` after each ~75-site batch. Report.
