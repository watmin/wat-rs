# Sub-DESIGN — Stone 233.3 — Errors-as-EDN extension

**Status:** ACTIVE (2026-05-23 night). Sub-DESIGN under arc 233 (after Stone 233.2 sub-chain a/b/c/d/f/g/h/i/j/k/l/e all SHIPPED).

**Driver:** Arc 211b shipped `#wat.kernel/AssertionFailure {...}` EDN envelope for panic payloads. Stone 233.3 GENERALIZES that pattern to ALL 28 `RuntimeError` variants. Errors flowing across IPC boundaries (cross-thread, cross-process, cross-language) serialize as EDN, not strings. Tools consume errors as data — `wat-edn::parse(stderr_line)` yields a `Value` the consumer can introspect, rather than regex-matching opaque text.

This is the **IPC interop payoff** of arc 233. With provenance + ValueSnapshot now meaningful (post-j/k/l/e), errors carry rich structured context. Letting that context flow as EDN over the wire makes downstream consumers (Clojure-IPC bridge per arc 217, wat-MCP horizon, future tooling) able to reason about errors structurally.

## Current state

**Arc 211b precedent** (`src/panic_hook.rs:137-180`):
```rust
pub(crate) fn payload_to_edn(payload: &AssertionPayload) -> OwnedValue {
    // ── 7 keys: thread, message, location, actual, expected, frames, _other_ ──
    // returns OwnedValue::Map with structured fields
}

fn write_assertion_failure<W: Write>(out: &mut W, payload: &AssertionPayload) {
    let edn_value = payload_to_edn(payload);
    let line = format!("#wat.kernel/AssertionFailure {}\n", wat_edn::write(&edn_value));
    let _ = out.write_all(line.as_bytes());
}
```

Wire format: `#wat.kernel/<Tag> {<map-body>}\n` — single line, newline-terminated, tag-prefixed EDN.

**Helpers** (`src/panic_hook.rs:198+`):
- `span_to_map(&Span) -> OwnedValue` — converts Span to EDN map
- `frame_to_map(&FrameInfo) -> OwnedValue` — converts backtrace frame to EDN map

**RuntimeError variants** (`src/runtime.rs:~2020`): **28 variants** spanning:
- Type errors: NotCallable, TypeMismatch, ArityMismatch, BadCondition
- Form errors: MalformedForm, EvalForbidsMutationForm
- Macro errors: NoMacroRegistry, MacroExpansionFailed
- Channel errors: ChannelDisconnected, ServiceNotRunning
- Assertion: AssertionFailed (mirrors AssertionPayload!)
- Sandbox: SandboxScopeLeak, EvalVerificationFailed
- Eval-stepper: NoStepRule, EffectfulInStep
- Pattern: PatternMatchFailed
- Tail call: TailCall
- EDN: EdnCoerceMismatch
- (plus more)

**Gap:** RuntimeError has Display impl (string formatting) but NO EDN serialization. When RuntimeError surfaces at process exit (wat-cli exit code 2 per lib.rs:83), only opaque text reaches stderr.

## Doctrine — what this enables structurally

**Pre-state:** errors are strings on the wire. Consumer parses Display text via heuristics or regex. Provenance + structured context (from 233.2) reaches Display via fmt::Write but loses structure on stderr.

**Post-state (233.3):** errors are EDN on the wire. Consumer parses with `wat-edn::parse` (or any EDN parser; arc 092 standardized the format). Provenance + ValueSnapshot fields surface as structured map entries. Tooling can:
- Pattern-match error variants by tag (`#wat.kernel/NotCallable` vs `#wat.kernel/TypeMismatch`)
- Extract span coordinates programmatically
- Read producer/binding provenance for trace reconstruction
- Forward errors across language boundaries (Clojure-IPC, MCP)

The diagnostic-richness substrate (now meaningful per 233.2.e) becomes machine-consumable.

## Shape decisions

### Decision 1 — Module location

**Option A (CHOSEN):** new module `src/runtime_error_edn.rs`. Self-contained; reuses helpers from panic_hook via `pub(crate)`. Mirrors crate layout.

**Option B (REJECTED):** inline in `runtime.rs`. Bloats already-large file (28 variant arms + helpers).

**Option C (REJECTED):** extend `panic_hook.rs`. Conflates concerns (panic emission vs runtime-error emission).

