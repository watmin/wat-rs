# EXPECTATIONS — Stone 241.8 — defstruct HARD CUT

Independent scorecard. No vigilia (legacy flat substrate). SCORE-green commit. Larger upper bound (120 min) due to HARD CUT cascade.

## Phase A — Scorecard (11 rows)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | Probe contracts 01-05 PASS (defstruct success paths) | `cargo test ... contract_0[1-5]` | 5/0 |
| 2 | Probe contract 06 PASS (empty `{}` rejected) | `cargo test ... contract_06` | 1/0 |
| 3 | Probe contracts 07-08 PASS (legacy struct + struct-restricted HARD CUT rejected) | `cargo test ... contract_0[78]` | 2/0 |
| 4 | Probe whole-suite 8/8 | `cargo test --release --test probe_arc241_stone8_defstruct` | 8/0 |
| 5 | Stone 241.7 probe preserved 5/5 | `cargo test --release --test probe_arc241_stone7_metadata_of_reflection` | 5/0 |
| 6 | Stone 241.6 probe preserved 6/6 | `cargo test --release --test probe_arc241_stone6_def_metadata_map` | 6/0 |
| 7 | Stone 241.1-241.5 probes + Gate 1 preserved | each | counts preserved |
| 8 | Arc 237/238 probes preserved | each | counts preserved |
| 9 | Lib baseline (post-cascade-migration) | `cargo test --release --lib -p wat` | 834 PASS / 0 FAIL |
| 10 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0 |
| 11 | Clippy delta ≤ 0 | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 902 |

## Structural verification (6 rows)

| Verification | Command | Expected |
|---|---|---|
| `parse_defstruct` present | `grep -n "fn parse_defstruct" src/types.rs` | 1 match |
| `parse_struct` DELETED | `grep -n "fn parse_struct\b" src/types.rs` | 0 matches |
| `parse_struct_restricted` DELETED | `grep -n "fn parse_struct_restricted" src/types.rs` | 0 matches |
| Legacy `:wat::core::struct` arms DELETED from dispatch | `grep -n ':wat::core::struct"' src/types.rs` | 0 matches (only `:wat::core::struct-restricted` shouldn't appear either) |
| `:wat::core::defstruct` arm ADDED | `grep -n ':wat::core::defstruct' src/types.rs` | ≥ 1 match |
| Field-vector routes through canonical | `grep -n "parse_argspec_triples" src/types.rs` | ≥ 1 match (in parse_defstruct) |

## Prediction: 60–120 min Mode A

Larger than recent stones. Cascade migration is the runtime variable (~35 files). Substrate mint mechanical given DESIGN locks.

Per `docs/SUBSTRATE-AS-TEACHER.md`: fail-count IS the progress meter. Sonnet should:
- Initial fail-count after substrate change: ~30-50 (each legacy site = 1+ test failures)
- After mechanical migration: 0
- Per failure: ~30-90 seconds (read error → identify site → convert syntax → re-run)

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | Cascade migration time-out (>120 min) | wall clock | STOP-3; orchestrator decides re-spawn (β-style) |
| **T2** | Some legacy syntax doesn't have clean defstruct equivalent | sonnet surfaces honest delta | TRAP-DOOR doctrine: BUILD the missing piece; surface as new SCORE delta |
| **T3** | Tests that EXPLICITLY test arc 203 struct-restricted behavior | files like `wat_arc203_struct_restricted.rs` | Migrate the TESTS to defstruct equivalent; preserve test INTENT |
| **T4** | `parse_field` helper becomes orphaned after parse_struct deletion | grep shows no callers | DELETE per HARD CUT |
| **T5** | Some wat sources in `wat/*.wat` use legacy struct | failed runtime/eval tests | Migrate per Pattern A/B |
| **T6** | Sonnet introduces an alias arm (e.g., `:wat::core::struct` → "defstruct" routing) | grep | STOP-10 (HARD CUT violation); re-brief |

## Pre-spawn baseline checks

1. Stone 241.8 probe at HEAD = 3/8 PASS (FM 2-bis perfect — defstruct doesn't exist; legacy still works).
2. Lib at HEAD = 834 PASS / 0 FAIL.
3. All Stone 241.x probes at current counts.
4. Clippy at HEAD = 902.

## What completion looks like

### Phase A — SCORE scorecard + structural verification

After sonnet returns Mode A:
- 11/11 scorecard verifies locally
- 6/6 structural rows verify locally  
- SCORE doc written with cascade audit + honest deltas + PHASE 3 OPENS inscription

### Phase B — NOT cast (legacy flat substrate; no namespaced home minted)

### Phase C — Commit + push

- Atomic commit covers: src/types.rs, src/runtime.rs (if needed), src/check.rs (if needed), ~35 migration target files, SCORE doc
- Push to origin
- Stone 241.9 (defenum HARD CUT) opens next

## Calibration history reference

| Stone | Class | Predicted | Actual |
|---|---|---|---|
| 241.5 | Runtime dispatch + Gate 1 | 20-40 min | ~10 min |
| 241.6 | Metadata-map storage | 25-45 min | ~28.8 min |
| 241.7 | Reflection verb | 15-30 min | ~19.4 min |
| **241.8 (this)** | **defstruct HARD CUT + 35-site cascade** | **60-120 min** | **TBD** |

Stone 241.8 is substantially larger. Per the no-broken-commits + HARD CUT discipline: one atomic commit; do not split into shim + cleanup.

## What this unblocks

**Stone 241.9** — defenum HARD CUT (Phase 3 second stone).
**Stone 241.10** — `define ⇒ defn` HARD CUT (Phase 3 third stone).
**Stone 241.11** — INSCRIPTION closes the arc.
**Arc 237.8b** — reopens after 241.11.
