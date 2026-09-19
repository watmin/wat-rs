## CONFORMARE — Cast Report (TARGET 2 — the compile side and its spec)

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.
> HTML entities (`&lt;` `&gt;` `&amp;`) are artifacts of the agent's own output encoding.

### SCOPE — commands + zeros

```
grep -rnE '^\s*(pub\s+)?(enum|struct)\s+\w*(Error|Kind)\b' <20 .rs files>
  → src/rete/validate/error.rs:23   pub enum ReteCheckErrorKind
    src/rete/validate/error.rs:435  pub struct ReteCheckError
    src/rete/reachability.rs:156    enum DefectKind          (excluded, see below)

grep -rn 'impl From<' <20 .rs files>          → ZERO matches (confirmed twice, exit 1)
grep -rn 'Span::unknown' <20 .rs files>       → ZERO matches
```

Beyond the grep pattern above (which only catches `*Error`/`*Kind` names), reading located **two more error-type families genuinely in scope**:

- `src/rete/expr_ir/mod.rs:188-199` — `LowerError { span: Span, kind: LowerErrorKind }`, `LowerErrorKind::{Unsupported, NonLexicalCallee, Unbound}`.
- `src/rete/purity.rs:174-178` — `AxisViolation { span: Span, head: String, axis: Axis }` (flat, no kind enum).
- `src/rete/purity.rs:1667-1686` — `ReteDefnCheckError { span: Span, kind: ReteDefnCheckErrorKind }`, wrapped in a matchable `ReteDefnCheckOutcome::{Ok, Err}` — a value, not a raise (mirrors the `FireOutcome` family from Prior Art, deliberately).
- `src/rete/validate/error.rs:563-567` — `KwargsReorderError { span: Span, field: String }` (`pub(crate)`, internal helper).

**Excluded from the error-type audit, with reason:**
- `src/rete/reachability.rs:124-148` (`Verdict`) / `:156-169` (`DefectKind`) — a self-test calibration ledger's classification enum for a coverage-generator, never carried in a `Result<_, E>` a caller of the rete wall sees, never rendered to a user. Not diagnostic surface.
- `src/rete/step_payload.rs:41` `Result<WatAST, String>` — the `String` becomes an in-band "why" note *embedded as a value* inside a `:wat::rete::explain::constraint-not-rendered` marker (`step_payload.rs:71-80`), not a raised diagnostic. Rendering-layer concern, excluded per the spell's own "What conformare does NOT audit."
- `src/rete/reachability.rs:1209,2052` `Result<_, String>` — same calibration-instrument file; same exclusion.
- The wat spec half — **defines no error type of its own.** The only error-shaped forms are calls to `:wat::core::macro-error "<string>"` (`wat/rete/syntax.wat:67,129`), a core primitive defined outside this target.
- `EvalBreak`/`RuntimeError`/`RuntimeErrorKind` — **defined in `src/value/signal.rs`, outside the target**; read only to resolve the threading question, not audited as a definition. It is already Pattern A (`signal.rs:99-107`: "The `span` field is mandatory at construction — Rust's struct-literal rule makes a span-less `RuntimeError` uncompilable").

### Audit — `ReteCheckErrorKind` / `ReteCheckError` / `ReteCheckErrors`

**Pattern:** A (outer struct + kind enum), matching the file's own header comment (`error.rs:432`: *"Pattern A (mirrors `crate::check::error::CheckError`): span at the outer struct, kind carries variant data"*).

- **Obvious?** YES. `ReteCheckError { span: Span, kind: ReteCheckErrorKind }` — one look tells you every kind carries a span; none of the 12 `ReteCheckErrorKind` variants (`error.rs:23-319`) declares its own `span`.
- **Simple?** YES. Field access is `err.span` — one path, no per-variant match.
- **Honest?** YES. `span: Span` (not `Option<Span>`), no `Default`, no builder with an omittable step. A struct literal cannot compile without supplying `span`. Checked empirically: every one of the 15 construction sites in `typing.rs` (60, 131, 155, 413, 441, 462, 770) and `mod.rs` (364, 699, 740, 975, 988, 1035, 1060, 1168, 1243) supplies a real, deliberately-chosen AST span — **zero** use `Span::unknown()` or `rust_caller_span!()`.
- **Good UX?** YES — type-system enforced, not practitioner convention.

