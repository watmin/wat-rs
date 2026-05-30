# EXPECTATIONS — Stone 243.3 R3 sweep

Independent scorecard for the R3 sweep that closes the remaining 6 of 12 R2 vigilia findings. Companion to `BRIEF-STONE-243.3-R3.md`.

## Phase A — substrate refactor scorecard (10 rows)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | R3.1 landed: RegistrationPrivilege enum replaces bool flag | `grep -nE "enum RegistrationPrivilege \{" src/types.rs` | 1 match |
| 2 | R3.1 call sites updated | `grep -nE "RegistrationPrivilege::(User\|Stdlib)" src/types.rs` | ≥ 2 matches |
| 3 | R3.1 bool flag retired | `grep -nE "bypass_prefix_gate" src/types.rs` | 0 matches |
| 4 | R3.2 landed: closure-threaded helper extracted | `grep -nE "^fn splice_type_decls<F>" src/types.rs` | 1 match |
| 5 | R3.2 thin wrappers in place | `grep -nE "splice_type_decls\(form, env, &\|" src/types.rs` | ≥ 2 matches |
| 6 | R3.3 landed: register_defclause_from_form helper extracted | `grep -nE "^fn register_defclause_from_form" src/check.rs` | 1 match |
| 7 | R3.3 callers thread through helper | `grep -nE "register_defclause_from_form\(form, env" src/check.rs` | ≥ 2 matches |
| 8 | R3.3 duplicated parse_defclause_form + build-clauses-Vec block reduced | `grep -cE "cl.args.iter\(\).map\(\|" src/check.rs` | reduced from baseline (≤ baseline-1) |
| 9 | Workspace test-build clean post-sweep | `cargo build --release --tests --workspace` | exit 0 |
| 10 | No new deferral language in stone-touched code | `grep -nE "future arc\|outside scope\|would require\|intentionally\|TODO" src/types.rs src/check.rs` (in NEW/CHANGED lines only) | 0 matches in stone-touched code |

## Structural verification (8 rows)

| Verification | Command | Expected |
|---|---|---|
| Lib baseline preserved | `cargo test --release --lib -p wat 2>&1 \| tail -3` | ≥ 890 PASS / 0 FAIL |
| tests/function preserved | `cargo test --release --test function 2>&1 \| tail -3` | 8 / 0 |
| FM 2-bis probe preserved | `cargo test --release --test probe_arc243_stone3_typeerror_pattern_a 2>&1 \| tail -3` | 3 / 0 |
| Clippy gate | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 897 |
| No backward-compat aliases | `grep -nE "type bypass_prefix_gate\|fn splice_type_decls_user_old\|fn splice_type_decls_stdlib_old" src/` | 0 matches |
| R3.1 enum is module-private (does not escape) | `grep -nE "^pub enum RegistrationPrivilege" src/types.rs` | 0 matches |
| R3.1 reserved-prefix gate still fires for User privilege | manual: lib test for ReservedPrefix error path passes | covered by lib test count = 890/0 |
| R3.3 caller semantics preserved (idempotent vs overwrite) | sonnet's return paragraph names the semantic verified | report present + matches existing behavior |

## R3.7 / R3.8 / R3.9 — conditional outcomes

Three inspection-and-conditional-rewrite tasks. Sonnet's return paragraph must name the path taken for each:

| Fix | Acceptable outcomes |
|---|---|
| **R3.7** (parse_type_expr redundant comment) | (a) "found redundant comment at line N; deleted" with diff cite; OR (b) "no redundant comment found; rune at 2859 is the only WHY" — both acceptable; report which |
| **R3.8** (types.rs:30-36 Scope notes) | (a) "058 Track 2 tracker verified to exist at <path>; left unchanged"; OR (b) "058 Track 2 tracker does NOT exist; rewrote closing sentence to <new text>"; OR (c) "Scope notes already present-tense; no edit" — three acceptable paths; report which |
| **R3.9** (check.rs:1680-1710 hint-extensibility) | (a) "found forward-promise phrasing at line N; rewrote to present-state"; OR (b) "docstring already present-tense; no rewrite needed" — both acceptable; report which |

