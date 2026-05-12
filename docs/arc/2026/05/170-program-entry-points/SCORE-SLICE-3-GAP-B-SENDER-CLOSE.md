# Arc 170 slice 3 Gap B — SCORE (Sender/close — explicit EOF signaling)

**Date:** 2026-05-11
**Branch:** arc-170-program-entry-points
**Status:** complete

## Scorecard verification

| Row | What | Pass criterion | Result |
|-----|------|----------------|--------|
| A | `AtomicBool closed` flag added to `SenderInner` (Option A or B) | `grep -n "AtomicBool" src/typed_channel.rs` | PASS — variant-local fields at typed_channel.rs:95,106; import at line 71 |
| B | `:wat::kernel::Sender/close` registered as a runtime primitive + dispatched | `grep "Sender/close" src/runtime.rs src/check.rs` | PASS — dispatch arm at runtime.rs:3651; eval fn at runtime.rs:14971; type scheme at check.rs:12505 |
| C | `typed_send` consults the closed flag before transport send | `grep "closed.load" src/typed_channel.rs` | PASS — Crossbeam arm at line 210; PipeFd arm at line 220; both Acquire-ordered |
| D | Close-then-send returns Err for Crossbeam AND PipeFd transports | unit tests `sender_close_crossbeam_close_then_send_returns_disconnected`, `sender_close_pipefd_close_then_send_returns_disconnected` | PASS — 6/6 new tests green |
| E | PipeFd close triggers reader EOF (`writer.close()` releases write-end fd) | unit test `sender_close_pipefd_triggers_reader_eof` | PASS — `typed_recv` returns `Disconnected` after `sender_close` |
| F | Close is idempotent (calling twice is a clean no-op) | unit tests `sender_close_crossbeam_idempotent`, `sender_close_pipefd_idempotent` | PASS |
| G | Workspace stays at 0 failed | `cargo test --release --workspace --no-fail-fast` | PASS — 2199 passed / 0 failed (+6 new tests; baseline 2193) |
| H | `cargo check --release` green | clean | PASS — one pre-existing dead-code warning (unrelated to this arc) |

**All 8 rows pass.**

## Option A vs Option B rationale

**Option A chosen** (variant-local named fields). Reasons:

1. **Symmetric per-transport semantics**: each transport variant owns its
   own `AtomicBool`. The Crossbeam flag is independent of the PipeFd
   flag — there is no shared state to contend. This mirrors the zero-
   Mutex architecture: state lives where it is used, not in a shared
   wrapper.

2. **No extra indirection**: Option B (wrapper struct `SenderState`)
   introduces one more heap layer (`Arc<SenderState>` containing
   `Arc<SenderInner>`). Option A keeps the existing `Arc<SenderInner>`
   depth unchanged; the flag is an interior field, not a level above.

3. **Structural honesty**: the closed flag means different things per
   transport — for Crossbeam it is pure signal; for PipeFd it also
   triggers `writer.close()`. Placing the flag inside each variant keeps
   that transport-specific logic local.

4. The variant change required updating two pattern-match sites
   (thread_io.rs and typed_channel.rs tests) from positional to named
   form — a mechanical update with no semantic change. Cost was minimal.

## PipeFd shutdown mechanism

The BRIEF specified `shutdown(SHUT_WR)`. For Linux pipes, `libc::shutdown`
is a socket-only syscall — it does not work on pipe file descriptors.
The correct EOF signal on a pipe's write end is `close(2)` of that fd.

`PipeWriter::close` (src/io.rs:665) already implements this: it atomically
swaps the fd to -1 via `AtomicI32` and calls `libc::close(2)`. The swap
is idempotent (no-op if fd is already -1). This is exactly the mechanism
`IOWriter/close` uses for byte-stream writers (the pipe sibling this Gap B
mirrors). No new trait methods were needed.

`sender_close` for PipeFd:
1. Sets `closed.store(true, Ordering::SeqCst)` — stops `typed_send` immediately
2. Calls `writer.close(span)` — releases the write-end fd

The child's reader sees EOF on its next `read_line` call, exactly as if
the Sender Value had been dropped.

## eval_kernel_sender_close location

Located in `src/runtime.rs`, immediately before `eval_kernel_recv` (~line
14966). Rationale: all kernel comm primitives (`send`, `recv`, `try-recv`,
`drop`, `select`) live in runtime.rs; the new function belongs in the same
cluster, adjacent to its sibling `eval_kernel_send`.

`typed_channel::sender_close` is a public function in
`src/typed_channel.rs` that holds the transport-specific close logic. The
runtime function is a thin dispatcher (arity check + Value extraction +
call to `typed_channel::sender_close`). This mirrors the
`eval_iowriter_close` → `writer.close()` delegation pattern in io.rs.

## Implementation locations

### Phase 1 — AtomicBool flag on SenderInner

**`src/typed_channel.rs`**
- `use std::sync::atomic::{AtomicBool, Ordering}` — new import at line 71
- `SenderInner::Crossbeam` changed from tuple-form to named fields (`sender`, `closed`) at line ~88
- `SenderInner::PipeFd` changed from tuple-form to named fields (`writer`, `closed`) at line ~98
- `sender_from_crossbeam` constructor initializes `closed: AtomicBool::new(false)`
- `sender_from_pipe` constructor initializes `closed: AtomicBool::new(false)`
- `typed_send` updated to named-field match arms; `closed.load(Ordering::Acquire)` check before transport send
- `sender_close` function added (public) — Crossbeam: SeqCst store; PipeFd: SeqCst store + `writer.close()`

