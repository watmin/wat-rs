# EXPECTATIONS — Stone 241.10 — `src/remedy/` + ranked-remedy schema

Independent scorecard. **VIGILIA-GATED** per `feedback_namespaced_home_vigilia_gate` and user-direction "must be remarkable." L1+L2=0 convergence required pre-commit. Upper bound 240 min — extended for vigilia cycles + schema cascade.

## Phase A — Scorecard (12 rows)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | Probe contracts 01-03 PASS (typo + retirement remedy production) | `cargo test ... contract_0[1-3]` | 3/0 |
| 2 | Probe contract 04 PASS (distant-unknown → no remedy) | `cargo test ... contract_04` | 1/0 |
| 3 | Probe contracts 05-07 PASS (Display formatting; single/multi/retirement-annotation) | `cargo test ... contract_0[5-7]` | 3/0 |
| 4 | Probe contract 08 PASS (threshold filter) | `cargo test ... contract_08` | 1/0 |
| 5 | Probe whole-suite 8/8 | `cargo test --release --test probe_arc241_stone10_remedy` | 8/0 |
| 6 | Stone 241.9 probe preserved | `cargo test --release --test probe_arc241_stone9_defenum` | 8/0 |
| 7 | Stone 241.8 probe preserved | `cargo test --release --test probe_arc241_stone8_defstruct` | 8/0 |
| 8 | Stone 241.1-241.7 probes + arc 237/238 probes preserved | each | counts preserved |
| 9 | Lib baseline (post-cascade-migration) | `cargo test --release --lib -p wat` | 834 PASS / 0 FAIL |
| 10 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0 |
| 11 | Clippy delta ≤ 0 | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 902 |
| 12 | `src/lib.rs` adds `pub mod remedy;` only (no other diff) | `git diff src/lib.rs` | single line addition |

## Phase B — Vigilia convergence (8 spells; L1+L2=0 required)

Per `feedback_namespaced_home_vigilia_gate` and arc 241 Stone 241.1.fix precedent.

| Spell | Concern | Acceptance | Notes |
|---|---|---|---|
| **intueri** | `src/remedy/*.rs` names speak | L1+L2=0 | mod/distance/retirement/rank read as their content; `Remedy`/`RemedyKind` field names speak |
| **solvere** | No braided concerns | L1+L2=0 | distance.rs holds ONLY Levenshtein; retirement.rs holds ONLY the table; rank.rs holds ranking-shape logic; mod.rs orchestrates |
| **purgare** | No dead code | L1+L2=0 | every public API used; no unused helper functions |
| **struere** | Structure mirrors discipline | L1+L2=0 | file layout matches the 4 concerns; type definitions in mod.rs; algorithm in distance.rs |
| **sequi** | Imports follow domain | L1+L2=0 | no unrelated imports; only std + crate-local |
| **temperare** (always-apply) | No magic numbers without doc | L1+L2=0 | threshold formula documented; top-N=5 documented; Wagner-Fischer table size justified |
| **complectens** | Test shape (for `tests/remedy/*` if minted) | L1+L2=0 | hermetic tests; single-concern per test; no test-helper sharing without rune |
| **vocare** | Caller-perspective tests | L1+L2=0 | tests verify caller-visible outputs (the structured remedies + Display strings), not internal state |

Amend cycles allowed. If 3 cycles + still divergent on any spell, STOP-11 (escalate to orchestrator).

## Structural verification (9 rows)

| Verification | Command | Expected |
|---|---|---|
| `src/remedy/mod.rs` exists | `ls src/remedy/mod.rs` | exists |
| `src/remedy/distance.rs` exists | `ls src/remedy/distance.rs` | exists |
| `src/remedy/retirement.rs` exists | `ls src/remedy/retirement.rs` | exists |
| `src/remedy/rank.rs` exists | `ls src/remedy/rank.rs` | exists |
| `Remedy` struct present | `grep -n "pub struct Remedy" src/remedy/mod.rs` | 1 match |
| `RemedyKind` enum present with Typo+Retirement | `grep -nA3 "pub enum RemedyKind" src/remedy/mod.rs` | both variants present |
| `nearest_match` public API | `grep -n "pub fn nearest_match" src/remedy/mod.rs` | 1 match |
| `hint: Option<String>` retired on error variants | `grep -nE "hint: Option<String>" src/{types,check}.rs` | 0 matches |
| `remedies: Vec<Remedy>` ADDED on error variants | `grep -nE "remedies: Vec<Remedy>" src/{types,check}.rs` | ≥ 3 matches |

## Prediction: 120–180 min Mode A

