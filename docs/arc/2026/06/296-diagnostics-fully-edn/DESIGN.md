# Arc 296 — Error → EDN, unified under ONE trait: every diagnostic is structured EDN by construction

> **Status: RE-SCOPED (2026-06-30, builder) — from "fix the macro prose-blob" to "unify error→EDN under one
> trait."** Arc-sized. **Slice 1 (the macro chain) LANDED (`f397aba6`, weighed 4141/0/91).** Parked behind the
> 293/294 line; this is the real shape. **Intueri the trait name at strike time.**

## What opened it, and what re-opened it
A macro error printed its whole cause-chain as ONE prose string inside an otherwise-EDN envelope (builder, while
debugging 293.K5: *the tagged wrappers are good; make this fully EDN*). Slice 1 fixed the macro chain. But the builder
then asked the load-bearing question: ***"is that definitively just macros being odd, or a deeper asymmetry we should
unify?"*** Grounded against the disk — **a deeper asymmetry.** This arc is the unification.

## The asymmetry (grounded 2026-06-30)
Error→EDN today is a pile of **ad-hoc free functions**, added per-type only when a consumer surfaces — **no contract,
no guarantee**:
- **Have a serializer:** `runtime_error_to_edn` (RuntimeError, arc 233) · `payload_to_edn` · `span_to_edn` ·
  `macro_error_to_edn` / `startup_error_to_edn` (296.1, `f397aba6` — Macro variant only).
- **Still stringify:** non-Macro `StartupError` → `format!("{}", e)` (`process/verbs.rs:86`) · `MainSignature` →
  `.to_string()` (`verbs.rs:408/616`) · the `ProcessDiedError` family payloads (String fields, `types.rs`) ·
  CheckError/Parse/Config/Load/Type → the **interim `Diagnostic` type** (`freeze.rs:605-617` → `diagnostic.rs:31`,
  flat `String`/`Int` fields — a half-built stepping stone, neither prose nor real EDN).

Three tells it is structural, not local:
1. **No trait** — serialization is free functions you must remember to write and wire; a new error type ships with no
   EDN form and nothing stops it.
2. **Whole error families still stringify** (StartupError-non-Macro, MainSignature, the ProcessDiedError family, CheckError).
3. **A half-finished `Diagnostic` type** sits between prose and EDN.

Slice 1 (`f397aba6`) added *another* ad-hoc serializer and **preserved the asymmetry** (Macro chain structured;
siblings still string, for `extract-panics` compat) — a seam, which the long-term-stability bias names a deferral.

## The thesis — ONE contract
**Every error/diagnostic type implements one `ToEdn` trait** (name TBD-intueri):
```rust
pub trait ToEdn {
    fn to_edn(&self) -> OwnedValue;   // structured EDN; recurses into causes/spans/snapshots
}
```
Structured EDN becomes guaranteed **by construction**: the **serialization boundary is generic over `ToEdn`**, so a
stringly error has no path to the wire — a `format!("{}", e)` into a payload field is uncompilable (extirpare's top
rung). The existing free functions (`runtime_error_to_edn`, `macro_error_to_edn`, …) become the trait impls — one
canonical path, not a scatter of functions. The interim `Diagnostic` type is **retired**, replaced by the trait.

## Decomposition
- ✅ **296.1 — the macro chain (`f397aba6`, weighed 4141/0/91).** `MacroErrorKind` split into structured
  `cause: Box<…>` variants; `macro_error_to_edn` + `startup_error_to_edn` (mirroring `runtime_error_edn.rs`); the
  Macro `StartupError` variant emits the nested `#wat.kernel/…` chain. **The first slice — and it ships the seam the
  rest of this arc removes.**
- **296.2 — mint the `ToEdn` trait + converge the existing serializers.** Define the trait; make `RuntimeError`,
  `MacroError`, `StartupError`, `Span`, `AssertionPayload`, `ValueSnapshot`, … implement it; the ad-hoc
  `*_to_edn` free functions become the impl bodies (one canonical path; no behavior change → SET-diff ∅ probe).
- **296.3 — bring the stringly holdouts under the trait.** non-Macro `StartupError`, `MainSignature`, the
  `ProcessDiedError` family: change the payload fields from `String` → a tagged-EDN value; rewrite the `verbs.rs`
  sites + `process_died_error_*` (`runtime.rs:22095+`). **Touches `types.rs`'s ProcessDiedError variant field types
  AND `extract-panics`'s round-trip** (the compat boundary slice 1 stopped at).
- **296.4 — retire the interim `Diagnostic`.** `diagnostic.rs` + `StartupError::diagnostics()` (`freeze.rs:600`):
  CheckError/Parse/Config/Load/Type → the trait; the `--check-output edn|json` modes consume `to_edn()` directly.
- **296.5 — the wall + close.** Make the serialization boundary GENERIC over `ToEdn` (a non-`ToEdn` error can't reach
  the wire — the structural guarantee); a probe that a new error variant without an impl fails to compile; INSCRIPTION.

## Blast radius (arc-sized)
`src/diagnostic.rs` (retire) · `src/freeze.rs` (`diagnostics()`) · `src/process/verbs.rs` (the sites + MainSignature) ·
`src/runtime.rs` (`process_died_error_*` + the ProcessDiedError payload construction) · `src/types.rs` (ProcessDiedError
variant field types `String`→EDN) · `extract-panics` (round-trip) · `src/runtime_error_edn.rs` + `src/macros/error_edn.rs`
(→ trait impls) · CheckError EDN · the new trait's home (a small `src/diagnostic_edn.rs` or `src/edn/` module).

## Out of scope (affirmative cuts)
- **The human-readable `Display` path stays** — text rendering for the in-process harness (`harness.rs`). EDN is the
  wire/IPC + `--check-output` face; `Display` is the human face. Both, not one replacing the other.

## Pairs / prior art
- **Arc 233** (shipped — `runtime_error_to_edn`, the leaf) — this arc GENERALIZES it from one free function to a trait
  every error implements.
- **Arc 243 (conformare)** — Pattern A error shape (outer struct + kind enum); the trait rides on top, unchanged shape.
- **The interim `Diagnostic`** (`diagnostic.rs`) — a stepping stone toward structured errors, **retired by this arc.**
- **Arc 280 (stdio-edn-bound)** — sibling "EDN all the way down" facet (WAT-level IO); independent.