**Verdict:** Option A. New file `src/runtime_error_edn.rs`.

### Decision 2 — Tag scheme

**Option A (CHOSEN):** `#wat.kernel/<VariantName>` per variant. Matches arc 211b AssertionFailure precedent + arc 092 FQDN-tag doctrine. Examples:
- `#wat.kernel/NotCallable {got <ValueSnapshot-EDN>, span <span-map>}`
- `#wat.kernel/TypeMismatch {op "...", expected "...", got <snap>, span <map>}`
- `#wat.kernel/AssertionFailed {message "...", actual <opt>, expected <opt>, span <map>}` (mirrors `#wat.kernel/AssertionFailure` from arc 211b but as a RuntimeError variant, not a panic payload)

**Option B (REJECTED):** generic `#wat.kernel/RuntimeError {:variant <name>, :body {...}}` wrapper. Nested; less direct pattern-match on tag.

**Verdict:** Option A. Per-variant tags. Pattern-match-friendly.

### Decision 3 — Field encoding

Each variant's struct fields → EDN map keys (keyword form: `:op`, `:got`, `:span`, etc.). Field types map as:
- `String` → `OwnedValue::String`
- `&'static str` → `OwnedValue::String` (Cow::Borrowed)
- `Span` → `span_to_map(&span)` (reuse existing helper)
- `ValueSnapshot` → `value_snapshot_to_edn(&snap)` (NEW helper)
- `Option<T>` → `Nil` for None, `T-EDN` for Some
- nested error types (e.g., `crate::hash::HashError`) → recurse via new helpers if needed
- arbitrary `Box<RuntimeError>` (if any) → recurse runtime_error_to_edn

### Decision 4 — Helpers

**Make accessible across modules:**
- `span_to_map` — currently `fn` (private) in panic_hook.rs. Mint `pub(crate) fn span_to_edn(&Span)` accessible to runtime_error_edn.rs.
- `value_snapshot_to_edn(&ValueSnapshot) -> OwnedValue` — NEW. Maps `{type_name, rendered, provenance}` to `{:type "...", :rendered "...", :provenance <provenance-edn>}`. Provenance variants get their own tags:
  - `Provenance::Unknown` → `OwnedValue::Nil` (or `#wat.kernel/Unknown nil`)
  - `Provenance::Literal { span }` → `#wat.kernel/Literal {:span <span-map>}`
  - `Provenance::SymbolBound { binding_span, head_span }` → `#wat.kernel/SymbolBound {:binding-span <map>, :head-span <map>}`
  - `Provenance::RuntimeBuilt { producer, call_span }` → `#wat.kernel/RuntimeBuilt {:producer "...", :call-span <map>}`

### Decision 5 — API shape

**Option A (CHOSEN):** free function `pub fn runtime_error_to_edn(err: &RuntimeError) -> OwnedValue` in runtime_error_edn module. Matches `payload_to_edn` precedent.

**Option B (REJECTED):** method `impl RuntimeError { pub fn to_edn(&self) -> OwnedValue }`. More ergonomic but bloats RuntimeError impl block (already large per the 28 variants).

**Verdict:** Option A. Free function.

Plus a wire-emission helper:
```rust
pub fn emit_runtime_error_envelope<W: Write>(out: &mut W, err: &RuntimeError) {
    let edn_value = runtime_error_to_edn(err);
    let tag = variant_tag(err);  // e.g., "wat.kernel/NotCallable"
    let line = format!("#{} {}\n", tag, wat_edn::write(&edn_value));
    let _ = out.write_all(line.as_bytes());
}
```

(Or fold the tag into the map shape — sonnet picks based on what reads cleanest.)

### Decision 6 — Wire emission boundary

**Option A (CHOSEN):** Wire emission at wat-cli's RuntimeError exit boundary (in `crates/wat-cli/`). When `:user::main` returns Err(RuntimeError), wat-cli currently exits with code 2 + Display text on stderr. Update to ALSO emit EDN envelope on stderr (or REPLACE the Display text with EDN envelope per HARD CUT).

**Option B (REJECTED):** Mint a panic_hook-like global runtime-error handler. Conflates panic + runtime-error; runtime errors are returned values not panics; structurally different.

