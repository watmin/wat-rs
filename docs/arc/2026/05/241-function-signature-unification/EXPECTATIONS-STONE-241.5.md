# EXPECTATIONS — Stone 241.5 — runtime dispatch wiring; defclause rest-binder integration

Independent scorecard for orchestrator-side verification. **No vigilia phase** (legacy flat substrate per DESIGN D7). Commit on SCORE-green.

## Phase A — Scorecard (12 rows)

| Row | Claim | Verification command | Expected result |
|-----|---|---|---|
| 1 | Stone 241.5 probe contracts 01-04 PASS (rest-binder success paths) | `cargo test --release --test probe_arc241_stone5_defclause_rest_dispatch contract_0[1-4]` | 4 passed; 0 failed |
| 2 | Stone 241.5 probe contracts 05-06 PASS (error paths) | `cargo test --release --test probe_arc241_stone5_defclause_rest_dispatch contract_0[56]` | 2 passed; 0 failed |
| 3 | Stone 241.5 probe contracts 07-08 PASS (regression + mixed dispatch) | `cargo test --release --test probe_arc241_stone5_defclause_rest_dispatch contract_0[78]` | 2 passed; 0 failed |
| 4 | Stone 241.5 probe whole-suite 8/8 | `cargo test --release --test probe_arc241_stone5_defclause_rest_dispatch` | 8 passed; 0 failed |
| 5 | 237.8b Gate 1 PASSES (un-ignored; integration test) | `cargo test --release --test probe_arc237_8b_defclause_arithmetic gate_1_defclause_supports_rest_binder` | 1 passed; 0 failed |
| 6 | Stone 241.4 canonical probe preserved 15/15 | `cargo test --release --test probe_arc241_stone1_argspec_canonical` | 15 passed; 0 failed |
| 7 | Stone 241.3 probe preserved 6/6 | `cargo test --release --test probe_arc241_stone3_defclause_parser_migration` | 6 passed; 0 failed |
| 8 | Stone 241.2 probe preserved 10/10 | `cargo test --release --test probe_arc241_stone2_fn_parser_migration` | 10 passed; 0 failed |
| 9 | Lib baseline preserved | `cargo test --release --lib -p wat` | 834 passed; 0 failed (or higher) |
| 10 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0; 0 errors |
| 11 | Clippy delta ≤ 0 | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 904 |
| 12 | Arc 237/238 probes preserved | `cargo test --release --test probe_arc237_stone5_conforms --test probe_arc237_stone5fix_nominal --test probe_arc237_stone6_is_predicate --test probe_arc238_eq_completeness` | counts preserved (12+12+10+8) |

## Structural verification (6 rows)

| Verification | Command | Expected |
|---|---|---|
| Variadic-min arity check present | `grep -n "has_rest\|called_arity >= fixed_arity" src/runtime.rs` | ≥ 1 match |
| Rest-binder type extraction present | `grep -n "wat::core::Vector.*args" src/runtime.rs` OR similar parametric extraction | ≥ 1 match |
| `Value::Vector` construction present | `grep -n "Value::Vector(" src/runtime.rs \| wc -l` | post-stone count > pre-stone count |
| Gate 1 `#[ignore]` REMOVED | `grep -B1 "fn gate_1_defclause_supports_rest_binder" tests/probe_arc237_8b_defclause_arithmetic.rs \| grep -c "#\[ignore"` | 0 |
| `src/argspec/*` UNCHANGED | `git diff src/argspec/` | empty diff |
| `src/lib.rs` UNCHANGED | `git diff src/lib.rs` | empty diff |

## Independent prediction (runtime band)

**Target band: 20-40 min Mode A.**
**Upper bound: 60 min (STOP-3).**

**Mode B triggers**:
- Stone 241.5 probe < 8/8 PASS
- Gate 1 still RED after un-ignore (STOP-10 — surface as gap; possibly Stone 241.6)
- Lib < 834
- Files outside discipline touched
- `src/argspec/*` modified
- `src/lib.rs` modified
- Stone 241.x probes regress; arc 237/238 probes regress
- New types/variants/fields
- Check-layer changes > ~10 lines (STOP-6)
- Vector construction > ~10 lines (STOP-6)
- Clippy > 904

