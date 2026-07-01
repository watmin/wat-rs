# 296 N3 — Per-phase tag namespaces: an error chain reads its own phases; the namespaces are single-source refactorable

> **Status: STRIKE-READY draft (2026-07-01). Names ratified by the builder; mechanism = single-source (refactorable
> toward 1.0.0).** Orchestrator designs + delegates + weighs by its own gate AND the emitted wire EDN. A WIRE CHANGE for
> every error tag — a wide, by-force cascade (298-scale). N3 was crowned (modeled = intended); this specs + solves it.

## The thesis
Every top-level error FAMILY tags under its **phase** namespace, not the uniform `#wat.kernel/`. A nested error chain
becomes **self-describing by phase** — you read the stack in the tags:
```
#wat.macro/ProgramBodyEvalFailed
  {:cause #wat.runtime/TypeMismatch
            {:snapshot #wat.kernel/ValueSnapshot {…}      ; ← shared infra stays kernel
             :location #wat.kernel/Location {…}}}
```
`wat.kernel` STOPS being an error catch-all and MEANS exactly "a shared value type" (a Location, a Snapshot, a Remedy).

## The names (builder-ratified 2026-07-01) — the tag ns mirrors the `:wat::<phase>::` module
| family (Rust) | tag namespace |
|---|---|
| `ConfigError` | `wat.config` |
| `CheckError` / `CheckErrors` | `wat.check` |
| `TypeError` | `wat.type` |
| `StdlibError` | `wat.stdlib` |
| `LoadError` | `wat.load` |
| `RuntimeError` | `wat.runtime` |
| `MacroError` | `wat.macro` |
| `ParseError` | `wat.parse` |
| `ResolveError` | `wat.resolve` |
| **shared infra** — `Span` `Location` `Frame` `ValueSnapshot` `Provenance` `ClauseAttempt` `Remedy` `LoadFetchError` `HashError` `FlatMessage` `AssertionPayload` | `wat.kernel` |

## The mechanism — single source of truth (builder: *"written such that a refactor can handle them if we change our minds as we near 1.0.0"*)
No namespace string is ever written as a literal at an emission site. ALL of them live in ONE module:
```rust
// src/error_ns.rs — THE single source of truth for error tag namespaces.
// Rename a namespace HERE and every production emission site follows (one edit).
// (Golden/CLI test literals carry the string by nature — a fix-wat/codemod sweep is the refactor for those.)
pub const CONFIG:  &str = "wat.config";
pub const CHECK:   &str = "wat.check";
pub const TYPE:    &str = "wat.type";
pub const STDLIB:  &str = "wat.stdlib";
pub const LOAD:    &str = "wat.load";
pub const RUNTIME: &str = "wat.runtime";
pub const MACRO:   &str = "wat.macro";
pub const PARSE:   &str = "wat.parse";
pub const RESOLVE: &str = "wat.resolve";
pub const KERNEL:  &str = "wat.kernel";   // shared value types (the old catch-all, now precise)
```
- **The derive** gains an enum-level sub-key `#[to_edn(namespace = <path>)]` (grammar-constrained to a `syn::Path`, like
  `via`'s bare-ident constraint — NO inline string; the smuggle-hole stays closed). The macro emits `Tag::ns(#path,
  variant)` — a **reference to the const**, never a baked literal. Absent → defaults to `crate::error_ns::KERNEL`.
- **The 4 hand wrappers** (`to_edn.rs edn_tag`, `check/error_edn.rs`, `runtime_error_edn.rs`, `macros/error_edn.rs`) +
  the Parse/Resolve orphan impls reference the consts (`Tag::ns(crate::error_ns::CHECK, …)`), never `"wat.kernel"`.
- **Result:** `grep -rn '"wat\.' src/ crates/wat-macros/ --include=*.rs` (production, not tests) → the ONLY hits are the
  10 const definitions in `error_ns.rs`. That is the refactor guarantee, mechanically checkable.

## The 7 derived families → their `namespace` attr; the 2 hand families → their wrapper ns
- `#[to_edn(namespace = crate::error_ns::CONFIG)]` on `ConfigErrorKind`; `CHECK` on `CheckErrorKind`; `TYPE` on
  `TypeErrorKind`; `STDLIB` on `StdlibErrorKind`; `LOAD` on `LoadErrorKind`; `RUNTIME` on `RuntimeErrorKind`; `MACRO` on
  `MacroErrorKind`.
- `ParseError` (orphan impl, `src/parser.rs`) → `PARSE`; `ResolveError` (`resolve/error.rs`) → `RESOLVE`.
- Nested causes already recurse through each type's own `to_edn`, so a `MacroError` whose cause is a `RuntimeError`
  automatically emits `#wat.macro/… {:cause #wat.runtime/…}` — the phase walk falls out for free.
- Embedded shared blocks keep `KERNEL` (they're not phase errors): their `edn_tag`/impls reference `error_ns::KERNEL`.

## Out of scope (affirmative — coupled elsewhere, not cut)
- **`Failure`, `ProcessDiedError`, `ThreadDiedError`** — registered wat types whose tag derives from their registered
  CLASS PATH (`:wat::kernel::…`), not the derive. Re-namespacing them = renaming a registered type, which the
  **de-stringify strike already reopens** (it retypes their `:String` fields). Their ns rides THAT strike so the
  registration seam is touched once, not split. Named here; tracked to de-stringify. NOT a cut.
- **`StartupError`** — transparent passthrough (delegates to the inner error's `to_edn`); the inner phase ns shows
  through unchanged. Nothing to do.

## Proof
- **RED probe** (behavioral, co-located): drive a check error, a type error, a runtime error through startup; assert each
  tag's namespace is its phase (`wat.check` / `wat.type` / `wat.runtime`), NOT `wat.kernel`; and assert an embedded
  `Location`/`ValueSnapshot` STAYS `wat.kernel`. RED at HEAD (all `wat.kernel`), GREEN after.
- **The refactor guarantee:** the production grep above → only the 10 consts.
- FULL gate `cargo nextest run --release` = 0 failed. The test cascade is WIDE — every golden/CLI/probe asserting
  `#wat.kernel/<ErrorVariant>` updates to its phase ns (intended wire change, by force). A fix-wat/codemod handles the
  bulk; NEVER weaken a byte-identical probe to pass (PROBATIO FLEXA MENTITVR).

## Blast radius (wide by nature)
`src/error_ns.rs` (NEW) · `crates/wat-macros/src/to_edn_derive.rs` (the `namespace` sub-key + emit the path) · its
ui/unit tests · the 7 derived KIND enums (annotate) · `src/to_edn.rs` + `src/check/error_edn.rs` +
`src/runtime_error_edn.rs` + `src/macros/error_edn.rs` (wrappers → consts) · `src/parser.rs` + `src/resolve/error.rs`
(orphan impls → consts) · the test cascade (every `#wat.kernel/<ErrorVariant>` assertion). STOP + report if a family's
ns can't come from a single const, or if the cascade reaches non-error code.
