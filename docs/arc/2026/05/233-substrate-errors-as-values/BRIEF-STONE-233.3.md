# BRIEF — Arc 233 Stone 233.3 — Errors-as-EDN extension

## What we're doing

Mint `src/runtime_error_edn.rs` with three serializer functions that map RuntimeError + ValueSnapshot + Provenance to wat-edn `OwnedValue`. Generalizes arc 211b's `payload_to_edn` pattern across all 28 RuntimeError variants. Wire emission at wat-cli's exit boundary replaces `eprintln!("{}", err)` with `emit_runtime_error_envelope(...)` (HARD CUT — one canonical wire format).

After this stone: errors are machine-consumable EDN across IPC boundaries (cross-thread, cross-process, cross-language).

**Implementation surface (5 phases per sub-DESIGN):**

1. **Helper extraction** — promote `fn span_to_map` in `src/panic_hook.rs:198` to `pub(crate) fn span_to_edn` (rename for consistency). Update internal callers.

2. **New module `src/runtime_error_edn.rs`** with three pub fns:
   ```rust
   pub fn runtime_error_to_edn(err: &RuntimeError) -> wat_edn::OwnedValue;
   pub fn value_snapshot_to_edn(snap: &ValueSnapshot) -> wat_edn::OwnedValue;
   pub fn provenance_to_edn(prov: &Provenance) -> wat_edn::OwnedValue;
   ```
   Plus `pub fn emit_runtime_error_envelope<W: Write>(out: &mut W, err: &RuntimeError)`.

3. **28 RuntimeError variant arms** in `runtime_error_to_edn`:
   - Per-variant tag: `#wat.kernel/<VariantName>` (use `wat_edn::Tag::ns("wat.kernel", "<VariantName>")`)
   - Struct fields → EDN map keys (`:op`, `:got`, `:span`, etc.)
   - Tuple variants (ParamShadowsBuiltin, DivisionByZero, DuplicateDefine, ReservedPrefix, DeclarationInExpressionPosition, TryPropagate, OptionPropagate, UserMainMissing) — map positional fields to descriptive keys (sonnet picks `:name + :span` or similar based on field semantics)
   - Helper fields: Span → `span_to_edn`; ValueSnapshot → `value_snapshot_to_edn`; Provenance → `provenance_to_edn`

4. **Provenance variants** — `provenance_to_edn` 4-arm dispatch:
   - `Provenance::Unknown` → `OwnedValue::Nil`
   - `Provenance::Literal { span }` → `#wat.kernel/Literal {:span <span-edn>}`
   - `Provenance::SymbolBound { binding_span, head_span }` → `#wat.kernel/SymbolBound {:binding-span <map>, :head-span <map>}`
   - `Provenance::RuntimeBuilt { producer, call_span }` → `#wat.kernel/RuntimeBuilt {:producer "...", :call-span <map>}`

5. **wat-cli wire integration** — find the boundary in `crates/wat-cli/` that handles RuntimeError exit (per lib.rs:83 comment: "exit code 2 for any RuntimeError"). Replace Display-text emission with `emit_runtime_error_envelope(&mut std::io::stderr(), &err)`. Exit code 2 unchanged.

## Design substrate (READ FIRST; MANDATORY)

1. **`docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.3.md`** (commit `7436a3f`) — sub-DESIGN; seven decisions inscribed (module location, tag scheme, field encoding, helper sharing, API shape, wire emission boundary, HARD CUT vs backward compatibility). **Authoritative for shape decisions.**

