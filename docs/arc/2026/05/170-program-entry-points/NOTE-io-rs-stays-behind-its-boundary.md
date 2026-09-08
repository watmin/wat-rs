# NOTE (arc 170) — "io.rs joins the reactor" is REJECTED on the four questions, and my framing of it was wrong twice

**Filed 2026-09-08.** Builder: *"i agree with this - the intent of this is just file system access?..."*
It is not. That question is what closed the item.

This file replaces `DESIGN-io-rs-is-three-transports-wearing-one-name.md`, whose **title asserted the
thing this note disproves.** Renamed rather than left standing, because a stale title is the kind of
graveyard that reads identically to live code.

## The item, and where it came from

`FINDING-the-writes-kept-the-1970s.md` (arc 278) named three stones. The third —
*"the write path joins io_uring"* — was scoped to `comms/process.rs` by the four questions and shipped
as `the sender grows a ring` (`c2a8e45e3`). `src/io.rs` was carried forward as *"its own stone once
ring ownership has an answer."*

Ring ownership **was** answered. The item still fails, for reasons that have nothing to do with rings.

## ⛔ REJECTED — the four questions

Applied to *migrate `src/io.rs`'s writers to io_uring*:

| | |
|---|---|
| **Obvious?** | **NO** — a reader must be able to answer *"why does a string buffer need a reactor?"* There is no answer. |
| **Simple?** | **NO** — it needs either fd-kind interrogation at construction (`fstat` + `S_IFMT`; **zero precedent in this tree** — `grep` for `S_ISFIFO`/`S_IFMT`/`S_ISREG`/`fstat` in `src/` returns nothing) or a ring handed to fds that can never park. |
| **Honest?** | **NO** — see below. This is the decisive one. |
| Good UX? | not reached; the first three must hold. |

## Why *Honest* fails — the abstraction's own contract forbids it

`src/io.rs:1-6` states the intent, and it is not filesystem access:

> *"Wat needs substitutable stdio: in production, a wat program receives real OS Stdin/Stdout/Stderr;
> in tests, the same program receives string-buffer stand-ins. Both must fit a single wat-level type
> so the source is identical. Ruby's StringIO model."*

**`io.rs` is a substitutability boundary.** Its promise is that *the caller cannot tell what is
underneath.* The filesystem verbs (`open-file`, `read-file`, `list-dir`, temp files) and
`eval_kernel_pipe` arrived **on top** of that, as things which also fit `IOReader`/`IOWriter`.

io_uring requires knowing exactly what is underneath — an fd, and a specific kind of fd. The trait
already records that this cannot be assumed. `io.rs:104-110`:

```rust
/// The raw fd this writer backs, if any … `None` for non-FD-backed writers
/// (RealStdout, StringIoWriter); overridden in PipeWriter to return Some(self.fd).
fn as_raw_fd_for_poll(&self) -> Option<i32> { None }
```

★★★ **Having an fd at all is `Option`al, and the default is `None`.** Verified by counting overrides:
`RealStdout` none, `StringIoWriter` none, `PipeWriter` overrides. **Two of the seven `WatWriter`/
`WatReader` impls have no fd whatsoever.** For `StringIoWriter` a reactor is not wasteful — it is
**incoherent**.

## The uniform wait is correct for every kind — measured, not assumed

```
poll(POLLOUT) on a REGULAR FILE     → READY in 3 µs
poll(POLLOUT) on /dev/null          → READY
4 MiB write, blocking vs O_NONBLOCK → 4194304 both; no short write
```

So today's `poll` + `O_NONBLOCK` path (arc 278 stone 1) is **correct on a pipe, a regular file, a char
device, and a socket alike**. It is load-bearing where the wait is real (pipes) and merely costs a
ready-immediately `poll` where it is not.

## The `poll` rider — also rejected, and bounded

Removing that `poll` for non-pipe fds: **Obvious? YES. Simple? NO** — knowing an fd is always-ready
needs the same absent fd-kind machinery. And the size argues against it: **3 µs** against this arc's own
measured units — bare round trip **179 µs**, `Store/put` **675 µs** — is ~1.7 % of a round trip.

**Rejected pending a measurement that exhibits a write volume where 3 µs matters.** That is the bound:
not "later", but *"this specific measurement, or not at all."*

## ★★ Two framings of mine that were wrong, kept visible

**First:** I told the builder `io.rs` had *"got easier while we weren't looking"* because
`the sender grows a ring` settled ring ownership. Ring ownership was never the obstacle.

**Second:** I then filed a DESIGN titled *"three transports wearing one name"*, reading `PipeWriter` as
a braided type that had failed to use an existing seam, and calling the stone `partire`. **That had the
direction backwards.** The seam is *deliberately above* `PipeWriter`: `IOWriter` is the substitutability
boundary, and `PipeWriter` is simply the one impl that sits on something with a real wait. Nothing needs
splitting. The reactor belongs **below** the boundary — in `comms/process.rs`, where it now lives.

★ Both errors had the same shape as the ones this arc spent the day extirpating: **a structure asserting
a guarantee that is not load-bearing.** `comms/process.rs:353` and `io.rs:670` wrote their safety down in
prose; a reactor on a `StringIoWriter` would write it down in types. Same failure, different medium.

## What this closes

`src/io.rs` is **done**, not deferred. It is correct for every fd kind it serves, its wait is
multiplexed where a wait exists, its two `dup` sites became `F_DUPFD_CLOEXEC` (`c5ba2da3f`), and the
reactor question is answered NO on the abstraction's own contract.

⚠ **No follow-up item is created.** The one open question in this line is the superlinear drain
(arc 278), which is unrelated to this file.
