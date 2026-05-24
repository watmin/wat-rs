# EXPECTATIONS — Arc 233 Stone 233.2.e — AST-derived provenance (Literal + SymbolBound)

Mode A target: **12/12 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **233.2.e probe FLIPS 1/5 → 5/5** | `cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 \| tail -5` | `test result: ok. 5 passed; 0 failed` |
| 3 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | Stone 233.2.l probe (seal regression guard) | `cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 \| tail -3` | `3 passed; 0 failed` |
| 5 | wat-macros tests (trybuild) | `cargo test --release -p wat-macros 2>&1 \| tail -3` | all pass |
| 6 | Stone 233.2.k probe (variant retirement regression guard) | `cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 7 | Stone 233.2.j probe (producer migration regression guard) | `cargo test --release --test probe_stone_233_2_j_producer_migration 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 8 | Stone 233.2.i probe | `cargo test --release --test probe_eval_signature_returns_tracked_value 2>&1 \| tail -3` | `3 passed; 0 failed` |
| 9 | Stone 233.2.h probe | `cargo test --release --test probe_tracked_value_mint_contract 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 10 | **Stone 233.1 ValueSnapshot probes** (LOAD-BEARING — diagnostic richness) | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 11 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 12 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction

**Target runtime:** 90–150 min Mode A
**Upper bound:** 180 min (STOP-3) — per Stone 233.2.e sub-DESIGN
**Confidence:** medium — focused phases (literal arms + BoundEntry + env.lookup + LetBinding shape + bind_let_binding + eval_let_tail). LetBinding shape change is the architectural touch.

**Rationale:**
- Phase 1 (6-7 literal arms): ~10 min
- Phase 2 (BoundEntry struct + EnvCell shape flip): ~10 min
- Phase 3 (env.lookup signature + 4 callers + head_span propagation): ~15-20 min
- Phase 4 (LetBinding shape + parse_let_binding span-extraction): ~20-30 min (parser care needed)
- Phase 5 (bind_let_binding propagation): ~10 min
- Phase 6 (eval_let_tail flip + callers): ~10-15 min
- Phase 7 (Display smoke + final verification): ~5 min
- Verification cascade + SCORE writing: ~15 min

**Risks:**
- LetBinding shape change ripples through parse_let_binding (parser); span-extraction needs care
- env.lookup head_span recursion through parent chain — must propagate via arg
- BoundEntry.value.value() clone — TrackedValue uses Arc internals so cheap; verify
- Probe 4 (destructure) requires LetBinding::Destructure carrying per-name spans + bind_let_binding propagating each; sonnet may need to add EnvironmentBuilder.bind_with_span or extend existing bind
- If cascade exceeds 180 min: STOP-3 fires; apply partial-state-grading per `feedback_partial_state_grading`

## Honest deltas (documented per sub-DESIGN)

- **recv/try-recv carrier-level provenance** — permanently lost (Decision 6). Indirect coverage via SymbolBound when let-bound is the common case; raw extraction stays Unknown. Original send-site span unrecoverable.
- **Chained provenance** (RuntimeBuilt → SymbolBound when let-bound producer result) — SymbolBound REPLACES stored RuntimeBuilt per Decision 2. Producer-context preserved in commits/SCORE; let-binding is the lexical scope.
- **Destructure source per-element provenance** — slot gets binding_span pointing at LHS pattern; tracing slot back to source tuple's element-span is out of scope.
- **List call-form provenance** — dispatch fn determines result provenance; not a "literal" per Decision 4.

## Out-of-scope rows (REJECTED)

- Chained provenance (RuntimeBuilt+SymbolBound composition)
- Carrier-level recv/try-recv restoration (Value::Tracked permanently retired)
- ValueSnapshot::of(&Value) sweep to of_tracked (incremental migration)
- Deeper destructure tracing
- List call-form provenance
- holon-rs touched (STOP-4)
- Deprecation aliases (HARD CUT)

## STOP triggers (from BRIEF — all REJECTION criteria)

- STOP-1: unexpected compile errors not tracing to cascade
- STOP-2: baseline regress below 827
- STOP-3: 180 min elapsed
- STOP-4: holon-rs touched
- STOP-5: new clippy warning above 54
- STOP-6: scope creep (out-of-scope items above)
- STOP-7: probe still has failures (any of 5 contracts not PASS)
- STOP-8: existing arc 233 probes regress
- STOP-9: cascade exceeds time-box — apply partial-state-grading

## SCORE doc

`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.e.md` (new file per `feedback_inscription_immutable`).

SCORE expected to break down:
- Phase 1 (literal-arm population): line count per arm
- Phase 2 (BoundEntry struct + EnvCell shape): line count
- Phase 3 (env.lookup signature flip + caller cascade): line count per caller
- Phase 4 (LetBinding shape change + parse_let_binding): line count
- Phase 5 (bind_let_binding propagation): line count
- Phase 6 (eval_let_tail flip): line count
- Phase 7 (Display smoke): verification only, no code change expected
- Time breakdown by phase
- Calibration band actual vs predicted (90-150 target; 180 STOP)
- 12-row scorecard with verbatim verification command outputs
- Honest deltas (recv/try-recv permanent loss; chained provenance future work; etc.)

## What this unblocks

- **arc 233 Stone 233.3** — Errors-as-EDN extension. Provenance now meaningful for EDN serialization; AssertionPayload pattern from arc 211b generalizes to all RuntimeError variants.
- **arc 233 Stone 233.4** — INSCRIPTION. arc 233 closes; the j→k→l→e chain delivers the diagnostic-richness substrate promised when arc 233 opened.
- **arc 232 defprotocol** — resumes on the diagnostic-rich + sealed substrate; substrate work no longer needs to defer to "errors will get better when arc 233 ships."
- **MTG horizon, Truth Engine, trading-lab v2, wat-MCP** — all downstream domains consume the substrate with full provenance for free.

## The diagnostic-richness payoff

After this stone:
- Literal `42` in source → carries source coordinates (line + col + file)
- `(let [x 42] (some-fn x))` — when `x` flows into a TypeMismatch error, the error names binding_span (let line) + head_span (some-fn call site)
- Producer-built values keep RuntimeBuilt (already populated per 233.2.j/k)
- Only escape-context values keep Unknown

The user/LLM debugging an error sees: WHERE the value was defined, WHERE it was used, and WHAT it was. The original thesis of arc 233 — "errors are remarkable" — reaches empirical delivery.

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/BRIEF-STONE-233.2.e.md` — paired BRIEF
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.e.md` — sub-DESIGN (commit `12bb8b1`)
- `tests/probe_stone_233_2_e_ast_derived_provenance.rs` — FM 2-bis probe (commit `97fa595`)
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.k.md` — Environment storage precedent
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.l.md` — sealed substrate this builds on
- `scratch/FAILURE-ENGINEERING.md` — the doctrine driving the chain
- `feedback_partial_state_grading` — discipline if STOP-3 fires