2. **`tests/probe_stone_233_3_runtime_error_edn.rs`** (commit `186e880`) — FM 2-bis probe. 5 contracts. Pre-stone: FAILS to compile (`wat::runtime_error_edn` doesn't exist). **The probe IS the success criterion** — sonnet flips to 5/5.

3. **`src/panic_hook.rs`** — arc 211b precedent. Study `payload_to_edn` (line 137+) + `span_to_map` (line 198+) + `write_assertion_failure` (line 126+) for the pattern. Replicate the shape across RuntimeError variants.

4. **`src/runtime.rs:~2020`** — full RuntimeError enum (28 variants). Field shapes per variant.

5. **`crates/wat-edn/src/writer.rs`** — `wat_edn::write(&v) -> String` is the serialization entry. `wat_edn::Tag::ns("namespace", "name")` constructs tags.

6. **`crates/wat-cli/src/lib.rs:83`** — comment locating the exit-code-2 RuntimeError boundary. Wire integration site.

## What does NOT change

- **RuntimeError enum** — variants unchanged; only ADD serialization machinery
- **Provenance enum** — unchanged
- **ValueSnapshot** — unchanged
- **payload_to_edn / write_assertion_failure (arc 211b)** — unchanged; coexists with new runtime-error path
- **panic-hook emission** — unchanged; panic envelope is `#wat.kernel/AssertionFailure`, runtime-error envelope is per-variant
- **Other arc 233 probes** — all stay GREEN
- **holon-rs** — NOT touched

## Out of scope (affirmative scope-bounding)

- **WAT_ERROR_FORMAT=text fallback** — HARD CUT per Decision 7; separate follow-up arc if needed
- **Cross-thread channel error EDN-on-the-wire** — receiver gets RuntimeError in-process; out of scope
- **Performance optimization** — error path is rare; no streaming/zero-alloc needed
- **Display impl changes** — Display fmt for RuntimeError stays as-is (used for error-message text in tests, debug output)
- **holon-rs** — STOP-4
- **HARD CUT** — no parallel API; the EDN envelope IS the wire format

## Verification flow

```bash
cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 | tail -5    # 5/5 PASS post-stone
cargo build --release -p wat 2>&1 | tail -5                                       # 0 errors
cargo build --release -p wat-cli 2>&1 | tail -5                                   # 0 errors (wire integration compiles)
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3                   # ≥ 827 passed; 0 failed
cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 | tail -3 # 5/5 PASS (regression guard)
cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 | tail -3     # 3/3 PASS
cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 | tail -3    # 5/5 PASS
cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 | tail -3 # 8/8 PASS
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"       # ≤ 54
git -C /home/watmin/work/holon/holon-rs/ status --short                           # empty
```

## STOP triggers (REJECTION criteria)

- **STOP-1:** unexpected compile errors NOT tracing to the new module
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** **180 min elapsed** (per sub-DESIGN calibration: 60-120 Mode A; 180 STOP)
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning above 54
- **STOP-6:** scope creep — WAT_ERROR_FORMAT=text fallback; cross-thread channel EDN; Display impl rewrite
- **STOP-7:** probe still has failures post-stone (any of 5 contracts not PASS)
- **STOP-8:** existing arc 233 probes regress
- **STOP-9:** cascade exceeds time-box — apply partial-state-grading per `feedback_partial_state_grading`

Per FM 2-bis: STOP triggers are REJECTION criteria; never permission-to-defer.

## Trap-door audit

- **wat-cli boundary may have nuance** — currently RuntimeError might surface via different paths (run-hermetic vs spawn-process vs direct). Sonnet greps + picks the right boundary; if multiple, update all.
- **Existing tests that assert on Display text of RuntimeError on stderr** — would break under HARD CUT. Sonnet runs baseline to identify; either fix the test to assert on EDN envelope OR document as test-side ripple in SCORE.
- **Tuple variants (ParamShadowsBuiltin, etc.)** — pick descriptive key names (e.g., :name + :span for ParamShadowsBuiltin, :span only for DivisionByZero, etc.). Don't use `:0 :1` positional — that's hostile UX.
- **Nested error types** (`crate::hash::HashError` in EvalVerificationFailed) — for now, render as `:error <Display string>` (lazy fallback). A future arc can deepen if needed.
- **The arc 211b AssertionPayload pattern uses `#wat.kernel/AssertionFailure` tag** — RuntimeError::AssertionFailed variant uses `#wat.kernel/AssertionFailed` (variant name, present tense). Distinct from the panic envelope. Document the parallel naming.

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly per FM 12)
- HARD CUT — no parallel API; EDN envelope replaces Display text on stderr at wat-cli boundary
- Per `feedback_inscription_immutable`: SCORE is a NEW file (`SCORE-STONE-233.3.md`)
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification.
- The probe at `tests/probe_stone_233_3_runtime_error_edn.rs` IS the success criterion. Flip 0/5 → 5/5.
- This is the **IPC INTEROP PAYOFF stone**. Downstream consumers (arc 217 Clojure-IPC, wat-MCP horizon) parse `#wat.kernel/*` envelopes as structured data.

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.3.md` — sub-DESIGN (commit `7436a3f`)
- `tests/probe_stone_233_3_runtime_error_edn.rs` — FM 2-bis probe (commit `186e880`)
- `src/panic_hook.rs` — arc 211b AssertionFailure precedent (the pattern this generalizes)
- `src/runtime.rs:~2020` — RuntimeError enum (28 variants)
- `crates/wat-edn/src/writer.rs` — wat-edn write API
- `crates/wat-edn/src/value.rs` — Tag::ns constructor
- `crates/wat-cli/src/lib.rs:83` — wat-cli exit boundary for RuntimeError
- `scratch/FAILURE-ENGINEERING.md` — annihilation doctrine
- `feedback_wat_llm_first_design` — HARD CUT justification (one canonical path)
- `feedback_partial_state_grading` — discipline if STOP-3 fires
