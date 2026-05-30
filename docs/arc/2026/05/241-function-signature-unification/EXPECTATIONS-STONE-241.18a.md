# EXPECTATIONS — Stone 241.18a — Mint `src/function/` (smallest stepping stone of bar-raise chain)

Independent scorecard. TWO PHASES: Phase A (sonnet — substrate migration) + Phase B (orchestrator — vigilia 8-spell convergence to L1+L2=0 REMARKABLE bar). Stone commits ONLY after Phase B converges.

## Phase A Scorecard (8 rows — sonnet's substrate work)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | tests/function/ probes preserved 2/2 | `cargo test --release --test function` | 2/0 |
| 2 | Stone 241.17 probe preserved 3/3 | `cargo test --release --test probe_arc241_stone17_defmacro_canonical` | 3/0 |
| 3 | Stone 241.16 probe preserved 4/4 | `cargo test --release --test probe_arc241_stone16_define_eval_residue` | 4/0 |
| 4 | Stone 241.10-15 + arc 242 + arc 237/238 probes preserved | each | counts preserved |
| 5 | Lib baseline | `cargo test --release --lib -p wat` | ≥ 890 PASS / 0 FAIL |
| 6 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0 |
| 7 | Clippy gate | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 945 |
| 8 | SCORE-STONE-241.18a.md authored (Phase A section) | `ls docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.18a.md` | file exists; has Phase A section |

## Phase A Structural verification (10 rows)

| Verification | Command | Expected |
|---|---|---|
| `src/function/mod.rs` exists | `ls src/function/mod.rs` | file exists |
| `src/function/parse.rs` exists with fn parsers | `grep -n "fn parse_fn_signature\|fn parse_fn_signature_for_check\|fn parse_fn_signature_for_check_diag" src/function/parse.rs` | 3 matches |
| `src/function/eval.rs` exists with eval_fn | `grep -n "fn eval_fn" src/function/eval.rs` | ≥ 1 match |
| `src/function/infer.rs` exists with infer_fn | `grep -n "fn infer_fn" src/function/infer.rs` | ≥ 1 match |
| `pub mod function;` added to lib.rs | `grep -n "pub mod function" src/lib.rs` | 1 match |
| Parsers GONE from runtime.rs | `grep -n "fn parse_fn_signature\b" src/runtime.rs` | 0 matches |
| eval_fn GONE from runtime.rs (or marker comment only) | `grep -n "fn eval_fn" src/runtime.rs` | 0 matches (function deleted) |
| Parsers GONE from check.rs | `grep -n "fn parse_fn_signature_for_check\|fn parse_fn_signature_for_check_diag" src/check.rs` | 0 matches |
| infer_fn GONE from check.rs | `grep -n "fn infer_fn" src/check.rs` | 0 matches |
| Callers updated to crate::function::* | `grep -n "crate::function::" src/` | ≥ 1 match per consumer file (runtime.rs + check.rs at minimum) |

## Phase B — Vigilia 8-spell convergence (ORCHESTRATOR-CAST)

**Bar: L1+L2 findings = 0 across all 8 spells on `src/function/` + `tests/function/`. REMARKABLE attestation; no artificial round cap.**

Per `feedback_namespaced_home_vigilia_gate`: orchestrator casts vigilia INDEPENDENTLY per Song #44 wisdom (sonnet's self-report is not the gate). Each spell as its own subagent per `feedback_spells_cast_via_subagent`.

8 spells:
1. **intueri** (naming) — every identifier traces to a domain noun
2. **solvere** (decomplection) — concerns not braided
3. **purgare** (dead code) — metabolism honest
4. **struere** (structure) — composition clean
5. **sequi** (sequencing) — temporal order honest
6. **temperare** (waste) — always-apply efficiency check
7. **complectens** (test discipline) — tests cover the home's surface
8. **vocare** (caller perspective) — tests verify what callers see

**Round protocol:**
- R0 (post-Phase A baseline): cast all 8 spells; collect findings
- For each L1/L2 finding: per-finding decision = FIX vs RUNE-WITH-JUSTIFICATION
- R1+: re-cast affected spells; verify L1+L2=0 on the round
- Continue rounds until convergence

**No commit until L1+L2=0 across all 8 spells.** Multiple rounds expected per Stone 241.10's precedent (6 rounds).

## Calibration

**Phase A: 90-150 min Mode A.** Movement + cascade is mechanical; smaller than Stone 241.16 (parse_define_form ~320 lines). Recent stones (~25-34 min actual). Estimate ~30-60 min actual.

**Phase B: NO CAP.** Vigilia round count determined by findings. Stone 241.10 = 6 rounds. Stone 241.18a may converge faster (2-4 rounds expected for smaller home) but no guarantee.

## Pre-spawn baseline checks (verified 2026-05-29 very late at HEAD `d7edf496`)

