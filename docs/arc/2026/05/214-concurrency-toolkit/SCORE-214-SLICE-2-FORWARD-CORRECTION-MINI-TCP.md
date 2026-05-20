# Arc 214 — Slice 2 forward-correction — SCORE

**Stone:** Mini-TCP at depth 1 (drop bounded; pair() at capacity 1)
**Date:** 2026-05-19
**Implementor:** claude-sonnet-4-6 (arc 214 Slice 2 forward-correction agent)
**Mode:** A (19/19 criteria satisfied)

## Build + test verification

```
cargo build --release
→ CLEAN (5 pre-existing dead_code warnings in check.rs/runtime.rs; 0 in comms)

cargo test --release --test comms -p wat -- thread
→ running 9 tests
→ test thread::probe_slice2_clone_receiver_exactly_one_gets_frame ... ok
→ test thread::probe_slice2_close_idempotent_with_clones ... ok
→ test thread::probe_slice2_pair_round_trip ... ok
→ test thread::probe_slice2_select_picks_fired_receiver ... ok
→ test thread::probe_slice2_select_indices_match_registration_order ... ok
→ test thread::probe_slice2_sender_drop_triggers_recv_err ... ok
→ test thread::probe_slice2_clone_sender_multi_producer ... ok
→ test thread::probe_slice2_try_recv_empty_returns_empty ... ok
→ test thread::probe_slice2_try_recv_disconnected_after_sender_drop ... ok
→ test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out

cargo test --release --test comms -p wat
→ running 33 tests (all pass; thread: 9; process: 15; foundation: 6 + 3 select)
→ test result: ok. 33 passed; 0 failed; 0 ignored

cargo test --release --workspace --no-fail-fast
→ 1 pre-existing compile failure: wat_arc170_slice_1f_alpha_helpers
  (crossbeam_channel::Sender vs wat::Sender type mismatch; verified pre-existing
   by git stash + test on unmodified tree; dirty-tree artifact, out of scope per BRIEF)
→ All other tests pass; no regressions introduced by this stone

grep -rn "comms::thread::bounded" --include="*.rs"
→ (no output — zero matches)
```

## LOC delta

- `src/comms/thread.rs`: module-level doc +18 lines (Mini-TCP section); pair() doc rewritten (-4/+8); bounded() fn deleted (-8); net ≈ +14
- `tests/comms/thread.rs`: module header updated; import line updated; bounded_round_trip deleted (-11); unbounded_round_trip renamed + comment updated (net -2)
- `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md`: ~150 lines changed/added (table, Rust-side types, section rewrite, Slice 2 forward-correction note, new subsection at end)
- `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md`: ~65 lines appended
- New file: `SCORE-214-SLICE-2-FORWARD-CORRECTION-MINI-TCP.md` (this document)

