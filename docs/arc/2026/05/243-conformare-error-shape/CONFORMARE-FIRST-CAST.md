# Conformare — First-cast verdict on the substrate

**Date:** 2026-05-30
**Status:** Captured artifact; the spell's grimoire-earning proof.
**Methodology:** Spell minted at `datamancy.dev/conformare/SKILL.md`; cast via subagent on the wat-rs substrate; agent received the SKILL.md + `scratch/FAILURE-ENGINEERING.md` only — NO orchestrator-prior recommendation included; the four-questions had to converge from first principles.

## Why this capture exists

The user's directive: *"the spell predicts its the solution - prove it."* The spell had to arrive at the substrate's structural answer (Pattern A: outer struct + kind enum) independently through its own discipline. If the spell converged on A through the four-questions, the spell earns its grimoire seat. If it diverged, either the spell needed revision or the orchestrator's pre-recommendation missed something.

The spell converged on Pattern A. This capture is the proof.

## Substrate-wide manifest (20 error types audited)

### Tier 1 — Primary diagnostic-bearing types

**`TypeError`** (`src/types.rs:1412`) — 16 variants. 15 carry `span: Span`. **1 spanless: `CyclicSubtype { child: String, parent: String }`** — emitter at `src/types.rs:435` (`register_subtype` operates on FQDN strings; no AST node in scope). Consumer at `src/function/parse.rs:172` injects `Span::unknown()` explicitly — documented acknowledgment, no rune.

**`RuntimeError`** (`src/runtime.rs:2080`) — 33 variants. 3 spanless with domain rationale (no rune documentation):
- `UserMainMissing` — startup invariant; no AST site
- `EvalVerificationFailed { err: HashError }` — operates on digests
- `TryPropagate(Value)` / `OptionPropagate` — internal control-flow signals; never user-facing

Plus field-name discipline gap: `TailCall { call_span }`, `SandboxScopeLeak { call_span, outer_define_span }`, `PostconditionFailed { body_span, ensure_span }` use non-standard span field names.

**`CheckError`** (`src/check.rs:87`) — 34 variants. ALL carry span data BUT field-name discipline inconsistent: `prior_loc`/`current_loc`/`join_span`/`output_span`/`bind_span` etc. The `diagnostic()` impl routes through 34-arm match — the N-path access pattern conformare flags.

**`ParseStep`** (`src/function/parse.rs:44`) — 6 variants. 5 carry span. **1 spanless: `ArityMismatch { actual: usize }`** — Stone 241.18a's documented NEW-2 residual.

**`ArgSpecError`** (`src/argspec/error.rs:16`) — 7 variants, ALL carry `span: Span`. **CONFORMANT by convention.** The `classify(self) -> (Span, String, String)` method provides single-path span access — the closest existing substrate precedent for what Pattern A enforces structurally.

### Tier 2 — Wrapper types

`StartupError`, `HarnessError`, `Error` (lib.rs) — wrap tier-1 types; span lives in wrapped error. Internal infrastructure variants (e.g., `SigmaFn(String)`, `MainSignature`, `StdioSnapshot`) are domain-spanless and appropriate.

### Tier 3 — Infrastructure types (mostly domain-spanless)

`LexError` (uses byte `Position`, not `Span` — pre-span-labeling tier), `LoadError`/`LoadFetchError` (file-path domain), `HashError` (digest bytes), `ResolveError` (paths not positions), `StdlibError` (baked-in source), `TryRecvError` (channel state).

`ConfigError`, `LowerError`, `MacroError`, `EdnReadError`, `ClauseGrammarError` — **CONFORMANT**; all variants carry span.

`ExtractionError` — `NonPortableCapture { name, type_name, path }` lacks span; **genuine diagnostic gap** — user would benefit from knowing which line captures the non-portable type.

## Pattern evaluation — Four-questions per pattern in substrate context

### Pattern A — outer struct + kind enum
```rust
struct MyError { span: Span, kind: MyErrorKind }
enum MyErrorKind { /* variants without span fields */ }
```
- **Obvious YES** — struct-level field declares the discipline; no per-variant scanning
- **Simple YES** (primary span single-path `err.span`); multi-span variants put secondary spans in kind
- **Honest YES** (bare `Span` not `Option<Span>`) — author MUST decide explicitly when no span exists; `Span::unknown()` requires intent, not silent omission
- **UX YES** — new variant author adds to `MyErrorKind` and gets span discipline for free; matches `std::io::Error` precedent

### Pattern B — newtype wrapper
```rust
struct Spanned<E>(Option<Span>, E);
type MyError = Spanned<MyErrorKind>;
```
- Obvious PARTIAL · Simple YES · Honest YES · **UX NO** — generic type signatures leak substrate-wide; `Result<T, Spanned<RuntimeErrorKind>>` proliferates into every function signature, From impl, and trait bound. The substrate is large; B's blast radius is incompatible.

