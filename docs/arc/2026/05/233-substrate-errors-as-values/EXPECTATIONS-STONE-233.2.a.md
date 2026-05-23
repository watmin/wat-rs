# EXPECTATIONS — Arc 233 Stone 233.2.a — Provenance + Value::Tracked scaffolding

Mode A target: **16/16 PASS**. Every row binds to a specific verification command. No row marked PASS without naming the verification.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | Lib tests baseline maintained | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed; 1 ignored (no regression from 233.1's 827) |
| 3 | Stone 233.1 probes still pass | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 4 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 52 (baseline match) |
| 5 | Provenance::Literal variant exists | `grep -c "Provenance::Literal" src/runtime.rs` | ≥ 1 |
| 6 | Provenance::SymbolBound variant exists | `grep -c "Provenance::SymbolBound" src/runtime.rs` | ≥ 1 |
| 7 | Provenance::RuntimeBuilt variant exists | `grep -c "Provenance::RuntimeBuilt" src/runtime.rs` | ≥ 1 |
| 8 | Value::Tracked variant added | `grep -c "Tracked {" src/runtime.rs` + `grep -c "Value::Tracked" src/runtime.rs` | both ≥ 1 |
| 9 | Value::inner() helper exists | `grep -c "fn inner.*&Value\|fn inner.*Value" src/runtime.rs` | ≥ 1 |
| 10 | Value::provenance() helper exists | `grep -c "fn provenance" src/runtime.rs` | ≥ 1 |
| 11 | **Transparency contract 1 — Display unwraps** | `cargo test --release --test probe_value_tracked_transparency contract_1 -- --nocapture` | PASS |
| 12 | **Transparency contract 2 — Eq compares inner** | `cargo test --release --test probe_value_tracked_transparency contract_2 -- --nocapture` | PASS |
| 13 | **Transparency contract 3 — Hash unwraps (HashMap correctness)** | `cargo test --release --test probe_value_tracked_transparency contract_3 -- --nocapture` | PASS |
| 14 | **Transparency contracts 4-8** — Clone preserves; inner() recurses; provenance() outermost; ValueSnapshot extracts; bare has Unknown | `cargo test --release --test probe_value_tracked_transparency -- --nocapture 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 15 | Sub-DESIGN Shape C respected — no TrackedValue struct minted | `grep -c "struct TrackedValue\|pub struct TrackedValue" src/` | 0 |
| 16 | Holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction (calibration record)

**Target runtime:** 90-150 min Mode A
**Upper bound:** 180 min (STOP-3 fires)
**Confidence:** medium

**Rationale:**
- Provenance enum extension: ~5 min (3 new variants)
- Value::Tracked variant addition: ~10 min (the enum edit)
- Eq impl transparency: ~30-60 min (existing impl likely needs restructuring to use `inner()`)
- Hash impl transparency: ~30 min (similar shape to Eq)
- render_value transparency: ~10 min (add `v.inner()` unwrap at top)
- Helpers (`inner`, `provenance`): ~15 min
- Transparency test file: ~20-30 min (8 tests)
- Variant-exhaustiveness sweep across Value match arms: ~20-40 min (could be many sites; sonnet finds them via cargo build)
- ValueSnapshot::of update: ~5 min

**Risks:**
- Eq/Hash impls may be more entangled than expected (arc 216 Stone 216.5a shaped them carefully). Sonnet may need to refactor the existing impls significantly to introduce Tracked transparency. Honest delta if this requires deeper restructuring than predicted.
- Variant-exhaustiveness sweep could surface many match-arm sites — each needs a Tracked arm that delegates to inner. Mechanical but could be 20-50 sites.
- HolonRepresentable concerns: confirmed NOT directly affected (trait is on Rust types, not Value). Honest delta if a Value→HolonAST conversion path is affected.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows (REJECTED if attempted)

- Tagging actual producers (233.2.b territory)
- Rendering Provenance in error messages (233.2.b will extend Display when real provenance flows)
- HolonRepresentable trait modifications (Tracked doesn't affect Rust-type serialization)
- Cross-boundary provenance transport
- Performance tuning / interning
- Any holon-rs touch
- Any wat-edn touch
- `pub struct TrackedValue` (Shape A — rejected at sub-DESIGN)
- Per-variant Option<Provenance> fields (Shape B — rejected at sub-DESIGN)
- Any aliases / deprecation shims (HARD CUT)
- "Stub", "future arc", "deferred to" language in SCORE

## Honesty deltas accepted

- Match arm count for Tracked-exhaustiveness sweep may grow larger than predicted — mechanical but more sites
- Eq/Hash impl restructuring may require splitting the existing impl into helpers
- Module placement of transparency test file (`tests/probe_value_tracked_transparency.rs` vs another name) — sonnet picks
- Specific names of transparency-related helper functions sonnet creates internally

## Honesty deltas NOT accepted (STOP triggers fire)

- Baseline lib test count regresses below 827 — STOP-2
- ANY transparency contract fails — STOP-7
- Tracked introduces a HashMap key bug (two Values that should hash equally don't) — STOP-7 (contract 3 specifically)
- Sonnet picks Shape A or Shape B instead of Shape C — STOP-9
- Sonnet tags producers "while we're here" — STOP-6 (that's 233.2.b)
- Sonnet edits holon-rs — STOP-4
- Existing Stone 233.1 probes regress — STOP-2 (5 of them must still pass)

## STOP triggers (cross-ref from BRIEF — all REJECTION criteria)

- **STOP-1:** unexpected compile errors beyond variant-exhaustiveness sweep
- **STOP-2:** test regression below baseline (827 lib + 5 stone-233.1 probes)
- **STOP-3:** 180 min elapsed
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning
- **STOP-6:** scope creep (producer tagging / Display extension / HolonRepresentable)
- **STOP-7:** transparency contract fails
- **STOP-8:** Value-construction sites stop working (NotCallable / TypeMismatch construction broken)
- **STOP-9:** Implementation shape deviation from Shape C

If any STOP fires: SCORE names it explicitly; ship nothing past the clean-stoppable state.

## SCORE doc

SCORE will live at `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.a.md`. Per `feedback_inscription_immutable`, that's a NEW file. Body cites each row's verification command + result + any honest delta.

## What this unblocks

- **233.2.b** — `keyword/from-string` becomes the first producer to tag returned Values with RuntimeBuilt provenance. Probe demonstrates runtime-built case teaches.
- **233.2.c** — sweep additional producers using the established 233.2.b pattern.
- **233.2.d** — AST-derived provenance for let-bindings + literals.

233.2 closes when 233.2.a-d ship + the umbrella INSCRIPTION (Stone 233.4) lands.
