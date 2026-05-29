# EXPECTATIONS — Stone 241.1 — mint canonical `parse_argspec_triples` at `src/argspec/`

Independent scorecard for orchestrator-side verification after sonnet returns. Each row is a fact to confirm via an explicit command; orchestrator re-runs locally and writes the verbatim result into `SCORE-STONE-241.1.md`.

## Scorecard (16 rows)

| Row | Claim | Verification command | Expected result |
|-----|---|---|---|
| 1 | Probe contract 1 PASS (empty argspec, no ret) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_01_empty_argspec_no_ret_type_expected` | 1 passed; 0 failed |
| 2 | Probe contract 2 PASS (single fixed param, no ret) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_02_single_fixed_param_no_ret` | 1 passed; 0 failed |
| 3 | Probe contract 3 PASS (multiple fixed + ret) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_03_multiple_fixed_params_with_ret` | 1 passed; 0 failed |
| 4 | Probe contract 4 PASS (ret-only signature) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_04_ret_only_signature` | 1 passed; 0 failed |
| 5 | Probe contract 5 PASS (non-Symbol at name slot) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_05_non_symbol_at_name_slot` | 1 passed; 0 failed |
| 6 | Probe contract 6 PASS (missing `<-` arrow) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_06_missing_arrow_token` | 1 passed; 0 failed |
| 7 | Probe contract 7 PASS (non-Keyword at type slot) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_07_non_keyword_at_type_slot` | 1 passed; 0 failed |
| 8 | Probe contract 8 PASS (missing `->` when ret expected) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_08_missing_ret_arrow_when_expected` | 1 passed; 0 failed |
| 9 | Probe contract 9 PASS (trailing items) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_09_trailing_items_after_ret` | 1 passed; 0 failed |
| 10 | Probe contract 10 PASS (rest-binder rejected) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_10_rest_binder_rejected_when_disallowed` | 1 passed; 0 failed |
| 11 | Probe whole-suite PASS 10/10 (no cross-contamination) | `cargo test --release --test probe_arc241_stone1_argspec_canonical` | 10 passed; 0 failed |
| 12 | Lib baseline preserved (no regression) | `cargo test --release --lib -p wat` | 834 passed; 0 failed (or ≥834 if other stones grew it; never < 834) |
| 13 | Workspace test-build clean (all tests compile) | `cargo build --release --tests --workspace` | exit 0; 0 errors |
| 14 | Clippy delta = 0 (no new warnings) | `cargo clippy --release 2>&1 \| grep -c "^warning"` | ≤ pre-stone count (~54 baseline per CLIFFNOTES; orchestrator captures exact pre-stone count before spawn) |
| 15 | Files touched match discipline | `git diff --name-only HEAD~1 HEAD` (post-commit) | EXACTLY: `src/argspec/mod.rs`, `src/argspec/parse.rs`, `src/argspec/error.rs`, `src/lib.rs`, `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.1.md` (plus the probe if part of same commit) |
| 16 | No prior arc 237 probes regress | `cargo test --release --test probe_arc237_8b_defclause_arithmetic --test probe_arc237_stone5_conforms --test probe_arc237_stone5fix_nominal --test probe_arc237_stone6_is_predicate --test probe_arc238_eq_completeness` | All-suite PASS counts preserved (no regression in any prior probe) |

## Independent prediction (runtime band)

**Target band: 30–50 min Mode A.**
**Upper bound: 60 min (STOP-3).**

**Mode B triggers** (any of these = re-brief, do not commit):
- Probe < 10/10 at sonnet return
- Lib baseline < 834
- Clippy warnings increased above pre-stone count
- Files touched outside the discipline (any STOP-5 hit)
- Any prior arc 237 probe regression

