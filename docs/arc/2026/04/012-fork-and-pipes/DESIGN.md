# Arc 012 — Fork and Pipes

**Status:** opened 2026-04-21. Planning phase.
**Motivation:** hermetic sandboxing is currently an 80-line Rust
primitive that resolves a binary path, writes a tempfile, spawns the
wat binary as a subprocess, and captures pipe output. It works, but
it couples to "the path to the wat binary" as a runtime concern the
wat layer shouldn't care about. Any other subprocess use case —
auxiliary tools, shell-style pipelines, preforked workers — would
duplicate the machinery.

The honest move is raw Unix `fork(2)`. The child is a copy-on-write
duplicate of the parent — it has the wat runtime already loaded, the
stdlib already registered, and the caller's `Vec<WatAST>` already in
memory via inherited heap. No binary path. No tempfile. No exec. The
child forks from wherever the parent was, redirects stdio to pipe
fds, evaluates the caller's forms, exits.

Builder direction: *"we need real fork."*

---

## Non-goals (named explicitly)

- **Windows.** Windows has no fork. This arc is Unix-only. wat-rs's
  existing libc-based signal handling is already Unix-only; this
  doesn't change the portability floor.
- **Binary-path resolution.** No `current-exe`, no
  `WAT_HERMETIC_BINARY`, no re-invocation of the wat binary. The
  hermetic child IS the wat process, forked from the parent. Any
  future "call a different binary" use case (shelling out to sqlite3,
  ffmpeg, etc.) is a separate arc — `spawn-process` in the
  fork+exec shape — and does NOT get factored into this one.
- **General filesystem-write capability.** No `write-tempfile`.
  Hermetic doesn't need it under fork.
- **exec() alternatives.** Fork-without-exec only. Replace-this-
  process flow is its own use case.
- **Signal forwarding, job control, fd inheritance policy beyond
  pipe stdio.** Explicit basic pipe-three-stdio-streams scope.

---

## What this arc ships

Three slices, ordered so each is independently testable.

### Slice 1 — `:wat::kernel::pipe` + `PipeReader` / `PipeWriter`

The raw-fd IO primitives every subsequent fork-based primitive
depends on.

```
(:wat::kernel::pipe -> :(wat::io::IOWriter, wat::io::IOReader))
```

Calls `libc::pipe(2)` (or `nix::unistd::pipe`), wraps the write end
as `PipeWriter(OwnedFd)` and the read end as `PipeReader(OwnedFd)`.
Both satisfy the existing `WatWriter` / `WatReader` traits from arc
008; callers see the same `:wat::io::IOReader` / `:wat::io::IOWriter`
wat types they use today for stdio.

**Critical: direct syscall writes, no `std::io::stdout` coupling.**
`PipeWriter::write_all` calls `libc::write(2)` directly on the fd.
Does NOT go through `std::io::Stdout` or its internal Mutex. This
matters for fork safety — a parent thread holding the stdlib stdio
lock at fork time would leave the child holding a dead lock if the
child ever tried to write via stdlib. Our PipeWriter sidesteps the
lock entirely.

**Dual role.** `PipeReader(OwnedFd)` / `PipeWriter(OwnedFd)` are
constructible from any `OwnedFd`, not only from a fresh `pipe(2)`
pair. Slice 2 constructs them over the child-side fd 0/1/2 after
`dup2` — exactly the same type, different owning fd. The child's
`:user::main` receives `(PipeReader(fd0), PipeWriter(fd1),
PipeWriter(fd2))` as its three IO arguments. Same trait
(`WatReader`/`WatWriter`); same syscall-direct write path; no
std::io Mutex inheritance. Slice 1 must expose a construction path
`PipeReader::from_owned_fd(OwnedFd)` and `PipeWriter::from_owned_fd`
that slice 2 uses after dup2.

**Thread-safety:** pipe fds are `Send + Sync`. Unlike `StringIoReader`
/ `StringIoWriter` (ThreadOwnedCell, single-thread), pipe-backed IO
is thread-safe — the kernel serializes fd access.

