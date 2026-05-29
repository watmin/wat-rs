# EXPECTATIONS — Stone 241.1.fix — vigilia-convergence + scope correction on `src/argspec/*`

Independent scorecard for orchestrator-side verification after sonnet returns. Each row is a fact to confirm via an explicit command; orchestrator re-runs locally and writes the verbatim result into `SCORE-STONE-241.1.fix.md`.

## Phase A — Scorecard (15 rows)

| Row | Claim | Verification command | Expected result |
|-----|---|---|---|
| 1 | Probe contract 01 PASS (empty argspec) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_01` | 1 passed; 0 failed |
| 2 | Probe contract 02 PASS (single fixed param) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_02` | 1 passed; 0 failed |
| 3 | Probe contract 03 PASS (multiple fixed params) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_03` | 1 passed; 0 failed |
| 4 | Probe contract 04 PASS (non-Symbol name) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_04` | 1 passed; 0 failed |
| 5 | Probe contract 05 PASS (missing arrow) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_05` | 1 passed; 0 failed |
| 6 | Probe contract 06 PASS (non-keyword type) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_06` | 1 passed; 0 failed |
| 7 | Probe contract 07 PASS (rest-binder rejected) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_07` | 1 passed; 0 failed |
| 8 | Probe contract 08 PASS (malformed type keyword) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_08` | 1 passed; 0 failed |
| 9 | Probe contract 09 PASS (incomplete triple) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_09` | 1 passed; 0 failed |
| 10 | Probe whole-suite PASS 9/9 | `cargo test --release --test probe_arc241_stone1_argspec_canonical` | 9 passed; 0 failed |
| 11 | Lib baseline preserved | `cargo test --release --lib -p wat` | 834 passed; 0 failed (or higher; never < 834) |
| 12 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0; 0 errors |
| 13 | Clippy delta = 0 | `cargo clippy --release 2>&1 \| grep -E "^warning:" \| wc -l` | ≤ 905 (pre-stone baseline) |
| 14 | Files touched match discipline | `git diff --name-only HEAD` | EXACTLY: `src/argspec/error.rs`, `src/argspec/mod.rs`, `src/argspec/parse.rs`, `tests/probe_arc241_stone1_argspec_canonical.rs`, `SCORE-STONE-241.1.fix.md` (the doc) |
| 15 | No prior arc 237 probe regresses | `cargo test --release --test probe_arc237_stone5_conforms --test probe_arc237_stone5fix_nominal --test probe_arc237_stone6_is_predicate --test probe_arc238_eq_completeness` | All-suite PASS counts preserved |

## Structural verification (after Phase A; orchestrator runs before Phase B)

| Verification | Command | Expected |
|---|---|---|
| `ret_type` field absent from `ArgSpec` | `grep -n "ret_type" src/argspec/parse.rs` | no matches |
| `include_ret_type` field absent from `ParseOptions` | `grep -n "include_ret_type" src/argspec/parse.rs` | no matches |
| `MissingRetArrow` variant absent | `grep -n "MissingRetArrow" src/argspec/error.rs` | no matches |
| `RetTypeNotKeyword` variant absent | `grep -n "RetTypeNotKeyword" src/argspec/error.rs` | no matches |
| `IncompleteTriple` variant present (renamed from `IncompleteSignature`) | `grep -n "IncompleteTriple" src/argspec/error.rs src/argspec/parse.rs` | matches in both files |
| `IncompleteSignature` variant absent | `grep -n "IncompleteSignature" src/argspec/error.rs src/argspec/parse.rs` | no matches |
| Loop break on `is_bare_symbol("->")` absent | `grep -n 'is_bare_symbol.*"->"' src/argspec/parse.rs` | no matches |
| `classify()` has exactly 7 match arms | `awk '/fn classify/,/^    }$/' src/argspec/error.rs \| grep -c "=>"` | 7 |
| `parse_keyword_type` still present (PRIVATE) | `grep -n "fn parse_keyword_type" src/argspec/parse.rs` | one match; not prefixed with `pub` |
| Two runes present in parse.rs | `grep -cE "rune:purgare\(future-fixture\)" src/argspec/parse.rs` | 2 |

## Independent prediction (runtime band)

**Target band: 20–35 min Mode A.**
**Upper bound: 40 min (STOP-3).**

**Mode B triggers** (any of these = re-brief, do not commit):
- Probe < 9/9 PASS at sonnet return
- Lib baseline < 834
- Clippy warnings > 905
- Files touched outside the discipline
- `src/lib.rs` touched
- Removed types/fields/variants persist (Phase A structural verification fails)
- Loop logic exhibits unexpected behavior (e.g., `->` at slot 0 doesn't surface as `MissingArrow`)
- Any prior arc 237 probe regression

**Mirror precedent: Stone 241.1.fix (Layer 1 alone)** shipped in ~8 min for ~88-line net delta with locked decisions. Stone 241.1.fix (Layer 1+2 combined) is mechanical strip on top of Layer 1's existing amends; ~+10-15 min for Layer 2 = 18-25 min total target. Adding contract restructure (rename + replacement) adds modest time. 20-35 min band.

## Trap-door risks (enumerated; orchestrator watches)

| # | Risk | Detection | Resolution if hit |
|---|---|---|---|
| **T1** | Sonnet KEEPS `ret_type` field (forgets S1) | Phase A structural row `ret_type field absent` fails | Re-brief — load-bearing S1 |
| **T2** | Sonnet KEEPS `include_ret_type` field (forgets S2) | Phase A structural row `include_ret_type field absent` fails | Re-brief — load-bearing S2 |
| **T3** | Sonnet KEEPS removed variants (`MissingRetArrow`, `RetTypeNotKeyword`) | Phase A structural rows fail | Re-brief |
| **T4** | Sonnet FORGETS to rename `IncompleteSignature` → `IncompleteTriple` | Phase A structural row `IncompleteSignature absent` fails | Re-brief |
| **T5** | Sonnet KEEPS the loop break on `is_bare_symbol("->")` (forgets the loop is now triple-only) | Grep finds the break | Re-brief |
| **T6** | Sonnet adds NEW types/fields/variants | Inspect diff for additions beyond the brief | STOP-6 — re-brief |
| **T7** | Sonnet mints `parse_ret_clause` (out of scope per STOP-6) | Grep for `parse_ret_clause` | STOP-6 — re-brief |
| **T8** | Sonnet DROPS contracts that should stay (e.g., contract for IncompleteTriple) | Probe < 9/9; missing contract files | Re-brief; contract list is locked |
| **T9** | Sonnet KEEPS old contract names (forgets to renumber) | `grep "contract_10\|contract_11\|contract_12\|contract_13" tests/probe...` | Re-brief — discipline says renumber |
| **T10** | Sonnet runs wrapper scripts or claims tool denial | Grep for wrapper invocation in output | False claim; FM 7 verification |

## Pre-spawn baseline checks (orchestrator runs BEFORE spawning)

1. **Lib baseline at HEAD = 834 PASS / 0 FAIL.** Verified at HEAD `9b3a9443` (from prior Phase A verification).
2. **Probe 13/13 PASS at HEAD with Layer 1 amends.** Verified earlier this session.
3. **Workspace test-build clean.** Verified.
4. **Clippy baseline = 905 warnings.** Captured at HEAD.
5. **Uncommitted state**: Layer 1 amends already applied to `src/argspec/error.rs`, `src/argspec/parse.rs`, `tests/probe_arc241_stone1_argspec_canonical.rs`. Sonnet builds Layer 2 on top.

## What completion looks like (TWO phases — Phase A = floor; Phase B = bar)

### Phase A — SCORE scorecard + structural verification (sonnet's correctness)

After sonnet returns Mode A:
- 15/15 scorecard rows verify locally
- 10/10 structural-verification rows verify locally
- `SCORE-STONE-241.1.fix.md` written with verbatim row results + honest deltas
- **DO NOT commit yet.** Phase A is the L0 floor. The bar is Phase B.

### Phase B — Vigilia re-cast on the namespaced home

Orchestrator casts **vigilia** on the amended `src/argspec/*` + amended probe. The 8 applicable spells (intueri / solvere / purgare / struere / sequi / temperare / complectens / vocare) in parallel.

**Expected verdict shift from prior cast:**
- **solvere**: L2 (RetTypeNotKeyword conflation) VANISHES structurally — variant gone; conflation impossible
- **complectens**: L2 (per-helper test deferral) may persist — rune-accept if so per acceptable-deferral history
- Other 6 spells: previously CONVERGED; should remain CONVERGED

**Bar:** L1 + L2 = 0. L2 mumbles MAY be rune-accepted only with load-bearing reasons (`rune:<spell>(<category>) — <reason>`).

If vigilia finds NEW issues: address before commit; re-cast; iterate until L1+L2=0.

### Phase C — Commit + push (only after Phase A + Phase B both green)

- SCORE doc amended with a **Vigilia Convergence** section listing each spell's verdict + any runes accepted
- Atomic commit covers: `src/argspec/error.rs`, `src/argspec/mod.rs`, `src/argspec/parse.rs`, `tests/probe_arc241_stone1_argspec_canonical.rs`, `SCORE-STONE-241.1.fix.md`
- Push to origin
- **Phase 1 foundation IMPECCABLE AND CORRECTLY SCOPED.** Stone 241.2 (migrate A1/A2/A3 fn parsers) begins on a foundation that's honest about its scope.

User direction governing this two-phase structure: *"we raise the bar fucking high for namespaced wat-rs files... we do not move from those until we are exceptional."* + *"args have nothing to do with ret type"* (the scope correction verdict).

## Calibration history reference

| Stone | Class | Surface delta | Actual runtime | Calibration accuracy |
|---|---|---|---|---|
| 241.1 (parent) | Mint parser + types + tests | +519 net | ~50 min | within 30-50 min target |
| 241.1.fix Layer 1 (prior strike) | Amend / extract / cleanup | -88 net | ~8 min | UNDER 20-30 min band |
| 241.1.fix Layer 1+2 (this) | Layer 1 + scope correction strip | -240 net (vs Stone 241.1 baseline) | TBD | predict: 20-35 min |

The net negative line count expresses the discipline: significant savings from classify() collapse + scope correction strip. The home becomes smaller AND more honest about its scope.

## What this unblocks

Stone 241.2 — A1/A2/A3 migration. Fn-form parsers compose:
1. Split args_vec at `->` arrow position
2. Call `parse_argspec_triples(prefix_slice, head, form_span, options)` for the args
3. Parse ret-clause on suffix (inline OR via Stone 241.2's helper)

Argspec is exceptional. Ret-clause is fn-form-parser concern. The substrate's structure honestly reflects the user's canonical form.