**`src/thread_io.rs`**
- `unwrap_value_sender` match arm updated from `SenderInner::Crossbeam(s)` to `SenderInner::Crossbeam { sender: s, .. }` (mechanical fix)
- `SenderInner::PipeFd(..)` updated to `SenderInner::PipeFd { .. }`

### Phase 2 — Runtime primitive

**`src/runtime.rs`**
- Dispatch arm `:wat::kernel::Sender/close` → `eval_kernel_sender_close` added at ~line 3651, adjacent to `:wat::kernel::send`
- `eval_kernel_sender_close` function at ~line 14966

### Phase 3 — Type-check scheme

**`src/check.rs`**
- `env.register(":wat::kernel::Sender/close", ...)` at ~line 12505, immediately after the `:wat::kernel::send` registration
  - Type scheme: `∀T. Sender<T> -> :()` (nil return; close always succeeds)
- `":wat::kernel::Sender/close"` added to the pair-deadlock skip-list at ~line 3308 (single-Sender call; arc 126 should not fire)

### Phase 4 — Unit tests

**`tests/wat_arc170_typed_channel_pipes.rs`**
- Import updated: `sender_close` added to use list; positional `SenderInner::PipeFd(writer)` in existing test updated to `SenderInner::PipeFd { writer, .. }`
- 6 new tests under `// ─── Arc 170 slice 3 Gap B — Sender/close unit tests ───`:
  1. `sender_close_crossbeam_close_then_send_returns_disconnected` — Row C/D (Crossbeam)
  2. `sender_close_crossbeam_idempotent` — Row F (Crossbeam)
  3. `sender_close_pipefd_close_then_send_returns_disconnected` — Row C/D (PipeFd)
  4. `sender_close_pipefd_idempotent` — Row F (PipeFd)
  5. `sender_close_pipefd_triggers_reader_eof` — Row E
  6. `wat_kernel_sender_close_dispatch_via_eval` — end-to-end wat-level integration test (Row B)

## Files modified

| File | Change |
|------|--------|
| `src/typed_channel.rs` | SenderInner variant shape; AtomicBool flag; constructors; typed_send flag-check; sender_close function |
| `src/thread_io.rs` | Match arm updated for named SenderInner fields (mechanical) |
| `src/runtime.rs` | eval_kernel_sender_close function; dispatch arm at Sender/* cluster |
| `src/check.rs` | Sender/close type scheme; pair-deadlock skip-list entry |
| `tests/wat_arc170_typed_channel_pipes.rs` | 6 new tests; import update; one existing pattern-match updated |

## Honest deltas

1. **`shutdown(SHUT_WR)` is a socket API, not a pipe API.** The BRIEF
   specified `shutdown(SHUT_WR)` but `libc::shutdown(2)` is defined only
   for sockets. For pipes, EOF on the read side is signaled by closing
   the write-end fd via `close(2)`. `PipeWriter::close` already does
   this correctly. No new mechanism was needed; `writer.close()` is the
   correct and sufficient call. This is not a gap — the BRIEF named the
   effect (`peer reader sees EOF`), not the specific syscall. The
   implementation achieves the documented effect through the right means.

2. **Named-field refactor of SenderInner variants.** `SenderInner::Crossbeam`
   and `SenderInner::PipeFd` changed from tuple-form to named fields to
   accommodate the `closed: AtomicBool` without ambiguity. Two external
   match sites needed mechanical updates (thread_io.rs + one test). This
   cost was anticipated in the Option A vs B analysis. Named fields are
   more self-documenting than positional; the change is a net positive.

3. **Receiver/close not added.** The symmetric `Receiver/close` form is
   out of this slice's scope. It would allow a Receiver holder to signal
   "I will not read further" to the sender (which currently surfaces as
   EPIPE/Disconnected naturally). For tier-1 (Crossbeam) this would
   require dropping the crossbeam Receiver — tricky when the Value is
   `Arc`-wrapped. For tier-2 (PipeFd) it would call `reader.close()` to
   release the read-end fd. Suggested as a follow-up arc if a Layer 2
   streaming pattern demands it.

4. **`Ordering::Acquire` for load, `Ordering::SeqCst` for store.** The
   load in `typed_send` uses Acquire to pair with the SeqCst store in
   `sender_close`. SeqCst on the store gives the strongest happens-before
   guarantee across all threads; Acquire on the load is sufficient for
   the "did this thread see the store?" question. This matches the
   pattern used by the wat-cli's `CHILD_PID` atomic
   (ZERO-MUTEX.md § "Arc 104 boundary") and avoids the cost of double-
   SeqCst without weakening safety.

5. **Pair-deadlock skip-list entry.** `Sender/close` takes one Sender
   argument (no Receiver); arc 126's pair-deadlock check is irrelevant.
   Adding it to the skip-list prevents a false positive if the checker
   encounters `(:Sender/close tx)` where `tx` traces to a
   `make-bounded-channel` binding. Without the entry the checker might
   attempt pair-analysis on a one-sided call and produce a spurious
   error.

## What's next

- **Receiver/close** (symmetric form) — follow-up arc if Layer 2
  streaming patterns demand explicit read-end teardown.
- **Slice 4** — BareLegacy walker + retired-verb eval arms +
  Process<I,O> legacy fields (per the original arc 170 plan).
