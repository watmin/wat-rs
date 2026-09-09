# SCORE — a duped fd does not outlive its exec

**SCORED.** Executor: grok, 2026-09-08. This stone changes `src/`. The
defect was demonstrated red, then the five `libc::dup` sites became
`fcntl(F_DUPFD_CLOEXEC)`, then the same probe went green. Did not
commit.

```
Summary [ 497.290s] 5225 tests run: 5225 passed (7 slow), 22 skipped
```

`.floor/2026-09-08T08-17-16Z/`

## THE RED — before the fix

Isolated `cargo test --release -p wat --test comms a_duped_fd_does_not_outlive_its_exec -- --nocapture` on the unfixed tree (HEAD `774c89fe7`, no `src/` edits yet). FAILED in 0.02 s:

```
io-writer fd=6 FD_CLOEXEC=clear
io-reader fd=7 FD_CLOEXEC=clear
ambient-stdin fd=8 FD_CLOEXEC=clear
ambient-stdout fd=9 FD_CLOEXEC=clear
ambient-stderr fd=10 FD_CLOEXEC=clear
OwnedFd::try_clone fd=12 FD_CLOEXEC=SET
bare libc::dup(  src/process/stdio.rs:80  let r = unsafe { libc::dup(fd) };
bare libc::dup(  src/io.rs:1364  let dup_fd = unsafe { libc::dup(fd as libc::c_int) };
bare libc::dup(  src/io.rs:1402  let dup_fd = unsafe { libc::dup(fd as libc::c_int) };

a duped fd must not outlive its exec; 8 finding(s):
io-writer fd=6 FD_CLOEXEC is clear
io-reader fd=7 FD_CLOEXEC is clear
ambient-stdin fd=8 FD_CLOEXEC is clear
ambient-stdout fd=9 FD_CLOEXEC is clear
ambient-stderr fd=10 FD_CLOEXEC is clear
bare libc::dup( outside post-fork: src/process/stdio.rs:80  let r = unsafe { libc::dup(fd) };
bare libc::dup( outside post-fork: src/io.rs:1364  let dup_fd = unsafe { libc::dup(fd as libc::c_int) };
bare libc::dup( outside post-fork: src/io.rs:1402  let dup_fd = unsafe { libc::dup(fd as libc::c_int) };
```

STOP-1 did not fire. The property was violated on every fd `from-fd` and
`lend_ambient` hand out. The three remaining `libc::dup(` in `src/` were
those sites.

## `OwnedFd::try_clone` — measured, not assumed

**`FD_CLOEXEC=SET`.** Same isolated run, same pipe: `OwnedFd::try_clone fd=12 FD_CLOEXEC=SET`.

`src/comms/process.rs` Clone and `sender_receiver_from_fd_with_budget` do
not join the fix. Comments there still say `libc::dup`; the measurement
is that this box's std already uses a CLOEXEC-atomic clone. Blast radius
forbade touching that file unless CLOEXEC was clear.

## THE GREEN — after the fix

Same isolated command after switching the three call sites (stdio helper
covers ambient 0/1/2):

```
io-writer fd=6 FD_CLOEXEC=SET
io-reader fd=7 FD_CLOEXEC=SET
ambient-stdin fd=8 FD_CLOEXEC=SET
ambient-stdout fd=9 FD_CLOEXEC=SET
ambient-stderr fd=10 FD_CLOEXEC=SET
OwnedFd::try_clone fd=12 FD_CLOEXEC=SET
test probe_arc278_duped_fd_does_not_outlive_its_exec::a_duped_fd_does_not_outlive_its_exec ... ok
```

Floor: `PASS [   0.037s] wat::comms probe_arc278_duped_fd_does_not_outlive_its_exec::a_duped_fd_does_not_outlive_its_exec`.

## WHAT LANDED

Acquisition only, the form already used at `exec_plan.rs:118`:

```rust
let dup_fd = unsafe { libc::fcntl(fd as libc::c_int, libc::F_DUPFD_CLOEXEC, 0) };
```

| site | before | after |
|---|---|---|
| `src/io.rs` `eval_iowriter_from_fd` | `libc::dup` | `fcntl(F_DUPFD_CLOEXEC, 0)` |
| `src/io.rs` `eval_ioreader_from_fd` | `libc::dup` | `fcntl(F_DUPFD_CLOEXEC, 0)` |
| `src/process/stdio.rs` `dup_fd` (ambient 0/1/2) | `libc::dup` | `fcntl(F_DUPFD_CLOEXEC, 0)` |

`< 0` branches, `format!("dup(2) on fd {fd} failed: {e}")`, and the
ambient abort `write`/`_exit` bytes are byte-identical. `exec_plan.rs`
`dup2` untouched. Prose in `io.rs`, `stdio.rs`, `intrinsic/io/writer.rs`,
`intrinsic/io/reader.rs` now names `F_DUPFD_CLOEXEC`.

`libc::dup(` no longer appears in `src/` (the post-fork region uses
`dup2`, which the gate does not match). Reintroducing a bare `dup`
outside `exec_plan.rs` fails the probe.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★ defect demonstrated before the fix | ✅ red output above; five substrate fds `FD_CLOEXEC=clear`; three bare `libc::dup(` |
| 2 | ★ every substrate fd has `FD_CLOEXEC` | ✅ all five `SET` after the fix |
| 3 | ★ `OwnedFd::try_clone` measured | ✅ **SET**. `process.rs` not edited |
| 4 | post-fork `dup2` left alone | ✅ `git diff -- src/process/exec_plan.rs` empty |
| 5 | error paths unchanged | ✅ `< 0` / `RuntimeError` / abort diagnostics identical; only the acquiring call differs |
| 6 | bare `dup` only where CLOEXEC would be wrong | ✅ tree-wide walk in the same probe; zero `libc::dup(` in `src/` after the fix |
| 7 | prose matches the syscall | ✅ comments at the changed sites name `F_DUPFD_CLOEXEC`; error strings kept as `dup(2)` per STOP-3 |
| 8 | blast radius | ✅ `src/io.rs`, `src/process/stdio.rs`, two `intrinsic/io/` prose files, one new test. Not `process.rs` |
| 9 | the floor | ✅ `Summary [ 497.290s] 5225 tests run: 5225 passed (7 slow), 22 skipped` |

STOP-1 / STOP-2 / STOP-3 / STOP-4 held. Did not re-run the floor.

0 FAIL, 0 TIMEOUT.

## BLAST

```
 src/intrinsic/io/reader.rs | 10 +++++-----
 src/intrinsic/io/writer.rs | 10 +++++-----
 src/io.rs                  | 29 +++++++++++++++--------------
 src/process/stdio.rs       |  5 +++--
?? tests/comms/probe_arc278_duped_fd_does_not_outlive_its_exec.rs
```
