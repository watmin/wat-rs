# EXPECTATIONS — Arc 233 Stone 233.1 — ValueSnapshot sweep

Mode A target: **16/16 PASS**. Every row binds to a specific verification command. No row marked PASS without naming the verification.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | Lib tests baseline match (or grow with new lib tests) | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed; 1 ignored (count may grow if sonnet adds lib tests; must not regress) |
| 3 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 52 (baseline match) |
| 4 | `ValueSnapshot` type defined | `grep -c "pub struct ValueSnapshot" src/*.rs` | ≥ 1 |
| 5 | `Provenance` enum defined with `Unknown` variant | `grep -c "pub enum Provenance" src/*.rs` + `grep -c "Provenance::Unknown" src/*.rs` | both ≥ 1 |
| 6 | `ValueSnapshot::of` constructor exists | `grep -c "ValueSnapshot::of\|pub fn of" src/*.rs` | ≥ 1 |
| 7 | `NotCallable` field shape promoted | `grep -A 1 "NotCallable {" src/runtime.rs \| grep -c "got: ValueSnapshot"` | ≥ 1 |
| 8 | `TypeMismatch` field shape promoted | `grep -A 4 "TypeMismatch {" src/runtime.rs \| grep -c "got: ValueSnapshot"` | ≥ 1 |
| 9 | `BadCondition` field shape promoted | `grep -A 1 "BadCondition {" src/runtime.rs \| grep -c "got: ValueSnapshot"` | ≥ 1 |
| 10 | Old `&'static str` got fields purged for the 3 variants | `grep -A 2 "NotCallable {\|BadCondition {" src/runtime.rs \| grep -c "got: &'static str"` | 0 hits |
| 11 | **Probe 1 (literal-bound keyword) flips FAIL → PASS** | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors probe_1 -- --nocapture 2>&1 \| tail -3` | `test result: ok. 1 passed` |
| 12 | **Probe 2 (runtime-built keyword) flips FAIL → PASS** | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors probe_2 -- --nocapture 2>&1 \| tail -3` | `test result: ok. 1 passed` |
| 13 | New probe covering TypeMismatch runtime trigger | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors -- --nocapture 2>&1 \| grep -c "type_mismatch"` (or sonnet's chosen name) | ≥ 1 added (PASS, or honest delta in SCORE if runtime trigger genuinely unreachable) |
| 14 | New probe covering BadCondition runtime trigger | as above | ≥ 1 added (PASS, or honest delta in SCORE) |
| 15 | Full probe file green | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors -- --nocapture 2>&1 \| tail -3` | `test result: ok. ≥ 2 passed; 0 failed` |
| 16 | Holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction (calibration record)

**Target runtime:** 90-180 min Mode A
**Upper bound:** 180 min (STOP-3 fires)
**Confidence:** medium-high

**Rationale:**
- Type mint: ValueSnapshot + Provenance + impl Display — mechanical (~15 min)
- Sweep 3 RuntimeError variants — mechanical, ~20-30 sites total (~30-45 min)
- Display impl updates — mechanical (~15-30 min)
- Probe additions for TypeMismatch + BadCondition — exploratory; runtime-trigger shapes need discovery (~30-60 min — sonnet has more visibility than orchestrator)
- Existing tests asserting error message contents — likely 5-15 sites to update (~15-30 min)
- Verification cascade — standard (~10 min)

**Risks:**
- Some error-construction sites may not have the Value in scope (rare; audit + fall-back to synthetic ValueSnapshot per case)
- Existing tests that pattern-match on the OLD error Display format will break; need updates as part of the sweep (in scope)
- BadCondition runtime-trigger may genuinely be unreachable from wat-level code (all paths caught at check time); honest delta if so
- Recursion guard: `render_value` has SHOW_MAX_DEPTH = 8; deeply-nested Values render as "…". Acceptable.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows (REJECTED if attempted)

- Provenance variants beyond `Unknown` (Stone 233.2 territory)
- Errors-as-EDN extension (Stone 233.3)
- Other RuntimeError variants (`ArityMismatch`, `MalformedForm`, etc.)
- `CheckError::TypeMismatch` (separate enum, separate stone)
- Any holon-rs touch
- Any wat-edn touch
- Any aliases / deprecation shims (HARD CUT)
- "Stub", "future arc", "deferred to" language in SCORE

## Honesty deltas accepted

- TypeMismatch / BadCondition runtime triggers may not have wat-level reproducers; the substrate-level sweep still happens, probes are partial
- Some test fixtures may need to update their error-message-string assertions; that's part of the sweep
- Module placement (runtime.rs vs sibling diagnostic.rs vs elsewhere) — sonnet picks the honest home
- Display format exact spacing/punctuation can vary as long as the rendered content appears
- Existing tests with brittle error-string matching may have small adjustments; sonnet documents per-test in SCORE

## Honesty deltas NOT accepted (STOP triggers fire)

- Baseline test count regresses below 827 — STOP-2
- Either of probes 1 or 2 still FAILS post-stone — STOP-7
- Display output for affected variants no longer includes the rendered value — STOP-8
- Sonnet promotes additional RuntimeError variants "while we're here" — STOP-6
- Provenance gains variants beyond `Unknown` in this stone — STOP-6
- Sonnet edits holon-rs accidentally — STOP-4

## STOP triggers (cross-ref from BRIEF — all REJECTION criteria)

- **STOP-1:** unexpected compile errors
- **STOP-2:** test regression from 827 baseline
- **STOP-3:** 180 min elapsed
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning introduced
- **STOP-6:** scope creep (Provenance variants / other RuntimeError variants / CheckError)
- **STOP-7:** existing probes still FAIL
- **STOP-8:** Display output drops the rendered value

If any STOP fires: SCORE names it explicitly; ship nothing past the clean-stoppable state.

## SCORE doc

SCORE will live at `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.1.md`. Per `feedback_inscription_immutable`, that's a NEW file. Body cites each row's verification command + result + any honest delta.

## What this unblocks

- Stone 233.2 (Provenance tracking on Values) becomes the natural next step — fill in Provenance variants now that the snapshot field exists
- Stone 233.3 (Errors-as-EDN) can ship in parallel — the EDN serializer renders the ValueSnapshot field cleanly
- Arc 232 resume (defprotocol) — defprotocol method-body failures benefit immediately from richer diagnostics; dev cycle of arc 232.1+ becomes the consumer-side validation
- Every subsequent substrate-dev session pays less diagnostic-investigation tax
