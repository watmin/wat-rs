# Arc 012 — Fork and Pipes — Backlog

**Opened:** 2026-04-21. Tracking doc, not specification.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.

Captures the author's understanding of the work between where the
codebase is now (arc 011 shipped; hermetic is a Rust primitive
coupled to the wat binary's path) and what retires that coupling (a
kernel-native `fork(2)` + `pipe(2)` substrate with hermetic
reimplemented as wat stdlib).

Every item below ships in the **inscription** mode: build honestly,
write the spec doc that describes what landed. Same pattern as
arc 004 + arc 006 + 058-033 + 058-034. "Blockers as they arise" is
the operating principle — each item's fog resolves when the prior
item lands.

---

## 1. `:wat::kernel::pipe` + `PipeReader` + `PipeWriter`

**Status:** **shipped 2026-04-21.** Commits `4dd2305` (slice 1a —
types + Rust unit tests) and `ee64f40` (slice 1b — primitive +
integration tests).

**Problem:** Rust's stdlib does not expose raw `pipe(2)` — only
`Command::spawn`'s internal plumbing. wat-rs's existing
`:wat::io::IOReader` / `IOWriter` trait surface has StringIo and
RealStdio impls (arc 008) but no fd-backed impl that covers both
sides of a kernel pipe. Without that, slice 2 has no IO abstraction
to hand the forked child (or the parent's pipe ends).

**Approach:**
- New `PipeReader { fd: OwnedFd }` and `PipeWriter { fd: OwnedFd }`
  structs in `src/io.rs`. Both wrap an `OwnedFd`; `Drop` closes via
  `OwnedFd::Drop`'s existing `close(2)`.
- `PipeReader::read_all` / `read` / `read_line` call `libc::read(2)`
  on `fd.as_raw_fd()` in a loop. `PipeWriter::write` / `write_all` /
  `flush` call `libc::write(2)` the same way. No detour through
  `std::io::Read` / `Write`.
- `impl WatReader for PipeReader` + `impl WatWriter for PipeWriter`.
  `Send + Sync` are straightforward — `OwnedFd` is both.
- `rewind()` on `PipeReader`: `RuntimeError::MalformedForm` with
  "pipe fds are not rewindable". Match the semantic `RealStdin`
  already has for real fds.
- `snapshot()` on `PipeWriter`: `None`. Pipes don't buffer in the
  impl; the kernel owns the buffer.
- New primitive `:wat::kernel::pipe -> :(wat::io::IOWriter,
  wat::io::IOReader)`. Calls `libc::pipe(2)` with `O_CLOEXEC`
  (well-formed flag; hygiene against future exec callers even
  though this arc doesn't exec). Wraps the two raw fds as
  `OwnedFd`, produces `(PipeWriter, PipeReader)` as a
  `Value::Tuple`.
- Public constructor path for slice 2: `PipeReader::from_owned_fd(fd)`
  and `PipeWriter::from_owned_fd(fd)` — so slice 2 can construct
  PipeReader/PipeWriter instances over fd 0/1/2 after `dup2`, not
  only from a fresh pipe pair. This is the "dual role" the DESIGN
  calls out.

**Why direct syscall and not `std::fs::File::from_raw_fd`:**
File-backed write paths in Rust stdlib may hold internal buffers or
cooperate with stdio locks. Pipe semantics are literally
`write(fd, buf, n)` — the shortest path is also the most honest.
Avoids any surprise lock inheritance across fork.

**Spec tension:** none yet. Pipe is kernel-level; it doesn't touch
FOUNDATION's algebra or type-system contracts.

**Inscription target:** extension to 058 where `:wat::io::*` is
documented (FOUNDATION's "where each lives" or the IO substrate
section). Arc 008 set the precedent — IO primitives are named in
58's substrate tier.

**Unblocks:** slice 2 (child's stdio wrappers), any future IPC
use case needing direct pipes without fork+exec.

**What shipped (2026-04-21):**
- `PipeReader { fd: OwnedFd }` + `PipeWriter { fd: OwnedFd }` in
  `src/io.rs`. Both impl `WatReader` / `WatWriter`; both `Send +
  Sync`; `Drop` closes via `OwnedFd`. 11 Rust unit tests in
  `io::pipe_tests` — round-trip, partial read, EOF on writer
  dropped, LF + CRLF read-line, bare-line (no trailing `\n`),
  rewind errors, write-return-count, flush, Send+Sync bounds.
- `:wat::kernel::pipe` nullary primitive — dispatch arm in
  `runtime.rs`, type scheme in `check.rs`. Returns
  `Value::Tuple([IOWriter, IOReader])`, writer first.
- 5 integration tests in `tests/wat_pipe.rs` — tuple-shape
  destructuring, writeln→read-line, multi-writeln line-by-line,
  write-string→read-exact-bytes, UTF-8 preservation.

**What was deferred (decisions resolved by not-doing):**
- `O_CLOEXEC` / `pipe2(2)` usage. This arc doesn't exec, so
  close-on-exec doesn't affect the fd lifecycle (dup2 + close
  manage it); plain `libc::pipe(2)` is the honest minimum.
  A future exec-using arc (`spawn-process`) would revisit.
- Rust stdlib's `std::fs::File::from_raw_fd` wrapper path. The
  shortest path — `libc::read/write` on `fd.as_raw_fd()` —
  sidesteps Rust's file-level buffering and locking layers
  entirely. No surprise fork inheritance.

**Caught in passing:**
- Memory feedback entry on wat keyword-path types forbidding
  interior whitespace (`:(A, B)` is a lexer error;
  `:(A,B)` is required). Sibling to the existing colon-quoting
  discipline. Integration tests caught this on first run; memory
  saved so future sessions don't re-trip.

---

## 2. `:wat::kernel::fork-with-forms` + `ForkedChild` + `ChildHandle` + `wait-child`

**Status:** **slice 2 shipped 2026-04-21** — commits `bd68a4e`
(core: fork-with-forms + ForkedChild + ChildHandle) and
`16a5fe5` (wait-child). All three sub-fogs resolved.

**Problem:** `Command::spawn` paired with `--current-exe` gives
hermetic the process isolation it needs, but at the cost of
binary-path coupling, a tempfile write, and a full re-parse in the
subprocess. Raw `fork(2)` supplies the same isolation via COW —
the child already has the parent's loaded wat runtime, already has
the caller's `Vec<WatAST>` in inherited heap — but raw `fork(2)`
isn't in Rust stdlib. wat owns the abstraction. This slice names
it.

**Approach:**
- New opaque struct `ChildHandle { pid: libc::pid_t, reaped:
  AtomicBool }`. `Drop` sends `SIGKILL` and blocks on
  `waitpid(pid, ..., 0)` if `reaped` is still false. Zombie-free
  even on early-exit paths.
- New struct `:wat::kernel::ForkedChild` with four positional
  fields matching DESIGN: `handle`, `stdin`, `stdout`, `stderr`.
  Register via existing struct-registration machinery so
  `ForkedChild/stdin` etc. accessors land automatically alongside
  `struct-new`.
- Primitive `:wat::kernel::fork-with-forms (forms
  :Vec<wat::WatAST>) -> :wat::kernel::ForkedChild`:
  1. Create three pipe pairs via `libc::pipe(2)`.
  2. Call `libc::fork()` (wrapped in `unsafe` with a full doc-
     comment justifying the async-signal-safety floor per POSIX).
  3. Child branch (`pid == 0`):
     - `dup2` the pipe ends onto fd 0 / 1 / 2.
     - Close every other inherited fd. Strategy resolves when code
       lands (sub-fog 2a below).
     - `PipeReader::from_owned_fd(0)` + two `PipeWriter` over 1 / 2
       — the child's three IO arguments.
     - Build `InMemoryLoader` (scope deferred).
     - `startup_from_forms(forms, None, loader)` → fresh
       `FrozenWorld`. On error, write diagnostic to stderr, exit
       with the startup-error code (sub-fog 2b).
     - `validate_user_main_signature(&world)` — same
       three-IO-arg contract arc 007 enforces.
     - `invoke_user_main(&world, [pipe-in, pipe-out, pipe-err])`
       inside `catch_unwind`. On `Err(runtime_err)` or panic
       payload, write diagnostic to stderr, exit with the
       runtime-error code.
     - On success: `libc::_exit(0)`. Use `_exit`, not
       `std::process::exit`, to skip atexit handlers that the
       parent registered. `_exit` is async-signal-safe and
       doesn't touch anything else in the inherited heap.
  4. Parent branch (`pid > 0`):
     - Close the child-side fds (stdin-read, stdout-write,
       stderr-write).
     - Wrap parent-side fds as `PipeWriter` / `PipeReader`.
     - Build `ChildHandle { pid, reaped: false }` and wrap in
       an `Arc<ChildHandle>` so the struct and future
       `wait-child` calls share reap state.
     - Return `Value::Struct(ForkedChild { handle, stdin,
       stdout, stderr })`.
- Primitive `:wat::kernel::wait-child (handle
  :wat::kernel::ChildHandle) -> :i64`: `waitpid(handle.pid,
  &status, 0)`; set `handle.reaped = true`; extract exit code
  from the waitpid status via `WEXITSTATUS(status)` on normal
  exit, or a signal-encoded code on signal termination.
  Exit-code → i64 convention pinned in sub-fog 2b below.

**Sub-fog 2a — close-inherited-fds strategy.** Options:
- `nix::unistd::close_range` if adding `nix` as a dep is
  acceptable (one more crate; not on today).
- `libc::close_range` (Linux 5.9+, macOS via libSystem) when
  available.
- Portable fallback: read `/proc/self/fd` on Linux and
  `/dev/fd` on BSD-derivatives; close anything except 0 / 1 / 2.
- Lazy option: call `fcntl(fd, F_SETFD, FD_CLOEXEC)` on everything
  at parent-side pipe setup. Doesn't help for non-fork-owned fds
  the parent opened before this call; only hygienic for THIS
  arc's pipes.
Pick concrete strategy when slice 2 code lands — the wrong
strategy has visible symptoms (child hangs waiting on a fd that
stays open because the child inherited a duplicate).

**Sub-fog 2b — exit-code convention.** Slice 3 reads these codes
back on the parent side to reconstruct Failure. Candidate
convention:
- `0` — success
- `1` — `RuntimeError` from `:user::main`
- `2` — panic in `:user::main`
- `3` — startup error (parse, types, macros)
- `4` — `:user::main` signature validation failure
- `64–79` — reserved for future diagnostic categories

Signal termination encodes differently (`128 + signum` per shell
convention). Slice 2's ShipResult must document the exact choice
so slice 3's reconstruction is reversible.

**Sub-fog 2c — `wait-child` on a handle that's already reaped.**
Double-`wait-child` on the same handle: error, or return the
cached exit code? Preliminary: return cached. Matches how Rust's
`Child::try_wait` handles "already waited" — idempotent. Pin
when slice 2 lands.

**Spec tension to name honestly:** FOUNDATION currently describes
hermetic as "a subprocess the kernel spawns" in its kernel-
primitives section. This arc rewrites it as "a forked child
running `:user::main`" — the isolation contract stays the same
(fresh FrozenWorld in the child, captured stdio, exit-code
failure reporting) but the mechanism no longer requires locating
the wat binary. Inscription target: FOUNDATION amendment +
possibly a new sub-proposal `058-NNN-fork-substrate` that
inscribes the fork primitives alongside the kernel tier (arc 006
→ 058-034 precedent).

**Unblocks:** slice 3 (wat-stdlib reimplementation of hermetic);
any future subprocess use case that wants "a fresh wat
evaluation on top of the same runtime" (daemon parallelism,
replay runners, test-per-process harnesses).

**What shipped for slice 2 core (2026-04-21, commit `bd68a4e`):**
- `src/fork.rs` — new module. `ChildHandleInner` (pid + reaped
  AtomicBool, Drop SIGKILLs + reaps if never waited);
  `make_pipe` helper; `close_inherited_fds_above_stdio`;
  `eval_kernel_fork_with_forms`; `child_branch` (never-returns,
  full in-child pipeline).
- `Value::wat__kernel__ChildHandle(Arc<ChildHandleInner>)`
  variant in `runtime.rs` + type_name arm.
- Dispatch arm `":wat::kernel::fork-with-forms"` in `runtime.rs`.
- `:wat::kernel::ForkedChild` StructDef in `types.rs` with four
  fields (handle, stdin, stdout, stderr). Auto-generated `/new`
  + per-field accessors land at freeze via
  `register_struct_methods`.
- Type scheme for `fork-with-forms` in `check.rs` —
  `:Vec<wat::WatAST>` → `:wat::kernel::ForkedChild`.
- 3 integration tests in `tests/wat_fork.rs` covering parent-
  reads-child-stdout, parent-reads-child-stderr, parent-writes-
  child-stdin-and-echo.

**Sub-fogs resolved:**

- **2a — close-inherited-fds strategy.** `/proc/self/fd` (Linux)
  or `/dev/fd` (macOS) iteration. First pass tried to close
  fds mid-iteration; the dir-reader's own fd was in the listing,
  closing it mid-walk panicked glibc's `closedir`. Fix: collect
  candidate fds from the listing, let the iterator drop cleanly,
  then close the collected fds (the iterator's fd is already
  closed by then — subsequent `close()` returns EBADF which we
  ignore). "Collect first, close after" is the honest pattern
  for fd-iteration-during-teardown.

- **2b — exit-code convention.** Five codes pinned and exposed
  as `pub const` in `src/fork.rs`:
  - `EXIT_SUCCESS` = 0
  - `EXIT_RUNTIME_ERROR` = 1
  - `EXIT_PANIC` = 2
  - `EXIT_STARTUP_ERROR` = 3
  - `EXIT_MAIN_SIGNATURE` = 4
  Slice 3's hermetic reconstruction will import these. Signal
  termination encodes as `128 + signum` via `WTERMSIG`
  (shell convention — readable as a normal `:i64` exit code
  alongside the EXIT_* slots without a separate discriminator).

- **2c — double-wait-child behavior (resolved by `16a5fe5`).**
  `ChildHandleInner` grew a `cached_exit: OnceLock<i64>`. First
  `wait-child` runs `waitpid` + caches + flips reaped; subsequent
  calls return the cached code. Idempotent — matches Rust
  `Child::try_wait` semantics. `OnceLock` is the honest
  primitive here (publish-once-read-many, not a lock); sits in
  ZERO-MUTEX.md's named exceptions alongside atomics and Arc.

**Caught in passing:**
- `:wat::io::IOWriter/writeln` returns `:i64` (byte count), not
  `:()`. A `:user::main` with `-> :()` signature can't use
  writeln as its tail expression. Use `IOWriter/println`
  (returns `:()`) for unit-returning main bodies, or bind
  writeln's return via `let*` and return `()` explicitly.
  Caught by the first round of integration tests.

---

## 3. Wat-stdlib reimplementation of `run-sandboxed-hermetic-ast`

**Status:** obvious in shape; Failure reconstruction has fog that
resolves when slice 2 lands and the exit-code convention is
pinned.

**Problem:** the current Rust primitive
`eval_kernel_run_sandboxed_hermetic_ast` (arc 011) reads as ~80
lines of Rust spanning binary-path lookup, tempfile I/O,
`Command::spawn`, child stdin write, wait + output capture, byte-
to-line splitting, and `RunResult` construction. Most of those
lines are glue — once slices 1+2 ship `fork-with-forms` +
`wait-child` + fd-backed IOReader/IOWriter + `:wat::core::string::split`
(already present) + `:wat::core::struct-new` (already present),
the reimplementation is composition.

**Approach:**
- New define in `wat/std/kernel/hermetic.wat` (or fold into
  existing `wat/std/test.wat` — decide at implementation):
  `(:wat::core::define (:wat::kernel::run-sandboxed-hermetic-ast
  (forms :Vec<wat::WatAST>) (stdin :Vec<String>) (scope
  :Option<String>) -> :wat::kernel::RunResult) ...)`.
- Body follows DESIGN's sketch: fork-with-forms → inner scope
  writes stdin → read-all stdout + stderr → wait-child → split to
  line vecs → struct-new `:wat::kernel::RunResult` with the parts.
- Scope-parameter behavior matches today: Failure when `Some`.
  wat-level check at the top; no fork needed for the failing
  path.
- Retire `eval_kernel_run_sandboxed_hermetic_ast` from
  `src/sandbox.rs`. The primitive registration vanishes; the
  stdlib define picks up the same keyword path.
- Decide on string-entry hermetic
  (`eval_kernel_run_sandboxed_hermetic`) — keep as Rust
  (parse-then-fork), reimplement as wat (parse + fork-with-forms),
  or retire entirely. Resolve when slice 3 is sitting in the
  editor and the caller surface is visible.

**Fog 3a — Failure reconstruction.** From `(exit_code: i64,
stderr_str: String)` on the parent side, how do we reconstruct
the `Option<Failure>` that current hermetic stuffs in RunResult?
Options:
- Simple: `exit_code == 0` → `:None`; anything else → `(Some
  (Failure { message: stderr_str, ... }))` with all other
  fields `:None`. Loses detail current hermetic keeps
  (distinguishing startup from runtime from panic).
- Layered: parse an agreed-upon prefix on the child's stderr
  diagnostic line (e.g., `"WAT-FAILURE-MODE: runtime\n<detail>"`)
  and split into fields. Slightly ugly; implies a protocol
  between child's exit path and parent's reconstruction.
- Exit-code dispatch: use sub-fog 2b's exit-code convention to
  pick the failure kind, use stderr_str as the message. Cleaner
  than prefix-parsing; relies on slice 2 pinning the codes.

Preliminary: exit-code dispatch is the right shape, resolves when
slice 2 ships.

**Fog 3b — the AST-to-source serializer's fate.** Arc 011's
`wat_ast_to_source` and `wat_ast_program_to_source` were born to
serialize forms for the subprocess's re-parse. Fork removes the
need — the child reads inherited memory. Options:
- Retire: delete the functions; 200 lines drop from `src/ast.rs`.
- Keep: no caller yet, but a future use case might want source-
  text round-trip (pretty-printer, REPL history, debugger, etc.).
- Expose as a stdlib primitive `:wat::core::ast-to-source` for any
  wat-level caller who wants AST→String.
Resolve when slice 3 is ready to commit and the caller surface
is known.

**Spec tension to name honestly:** the hermetic primitive moves
from Rust substrate to wat stdlib. 058's description of hermetic
currently says "kernel-registered primitive"; post-slice-3 it's
"stdlib-defined, kernel-dependent." FOUNDATION amendment needed
— hermetic's contract stays, its implementation layer shifts.

**Inscription target:** same 058 amendment as slice 2's. One
sub-proposal (or one FOUNDATION section) inscribing fork + pipes
+ stdlib-hermetic as a package.

**Unblocks:** any future subprocess pattern written as wat
stdlib. Also retires one of the last places where "the wat
binary needs to know its own path on disk" leaked into the
language surface.

---

## Open questions carried forward

Resolve when the slice they depend on lands. Listed here so they
don't evaporate.

- **`wait-child` return shape.** `:i64` exit code vs richer
  `:wat::kernel::ExitStatus` struct (normal / signal; signum;
  core-dump flag). Start `:i64`; upgrade if a caller demands
  signal introspection.
- **`wait-child` on already-reaped handle.** Sub-fog 2c.
- **Non-blocking `try-wait-child`.** Future. Not needed by slice
  3's hermetic; a daemon-supervision caller would demand it.
- **Scope parameter honored inside fork.** Requires wiring
  ScopedLoader through to the child's `startup_from_forms`.
  Separate slice when a caller demands.
- **Kill primitive.** `:wat::kernel::kill-child handle signal`.
  Scope-based reaping via Drop covers misuse; explicit kill
  when a caller surfaces demand.
- **Retire vs keep `run-sandboxed-hermetic` (string-entry) Rust
  primitive.** Fog 3a-adjacent. Resolve in slice 3.
- **Retire vs keep `wat_ast_program_to_source`.** Fog 3b.

---

## What this arc does NOT ship

Explicitly deferred (not merely "fog"):

- **Windows support.** Fork doesn't exist there. Unix-only is
  a named non-goal.
- **`spawn-process` fork+exec primitive.** Calling external
  binaries (sqlite3, ffmpeg, etc.) via argv. Different shape:
  exec replaces the process image; this arc keeps it.
- **`current-exe` / `env-var` / `write-tempfile` primitives.**
  They were on the old Path-A plan. Fork removes the need.
  Ship when a future caller demands.
- **Scope forwarding into the fork child.** Noted above.
- **Cross-thread survival across fork.** POSIX says only the
  forking thread survives. Any kernel-spawned worker threads
  in the parent are gone in the child. Hermetic doesn't need
  them — the child runs a fresh `startup_from_forms` world.
  A future "fork-and-keep-workers" primitive (if ever
  demanded) would be a separate arc and would have to solve
  the inherited-channel-handles problem.

---

## Why this matters

Three prior arcs factored ceremony into substrate: names-are-
values (arc 009), forms-are-values (arc 010), AST-entry hermetic
(arc 011). Arc 012 removes the last coupling: "hermetic requires
the wat binary's path." The new coupling is honest — hermetic
requires fork, and fork IS a substrate capability every Unix
process has.

The reimplementation in wat stdlib closes the loop. Three
substrate primitives (`pipe`, `fork-with-forms`, `wait-child`) +
the existing arc 008 IO traits + arc 010's `program` macro +
arc 009's name-as-value convention + existing `struct-new` +
existing `string::split` = a wat-stdlib hermetic that reads like
the program it runs.

That's the substrate teaching itself to listen, one more time.
