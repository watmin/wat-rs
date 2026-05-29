# EXPECTATIONS — Stone 242.2 — Doctrine 1 enforcement + value-position cascade

Independent scorecard. NO vigilia required (D6 — no namespaced home). SCORE-green commit. Upper bound 180 min.

## Phase A — Scorecard (10 rows)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | Probe contracts 01-06 PASS | `cargo test ... probe_arc242_stone2_value_position_doctrine` | 6/0 |
| 2 | Stone 242.1 probe preserved 4/4 | `cargo test --release --test probe_arc242_stone1_lexeme_role` | 4/0 |
| 3 | Stone 241.11 probe preserved 5/5 | `cargo test --release --test probe_arc241_stone11_define_hard_cut` | 5/0 |
| 4 | Stone 241.10 probe preserved 8/8 | `cargo test --release --test probe_arc241_stone10_remedy` | 8/0 |
| 5 | Stone 241.1-241.9 + arc 237/238 probes preserved | each | counts preserved |
| 6 | Lib baseline | `cargo test --release --lib -p wat` | ≥ 890 PASS / 0 FAIL |
| 7 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0 |
| 8 | Clippy gate | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 902 |
| 9 | Type-check rejection arm present | `grep -n "Doctrine 1\|value position" src/check.rs` | ≥ 1 match (the rejection arm) |
| 10 | No active type-keyword-in-value-position uses | per cascade audit | 0 active uses outside error message strings + retirement tests |

## Structural verification (6 rows)

| Verification | Command | Expected |
|---|---|---|
| Type-check rejection arm fires on `:wat::core::nil` in value position | probe C01 + C05 | both PASS |
| Type-check rejection arm fires on other type keywords in value position | probe C03 | PASS |
| Reflection emitter for `Value::Unit` produces bare nil AST | `grep -n "Value::Unit" src/closure_extract.rs` | emits Symbol("nil") or equivalent bare form |
| INTERSTITIAL UNCHANGED by sonnet | `git diff docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` | empty diff |
| RETIREMENT_TABLE UNCHANGED (5 entries) | `grep -c '(\":wat::core' src/remedy/retirement.rs` | 5 LHS matches |
| Auto-fixer crate DELETED | `ls crates/fix-*/ 2>&1` | "No such file or directory" |

## Prediction: 60-180 min Mode A

Substrate work:
- Type-check rejection arm: ~30-60 lines (find entry point + add Keyword arm + remedy construction)
- Reflection emitter migration: ~5-15 sites (closure_extract + runtime + check)
- Cascade migration (test sources + Rust internal): ~20-60 sites
- Probe verification + SCORE doc

Per `feedback_stone_briefs_cite_prior_score`: BRIEF cites SCORE-STONE-242.1.md for arc 242 context + SCORE-STONE-241.10.md for the remedy apparatus pattern.

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | Type-check entry point hard to identify | sonnet's audit | follow check_expr / check_value naming; document path in SCORE |
| **T2** | TypeEnv lookup at every value-position Keyword expensive | profile (if needed) | acceptable — type-check is not hot path; D2 from DESIGN |
| **T3** | Reflection emitter migration breaks existing reflection tests | cascade | migrate per substrate-as-teacher; document |
| **T4** | Test source migration touches many files | cascade | apply consistently; auto-fixer if surfaces |
| **T5** | Auto-fixer ephemeral discipline | git status post-strike | STOP if survives |
| **T6** | "Intentional bypass" framing for substrate-internal nil uses | sonnet self-audit | STOP per `feedback_hard_cut_admits_no_bypasses`; migrate |
| **T7** | INTERSTITIAL temptation (drafting under cover of SCORE) | git diff INTERSTITIAL post-strike | STOP per `feedback_sonnet_never_drafts_interstitial`; revert |

## Pre-spawn baseline checks

1. Stone 242.2 probe at HEAD = 3/6 PASS (verified — C02/C04/C06 legal forms work; C01/C03/C05 fail because rejection arm not yet minted)
2. Lib at HEAD = 890 PASS / 0 FAIL
3. All Stone 241.x + Stone 242.1 probes at current counts
4. Clippy at 902 (at gate)

## What completion looks like

### Phase A — SCORE + structural verification

After sonnet returns:
- 10/10 scorecard verifies locally
- 6/6 structural rows verify locally
- SCORE doc at `docs/arc/2026/05/242-lexeme-role-doctrine/SCORE-STONE-242.2.md`
- INTERSTITIAL UNTOUCHED (orchestrator authors after arc 242 closes)

### Phase B — NOT cast (no vigilia per D6)

### Phase C — Commit + push

- Atomic commit covers: `src/check.rs` (rejection arm), `src/closure_extract.rs` (reflection emitter migration), `src/runtime.rs` (if needed), cascade target files, SCORE doc
- INTERSTITIAL NOT in commit (orchestrator authors separately after Stone 242.3 INSCRIPTION)
- Push to origin
- Stone 242.3 INSCRIPTION opens next (orchestrator-direct paperwork)
- After 242.3 INSCRIPTION ships, arc 241 resumes at Stone 241.12

## Calibration history reference

| Stone | Class | Predicted | Actual |
|---|---|---|---|
| 242.1 | Bare nil audit + Char HARD CUT (~18 sites) + doctrine memory | 60-150 min | shipped within band |
| **242.2 (this)** | **Type-check rejection arm + reflection emitter migration + cascade (~25-75 sites)** | **60-180 min** | **TBD** |

## What this unblocks

**Stone 242.3** — INSCRIPTION closes arc 242 (orchestrator-direct; no substrate edits)

**Arc 241 RESUMES** — Stone 241.12 (defalias mint) opens fresh after Stone 242.3 closes

**Doctrine 1 SELF-ENFORCING** — future writes of `:wat::core::*` keywords in value position are AUTOMATICALLY rejected with structured guidance. The doctrine becomes the substrate's enforced law, not just inscribed convention.
