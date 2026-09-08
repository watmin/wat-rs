# NOTE — the write path joins io_uring

**Stone 3 of 3 from `FINDING-the-writes-kept-the-1970s.md`, and the only one that removes the
pattern instead of patching it.**

> ⛔ **THIS IS A NOTE, NOT A DRAWN STONE.** It is arc-shaped, and **opening or aiming an arc is the
> builder's ruling, never a side effect of the work.** Stones 1 and 2 are drawn and can proceed;
> this one is written down so it is not lost and not started.

## THE ASYMMETRY, IN THE FILE'S OWN WORDS

`src/comms/process.rs`, header line 10:

```
newline-framed bytes → libc::write → io_uring Read → bytes → EDN string
                       ^^^^^^^^^^^   ^^^^^^^^^^^^^
                       raw syscall   Linux-native, cancellable
```

★★★★★ **The reactor got the reads. The writes kept the 1970s.**

```
src/io.rs              10 blocking libc calls,  ZERO io_uring       ← never migrated at all
src/comms/process.rs    3 blocking libc calls,  io_uring on READ only
src/runtime.rs          5
src/process/boot/       5
```

And the copy is documented: `comms/process.rs`'s send poll mirrors *"`io::PipeWriter::write` …
(`src/io.rs`, arc 170 closure #5)."* **`comms` migrated its read path and inherited the un-migrated
write pattern from `io.rs`.**

## WHY THIS IS THE REAL FIX

Stone 1 makes the write non-blocking so the `poll` owns the wait. That is correct and small. But it
leaves the shape: **a hand-rolled `poll` + syscall pair, duplicated in two files, each carrying its
own tie-break reasoning and its own comment about EINTR.**

`IORING_OP_WRITE` — with the shutdown as a linked or cancellable op — removes the shape:

- no poll/write split, so no gap between what the poll promises and what the call demands
- **cancellable by construction**, so signal semantics stop being load-bearing anywhere near the
  transport
- one submission path for reads *and* writes, in a file that already runs a persistent `IoUring`
  (`comms/process.rs:665`, *"capacity 4 covers Read"*)

★ And there is no portability argument left to weigh against it:

```
src/process/mod.rs:9    Linux 5.3+ (clone3 + CLONE_PIDFD + CLONE_CLEAR_SIGHAND), 5.9+
src/process/mod.rs:13   "Production use requires Linux 5.9+; the 6.x floor is …"
```

The builder: *"we are a linux programming language... it screams to me that we are not sticking
that."*

## WHAT WOULD HAVE TO BE DECIDED — the builder's, not mine

1. **Scope.** `comms/process.rs` alone, or `src/io.rs` too? `io.rs` has **zero** io_uring today, so
   migrating it is a larger thing than fixing one send path.
2. **Whether `signalfd` comes along.** Folding signals into the same ring would retire the shutdown
   broadcast pipe as a separate mechanism. That is a real simplification and a real blast radius.
3. **Whether this is arc 278 work at all.** 278 is the rules engine; this is transport substrate.
   The FINDING arrived through 278's perf line, but the work may belong elsewhere.

⚠ **I am not choosing any of these.** `feedback_opening_an_arc_is_the_builders_ruling`.

## WHAT IS ALREADY TRUE AND WORTH KEEPING

- The persistent `IoUring` exists in `comms/process.rs` and is sized for Read (`capacity 4`). A write
  op would need that revisited.
- `try_send` (`:476`) is genuinely non-blocking today via an `O_NONBLOCK` toggle — a third mechanism
  in the same file, which the io_uring path would also subsume.
- Stone 1 does **not** block this. If stone 1 lands, this becomes a cleanup rather than a fix, which
  is a better position to do it from.
