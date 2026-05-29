# EXPECTATIONS — Stone 241.2 — migrate A1/A2/A3 fn parsers through canonical

Independent scorecard for orchestrator-side verification after sonnet returns. Each row is a fact to confirm via an explicit command; orchestrator re-runs locally and writes the verbatim result into `SCORE-STONE-241.2.md`.

## Phase A — Scorecard (14 rows)

| Row | Claim | Verification command | Expected result |
|-----|---|---|---|
| 1 | Behavioral-parity probe contracts 01-04 PASS (happy paths preserved) | `cargo test --release --test probe_arc241_stone2_fn_parser_migration contract_0` | 4 passed; 0 failed (regex matches `contract_0[1-4]`) |
| 2 | Behavioral-parity probe contract 05 PASS (NameNotSymbol errors) | `cargo test --release --test probe_arc241_stone2_fn_parser_migration contract_05` | 1 passed; 0 failed |
| 3 | Behavioral-parity probe contract 06 PASS (MissingArrow errors) | `cargo test --release --test probe_arc241_stone2_fn_parser_migration contract_06` | 1 passed; 0 failed |
| 4 | Behavioral-parity probe contract 07 PASS (non-keyword type errors) | `cargo test --release --test probe_arc241_stone2_fn_parser_migration contract_07` | 1 passed; 0 failed |
| 5 | Behavioral-parity probe contract 08 PASS (incomplete triple errors) | `cargo test --release --test probe_arc241_stone2_fn_parser_migration contract_08` | 1 passed; 0 failed |
| 6 | Behavioral-parity probe contracts 09-10 PASS (ret-clause inline unchanged) | `cargo test --release --test probe_arc241_stone2_fn_parser_migration contract_09 contract_10` | 2 passed; 0 failed |
| 7 | Behavioral-parity probe whole-suite 10/10 | `cargo test --release --test probe_arc241_stone2_fn_parser_migration` | 10 passed; 0 failed |
| 8 | Stone 241.1 probe preserved 9/9 | `cargo test --release --test probe_arc241_stone1_argspec_canonical` | 9 passed; 0 failed |
| 9 | Lib baseline preserved | `cargo test --release --lib -p wat` | 834 passed; 0 failed (or higher; never < 834) |
| 10 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0; 0 errors |
| 11 | Clippy delta = 0 | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 905 (pre-stone baseline) |
| 12 | Files touched match discipline | `git diff --name-only HEAD` (pre-commit) | ONLY: `src/runtime.rs`, `src/check.rs`, test files with message-assertion updates, `SCORE-STONE-241.2.md` |
| 13 | `src/argspec/*` UNCHANGED | `git diff src/argspec/` | empty diff |
| 14 | `src/lib.rs` UNCHANGED | `git diff src/lib.rs` | empty diff |

## Structural verification (orchestrator runs before SCORE acceptance)

| Verification | Command | Expected |
|---|---|---|
| A1's inline triple walker GONE | `awk '/^fn parse_fn_signature\b/,/^}/' src/runtime.rs \| grep -c "i + 2 >= args_vec.len()"` | 0 |
| A2's inline triple walker GONE | `awk '/^fn parse_fn_signature_for_check\b/,/^}/' src/check.rs \| grep -c "while i < args_vec.len()"` | 0 |
| A3's inline triple walker GONE | `awk '/^fn parse_fn_signature_for_check_diag\b/,/^}/' src/check.rs \| grep -c "while i < args_vec.len()"` | 0 |
| A1 routes through canonical | `awk '/^fn parse_fn_signature\b/,/^}/' src/runtime.rs \| grep -c "parse_argspec_triples"` | ≥ 1 |
| A2 routes through canonical | `awk '/^fn parse_fn_signature_for_check\b/,/^}/' src/check.rs \| grep -c "parse_argspec_triples"` | ≥ 1 |
| A3 routes through canonical | `awk '/^fn parse_fn_signature_for_check_diag\b/,/^}/' src/check.rs \| grep -c "parse_argspec_triples"` | ≥ 1 |
| A1 public signature unchanged | `grep "fn parse_fn_signature(" src/runtime.rs` | one match; signature `(args: &[WatAST]) -> Result<(Vec<String>, Vec<...>, ...), RuntimeError>` |
| A2 public signature unchanged | `grep "fn parse_fn_signature_for_check(" src/check.rs` | one match; signature unchanged |
| A3 public signature unchanged | `grep "fn parse_fn_signature_for_check_diag(" src/check.rs` | one match; signature unchanged |
| No new helpers minted | `grep -n "fn parse_ret_clause\|fn split_at_arrow" src/runtime.rs src/check.rs` | no matches |

## Independent prediction (runtime band)

**Target band: 40-60 min Mode A.**
**Upper bound: 90 min (STOP-3).**

**Mode B triggers** (any of these = re-brief, do not commit):
- Behavioral-parity probe < 10/10 PASS at sonnet return
- Stone 241.1 probe < 9/9 PASS (substrate corruption)
- Lib baseline < 834 (after assertion updates)
- Files touched outside discipline (any STOP-5 hit)
- `src/argspec/*` modified (D-violation)
- `src/lib.rs` modified
- A1/A2/A3 public signatures changed
- New types / fields / variants introduced
- A4 (defclause) touched (Stone 241.3 scope creep)
- `parse_ret_clause` minted (BRIEF discipline violation)
- Clippy warnings > 905
- Any prior arc 237 probe regression

**Mirror precedent: Stone 241.1.fix** shipped Layer 2 (~127 lines deleted across 4 files) in ~8 min Mode A. Stone 241.2 is broader (3 substrate sites + N test-assertion updates) but mechanical. The test-assertion cascade is the main runtime variable.

