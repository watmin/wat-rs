# BRIEF — arc 292 L2: timerfd-backed process Receiver (`Source {Pipe|Timer}`)

**You are a LEAF executor.** ONE bounded change in `src/comms/process.rs` ONLY. Do NOT
spawn subagents. Do NOT touch any other file. If the work needs another file, STOP and
report. This file is WARDED (comms ward) — be faithful to its existing style.

## The work (one paragraph)
The process-tier `Receiver<T>` is fd-backed over a pipe. Add a second fd-SOURCE — a
one-shot **timerfd** — so a later strike can build a process-tier timer peer. Factor the
fd source into a named enum `Source { Pipe{read_fd}, Timer{timer_fd, msg} }`, keeping the
shared io_uring/accumulator/frame machinery common. The `Timer` source, when its timerfd
fires, delivers a pre-encoded message **frame** exactly once (atomic-gated, zero-mutex) by
appending it to the accumulator — so the EXISTING frame-extraction path
(`take_buffered_frame`) and the EXISTING `process::Select` poll loop both work unchanged
(a timerfd is a pollable fd). Add a `pub fn timer(duration, msg_frame) -> Receiver<String>`
constructor and a `#[cfg(test)]` test that fires a timer through `process::Select`.

## Worked reference — READ THIS FIRST (do not modify it)
`src/comms/thread.rs:118-245` is the SHIPPED thread-tier equivalent: `enum ReceiverKind<T>
{ Channel(..), Timer{ instant_rx, msg: Arc<OwnedMoveCell<T>> } }`, every method matches on
it, and the one-shot msg is taken via `msg.take(":wat::kernel::after", Span::unknown())`
(thread.rs:200) — **`OwnedMoveCell`, atomic-gated, ZERO-MUTEX. A `Mutex` here is a heresy
that was already caught once — do NOT use one** (see `docs/ZERO-MUTEX.md`; `OwnedMoveCell`
lives in `src/rust_deps/custodia.rs`: `OwnedMoveCell::new(v)` + `.take(op, span)
-> Result<T>`). The thread `timer()` fn is at `src/comms/thread.rs:~428`. You are building
the io_uring/process analog of this, NOT crossbeam.

## Rooms — read in order (process.rs)
1. `:448-475` — the `Receiver<T>` struct: `read_fd: OwnedFd, accumulator, max_frame_bytes,
   ring, _phantom`. You replace `read_fd: OwnedFd` with `source: Source`.
