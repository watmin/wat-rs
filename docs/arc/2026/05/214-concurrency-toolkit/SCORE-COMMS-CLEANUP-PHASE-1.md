# SCORE — comms cleanup Phase 1 (mechanical fixes)

Date: 2026-05-19  
Session: vigilia cast follow-through — 13-site mechanical pass  
Files touched: `src/comms/thread.rs`, `src/comms/process.rs`, `tests/comms/process.rs`

---

## Scorecard

| Row | Check | Result | Evidence |
|---|---|---|---|
| 1 | Group A: 3× `rune:forge(escape)` replaced with `rune:sequi(ambient-context)` in thread.rs | **PASS** | `grep -c 'rune:forge' src/comms/thread.rs` → `0` |
| 2 | Group B: 3× `rune:sequi(ambient-context)` added above SHUTDOWN_BROADCAST_READ_FD loads in process.rs | **PASS** | `grep -c 'rune:sequi.ambient-context' src/comms/process.rs` → `3` |
| 3 | Group C: 5× `rune:temperare(no-reactor)` added above IoUring::new sites in process.rs | **PASS** | `grep -c 'rune:temperare.no-reactor' src/comms/process.rs` → `5` |
| 4 | Group D: SeqCst → Acquire on 3 SHUTDOWN_BROADCAST_READ_FD loads | **PASS** | `grep -c 'Ordering::Acquire' src/comms/process.rs` → `3`; `grep -c 'Ordering::SeqCst' src/comms/process.rs` → `0` |
| 5 | Group E: Default impls deleted from both thread.rs and process.rs | **PASS** | `grep -c 'impl.*Default for Select' src/comms/thread.rs src/comms/process.rs` → `0` each |
| 6 | Group F: Stone A sentence reworded in tests/comms/process.rs | **PASS** | `grep -n 'Stone A' tests/comms/process.rs` → no output (empty) |
| 7 | `cargo build --release` succeeds | **PASS** | exit 0; `Finished release profile [optimized]` in 17.41s |
| 8 | `cargo test --release --test comms` succeeds | **PASS** | `35 passed; 0 failed` |

All 8 rows PASS.

---

## Actual line numbers edited

### src/comms/thread.rs

| Group | Original line | Action |
|---|---|---|
| A | 108 | `rune:forge(escape)` comment (2-line) → single-line `rune:sequi(ambient-context)` above `let shutdown_rx` |
| A | 208–209 | `rune:forge(escape)` comment (2-line) → single-line `rune:sequi(ambient-context)` above `let shutdown_arm` |
| A | 256–257 | `rune:forge(escape)` comment (2-line) → single-line `rune:sequi(ambient-context)` above `let srx` |
| E | 281–285 | Deleted entire `impl<'a, T: Send + 'static> Default for Select<'a, T>` block (4 lines + blank) |

### src/comms/process.rs

Group D edits (SeqCst→Acquire) ran first per brief instruction; Group B (rune comment above) followed.

| Group | Original line | Action |
|---|---|---|
| D+B | 240 | `Ordering::SeqCst` → `Ordering::Acquire`; rune comment added on line above (239→240 shift) |
| D+B | 326 | `Ordering::SeqCst` → `Ordering::Acquire`; rune comment added on line above |
| D+B | 699 | `Ordering::SeqCst` → `Ordering::Acquire`; rune comment added on line above |
| C | 260 | `rune:temperare(no-reactor)` added above `IoUring::new(2)` in `recv` read step |
| C | 368 | `rune:temperare(no-reactor)` added above `IoUring::new(2)` in `try_recv` read step |
| C | 517 | `rune:temperare(no-reactor)` added above `IoUring::new(4)` in `wait_for_data_or_cascade` |
| C | 712 | `rune:temperare(no-reactor)` added above `IoUring::new(ring_capacity)` in `Select::select` poll ring |
| C | 794 | `rune:temperare(no-reactor)` added above `IoUring::new(2)` in `Select::select` read ring |
| E | 859–863 | Deleted entire `impl<'a, T: HolonRepresentable> Default for Select<'a, T>` block (4 lines + blank) |

### tests/comms/process.rs

| Group | Original lines | Action |
|---|---|---|
| F | 27–29 | Reworded sentence: "Stone A 'no newlines in payload' constraint no longer applies" → "wire layer never sees a literal newline except as a frame delimiter" |

---

## Honest deltas

- **Group C count is 5, not 4.** The brief said "likely 4-5". Actual IoUring::new sites in process.rs are 5: recv (line 260), try_recv (line 368), wait_for_data_or_cascade (line 517), Select::select poll ring (line 712), Select::select read ring (line 794). All 5 annotated. The scorecard row 3 evidence reflects this honestly.
- **Group A comment collapsed from 2-line to 1-line.** The old `rune:forge(escape)` pattern used two lines (rune comment + "global access is the cascade contract" follow-on). The new `rune:sequi(ambient-context)` is a single-line comment per the brief's exact wording. The follow-on sentence was deleted (it was an artifact of the old forge-rune format; the new wording is self-contained).
- No unexpected scope expansion encountered. All edits stayed within src/comms/* and tests/comms/process.rs. src/fork.rs and src/spawn_process.rs were not touched.
