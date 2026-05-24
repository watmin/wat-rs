# EXPECTATIONS — Arc 233 Stone 233.2.k — Value::Tracked variant retirement + Environment stores TrackedValue

Mode A target: **12/12 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **233.2.k probe FLIPS 0/5 → 5/5** | `cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 \| tail -5` | `test result: ok. 5 passed; 0 failed` |
| 3 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | Stone 233.2.j probe still passes (exemption mechanism removed) | `cargo test --release --test probe_stone_233_2_j_producer_migration 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 5 | Stone 233.2.i eval signature probe still passes | `cargo test --release --test probe_eval_signature_returns_tracked_value 2>&1 \| tail -3` | `3 passed; 0 failed` |
| 6 | Stone 233.2.h TrackedValue mint probe still passes | `cargo test --release --test probe_tracked_value_mint_contract 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | Stone 233.2.d substrate-symmetry probe still passes | `cargo test --release --test probe_substrate_symmetry_list_span_threading 2>&1 \| tail -3` | `1 passed; 0 failed` |
| 8 | **Stone 233.1 ValueSnapshot diagnostic probes** (LOAD-BEARING — let-binding probes 6/7/8 MUST stay green via Option A) | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 9 | Stone 232.0 dynamic-keyword probes still pass | `cargo test --release --test probe_diagnostic_dynamic_keyword_invocation 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 10 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 11 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |
| 12 | **probe_value_tracked_transparency.rs DELETED** | `ls tests/probe_value_tracked_transparency.rs 2>&1 \| grep -c "No such"` | 1 |

## Independent prediction

**Target runtime:** 60–120 min Mode A
**Upper bound:** 180 min (STOP-3) — per Stone 233.2.k sub-DESIGN
**Confidence:** medium — cascade contained (~50-100 sites vs 383 in 233.2.j); variant retirement is mostly mechanical deletion + sweep; Environment storage flip is the architectural touch (small: 6 lookup callers, 2 internal HashMap declarations, builder pattern)

**Rationale:**
- Environment storage type flip + builder + lookup signature: ~10 min
- 6 lookup caller updates: ~5 min
- bind_let_binding simplification: ~5 min
- Variant + 3 helpers delete: ~5 min
- ~6-8 dead match arm cleanup: ~10 min
- 19 .inner() call site sweep: ~15-20 min
- 26 .into_tracked() call site sweep: ~15-20 min (mechanical replace with TrackedValue::from)
- DELETE probe_value_tracked_transparency.rs: ~1 min
- Remove probe-3-exempt mechanism in probe_stone_233_2_j: ~3 min
- Verification cascade + SCORE writing: ~15 min

**Risks:**
- Borrow checker friction on `.into_tracked()` → `TrackedValue::from()` if surrounding scope expected an owned value (some sites may need `.clone()` adjustments)
- `.inner()` sites may have non-trivial usage patterns; strip vs replace decision per site
- Watch for hidden Value::Tracked references in `*.rs` doc comments embedded in source (probe 1's heuristic may flag them; sonnet should clean those too)
- If cascade exceeds 180 min: STOP-3 fires; apply partial-state-grading per `feedback_partial_state_grading`

## Honest delta zones (planned)

- **Destructure slot provenance** — each tuple slot gets `Provenance::Unknown` via `TrackedValue::from(elem)`. Not a regression from current behavior (current destructure path strips provenance via `.value_owned()`). Arc 233 Stone 233.2.e may revisit if per-slot provenance becomes load-bearing.
- **No other planned regressions** — Option A's structural fix means probes 6/7/8 stay GREEN through the structural mechanism; no #[ignore] markers needed.

## Out-of-scope rows (REJECTED)

- Stone 233.2.l proc-macro structural seal (next stone)
- Stone 233.2.e AST-derived provenance for destructure / recv / try-recv
- runtime_def_values HashMap storage (different concern)
- holon-rs touched (STOP-4)
- Deprecation aliases or parallel APIs (HARD CUT)

## STOP triggers (from BRIEF — all REJECTION criteria)

- STOP-1: unexpected compile errors not tracing to cascade
- STOP-2: baseline regress below 827
- STOP-3: 180 min elapsed
- STOP-4: holon-rs touched
- STOP-5: new clippy warning above 54
- STOP-6: scope creep (proc-macro, AST-derived, runtime_def_values)
- STOP-7: probe still has failures (any of 5 contracts not PASS)
- STOP-8: existing arc 233 probes regress (ESPECIALLY 233.1 probes 6/7/8)
- STOP-9: cascade exceeds time-box — apply partial-state-grading discipline

## SCORE doc

`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.k.md` (new file per `feedback_inscription_immutable`).

SCORE expected to break down:
- Environment storage type flip + caller cascade (file diff, line count)
- Variant + helper deletions (line count saved)
- Dead match arm removals (per-file count)
- .inner() + .into_tracked() sweeps (per-file count)
- probe_value_tracked_transparency.rs deletion (file removed)
- probe-3-exempt mechanism removal (lines deleted)
- Time breakdown by phase
- Calibration band actual vs predicted (60-120 target; 180 STOP)
- 12-row scorecard with verbatim verification command outputs

## What this unblocks

- **Stone 233.2.l** — `#[wat_value]` proc-macro structural seal. Can now apply (Value enum no longer contains Tracked variant). The compile-time meta-class prevention. ✅✅✅.
- **arc216 stone1 7 probes** (task #496) — auto-resolve. The trap-door class instance is structurally absent.
- **Stone 233.2.e** — AST-derived provenance on the fully-sealed substrate (restores destructure / recv / try-recv provenance via different mechanism that doesn't depend on Value-side carriers).

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/BRIEF-STONE-233.2.k.md` — paired BRIEF
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.k.md` — sub-DESIGN (commit `f830de8`)
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.l.md` — next stone (commit `57eced2`)
- `tests/probe_stone_233_2_k_variant_retired.rs` — FM 2-bis probe (commit `f43c577`)
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.j.md` — establishes the Phase 5 exemption this stone dissolves
- `scratch/FAILURE-ENGINEERING.md` — the doctrine driving Option A
- `feedback_partial_state_grading` — discipline if STOP-3 fires
