# EXPECTATIONS — a duped fd does not outlive its exec

Written **before** the strike. **This stone changes `src/`** — the floor is the real gate, not a
formality.

Rows state what must be true, not where to look.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ★ **the defect is demonstrated before it is fixed** | run the new probe on the unfixed tree | **RED**, naming the fd and that `FD_CLOEXEC` is clear. A fix with no prior red proves nothing |
| 2 | ★ **every fd the substrate hands out has `FD_CLOEXEC`** | the probe after the fix | `fcntl(fd, F_GETFD) & FD_CLOEXEC != 0` for each fd the io intrinsics and ambient stdio produce |
| 3 | ★ **`OwnedFd::try_clone` is measured, not assumed** | a row in the probe | its result stated as fact — CLOEXEC set or clear. If clear, those sites join the fix |
| 4 | **the post-fork path is left alone** | read the diff | `exec_plan.rs`'s `dup2` calls unchanged; child fd 0/1/2 must **not** be CLOEXEC |
| 5 | **the error paths are unchanged** | read the diff | every `< 0` branch, message, and `RuntimeError` identical; only the acquiring call differs |
| 6 | **bare `dup` survives only where CLOEXEC would be wrong** | a tree-wide invariant test | `libc::dup(` appears only in the post-fork region; anywhere else fails the gate |
| 7 | **the prose matches the syscall** | grep the changed files | no comment still says `dup(2)`/`libc::dup` where the code now calls `F_DUPFD_CLOEXEC` |
| 8 | **blast radius is bounded** | `git diff --stat` | `src/io.rs`, `src/process/stdio.rs`, prose in the two `intrinsic/io/` files, one new test |
| 9 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5225 passed (5224 + this probe), 22 skipped, **0 FAIL, 0 TIMEOUT** |

## The rows that matter most

⚠ **Row 1 is the stone.** This is failure engineering: the failure must be *shown* before it is
pulled. A green probe on a fixed tree, with no red beforehand, cannot tell a fix from a tautology.

⚠ **Row 3 is the one I could not answer.** Rust std sources are not installed on this box, so whether
`OwnedFd::try_clone` uses `F_DUPFD_CLOEXEC` is genuinely unknown here. **Measure it.** Either answer
is a pass; guessing is not.

⚠ **Row 6 is the ladder's top rung available in this material.** The convention is "remember to use
the atomic form"; the check is "a bare `dup` outside the post-fork region fails the build." The
second is what stops the next one, and it is the shape that found a second `libc::signal` installer
when a location-shaped instruction could not.

⚠ **Row 9 is not a formality this time.** Three tests already sit at 19–25 s against a 30 s wall, and
this stone touches code every test loads. Run it on a quiet box.

## Runtime prediction

**40–70 minutes.** Five call sites, one probe, one invariant test, and a prose pass. The floor is the
long pole.

## Trap-doors named in advance

- **`dup2` is not `dup`.** The post-fork `dup2` calls put fds *onto* 0/1/2 and must stay non-CLOEXEC,
  or the child loses its stdio. Do not sweep them up in the change.
- **`F_DUPFD_CLOEXEC` takes a third argument** — the lowest acceptable fd. `0` reproduces `dup`'s
  choice; anything else changes which fd comes back.
- **The ambient-stdio dups run post-fork in a child** that can itself spawn. Being "already in the
  child" is not an exemption.
- **`EXEC_IMAGE_FD` sets `FD_CLOEXEC` explicitly** at `exec_plan.rs:298`. That site is already
  correct; leave it.
- **A red floor is a red.** No test here is pre-blessed; if something goes red, capture it, name the
  arm, and surface it. Do not re-run.

## What this stone does NOT claim

⚠ It does **not** demonstrate the race — it fixes the **property** the race exploits.
⚠ It is **not** stone 3, and does not decide stone 3's arc home.
⚠ It does **not** audit `dup` under `tests/`.
