# EXPECTATIONS — Arc 233 Stone 233.2.h — mint `TrackedValue` struct + adapter

Mode A target: **9/9 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **TrackedValue mint probe FLIPS 0/6 → 6/6** | `cargo test --release --test probe_tracked_value_mint_contract 2>&1 \| tail -5` | `test result: ok. 6 passed; 0 failed` |
| 3 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | Substrate-symmetry probe still passes | `cargo test --release --test probe_substrate_symmetry_list_span_threading 2>&1 \| tail -3` | `1 passed; 0 failed` |
| 5 | Stone 233.1 probes still pass | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 6 | Stone 233.2.a transparency tests still pass | `cargo test --release --test probe_value_tracked_transparency 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 7 | Stone 232.0 dynamic-keyword probes still pass | `cargo test --release --test probe_diagnostic_dynamic_keyword_invocation 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 8 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 9 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction

**Target runtime:** 15-30 min Mode A
**Upper bound:** 45 min (STOP-3)
**Confidence:** high — pure mint stone; mirrors Stone 233.2.a precedent shape; no behavioral change

**Rationale:**
- Struct + 4 methods + From impl: ~5-10 min
- Re-export wiring: ~3 min
- Compile iteration: ~5 min
- Verification cascade: ~5 min
- SCORE writing: ~5-10 min

**Risks:**
- Re-export path may need wiring update in lib.rs or runtime.rs's pub-use block — minor
- If existing Provenance re-export pattern is unusual, follow it exactly (don't invent a new pattern)
- Clippy may flag the new struct for some style issue (e.g., missing docs) — `#[doc = "..."]` per the template above; aim for clean baseline

## Out-of-scope rows (REJECTED)

- Eq/PartialEq/Hash derive (forced explicit comparison)
- Display impl (defer)
- Touching Value enum or Value::Tracked variant
- Touching eval signature (Stone 233.2.i)
- Touching producers (Stone 233.2.j)
- holon-rs touched (STOP-4)

## STOP triggers (from BRIEF — all REJECTION criteria)

- STOP-1: unexpected compile errors not tracing to the mint
- STOP-2: baseline lib tests regress below 827
- STOP-3: 45 min elapsed
- STOP-4: holon-rs touched
- STOP-5: new clippy warning above 54
- STOP-6: scope creep (touching Value::Tracked, eval, producers, or existing code beyond mint + re-export)
- STOP-7: probe still has failures
- STOP-8: existing arc 233 probes regress

If any STOP fires: ship NOTHING beyond clean-stoppable state; surface as honest delta in SCORE.

## SCORE doc

`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.h.md` (new file per `feedback_inscription_immutable`).

## What this unblocks

- **Stone 233.2.i** — eval signature flip can begin (TrackedValue exists as the target type)
- **Future producer ergonomics** — Stone 233.2.j has the type to migrate producers TO
- **Final structural enforcement** — Stones 233.2.h+i+j+k together eliminate the Tracked-unwrap class

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/BRIEF-STONE-233.2.h.md` — paired BRIEF
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.g.md` — sub-DESIGN (Shape A verdict)
- `tests/probe_tracked_value_mint_contract.rs` — FM 2-bis probe (commit `0f4e318`)
