# DESIGN — Stone 259.S3.6 — one frame-finder (value-frame the comms reader; decomplect the duplicate)

**STRIKE-READY.** RED probe verified: `tests/nursery/probe_arc259_comms_recv_multiline_frame.rs`
fails at HEAD — comms `recv` of a multi-line value yields `"{"` (first-`\n` split), not the whole
`"{\n  :a 1\n}"`.

## Why (the duplicate, and the gap it left)
There are TWO fd-consumption framers doing newline-detection + frame-extraction:
- `read_framed_edn` (`src/edn_shim.rs`) — the ambient/channel **WatReader** (blocking `read_line`)
  path; value-frames (accumulate lines until a complete EDN value). Shipped this session.
- `take_frame` (`src/comms/process.rs:849`) — the comms **io_uring** path; splits on the FIRST
  `'\n'`, on a stale assumption ("wat-edn is single-line", process.rs:51).

They diverged because the I/O backends differ (blocking vs io_uring — unifying *those* is the
reactor, out of scope). But the **framing** got built twice, and the comms copy never learned
value-framing. So a multi-line value crossing a process peer (a child `pprintln`-ing a pretty map →
parent `recv'`) is mis-framed: only the first line `{` is read. This is the 259 cross-proc-multi-line
blocker (and it blocks arc 255's gold-standard IPC tests).

## The contract — ONE frame-finder, both readers route through it
Do NOT teach `take_frame` to value-frame as a *second* copy (drift planted). **Extract the
frame-finding as one pure byte-level core; route both readers through it.**

```rust
// src/edn_shim.rs — the ONE frame-finder. Pure, no I/O.
// "given accumulated bytes, where does the first COMPLETE EDN value end?"
pub enum FrameScan { Frame(usize) /* end offset, incl. terminating \n */, Incomplete, TooLarge(usize), Malformed(String) }
pub fn next_complete_frame(buf: &[u8], max_bytes: usize) -> FrameScan
```
- Line-granular: scan `'\n'` positions; for each prefix, `edn_frame_status` (the existing
  completeness predicate); the FIRST prefix that is `Complete` → `Frame(end)` (end = byte after that
  `\n`). No `'\n'` yet / no Complete prefix → `Incomplete`. `buf.len() > max_bytes` before a Complete
  → `TooLarge`. A prefix that is `Malformed` (parse error, not incomplete) → `Malformed`.
- Owns newline-scan + completeness + the `DEFAULT_MAX_FRAME_BYTES` cap + anti-smuggle, in ONE place.

### Route both through it
- **`read_framed_edn`** (blocking-pull wrapper): accumulate `read_line` bytes into a buffer; call
  `next_complete_frame`; `Incomplete` → read another line + append; `Frame(end)` → decode `buf[..end]`;
  `TooLarge`/`Malformed` → the existing error returns. Behavior MUST be preserved (its tests stay green).
- **`take_frame`** (`comms/process.rs`, non-blocking-peek wrapper): `next_complete_frame(&acc, cap)`;
  `Frame(end)` → `acc.split_off(end)` keep-remainder + return the frame bytes (terminating `\n`
  stripped, as today); `Incomplete` → `None` (io_uring reads more); `TooLarge`/`Malformed` → surface
  via the existing recv error path (a malformed/over-cap frame → `recv` errors, matching today's
  bad-line behavior). Keep `take_frame`'s `Option<Frame>` contract if it can carry this; if not, STOP.

## STOP triggers (surface, don't improvise)
1. If refactoring `read_framed_edn` onto `next_complete_frame` changes its OBSERVABLE behavior — the
   existing framing tests (`probe_edn_value_framing.rs`: multi-line, pprintln round-trip,
   anti-smuggle, tiny-cap) must stay green. If they shift, STOP.
2. If `take_frame`'s `Option<Frame>` signature can't cleanly carry `TooLarge`/`Malformed` (the comms
   recv has no error channel there) — STOP and report the minimal signature change (don't silently
   drop the cap).
3. If the io_uring recv loop (`read_into_acc` + `take_buffered_frame`) depends on the first-`\n`
   semantics anywhere else — STOP, report.

## Out of scope (affirmative cuts)
- Unifying the I/O backends (blocking `WatReader` vs io_uring) — that is the reactor, a separate arc.
  This stone unifies only the **framing**.
- The gold-standard 255 IPC tests + the unbounded-line bound (#268) — they ride on TOP of this; after.

## Gate (independent scorecard)
| # | what | command | expected |
|---|---|---|---|
| 1 | the comms multi-line probe goes green | `cargo test --release -p wat --test nursery comms_recv_value_frames` | 1 passed |
| 2 | ambient framing unchanged | `cargo test --release -p wat --test nursery -- multiline_edn_value_frames pprintln_multiline_map anti_smuggling tiny_cap` | all green |
| 3 | comms suite green (compact send'/recv' still works) | `cargo test --release -p wat --test comms` | green |
| 4 | channel suite green | `cargo test --release -p wat --test channel` | green |
| 5 | lib floor | `cargo test --release -p wat --lib` | 953 / 36 / 1 (identical baseline) |
| 6 | nursery floor | `cargo test --release -p wat --test nursery` | the comms probe flips green; no NEW failures |
| 7 | the live proxy still round-trips | `wat wat-scripts/intrinsic-metadata.wat \| wat wat-scripts/read-flat.wat` | pretty in → flat out |
| 8 | clippy clean on touched files | `cargo clippy --release -p wat` | no new warnings |

Runtime prediction: 40–70 min (extract `next_complete_frame` + two wrapper refactors + edge handling).
Trap-door: STOP-2 (the comms `Option` carrying cap/malformed) is the likely wrinkle.

## After this lands
255 unblocks → the four gold-standard `deftest-hermetic'` IPC tests (round-trip + over-cap + truncated
+ anti-smuggle) land on `recv'` value-framing: a child `pprintln`s the map, the parent `recv'`s it as
one value; negatives via `poll'`→`:Closed`. Then #268 (the unbounded-line bound).
