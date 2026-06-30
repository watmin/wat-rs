# Arc 296 — Diagnostics are fully EDN: structured cause-chains, never a prose blob

> **Status: STUB — SCOPED (2026-06-30), NOT BUILT. Parked behind the 293/294 line.** Opened from a builder
> observation while debugging a macro error (the 293.K5 build): the tagged wrappers render as clean EDN, but the
> innermost diagnostic payload is one prose string. Survey grounded against the disk this session (file:line
> spot-verified by the orchestrator). Working dir name; **intueri the real arc name at strike time.**

## The observation that opened it (builder, 2026-06-30)
A macro error printed:
```
#wat.kernel/ProcessPanics [#wat.kernel.ProcessDiedError/StartupError
  ["macro: <file>:18:1: malformed template: macro :debug::show-extend-surface — program body eval failed:
    <file>:16:98: malformed template: macro_eval: runtime::eval failed: <file>:16:98: unbound symbol: str"]]
```
Builder: *the tagged wrappers are good-looking; make this **fully EDN**.* The outer `#wat.kernel/…` tags ARE EDN
(good). The **innermost element is a single prose string** — a colon-chained cause-chain with `file:line` and
`"X failed: Y failed: unbound symbol str"` baked in. A diagnostic should be **navigable structured data all the way
down**, never a string blob.

## The thesis
**Every diagnostic is a fully-EDN value — a nested, tagged cause-chain carrying `:span` / `:kind` / `:cause` as
structured data, recursively, to the leaf.** This continues **arc 233's "errors-as-EDN wire protocol"** (which
shipped `runtime_error_to_edn` for all `RuntimeError` variants but stopped at the leaf) one level up, through
`MacroError` and `StartupError`. The prose `Display` rendering stays — for humans, in-process — but the **wire /
IPC diagnostic is EDN**.

## The chain (mapped outer→leaf, grounded this session)
| layer | type | where | EDN today? |
|---|---|---|---|
| envelope | `#wat.kernel/ProcessPanics [...]` | `src/process/verbs.rs:49` (`emit_chain_envelope`) | ✅ EDN |
| outer enum | `Value::Enum ":wat::kernel::ProcessDiedError"/"StartupError"` fields `[String]` | built `runtime.rs:22095–22101`; registered `types.rs:987–990` | ⚠ tag is EDN, **the field is the prose String** |
| startup union | `StartupError::Macro(MacroError)` | `freeze.rs:516–538` (+ `Display` `540–555`) | ❌ Display only |
| macro expand err | `MacroError { span, kind: MalformedTemplate { reason: String } }` | `macros/error.rs:8–70` (`MalformedTemplate` at **`:50`** — verified) | ❌ Display only |
| inner macro-eval err | `MacroError::MalformedTemplate` wrapping a `RuntimeError` | built `macros/eval.rs:117–123` | ❌ collapsed to String |
| leaf runtime err | `RuntimeError { span, UnboundSymbol("str") }` | `value/signal.rs:586` | ✅ **`runtime_error_to_edn` exists** (`runtime_error_edn.rs:40` — verified) but is bypassed on this path |

`Span` already has an EDN form (`span_to_edn`, `panic_hook.rs:209` → `{:file … :line … :col …}`).

## The breach — the stringification sites (verified file:line)
- **PRIMARY (the IPC boundary):** `src/process/verbs.rs:349`, `:436`, `:561` — all three do
  `process_died_error_startup_value(format!("{}", e))` on a `StartupError`, stuffing the whole Display-chain into a
  `Value::String`. (Verified — three identical sites.)
- **THE STRUCTURAL COLLAPSE (inside freeze):** `macros/expand.rs:515` — `format!("macro {} — program body eval
  failed: {}", name, e)` where `e: MacroError`; and `macros/eval.rs:117` — `format!("macro_eval: runtime::eval
  failed: {}", e)` where `e: RuntimeError` (which ALREADY has an EDN form) — both concatenate a structured cause
  into the `reason: String` of `MalformedTemplate`.
- **SECONDARY (spans into prose):** `macros/error.rs:138` (`span_prefix` into the Display); `freeze.rs:543–552`.

## The cure — the high-leverage seam (extirpare's top rung)
**Split `MacroErrorKind::MalformedTemplate { reason: String }` into structured cause-carrying variants** so the
collapse is *uncompilable*:
```rust
ProgramBodyEvalFailed { macro_name: String, cause: Box<MacroError> },   // replaces the expand.rs:515 collapse
MacroEvalRuntimeFailed { cause: Box<RuntimeError> },                     // replaces the eval.rs:117 collapse
```
This makes the `format!("… failed: {}", e)` calls fail to compile — they must carry `cause: Box::new(e)`. **The whole
class of macro-cause-chain stringification becomes unrepresentable** — not patched per-site, structurally forbidden.
(Keep `MalformedTemplate { reason }` only for genuine static-string reasons, or retire it.)

