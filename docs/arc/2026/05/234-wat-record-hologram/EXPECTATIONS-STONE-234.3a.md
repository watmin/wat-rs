# EXPECTATIONS — Arc 234 Stone 234.3a — read verbs

Mode A target: **11/11 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **New probe FLIPS 6/6 FAIL → 6/6 PASS** (LOAD-BEARING) | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -5` | `test result: ok. 6 passed; 0 failed` |
| 3 | Stone 234.2c regression guard | `cargo test --release --test probe_arc234_stone2c_accessor_class_safety 2>&1 \| tail -5` | `5 passed; 0 failed` |
| 4 | Stone 234.2b regression guard | `cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 5 | Stone 234.5 regression guard | `cargo test --release --test probe_arc234_stone5_holon_auto_dispatch 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 6 | Stone 234.2a regression guard | `cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 \| tail -5` | `6 passed; 0 failed` |
| 7 | Stone 234.1 regression guard | `cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 \| tail -5` | `7 passed; 0 failed` |
| 8 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 9 | Stone 232.0a regression guard | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 11 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction

**Target runtime:** 30–60 min Mode A
**Upper bound:** 75 min (STOP-3 hard cap)
**Confidence:** medium-high — small focused stone; precedented patterns (Value variant match, HolonAST traversal, HashMap construction).

**Rationale:**
- Runtime: `eval_record_q` (~15 lines) + `eval_record_to_map` (~40-60 lines) + 2 dispatch arms = ~70-85 lines
- check.rs: 2 TypeScheme registrations OR custom handler = ~15-30 lines
- Probe committed pre-spawn: no probe authoring time
- Compile cycles: 1-3 rounds expected (HolonAST leaf extraction is the novel piece)

**Calibration precedents:**
- Stone 234.2a-CORRECTION (~25 min): single custom handler
- Stone 234.2a (~58 min): substrate primitives + 2 TypeSchemes (similar surface)
- Stone 234.5 (~75 min): centralized helper + 5 verb threading (larger scope)
- Stone 234.3a estimate: ~40-50 min predicted; band's middle

**Risks:**
- **HolonAST leaf extraction for field-names** — the holon_form structure encodes field-names as nested Atoms; sonnet investigates the exact extraction path (likely via `Bundle/children` → iterate → `Bind/left` → unwrap Atom → extract String leaf). Per arc 225 + arc 230 substrate.
- **HashMap<:keyword, :T> return polymorphism** — may need custom handler if standard TypeScheme polymorphic-T doesn't propagate through HashMap's typed K/V params. Stone 234.2a-CORRECTION's `infer_record_of` is the precedent.
- **HashMap construction with keyword keys** — `Value::wat__core__keyword(Arc::new(format!(":{}", name)))` per Stone 216 storage refactor; verify the key format (with or without leading colon).

## Rank-up demonstration

Per `project_party_comp_inquisitor_shadowdancer`: orchestrator marked the targets (sub-DESIGN + probe + initial-state verification); sonnet strikes in the cascade.

For Stone 234.3a:
- Two focused primitives; well-precedented patterns
- HolonAST walking is the novel piece — substrate-as-teacher should surface any extraction mismatch cleanly
- The split-out from umbrella 234.3 (per stepping-stone discipline) keeps this stone digestible

Capture in SCORE:
- Which existing HolonAST extraction helper was reused for field-name extraction
- HashMap construction pattern + key format
- Did the polymorphic-T return for `record->map` require custom-handler treatment

## Out-of-scope rows (REJECTED)

- `:wat::core::assoc` polymorphic record arm (Stone 234.3b)
- Keyword-as-accessor fall-through (Stone 234.3c)
- `:wat::core::record->holon` (dropped per scope — synonym with `:wat::holon::to-holon`)
- Per-class accessor or predicate variants
- Changes to `wat/Record.wat` (the macro is correct as-shipped)
- Changes to existing probes
- holon-rs touched (STOP-4)

## STOP triggers (from BRIEF — all REJECTION criteria)

- **STOP-1** — unexpected compile errors not tracing to new primitives
- **STOP-2** — lib baseline < 827
- **STOP-3** — 75 min elapsed
- **STOP-4** — holon-rs touched
- **STOP-5** — Rust changes outside runtime.rs + check.rs
- **STOP-6** — scope creep (assoc, keyword-as-accessor, record->holon, per-class variants)
- **STOP-7** — probe doesn't flip 6/6 PASS
- **STOP-8** — 234.2b regression guard regresses
- **STOP-9** — any prior arc 234 regression guard regresses
- **STOP-10** — clippy > 54

Each STOP is REJECTION. None is permission-to-defer. If hit: report; surface; do NOT ship workaround.

## SCORE doc

`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.3a.md` (NEW file per `feedback_inscription_immutable`).

SCORE expected to include:
- 11-row scorecard with verbatim verification command outputs
- Implementation surface: eval fn line counts; HolonAST extraction pattern; HashMap construction
- Cascade depth: compile rounds + iteration cycles
- Time breakdown
- Calibration delta (30-60 target; 75 STOP)
- Trap-door audit (T1-T8) outcomes
- Honest deltas if any surface
- Rank-up evidence — predecessor pattern reuse

## What this completes

When 234.3a ships:
- The polymorphic read surface for records is in place
- `record?` and `record->map` are first-class verbs alongside the per-class macro-generated ones
- Stone 234.3b (assoc record arm) is unblocked — can reuse the field-name extraction
- Stone 234.4 (hash-destructure) gains the field-name extraction precedent
- Stone 234.6 (migration sweep) gets the v1 polymorphic read surface

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.3a.md` — paired BRIEF
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.3a.md` — sub-DESIGN (12 locked decisions)
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella
- `tests/probe_arc234_stone3a_record_read_verbs.rs` — FM 2-bis probe (6/6 FAIL verified)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.5.md` — VSA integration precedent (holon_form access)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a-CORRECTION.md` — custom-handler precedent
- `wat-rs/docs/WAT-CHEATSHEET.md` § 1 — colon rule (symbol-quote framing)
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs + scores; sonnet writes
- `feedback_iterative_complexity.md` — discipline behind the 234.3a/b/c split