## Calibration

**Predicted band: 30-60 min Mode A.** Six mechanical fixes (3 refactors + 3 inspections). Comparable to Stone 241.18a R3.4/R3.5/R3.6/R3.7 micro-fix sweeps (~10-15 min each in clean cases; ~30 min when inspection surfaces additional work).

**Time-box cap (orchestrator-side ScheduleWakeup): 60 min (the band's upper bound; 2× is impractical for the mechanical surface).** If exceeded → orchestrator TaskStop + score as Mode B-time-violation per recovery doc § 7.

## Pre-spawn baseline (verified at HEAD `93760ecc`, substrate at checkpoint `48d3393e`)

- Lib: 890 PASS / 0 FAIL
- tests/function: 8 / 0
- probe_arc243_stone3_typeerror_pattern_a: 3 / 0
- Workspace test-build: clean (exit 0)
- Clippy: 897
- All 6 fixes' sites verified at the line numbers cited in BRIEF (R3.1 site 280-334; R3.2 site 1742-1858; R3.3 site 10043-10121; R3.7 site 2840-2880; R3.8 site 30-36; R3.9 site 1660-1720)
- R3.4/R3.5/R3.6/R3.10/R3.11/R3.12 verified landed-or-reversed in checkpoint per commit message

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | R3.3 caller semantic divergence (collect_splice_defs_ctx may want non-idempotent; preregister_defclause_in_env wants idempotent) | Read both call sites' surrounding comments; verify which semantic each requires | Helper takes `idempotent: bool` param; each caller passes their semantic; sonnet's return cites which path each takes |
| **T2** | R3.2 closure threading conflicts with TypeEnv mutation borrow | rustc surfaces lifetime/borrow error; substrate-as-teacher names the site | Rework closure signature to take `&mut TypeEnv` explicitly; verify the original recursive call's borrow pattern; per types.rs:1769 and :1793 the recursion already threads `env` mutably |
| **T3** | R3.7 inspection finds the redundant comment is actually the rune itself (mistaken identity) | manual read of lines 2840-2880 | Per BRIEF: keep the rune (it's the structural why); only drop comments that REPEAT the rune's content |
| **T4** | R3.8 058 Track 2 tracker verification finds the path exists but no Track 2 reference inside | bash test on `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/` | Treat absence of explicit Track 2 reference as "tracker exists but Track 2 may be implicit/aspirational" — STOP, surface to orchestrator for triage rather than autonomous rewrite |
| **T5** | R3.9 inspection finds present-tense framing throughout | normal read of lines 1660-1720 | Report "no rewrite needed" path — acceptable outcome |
| **T6** | Additional R2-style findings surface in touched files (not in the named 6) | sonnet notices during inspection | Report HONESTLY in return paragraph per `feedback_pre_existing_is_not_exemption`; orchestrator triages rune-vs-FIX; do NOT silently skip and do NOT autonomously fix outside named scope |

## What completion looks like

### R3 sweep verified
- All 10 scorecard rows green
- All 8 structural verification rows green
- R3.7/R3.8/R3.9 outcome paths named in sonnet's return paragraph
- Lib + tests/function + probe + workspace + clippy gates all preserved
- No new deferral language; no backward-compat shims; no runes added

### Then (orchestrator-direct, post-strike)
- R2 vigilia re-cast (8 spells; corrected briefs — no skip-pre-existing instruction); iterate until L1+L2=0
- SCORE-STONE-243.3.md Phase B section authored (vigilia ledger + conformare attestation + final gates + doctrine reconciliation + R3 ledger reconciliation)
- Atomic commit: Stone 243.3 closure (R3 sweep + vigilia artifacts + SCORE Phase B)
- Push to `arc-170-gap-j-v5-deadlock-state`

### Calibration record
Compare actual runtime to predicted band (30-60 min); honest delta in SCORE Phase B if outside band; trap-door encounters logged.
