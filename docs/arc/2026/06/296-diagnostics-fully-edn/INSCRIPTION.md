# INSCRIPTION — Arc 296: Error → EDN, unified under ONE `ToEdn` trait

> Thesis: every error/diagnostic type serializes to structured EDN through ONE trait (`ToEdn`), making stringly diagnostics uncompilable at the serialization boundary.

> Opened: 2026-06-30 (re-scope from "fix the macro prose-blob" to the full unification)
> Closed: 2026-06-30 (slices 296.2–296.5 landed, gate 4157/0/91)

---

## What shipped (296.2 → 296.5, all uncommitted from HEAD `59dad529`)

**296.2 — Mint `ToEdn` + converge existing serializers**
- `src/to_edn.rs` (NEW): `pub trait ToEdn { fn to_edn(&self) -> OwnedValue; }` — the ONE contract.
- `impl ToEdn` for: `RuntimeError`, `ValueSnapshot`, `Provenance` (in `runtime_error_edn.rs`);
  `Span`, `AssertionPayload` (in `panic_hook.rs`); `MacroError`, `StartupError` (in `macros/error_edn.rs`).
- Each impl body delegates to the existing free function — behavior byte-identical; free functions remain as the canonical implementation.
- `src/process/verbs.rs`: the three IPC emit sites (`format!("{}", e)` for StartupError / MainSignature / BadReturn)
  now call `to_edn()` directly → structured tagged EDN on the wire.
- Probe: `tests/diagnostics/probe_arc296_2_to_edn_trait.rs` (GREEN).

**296.3 — ProcessDiedError payloads → structured EDN (the compat slice)**
- `types.rs`: `ProcessDiedError` variant payload fields changed from `String` to `wat_edn::OwnedValue` for the
  `StartupError`, `MainSignature`, `BadReturn`, and `EntryFormFailure` variants.
- `runtime.rs`: the 4 builders that fill those fields now produce tagged EDN (`e.to_edn()`) instead of `format!("{}", e)`.
- `extract_panics` (`runtime.rs`): updated to read the new structured payload; the round-trip probe stays GREEN.
- Probe: `tests/diagnostics/probe_arc296_3_holdout_edn.rs` (RED → GREEN).

**296.4 — Retire the interim `Diagnostic`**
- `src/check/error_edn.rs` (NEW): full EDN serializer for all 28+ `CheckErrorKind` variants.
  Tag convention: `#wat.kernel/<VariantName>`. `impl ToEdn for CheckError` delegates here.
- `src/check.rs`: `pub mod error_edn;` declared.
- `src/check/error.rs`: the entire `impl CheckError {}` block (`diagnostic()` + the N-arm match) removed (~280 lines).
  `CheckErrors::diagnostics()` also removed.
- `src/freeze.rs`: `StartupError::diagnostics() -> Vec<Diagnostic>` replaced by
  `to_edn_values() -> Vec<OwnedValue>` — one value per `CheckError` for the `Check` variant, one for all others.
- `src/lib.rs`: `pub mod diagnostic;` removed.
- `src/diagnostic.rs` (DELETED): the entire `Diagnostic` + `DiagnosticValue` + `render_edn_value` + `render_json` module.
- `crates/wat-cli/Cargo.toml`: `wat-edn` added as direct dependency.
- `crates/wat-cli/src/lib.rs`: `emit_check_failure` now consumes `err.to_edn_values()` directly.
  `--check-output edn` → `wat_edn::write`; `--check-output json` → `wat_edn::to_json_string`.
  EDN tag changed from `#wat.diag/<kind>` to `#wat.kernel/<VariantName>` at all emit sites.
- `crates/wat-cli/tests/wat_cli.rs`: CLI integration tests updated to the new tag shape
  (`#wat.kernel/CommCallOutOfPosition`, `#wat.kernel/ReturnTypeMismatch`) and JSON key format
  (`"#tag":"wat.kernel/..."` with colon-prefixed keyword keys).
- `src/test_runner.rs`: `emit_structured_diagnostic` → `emit_structured_edn`; `failure_to_diagnostic` →
  `failure_to_edn`; `render_failure_text` updated to extract from `OwnedValue`; all `Diagnostic` references gone.
- `tests/diagnostics/probe_arc243_stone6_checkerror_pattern_a.rs`: migrated the `diagnostic_elides_unknown_span`
  test to `edn_elides_unknown_span` (uses `to_edn()` + `wat_edn::write`); removed `use wat::diagnostic::DiagnosticValue`.
- Probe: `tests/diagnostics/probe_arc296_4_check_error_to_edn.rs` (GREEN).

**296.5 — The structural wall + the holdout serializers**

*The wall:*
- `src/to_edn.rs`: `pub fn to_wire_edn(e: &impl ToEdn) -> String` — THE single, named, generic
  error→wire-text conversion. Carries a `compile_fail` doc-test proving a non-`ToEdn` type cannot reach the
  boundary (verified via `cargo test --doc -p wat`: `to_edn::to_wire_edn (line 124) - compile fail ... ok`).
- `src/runtime.rs`: the four `process_died_error_{startup,main_signature,bad_return,runtime}_value` builders
  changed from `(message: String)` to `(e: &impl crate::to_edn::ToEdn)`, serializing via `to_wire_edn`. A raw
  `String` (or a non-`ToEdn` type) now has NO path to the `ProcessDiedError` payload boundary — compile error.