**Mirror precedent: Stone 236.0** (pure additive type-system foundation; ~80–150 lines net; 25-min ship). Stone 241.1 is structurally identical at higher line count due to the parser walker + three error conversions; band adjusted accordingly (30–50 min vs 236.0's 25–45 min).

## Trap-door risks (enumerated; orchestrator watches)

| # | Risk | Detection | Resolution if hit |
|---|---|---|---|
| **T1** | Sonnet implements `From<ArgSpecError>` impls as `todo!()` panics rather than wiring to real RuntimeError/CheckError/TypeError variants | Inspect `src/argspec/error.rs` post-return for `todo!()` macros | Row 13 still passes (compile succeeds) BUT the impls are dead substrate. Re-brief or amend in 241.1.fix. Hard fail. |
| **T2** | Span construction in error variants doesn't match existing per-element discipline (uses form_span everywhere instead of offending-element span where applicable) | Inspect each error-construction site in parse.rs against sub-DESIGN T5 + T8 | Re-brief — span quality is load-bearing for diagnostic UX |
| **T3** | Sonnet adds a 4th file under `src/argspec/` (e.g., `types.rs` or `helpers.rs`) splitting `ArgSpec` away from `parse.rs` | Inspect `ls src/argspec/` post-return | The intueri-recommended decomposition is mod/parse/error specifically. A 4th file is scope creep. Re-brief. |
| **T4** | Sonnet wires A1/A2/A3/A4 to route through canonical in 241.1 (instead of leaving them untouched for 241.2/241.3) | Inspect `git diff src/runtime.rs src/check.rs` post-return | STOP-5 hit (Rust files outside `src/argspec/*` + `src/lib.rs` touched). Hard re-brief. |
| **T5** | Sonnet adds `name_symbol_only` to `ParseOptions` (the rejected D4 surface) | Inspect `src/argspec/parse.rs` for `name_symbol_only` field | STOP-6 hit (scope creep / rejected surface added). Hard re-brief. |
| **T6** | Sonnet implements `&` rest-binder LOGIC (parsing the rest-marker + the following triple), not just the rejection | Inspect probe contract 10 implementation + parse.rs for rest-binder walk | STOP-6 hit (241.4 scope). 241.1 returns `RestBinderNotSupported`; nothing more. Re-brief. |
| **T7** | `mod.rs` is fat (declares types or fn bodies) instead of thin (re-exports only per comms/ precedent) | Inspect `src/argspec/mod.rs` line count + content | Compositional discipline broken. Re-brief. |
| **T8** | Module-level doc on `mod.rs` missing or shallow (doesn't inscribe the failure-class doctrine per sub-DESIGN D8) | Read first 40 lines of `src/argspec/mod.rs` | Doctrine inscription missing. Re-brief — this is load-bearing for future maintainers. |
| **T9** | Sonnet panics on the FM 2-bis probe construction (parsing fails on a contract input due to wat-parse edge case) — e.g., `[:keyword-not-symbol <- :wat::core::i64]` doesn't parse to Vector cleanly | Probe whole-suite < 10/10; specific failure traces to `argspec_inputs` panic | Inspect failing contract source; adjust probe source string to a different input that triggers the same error variant. Update probe + commit; re-spawn. |
| **T10** | `TypeError` import path or `parse_type_expr_with_span` signature doesn't match sub-DESIGN's assumption (e.g., signature changed since AUDIT was written) | Compile error inside `parse.rs` mentioning `parse_type_expr_with_span` | Sonnet uses existing canonical type-keyword parser (whatever it actually is); the function name in the BRIEF is illustrative. Re-brief only if sonnet can't find ANY canonical type-keyword parser. |

## Pre-spawn baseline checks (orchestrator runs BEFORE spawning)

These confirm HEAD is clean enough for the substrate-as-teacher cascade to produce honest signal:

1. **Lib baseline at HEAD = 834 PASS / 0 FAIL.** Already verified — see git commit `e0d1d054` log + this turn's `cargo test --release --lib -p wat` (834 passed; 0 failed; 1 ignored).
2. **Probe compile-fails on EXACTLY `wat::argspec`.** Already verified — single error `unresolved import wat::argspec` (line 18); everything else compiles.
3. **Workspace test-build at HEAD = 1 error** (the probe; expected). No OTHER errors in test-build.
4. **Clippy at HEAD baseline.** Capture exact warning count immediately before spawn; row 14 compares against it.

## What completion looks like (TWO phases — SCORE-green is the floor; vigilia-convergence is the bar)

### Phase A — SCORE scorecard verification (sonnet's behavioral correctness)

After sonnet returns Mode A:
- 16/16 rows of the scorecard verify locally (orchestrator's independent re-run)
- `SCORE-STONE-241.1.md` written with verbatim row results + honest deltas
- **DO NOT commit yet.** Phase A is the L0 floor — substrate works. The bar is Phase B.

### Phase B — Vigilia convergence on the namespaced home (per `feedback_namespaced_home_vigilia_gate`)

Once behavior is dialed in (Phase A green), orchestrator casts **vigilia** on `src/argspec/*` + `tests/probe_arc241_stone1_argspec_canonical.rs`. Vigilia (`~/work/holon/datamancy/vigilia/SKILL.md`) is the aggregator; it spawns the applicable defensive subset in parallel:

| Spell | Concern | Expected for this home |
|---|---|---|
| intueri | Names + structure + communication | Module doc inscribes the doctrine; names speak; ArgSpec/ParseOptions/ArgSpecError say what they hold |
| solvere | Braided concerns; misplaced logic | Parser walker stays in `parse.rs`; error variants in `error.rs`; re-exports only in `mod.rs` |
| purgare | Dead code; unused state | Every variant of ArgSpecError reachable; no unused ParseOptions field; helpers reachable |
| struere | Per-function craft | `parse_argspec_triples` is one function doing one thing; helper `is_bare_symbol` is atomic; values flowing through |
| sequi | Per-chain state threading via types | `Result<ArgSpec, ArgSpecError>` carries everything; no `&mut` accumulation |
| temperare | Redundant work | No re-parsing; no shadow allocations; helper reused, not duplicated |
| complectens | Test composition | Probe's 10 contracts compose from one helper (`argspec_inputs`); each test is one layer |
| vocare | Caller-vantage testing | Probe tests the parser as a caller would invoke it (slice + options); not implementation internals |

**Bar:** L1 + L2 findings = 0. L3 taste noted, not counted. L2 mumbles MAY be accepted via `rune:<spell>(<category>) — <reason>` inscribed at the line if the rune's REASON is load-bearing (per intueri's rune discipline; per vigilia's "do not skip inconvenient findings" rule).

If vigilia finds anything: orchestrator addresses OR directs sonnet to amend; re-cast vigilia; iterate until L1+L2=0.

### Phase C — Commit + push (only after Phase A + Phase B both green)

- SCORE doc amended with a **Vigilia Convergence** section listing each spell's verdict + any runes accepted
- Atomic commit covers: `src/argspec/*` (3 files) + `src/lib.rs` (1 line) + `SCORE-STONE-241.1.md` (with Vigilia Convergence section)
- Push to origin
- **Phase 1 stepping stone laid.** Stone 241.2 (migrate A1/A2/A3) can begin against the **proven AND impeccable** foundation.

User direction governing this two-phase structure: *"we raise the bar fucking high for namespaced wat-rs files - we ensure {src,tests}/argspec/ are shockingly good, remarkably well written - the spells ensure this - we do not move from those until we are exceptional."*

## Calibration history reference

| Stone | Class | Surface | Actual runtime | Calibration accuracy |
|---|---|---|---|---|
| 236.0 (precedent) | Mint type + constructors + tests | ~150 lines net | ~25 min | within 30-45 min target |
| 241.1 (this) | Mint parser + types + tests | ~250 lines net | TBD | predict: 30-50 min |

Per `feedback_stone_briefs_cite_prior_score`: the precedent informs the band; the higher line count justifies the wider band. If 241.1 ships substantially over 50 min, the calibration model needs revision before 241.2's BRIEF.