**Mirror precedent: Stone 241.2** (A1+A2+A3 migration; ~7 min Mode A). Stone 241.5 is similar scope (single substrate site; multiple coordinated edits; clear semantic decisions locked).

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | Sonnet uses strict equality for variadic case (forgets `has_rest` branch) | Inspect S1 implementation | Re-brief |
| **T2** | Element-type extraction missing or wrong (treats Vector<T> as T) | Inspect S3 — must extract args[0] from Parametric Vector | Re-brief |
| **T3** | Vector construction requires deep substrate work (new HolonVector ctor) | grep "Value::Vector(" + read sonnet's approach | STOP-6; surface as follow-up if depth > ~10 lines |
| **T4** | Check layer rejects rest-binder body type-check | Startup fails on Gate 1's source | STOP-6; surface as follow-up; Stone 241.6 may need check-layer work |
| **T5** | Sonnet touches src/argspec/ | git diff | STOP-5; re-brief |
| **T6** | Sonnet adds new ClauseFailureReason variants | grep ClauseFailureReason enum | STOP-6; reuse existing |
| **T7** | Sonnet adds new Clause struct fields | git diff src/runtime.rs near struct Clause | STOP-6; Clause already has rest_param from 241.4 |
| **T8** | Test cascade is larger than Stone 241.2-4 (test assertions on rest-binder behavior break elsewhere) | Lib test failures | Acceptable; document each |
| **T9** | Sonnet runs wrapper scripts | grep output | False claim; FM 7 verification |
| **T10** | Gate 1 passes BUT subtle behavior shift in Stone 241.x probes | Re-run all probes | STOP-8; honest delta |

## Pre-spawn baseline checks

1. **Stone 241.5 probe at HEAD = 3 PASS / 5 FAIL** (verified; FM 2-bis disconfirmation — UnboundSymbol("rest") at body-eval time).
2. **Stone 241.4 canonical probe at HEAD = 15/15 PASS.**
3. **Stone 241.3 probe at HEAD = 6/6 PASS.**
4. **Stone 241.2 probe at HEAD = 10/10 PASS.**
5. **237.8b Gate 1 at HEAD = #[ignore]'d** (Stone 241.4's deferral; un-ignore is the integration verification).
6. **Lib at HEAD = 834 PASS / 0 FAIL.**
7. **Clippy at HEAD = 904 warnings.**

## What completion looks like

### Phase A — SCORE scorecard + structural verification

After sonnet returns Mode A:
- 12/12 scorecard rows verify
- 6/6 structural rows verify
- `SCORE-STONE-241.5.md` written with verbatim results + honest deltas
- **PHASE 1 TRULY CLOSED inscription** in SCORE (canonical parser shape complete + runtime dispatch wired)

### Phase B — NOT cast (gate doctrine doesn't apply per DESIGN D7)

### Phase C — Commit + push

- Atomic commit covers: `src/runtime.rs`, `tests/probe_arc241_stone5_defclause_rest_dispatch.rs`, `tests/probe_arc237_8b_defclause_arithmetic.rs` (un-ignore), `SCORE-STONE-241.5.md`, any test files with assertion updates
- Push to origin
- **PHASE 1 TRULY CLOSED**: canonical parser shape complete (241.1-241.4) + dispatch wired (241.5) → defclause has full rest-binder + Gate 1 integration confirmed; arc 237.8b unpauses

## Calibration history reference

| Stone | Class | Surface delta | Predicted | Actual | Status |
|---|---|---|---|---|---|
| 241.1 | Mint canonical | +519 net | 30-50 min | ~50 min | within |
| 241.1.fix L1 | Vigilia amends | -88 net | 20-30 min | ~8 min | UNDER |
| 241.1.fix L2 | Scope correction | -127 net | 20-35 min | ~8 min | UNDER |
| 241.1.fix struere | 3-line amend | -3 net | 5-10 min | ~5 min | within |
| 241.2 | A1+A2+A3 migration | -100 net | 40-60 min | ~7 min | UNDER |
| 241.3 | A4 migration | -57 net | 15-30 min | ~5.6 min | UNDER |
| 241.4 | Rest-binder ext + helper + opt-in + L2 closures | +125 net | 30-50 min | ~30 min total | within |
| 241.5 (this) | Runtime dispatch + Gate 1 unblock | ~+190 net (mostly probe) | 20-40 min | TBD | — |

Per Stone 241.x calibration: substrate changes mechanical; runtime variable is the test-assertion cascade depth. Stone 241.5's behavior change (variadic dispatch) MAY break tests that asserted "rest-binder rejected" expectations elsewhere; expect small cascade.

## What this unblocks

**Arc 237.8b** unpauses fully: Gate 1 green; Gates 2-4 + mint-confirmers proceed. The original blocker that drove arc 241's opening (seven stones ago).

**Phase 1 of arc 241 TRULY CLOSED**: canonical parser shape complete + runtime dispatch wired + Gate 1 integration verified.

**Phase 2 of arc 241 opens**: 241.6 `:wat::runtime::metadata-of` reflection verb + 241.7 optional `{...}` metadata-map on `def`/defn.
