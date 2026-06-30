# BRIEF — 296: error → EDN, unified under ONE `ToEdn` trait (the full arc)

> **Executor: one sonnet, MAIN tree (NOT a worktree — the `../holon-rs` path dep breaks worktree builds).**
> Orchestrator drew this + the DESIGN; weighs the kill forced-clean. **Commit nothing** — leave the tree for the
> orchestrator. Build the slices IN ORDER; run the FULL gate after each slice; STOP + report on the named boundaries.

## The work (one paragraph)
Make **every error/diagnostic type serialize to structured EDN through ONE trait** (`ToEdn` — or intueri a better
name and note it), so a stringly diagnostic is uncompilable at the serialization boundary. Today it is a pile of
ad-hoc free functions (`runtime_error_to_edn`, `macro_error_to_edn`, `startup_error_to_edn`, `payload_to_edn`,
`span_to_edn`) with whole error families still stringifying (non-Macro `StartupError`, `MainSignature`, the
`ProcessDiedError` payloads, CheckError via the interim `Diagnostic` type). Mint the trait, converge the existing
serializers into impls, bring the holdouts under it, retire the interim `Diagnostic`, and make the boundary generic
over `ToEdn`. **Slice 296.1 (the macro chain) is already landed (`f397aba6`)** — build ON it.

## Read first
- **`docs/arc/2026/06/296-diagnostics-fully-edn/DESIGN.md`** — the full model, the asymmetry (grounded), the target
  shape, the decomposition, the blast radius, the out-of-scope cuts. This brief is the build order + the rooms.
- **`src/runtime_error_edn.rs:40`** (`runtime_error_to_edn`) — the reference shape every serializer mirrors; arc 233.
- **`src/macros/error_edn.rs:38/140`** (`macro_error_to_edn` / `startup_error_to_edn`) — slice 1's serializers.

## The rooms (grounded 2026-06-30)
- **The serializers to converge into trait impls:** `runtime_error_to_edn` (`runtime_error_edn.rs:40`),
  `macro_error_to_edn` + `startup_error_to_edn` (`macros/error_edn.rs:38/140`), `payload_to_edn` + `span_to_edn`
  (`panic_hook.rs:145/209`), `value_snapshot_to_edn` / `provenance_to_edn` (`runtime_error_edn.rs:275/289`).
- **The ProcessDiedError payloads (String → tagged EDN):** the 4 builders at `runtime.rs:22095` (StartupError),
  `:22112` (MainSignature), `:22128` (BadReturn), + EntryFormFailure; the variant decls `types.rs:988-1000`.
- **THE COMPAT BOUNDARY — `extract-panics`:** `runtime.rs:11802` (`fn extract_panics(err: &Value)`), the verb at
  `:4818`. It reads the `ProcessDiedError` chain back. **If you change a payload from `String` → a tagged EDN value,
  `extract_panics` MUST read the new structured form and its round-trip MUST stay green.**
- **The IPC emit sites:** `process/verbs.rs` (the `process_died_error_*_value(format!("{}", e))` sites — slice 1
  structured only the Macro one; bring the rest).
- **The interim `Diagnostic` to retire:** `src/diagnostic.rs` (struct + `DiagnosticValue` + `render_edn_value`) +
  `freeze.rs:600` (`StartupError::diagnostics()`, the Parse/Config/Load/Type `format!` sites `:605-617`) + the
  `--check-output edn|json` consumers.

## Decomposition (build IN ORDER; FULL gate `cargo nextest run --release` GREEN after EACH slice)
### 296.2 — mint `ToEdn` + converge the existing serializers (LOW risk; locks the contract)
Define `trait ToEdn { fn to_edn(&self) -> OwnedValue; }` in a small home (e.g. `src/diagnostic_edn.rs`). Implement it
for `RuntimeError`, `MacroError`, `StartupError`, `Span`, `AssertionPayload`, `ValueSnapshot`, `Provenance` — the impl
bodies are the EXISTING free functions (move the body in, or have the impl call the existing fn — ONE canonical path,
no behavior change). Route slice 1's macro-chain emission (`verbs.rs`) through `.to_edn()`. **Gate.** Probe: a unit
test that `<RuntimeError>.to_edn()` == the old `runtime_error_to_edn(&e)` (behavior-preserving).