## Trap-door risks (enumerated; orchestrator watches)

| # | Risk | Detection | Resolution if hit |
|---|---|---|---|
| **T1** | Test-assertion cascade is bigger than expected (many tests assert against old error messages) | SCORE-STONE-241.2.md Honest Deltas section enumerates >10 updated assertions | Acceptable — surface as honest delta; document each update; the cascade IS the migration brief |
| **T2** | Sonnet doesn't capture args-vector span when destructuring (uses Span::unknown() or args[0].span() instead) | Inspect M1/M2/M3 bodies for span sourcing | Re-brief — span quality is load-bearing |
| **T3** | Sonnet mints `parse_ret_clause` helper despite BRIEF prohibition | grep `parse_ret_clause` in src/runtime.rs + src/check.rs | STOP-6 hit; re-brief |
| **T4** | Sonnet "fixes" T6 finding (type-keyword helper inconsistency) in Stone 241.2 scope | Inspect ret-clause inline blocks for changed parse_type_* calls | STOP-6 — surface as queued for future arc; do not unify in 241.2 |
| **T5** | Sonnet adds explicit `debug_assert!(spec.rest_param.is_none())` (D7 says optional; default omit) | Inspect for new debug_assert! macros | Acceptable either way; no STOP; document choice in SCORE |
| **T6** | Sonnet uses `wat::argspec::*` instead of `crate::argspec::*` from inside src/* | Inspect import paths | Style consistency check; both work but `crate::` is more conventional inside the crate; mild re-brief at most |
| **T7** | Sonnet alters the inline ret-clause check at A1/A2/A3 (BRIEF discipline says UNCHANGED) | Inspect the `match arrow_node` + `match ret_type_node` blocks at each site | Re-brief — ret-clause stays inline UNCHANGED |
| **T8** | Sonnet's `.into_iter().unzip()` lacks type annotation (inference ambiguity) | Compile error mentioning type annotation | Add `(Vec<String>, Vec<TypeExpr>)` annotation per BRIEF |
| **T9** | A test breaks STRUCTURALLY (not just message-string) — e.g., variant changed | Test failure not traceable to message-string assertion | STOP — surface as finding; investigate; this is a real regression |
| **T10** | Sonnet runs wrapper scripts or claims tool denial | Grep for wrapper invocation in output | False claim; FM 7 verification |

## Pre-spawn baseline checks (orchestrator runs BEFORE spawning)

1. **Behavioral-parity probe at HEAD = 10/10 PASS.** Verified this turn (probe just committed).
2. **Stone 241.1 probe at HEAD = 9/9 PASS.** Verified earlier this session.
3. **Lib baseline at HEAD = 834 PASS / 0 FAIL.** Verified earlier.
4. **Workspace test-build clean at HEAD.** Verified.
5. **Clippy baseline at HEAD = 905 warnings.**

## What completion looks like

### Phase A — SCORE scorecard verification + structural verification

After sonnet returns Mode A:
- 14/14 scorecard rows verify locally
- 10/10 structural-verification rows verify locally
- `SCORE-STONE-241.2.md` written with verbatim row results + honest deltas (especially error-message changes inventory)

### Phase B — Vigilia: NOT CAST

Per DESIGN D9: Stone 241.2 touches `src/runtime.rs` + `src/check.rs` — pre-existing flat substrate, NOT a namespaced home. The `feedback_namespaced_home_vigilia_gate` doctrine applies to `src/<noun>/` homes; the legacy flat codebase is per `feedback_ward_zone_comms_only` (wards-optional for the broader codebase).

**Stone 241.2 commits on SCORE-green.** No vigilia cast owed.

### Phase C — Commit + push (only after SCORE-green)

- Atomic commit covers: `src/runtime.rs`, `src/check.rs`, any test files with message-assertion updates, `SCORE-STONE-241.2.md`
- Push to origin
- **Phase 1 stone 2 of 4 LANDED.** Stone 241.3 (A4 defclause migration) opens; same canonical-routing pattern with `include_ret_type` = NOT APPLICABLE (defclause has no ret-clause; arity check differs from fn).

## Calibration history reference

| Stone | Class | Surface delta | Predicted | Actual | Status |
|---|---|---|---|---|---|
| 241.1 (parent) | Mint parser + types + tests | +519 net | 30-50 min | ~50 min | within band |
| 241.1.fix Layer 1 | Amend / extract / cleanup | -88 net | 20-30 min | ~8 min | UNDER (mechanical) |
| 241.1.fix Layer 2 | Scope correction strip | -127 net | 20-35 min | ~8 min | UNDER (mechanical) |
| 241.1.fix struere closure | 3-line amend | -3 net | 5-10 min | ~5 min | within |
| 241.2 (this) | Migration through canonical | -50 to -70 net + N test updates | 40-60 min | TBD | — |

Per `feedback_stone_briefs_cite_prior_score`: precedent informs the band; the N test-assertion cascade is the main runtime variable. If 241.2 ships substantially over 60 min, the cascade was larger than expected — surface as honest delta + calibration learning before Stone 241.3.

## What this unblocks

Stone 241.3 — A4 `parse_defclause_args` migration at `src/runtime.rs:6880`. Same canonical-routing pattern; defclause has NO ret-clause and NO `include_ret_type` semantics. After 241.3, all 4 fn/defclause parsers route through canonical; the parser-divergence class is closed.

Beyond Phase 1: 241.4's `&` rest-binder extension lands on the settled canonical API.