Notably, `typing.rs:76-97` and `mod.rs:90-97` document a **past** instance of exactly the defect class this ward hunts (a producer that took an enclosing form's span instead of the field's own), and record it as fixed: *"A `Span` parameter accepts the clause's, the fact's and the field's with equal ease, so the promise had no way to be wrong out loud... Taking the NODE makes the wrong span unwritable at the call."* This target has already been through a conformare-shaped remediation once.

**Constructor surface:** cannot be built spanless. `malformed()` (`error.rs:569-578`) is the only free-function constructor and also demands `span: Span`.

**Collection (`ReteCheckErrors(pub Vec<ReteCheckError>)`, `error.rs:457`):** preserves the guarantee trivially — a `Vec` of an already-mandatory-span element type, so no dilution is structurally possible. `WatError::location()` for the plural (`error.rs:526-528`) legitimately returns `Nil` (a batch of N located errors has no single location); each element's own `location()` still returns its span (`error.rs:502-504`).

**Verdict: CONFORMS.** No retrofit warranted.

### The three secondary error types found in scope

- **`LowerError`/`LowerErrorKind`** (`expr_ir/mod.rs:188-199`) — same Pattern A, same guarantees: three constructors (`:202-221`) all require `span: Span`. Its `into_eval()` (`:223-252`) converts to the out-of-scope `EvalBreak`, **preserving `self.span` in all three arms**. One consumer, `matcher.rs:854`, discards it via `.ok()` — investigated and judged **not a finding**: that is the fire-time interpreted matcher answering "does this clause hold" as `Option`, the same "unhandled clause = no match" convention the file states three times (`matcher.rs:785`, `:800`). A **solvere** question, explicitly outside conformare's scope. CONFORMS.
- **`ReteDefnCheckError`/`Kind`/`Outcome`** (`purity.rs:1659-1686`) — Pattern A, header says so explicitly. Both construction sites (`:1773-1780`, `:1785-1791`) supply real located spans. CONFORMS.
- **`AxisViolation`** (`purity.rs:174-183`) — flat struct, no kind enum (doesn't need one). `span: Span` mandatory, sole constructor `AxisViolation::at()`. All ~20 production call sites pass a real threaded span. **One exception, flagged below.**

### Finding — L2, undocumented-by-rune span exception

**`src/rete/purity.rs:1513`**, inside `classify_native_fn`:
```rust
Err(AxisViolation::at(crate::rust_caller_span!(), path, axis))
```
This is the one production call site (outside `#[cfg(test)]`) that raises `AxisViolation` with `rust_caller_span!()` instead of a real user span — genuinely justified by domain (`purity.rs:168-169`: *"`classify_native_fn` / unregistered names use `rust_caller_span` (no body AST)"*). Its one live caller, `src/freeze.rs:805` (outside target), is itself a documented dead-in-practice defensive arm (`purity.rs:1494-1500`).

The domain justification is real and correctly documented in prose. What is missing is the ward's **required structural marker**: no `// rune:conformare(spanless-by-domain) — <reason>` sits at line 1513, so the exception is invisible to any future conformare re-cast or grep-for-runes census — it survives only as long as someone reads the prose above the struct definition, 1,300 lines away.

Cascade: 1 call site inside target, 1 outside (`freeze.rs:805`, out of scope).

### Pattern recommendation

**None of the four in-scope error types warrants a retrofit.** All four already use **Pattern A**, independently and consistently — the substrate-wide answer this cast would otherwise have had to pick is already in force. The four questions pass on all four. The lone gap is not a shape defect but a missing rune tag on one already-justified, already-dead-in-practice exception.

### Runes

`grep -rn 'rune:conformare' <20 files>` → none. One should exist at `purity.rs:1513`; it does not.

### FINDINGS

One finding: L2, `src/rete/purity.rs:1513`. All four in-scope error types conform fully to Pattern A with zero constructible spanless instances and zero span-discarding `From`/conversion boundaries (0 `impl From` in the entire 20-file Rust surface, confirmed by grep, exit 1).
