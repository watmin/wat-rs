# EXPECTATIONS — Stone 241.3 — migrate A4 defclause parser through canonical

Independent scorecard for orchestrator-side verification after sonnet returns. Each row is a fact to confirm via an explicit command; orchestrator re-runs locally and writes the verbatim result into `SCORE-STONE-241.3.md`.

## Phase A — Scorecard (11 rows)

| Row | Claim | Verification command | Expected result |
|-----|---|---|---|
| 1 | Probe contracts 1-3 PASS (happy paths preserved) | `cargo test --release --test probe_arc241_stone3_defclause_parser_migration contract_0` | 3 passed; 0 failed |
| 2 | Probe contract 4 PASS (NameNotSymbol errors) | `cargo test --release --test probe_arc241_stone3_defclause_parser_migration contract_04` | 1 passed; 0 failed |
| 3 | Probe contract 5 PASS (MissingArrow errors) | `cargo test --release --test probe_arc241_stone3_defclause_parser_migration contract_05` | 1 passed; 0 failed |
| 4 | Probe contract 6 PASS (IncompleteTriple errors) | `cargo test --release --test probe_arc241_stone3_defclause_parser_migration contract_06` | 1 passed; 0 failed |
| 5 | Probe whole-suite 6/6 | `cargo test --release --test probe_arc241_stone3_defclause_parser_migration` | 6 passed; 0 failed |
| 6 | Stone 241.2 probe preserved 10/10 | `cargo test --release --test probe_arc241_stone2_fn_parser_migration` | 10 passed; 0 failed |
| 7 | Stone 241.1 probe preserved 9/9 | `cargo test --release --test probe_arc241_stone1_argspec_canonical` | 9 passed; 0 failed |
| 8 | Lib baseline preserved | `cargo test --release --lib -p wat` | 834 passed; 0 failed (or higher; never < 834) |
| 9 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0; 0 errors |
| 10 | Clippy delta = 0 | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 905 |
| 11 | No prior arc 237 probe regresses | `cargo test --release --test probe_arc237_stone5_conforms --test probe_arc237_stone5fix_nominal --test probe_arc237_stone6_is_predicate --test probe_arc238_eq_completeness` | All PASS counts preserved (12+12+10+8) |

## Structural verification

| Verification | Command | Expected |
|---|---|---|
| A4's inline triple walker GONE | `grep -A 80 "^fn parse_defclause_args" src/runtime.rs \| grep -c "while i < args_vec.len()"` | 0 |
| A4 routes through canonical | `grep -A 15 "^fn parse_defclause_args" src/runtime.rs \| grep -c "parse_argspec_triples"` | ≥ 1 |
| A4 returns `spec.fixed_params` directly | `grep -A 15 "^fn parse_defclause_args" src/runtime.rs \| grep -c "spec.fixed_params"` | ≥ 1 |
| A4 public signature unchanged | `grep "fn parse_defclause_args" src/runtime.rs` | one match; signature `(args_vec: &[WatAST], head: &str, form_span: &Span) -> Result<Vec<(String, ...)>, RuntimeError>` |
| Caller `parse_defclause_clause` UNTOUCHED | `git diff src/runtime.rs \| grep "parse_defclause_clause"` | no matches in diff (function body untouched) |
| `src/argspec/*` UNCHANGED | `git diff src/argspec/` | empty diff |
| `src/lib.rs` UNCHANGED | `git diff src/lib.rs` | empty diff |
| `src/check.rs` UNCHANGED | `git diff src/check.rs` | empty diff |

## Independent prediction (runtime band)

**Target band: 15-30 min Mode A.**
**Upper bound: 30 min (STOP-3).**

**Mode B triggers** (any of these = re-brief, do not commit):
- Probe < 6/6 PASS
- Lib baseline < 834
- Stone 241.1 / 241.2 probes regress
- Files touched outside discipline (any STOP-5 hit)
- `src/check.rs`, `src/argspec/*`, `src/lib.rs` modified
- A4 public signature changed
- Caller `parse_defclause_clause` modified
- New types/fields/variants introduced
- Clippy > 905
- Any prior arc 237 probe regression

**Mirror precedent: Stone 241.2** shipped ~100-line net delta + zero test cascade in ~7 min Mode A. Stone 241.3 is even simpler (single site; direct return of fixed_params; no unzip). Expect 5-15 min actual.

