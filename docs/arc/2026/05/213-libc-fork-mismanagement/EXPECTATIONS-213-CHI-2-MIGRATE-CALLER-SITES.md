# Arc 213 stone χ-2 — EXPECTATIONS

## Independent prediction

- **Runtime band:** 60-90 min Mode A. ~35 caller sites + ~18 field types + ~22 factory calls + ~4 imports = ~75 mechanical edits across 4 files. Substrate-as-teacher cascade does the navigation; sonnet walks cargo errors.
- **LOC changed:** ~80-120 (most edits are token-level type-path swaps; some field-type lines may reformat)
- **New files:** 1 (SCORE doc)
- **Surprises expected:** LOW-MEDIUM. The wrapper's method signature parity (proven by χ-1's 24/24 baseline PASS) means most sites compile without further touch. The cascade-primitive exclusion is the main hand-execution risk — sonnet must NOT migrate SHUTDOWN_RX/TX lines in runtime.rs.

## Honest-delta watch

### Risk 1 — Caller uses crossbeam method the wrapper doesn't expose

The wrapper has: `send` (Sender), `recv` / `try_recv` (Receiver), `Clone` (both). Crossbeam offers MORE: `len`, `is_empty`, `capacity`, `iter`, `into_iter`, `recv_timeout`, `send_timeout`, etc.

If any of the 35 caller sites use a method not in our wrapper, cargo will fail with `no method named 'X' found for struct 'Sender' / 'Receiver' in scope`. Sonnet should STOP at first such error, write the SCORE noting which method + which site, NOT extend the wrapper. The decision to extend the wrapper (or refactor the caller) is a separate stone.

### Risk 2 — Cascade primitive accidentally migrated

`src/runtime.rs:179` SHUTDOWN_RX + `:185` SHUTDOWN_TX_PTR + `:233` init factory MUST stay bare. If sonnet's substrate-as-teacher cascade catches these as "migration targets," the result is circular dependency (wrapper queries SHUTDOWN_RX; SHUTDOWN_RX queries SHUTDOWN_RX). Sonnet's BRIEF lists these explicitly as DO-NOT-TOUCH; sonnet must verify each `crossbeam_channel::` reference in runtime.rs before editing.

### Risk 3 — Type inference / lifetime / Send+Sync bound differences

The wrapper's `Sender<T>` / `Receiver<T>` are simple newtypes; they should auto-derive Send + Sync for any T that crossbeam supports. But if existing callers depend on specific trait bounds that the newtype doesn't propagate, cargo may complain about `T: Send` etc. Likely-easy to fix (add `where T: Send` bounds on the wrapper if needed), but a Mode B surface if it requires non-trivial wrapper changes.

### Risk 4 — Pattern-matching against bare crossbeam types

If any caller pattern-matches `crossbeam_channel::SendError(value)` or similar, the wrapper's re-exports may not destructure identically. χ-1 re-exported `SendError`, `RecvError`, `TryRecvError` from crossbeam, so the variant types ARE the same — destructuring should work. If a caller imports `crossbeam_channel::SendError` directly and pattern-matches, the import needs to switch to `wat::typed_channel::SendError`.

### Risk 5 — Generic context interactions with the substrate-internal type registry