**Lifetime:** fd close is automatic on `Drop` via `OwnedFd::Drop`
calling `close(2)`. No force-drop primitive at the wat level; scope
IS shutdown (per existing discipline). Child sees EOF on stdin when
the parent's stdin-writer scope exits and the fd closes.

### Slice 2 — `:wat::kernel::fork-with-forms` + `ChildHandle` + `ForkedChild` + `:wat::kernel::wait-child`

The core fork primitive. Caller passes a vec of forms to evaluate in
the child; the primitive returns a `ForkedChild` struct holding the
parent-side handles plus a child handle.

```
(:wat::core::struct :wat::kernel::ChildHandle ... opaque ...)

(:wat::core::struct :wat::kernel::ForkedChild
  (handle :wat::kernel::ChildHandle)
  (stdin  :wat::io::IOWriter)   ;; parent writes → child stdin
  (stdout :wat::io::IOReader)   ;; parent reads ← child stdout
  (stderr :wat::io::IOReader))  ;; parent reads ← child stderr

(:wat::kernel::fork-with-forms
  (forms :Vec<wat::WatAST>)
  -> :wat::kernel::ForkedChild)

(:wat::kernel::wait-child (handle :wat::kernel::ChildHandle) -> :i64)
```

**Why a struct and not a 4-tuple.** wat's `:wat::core::first` /
`second` / `third` stop at three. Extending tuples with `fourth` for
one caller would make the tuple sketch the shape rather than the
accessor. Named fields through `ForkedChild/stdin` etc. match how
`RunResult` already surfaces — three-plus-field returns are struct
territory in wat, not tuple.

**What the primitive does:**

1. Creates three pipe pairs via `libc::pipe`.
2. Calls `libc::fork()` (marked `unsafe`; the unsafety justified in
   the doc-comment per fork-safety rules below).
3. **In the child:**
   - `dup2` the pipe ends onto fd 0/1/2.
   - Close every other inherited fd (strategy resolves when slice 2
     lands — `nix::unistd::close_range` if MSRV allows, else
     `/proc/self/fd` iteration on Linux and /dev/fd on BSD-derivatives).
   - Construct `PipeReader::from_owned_fd(0)` + two
     `PipeWriter::from_owned_fd(1)` / `(2)` — same types slice 1
     ships, over the redirected stdio fds. No `RealStdin` /
     `RealStdout` / `RealStderr` (those wrap `std::io::Stdin`'s
     reentrant Mutex, which would inherit parent-thread lock state
     across fork; PipeReader/Writer bypass std::io entirely).
   - Call `startup_from_forms(forms, None, loader)` to build a
     fresh `FrozenWorld` from the forms the caller passed. The
     forms are already in the child's inherited memory — no
     re-parse needed.
   - Enforce `:user::main` signature on the new frozen world via
     `validate_user_main_signature`.
   - Call `invoke_user_main(&new_world, [pipe-stdin, pipe-stdout,
     pipe-stderr])` inside `catch_unwind`.
   - Exit with code derived from outcome. Exact code convention
     resolves when slice 2 lands — candidates: 0 on success, 1 on
     RuntimeError, 2 on panic, 3+ on startup/validation failure.
     The convention must be reversible — slice 3's parent-side
     Failure reconstruction reads the exit code.
4. **In the parent:**
   - Close the child-side of each pipe pair (stdin read end,
     stdout/stderr write ends).
   - Wrap the parent-side ends as `PipeWriter` / `PipeReader`.
   - Construct the `ForkedChild` struct value holding the
     `ChildHandle` + three pipe handles.
   - Return as `Value::Struct`.

`wait-child` blocks on `waitpid(child_pid)` and returns the exit
status as an `:i64`. `ChildHandle::Drop` reaps the child with
`waitpid(WNOHANG)` + then blocking `waitpid` if the caller dropped
without explicitly waiting — prevents zombies on early-exit paths.

---

## Fork safety discipline

`unsafe { libc::fork() }` is safe-to-use-correctly only if the child
uses async-signal-safe operations until it either exec()s or exit()s,
per POSIX. Our child does NOT exec — it runs full wat evaluation.
This requires active care:

