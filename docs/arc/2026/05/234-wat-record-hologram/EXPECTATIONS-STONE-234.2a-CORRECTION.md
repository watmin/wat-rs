# EXPECTATIONS — Arc 234 Stone 234.2a — forward-correction

Mode A target: **11/11 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **Stone 234.2b probe FLIPS 5/6 PASS → 6/6 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 \| tail -5` | `test result: ok. 6 passed; 0 failed` |
| 3 | **Stone 234.2a regression guard stays GREEN** (SUBSIDIARY LOAD-BEARING) | `cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 \| tail -5` | `test result: ok. 6 passed; 0 failed` |
| 4 | Stone 234.1.5 regression guard | `cargo test --release --test probe_arc234_stone15_namespace_promotion 2>&1 \| tail -5` | `5 passed; 0 failed` |
| 5 | Stone 234.1 regression guard | `cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 \| tail -5` | `7 passed; 0 failed` |
| 6 | Stone 234.0 regression guard | `cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 \| tail -5` | `8 passed; 0 failed` |
| 7 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 8 | Stone 232.0a regression guard | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 9 | `:wat::holon::defrecord` regression guard | `cargo test --release --test probe_arc227_stone2_defrecord 2>&1 \| tail -5` | `35 passed; 0 failed` |
| 10 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 11 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction

**Target runtime:** 20–35 min Mode A
**Upper bound:** 40 min (STOP-3 hard cap)
**Confidence:** high — focused single-file change; `infer_arithmetic` precedent is well-established; the load-bearing test is ONE probe flipping.

**Rationale:**
- check.rs change: ~30-50 lines (custom handler `infer_record_of` + dispatch hook)
- Read precedent + dispatcher orientation: ~10 min
- Author + compile cycles: ~10-15 min (1-2 rounds expected)
- Full scorecard run: ~3-5 min
- SCORE writing: ~5-10 min

**Calibration precedents:**
- Stone 232.0a (typed-entities reflection): ~52 min — but that was 3 verbs + dispatch + 2 check.rs special-cases. The correction is 1 verb + dispatch.
- Stone 234.0 (polymorphic type primitive): ~38 min ZERO iteration — single eval + dispatch + single TypeScheme. Similar surface.
- The correction estimate: ~25-30 min predicted; band's middle.

**Risks:**

- **T1 dispatch-hook location** — finding the exact spot where `:wat::core::+` routes to `infer_arithmetic` might take 5-10 min of orientation in the `infer_list` (or similar) primary dispatcher.
- **T2 coexistence** — whether to leave the existing `env.register(":wat::Record::of", TypeScheme)` registration in place or remove it depends on the dispatcher's order-of-resolution. Sonnet investigates + chooses.
- **Vec-shape recognition for arg #2** — the custom handler needs to peer into the arg's call-shape to bypass uniform-T. Pattern is established (other custom handlers do similar inspection); should compose cleanly.

## Rank-up demonstration

Per `project_party_comp_inquisitor_shadowdancer`: the orchestrator marked the substrate gap (sub-DESIGN with honest "we authored it incorrectly" framing); sonnet strikes-to-kill (focused check.rs change).

For Stone 234.2a forward-correction specifically:
- The correction is SHALLOW (one file, one new fn, one dispatch entry)
- The probe is already on disk + already FAILS in a specific deterministic way (TypeMismatch on vec param positions)
- The fix unblocks 234.2b's full PASS without touching anything else
- The `infer_arithmetic` precedent SHORTENS authoring significantly

Capture in SCORE:
- How `infer_arithmetic` precedent SHORTENED the work (dispatch-hook pattern; signature; inspection style)
- The dispatch-hook investigation outcome (where in check.rs the routing happens)
- Whether the existing TypeScheme registration stayed or was removed
- Any honest deltas (was T1 or T2 the actual challenge; was the predicted ~30 min accurate)

## Out-of-scope rows (REJECTED)

- New probe authoring (the 234.2b probe is the load-bearing test; D4 of correction sub-DESIGN)
- Changes to `eval_record_of` runtime (STOP-5; D3)
- Changes to Value variant (STOP-5)
- Changes to other primitives' TypeSchemes (STOP-6)
- Changes to `wat/Record.wat` (STOP-5)
- Changes to `src/stdlib.rs` (STOP-5)
- Changes to SCORE-STONE-234.2a.md (INSCRIPTION-immutable; D7)
- Changes to SCORE-STONE-234.2b.md (sibling; sonnet's earlier authoring stays)
- holon-rs touched (STOP-4)
- HARD CUT discipline considerations (pure substrate correction; no surface change)

## STOP triggers (from BRIEF — all REJECTION criteria)

- **STOP-1** — unexpected compile errors not tracing to the check.rs change
- **STOP-2** — lib tests baseline regresses below 827
- **STOP-3** — 40 min elapsed (small change; tight cap)
- **STOP-4** — `holon-rs` touched
- **STOP-5** — Rust changes outside `src/check.rs`
- **STOP-6** — scope creep: changes to runtime, Value variant, other primitives, the macro
- **STOP-7** — 234.2b probe 5 does not flip to PASS (probe must reach 6/6)
- **STOP-8** — Stone 234.2a probe regresses (6/6 PASS must stay)
- **STOP-9** — any prior arc 234 regression guard regresses (234.0, 234.1, 234.1.5)
- **STOP-10** — clippy warnings exceed 54

Each STOP is REJECTION. None is permission-to-defer. If hit: report; surface; do NOT ship workaround.

## SCORE doc

`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a-CORRECTION.md` (NEW file per `feedback_inscription_immutable`).

SCORE expected to include:
- 11-row scorecard with verbatim verification command outputs
- Implementation surface: check.rs added fn + dispatch hook line counts; existing TypeScheme registration disposition (kept / removed)
- Dispatch-hook investigation finding (where the dispatcher routes primitives to custom handlers)
- Cascade depth: compile rounds + any iteration cycles
- Time breakdown
- Calibration delta (actual vs predicted ~20-35 min)
- Trap-door audit (T1-T8 with concrete outcomes; predicted-vs-actual)
- Honest deltas if any surface
- Rank-up evidence — how `infer_arithmetic` precedent shortened authoring

## What this completes

When the correction ships + atomic commit lands:
- Stone 234.2b achieves full 6/6 PASS on its probe
- Stone 234.2a's TypeScheme matches the umbrella DESIGN intent + the runtime's behavior
- The umbrella DESIGN gets a brief note pointing to the correction commit (per D6)
- Three artifacts are now consistent (DESIGN.md intent ↔ check.rs TypeScheme ↔ runtime behavior)
- The arc 234 chain proceeds clean to Stone 234.2c (runtime class-safety check) or Stone 234.3 (polymorphic record-y verbs)

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.2a-CORRECTION.md` — paired BRIEF
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.2a-CORRECTION.md` — sub-DESIGN with 8 locked decisions
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella (line 19: heterogeneous intent)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a.md` — predecessor SCORE (stays immutable)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` — sibling SCORE (ships in same atomic commit)
- `tests/probe_arc234_stone2b_defrecord_macro.rs` — load-bearing test (probe 5 flips)
- `tests/probe_arc234_stone2a_record_primitives.rs` — regression guard (stays green)
- `src/check.rs` line 10885-10980 — `infer_arithmetic` precedent
- `src/check.rs` line 16989-17001 — existing `:wat::Record::of` TypeScheme registration (target)
- `src/runtime.rs::eval_record_of` — runtime is already correct
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs + scores; sonnet writes
- `feedback_inscription_immutable.md` — predecessor SCORE stays unchanged
- `feedback_no_broken_commits.md` — atomic commit with Stone 234.2b
