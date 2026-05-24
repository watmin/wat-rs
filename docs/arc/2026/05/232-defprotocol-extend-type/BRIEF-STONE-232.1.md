# BRIEF — Arc 232 Stone 232.1 — defprotocol + extend-type macros (BUNDLED)

## What we're doing

Mint TWO wat-callable defmacros that complete the Clojure protocols surface on top of arc 226 (`is?`/dispatch) + arc 227 (defrecord) + arc 232.0 (`apply`) + arc 232.0a (`extract-classifier` + `Bind/{left,right}`):

1. **`:wat::holon::defprotocol`** — declares a protocol (a named set of method signatures). Generates ONE polymorphic dispatcher per declared method. The dispatcher routes per-first-arg-classifier via the canonical composition: `extract-classifier` → `string::concat` → `keyword/from-string` → `apply`.

2. **`:wat::holon::extend-type`** — extends a type with one or more method implementations for a protocol. Generates ONE `defn` per method-body at a mangled keyword name (`:NS::Type/Protocol-method`) the dispatcher routes to.

After this stone: defrecord-defined types can implement protocols via open extension; calling `(:NS::Protocol/method instance args...)` routes correctly to the per-type impl; missing impls raise observable `UnknownFunction` errors.

**Bundle decision** — Stone 232.1 ships BOTH macros together (originally split across 232.1 + 232.2; bundled via four-questions verdict 2026-05-23 night latest). Rationale: defprotocol alone produces a panic-generator until extend-type ships; the bundle ships one complete-and-useful stone. See `DESIGN-STONE-232.1.md` § Locked decisions D1.

## The rank-up — arc 233 + Stone 232.0a tools active

The same diagnostic substrate as Stone 232.0a:

- **ValueSnapshot in errors** — TypeMismatch errors render actual values + provenance. Iteration is faster.
- **Provenance tracking** — defrecord instances created at let-bindings carry SymbolBound provenance. Errors in dispatcher internals name binding-span + call-span.
- **`#[wat_value]` seal** — sonnet can't accidentally extend Value; structural confidence.
- **Errors-as-EDN** — panic surfaces are parseable EDN at IPC boundary.
- **Reflection primitives** — `:wat::holon::extract-classifier` + `Bind/{left,right}` (Stone 232.0a) are LIVE; the dispatcher consumes them.

**Use these tools.** When a macro-expansion error or runtime test failure surfaces, READ the error's ValueSnapshot + Provenance before guessing. The FM 2-bis probe already showed arc 233 firing in this exact domain (probe 3's `UnknownFunction(":myapp::Unhandled/Formattable-format", Span { ... })` named the missing verb + span without scaffolding).

## Implementation surface

This is PURE WAT-SIDE MACRO WORK. No Rust changes.

1. **`wat/holon/defprotocol.wat`** — NEW file. Defmacro `:wat::holon::defprotocol` per the canonical expansion template in `DESIGN-STONE-232.1.md` § "defprotocol expansion". Mirror `wat/holon/defrecord.wat` shape (defmacro form + quasiquote + per-N iteration via `:wat::core::map` + macro-time string concat for mangled-name suffix).

2. **`wat/holon/extend-type.wat`** — NEW file. Defmacro `:wat::holon::extend-type` per § "extend-type expansion". Per-method-body iteration via `:wat::core::map`; per-method mangled-name construction at macro-expand time via `keyword/to-string` + `string::concat` + `keyword/from-string`.

3. **`src/stdlib.rs`** — TWO new `WatSource` entries alongside the existing defrecord entry (around line 84). Same pattern: `path` + `source: include_str!(...)`.