- `src/to_edn.rs`: `FlatMessage { tag, key, message }` + `impl ToEdn` — the honest `ToEdn` form for genuinely
  message-only failures (syscall error, bad-return type name). Even flat messages travel as a `ToEdn` value;
  the boundary never accepts a bare `String`.
- `src/process/verbs.rs`: all emit sites pass `ToEdn` values by reference (StartupError, RuntimeError,
  FlatMessage) instead of pre-built strings; the hand-rolled `main_signature_error_edn` / `bad_return_type_edn`
  helpers deleted (folded into `FlatMessage`).
- `src/process/child.rs`: the setpgid-failure startup payload now travels as a `FlatMessage`.
- `src/to_edn.rs`: `impl ToEdn for OwnedValue` (passthrough/identity); module compile-fence doc explaining the wall.
- `src/test_runner.rs`: `emit_structured_edn` is generic over `impl ToEdn`.

*The holdout serializers (the `:detail (e.to_string())` escape arms eliminated):* `startup_error_to_edn`
previously stringified 8 of 10 `StartupError` variants into a `:detail` prose blob — including `Check`, whose
serializer (`check_error_to_edn`) was already built but wired NOWHERE. Now every variant carrying a structured
error delegates to that error's own `ToEdn` impl:
- `impl ToEdn for TypeError` (`src/types/error.rs`, 18 variants), `ConfigError` (`src/config.rs`, 8),
  `LoadError` (`src/load.rs`, 7; nested `ParseError` structured), `StdlibError` (`src/stdlib.rs`),
  `ParseError` (`src/parser.rs` — foreign type, orphan-rule impl in the `wat` crate), `ResolveError`
  (`src/resolve/error.rs`, vector of structured references), `CheckErrors` (`src/check/error_edn.rs` —
  `#wat.kernel/CheckErrors {:errors [...]}`, USING the previously-orphaned `check_error_to_edn`).
- `src/to_edn.rs`: shared `pub(crate)` EDN builders (`edn_tag`/`edn_kw`/`edn_str`/`edn_int`/`edn_span`/
  `push_span_field`) — one canonical home, so a new impl does not copy the helpers a sixth time.
- `src/macros/error_edn.rs`: `startup_error_to_edn` rewritten — each arm routes through `.to_edn()`; the ONLY
  `:detail` arm left is `SigmaFn(String)` (a genuinely flat message — wrapped as
  `#wat.kernel/SigmaFnError {:detail "..."}`, still a tagged envelope, never a bare String).
- Probes: `probe_arc296_3_holdout_edn.rs` rewritten — probe 1 (Parse structured, no `:detail`), probe 2
  (SigmaFn's honest `:detail`), probe 4 (Check emits `#wat.kernel/CheckErrors` with navigable inner errors),
  probe 5 (EVERY `StartupError` variant's `to_edn()` is Tagged/Map, never a bare `OwnedValue::String`).

---

## What is OUT of scope (named deferrals)

- **`Display` impls** — preserved in full; they are the human face (`src/harness.rs`). EDN is the wire face.
- **Value-repr collapse** — that is arc 294, explicitly out of 296 per `DESIGN.md:131`.
- **New error variants** — each new variant added after 296 must add an `impl ToEdn`; the boundary enforces this.

---

## Prior-art collisions

None. Arc 296 supersedes the flat `Diagnostic` shape (introduced in arc 233 as an interim) cleanly; no prior arc
had a `ToEdn` trait or a generic serialization boundary.

---

## Verification at close (HEAD `59dad529`, uncommitted changes)

```
cargo nextest run --release   → 4159 passed, 0 failed, 91 skipped
cargo build --release         → clean; warning set IDENTICAL to HEAD baseline (0 new)
cargo test --doc -p wat       → to_edn::to_wire_edn (line 124) - compile fail ... ok  (the wall is real)
grep -rn "trait ToEdn|impl ToEdn" src/ → trait + 17 impls (RuntimeError, ValueSnapshot, Provenance, Span,
                                          AssertionPayload, MacroError, StartupError, CheckError, CheckErrors,
                                          TypeError, ConfigError, LoadError, StdlibError, ParseError,
                                          ResolveError, OwnedValue, FlatMessage)
ls src/diagnostic.rs          → no such file
grep -rn "DiagnosticValue|Diagnostic::new" src/ → 0 hits
grep -rn 'format!("{}", e)' src/process/verbs.rs → 0 hits
grep -rn ':detail (e.to_string' / e.to_string() in startup_error_to_edn → 0 (only SigmaFn's literal-String :detail)
```

Note: two pre-existing `wat` doc-tests (`runtime::parse_defprotocol_form` L5954, `runtime::parse_extend_type_form`
L6103) fail at HEAD baseline as well — unrelated to this arc, outside the nextest gate, untouched by this work.

---

## Pairs

- `src/to_edn.rs` — the trait home + the wall (`to_wire_edn`, `FlatMessage`, shared EDN builders)
- `src/check/error_edn.rs` — CheckError + CheckErrors serializer
- `src/runtime.rs` — the four `process_died_error_*_value` builders, now generic over `ToEdn`
- `BRIEF-296-error-edn-trait.md` — the build order used