## Trap-door risks

| # | Risk | Detection | Resolution if hit |
|---|---|---|---|
| **T1** | Sonnet adds `.into_iter().unzip()` despite not needing it | Inspect M1 body for `unzip()` | Re-brief — D6 says direct return |
| **T2** | Sonnet hardcodes `:wat::core::defclause` instead of forwarding `head` | grep new body for hardcoded string | Re-brief — D4 says forward variable head |
| **T3** | Sonnet "fixes" T2 (position-index loss) by adding `triple_pos` field | Inspect `src/argspec/error.rs` for `triple_pos` | STOP-6 — DESIGN T2 verdict β |
| **T4** | Sonnet touches `src/check.rs` (A2/A3 territory from 241.2) | `git diff src/check.rs` | STOP-5 — re-brief |
| **T5** | Sonnet touches the caller `parse_defclause_clause` | grep new diff lines for `parse_defclause_clause` | STOP-6 — caller MUST stay unchanged |
| **T6** | A4's doc comment misrepresents post-migration behavior | Read the comment + new body | Optional update; surface as honest delta |
| **T7** | Test cascade is larger than zero (assertion updates needed) | Lib test failures | Update assertions; document each in SCORE Honest Deltas |
| **T8** | Sonnet runs wrapper scripts or claims tool denial | grep output for wrapper invocation | False claim; FM 7 verification |

## Pre-spawn baseline checks (orchestrator runs BEFORE spawning)

1. **Stone 241.3 probe at HEAD = 6/6 PASS.** Verified this turn.
2. **Stone 241.2 probe at HEAD = 10/10 PASS.** Verified at prior commit.
3. **Stone 241.1 probe at HEAD = 9/9 PASS.** Verified earlier.
4. **Lib baseline at HEAD = 834 PASS / 0 FAIL.** Verified.
5. **Clippy baseline = 905 warnings.** Verified.

## What completion looks like

### Phase A — SCORE scorecard verification + structural verification

After sonnet returns Mode A:
- 11/11 scorecard rows verify locally
- 8/8 structural rows verify locally
- `SCORE-STONE-241.3.md` written with verbatim row results + honest deltas
- **Phase 1 closure inscribed** in SCORE: all 4 parsers route through canonical

### Phase B — Vigilia: NOT CAST

Per DESIGN D9: Stone 241.3 touches `src/runtime.rs` (flat substrate). Gate doctrine doesn't apply. Commits on SCORE-green.

### Phase C — Commit + push

- Atomic commit covers: `src/runtime.rs`, `SCORE-STONE-241.3.md`, any test files with assertion updates (likely zero)
- Push to origin
- **PHASE 1 CLOSES**: parser-divergence class structurally eliminated; canonical home is exceptional; 4 parsers route through 1.
- Stone 241.4 opens next: extend canonical with `&` rest-binder logic; unblocks 237.8b

## Calibration history reference

| Stone | Class | Surface delta | Predicted | Actual | Status |
|---|---|---|---|---|---|
| 241.1 | Mint parser + types + tests | +519 net | 30-50 min | ~50 min | within band |
| 241.1.fix Layer 1 | Vigilia amends | -88 net | 20-30 min | ~8 min | UNDER (mechanical) |
| 241.1.fix Layer 2 | Scope correction | -127 net | 20-35 min | ~8 min | UNDER (mechanical) |
| 241.1.fix struere closure | 3-line amend | -3 net | 5-10 min | ~5 min | within |
| 241.2 | A1/A2/A3 migration | -100 net + 0 test updates | 40-60 min | ~7 min | UNDER (zero cascade) |
| 241.3 (this) | A4 migration | -57 net + 0 expected updates | 15-30 min | TBD | — |

Per Stone 241.2's zero-cascade learning: Stone 241.3 cascade expected near-zero. If non-zero, surface as honest delta.

## What this unblocks

**Phase 1 closure** (parser-divergence class structurally eliminated): four parsers route through one canonical; same structural failures produce same `ArgSpecError` variants; per-site error conversion at the call boundary via `From<>` impls.

**Stone 241.4** opens: extend canonical with `&` rest-binder logic when `allow_rest_binder: true`. Unblocks probe 237.8b Gate 1 (defclause arithmetic + rest-binder).