Then the serializers, mirroring the shipped `runtime_error_to_edn`:
- `macro_error_to_edn(&MacroError) -> OwnedValue` (new `src/macros/error_edn.rs`, ~50 lines) — delegates to
  `runtime_error_to_edn` at the `MacroEvalRuntimeFailed` leaf.
- `startup_error_to_edn(&StartupError) -> OwnedValue` — dispatches per variant (`Macro` → macro_error_to_edn,
  `Runtime` → runtime_error_to_edn, …).
- The three `verbs.rs` sites pass `&e` (structured) instead of `format!("{}", e)`; `process_died_error_startup`
  (`runtime.rs:22104`) takes the structured EDN, and the `ProcessDiedError::StartupError` payload carries a tagged
  EDN value (the `types.rs:1153–1154` comment — *"extensible to kind / location if a real consumer surfaces"* — **this
  arc is that consumer**).

## Target shape
```edn
#wat.kernel.ProcessDiedError/StartupError
  #wat.kernel/MacroError
    {:phase :macro  :span {:file "…" :line 18 :col 1}  :kind :malformed-template
     :macro "debug::show-extend-surface"
     :cause #wat.kernel/MacroError
       {:span {:file "…" :line 16 :col 98}  :kind :malformed-template  :stage :macro-eval-runtime
        :cause #wat.kernel/UnboundSymbol {:name "str"  :span {:file "…" :line 16 :col 98}}}}
```
(The leaf `#wat.kernel/UnboundSymbol` is exactly what `runtime_error_to_edn` already emits.)

## Decomposition (provisional — when this arc opens)
- **296.1 — the type surgery:** split `MalformedTemplate` → structured cause variants; let the compiler waterfall
  the `expand.rs`/`eval.rs` construction sites onto `cause: Box<…>`. The collapse becomes unrepresentable.
- **296.2 — the serializers:** `macro_error_to_edn` + `startup_error_to_edn` (mirror `runtime_error_edn.rs`).
- **296.3 — the IPC boundary:** `verbs.rs:349/436/561` + `process_died_error_startup` carry the structured EDN; the
  `ProcessDiedError::StartupError` payload becomes a tagged EDN value, not a String.
- **296.4 — supersede the interim `Diagnostic`:** `freeze.rs:591–634` `StartupError::diagnostics()` + `diagnostic.rs`
  (flat-field early form, `String`/`Int` only) was the stepping stone; full EDN is its stated goal (`diagnostic.rs:10–16`).

## Blast radius (bounded)
`src/macros/error.rs` (type), `src/macros/expand.rs` (~8 construction sites), `src/macros/eval.rs:107–123`,
new `src/macros/error_edn.rs`, `src/freeze.rs` (startup_error_to_edn; Display unchanged), `src/runtime.rs:22095–22106`,
`src/process/verbs.rs:349/436/561`. **NOT touched:** `runtime_error_edn.rs`, `panic_hook.rs`, the `Display` impls
(stay for human text), `edn_shim.rs::value_to_edn_with` (its `Value::Enum` arm is correct — the problem is what goes
INTO the fields vec).

## Out of scope (affirmative cuts)
- **The human-readable `Display` path stays** — the in-process harness (`harness.rs:69–78`,
  `HarnessError::Startup`) renders text and needs no EDN. EDN is for the wire/IPC diagnostic, not a replacement for
  prose rendering.
- **Re-architecting `RuntimeError`'s EDN** — done (arc 233). This arc only extends UP from the leaf.

## Pairs / prior art
- **Arc 233** (shipped — `runtime_error_to_edn`, "errors-as-EDN wire protocol", stopped at the RuntimeError leaf) —
  this arc continues the same axis up through Macro/Startup.
- **Arc 243 (conformare)** — Pattern A (outer struct + kind enum); `MacroError` already follows it; this adds
  structured cause fields to the kind variants, doesn't change the shape.
- **Arc 280 (stdio-edn-bound, STUB)** — orthogonal neighbor (WAT-level `println`/`readln` EDN-bound); both are facets
  of "EDN all the way down." No dependency either way.
- **Arc 170 slice 1i** — introduced `StartupError`/`MainSignature` with the `types.rs:1153–1154` "extensible if a
  real consumer surfaces" comment — **this arc is that consumer.**