**Option C (DEFERRED):** Cross-thread / cross-process channel boundaries. When typed-channel decode produces RuntimeError on receive, the receiver gets it in-process; no IPC at error-display time. Wire emission is on the EMITTING side (when emitting to stderr/file). Out of 233.3 scope.

**Verdict:** Option A. wat-cli's exit boundary gets the wire emission. HARD CUT — replace Display text with EDN envelope (one canonical form; mirror arc 211b's pattern for panics).

### Decision 7 — Backward compatibility

**HARD CUT:** replacing Display-on-stderr with EDN-on-stderr changes the user-facing surface. Any consumer scripting against Display text format BREAKS. Per arc 233's doctrine (errors are remarkable; tools consume as data) + `feedback_wat_llm_first_design` (one canonical path): the HARD CUT is the right call.

If we discover a real consumer that needs Display text post-stone, a SEPARATE follow-up arc adds a `WAT_ERROR_FORMAT=text` env-flag fallback. Out of 233.3 scope.

## Implementation surface

### Phase 1 — Helper extraction + extension

1. `src/panic_hook.rs`: change `fn span_to_map` to `pub(crate) fn span_to_edn` (rename for consistency). Update internal callers.
2. New `src/runtime_error_edn.rs` module:
   - `pub fn value_snapshot_to_edn(&ValueSnapshot) -> OwnedValue` — maps {type_name, rendered, provenance}
   - `pub fn provenance_to_edn(&Provenance) -> OwnedValue` — 4-arm dispatch with per-variant tag
   - `pub fn runtime_error_to_edn(&RuntimeError) -> OwnedValue` — 28-arm dispatch returning tagged EDN

### Phase 2 — 28 RuntimeError variant arms

Each arm maps the struct/tuple fields to EDN keys. Spec:
- NotCallable {got, span} → `#wat.kernel/NotCallable {:got <snap-edn>, :span <span-edn>}`
- TypeMismatch {op, expected, got, span} → `#wat.kernel/TypeMismatch {:op "...", :expected "...", :got <snap-edn>, :span <span-edn>}`
- ArityMismatch {op, expected, got, span} → `#wat.kernel/ArityMismatch {:op "...", :expected N, :got N, :span <span-edn>}`
- BadCondition {got, span} → `#wat.kernel/BadCondition {:got <snap-edn>, :span <span-edn>}`
- ... (24 more)

### Phase 3 — Wire emission helper

```rust
pub fn emit_runtime_error_envelope<W: Write>(out: &mut W, err: &RuntimeError) {
    let edn_value = runtime_error_to_edn(err);
    let tag = err.variant_tag();  // method on RuntimeError or free fn
    let line = format!("#{} {}\n", tag, wat_edn::write(&edn_value));
    let _ = out.write_all(line.as_bytes());
}
```

### Phase 4 — wat-cli integration

Find the boundary in `crates/wat-cli/` where `:user::main`'s RuntimeError is handled. Replace `eprintln!("{}", err)` (Display text) with `emit_runtime_error_envelope(&mut stderr(), &err)`. Exit code 2 unchanged.

### Phase 5 — Update src/lib.rs / re-exports

Make `runtime_error_to_edn` + `emit_runtime_error_envelope` publicly accessible if needed by external consumers (e.g., wat-cli is a downstream crate).

## FM 2-bis probe plan

Write `tests/probe_stone_233_3_runtime_error_edn.rs` with 5 contracts BEFORE the BRIEF:

1. **NotCallable round-trip:** construct a NotCallable RuntimeError; serialize via runtime_error_to_edn; parse the result with wat-edn::parse; assert the parsed Value's tag is `wat.kernel/NotCallable` and the map has `:got` + `:span` keys.

2. **TypeMismatch round-trip:** same shape; verify op, expected, got, span all surface as map keys.

3. **AssertionFailed round-trip:** consistent with arc 211b AssertionFailure precedent — same tag pattern, same key naming convention. Mirror not equal but compatible.

4. **All 28 variants have unique tags:** static enumeration; for each variant construct an instance + serialize + check tag is unique + matches `wat.kernel/<VariantName>` pattern.

5. **Provenance variants render with tags:** SymbolBound + Literal + RuntimeBuilt each get their tag in value_snapshot_to_edn output.

