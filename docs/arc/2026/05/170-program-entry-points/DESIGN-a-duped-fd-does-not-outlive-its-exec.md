# DESIGN — a duped fd does not outlive its exec

**This stone changes `src/`.** The prior three were measurement only; this one demonstrates a defect,
fixes it, and leaves a gate that catches the next one.

## The defect

`libc::dup(2)` is **not** CLOEXEC-atomic. The fd it returns has `FD_CLOEXEC` clear, so it survives
into any `exec` that happens afterward. `FD_CLOEXEC` and `F_SETFD` appear **nowhere** in `src/`
outside `exec_plan.rs`, so nothing sets it later either.

★ **Every other fd-creating call in this tree is CLOEXEC-atomic, deliberately and with comments
saying why:**

```
pipe2            ×16   bare libc::pipe()  ZERO
socket                 SOCK_STREAM | SOCK_NONBLOCK | SOCK_CLOEXEC   "so it does not leak across an unrelated exec"
open                   O_PATH | O_CLOEXEC
timerfd_create         TFD_NONBLOCK | TFD_CLOEXEC                   "atomic at creation"
exec_plan.rs:118       F_DUPFD_CLOEXEC                              "async-signal-safe; never picks an open fd"
```

`libc::dup` is the sole outlier — and it is in `src/io.rs`, the same file that had zero io_uring and
the blocking-write pattern. **The 1970s corner of this codebase is a place, not an accident**, and
both times the correct instrument was already in the tree.

## Why it bites here specifically

`comms/process.rs:63`, `:1063`, `:1082`, `:1108` — the transport uses **`POLLHUP` as SEVER**:

> data fd: `POLLIN | POLLHUP` (data ready **OR EOF**) … SEVER (POLLHUP, the drop)

A leaked copy of a pipe's write end **holds the pipe open**, so the read end never sees `POLLHUP`.
The peer waits forever for a hangup that cannot arrive. **That is a hang, not a leak** — and it would
present exactly like the class this arc has spent a month chasing.

## Reachability — established, not assumed

```
kernel/spawn.rs:693                std::thread::Builder::spawn         services run on threads
io.rs:1343 / :1383                 eval_iowriter_from_fd / …reader     wat intrinsic, bare libc::dup
intrinsic/kernel/resource.rs:401   "forks a real OS process"           wat intrinsic, clone3 + exec
```

Both sides are wat-callable intrinsics and services run on separate threads, so one service dup'ing
while another spawns is an ordinary program.

⚠ **The stone does not depend on winning that race.** The race is how it *bites*; the **property** is
what is wrong, and a property is deterministic to gate. Gate on the property, not the path.

## The sites

| site | verdict |
|---|---|
| `io.rs:1364`, `io.rs:1402` | ⛔ bare `dup`, parent-side, handed to wat as a service fd |
| `process/stdio.rs:92-94` | ⛔ bare `dup` ×3 for ambient stdio — *child*-side, but that child can itself `spawn-process'` |
| `comms/process.rs:1013-1025`, `:2094` | ❓ `OwnedFd::try_clone()`. The comments call it `libc::dup`. **Whether std uses `F_DUPFD_CLOEXEC` is UNVERIFIED on this box — measure it, do not assume it** |
| `exec_plan.rs:290-297` | ✅ `dup2` post-fork onto 0/1/2 and `LIFELINE_FD`. CLOEXEC would be **wrong** here; `EXEC_IMAGE_FD` sets it explicitly at `:298` |

## What it delivers

1. **A property gate** — every fd this substrate hands out has `FD_CLOEXEC` set. Deterministic; no
   race to win. It is expected to be **RED before the fix** — that is the demonstration.
2. **The fix** — `libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 0)` at the five bare-`dup` sites, the form
   already used at `exec_plan.rs:118`. Identical return contract: a fresh fd, or `-1`.
3. **A tree invariant** — bare `libc::dup(` survives only where CLOEXEC would be wrong. This is the
   gate that finds the *next* one, and it is the shape that found the second `libc::signal` installer
   when a path-shaped instruction could not.

## The one contract decision

**`F_DUPFD_CLOEXEC` returns the lowest fd ≥ the third argument that is free — the same fd `dup` would
have returned — and `-1` on failure.** Every existing error path stays exactly as written.

## Out of scope — REJECTED

- `exec_plan.rs`'s `dup2` calls. Correct as they stand.
- Stone 3 and the io_uring migration. Separate, and its arc home is the builder's ruling.
- Auditing `dup` in `tests/`. The invariant is about what the substrate hands out.

## Files

`src/io.rs`, `src/process/stdio.rs`, one new test file, and the prose in the files that name the
syscall (`intrinsic/io/writer.rs`, `intrinsic/io/reader.rs`, and `io.rs`'s own doc comments).
