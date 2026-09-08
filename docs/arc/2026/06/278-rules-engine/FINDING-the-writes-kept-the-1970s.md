# FINDING — the writes kept the 1970s

**A `send` that has partially written cannot be stopped.** Two files, one inherited pattern, found by
a floor RED on 2026-09-08 after months of green.

> ⛔ **This was found by using the services in anger, not by a unit test.** Eleven consecutive green
> floors ran over it. It surfaced only because a drain benchmark at depth put enough concurrent
> blocked sends through the transport to construct the one pipe state no existing test constructs.

## THE RED

```
FAIL [20.028s] (2345/5221)
wat::comms probe_arc278_partial_frame_residue::probe_sender_send_leaves_headless_partial_frame_on_shutdown

tests/comms/probe_arc278_partial_frame_residue.rs:297:9
"the child did not report within 20s of SIGTERM — the blocked send never woke; this is the
 poll-arm-missing RED state probe_arc278_send_poll_arm.rs already covers, not this probe's subject."
```

Artifact: `.floor/2026-09-08T04-10-37Z/`. **Not re-run.**

★ **The probe's own attribution is wrong.** The sibling it blames **passed in the same run, in 34 ms**:

```
PASS [ 0.034s] (2018/5221) probe_arc278_send_poll_arm::probe_sender_send_wakes_on_shutdown_broadcast
```

So the poll arm is present and *does* wake a blocked send. The two probes differ by one variable.

## THE MECHANISM — three ingredients, each individually defensible

```
1.  poll([fd → POLLOUT, broadcast → POLLIN])     POLLOUT means "≥ 1 byte writable"
2.  break                                        "writable — proceed, stop or no stop"
3.  libc::write(fd, buf, FULL remaining length)  on a BLOCKING fd
4.  kernel writes what fits, then BLOCKS for the rest
5.  SIGTERM → handler installed via libc::signal() → glibc BSD semantics → SA_RESTART
6.  the write AUTO-RESTARTS and blocks again.  Nothing can wake it.
```

★★★ **`poll` guarantees one byte; the code then demands all of them.** The shutdown broadcast is an
**fd**, and an fd cannot interrupt a write already inside the kernel — while the one thing that
could, `EINTR`, is disabled by `libc::signal`'s auto-restart.

## WHY IT HID FOR MONTHS

| probe | pipe state before `send` | `POLLOUT` | outcome |
|---|---|---|---|
| `send_poll_arm` | **completely full** | never set | shutdown arm fires → `Err(Shutdown)` ✅ |
| `partial_frame_residue` | **one frame of room** | **set** | `break` → blocking write → stuck ⛔ |

**Every existing test fills the pipe completely.** You need *partial* room — and the only probe that
engineers it was written for an entirely different purpose (measuring the residue a stopped send
leaves behind). It cannot reach its own subject, because the send never stops.

## ⛔ BOTH FILES REASONED TO THE EDGE AND STOPPED ONE STEP SHORT

**`src/comms/process.rs:353`**

> *"THE BLOCKING IS NOT THE BUG (STOP-1) — this still blocks on `libc::write` below when the pipe has
> room; the poll only makes the wait WAKEABLE on a stop instead of uncancellable."*

**`src/io.rs:670`**

> *"EINTR re-polls; never a blind retry (constraint 3 — **whether EINTR ever reaches here under this
> repo's signal handlers is deliberately unresolved**; a poll-first loop is correct either way)."*

★★★★ Both claims are true **when the pipe is completely full** — the only state any test builds.
Neither holds at partial room. And `io.rs` names the exact question (does this repo's `signal()`
defeat EINTR?) and rules it irrelevant on the grounds that the poll-first loop is safe regardless.
**It is not.** The poll-first loop still ends in a full-length blocking write.

## ⛔ THE PATTERN WAS INHERITED, NOT INVENTED

`src/comms/process.rs` says so itself: the send poll mirrors
*"`io::PipeWriter::write` … (`src/io.rs`, arc 170 closure #5)."*

```
src/io.rs              10 blocking libc calls,  ZERO io_uring       ← the ORIGIN
src/comms/process.rs    3 blocking libc calls,  io_uring on READ only ← the COPY
src/runtime.rs          5
src/process/boot/       5
```

And `comms/process.rs`'s own header, line 10:

```
newline-framed bytes → libc::write → io_uring Read → bytes → EDN string
                       ^^^^^^^^^^^   ^^^^^^^^^^^^^
                       raw syscall   Linux-native, cancellable
```

★★★★★ **The reactor got the reads. The writes kept the 1970s.** `src/io.rs` was never migrated at
all; `comms/process.rs` migrated its read path and copied the un-migrated write pattern from its
neighbour.

## ⛔ AND IT CONTRADICTS THE PROJECT'S OWN PLATFORM FLOOR

```
src/process/mod.rs:9    Linux 5.3+ (clone3 + CLONE_PIDFD + CLONE_CLEAR_SIGHAND), 5.9+
src/process/mod.rs:13   "Production use requires Linux 5.9+; the 6.x floor is …"
clone3 · CLONE_PIDFD · CLONE_CLEAR_SIGHAND · P_PIDFD · io_uring   — all Linux-only
```

The builder: *"we are a linux programming language... it screams to me that we are not sticking
that."* A runtime that hard-requires `CLONE_CLEAR_SIGHAND` has no portability argument left for
`libc::signal()` — the least-specified signal interface there is, whose BSD-vs-System-V ambiguity is
*why* glibc chose auto-restart.

## ⛔ EINTR IS THE WRONG FIX

The instinct is *"add EINTR support"* — clear `SA_RESTART` via `sigaction` so the write returns
`EINTR`. **That would make it usually wake, which is worse than always failing:** the signal can land
between the check and the syscall, and the write blocks anyway. That race is the entire historical
motivation for self-pipes and `signalfd`.

★★★ **The rule both files already know and break: all waiting belongs in the multiplexer; no
unmultiplexed syscall may block.** The `poll` *is* the multiplexer. `libc::write` on a blocking fd is
a second, invisible wait the multiplexer does not cover.

★ The short-write loop already exists one level up (`io.rs` returns `Ok(ret)` and its caller loops
`while !remaining.is_empty()`). **The code is already prepared to resume from a partial write.** It
blocks anyway, only because the fd is blocking and it asks for the full length. Nothing is bought by
that blocking.

## THE THREE STONES

1. **`a write never blocks outside the multiplexer`** — `O_NONBLOCK` + poll owns the wait, both
   files. Small, fixes the RED, and the probe that found it is the gate.
2. **`sigaction, not signal`** — correctness on its own merits, and it retires `io.rs`'s
   "deliberately unresolved" comment.
3. **`the write path joins io_uring`** — architectural; matches the 5.9+ floor and the read path in
   the same file. Removes the pattern rather than patching it. ⚠ **May deserve its own arc — the
   builder's ruling, not a side effect.**

## ⚠ WHAT IS NOT ESTABLISHED

- **Whether today's work made this reachable.** `a reconnect is not an abandonment` (uncommitted)
  makes `Lost`/`Closed` retry instead of abandon, putting more concurrent blocked sends through the
  transport than any prior floor — and this RED appeared on the first floor after it. **Suggestive,
  not evidence.** Eleven floors on adjacent trees were green.
- **Whether it is deterministic.** One RED, not re-run. It needs the probe in isolation, repeatedly,
  on a quiet box — a measurement, not a re-run for luck.