Probe ships FAILING pre-stone (runtime_error_to_edn doesn't exist; compile-fails). Sonnet's mission: flip to 5/5.

## Calibration prediction

| Stone | Predicted |
|---|---|
| 233.3 | **60–120 min Mode A; 180 min STOP** |

Mechanical sweep of 28 variants + helpers + wat-cli wire-up + probe. Smaller than 233.2.j (eval_inner cascade) but larger than 233.2.l (focused proc-macro).

**Risks:**
- 28 variants × correct field-shape mapping: tedious but not hard
- wat-cli integration may have boundary subtleties (panic vs runtime-error handling already coexist)
- HARD CUT on wire format may break a test that asserts on Display text — verify baseline holds

## Trap-door audit (FM 2-bis pre-flight)

- [x] arc 211b AssertionFailure pattern enumerated; payload_to_edn shape documented
- [x] 28 RuntimeError variants counted; field shapes available via grep
- [x] span_to_map + frame_to_map exist as helpers (need pub(crate) extraction)
- [x] wat-edn `write(&OwnedValue) -> String` is the serialization entry
- [ ] Verify wat-cli's exact error-handling boundary (sonnet greps during work)
- [ ] Verify no test asserts on Display text of RuntimeError that would break under HARD CUT (sonnet runs baseline)
- [ ] Verify provenance fields in NotCallable/TypeMismatch/BadCondition (the 3 ValueSnapshot carriers) flow into EDN via value_snapshot_to_edn
- [ ] Verify nested error types (HashError in EvalVerificationFailed) get reasonable encoding (simple struct → EDN map OR display-string fallback if too complex)

## Builds on / unblocks

**Builds on:**
- 233.1 (ValueSnapshot — the rich structured got/expected fields)
- 233.2.j/k/l/e (provenance machinery — now meaningful in EDN output)
- arc 211b (AssertionPayload precedent — the pattern this generalizes)
- arc 092 (wat-edn — the serialization substrate)

**Unblocks:**
- arc 233 Stone 233.4 (INSCRIPTION — arc 233 closes once 233.3 lands)
- arc 217 (Clojure-IPC bridge — Clojure consumer can parse `#wat.kernel/*` envelopes as `ex-info`-equivalent)
- wat-MCP horizon — MCP tools consume structured errors instead of regex-matching text
- Cross-language error propagation in general

## Out of scope (affirmative scope-bounding)

- **Cross-thread channel error propagation as EDN** — receiver gets RuntimeError in-process; no wire. Wire emission is on the EMITTING side only. Out of scope.
- **WAT_ERROR_FORMAT=text fallback** — HARD CUT replacement; if a consumer needs Display text, a separate follow-up arc adds the flag. Not load-bearing today.
- **Error-recovery patterns** (try/catch) — arc 108 typed-expect handles this layer; 233.3 is about the WIRE format, not the runtime semantics.
- **Performance optimization** — EDN serialization is at error-time (rare-path); no need for streaming/zero-alloc design.
- **holon-rs** — NOT touched.
- **HARD CUT** — no parallel API; the EDN envelope IS the error wire format post-233.3.

## Four-questions verdict

| Question | Verdict |
|---|---|
| **Obvious?** | YES — extend the AssertionFailure pattern across all RuntimeError variants; the seed is already in place |
| **Simple?** | YES — atomic per-variant mapping; uniform tag scheme; helpers reusable |
| **Honest?** | YES — HARD CUT on Display-as-stderr; no fallback flag; one canonical wire format. If a consumer breaks, the breakage surfaces honestly |
| **Good UX?** | YES — downstream tools parse errors as data; pattern-match on tag; extract span coordinates programmatically; arc 217 Clojure-IPC + wat-MCP get full structural error access |

PROCEED.

## Cross-references

- `src/panic_hook.rs` — arc 211b AssertionFailure precedent (the pattern this generalizes)
- `src/runtime.rs:~2020` — RuntimeError enum (28 variants)
- `crates/wat-edn/src/writer.rs` — wat-edn write API
- `crates/wat-cli/src/lib.rs` — wat-cli's exit boundary (Phase 4 integration site)
- `docs/arc/2026/04/092-wat-edn-v4/` — wat-edn substrate
- `docs/arc/2026/05/211b-panic-as-edn/` — the seed arc
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 2-bis — probe-before-BRIEF
- `feedback_wat_llm_first_design` — one canonical path (HARD CUT justification)
- `feedback_partial_state_grading` — discipline if STOP-3 fires
