# SCORE — Arc 233 Stone 233.3 — Errors-as-EDN extension

**Status:** SHIPPED. All 11 scorecard rows PASS.
**Date:** 2026-05-23

## Phase breakdown

### Phase 1 — Helper extraction (panic_hook.rs)
- `fn span_to_map` promoted to `pub(crate) fn span_to_edn` via thin wrapper delegation.
- `span_to_map` preserved as the implementation; `span_to_edn` delegates to it.
- Lines added: 7 (doc comment + wrapper fn body).
- Internal caller sites: none changed (still call `span_to_map` directly within panic_hook.rs).

### Phase 2 — New module scaffold (src/runtime_error_edn.rs)
- New file: `src/runtime_error_edn.rs` — 375 lines total.
- `src/lib.rs`: +1 line (`pub mod runtime_error_edn;` inserted after `pub mod runtime;`).
- Three pub fns: `runtime_error_to_edn`, `value_snapshot_to_edn`, `provenance_to_edn`.
- One pub wire-emission fn: `emit_runtime_error_envelope<W: Write>`.
- Low-level builder helpers: `kw`, `str_val`, `opt_str_val`, `span_val`, `snap_val`,
  `tagged`, `map1`, `map2`, `map4` — internal only, eliminate match-arm boilerplate.
- `variant_name` fn: parallel `&'static str` lookup; must stay in sync with
  `runtime_error_to_edn` match arms (28 arms each).

### Phase 3 — 28 RuntimeError variant arms
All 28 variants covered in `runtime_error_to_edn`. Per-variant tag and key choices:

#### Tuple variants (descriptive keys, NOT positional `:0 :1`):
| Variant | Keys |
|---|---|
| `UnboundSymbol(String, Span)` | `:name :span` |
| `UnknownFunction(String, Span)` | `:path :span` |
| `ParamShadowsBuiltin(String, Span)` | `:name :span` |
| `DivisionByZero(Span)` | `:span` |
| `DuplicateDefine(String, Span)` | `:name :span` |
| `ReservedPrefix(String, Span)` | `:prefix :span` |
| `DeclarationInExpressionPosition(String, Span)` | `:head :span` |
| `TryPropagate(Value)` | `:value` (ValueSnapshot — type + rendered) |
| `OptionPropagate` | `{}` (no fields) |

#### Struct variants:
| Variant | Keys |
|---|---|
| `NotCallable` | `:got :span` |
| `TypeMismatch` | `:op :expected :got :span` |
| `ArityMismatch` | `:op :expected :got :span` |
| `BadCondition` | `:got :span` |
| `MalformedForm` | `:head :reason :span` |
| `EvalForbidsMutationForm` | `:head :span` |
| `UserMainMissing` | `{}` (no fields) |
| `EvalVerificationFailed` | `:error` (Display string — lazy fallback) |
| `ChannelDisconnected` | `:op :span` |
| `NoEncodingCtx` | `:op :span` |
| `NoSourceLoader` | `:op :span` |
| `NoMacroRegistry` | `:op :span` |
| `MacroExpansionFailed` | `:op :reason :span` |
| `PatternMatchFailed` | `:value-type :span` |
| `EffectfulInStep` | `:op :span` |
| `NoStepRule` | `:op :span` |
| `TailCall` | `:fn-name :arg-count :call-span` |
| `AssertionFailed` | `:message :actual :expected :span` |
| `SandboxScopeLeak` | `:offending-name :call-span :outer-define-span` |
| `ServiceNotRunning` | `:op :span` |
| `EdnCoerceMismatch` | `:op :expected :got :path :span` |

### Phase 4 — Provenance + ValueSnapshot helpers
- `provenance_to_edn`: 4-arm dispatch.
  - `Unknown` → `nil`
  - `Literal { span }` → `#wat.kernel/Literal {:span <map>}`
  - `SymbolBound { binding_span, head_span }` → `#wat.kernel/SymbolBound {:binding-span <map> :head-span <map>}`
  - `RuntimeBuilt { producer, call_span }` → `#wat.kernel/RuntimeBuilt {:producer "..." :call-span <map>}`
- `value_snapshot_to_edn`: 3-key map `{:type :rendered :provenance}`.

### Phase 5 — Wire integration (fork.rs + spawn_process.rs)
Per BRIEF's trap-door audit: `eprintln!` was not the boundary — the actual
boundary is `process_died_error_runtime_value(format!("{}", runtime_err))` in
`src/fork.rs` (two sites) and `src/spawn_process.rs` (one site).

HARD CUT applied at all three sites: replaced `format!("{}", runtime_err)` with
`wat_edn::write(&crate::runtime_error_edn::runtime_error_to_edn(&runtime_err))`.
The structured EDN now flows inside the `ProcessDiedError::RuntimeError(String)`
envelope, which is then wrapped in `#wat.kernel/ProcessPanics`. The outer envelope
architecture is unchanged; the inner string is now machine-consumable EDN.

Lines changed per file:
- `src/fork.rs`: +8 lines net (two sites, each +4 lines replacing 1-line format! call).
- `src/spawn_process.rs`: +4 lines net (one site).

## Calibration