`src/types.rs` lines 922, 929 register `"rust::crossbeam_channel::Sender"` and `"rust::crossbeam_channel::Receiver"` as wat-visible type names. If the χ-2 migration also renames these registry entries (out of scope per BRIEF), wat programs that declare those types would break. BRIEF explicitly excludes types.rs from migration — verify no accidental edit.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `src/thread_io.rs` import line migrated (`use crate::typed_channel::{Receiver, Sender}`) | YES |
| 2 | `src/thread_io.rs` ALL `crossbeam_channel::{Sender,Receiver,unbounded,bounded}` references migrated to `crate::typed_channel::*` | YES |
| 3 | `src/runtime.rs` non-cascade-primitive `crossbeam_channel::{Sender,Receiver,unbounded,bounded}` references migrated | YES |
| 4 | `src/runtime.rs:179` `SHUTDOWN_RX` UNCHANGED (still bare crossbeam) | YES |
| 5 | `src/runtime.rs:185` `SHUTDOWN_TX_PTR` UNCHANGED (still bare crossbeam) | YES |
| 6 | `src/runtime.rs:233` `init_shutdown_signal` factory UNCHANGED (still bare crossbeam) | YES |
| 7 | `src/freeze.rs` `crossbeam_channel::*` migrated | YES |
| 8 | `src/spawn.rs` `crossbeam_channel::*` migrated | YES |
| 9 | `cargo build --release` clean | YES |
| 10 | `cargo test --release --test probe_channel_primitive` 3/3 PASS | YES |
| 11 | `cargo test --release --test probe_pidfd_primitive` 2/2 PASS | YES |
| 12 | NO touches to `src/typed_channel.rs` (the wrapper home) | YES |
| 13 | NO touches to `src/check.rs` / `src/lexer.rs` / `src/parser.rs` / `src/types.rs` (type-name strings, not callers) | YES |
| 14 | NO touches to `src/fork.rs` / `src/spawn_process.rs` (dirty tree δ-1) | YES |
| 15 | NO wat_arc170_program_contracts run during verification | YES |
| 16 | SCORE doc inscribes site-counts migrated per file + cargo build output + probe outputs | YES |
| 17 | Total `crossbeam_channel::` references outside typed_channel.rs / runtime.rs cascade lines / check.rs / lexer.rs / parser.rs / types.rs → ZERO post-migration | YES |

## Mode classification

- **Mode A:** all 17 criteria satisfied; 4 caller files migrated; cascade primitives preserved; build clean; probes still pass; wat_arc170 NOT re-run
- **Mode B (acceptable; honest surface):**
  - Risk 1 fires (caller uses crossbeam method wrapper doesn't have): sonnet STOPs + reports method + site; orchestrator decides
  - Risk 3 fires (trait bound surface): sonnet documents + STOPs
  - Risk 4 fires (pattern-match on imported error type): sonnet documents + STOPs
- **Mode C (failure):**
  - Touched any file outside the 4 caller files + SCORE doc
  - Touched the cascade primitives in runtime.rs
  - Touched the dirty tree (src/fork.rs / src/spawn_process.rs)
  - Ran wat_arc170_program_contracts (violates `feedback_no_hang_vector_in_additive_scorecard`)
  - Extended the wrapper unilaterally (added methods or factories without orchestrator decision)
  - Migrated type-name strings in check/lexer/parser/types (those are wat-visible names, not Rust callers)
  - Committed the work (orchestrator commits)

## Calibration metadata

- **Orchestrator confidence:** MEDIUM-HIGH on Mode A first-attempt. The mechanical pattern is uniform; substrate-as-teacher cascade has shipped 5+ stones successfully in arc 213's γ phase. The cascade-primitive exclusion is the main hand-execution risk; sonnet should grep `SHUTDOWN_` before editing any line in runtime.rs.
- **Risk factors:**
  - Cascade-primitive exclusion (Risk 2) is the highest discipline-cost; mitigated by explicit DO-NOT-TOUCH list in BRIEF
  - Method-coverage gap (Risk 1) is unknown until sonnet hits a site; Mode B is the honest reporting path
- **Why this matters:** χ-2 completes the migration → χ-3 (restricted_to wall) + χ-4 (50-trial proof) gate arc 213's η INSCRIPTION. After χ-2: every Rust-level T-typed channel in the substrate routes through the cascade-aware wrapper. The orphan-leak class becomes structurally impossible at this layer.

## Tractability tiebreaker rationale (per `feedback_tractability_tiebreaker`)

χ-1 minted → χ-2 migration is forced-next (the migration cannot precede the wrapper). Within χ-2 itself: ship as one stone because the file count is bounded (4), the mechanical pattern is uniform across files (per `feedback_simple_is_uniform_composition`), and sonnet's substrate-as-teacher cascade naturally drives file-by-file completion. No further splitting needed.

## Cross-references

- BRIEF-213-CHI-2-MIGRATE-CALLER-SITES.md — this stone's work-order
- SCORE-213-CHI-1-MINT-CHANNEL-WRAPPER.md — χ-1's verification (the wrapper sonnet migrates TO)
- INTERSTITIAL § 2026-05-18 (post-δ-1 investigation) "Channel-cascade-completeness wall" — doctrine
- `feedback_no_hang_vector_in_additive_scorecard` — why χ-2 does NOT verify via workspace tests
- `feedback_defect_fix_or_panic_never_revert` — why dirty tree stays untouched
- `feedback_simple_is_uniform_composition` — N identical edits = simple
