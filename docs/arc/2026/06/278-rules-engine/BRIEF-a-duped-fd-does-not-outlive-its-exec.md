# BRIEF — a duped fd does not outlive its exec

## The work, in one paragraph

`libc::dup(2)` hands back an fd with `FD_CLOEXEC` clear, so it rides into any later `exec`. Five
sites in this substrate use it; every other fd-creating call in the tree is already CLOEXEC-atomic.
Write a probe that **demonstrates** the property is violated, switch those five sites to the atomic
form already used at `exec_plan.rs:118`, watch the probe go green, and leave a tree-wide invariant so
the next bare `dup` fails the build.

## Read in order

1. **`src/process/exec_plan.rs:110-120`** — the exemplar. `F_DUPFD_CLOEXEC` used correctly, with the
   comment explaining why it is safe. Copy this call shape.
2. **`src/io.rs:1331-1375`** and **`:1379-1412`** — two of the five sites. Read the doc comment at
   `:1331` ("DUP-then-own (load-bearing)") and `:1362` ("the service owns a PRIVATE copy") — the
   prose describes a privacy the fd does not have.
3. **`src/process/stdio.rs:70-95`** — the other three, inside `dup_fd`. One change fixes all three.
4. **`src/comms/process.rs:1013-1025`** and **`:2094`** — `OwnedFd::try_clone()`, which the comments
   call `libc::dup`. **Measure whether it sets CLOEXEC** (row 3); if it does not, these join the fix.
5. **`src/comms/process.rs:63`, `:1063`, `:1082`, `:1108`** — where `POLLHUP` is read as EOF/SEVER.
   This is why a leaked fd is a hang rather than a leak. You are not editing these.

## Implementation sketch

The acquisition changes; nothing else does.

```rust
// before
let dup_fd = unsafe { libc::dup(fd as libc::c_int) };

// after — the form already used at exec_plan.rs:118
let dup_fd = unsafe { libc::fcntl(fd as libc::c_int, libc::F_DUPFD_CLOEXEC, 0) };
```

`F_DUPFD_CLOEXEC` with a third argument of `0` returns the lowest free fd — the same one `dup` would
have picked — or `-1`. **Every `< 0` branch, error message and `RuntimeError` stays byte-identical.**

The probe, in two parts:

```rust
// Part 1 — the property, per fd the substrate hands out.
fn cloexec_set(fd: RawFd) -> bool {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    assert!(flags >= 0, "F_GETFD on a live fd");
    flags & libc::FD_CLOEXEC != 0
}
// assert cloexec_set(...) for: an io-writer fd, an io-reader fd, an ambient-stdio fd,
// and an OwnedFd::try_clone() result — reporting the try_clone answer either way.

// Part 2 — the invariant. Bare `dup` survives only where CLOEXEC would be wrong.
// Walk src/**/*.rs; a `libc::dup(` outside the post-fork region fails.
```

## Blast radius

`src/io.rs` (2 sites), `src/process/stdio.rs` (1 helper covering 3), one new test file, and the prose
in `src/intrinsic/io/writer.rs` and `src/intrinsic/io/reader.rs` that names the syscall. Plus
`src/comms/process.rs` **only if** row 3 shows `try_clone` leaves CLOEXEC clear.

## STOP triggers

**STOP-1** — if the probe passes on the **unfixed** tree, STOP. Either the property is already held
somewhere you did not expect, or the probe is not measuring what it claims. Report which, and do not
proceed to a fix that would prove nothing.

**STOP-2** — if any `exec_plan.rs` `dup2` call appears to need changing, STOP. Those put fds onto
0/1/2 in the child and must stay non-CLOEXEC; a change there breaks the child's stdio.

**STOP-3** — if the fix requires altering an error path, a message, or a returned `RuntimeError`,
STOP and surface it. Only the acquiring call is in scope.

**STOP-4** — on any red floor arm, STOP, capture it whole, name the exact arm, and surface it. No
test here is pre-blessed and a red is a red. Do not re-run it.

## What "done" looks like

The probe is red on the tree as you found it, green after the fix, and the invariant test fails if a
bare `libc::dup(` is reintroduced outside the post-fork region. The floor's Summary line reads 5225
passed / 22 skipped / 0 FAIL / 0 TIMEOUT, run on a quiet box. The SCORE states the `try_clone`
measurement as a fact either way, and shows the probe's **red output before** the fix alongside its
green after — the red is the evidence, not a preamble.
