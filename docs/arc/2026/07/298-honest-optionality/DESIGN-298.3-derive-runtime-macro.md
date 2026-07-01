# Strike 298.3 — derive RuntimeError + MacroError: the last hand-written error serializers fall, 296 closes

> **Status: STRIKE-READY (2026-07-01). The way out.** The 296 derive sweep resumes now that the data is honest
> (298.1 tagged Option/Result; 298.2 annihilated the `Span::unknown()` sentinel). RuntimeError (~28 variants) + MacroError
> (~11) are the **last smuggle-capable families**. When their hand serializers are deleted and the derive covers them,
> **296's R1 *NE SIBI OBSOLESCAT* turns to PROBATUM EST** — every top-level error a structural derive, no hand-written
> body left to smuggle prose into.

## The proven pattern (already applied to 5 families: Config/Check/Type/Stdlib/Load)
`#[derive(crate::to_edn::ToEdn)]` on the KIND enum → `impl ToEdn for <Kind>` (a match over variants, snake→kebab keys,
each field `.to_edn()`, tag = variant name, ns `wat.kernel`); the outer Pattern-A struct's `impl ToEdn` becomes the
`splice_span(self.kind.to_edn(), &self.span)` wrapper; the hand serializer is DELETED. The `#[to_edn(via/literal/key/skip)]`
DSL handles the irregular fields. Mirror `src/config.rs` (Strike 1) / `src/types/error.rs` (3a) / `src/load.rs` (3b).

## Three small support additions (grounded 2026-07-01)
1. **`error_edn_of_boxed<T: crate::to_edn::WatError>(cause: &Box<T>) -> OwnedValue { cause.error_edn() }`** in
   `src/to_edn.rs` — the `via` helper for `Box<>` causes. (`error_edn_of` takes `&impl WatError`; a `&Box<MacroError>`
   does NOT coerce to it, but `cause.error_edn()` auto-derefs through the Box.) Used by the three Box-cause variants.
2. **`impl crate::to_edn::ToEdn for ClauseAttempt`** (in `src/runtime_error_edn.rs`, wrapping the existing
   `clause_attempt_to_edn` free fn) — so `NoMatchingClause`'s `Vec<ClauseAttempt>` field serializes via `.to_edn()`.
   (`ValueSnapshot` + `Provenance` already have `impl ToEdn` — no change.)
3. **Secondary-span fields** (`SandboxScopeLeak`, `PostconditionFailed`, and any variant with a second `Span` field) →
   `#[to_edn(key = "call-span")]` / `#[to_edn(key = "outer-define-span")]` / `#[to_edn(key = "body-span")]` /
   `#[to_edn(key = "ensure-span")]` per the CURRENT serializer's keys (read `runtime_error_edn.rs` for the exact key each
   secondary span uses). The PRIMARY span stays on the outer struct via the wrapper.

## The families + their irregular fields
- **RuntimeErrorKind** (`src/value/signal.rs`, ~28 variants; serializer `src/runtime_error_edn.rs`):
  - `MacroExpansionFailed { op, cause: Box<MacroError> }` → `cause` `#[to_edn(via = crate::to_edn::error_edn_of_boxed)]`.
  - `AssertionFailed { …, actual: Option<String>, expected: Option<String> }` → plain `.to_edn()` (Option is TAGGED now,
    298.1 — `#wat.core.Option/None nil` / `#wat.core.Option/Some "…"`; this is a WIRE CHANGE from the old transparent form,
    so the byte-identical target is the NEW tagged form — capture it).
  - `NoMatchingClause { …, attempted: Vec<ClauseAttempt> }` → plain `.to_edn()` (via the new ClauseAttempt impl).
  - `TypeMismatch` / `NotCallable` embed `ValueSnapshot` → plain `.to_edn()` (already ToEdn).
  - secondary-span variants → `key` per above.
- **MacroErrorKind** (`src/macros/error.rs`, ~11 variants; serializer `src/macros/error_edn.rs`):
  - `ProgramBodyEvalFailed { macro_name, cause: Box<MacroError> }` → `cause` `#[to_edn(via = error_edn_of_boxed)]`.
  - `MacroEvalRuntimeFailed { cause: Box<RuntimeError> }` → `cause` `#[to_edn(via = error_edn_of_boxed)]`.
  - `StartupError` delegation (`startup_error_to_edn`) is NOT part of this — it stays hand-written (298-carve: transparent
    passthrough, no smuggle hazard). Only `MacroError`'s own serializer is derived.

## Proof — BYTE-IDENTICAL, captured not guessed (the 298.2 lesson)
- A co-located probe per family (`tests/diagnostics/probe_arc298_3_runtime_derive_identical.rs` +
  `..._macro_derive_identical.rs`): for a representative value of EVERY variant (with an explicit FIXED span
  `Span::new(std::sync::Arc::new("test.wat".to_string()), 1, 0)` — deterministic, NOT `rust_caller_span!()`), assert
  `wat_edn::write(&e.to_edn())` == the exact string. **CAPTURE each golden**: construct → temporary
  `eprintln!("{}", write(&e.to_edn()))` → run → verify the emitted bytes are structurally what the derive should emit →
  paste that EXACT string as the `assert_eq!` expected → remove the eprintln. NEVER hand-guess a literal; NEVER weaken to
  `assert!(contains)`.
- SET-diff ∅ against the pre-derive hand serializer where the shape is unchanged; where 298.1's Option-tagging changed the
  `AssertionFailed` actual/expected wire form, that is the intended new byte-identical target (capture it).
- DELETE `runtime_error_to_edn` + the `MacroError` hand serializer. FULL gate `cargo nextest run --release` = 0 failed;
  `cargo build --release` clean.

## Blast radius
`src/to_edn.rs` (the `error_edn_of_boxed` helper) · `src/value/signal.rs` (derive + annotations on RuntimeErrorKind) ·
`src/runtime_error_edn.rs` (splice_span wrapper; DELETE `runtime_error_to_edn`; the ClauseAttempt impl) ·
`src/macros/error.rs` (derive + annotations on MacroErrorKind) · `src/macros/error_edn.rs` (wrapper; DELETE the MacroError
serializer; keep `startup_error_to_edn`) · two new probes · the test cascade (any test asserting the old RuntimeError/
MacroError wire — update to the captured tagged/derived form, byte-identical, NEVER weakened). STOP + report if a variant
cannot be made byte-identical without a weakening — surface it, do NOT soften the probe.

## The anti-weakening rule (PROBATIO FLEXA MENTITVR — the 298.2 wound is fresh)
This is the last derive; RuntimeError is big; the cascade will be wide. NEVER weaken a probe. A byte-identical probe stays
`assert_eq!` on captured exact bytes, or the test is deleted as a proven duplicate — never `assert!(contains)`, never an
inverted assertion. The orchestrator weighs the emitted diff char-by-char, not the report.

## On landing
Zero hand-written top-level error `to_edn` match bodies remain (ParseError orphan + Resolve/Startup passthrough +
building-block leaves are the affirmatively-hand-written non-hazards). **296's R1 *NE SIBI OBSOLESCAT* → PROBATUM EST.**
Then the 296 tail (S7, consonare R3/R4/R5/R9) + the 296 INSCRIPTION + the 298 INSCRIPTION close both arcs.