4. **`tests/probe_arc232_stone1_defprotocol_macros.rs`** — NEW probe authored by sonnet. Mirrors `tests/probe_diagnostic_defprotocol_dispatch.rs` contracts BUT uses the macros (not manual composition). Three contracts minimum:
   - End-to-end: `defprotocol` + `extend-type` for two types + call → correct dispatch per-classifier
   - Open extension: extend-type AFTER defprotocol works (dispatch resolves at call time, not expand time)
   - Missing impl: type extended for one protocol-method but not another → observable `UnknownFunction`

   Initial state: parse errors (`:wat::holon::defprotocol` doesn't exist). Post-stone: 3/3 PASS.

## Design substrate (READ FIRST; MANDATORY)

1. **`docs/arc/2026/05/232-defprotocol-extend-type/DESIGN-STONE-232.1.md`** — sub-DESIGN with 8 locked decisions (D1-D8) + canonical expansion templates for both macros. **The expansion templates are non-negotiable**; they mirror the FM 2-bis probe verbatim.

2. **`tests/probe_diagnostic_defprotocol_dispatch.rs`** — FM 2-bis probe (commit `f38e120`; 3/3 PASS). The MANUAL composition the macros must replicate. Each macro-generated dispatcher matches this probe's dispatcher template with name substitutions; each macro-generated per-class defn matches this probe's per-class fns.

3. **`wat/holon/defrecord.wat`** — defmacro precedent (same family: splice/quasiquote + macro-time string building + per-N codegen via `:wat::core::map`).

4. **`tests/probe_diagnostic_macro_splice_from_let.rs`** — splice/quasiquote design substrate from arc 227 v3 (commit `c18fa6b`). Proves `~@(let [forms (map xs fn)] forms)` splice works.

5. **`tests/probe_diagnostic_bundle_result_compose.rs`** — Bundle composition design substrate (commit `72367f1`).

6. **`docs/arc/2026/05/232-defprotocol-extend-type/DESIGN.md`** — arc umbrella (forward-corrected 2026-05-23 night latest).

7. **`docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0a.md`** — predecessor SCORE; rank-up tools in action.

## What does NOT change

- **All Rust code** — HARD CUT. No `src/*.rs` modifications. Pure wat-side macros.
- **`Value` enum** — sealed by `#[wat_value]`; cannot accidentally extend even if attempted.
- **`HolonAST` enum** — unchanged (holon-rs frozen per STOP-4).
- **`defrecord` macro** — unchanged; defprotocol consumes its output.
- **`apply` primitive** — unchanged; defprotocol's dispatcher consumes it.
- **`extract-classifier` / `Bind/{left,right}`** — unchanged; defprotocol's dispatcher consumes them.
- **Arc 233 deliverables** — unchanged; regression guards stay GREEN.
- **holon-rs** — NOT touched.

## Out of scope (affirmative scope-bounding)

- **Default implementations** — defprotocol method declarations are signatures only; no default bodies. Deferred to v2 (Clojure shipped defaults later; per D4).
- **Multi-arg dispatch** — protocols dispatch on FIRST argument only (Clojure precedent per D3). Multimethods (arc 146/147) handle multi-arg.
- **`satisfies?` predicate** — runtime "does this type extend this protocol?" check. Out of v1 scope.
- **Protocol inheritance** — one protocol extending another. Out of v1 scope.
- **Built-in-type extension proof** — Stone 232.3 (extend `:wat::holon::Vector` or similar). Out of Stone 232.1.
- **defrecord accessor synthesis** — `:ns::Type/field-name` accessors generated by defrecord macro. NOT IN ARC 232 (per DESIGN.md table row 232.4); separate future stone.
- **Method-name validation at expand time** — D7 says extend-type MAY verify method-names match protocol declarations at expand time. If implementing this proves complex (requires macro-time registry), DEFER to v2; v1 accepts that typos surface at runtime as `UnknownFunction` (arc 233 names the missing verb).
- **holon-rs** — STOP-4.
- **Parallel API / aliases** — HARD CUT per D5.

## Verification flow

```bash
cargo build --release -p wat 2>&1 | tail -5                                                  # 0 errors
cargo test --release --test probe_arc232_stone1_defprotocol_macros 2>&1 | tail -5             # 3/3 PASS (the NEW probe)
cargo test --release --test probe_diagnostic_defprotocol_dispatch 2>&1 | tail -5              # 3/3 PASS (FM 2-bis regression guard)
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3                               # ≥ 827 passed; 0 failed
cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 | tail -3         # 7/7 PASS (Stone 232.0a guard)
cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 | tail -3                # 5/5 PASS
cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 | tail -3         # 5/5 PASS
cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 | tail -3                 # 3/3 PASS
cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 | tail -3                # 5/5 PASS
cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 | tail -3          # 8/8 PASS
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"                   # ≤ 54
git -C /home/watmin/work/holon/holon-rs/ status --short                                       # empty
```

## STOP triggers (REJECTION criteria; per FM 2-bis these do NOT defer)

- **STOP-1:** unexpected compile errors NOT tracing to the 2 new wat files + stdlib.rs entries
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** **180 min elapsed** (predicted 90-150; apply partial-state-grading per `feedback_partial_state_grading`)
- **STOP-4:** holon-rs touched (any Rust file outside stdlib.rs touched)
- **STOP-5:** clippy warnings above 54
- **STOP-6:** scope creep — default impls, multi-arg dispatch, satisfies?, accessor synthesis, built-in extension proof
- **STOP-7:** new probe `probe_arc232_stone1_defprotocol_macros` doesn't PASS 3/3 — the macros aren't working end-to-end
- **STOP-8:** FM 2-bis probe `probe_diagnostic_defprotocol_dispatch` regresses — the substrate composition broke
- **STOP-9:** any arc 233 regression guard regresses — the rank-up substrate must STAY working
- **STOP-10:** Stone 232.0a probe regresses — the reflection layer must STAY working

## Trap-door audit (per FM 2-bis BRIEF discipline)

- **NO Rust changes** — `src/stdlib.rs` only (two WatSource entries). No `src/runtime.rs`, no `src/check.rs`, no `src/macros.rs`. Pure wat-side macro work. If you find yourself in `src/runtime.rs`, STOP — the macros DO NOT need substrate extension.
- **NO new substrate primitives** — Stone 232.1 is pure wat-side macro sugar. The FM 2-bis probe PROVED the substrate is sufficient.
- **NO invented syntax** — use canonical inline `-> :T` (no brackets). Apply syntax is `(:wat::core::apply -> :T <head> <leading...> <spread-vec>)`. Verify against `tests/probe_diagnostic_dynamic_keyword_invocation.rs`.
- **NO made-up primitive names** — verify every primitive referenced (extract-classifier, string::concat, keyword/from-string, apply, Option/expect, etc.) via grep before authoring. The defrecord precedent uses the exact primitive names you need.
- **Mangled name convention** — `:NS::Type/Protocol-method` per D2 in sub-DESIGN. Single hyphen between protocol+method. Type and Protocol use full names (no abbreviations). Verify with FM 2-bis probe output: probe 3's error names `:myapp::Unhandled/Formattable-format` — that's the exact shape.
- **Dispatcher self-parameter type** — `[self <- :wat::holon::HolonAST]` per D8. Per-class impls also typed as HolonAST to avoid subtyping questions in v1.
- **Method-name validation deferral** — D7 says compile-time validation is preferred but MAY defer if complex. If you attempt + scope creeps, defer + document in SCORE; runtime UnknownFunction is honest.
- **Macro-time string building** — verify defrecord.wat's pattern: `:wat::core::keyword/to-string` (keyword → string for splicing into mangled name) + `:wat::core::string::concat` (string building) + `:wat::core::keyword/from-string` (string → keyword for emitting at name position).

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly per FM 12)
- HARD CUT — no aliases or parallel macro names
- Per `feedback_inscription_immutable`: SCORE is a NEW file (`SCORE-STONE-232.1.md`)
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification.
- The new probe `tests/probe_arc232_stone1_defprotocol_macros.rs` IS the success criterion. Author it; flip its initial-FAIL → 3/3 PASS via the macros.
- **This is a 2-macro bundle stone** — the calibration band (90-150 min) accounts for both macros + the new probe + integration testing.
- Trust the substrate-as-teacher cascade (FM 15) — `cargo test` reveals one issue at a time; iterate until green.