## 19-row scorecard

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | `src/comms/thread.rs` module-level doc gains "Mini-TCP at depth 1" section | PASS | Section added after cascade-contract paragraph, before audience; `grep "Mini-TCP at depth 1" src/comms/thread.rs` → 1 hit |
| 2 | `src/comms/thread.rs` `pair<T>` body uses `crossbeam_channel::bounded(1)` | PASS | `grep "crossbeam_channel::bounded(1)" src/comms/thread.rs` → 1 hit in pair() body |
| 3 | `src/comms/thread.rs` `pair<T>` doc-comment updated to name mini-TCP + capacity-1 | PASS | Doc comment: "Construct a mini-TCP thread-tier channel pair at depth 1. Capacity is 1..." + "(see module-level doc § 'Mini-TCP at depth 1')" |
| 4 | `src/comms/thread.rs` `bounded<T>` fn deleted entirely | PASS | `grep "pub fn bounded" src/comms/thread.rs` → 0 hits |
| 5 | `tests/comms/thread.rs` module-header doc updated ("Nine tests"; drops "unbounded + bounded") | PASS | "Nine tests covering: round-trip, sender-drop..." — "unbounded + bounded" phrase gone |
| 6 | `tests/comms/thread.rs` import line drops `bounded` | PASS | `use wat::comms::thread::{pair, Select};` — bounded not present |
| 7 | `tests/comms/thread.rs` `probe_slice2_unbounded_round_trip` renamed to `probe_slice2_pair_round_trip` | PASS | `grep "probe_slice2_pair_round_trip" tests/comms/thread.rs` → 1 hit; `grep "probe_slice2_unbounded_round_trip" tests/comms/thread.rs` → 0 hits |
| 8 | `tests/comms/thread.rs` `probe_slice2_bounded_round_trip` deleted entirely | PASS | `grep "probe_slice2_bounded_round_trip" tests/comms/thread.rs` → 0 hits |
| 9 | DESIGN.md Rust-side types listing drops `bounded` line; pair() comment updated | PASS | `pub fn pair<T>() -> (Sender<T>, Receiver<T>);  // capacity-1 mini-TCP (see DESIGN § "Mini-TCP at depth 1 — universal symmetry")`; `bounded` line gone from types listing |
| 10 | DESIGN.md § "Universe-residency + bounded() asymmetry" renamed + asymmetry #3 removed + symmetry inscribed | PASS | Section renamed to "Universe-residency + Mini-TCP at depth 1 (universal symmetry)"; asymmetry #3 removed; "Two substrate-internal asymmetries" replaces "Three"; new Mini-TCP subsection with four-questions verdict inscribed |
| 11 | DESIGN.md Slice 2 description gets forward-correction note (does NOT edit original body) | PASS | Original "Factories: `pair<T>()`, `bounded<T>(n)`" line preserved; forward-correction note appended below it |
| 12 | DESIGN.md gets new "Slice 2 forward-correction (2026-05-19) — Mini-TCP at depth 1" subsection at end | PASS | Subsection appended after the closing `---` + Arc OPENED line; four-questions verdicts + universal symmetry table + trading-lab origin + cross-references present |
| 13 | INTERSTITIAL gets new entry after line 6417 with mini-TCP origin + four-questions verdict + cross-refs | PASS | Entry appended after "See you on the other side..."; includes verbatim user quote + grep evidence + four-questions verdict + three-pivots tally + universal symmetry recognition + cross-references |
| 14 | INTERSTITIAL entry closes with the voice-anchor line ("the substrate dreams the depth...") | PASS | Closing line: "*the substrate dreams the depth. The substrate dreams 1. So do we.*" |
| 15 | `cargo build --release` clean | PASS | 0 errors; 5 pre-existing dead_code warnings in check.rs/runtime.rs; 0 in comms |
| 16 | `cargo test --release --test comms -p wat -- thread` shows exactly 9 tests; all pass | PASS | 9 passed; 0 failed; 24 filtered out (process + foundation tests) |
| 17 | `cargo test --release --workspace --no-fail-fast` shows workspace clean (no regressions outside the retired test) | PASS | `wat_arc170_slice_1f_alpha_helpers` fails — verified pre-existing via git stash on unmodified tree; not introduced by this stone. `probe_slice2_bounded_round_trip` is retired (deleted, not failing). All other tests pass. |
| 18 | `grep -rn "comms::thread::bounded" --include="*.rs"` returns zero matches | PASS | Zero output — factory truly gone; no orphan callers |
| 19 | SCORE doc inscribed with verification command output + honest-delta surfaces | PASS | This document; all verification commands run and output shown above; honest-delta for workspace run (pre-existing failure) documented |

## Honest deltas

**One BRIEF invocation clarification (Risk 7 — Cargo test invocation):**

The BRIEF specified `cargo test --release --test thread -p wat`. The test target is actually `comms` (the integration test module at `tests/comms/mod.rs` includes `tests/comms/thread.rs` as a submodule). The correct invocation is `cargo test --release --test comms -p wat -- thread` to filter to the thread module. Ran this and got exactly 9 tests passing.

**One pre-existing workspace failure (Risk 1-adjacent):**

`wat_arc170_slice_1f_alpha_helpers` fails with `crossbeam_channel::Sender` vs `wat::Sender` type mismatch — 6 errors, pre-existing dirty-tree artifact. Verified pre-existing by stashing all changes and confirming the same 6 errors appeared on the unmodified tree. Per BRIEF § "Out of scope": dirty tree untouched.

**DESIGN.md — two additional sites updated beyond the 3 explicitly spec'd:**

The Layer 0a table row (`pair`, `bounded`) and the architecture diagram (`Layer 0a — Comms tier primitives` listing) both referenced `bounded`. Updated both to be consistent with the retired factory. These were caught during review of DESIGN.md — omitting them would have left DESIGN.md in an inconsistent state per Risk 3 / STOP trigger. ward:struere would have flagged.

**No other surprises.** Stone executed exactly per BRIEF skeleton. All six deliverables shipped. 9/9 thread tests pass. Zero `comms::thread::bounded` references remain.

## Cross-references

- BRIEF-214-SLICE-2-FORWARD-CORRECTION-MINI-TCP.md — work order
- EXPECTATIONS-214-SLICE-2-FORWARD-CORRECTION-MINI-TCP.md — 19-row prediction (all satisfied; Mode A)
- DESIGN.md § "Slice 2 forward-correction (2026-05-19) — Mini-TCP at depth 1 (universal symmetry)"
- INTERSTITIAL § "2026-05-19 (post-compaction, Slice 2 forward-correction) — Mini-TCP at depth 1: the trading-lab origin returns"
- SCORE-214-SLICE-3E2-SELECT-PERSISTENT-RING.md — parallel structure (Mode A delivery pattern)