### 296.3 — the holdouts: ProcessDiedError payloads → structured EDN (HIGH risk — the compat slice)
Change the `ProcessDiedError` variant payloads (`StartupError`/`MainSignature`/`BadReturn`/`EntryFormFailure`) from a
`String` field to a **tagged EDN value** (`types.rs:988-1000` + the 4 builders `runtime.rs:22095+`). The `verbs.rs`
sites pass the structured `to_edn()` instead of `format!("{}", e)`. **`extract_panics` (`runtime.rs:11802`) reads the
new structured payload** — keep its round-trip GREEN. Write a RED probe FIRST: a non-Macro startup/main-signature
error across the process boundary emits a structured (tagged, navigable) payload, not a `Value::String`. **Gate.**
> **STOP-1 (the round-trip):** if `extract_panics` cannot read the structured payload without breaking its consumers,
> or the payload-type change ripples past `extract_panics` + the 4 builders + `verbs.rs` into unrelated code — **STOP
> and report** with exactly what breaks. Do NOT half-migrate or leave a stringly fallback for a variant you touched.

### 296.4 — retire the interim `Diagnostic`
`CheckError` (and Parse/Config/Load/Type) implement `ToEdn`; `freeze.rs:600 diagnostics()` returns `to_edn()` values;
the `--check-output edn|json` modes consume `to_edn()` directly; delete `src/diagnostic.rs` (struct + `DiagnosticValue`
+ `render_edn_value`) once nothing references it. **Gate.** Probe: `--check-output edn` on a type error emits a
structured `#wat.kernel/CheckError {…}`, not the flat `Diagnostic` shape.

### 296.5 — the wall + close
Make the serialization boundary **generic over `ToEdn`** (the wire/emit entry takes `impl ToEdn` / `&dyn ToEdn`) so a
non-`ToEdn` error has no path to the wire. Add a doc/compile-fence note (a new error variant without a `ToEdn` impl
fails to reach the boundary). Write the INSCRIPTION; flip the 296 DESIGN status to closed. **Gate.**

## Out of scope (do NOT touch)
The human-readable **`Display` impls STAY** — they render text for the in-process harness (`src/harness.rs`). EDN is
the wire / IPC / `--check-output` face; `Display` is the human face. Do not delete or rewrite a `Display` to pass a test.

## EXPECTATIONS (the scorecard)
| # | what | command | expected |
|---|---|---|---|
| 1 | the trait exists + existing serializers are impls | `grep -rn "trait ToEdn\|impl ToEdn" src/` | the trait + impls for RuntimeError/MacroError/StartupError/Span/CheckError/… |
| 2 | NO error family stringifies its wire payload | `grep -rn 'format!("{}", e)' src/process/verbs.rs` | 0 (all sites carry `to_edn()`) |
| 3 | the interim `Diagnostic` is GONE | `ls src/diagnostic.rs` ; `grep -rn "DiagnosticValue\|Diagnostic::new" src/` | file deleted; 0 hits |
| 4 | extract-panics round-trip holds | the existing extract-panics tests + the new holdout probe | GREEN |
| 5 | each slice green + final | `cargo nextest run --release` after each slice | 0 failed throughout |
| 6 | clean build | `cargo build --release` | clean (no new warnings) |

## Discipline
You are the executor. Anchor `/home/watmin/work/holon/wat-rs`; `pwd` first; reject `.claude/worktrees/`. Do NOT spawn
subagents. Do NOT commit. Build incrementally; **run the FULL gate after each slice** and report the count. Read every
diff end-to-end (you are deleting + retyping error payloads — confirm nothing outside the named rooms moved). STOP +
report at STOP-1 (the extract-panics round-trip) or if any slice exceeds its named blast radius. Report per slice:
the trait/impls, the diff stat, the new probe + its RED→GREEN, the gate count, any STOP, any deviation.
