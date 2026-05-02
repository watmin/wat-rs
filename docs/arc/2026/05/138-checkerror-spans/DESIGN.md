# Arc 138 — Errors carry point-in-code coordinates

**Status:** opened 2026-05-03. **Widened 2026-05-03 mid-session** —
original scope was `CheckError` only; audit surfaced the gap is
project-wide. Every error type a user/agent sees from the substrate
either carries source coordinates or doesn't. Most don't. This arc
fixes that uniformly.

**Blocks** arc 139 (generic-T tuple return).

## Why this matters — agents need to navigate

The receiver of an error message is increasingly an agent. When
sonnet hits an error without file:line:col, it has to GUESS where
the offending form lives:

- It greps the source for the offending shape.
- If multiple matches, it tries each.
- It guesses again on the next layer of confusion.
- Iteration cost compounds.

The user observes: *"sonnet takes way longer to do work than I
expect.. it's having to guess way too much."* Spans collapse the
guess-loop. The fix is mechanical (add `Span`, thread through
emission sites). The payoff is structural — every future debug
session, by every user (human or agent), gets shorter.

This is foundation infrastructure. It supersedes ergonomic
work (`do` form, `/dissimulans`) until shipped.

## Audit — what carries spans today

| Error type | Spans? | Why |
|---|---|---|
| `LexError` | ALL 5 variants carry `Position` | lexer was first; obviously needed |
| `ParseError` | 4 of 5 variants carry `Span` (`Empty` has no source loc — n/a) | parser learned the lesson |
| `CheckError` | 4 newer variants carry `Span`; 5 OG variants don't | **gap** |
| `TypeError` | 1 of 8 variants (`MalformedVariant`, arc 130) | **gap** |
| `RuntimeError` | Internal signals (`TailCall.call_span`); user-facing variants don't | **gap** |
| `MacroError` | 0 of 9 variants | **gap** |
| `EdnReadError` | 0 of 6 variants | **gap** |
| `ClauseGrammarError` | 0 of 7 variants | **gap** |
| `LowerError` | 0 of 13 variants | **gap** |
| `ConfigError` | uses `form_index` (positional only) | **gap** |
| `LoadError`, `StdlibError`, `HashError` | uses paths / algorithms | n/a — content-addressed payloads, no source coords |

**Why the drift accumulated:** the parser/lexer needed
coordinates to be usable. Every other error type accreted
variants over time. Each contributor added a new variant without
a span because none of the surrounding variants had one. When a
sufficiently painful session surfaced (arc 117 `ScopeDeadlock`,
arc 109 `BareLegacyPrimitive`, arc 130 `MalformedVariant`), THAT
specific variant got a retrofit — but the surrounding gap stayed.
Nobody ever ran the sweep asking *"all of them?"*

This arc is the sweep.

## Reference instance — what triggered the arc

Arc 135 slice 3 sweep. Sonnet's report quoted:

```
4 type-check error(s):
  - :wat::core::rest: parameter #1 expects :Vec<?302>; got :(i64,bool,String)
  - :wat::core::let*: parameter binding 'rest1' expects :(bool,String); got :Vec<?302>
```

No file. No line. No column. Sonnet had to grep the source for
the offending shape, guess which deftest body held it, and try
the workaround in each. Lost ~24 minutes to navigation guessing
that should have taken seconds. Filed as the trigger; the audit
surfaced the wider pattern.

## Scope — five execution slices + doctrine slice

### Slice 1 — `CheckError` 5 OG variants

```rust
TypeMismatch       { callee, param, expected, got, span: Span }
ArityMismatch      { callee, expected, got, span: Span }
ReturnTypeMismatch { function, expected, got, span: Span }
UnknownCallee      { callee, span: Span }
MalformedForm      { head, reason, span: Span }
```

~110 emission sites in `src/check.rs`. Each construction has
local access to a `WatAST` node — the AST node it's checking
— and that node has `.span()`. Threading is mechanical.

Display: prefix `{span}: ` to existing message; skip if
`span.is_unknown()`. Same precedent as `ScopeDeadlock`.

`diagnostic()` arm: add `.field("span", span.to_string())` when
non-unknown.

Verification: re-run `wat-tests/tmp-3tuple-probe.wat`; type errors
now name the file:line:col. **Unblocks arc 139.**

### Slice 2 — `TypeError` 7 variants

```rust
DuplicateType        { name, span: Span }
ReservedPrefix       { name, span: Span }
MalformedDecl        { head, reason, span: Span }
MalformedName        { raw, reason, span: Span }
MalformedField       { reason, span: Span }
MalformedTypeExpr    { raw, reason, span: Span }
AnyBanned            { raw, span: Span }
CyclicAlias          { name, span: Span }
AliasArityMismatch   { name, expected, got, span: Span }
InnerColonInCompoundArg { raw, offending, span: Span }
```

Type-registration runs over user `(:wat::core::struct ...)` /
`(:wat::core::enum ...)` / `(:wat::core::typealias ...)` forms;
each carries the form's outer span.

### Slice 3 — `RuntimeError` user-facing variants

Audit which variants surface to user code (vs. internal signals
like `TailCall` / `TryPropagate` that never escape `:user::main`).
The user-facing list (preliminary):

