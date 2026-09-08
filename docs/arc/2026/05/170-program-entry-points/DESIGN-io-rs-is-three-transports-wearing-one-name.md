# DESIGN — `src/io.rs` is three transports wearing one name

**DESIGN ONLY. No BRIEF, deliberately.** The migration this arc kept calling *"io.rs joins the
reactor"* cannot be briefed yet, and this document is the reason why.

## The correction

I told the builder that `src/io.rs`'s only blocker was **ring ownership**, and that
`the sender grows a ring` (`c2a8e45e3`) had made it easier by settling that question. **That was
wrong.** Ring ownership *was* answered. A different and larger question was standing behind it.

## What `PipeWriter` actually wraps

Three unrelated kinds of file descriptor, through one type:

| construction site | function | the fd is |
|---|---|---|
| `io.rs:1288` | `eval_iowriter_open_file` | a **regular file** |
| `io.rs:1375` | `eval_iowriter_from_fd` | a **dup of process stdio**, sharing a file description with our own 0/1/2 |
| `io.rs:1658` | `eval_kernel_pipe` | a **pipe** end |

★ This is the fact that killed construction-time `O_NONBLOCK` earlier in this arc, recorded in R70's
honest register: grok declined it *"for a reason I had not seen — `PipeWriter` also wraps regular files
and dups of fd 1/2 sharing a file description with process stdio."* The same fact is fatal to a
whole-type reactor migration, and for stronger reasons.

## Why one ring cannot serve all three

io_uring's readiness model — `PollAdd` on `POLLOUT`, a short write, resume — **is a pipe model.**

- **A regular file** is always writable. There is no readiness to wait on, a write never blocks
  waiting for a peer to drain, and so the shutdown-broadcast arm protects against a state that cannot
  occur. The kernel does not poll regular-file I/O; it punts it to a worker.
- **A stdio dup** shares a *file description* with the process's own stdout/stderr. Per-fd state
  belongs to the description, not the descriptor — which is exactly why arming it is unsafe, and the
  reason already on the record.
- **A pipe** is the one case the reactor was designed for, and the only one where the wait is real.

⚠ So *"give `PipeWriter` a ring"* is not a migration. It is a request to give a reactor to two
transports that have nothing to reactor over.

## ★ The seam is already cut everywhere else

`WatWriter` / `WatReader` has **seven** implementations, one per transport:

```
io.rs:128  RealStdin      io.rs:211  RealStdout    io.rs:252  RealStderr
io.rs:312  StringIoReader io.rs:411  StringIoWriter
io.rs:479  PipeReader     io.rs:664  PipeWriter
```

`RealStdout` is its own impl. `StringIoWriter` is its own impl. **`PipeWriter` is the only impl that
serves three different fd kinds** — and therefore the only one that cannot be given a single wait
strategy honestly. The abstraction this needs already exists; one type is failing to use it.

★★ The stone is therefore **`partire`, not migration**: split `PipeWriter` along the seam the trait
already provides — a file-backed writer, a stdio-backed writer, a pipe-backed writer — and *then* only
the pipe-backed one grows a ring. Two of the three never needed a multiplexer, and after the split
they stop pretending to have one.

## A cost the split also removes — measurable, not asserted

`io.rs:711-722` gates the multi-arm poll on `broadcast_fd >= 0`, **not on the fd's kind.** So every
`open-file` write pays a `poll()` for `POLLOUT` that a regular file can never fail to satisfy — a
syscall per write, on a path where the wait it guards cannot happen.

That is a `temperare` finding (correct but hot) and it should be **measured before it is claimed as a
win**; the clock read in this arc's own units table is 322 ns, so a per-write `poll` is the same order,
and whether it matters depends entirely on write volume.

## What must be ruled before a BRIEF exists

1. **Is the split the shape?** Three impls behind the existing trait, or a wait-strategy parameter on
   one type. The trait's other five members are evidence for the former.
2. **Where does the stdio-dup writer's wait belong at all?** It shares a description with the
   process's stdio; the honest answer may be *no multiplexing and no `O_NONBLOCK`*, which is what it
   does today for a reason already on the record.
3. **Does the file-backed writer keep any broadcast arm?** If a regular-file write cannot block on a
   peer, the arm is dead code that reads as safety.

⚠ **A BRIEF written before those are ruled would hand an executor a task whose substrate assumption is
false** — the Layer-2 failure `docs/COMPACTION-AMNESIA-RECOVERY.md` §4 records with a worked example
and a two-hour cost. So this stone stops at DESIGN.

## Out of scope — REJECTED

- **Anything in `comms/process.rs`.** It owns a private pipe pair; it is done and correct.
- **A ring for the file-backed or stdio-backed writer.** That is the error this document exists to
  prevent.
- **Claiming the removed `poll` as a speedup.** Measure it first.