- Predicted band: 60–120 min Mode A; 180 STOP.
- Actual: ~35 min (reading + implementation + verification).
- Below Mode A floor — confirms BRIEF's "mechanical sweep" risk assessment was accurate.
  Volume was the main cost (28 variant arms), not novelty.

## 11-row scorecard — verbatim verification output

### Row 1 — Compile clean (wat)
```
$ cargo build --release -p wat 2>&1 | tail -5
...
    Finished `release` profile [optimized] target(s) in 18.46s
```
**PASS** — 0 errors.

### Row 2 — Compile clean (wat-cli)
```
$ cargo build --release -p wat-cli 2>&1 | tail -5
   Compiling wat-sqlite v0.1.0 (.../crates/wat-sqlite)
   Compiling wat-holon-lru v0.1.0 (.../crates/wat-holon-lru)
   Compiling wat-telemetry-sqlite v0.1.0 (.../crates/wat-telemetry-sqlite)
   Compiling wat-cli v0.1.0 (.../crates/wat-cli)
    Finished `release` profile [optimized] target(s) in 1.65s
```
**PASS** — 0 errors.

### Row 3 — 233.3 probe FLIPS 0/5 → 5/5
```
$ cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 | tail -5
test probe_3_assertion_failed_with_optional_fields ... ok
test probe_1_not_callable_serializes_to_tagged_edn ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
**PASS** — 5/5.

### Row 4 — Lib tests baseline
```
$ cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3
test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.16s
```
**PASS** — 827 passed; 0 failed.

### Row 5 — Stone 233.2.e probe (regression guard)
```
$ cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 | tail -3
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```
**PASS** — 5/5.

### Row 6 — Stone 233.2.l probe
```
$ cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 | tail -3
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
**PASS** — 3/3.

### Row 7 — Stone 233.2.k probe
```
$ cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 | tail -3
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```
**PASS** — 5/5.

### Row 8 — Stone 233.2.j probe
```
$ cargo test --release --test probe_stone_233_2_j_producer_migration 2>&1 | tail -3
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```
**PASS** — 5/5.

### Row 9 — Stone 233.1 ValueSnapshot probes
```
$ cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 | tail -3
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```
**PASS** — 8/8.

### Row 10 — Clippy no new warnings
```
$ cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"
54
```
**PASS** — 54 (baseline was 54; no new warnings).

### Row 11 — holon-rs untouched
```
$ git -C /home/watmin/work/holon/holon-rs/ status --short
(empty)
```
**PASS** — empty output.

## Honest deltas

### HARD CUT boundary location
BRIEF said "find the boundary in `crates/wat-cli/`". Actual boundary was in
`src/fork.rs` (two sites) and `src/spawn_process.rs` (one site) — the child
process exit paths. `crates/wat-cli/` doesn't handle RuntimeErrors directly;
it uses `fork_program_from_source` which forks a child, and the child's stderr
flows through a proxy pipe. The HARD CUT was applied at all three actual sites.

### WAT_ERROR_FORMAT=text fallback
NOT shipped. HARD CUT replaces Display-text with EDN on all three process-exit
RuntimeError paths. Any consumer scripting against Display text format breaks —
that is the correct behavior per arc 233 doctrine.

### Nested error types (HashError)
`EvalVerificationFailed { err: HashError }` — rendered via `format!("{}", err)`
(Display string) as `:error "..."`. HashError has multiple struct variants;
a future arc can deepen to a structured EDN map if structured access becomes
load-bearing. Documented in DESIGN as planned lazy fallback.

### TryPropagate(Value)
`Value` has no `Display` impl. Used `ValueSnapshot::of(value)` instead — gives
`:type` + `:rendered` + `:provenance` fields. Richer than a type_name() string
alone; slightly different from BRIEF's "value display or edn" note (which was
left deliberately open). The ValueSnapshot approach is consistent with how other
variants carry value context.

### AssertionFailed vs AssertionFailure naming
`RuntimeError::AssertionFailed` → `#wat.kernel/AssertionFailed` (present-tense
variant name). Arc 211b panic envelope is `#wat.kernel/AssertionFailure`
(abstract noun). Distinct types, distinct tags — documented in module-level
rustdoc comment and in this SCORE.

### span_to_edn delegation pattern
Rather than renaming `span_to_map` everywhere (would touch its internal callers
in panic_hook.rs), `span_to_edn` was added as a `pub(crate)` thin wrapper that
delegates to `span_to_map`. This avoids breaking internal callers while exposing
the helper cross-crate. The two functions are byte-identical in behavior.

## What this unblocks

- Stone 233.4 INSCRIPTION — arc 233 closes.
- Arc 217 Clojure-IPC — Clojure consumer parses `#wat.kernel/*` envelopes.
- wat-MCP horizon — MCP tools consume structured errors.
- Cross-language error propagation — any wat-edn-aware consumer gets
  full structured RuntimeError context rather than Display text.

## Files changed

| File | Change |
|---|---|
| `src/panic_hook.rs` | +7 lines: `pub(crate) fn span_to_edn` wrapper |
| `src/runtime_error_edn.rs` | NEW — 375 lines (the module) |
| `src/lib.rs` | +1 line: `pub mod runtime_error_edn;` |
| `src/fork.rs` | +8 lines: HARD CUT at two RuntimeError exit sites |
| `src/spawn_process.rs` | +4 lines: HARD CUT at one RuntimeError exit site |