2. `:481-494` — manual `Debug` impl (no derive; `IoUring` is `!Debug`). Add the source.
3. `:496-546` — `recv()` (blocking; does `let read_fd = self.read_fd.as_raw_fd();` then
   `wait_for_data_or_cascade(read_fd, ...)` + `read_into_acc` + `take_buffered_frame` in a
   loop). Change the hardcoded `self.read_fd` to `self.poll_fd()` so the SAME loop serves
   both sources (a Timer's `read_into_acc` will drain+deliver — see below).
4. `:561-563` — `read_into_acc()` → delegates to `uring_read_into_acc(self.read_fd...)`.
   This is THE method that must branch on source (see sketch).
5. `:579-581` — `take_buffered_frame()` → `take_frame(&mut acc, max_frame_bytes)`.
   UNCHANGED (both sources feed the same accumulator).
6. `:591-593` — `poll_fd()` → returns `read_fd`. Make it `match &self.source`.
7. `:608-610` — `raw_fds()` → `[read_fd, ring_fd]`. Make it `match &self.source` (timer:
   `[timer_fd, ring_fd]`).
8. `:622-632` — `len()`; `:642-644` — `close()`; `:661-689` — `recv_wire_raw()` (mirrors
   recv's loop — same `self.read_fd` → `self.poll_fd()` change).
9. `:692-725` — `Clone` impl: dup the fd + fresh acc/ring. Branch on source (Pipe: dup
   read_fd; Timer: dup timer_fd + `Arc::clone(msg)`).
10. `:927-987` — the standalone `uring_read_into_acc` fn + the io_uring Read SQE pattern.
    COPY THIS PATTERN for the Timer's 8-byte expiration read (read into a scratch `[u8;8]`,
    NOT into the accumulator).
11. `:1535-1560` — `pair()` / `pair_with_budget()` construct `Receiver { read_fd, ... }` →
    update to `Receiver { source: Source::Pipe { read_fd }, ... }`.
12. `:123-126` — `type Frame = Vec<u8>;` and `decode_frame` (`:856`). The timer's `msg` is
    a `Frame` (pre-encoded EDN bytes + `'\n'`).

## Implementation sketch (fill it; the shape is fixed)

```rust
// near the Receiver struct:
enum Source {
    /// EDN frames over a pipe read-end.
    Pipe { read_fd: OwnedFd },
    /// One-shot timerfd: on fire, deliver `msg` (a pre-encoded frame) exactly once.
    /// `msg` taken via OwnedMoveCell (atomic-gated, ZERO-MUTEX — mirrors thread.rs:200).
    Timer { timer_fd: OwnedFd, msg: std::sync::Arc<crate::rust_deps::custodia::OwnedMoveCell<Frame>> },
}

pub struct Receiver<T: EdnRepresentable> {
    source: Source,
    accumulator: Accumulator,
    max_frame_bytes: usize,
    ring: RefCell<IoUring>,
    _phantom: PhantomData<T>,
}
```

`poll_fd`: `match &self.source { Source::Pipe{read_fd} => read_fd.as_raw_fd(), Source::Timer{timer_fd,..} => timer_fd.as_raw_fd() }`

`read_into_acc` (the heart):
```rust
pub(crate) fn read_into_acc(&self) -> Result<usize, ()> {
    match &self.source {
        Source::Pipe { read_fd } =>
            uring_read_into_acc(read_fd.as_raw_fd(), &self.accumulator, &self.ring),
        Source::Timer { timer_fd, msg } => {
            // Drain the 8-byte expiration count via io_uring Read into a scratch buffer
            // (copy the SQE pattern from uring_read_into_acc, but read into [u8;8], NOT acc).
            let n = uring_read_n_into_scratch(timer_fd.as_raw_fd(), &self.ring, 8)?; // n==8 on fire
            if n == 0 { return Ok(0); } // EOF — spent
            // Take the msg ONCE (atomic-gated) and append the frame to the accumulator so
            // the normal take_buffered_frame path extracts it.
            if let Ok(frame) = msg.take(":wat::kernel::after", crate::span::Span::unknown()) {
                self.accumulator.borrow_mut().extend_from_slice(&frame); // frame already ends in '\n'
            }
            Ok(n)
        }
    }
}
```
(Implement `uring_read_n_into_scratch` as a small private fn next to `uring_read_into_acc`,
or inline — your call; same io_uring Read SQE shape, scratch dest, retry-on-EINTR like the
existing helper.)

`timer()` constructor:
```rust
/// One-shot process-tier timer. `timer_fd` = `timerfd_create(CLOCK_MONOTONIC,
/// TFD_NONBLOCK|TFD_CLOEXEC)` armed via `timerfd_settime` with it_value=`duration`,
/// it_interval=0 (fires once). On fire it delivers `msg_frame` (pre-encoded EDN + '\n').
/// io_uring polls the timerfd like any data fd — process::Select is UNCHANGED.
pub fn timer(duration: std::time::Duration, msg_frame: Frame) -> std::io::Result<Receiver<String>> {
    // libc::timerfd_create + libc::timerfd_settime with struct itimerspec.
    // Wrap the raw fd in OwnedFd. ring = IoUring::new(4) (same as pair()).
    // source: Source::Timer { timer_fd, msg: Arc::new(OwnedMoveCell::new(msg_frame)) }.
}
```
(Return `io::Result` for the syscall failures; the L3 caller maps it. Match the existing
`pair()` error style.)

`#[cfg(test)]` test (add to the existing `#[cfg(test)] mod` at `:246` or a sibling):
build `timer(Duration::from_millis(50), b":tick\n".to_vec())`, register it in a
`process::Select`, call `select()`, assert it returns `SelectOutcome::Recv { result:
Ok("tick"-decoding-frame) }` after ~50ms. (Look at existing process.rs tests for the Select
test harness shape; if the substrate broadcast fd must be set for Select, follow what the
existing tests do.) Name it e.g. `timer_source_fires_through_select`.

## Blast radius (bounded)
`src/comms/process.rs` ONLY. No other file. No `kernel/`, no `runtime.rs`, no `check.rs`
(those are L3). The wat surface is NOT touched in L2.

## STOP triggers (halt + report; do NOT improvise)
1. If making `Source` an enum forces a change OUTSIDE `process.rs` (e.g. another module
   names `Receiver { read_fd }` directly), STOP and report the site.
2. If the `process::Select` test harness cannot run a timer without substrate setup you
   can't replicate from existing tests, STOP and report (do not weaken the test to a
   non-firing stub).
3. If you find yourself reaching for `Mutex`/`RwLock`/`RefCell<Option<..>>` to hold the
   timer msg, STOP — the answer is `OwnedMoveCell` (atomic-gated), already imported pattern.

## Gate (run it yourself; report real output)
```
cargo build 2>&1 | tail -20
cargo test --lib timer_source_fires_through_select -- --nocapture 2>&1 | tail -30
cargo test --no-fail-fast 2>&1 | grep -E '\.\.\. FAILED$' | sort -u | wc -l   # must not exceed HEAD's ~218 floor
```
Report: the build tail, the new test's result line, and the total FAILED count (the
pre-existing ~218 floor must be unchanged — the stdlib `deporder`/`lint_stdlib_runs` tests
flap ±1, that is known/unrelated).

## Report back (raw facts — your text is the return value)
1. `git diff --stat` (do NOT commit — I commit after weighing).
2. The exact build tail + the new test result line.
3. The total FAILED count vs ~218.
4. Any STOP trigger hit.
