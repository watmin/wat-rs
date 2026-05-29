# EXPECTATIONS — Stone 241.11 — `:wat::core::define` ⇒ `:wat::core::defn` HARD CUT

Independent scorecard. NO vigilia required (D5 — legacy flat substrate; this stone consumes Stone 241.10's `src/remedy/` apparatus via single-line table append; does not mint a new namespaced home). SCORE-green commit. Upper bound 240 min — the LARGEST cascade in arc 241 (~271 sites).

## Phase A — Scorecard (11 rows)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | Probe contract 01 PASS (defn baseline) | `cargo test ... contract_01` | 1/0 |
| 2 | Probe contract 02 PASS (legacy define HARD CUT rejected) | `cargo test ... contract_02` | 1/0 |
| 3 | Probe contract 03 PASS (remedy names :wat::core::defn) | `cargo test ... contract_03` | 1/0 |
| 4 | Probe contract 04 PASS ([retirement replacement] annotation) | `cargo test ... contract_04` | 1/0 |
| 5 | Probe contract 05 PASS (retirement table has define→defn entry) | `cargo test ... contract_05` | 1/0 |
| 6 | Probe whole-suite 5/5 | `cargo test --release --test probe_arc241_stone11_define_hard_cut` | 5/0 |
| 7 | Stone 241.10 probe preserved 8/8 | `cargo test --release --test probe_arc241_stone10_remedy` | 8/0 |
| 8 | Stone 241.9 probe preserved 8/8 | `cargo test --release --test probe_arc241_stone9_defenum` | 8/0 |
| 9 | Stone 241.8 + 241.1-7 probes preserved + arc 237/238 probes preserved | each | counts preserved |
| 10 | Lib baseline (post-cascade-migration) | `cargo test --release --lib -p wat` | ≥ 890 PASS / 0 FAIL |
| 11 | Workspace test-build clean + clippy ≤ 902 | `cargo build --release --tests --workspace` + `cargo clippy --release` | exit 0; warnings ≤ 902 |

## Structural verification (8 rows)

| Verification | Command | Expected |
|---|---|---|
| `RETIREMENT_TABLE` has 4 entries | `grep -c '":wat::core::' src/remedy/retirement.rs \| head -1` | ≥ 4 LHS matches |
| `:wat::core::define` entry in retirement table | `grep -n '":wat::core::define"' src/remedy/retirement.rs` | 1 match (in RETIREMENT_TABLE) |
| HARD-CUT arm for `:wat::core::define` in check.rs | `grep -n '":wat::core::define"' src/check.rs` | ≥ 1 match (the new arm) |
| `register_defines` DELETED | `grep -n "fn register_defines\b" src/freeze.rs` | 0 matches |
| `register_stdlib_defines` DELETED | `grep -n "fn register_stdlib_defines\b" src/freeze.rs` | 0 matches |
| **`register_define_dispatches` PRESERVED** (arc 146; D4) | `grep -n "fn register_define_dispatches" src/dispatch.rs` | ≥ 1 match (still there) |
| **`parse_define_dispatch_form` PRESERVED** (arc 146; D4) | `grep -n "fn parse_define_dispatch_form" src/dispatch.rs` | ≥ 1 match |
| `crates/fix-defines/` DELETED (ephemeral) | `ls crates/fix-defines/ 2>&1` | "No such file or directory" |

## Prediction: 120–240 min Mode A

Substantially larger than all prior arc 241 stones combined. The cascade dominates runtime. Substrate work itself is mechanical (single-line retirement append + check.rs HARD-CUT arm + freeze.rs/dispatch.rs deletions).

Per `docs/SUBSTRATE-AS-TEACHER.md` + Stone 241.10's auto-fixer precedent:
- Initial fail-count after substrate change: hundreds (every define caller errors)
- Auto-fixer pass: drops fail-count by 90%+
- Manual residuals: 10-30 sites
- Per residual: ~30-90 seconds

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | Cascade migration time-out (>240 min) | wall clock | STOP-3; orchestrator decides re-spawn |
| **T2** | Pattern B (multi-arg define) requires arg names — auto-fixer can't extract from body | sonnet surfaces honest delta | Generate placeholder names + manual review per-site; document strategy in SCORE |
| **T3** | `:wat::core::define-dispatch` accidentally retired (D4 violation) | grep post-strike: `grep -n ":wat::core::define-dispatch" src/dispatch.rs` should still match | STOP-11 (D4 violation); restore dispatch machinery |
| **T4** | `crates/fix-defines/` survives the commit (D2 + T4 from DESIGN) | git status post-strike includes `crates/fix-defines/` | STOP-5 (D2 violation); delete before commit |
| **T5** | Wat-source files have `define` in comments or strings (not actual forms) | regex audit | Auto-fixer must distinguish; sonnet adds guard in fix-defines logic |
| **T6** | The check.rs:933 `:user::main` retirement prose pre-existing (arc 170) — duplication concern | sonnet judges | Either coexist (new arm handles all defines; old arm specifically for user::main signature shape); or merge (new arm subsumes) |
| **T7** | Probe contract 05 (retirement table structural proof) is indirect | per-contract review | Indirect proof via remedy text output is acceptable; alternative direct unit test in retirement.rs is also acceptable; sonnet judges |

## Pre-spawn baseline checks

1. Stone 241.11 probe at HEAD = 1/5 PASS (verified — C01 baseline passes; C02-C05 disconfirm cleanly).
2. Lib at HEAD = 890 PASS / 0 FAIL (Stone 241.10 R6 final state).
3. All Stone 241.x probes preserved at current counts.
4. Clippy 900 ≤ 902 gate.
5. `RETIREMENT_TABLE` has 3 entries (struct, struct-restricted, enum) at HEAD.

## What completion looks like

### Phase A — SCORE scorecard + structural verification

After sonnet returns Mode A:
- 11/11 scorecard verifies locally
- 8/8 structural rows verify locally
- SCORE doc written with cascade audit + auto-fixer story (built? used? deleted?) + honest deltas

### Phase B — NOT cast (no vigilia per D5)

### Phase C — Commit + push

- Atomic commit covers: `src/remedy/retirement.rs` (single-line append), `src/check.rs` (HARD-CUT arm), `src/freeze.rs` (register_defines deletion), `src/dispatch.rs` (define-only paths if any; define-dispatch paths PRESERVED), `src/runtime.rs` (if needed), 271 cascade target files, SCORE doc
- Push to origin
- Stone 241.12 (INSCRIPTION) opens next; arc 237.8b reopens after

## Calibration history reference

| Stone | Class | Predicted | Actual |
|---|---|---|---|
| 241.8 | defstruct HARD CUT + 33-site cascade | 60-120 min | ~41 min (UNDER band) |
| 241.9 | defenum HARD CUT + 33-site cascade + R-gap trap-door | 60-120 min | ~50 min (UNDER band) |
| 241.10 | src/remedy/ mint + schema HARD CUT + 160-site cascade + 6-round vigilia | 120-180 min ship + 6 rounds | ship within band + substantial post-ship work |
| **241.11 (this)** | **define HARD CUT + ~271-site cascade + auto-fixer authorized** | **120-240 min** | **TBD** |

Stone 241.11 is structurally simpler (no schema change; single-line retirement append; existing remedy infrastructure handles teaching) but cascade is 8× larger than prior. The auto-fixer precedent from 241.10 directly informs strategy.

## What this unblocks

**Stone 241.12** — INSCRIPTION closes arc 241. Pre-INSCRIPTION grep enforced (per FM 11 + Stone S11 of recovery doc).

**Arc 237.8b** — reopens after Stone 241.12 per `feedback_no_regression_until_arc_done`.

**Future HARD CUTs** — the bandaid-rip-with-receipts pattern is now THE pattern. Any future form retirement appends one line to RETIREMENT_TABLE; remedy infrastructure does the rest. Stone 241.11 is the first demonstrated consumer.
