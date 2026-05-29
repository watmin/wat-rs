# EXPECTATIONS — Stone 241.4 — canonical `&` rest-binder + defclause opt-in; unblocks 237.8b Gate 1

Independent scorecard for orchestrator-side verification after sonnet returns. Vigilia gate APPLIES (`src/argspec/` namespaced home); two phases.

## Phase A — Scorecard (16 rows)

| Row | Claim | Verification command | Expected result |
|-----|---|---|---|
| 1 | Canonical probe contracts 01-09 still PASS (regression) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_0` | 9 passed; 0 failed |
| 2 | Canonical probe contract 10 PASS (rest-only succeeds) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_10` | 1 passed; 0 failed |
| 3 | Canonical probe contract 11 PASS (fixed+rest succeeds) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_11` | 1 passed; 0 failed |
| 4 | Canonical probe contract 12 PASS (TrailingItems verified — DESIGN T2 verdict β confirmed) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_12` | 1 passed; 0 failed |
| 5 | Canonical probe contracts 13-15 PASS (incomplete + non-symbol + regression) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_1[345]` | 3 passed; 0 failed |
| 6 | Canonical probe whole-suite 15/15 | `cargo test --release --test probe_arc241_stone1_argspec_canonical` | 15 passed; 0 failed |
| 7 | Stone 241.2 probe preserved 10/10 | `cargo test --release --test probe_arc241_stone2_fn_parser_migration` | 10 passed; 0 failed |
| 8 | Stone 241.3 probe preserved 6/6 | `cargo test --release --test probe_arc241_stone3_defclause_parser_migration` | 6 passed; 0 failed |
| 9 | 237.8b Gate 1 PASSES (was `#[ignore]`'d; now active) | `cargo test --release --test probe_arc237_8b_defclause_arithmetic gate_1_defclause_supports_rest_binder` | 1 passed; 0 failed |
| 10 | Lib baseline preserved | `cargo test --release --lib -p wat` | 834 passed; 0 failed (or higher) |
| 11 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0; 0 errors |
| 12 | Clippy delta = 0 | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 905 |
| 13 | No prior arc 237 probe (non-Gate-1) regresses | `cargo test --release --test probe_arc237_stone5_conforms --test probe_arc237_stone5fix_nominal --test probe_arc237_stone6_is_predicate --test probe_arc238_eq_completeness` | counts preserved (12+12+10+8) |
| 14 | `src/lib.rs` UNCHANGED | `git diff src/lib.rs` | empty diff |
| 15 | `src/argspec/mod.rs` UNCHANGED | `git diff src/argspec/mod.rs` | empty diff |
| 16 | `src/check.rs` UNCHANGED | `git diff src/check.rs` | empty diff |

## Structural verification (8 rows; orchestrator runs before Phase B)

| Verification | Command | Expected |
|---|---|---|
| All 3 future-fixture runes REMOVED | `grep -cE "rune:purgare\(future-fixture\)" src/argspec/parse.rs src/argspec/error.rs` | 0 |
| `_options` un-prefixed to `options` | `grep -n "_options:" src/argspec/parse.rs` | no matches |
| `parse_triple` helper present (PRIVATE) | `grep -n "fn parse_triple" src/argspec/parse.rs` | one match; not `pub` |
| Rest-binder branch present in canonical | `grep -n "rest_param: Some" src/argspec/parse.rs` | ≥ 1 match |
| A4 returns ArgSpec | `grep -A 2 "^fn parse_defclause_args" src/runtime.rs \| grep "ArgSpec"` | match present |
| Defclause sets allow_rest_binder: true | `grep -A 5 "parse_argspec_triples" src/runtime.rs \| grep "allow_rest_binder: true"` | ≥ 1 match |
| Gate 1 `#[ignore]` REMOVED | `awk '/gate_1_defclause_supports_rest_binder/{print prev}{prev=$0}' tests/probe_arc237_8b_defclause_arithmetic.rs \| grep -c "#\[ignore"` | 0 |
| Stone 241.2 + 241.3 probes UNCHANGED | `git diff tests/probe_arc241_stone2_fn_parser_migration.rs tests/probe_arc241_stone3_defclause_parser_migration.rs \| wc -l` | 0 |

## Independent prediction (runtime band)

**Target band: 30-50 min Mode A.**
**Upper bound: 60 min (STOP-3).**