- **No `std::io::stdout()` / `::stderr()` / `::stdin()` in the child.**
  Those hold Mutexes that could be locked at fork time by another
  parent thread. Child must never touch them. Our `RealStdin` /
  `RealStdout` / `RealStderr` wrappers already call the underlying
  fd via std's handles, not via `io::stdout()` locks — but we'll
  audit and ensure all hermetic-child write paths go through
  `PipeWriter` / raw syscalls. The child's `RealStdout` etc. wrap
  the dup2'd fds as raw `OwnedFd`, not `std::io::Stdout`.

- **No Mutex anywhere in the runtime.** We already have this
  discipline from ZERO-MUTEX.md. Confirmed: `SymbolTable`, `TypeEnv`,
  `MacroRegistry`, `EncodingCtx` all use `Arc<T>` of immutable data
  post-freeze. `StringIoReader` / `StringIoWriter` use `ThreadOwnedCell`
  which is single-thread and won't be used across the fork boundary.

- **Close inherited fds in the child.** Any fd the parent had open
  (files, sockets, child-side pipe ends the child doesn't want)
  leaks into the child and could keep resources alive. Close-on-
  exec (`O_CLOEXEC`) doesn't help here — we're not exec'ing. Iterate
  `/proc/self/fd` post-fork and close everything except 0/1/2.
  Alternative on BSD: `closefrom(2)`. `nix` exposes both.

- **ThreadOwnedCell identity in the child.** Thread IDs are unique
  per OS thread per process. The child's forking thread has a new
  (OS-assigned) thread ID distinct from the parent's. Parent-
  allocated `ThreadOwnedCell<T>` instances in the child would fail
  their owner check. **Mitigation:** the child never touches a
  ThreadOwnedCell allocated in the parent. The child allocates its
  own cells inside its fresh `startup_from_forms`-produced frozen
  world.

- **Other threads don't fork.** Only the calling thread exists in
  the child. Any wat-kernel-spawned worker threads in the parent
  are gone in the child. For hermetic, this is correct — the child
  is supposed to be a fresh evaluation, not a continuation of
  parent workloads. Any pending crossbeam-channel state on parent
  threads is irrelevant; the child's new frozen world has its own
  channels.

- **libc::malloc.** glibc's allocator uses atfork handlers and is
  fork-safe. jemalloc etc. similarly. Not a concern on our target
  platforms.

- **Signal handlers.** Inherited from parent. Wat-vm's SIGINT /
  SIGTERM handlers stay active in the child. Desirable — Ctrl-C in
  the parent's terminal propagates to the child, the child's
  `:wat::kernel::stopped?` flag flips, userland polls and cascades
  shutdown. Same discipline the parent uses.

The cost of these rules is real but bounded: two careful spots in
the child's post-fork code (no std stdio; close inherited fds). The
payoff is a clean, fast, honest subprocess primitive.

---

## Slice 3 — Hermetic reimplementation in wat stdlib

The proof. `run-sandboxed-hermetic-ast` becomes wat stdlib on top of
slices 1 + 2. The current Rust primitive
(`eval_kernel_run_sandboxed_hermetic_ast`) retires entirely — the
AST-to-source serializer (arc 011's `wat_ast_program_to_source`) is
no longer needed for this purpose and may stay unused (or be retired
as a follow-up if no other caller surfaces).

Shape (sketch; exact Failure reconstruction resolves when slice 2
lands and exit-code convention is pinned):

```scheme
(:wat::core::define (:wat::kernel::run-sandboxed-hermetic-ast
                    (forms :Vec<wat::WatAST>)
                    (stdin :Vec<String>)
                    (scope :Option<String>)
                    -> :wat::kernel::RunResult)
  (:wat::core::let*
    (((spawned  :wat::kernel::ForkedChild)
      (:wat::kernel::fork-with-forms forms))
     ((handle   :wat::kernel::ChildHandle)
      (:wat::kernel::ForkedChild/handle spawned))
     ((out-rx   :wat::io::IOReader)
      (:wat::kernel::ForkedChild/stdout spawned))
     ((err-rx   :wat::io::IOReader)
      (:wat::kernel::ForkedChild/stderr spawned))

     ;; Write stdin in an inner scope — when it exits, stdin-writer
     ;; drops, the fd closes, the child sees EOF on fd 0.
     ((_ :())
      (:wat::core::let*
        (((stdin-wr :wat::io::IOWriter)
          (:wat::kernel::ForkedChild/stdin spawned))
         ((joined   :String)
          (:wat::core::string::join "\n" stdin)))
        (:wat::io::IOWriter/write-all stdin-wr joined)))

     ((stdout-str :String) (:wat::io::IOReader/read-all out-rx))
     ((stderr-str :String) (:wat::io::IOReader/read-all err-rx))
     ((exit-code  :i64)    (:wat::kernel::wait-child handle))

     ((stdout-lines :Vec<String>)
      (:wat::core::string::split stdout-str "\n"))
     ((stderr-lines :Vec<String>)
      (:wat::core::string::split stderr-str "\n")))

    ;; Failure from exit-code: shape resolves when slice 2 pins the
    ;; exit-code convention. Sketch: 0 → :None, non-zero → (Some
    ;; (struct-new :wat::kernel::Failure ...)).
    (:wat::core::struct-new :wat::kernel::RunResult
      stdout-lines
      stderr-lines
      (... failure-reconstruction-from exit-code stderr-str ...))))
```

**What slice 3 proves:**
- The substrate primitives compose into the existing capability.
- The reimplementation is readable wat source (not Rust) so future
  authors can audit hermetic behavior without leaving the language.
- Any other subprocess use case on top of `fork-with-forms` has the
  same building blocks available — reading stdout as an
  `IOReader`, waiting the child, inspecting its exit code.

**`struct-new` is load-bearing.** Slice 3 constructs
`:wat::kernel::RunResult` at the wat level via `:wat::core::struct-new`.
That primitive already exists and takes positional field values. No
new constructor primitive needed. Equally, `:wat::kernel::Failure` is
constructible the same way when the reconstruction shape resolves.

**Scope handling.** The `scope` parameter on the current hermetic
returns a Failure when `Some`. The wat-level reimplementation keeps
that behavior: `ScopedLoader` inside a fork'd child is not currently
threaded through — the child's `startup_from_forms` receives whatever
loader the fork primitive provides (default `InMemoryLoader` for
hermetic-style use). A future caller demanding scope-inside-fork
can add the wiring; this arc punts.

---

## Convergence with prior art

Fork + pipes are as old as Unix. Every POSIX language surfaces them:

| Language | fork | pipe | wait |
|---|---|---|---|
| C (POSIX)      | `fork`                          | `pipe`                  | `waitpid`  |
| Ruby           | `Kernel#fork { ... }`           | `IO.pipe`               | `Process.wait` |
| Python         | `os.fork()` + `os.pipe()`       | `os.pipe()`             | `os.waitpid` |
| Go             | `syscall.ForkExec` (primitive)  | `os.Pipe`               | `Cmd.Wait` |
| Erlang/Elixir  | (portable process spawn)        | (via Port)              | (via Port) |
| Rust stdlib    | (no raw fork; `Command::spawn`) | `nix::unistd::pipe`     | `Child::wait` |
| Wat            | `:wat::kernel::fork-with-forms` | `:wat::kernel::pipe`    | `:wat::kernel::wait-child` |

Rust stdlib doesn't expose raw fork — `Command::spawn` is fork+exec
abstracted. Lower-level languages (C, Python, Ruby, Go at the syscall
layer) expose both separately. wat joins that line because forked-
without-exec is the capability we actually want for hermetic
isolation, and we have the infrastructure (libc dep already present)
to do it honestly.

The convergence is informational, not prescriptive: we use it to
pick honest names (fork, pipe, wait-child), not to dictate
signature shapes.

---

## Resolved design decisions

- **Unix-only.** Windows is not a target. Named non-goal.
- **Raw fork, not fork+exec.** Child inherits parent's loaded wat
  runtime via COW; no binary path needed, no tempfile, no re-load.
- **No `current-exe`, `env-var`, or `write-tempfile` primitives
  in this arc.** They were under the old plan; fork makes them
  unnecessary. A future arc can ship any of them if a caller
  demands (e.g., shelling out to sqlite3 via a general
  `spawn-process` fork+exec primitive would revisit them).
- **PipeWriter/PipeReader bypass Rust stdlib stdio.** Direct syscall
  writes; no Mutex dependency; fork-safe.
- **Child uses fresh `startup_from_forms` on inherited AST.** The
  inner program gets its own FrozenWorld. Same isolation hermetic
  currently provides; just without the binary reload.
- **Close-all-inherited-fds in the child after dup2.** Standard
  fork hygiene.
- **`ChildHandle` opaque struct.** Holds the child pid and a
  reap-state flag. `Drop` reaps if the caller didn't `wait-child`.
- **Slice independence.** Slice 1 is useful standalone (any pipe
  user). Slice 2 is the fork capability. Slice 3 is the
  reimplementation proof.

---

## Open questions to resolve as slices land

- **`wait-child` return shape.** `:i64` exit code is simplest.
  Richer `:ExitStatus` struct with signal-vs-normal-exit distinction
  is nicer but adds a type. Start `:i64`; upgrade if a caller
  demands.
- **Error surface for `fork-with-forms` failures.** If the fork
  itself fails (ENOMEM, process limits), Rust `Result<_, RuntimeError>`
  at primitive level. If the child fails during startup (parse
  error in the forms, main-signature mismatch), the child exits
  with a diagnostic code and the parent observes via `wait-child`.
  What counts as "failure of fork-with-forms itself" vs. "failure
  of child program"?  Resolve when slice 2 lands.
- **`/proc/self/fd` iteration portability.** Linux has /proc always;
  macOS exposes `/dev/fd` but behavior differs; BSD has `closefrom(2)`.
  `nix::unistd::close_range` is the cross-platform abstraction.
  Commit to `close_range` when slice 2 lands.
- **Reap on drop vs block-on-drop.** `ChildHandle::Drop` with no
  explicit `wait-child` needs a policy. Easiest: if not yet reaped,
  drop calls blocking `waitpid`. If the caller wants non-blocking,
  they must explicitly `wait-child`. Document the default.

---

## What this arc does NOT ship (deferred)

- **`spawn-process` for fork+exec use cases.** Calling sqlite3 or
  other external binaries via argv + args. Separate arc, uses
  libc::execvp, does NOT depend on arc 012's fork-with-forms
  (different shape: exec replaces process image; fork-with-forms
  keeps it).
- **Kill.** `:wat::kernel::kill-child handle signal`. Scope-based
  reaping via Drop covers common cases; explicit kill when a
  caller surfaces demand.
- **Non-blocking `try-wait-child`.** Future if needed.
- **Scope parameter honored in hermetic.** Currently returns Failure
  on `:Some`. Wiring a ScopedLoader through fork is its own slice.
- **`:wat::core::ast-to-source` at wat level.** Arc 011's serializer
  stays Rust-public; no longer needed for this arc's hermetic
  reimplementation (fork passes AST via memory, not text). Can
  expose when a general caller demands.

---

## The thread from arcs 009 / 010 / 011 continues

Three prior arcs factored ceremony into substrate: names-are-values
(arc 009), forms-are-values (arc 010), AST-entry hermetic (arc 011).
Arc 012 removes the last coupling: "hermetic requires the wat
binary's path." The new coupling is honest — hermetic requires fork,
and fork IS a substrate capability every process needs at times.

The reimplementation in wat stdlib closes the loop. Three substrate
primitives (`pipe`, `fork-with-forms`, `wait-child`) + the existing
arc 008 IO trait abstractions + the arc 010 `program` macro + the
arc 009 name-as-value convention = ~20 lines of wat that replace
~80 lines of purpose-built Rust.

That's the substrate teaching itself to listen, one more time.