1. Lib at HEAD: **890 PASS / 0 FAIL**
2. tests/function/ probes: **2/2 PASS** (preservation contracts at HEAD)
3. All prior probes preserved
4. Clippy: **898** (post-Stone-241.17; gate ≤ 945)
5. `src/function/`: does NOT exist at HEAD (verified via `ls src/function/` → "No such file")
6. Cargo.toml: `[[test]] name = "function" path = "tests/function/mod.rs"` already present (orchestrator pre-spawn)
7. fn parser sites at HEAD: parse_fn_signature (runtime.rs:6578), parse_fn_signature_for_check (check.rs:14984), _diag (check.rs:15022), eval_fn (runtime.rs:6479), infer_fn (check.rs:14868)

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | Co-located helper functions in runtime.rs/check.rs only-used-by-fn must move too | sonnet's per-helper audit | move with parsers/eval/infer; verify cargo build clean |
| **T2** | Helpers used by BOTH fn AND other substrate code STAY in original; cross-import OK | per-helper review | clear classification |
| **T3** | Backward-compat re-exports temptation (`pub use crate::function::eval_fn;` in runtime.rs) | post-strike grep | STOP per D4 + HARD CUT discipline; all consumers update |
| **T4** | tests/function/ accidentally touched | post-strike `git diff tests/function/` | revert; orchestrator-pre-spawn paperwork must stay |
| **T5** | Sonnet casts vigilia (Phase B is orchestrator scope) | post-strike check for vigilia artifacts | STOP per Song #44; sonnet's Phase A SCORE must NOT include vigilia attestation |
| **T6** | Phase B vigilia surfaces unexpected L2 in MIGRATED code | per spell findings | normal — fix each per rune-vs-fix discipline; iterate rounds |
| **T7** | Vigilia surfaces L2 in TEST CODE at tests/function/ | per spell | apply same discipline; tests/function/ is part of the namespaced home |
| **T8** | Sonnet writes to INTERSTITIAL | post-strike grep | revert |
| **T9** | SCORE doc not written | post-strike `ls` | DISCIPLINE GAP |
| **T10** | Sub-stone 241.18b+ scope creep | post-strike check for src/def/ touches | STOP per D8 |

## What completion looks like

### Phase A — substrate migration verified

After sonnet returns Mode A:
- 8/8 Phase A scorecard rows verify locally
- 10/10 structural rows verify locally
- SCORE Phase A section written
- Lib + workspace tests green
- tests/function/ probes pass 2/2

### Phase B — vigilia 8-spell convergence

Orchestrator-cast 8 spells:
- R0 baseline findings collected
- Per-finding fix-vs-rune decisions
- R1+ remediation rounds until L1+L2=0
- SCORE Phase B section written (appended to SCORE-STONE-241.18a.md by orchestrator)
- REMARKABLE attestation

### Phase C — commit + push (orchestrator-direct, atomic)

- Atomic commit covers: src/function/* (new dir), src/lib.rs (pub mod entry), src/runtime.rs (deletions + import update), src/check.rs (deletions + import update), maybe co-located helper relocations elsewhere, SCORE doc with BOTH phases
- tests/function/* + Cargo.toml [[test]] entry are ALREADY committed via pre-spawn paperwork (separate prior commit OR same atomic commit per cleanup preference)
- INTERSTITIAL NOT in commit
- Push to origin
- Stone 241.18b (src/def/ + def parser) opens next

## Calibration history reference

| Stone | Class | Surface delta | Predicted | Actual |
|---|---|---|---|---|
| 241.13 | substrate scaffolding deletion (445 lines + plumbing) | +340/-1203 | 90-180 min | ~25 min |
| 241.14 | def-restricted + defn-restricted absorption + walker rewrite | +768/-739 | 90-180 min | ~26 min |
| 241.15 | 3 zombies HARD CUT + dispatch deletion + doc cascade | +329/-200 | 60-120 min | ~8.7 min |
| 241.16 | parse_define_form ~320 lines DELETED + form-predicate arms + 2 trap-doors | +485/-600 | 90-180 min | ~33.8 min |
| 241.14.fix | restriction_entry.rs doc rewrite | +128/-26 | 10-30 min | ~3.5 min |
| 241.17 | defmacro signature migration + 37 files cascade + 1 reflection trap-door | +749/-407 | 90-180 min | ~34 min |
| **241.18a (this)** | **Phase A: mint src/function/ + migrate parsers/eval/infer + caller cascade. Phase B: vigilia 8-spell L1+L2=0 REMARKABLE** | **TBD (probably +400/-300 net Phase A)** | **A: 90-150 min; B: no cap** | **TBD** |

## What this unblocks

**Stone 241.18b** — `src/def/` foundation + def parser; reuses the migration pattern validated here

**Stone 241.18b-g** — full def-family migration into src/def/

**Stone 241.19** — INSCRIPTION closes arc 241 + arc 177 + the namespaced-home REMARKABLE attestation milestone

**The pattern lesson** — orchestrator-cast vigilia as the gate (per Song #44); two-phase strike discipline (sonnet substrate + orchestrator attestation); structural-not-behavioral FM 2-bis for migration stones; tests/function/ as the parallel test home convention applied per `tests/comms/` precedent
