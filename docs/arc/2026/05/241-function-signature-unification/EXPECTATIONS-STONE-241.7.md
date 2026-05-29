# EXPECTATIONS — Stone 241.7 — mint `:wat::runtime::metadata-of`

Independent scorecard. No vigilia (legacy flat substrate). SCORE-green commit.

## Phase A — Scorecard (10 rows)

| Row | Claim | Verification | Expected |
|---|---|---|---|
| 1 | Probe contract_01 (def-with-metadata → Some) | `cargo test ... contract_01` | 1/0 |
| 2 | Probe contract_02 (defn-with-metadata → Some via fn-peel round-trip) | `cargo test ... contract_02` | 1/0 |
| 3 | Probe contract_03 (multi-entry → Some) | `cargo test ... contract_03` | 1/0 |
| 4 | Probe contract_04 (def-without-metadata → None) | `cargo test ... contract_04` | 1/0 |
| 5 | Probe contract_05 (unknown binding → None) | `cargo test ... contract_05` | 1/0 |
| 6 | Probe whole-suite 5/5 | `cargo test --release --test probe_arc241_stone7_metadata_of_reflection` | 5/0 |
| 7 | Stone 241.6 probe preserved 6/6 | `cargo test --release --test probe_arc241_stone6_def_metadata_map` | 6/0 |
| 8 | Stone 241.1/241.2/241.3/241.5 + Gate 1 preserved | each test | counts preserved |
| 9 | Lib + workspace test-build + clippy | standard | 834 / clean / ≤902 |
| 10 | `src/argspec/* + src/check.rs + src/lib.rs` UNCHANGED | `git diff` | empty |

## Structural verification (4 rows)

| Verification | Command | Expected |
|---|---|---|
| `eval_metadata_of` present | `grep -n "fn eval_metadata_of" src/runtime.rs` | 1 match |
| Dispatch entry present | `grep -n ":wat::runtime::metadata-of" src/runtime.rs` | ≥ 2 matches |
| binding_metadata read | `grep -n "binding_metadata.get" src/runtime.rs` | ≥ 1 (in metadata-of) |
| body-of UNCHANGED | `git diff src/runtime.rs \| grep "fn eval_body_of"` | no diff lines |

## Prediction: 15–30 min Mode A; 30 min STOP-3

Mirror Stone 241.5/241.6 single-substrate-file mints. Mostly mechanical pattern-mirror of body-of.

## What this closes

**Phase 2 of arc 241 CLOSES** (storage 241.6 + reflection 241.7). Phase 3 opens at Stone 241.8 (defstruct HARD CUT using the metadata-map mechanism).