**Mode B triggers** (any of these = re-brief, do not commit):
- Canonical probe < 15/15 PASS
- Gate 1 fails after un-ignore (deeper integration needed; surface as gap)
- Lib baseline < 834 (after assertion updates)
- Files touched outside discipline
- A1/A2/A3 fn-form parsers touched (they don't opt in)
- Clause runtime dispatch goes >30 lines (STOP-6 scope creep — surface as future stone)
- New ArgSpecError variants / ParseOptions fields / public API changes beyond A4's return type
- Clippy > 905
- Vigilia DIVERGES with non-trivial findings

**Mirror precedent: Stone 241.1.fix Layer 1** (~88 lines net, ~8 min Mode A); Stone 241.4 is larger surface (rest-binder branch + helper extract + defclause integration + Gate 1 unblock + 6 new probe contracts) but mechanical given the locked decisions. Predict 30-50 min.

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | Sonnet preserves any of the 3 runes | `grep -cE "rune:purgare\(future-fixture\)" src/argspec/*` returns > 0 | Re-brief; the prediction holds; runes retire |
| **T2** | Sonnet doesn't extract `parse_triple` (leaves duplication) | `grep -c "fn parse_triple" src/argspec/parse.rs` returns 0 | Re-brief; vigilia would flag the duplication anyway |
| **T3** | A4 still returns `Vec<(String, TypeExpr)>` (didn't evolve to ArgSpec) | Inspect A4 signature | Re-brief; D6 of DESIGN mandates ArgSpec return |
| **T4** | Defclause caller doesn't consume spec.rest_param | Inspect parse_defclause_clause body | Re-brief; the rest-binder must FLOW to Clause |
| **T5** | Clause integration is too deep (>30 lines) | Inspect Clause struct + dispatch changes | STOP-6 — surface as follow-up stone; Stone 241.4 ships parser+opt-in; runtime dispatch may belong elsewhere |
| **T6** | Gate 1 fails despite substrate changes (deeper gap) | Run `cargo test ... gate_1` post-stone | Surface as honest delta; Stone 241.5 may be needed to fully unblock 237.8b |
| **T7** | Doc comments still future-tense ("Stone 241.4 wires...") | Read parse.rs doc comments | Re-brief per intueri (S4 task) |
| **T8** | Test-assertion cascade is large (many lib tests update) | SCORE Honest Deltas inventory | Acceptable; document each |
| **T9** | Sonnet touches A1/A2/A3 (fn-form parsers) | `git diff src/runtime.rs src/check.rs \| grep -i "parse_fn"` | STOP-5; A1/A2/A3 stay |
| **T10** | Vigilia Phase B surfaces L1+L2 > 0 | Vigilia aggregate verdict | Address before commit; Stone 241.4.fix if non-trivial |

## Pre-spawn baseline checks

1. **Canonical probe at HEAD = 10 PASS / 5 FAIL.** Verified this turn — contracts 10-14 fail with RestBinderNotSupported on EXACTLY the missing behavior; contract 15 PASS (regression).
2. **Stone 241.2 probe at HEAD = 10/10 PASS.**
3. **Stone 241.3 probe at HEAD = 6/6 PASS.**
4. **237.8b Gate 1 at HEAD = `#[ignore]`'d** (not RUN; un-ignore post-stone is the integration test).
5. **Lib baseline = 834 PASS / 0 FAIL.**
6. **Clippy baseline = 905 warnings.**

## What completion looks like (TWO phases — gate doctrine applies)

### Phase A — SCORE scorecard + structural verification

After sonnet returns Mode A:
- 16/16 scorecard rows verify locally
- 8/8 structural rows verify locally
- `SCORE-STONE-241.4.md` written with verbatim results + honest deltas
- **DO NOT commit yet.** Phase A is the L0 floor; the bar is Phase B.

### Phase B — Vigilia re-cast on `src/argspec/*` + extended canonical probe

Orchestrator casts vigilia. 8 spells in parallel (intueri / solvere / purgare / struere / sequi / temperare / complectens / vocare).

Expected verdict shape:
- **purgare**: ZERO runes left in argspec — the prediction held; the future is the present. Convergence expected.
- **struere**: `parse_triple` helper at the right level; rest-binder branch composes from existing helpers; no panic paths. Convergence expected.
- **intueri**: doc comments updated to descriptive (not future-tense); names speak; rune comments removed cleanly. Convergence expected.
- **solvere**: rest-binder branch is one coherent block; fixed-param + rest-binder share via parse_triple. Convergence expected.
- **sequi**: ArgSpec flow through A4 → parse_defclause_clause → Clause struct visible end-to-end. May surface if Clause integration is messy.
- **temperare**: no new redundant work in the loop. Convergence expected.
- **complectens**: probe 15 contracts compose from parse_triples helper; new contracts shape clean. Convergence expected.
- **vocare**: contracts at canonical caller vantage (no private helper reach). Convergence expected.

**Bar:** L1 + L2 = 0. Rune-accepts only with load-bearing reasons.

If vigilia surfaces findings: address before commit; re-cast; iterate until L1+L2=0 (or Stone 241.4.fix if substantial).

### Phase C — Commit + push (only after Phase A + Phase B both green)

- SCORE doc amended with Vigilia Convergence section
- Atomic commit covers: `src/argspec/{parse,error}.rs`, `src/runtime.rs`, `tests/probe_arc241_stone1_argspec_canonical.rs`, `tests/probe_arc237_8b_defclause_arithmetic.rs`, `SCORE-STONE-241.4.md`
- Push to origin
- **Phase 1 capstone landed.** All three runes retired; defclause has rest-binder; 237.8b Gate 1 green; arc 237.8b unpauses.

## Calibration history reference

| Stone | Class | Surface delta | Predicted | Actual |
|---|---|---|---|---|
| 241.1 | Mint parser + types + tests | +519 net | 30-50 min | ~50 min |
| 241.1.fix Layer 1 | Vigilia amends | -88 net | 20-30 min | ~8 min |
| 241.1.fix Layer 2 | Scope correction | -127 net | 20-35 min | ~8 min |
| 241.1.fix struere | 3-line amend | -3 net | 5-10 min | ~5 min |
| 241.2 | A1/A2/A3 migration | -100 net | 40-60 min | ~7 min |
| 241.3 | A4 migration | -57 net | 15-30 min | ~5.6 min |
| 241.4 (this) | Rest-binder ext + helper + opt-in + Gate 1 unblock | ~+125 net | 30-50 min | TBD |

Stone 241.4 is larger surface than recent stones (the migrations were net-negative; this is net-positive — new behavior). The vigilia gate adds Phase B overhead. Total session may be 45-75 min.

## What this closes / unblocks

**Phase 1 capstone**: canonical parser ships its first-release shape complete; three future-fixture runes retire; defclause has rest-binder support.

**Arc 237.8b UNPAUSES**: Gate 1 green; Gates 2-4 + mint-confirmers can proceed.

**Phase 2 of arc 241 opens**: 241.5 mints `:wat::runtime::metadata-of` reflection verb + 241.6 optional `{...}` metadata-map on `def` (defn inherits).
