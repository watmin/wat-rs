# EXPECTATIONS — Arc 233 Stone 233.3 — Errors-as-EDN extension

Mode A target: **11/11 PASS**. Every row binds to a specific verification command.

## Scorecard

| # | Row | Binding verification | Expected |
|---|---|---|---|
| 1 | Compile clean (wat) | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | Compile clean (wat-cli) | `cargo build --release -p wat-cli 2>&1 \| tail -5` | 0 errors |
| 3 | **233.3 probe FLIPS 0/5 → 5/5** | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -5` | `test result: ok. 5 passed; 0 failed` |
| 4 | Lib tests baseline | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 5 | Stone 233.2.e probe (regression guard) | `cargo test --release --test probe_stone_233_2_e_ast_derived_provenance 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 6 | Stone 233.2.l probe | `cargo test --release --test probe_stone_233_2_l_wat_value_seal 2>&1 \| tail -3` | `3 passed; 0 failed` |
| 7 | Stone 233.2.k probe | `cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 8 | Stone 233.2.j probe | `cargo test --release --test probe_stone_233_2_j_producer_migration 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 9 | **Stone 233.1 ValueSnapshot probes** (LOAD-BEARING — diagnostic-richness flows into EDN) | `cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 \| tail -3` | `8 passed; 0 failed` |
| 10 | Clippy no new warnings | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | ≤ 54 |
| 11 | holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ status --short` | empty output |

## Independent prediction

**Target runtime:** 60–120 min Mode A
**Upper bound:** 180 min (STOP-3) — per sub-DESIGN
**Confidence:** medium — mechanical sweep of 28 variants + 3 helpers + wat-cli wire-up. Pattern lineage from arc 211b is clear; risk is volume not novelty.

**Rationale:**
- Phase 1 (helper extraction in panic_hook): ~5 min
- Phase 2 (new module scaffold + 3 fn signatures): ~10 min
- Phase 3 (28 variant arms in runtime_error_to_edn): ~30-50 min
- Phase 4 (provenance_to_edn 4 arms + value_snapshot_to_edn): ~10 min
- Phase 5 (wat-cli integration): ~10-15 min
- Verification + SCORE writing: ~10 min

**Risks:**
- 28 variants × correct field-shape mapping: tedious but uniform; main risk is forgetting one
- HARD CUT on wire format may break a test asserting on Display text — sonnet identifies during baseline check
- Tag::ns construction may have quirks (validation, namespace translation per wat-edn arc 219)
- Nested error types (HashError, MacroError) — lazy fallback per BRIEF; sonnet documents in SCORE if a more structured encoding becomes load-bearing

## Honest deltas (planned)

- **WAT_ERROR_FORMAT=text fallback** — NOT shipped. HARD CUT replaces Display text with EDN on stderr at wat-cli exit. If a real consumer surfaces needing text format, separate follow-up arc adds the flag.
- **Cross-thread channel EDN** — receiver is in-process; not on a wire. Out of 233.3 scope.
- **Nested error types** (e.g., `crate::hash::HashError`) — rendered as `:error <Display string>` (lazy fallback). Future arc can deepen if structured access becomes load-bearing.

## Out-of-scope rows (REJECTED)

- WAT_ERROR_FORMAT=text fallback (HARD CUT)
- Cross-thread channel error EDN (receiver in-process)
- Display impl rewrite (Display stays for test/debug uses)
- holon-rs touched (STOP-4)
- Parallel API or deprecation aliases (HARD CUT)

## STOP triggers (from BRIEF — all REJECTION criteria)

- STOP-1: unexpected compile errors not tracing to new module
- STOP-2: baseline regress below 827
- STOP-3: 180 min elapsed
- STOP-4: holon-rs touched
- STOP-5: new clippy warning above 54
- STOP-6: scope creep (out-of-scope items above)
- STOP-7: probe still has failures (any of 5 contracts not PASS)
- STOP-8: existing arc 233 probes regress
- STOP-9: cascade exceeds time-box — apply partial-state-grading

## SCORE doc

`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.3.md` (new file per `feedback_inscription_immutable`).

SCORE expected to break down:
- Phase 1 (helper extraction): line count + sites updated
- Phase 2 (new module scaffold): line count
- Phase 3 (28 variant arms): per-variant key-name choices (especially tuple variants); aggregate line count
- Phase 4 (provenance + value_snapshot helpers): line count
- Phase 5 (wat-cli integration): exact site identified + line count
- Time breakdown by phase
- Calibration band actual vs predicted (60-120 target; 180 STOP)
- 11-row scorecard with verbatim verification command outputs
- Honest deltas (test-side ripple from HARD CUT; nested error fallback; etc.)

## What this unblocks

- **Stone 233.4 INSCRIPTION** — arc 233 closes once 233.3 ships
- **arc 217 Clojure-IPC bridge** — Clojure consumer parses `#wat.kernel/*` envelopes as `ex-info`-equivalent structured errors
- **wat-MCP horizon** — MCP tools consume structured errors instead of regex-matching text
- **Cross-language error propagation** — any wat-edn-aware consumer (Python via wat-edn parser, Rust via wat-edn crate, etc.) gets structured errors

## The IPC interop payoff

After this stone:
- Errors flowing across IPC boundaries are TAGGED EDN MAPS
- Consumers pattern-match on tag: `#wat.kernel/NotCallable` vs `#wat.kernel/TypeMismatch` vs ...
- Extract span coordinates programmatically (`(:span err)` in Clojure)
- Read provenance for trace reconstruction (`(:provenance (:got err))`)
- Forward errors across language boundaries without lossy text round-trips

Arc 233's complete thesis — "errors are remarkable" — reaches the WIRE level. Diagnostic richness (post-233.2.e) is now MACHINE-CONSUMABLE.

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/BRIEF-STONE-233.3.md` — paired BRIEF
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.3.md` — sub-DESIGN (commit `7436a3f`)
- `tests/probe_stone_233_3_runtime_error_edn.rs` — FM 2-bis probe (commit `186e880`)
- `src/panic_hook.rs` — arc 211b AssertionFailure precedent
- `crates/wat-edn/` — wat-edn substrate (arc 092)
- `feedback_partial_state_grading` — discipline if STOP-3 fires