### Pattern C — derive macro with field check
```rust
#[derive(Conformare)]
enum MyError { Variant { span: Span, .. }, #[spanless] Other { .. } }
```
- Obvious PARTIAL · **Simple NO** · Honest YES · UX PARTIAL — macro helps authorship but consumer N-path access unchanged. TWO authorship paths within a type (with span / with `#[spanless]`) violates "exactly one way to do a thing."

## Verdict matrix

| Pattern | Obvious | Simple | Honest | UX | Atomic |
|---|---|---|---|---|---|
| **A** | YES | YES | YES | YES | **4/4** |
| B | PARTIAL | YES | YES | NO | 2/4 |
| C | PARTIAL | NO | YES | PARTIAL | 1/4 |

**Pattern A wins decisively.** Single pattern that passes all four atomic YES.

## Refinements the spell surfaced

These were NOT in the orchestrator's prior recommendation; the spell added them:

1. **`span: Span` not `Option<Span>`** — the spell argued that `Option<Span>` is "convention-safe but not structurally absolute." Bare `Span` FORCES authors to write `Span::unknown()` explicitly when no source exists — making the decision visible rather than silently omitted. A rune documents legitimately-spanless cases.

2. **Multi-span variants** — primary span lives in outer struct (`err.span` always works); secondary spans (call_span, outer_define_span, prior_loc, current_loc) stay in kind variants. The N-path access becomes "do you need the SECONDARY span" matching, which is semantically honest — not the "do you have a span at all" failure.

3. **Field-name discipline normalization** — Pattern A makes THE span at the outer level; the substrate's `call_span` / `prior_loc` / `body_span` non-uniformity dissolves because the primary field is always `.span`.

4. **ArgSpecError as substrate precedent** — its `classify()` method IS the closest existing approximation of `err.span` single-path access. Conformare elevates argspec's hand-discipline (AUDIT.md line 161 convention) to type-system enforcement substrate-wide.

5. **Tier-aware exemption framework** — Tier 1 types MUST conform; Tier 2 wrappers inherit conformance from wrapped types; Tier 3 infrastructure types are domain-spanless by design (file paths, digests, byte positions) — appropriate exemptions documented per type.

## Findings

**L1** — `TypeError::CyclicSubtype` (`src/types.rs:1542`). Variant lacks `span: Span`; constructible without source location; consumer at `parse.rs:172` injects `Span::unknown()` explicitly. Cascade: every `TypeError` consumer must match exhaustively across 16 variants. Immediate action: document with `rune:conformare(spanless-by-domain)` OR thread span into `register_subtype` (attested-arc); full structural fix in retrofit stone.

**L2** — `ParseStep::ArityMismatch` (`src/function/parse.rs:45`). Stone 241.18a's documented NEW-2 residual. `parse_fn_signature_prefix` lacks `head_span` parameter; caller has span context but the parser boundary drops it. Closes structurally when Pattern A retrofits ParseStep + parser-API sister-walk adds head_span parameter.

## Retrofit ordering recommendation

The spell's proposed order:

1. **TypeError::CyclicSubtype** (smallest blast radius; one emitter, one consumer; cleans the base of the type hierarchy — TypeError is what ArgSpecError converts INTO)
2. **ParseStep::ArityMismatch** (Stone 241.18a leftover; small scope; pub(in crate::function))
3. **RuntimeError spanless variants** (rune-document UserMainMissing, EvalVerificationFailed, TryPropagate/OptionPropagate per domain rationale)
4. **ExtractionError::NonPortableCapture** (genuine diagnostic gap; user-facing)
5. **Full Pattern A retrofit cascade** for TypeError, RuntimeError, CheckError, ParseStep
6. **Parser-API sister-walk** — `head_span: &Span` parameter threading on every API taking a bare slice

## Out-of-scope observation (noted for future audit)

The `span_prefix(span: &Span)` helper is copy-pasted 5× across `check.rs`, `types.rs`, `runtime.rs`, `macros.rs`, `form_match.rs`. Outside conformare's scope; flagged as a scry/forge concern for a future audit (probably during arc 243's retrofit work — the helper consolidation lands naturally when error types restructure).

## Conclusion: spell EARNED its grimoire seat

The spell arrived at Pattern A through honest four-questions from first principles. The substrate's existing precedent (ArgSpecError's `classify()` discipline) validates the direction. The refinements the spell surfaced (`span: Span` vs `Option<Span>`; multi-span variants handling; field-name normalization; tier-aware exemption framework) sharpen the orchestrator's prior recommendation.

Conformare joins the grimoire's defensive set. Next casting: after each retrofit stone in arc 243, conformare re-casts to verify the type now CONFORMS structurally.
