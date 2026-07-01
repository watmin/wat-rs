# 296 — Structuring sweep: typed causes, not stringified (audit findings S1/S2/S6/S7)

> **Status: STRIKE-READY (2026-07-01).** The "structure the data first" work (Option A) before the derive sweep. Four
> localized findings from the structure-as-prose audit (the `collect_hints` case's siblings), each grounded. One theme:
> **a typed error nested as a typed CAUSE / structured fields — never `format!`'d away.**

## S1 — `RuntimeErrorKind::MacroExpansionFailed.reason` (L1)
`src/runtime.rs:11282`: `.map_err(|e| … MacroExpansionFailed { reason: format!("{}", e) })` flattens a whole typed
`MacroError` (kind/span/nested cause) to prose. `MacroError` already impls `ToEdn`. **Cure:** field `reason: String` →
`cause: Box<MacroError>` (`src/value/signal.rs`); construction stores `Box::new(e)`; the serializer arm
(`src/runtime_error_edn.rs`) emits the nested cause structurally — **mirror the EXISTING `MacroEvalRuntimeFailed`
(`Box<RuntimeError>`) / `ProgramBodyEvalFailed` (`Box<MacroError>`) arms** (this is the same shape, already solved). Display
renders the nested error.

## S2 — the `ProcessDiedError::RuntimeError` bypass (L1, a live bug — NOT a field change)
`src/runtime.rs:20876` + `:20963` (`Ok(SpawnOutcome::RuntimeErr(e)) => … process_died_error_runtime(e.to_string())`) call
the LOW-LEVEL string builder with **prose**, while `process_died_error_runtime_value(e: &impl WatError)` (`:22096`) is the
structured path (it calls `to_wire_edn(e)` → the low-level builder at `:22097`). **Cure:** route both bypass sites through
`process_died_error_runtime_value(&e)` (structured EDN), **IF `e` (the `SpawnOutcome::RuntimeErr` payload) implements
`WatError`** — verify its type first; if it does not, STOP + report (do not guess a wrap). The message field stays
`String`-carrying-EDN (turning it into a typed field is S3/S4 — a SEPARATE breaking change, out of scope here). Optional
hole-close: if a clean module boundary exists, make `process_died_error_runtime(String)` private so only `_value` reaches
it (the bypass becomes unrepresentable) — but only if trivial; else note it for later.

## S6 — `StdlibErrorKind::ParseFailed.source` (L2)
`src/stdlib.rs:354`: `parse_all_with_file(…).map_err(|e| … ParseFailed { source: e.to_string() })` flattens a typed
`ParseError` (which now has `ToEdn`, span + variant). **Cure:** field `source: String` → `cause: ParseError`
(`src/stdlib.rs`); store the typed `e`; the serializer (`stdlib.rs` `ParseFailed` arm) emits `cause.to_edn()`. Mirror
`LoadErrorKind::Parse`'s nested-`ParseError` serialization (already structured).

## S7 — `CheckErrorKind::EnsureFnInvalid.reason` (L2)
`src/check.rs:8731`: `reason: format!(":ensure :fn takes `{}` but clause returns `{}`", format_type(arg_ty),
format_type(&clause_ret))` buries a `{arg-type, clause-return-type}` type pair in prose. **Cure:** split the field —
`reason: String` → `arg_type: String, clause_return_type: String` (`src/check/error.rs`); construction stores the two
`format_type(...)` strings separately (no `format!`); the serializer (`src/check/error_edn.rs` `EnsureFnInvalid` arm)
emits `:arg-type` + `:clause-return-type`; Display renders the sentence from the two fields. **Grep `EnsureFnInvalid`
first** — there may be OTHER construction sites with genuinely free-form reasons; if a site's reason is NOT this type-pair,
STOP + report (only THIS one site is the structured smuggle per the audit).

## Out of scope (affirmative cuts)
- **S3/S4** (ProcessDiedError EDN-in-String → typed field) — a breaking change to a registered wat type; its own
  four-questions decision, later.
- **S5** (ThreadDiedError::Panic assertion-envelope) — L2, later.
- **The `#[derive(ToEdn)]` sweep** — follows, over this now-honest data.

## Proof
- Each finding: a probe asserting the structured shape (`:cause` is a nested `#wat.kernel/…` tagged value not a String;
  `:arg-type`/`:clause-return-type` present, no `:reason`; the two RuntimeErr sites emit a structured payload).
- S2: a probe that a thread `RuntimeErr` outcome's `ProcessDiedError` payload is the structured `to_wire_edn` form, not
  `e.to_string()` prose.
- FULL gate `cargo nextest run --release` = 0 failed; `cargo build --release` clean; `extract_panics` round-trip stays green.

## Blast radius
`src/value/signal.rs` (S1 field) · `src/runtime.rs` (S1 construction + S2 routing) · `src/runtime_error_edn.rs` (S1
serializer) · `src/stdlib.rs` (S6) · `src/check.rs` (S7 construction) · `src/check/error.rs` (S7 field + Display) ·
`src/check/error_edn.rs` (S7 serializer) · probes. NOTHING else. STOP on any variant whose shape doesn't fit its cure.