Larger than 241.8 (~41 actual) and 241.9 (TBD) because schema change ripples + vigilia gate adds verification cycles. The cycle pattern from 241.1.fix:
- Initial mint: 60-90 min
- Vigilia cast: 15-30 min per spell (8 spells)
- Amend cycles: 5-15 min per finding
- Cascade hint-asserting tests: 30-60 min

Per `docs/SUBSTRATE-AS-TEACHER.md`: fail-count IS the progress meter. Initial schema-change fail-count after substrate edit: ~20-50 (hint-asserting tests + Display format tests). After mechanical migration: 0.

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | Cascade migration time-out (>240 min) | wall clock | STOP-3; orchestrator decides re-spawn (β-style) |
| **T2** | `hint:` field has more users than `src/{types,check}.rs` | `grep -rn "\.hint\b\\|hint:" src/` post-S2 | Migrate per substrate-as-teacher; document in SCORE |
| **T3** | Tests asserting on legacy hint-string content | grep + test inventory | Migrate the TESTS to remedies-field or Display-substring checks; preserve test INTENT |
| **T4** | Wat source files with error-message expectations | `grep -rn "hint:" wat-tests/ wat/` | If only Display matters, substring-match the new format |
| **T5** | Levenshtein cost at large candidate sets | profile (if needed) | Early-exit on threshold-exceeded per D10 (lazy); future stone for performance if surfaced |
| **T6** | Sonnet introduces `Option<Vec<Remedy>>` schema | grep | STOP-12 (`feedback_no_semantic_abuse_of_option`); re-brief |
| **T7** | Sonnet adds unknown-form-head rejection (scope expansion) | grep for new top-level classification arms | STOP-6 (scope creep); 241.10 only wires into EXISTING error paths |
| **T8** | Vigilia divergent after 3 amend cycles | per-spell tracker | STOP-11 (escalate) |
| **T9** | Retirement table grows beyond arc 241 retirements | grep retirement.rs | STOP-6 (D6 violation; arc 241's entries only this stone) |

## Pre-spawn baseline checks (post-241.9-ship)

1. Stone 241.10 probe at HEAD (post-241.9) = 2/8 PASS (C04 + C08 trivially; the 6 disconfirming contracts target structured-remedy format that doesn't exist yet)
2. Lib at HEAD = 834 PASS / 0 FAIL
3. All Stone 241.x probes + arc 237/238 probes preserved at current counts
4. Clippy ≤ 902

## What completion looks like

### Phase A — SCORE scorecard + structural verification

After sonnet returns Mode A:
- 12/12 scorecard verifies locally
- 9/9 structural rows verify locally

### Phase B — Vigilia 8/8 CONVERGED (L1+L2=0 each)

The vigilia table per-spell carries either:
- "PASS — no L1/L2 findings" (clean cast)
- "CONVERGED after N amend cycles — final state L1+L2=0" (with cycle log)

### Phase C — Commit + push

- Atomic commit covers: `src/remedy/*` (4 new files), `src/types.rs`, `src/check.rs`, `src/runtime.rs` (if hint usage), `src/lib.rs` (single-line add), hint-asserting test migrations, SCORE doc
- Push to origin
- Stone 241.11 (`define ⇒ defn` HARD CUT) opens next; consumes remedy infrastructure

## Calibration history reference

| Stone | Class | Predicted | Actual |
|---|---|---|---|
| 241.6 | Metadata-map storage | 25-45 min | ~28.8 min (within band) |
| 241.7 | Reflection verb | 15-30 min | ~19.4 min (within band) |
| 241.8 | defstruct HARD CUT + 33-site cascade | 60-120 min | ~41 min (UNDER band) |
| 241.9 | defenum HARD CUT + cascade | 60-120 min | TBD (in-flight at draft time) |
| **241.10 (this)** | **`src/remedy/` mint + schema upgrade + cascade + 8-spell vigilia** | **120-180 min** | **TBD** |

Stone 241.10 is the most substantial of arc 241's substrate-mint stones — namespaced home (4 files) + schema cascade + Display update + vigilia cycle. Same atomic-commit discipline (HARD CUT on `hint:` field).

## What this unblocks

**Stone 241.11** — `define ⇒ defn` HARD CUT. With remedy live, the HARD CUT lands on a substrate that TEACHES — every `:wat::core::define` typo'd or stale surfaces `did you mean: :wat::core::defn [retirement replacement]`. Bandaid-rip with receipts.

**Stone 241.12** — INSCRIPTION closes arc 241.

**Arc 237.8b** — reopens after 241.12 per `feedback_no_regression_until_arc_done`.

**Future arcs that consume remedy infrastructure:**
- Any HARD CUT registers its retirement entry; substrate self-documents evolution
- LLM-agent-facing structured error consumers; IDE quick-fix integration; telemetry on remedy acceptance
- Convergence #18 candidate (Lisp condition-system) inscribed in INTERSTITIAL on ship
