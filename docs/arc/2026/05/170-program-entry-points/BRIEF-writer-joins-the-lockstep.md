# BRIEF — the writer joins the lock-step (arc 170 closure #5)

## The work, in one paragraph

`RealStdin` joined the poll multiplex at `3e297846` (24y) and closed the last *read* that could not
observe a stop. The *write* side never joined. `PipeWriter::write` (`src/io.rs:634`) is a bare
blocking `libc::write(2)` with no poll and a blind `EINTR → continue`. A write to a pipe whose
reader is not draining blocks, and the process cannot be stopped while it does. This brief makes the
writer poll `[fd, SHUTDOWN_BROADCAST_READ_FD]` before each attempt — exactly what the reader at
`src/channel/transfer.rs:200` already does — and makes the resulting stop a NAMED value rather than
a flattened one.

## Why the backlog's framing is wrong (read this, it changes what you build)

`CLOSURE-BACKLOG.md` item 5 asks: is `as_raw_fd_for_poll` on `WatWriter` *a gap* or *dead surface*?
It is neither, and the answer was grounded before this brief was written:

- It is NOT dead. `src/freeze.rs:269-271` reads `stdio.stdout.as_raw_fd_for_poll()` /
  `.stderr…` to seed the primed stdio defservices with fd NUMBERS. It has a live consumer.
- It is a hook whose *poll* consumer was never built. The reader overrides it AND polls with it;
  the writer overrides it and nobody polls.

So the name does not lie about the hook — it describes a capability that was declared and never
used. **Building the consumer is what makes the name true.** Do not rename it, and do not delete it.

## The rooms, in order, with why

1. `src/channel/transfer.rs:200-230` — **the exemplar.** The `ReceiverInner::PipeFd` read path:
   grabs `as_raw_fd_for_poll()`, loads `SHUTDOWN_BROADCAST_READ_FD`, and polls both fds around EACH
   `read_line`, with a named `Shutdown` result distinct from `Eof`. Its comment states the rule:
   *"shutdown wins ties."* Copy this shape; do not invent a new one.
2. `src/io.rs:634-657` — **the defect.** `PipeWriter::write`: `libc::write` in a loop, `EINTR →
   continue`, no poll, no awareness of the broadcast fd.
3. `src/channel/transfer.rs:87-94` — **the second defect, and it is separate.** The send path does
   `match writer.write_all(…) { Ok(()) => SendOutcome::Ok, Err(_) => SendOutcome::Disconnected }`.
   That `Err(_)` is a wildcard that erases every distinct failure into one variant — an EPIPE and a
   stop become the same value. This is the class 24x named at `src/kernel/peer.rs:118`, in this
   file's own neighbourhood.
4. `src/io.rs:104-110` — the `WatWriter::as_raw_fd_for_poll` trait default (`None`), and `:697` —
   `PipeWriter`'s override returning `Some(fd)`. The hook you will finally use.

## The constraint set

1. **Poll before every write attempt.** `[fd → POLLOUT, broadcast_fd → POLLIN]`, mirroring the
   reader. When `as_raw_fd_for_poll()` is `None` or `broadcast_fd < 0`, fall through to today's
   direct write unchanged — that is the `StringIoWriter`/no-broadcast path and it must not regress.
2. **A stop is a NAMED outcome, never a flattened one.** The broadcast firing is not an I/O error
   and must not be reported as one. The read side proves the shape: a distinct result the caller
   faces.
3. **`EINTR` re-polls; it never blind-retries.** Today's `continue` retries the write without
   consulting anything. Whether `EINTR` ever surfaces under this repo's `libc::signal` handlers
   (BSD semantics set `SA_RESTART`) is UNRESOLVED and deliberately left so — the fix is correct
   either way, because a poll-first loop cannot block and a re-poll cannot ignore a stop. Do not
   spend time resolving it; do not build anything that depends on the answer.
4. **Kill the `Err(_)` wildcard** at `transfer.rs:93`. Bind the error and let a stop be
   distinguishable from a disconnect. If `SendOutcome` cannot express a stop today, that is a real
   finding — see STOP-2.
5. **Shutdown wins ties**, as the reader already states.

## The RED gate — it must be able to hang

A gate that cannot fail proves nothing (`NISI FRANGAS, NIHIL PROBAS`), and this arc has now found
four gates that were green on nothing. So the probe must demonstrate the hang, not assert it:

- Create a pipe; fill it (a pipe buffer is ~64 KiB by default) with nobody draining.
- Attempt one more write from a spawned thread, so the test itself never blocks.
- Trigger the shutdown broadcast.
- Join with a timeout.

**Before the change:** the join times out — the write is unstoppable. **After:** the write returns
the named stop promptly. Assert on the returned value, not on a timing artifact alone.

Run it BEFORE your change and paste the timeout. If it does not hang before, the gate is not
measuring the defect and you must say so rather than proceed.

## Gates — foreground, real output pasted

1. `cargo build --release --all-targets` — exit 0.
2. Your new probe: the before-run (hangs/times out) and the after-run (returns the stop).
3. `cargo nextest run --release -E 'binary_id(wat::channel) or binary_id(wat::comms)'` — the
   transports nearest this change.

Run everything in the foreground and wait for it. Do not background a command and return. Do not
run the whole `cargo nextest run` suite — the central weigh is the orchestrator's, and a full run
here forks child processes that can outlive you.

Do not commit.

## STOP triggers — halt and report, do not work around

1. If making the stop a named outcome requires changing `WatWriter::write`'s signature across every
   implementor, STOP and describe the surface. Do NOT invent a sentinel return value (a magic `0`,
   a special errno) to avoid the trait change — a flattened stop is the exact defect being fixed.
2. If `SendOutcome` (the internal one in `src/channel/`) has no variant that can carry "stopped"
   and adding one cascades beyond `transfer.rs`, STOP and report the blast radius. That is a design
   call, not yours to make.
3. If the RED probe cannot be made to hang deterministically before the change, STOP. Do not ship a
   gate whose red state you have not personally observed.
4. If the poll changes behaviour for `StringIoWriter` or any non-fd writer, STOP — those have no fd
   and must take the unchanged path.

## Report back

The diff, the before/after of the RED probe with real timings, the gate output, and anything that
surprised you.