```rust
UnboundSymbol(String, Span)
UnknownFunction(String, Span)
NotCallable      { got, span: Span }
TypeMismatch     { op, expected, got, span: Span }
ArityMismatch    { op, expected, got, span: Span }
BadCondition     { got, span: Span }
MalformedForm    { head, reason, span: Span }
ParamShadowsBuiltin(String, Span)
DivisionByZero(Span)
DuplicateDefine(String, Span)
ReservedPrefix(String, Span)
DefineInExpressionPosition(Span)
EvalForbidsMutationForm { head, span: Span }
ChannelDisconnected     { op, span: Span }
NoEncodingCtx           { op, span: Span }
NoSourceLoader          { op, span: Span }
NoMacroRegistry         { op, span: Span }
MacroExpansionFailed    { op, reason, span: Span }
PatternMatchFailed      { value_type, span: Span }
EffectfulInStep         { op, span: Span }
NoStepRule              { op, span: Span }
AssertionFailed         { message, actual, expected, span: Span }
```

Internal signals (`TailCall`, `TryPropagate`, `OptionPropagate`)
already carry spans where needed; preserve.

Threading source: at runtime, `eval_*` functions take an AST node
and a span-tracking call stack. Most error sites have the span
trivially in scope.

### Slice 4 — Remaining error types

- `MacroError` 9 variants → all gain `span: Span`. Macroexpansion
  fires while walking a `WatAST`; nodes are in scope.
- `EdnReadError` 6 variants → spans on EDN parse errors. EDN parser
  already tracks position internally; surface it.
- `ClauseGrammarError` 7 variants → form_match clauses are AST
  fragments; carry the clause's span.
- `LowerError` 13 variants → algebra-core MVP path; spans match
  the form being lowered.

### Slice 5 — `ConfigError` form_index → Span

`ConfigError` uses `form_index: usize` (which form in the file
fired). Convert to `span: Span` for consistency. Setter forms
have spans; just thread through.

### Slice 6 — Doctrine + closure

- CONVENTIONS.md § new "Errors carry coordinates" — the rule + the
  three exceptions (content-addressed payloads, paths, algorithms)
  + drift-prevention guidance for new variant authors.
- `/wards` skill (or whichever ward owns it) gains a check: a new
  error variant without a span field needs a documented reason in
  the variant's doc comment.
- INSCRIPTION + USER-GUIDE row + 058 changelog row.

## Display convention

`Span::Display` is `file:line:col`. Two patterns:

- **Prefix** — `"{span}: {existing message}"` — the
  `ScopeDeadlock` precedent. Default for new spans.
- **Inline `at {span}`** — `"... at {span} ..."` — used by
  `BareLegacyPrimitive` etc. when the message reads naturally
  with the location embedded.

Default to **prefix**. Inline only when the message is a
complete sentence already.

`is_unknown()` skip: synthetic spans (`Span::unknown()`) shouldn't
print `<runtime>:0:0` noise. Display arm checks `is_unknown()`
and falls back to the un-prefixed message.

## Compatibility

- Pattern-match-based tests (`matches!(e, X { .. })`) absorb new
  fields automatically — no change.
- Field-destructure tests (`X { callee, param, expected, got }
  => ...`) need `..` rest pattern added. Mechanical.
- Display-string tests need re-baselining. Necessary one-time cost.
- `Span::PartialEq` is structural-transparent; `Hash` is no-op.
  Existing `CheckError`-as-key behaviors unaffected.
- JSON / EDN serialization: `diagnostic()` arms gain a `:span`
  field. External consumers tolerate extra fields by convention.
  Verify wire format on slice 1.

## Done when

- Every error variant in the audit either carries `Span` or has
  a doc comment explaining why it can't (e.g., `LoadError::Fetch`
  references a path; the underlying loader I/O has no source).
- `cargo test --release --workspace` exit=0.
- `tmp-3tuple-probe.wat` shows file:line:col on every error
  surfaced.
- CONVENTIONS § "Errors carry coordinates" rule lands.
- 058 changelog row + INSCRIPTION + USER-GUIDE.
- Arc 139 unblocked.

## Cross-references

- `src/check.rs:72` — CheckError enum.
- `src/types.rs:957` — TypeError enum.
- `src/runtime.rs:829` — RuntimeError enum.
- `src/macros.rs:142` — MacroError.
- `src/edn_shim.rs:206` — EdnReadError.
- `src/form_match.rs:82` — ClauseGrammarError.
- `src/lower.rs:43` — LowerError.
- `src/config.rs:117` — ConfigError.
- `src/span.rs` — Span shape; structural-transparent equality.
- `docs/arc/2026/05/130-cache-services-pair-by-index/REALIZATIONS.md`
  — arc 130 follow-up that added spans to MalformedVariant; the
  precedent that informs this sweep.
- `docs/arc/2026/05/135-complectens-cleanup-sweep/SCORE-SLICE-3.md`
  — surfaced the trigger.

## Failure modes to watch

- **Span availability for synthetic errors.** A check rule that
  fires "no main function found" has no originating node. Use
  `Span::unknown()`; Display skips the prefix.
- **Display backwards compat.** Existing tests matching exact
  error strings break. Update mechanically; track per-slice.
- **Wire format drift.** `diagnostic()` JSON/EDN gains `:span`
  fields. External consumers of these (test harnesses, IDE
  integrations) need verification — likely none today, but check.
- **Doc burden.** Some error variants genuinely don't have a
  source span (path-only errors, algorithm errors). The doctrine
  must NOT require span on those — it requires either span OR
  a documented reason. Avoids forcing `Span::unknown()` everywhere.

## Why this is one arc not six

Single principle ("errors carry coordinates"); single doctrine
slice ($wards rule + CONVENTIONS row); the per-error-type
execution slices share the same threading pattern. Splitting
into six arcs would scatter the doctrine across six commits and
lose the cohesion. Slices preserve incremental ship — each slice
green-tests independently — without dissolving the principle.