## Rank-up evidence — CAPTURE IN SCORE

Per the SCORE methodology in EXPECTATIONS, include a Rank-Up Evidence section. Capture any cases during iteration where arc 233 + Stone 232.0a tools saved time:

- Compile-error precision (Rust types if any Rust touched; macro-expansion errors at wat level)
- Diagnostic surface (ValueSnapshot rendering, Provenance traces, EDN parseability)
- Failure-engineering structural seal (cases where `#[wat_value]` or `extract-classifier`'s shape prevented errors structurally)

If iteration was clean without firing these tools (no failures to debug), the rank-up evidence is the ABSENCE OF NEED FOR SCAFFOLDING — also worth noting.

## Cross-references

- `docs/arc/2026/05/232-defprotocol-extend-type/DESIGN-STONE-232.1.md` — sub-DESIGN with 8 locked decisions + expansion templates
- `docs/arc/2026/05/232-defprotocol-extend-type/EXPECTATIONS-STONE-232.1.md` — paired 12-row scorecard
- `docs/arc/2026/05/232-defprotocol-extend-type/DESIGN.md` — arc 232 umbrella (forward-corrected)
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0a.md` — predecessor SCORE
- `tests/probe_diagnostic_defprotocol_dispatch.rs` — FM 2-bis probe (commit `f38e120`)
- `wat/holon/defrecord.wat` — defmacro precedent
- `tests/probe_diagnostic_macro_splice_from_let.rs` — splice substrate (`c18fa6b`)
- `tests/probe_diagnostic_bundle_result_compose.rs` — Bundle compose substrate (`72367f1`)
- `docs/arc/2026/05/233-substrate-errors-as-values/INSCRIPTION.md` — the rank-up arc
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs + scores; sonnet writes the macros
