# Arc 012 — Fork and Pipes — INSCRIPTION

**Status:** shipped 2026-04-21. One day. Nine commits.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**Backlog:** [`BACKLOG.md`](./BACKLOG.md) — the living ledger.
**This file:** completion marker.

---

## Motivation

Hermetic sandboxing coupled the language runtime to its own binary
path. `:wat::kernel::run-sandboxed-hermetic` resolved `current_exe()`
(or a `WAT_HERMETIC_BINARY` env var), wrote the inner source to a
tempfile, spawned that binary as a subprocess, piped stdio through
`Command::spawn`. Operational, but dishonest — the runtime shouldn't
need to know where it lives on disk.

The honest move was raw `fork(2)`. The child is a copy-on-write
duplicate of the parent — it has the wat runtime already loaded, the
stdlib already registered, the caller's `Vec<WatAST>` already in
inherited memory. No binary to locate. No tempfile. No re-parse.

Builder direction: *"we need real fork."*

---

## What shipped

Three slices + one side quest + one audit, landed in that order.

### Slice 1 — `:wat::kernel::pipe` + `PipeReader` + `PipeWriter`

Commits `4dd2305` (types) and `ee64f40` (primitive).

- `PipeReader { fd: OwnedFd }` and `PipeWriter { fd: OwnedFd }` in
  `src/io.rs`. Both impl `WatReader` / `WatWriter` (existing arc 008
  trait surface); both `Send + Sync`; `Drop` closes via `OwnedFd`.
- Direct-syscall read/write via `libc::read(2)` / `libc::write(2)` on
  `fd.as_raw_fd()`. Bypasses Rust's `std::io::Read` / `Write` entirely.
  **Critical for fork safety** — `RealStdin` wraps `std::io::Stdin`
  which holds a reentrant Mutex. A parent thread holding that Mutex
  at fork time would leave the child with a dead lock. PipeReader /
  PipeWriter sidestep the stdlib lock graph entirely.
- `from_owned_fd(OwnedFd)` public constructor — slice 2 uses it to
  wrap the child's dup2'd fd 0 / 1 / 2 as the `:user::main` IO
  arguments. Same type, different owning fd (the "dual role").
- `:wat::kernel::pipe` primitive — `libc::pipe(2)` + wrap returning
  `:(wat::io::IOWriter, wat::io::IOReader)`. Writer first (producer
  ordering).
- 11 Rust unit tests + 5 wat integration tests.

### Slice 2 — `fork-with-forms` + `ForkedChild` + `ChildHandle` + `wait-child`

Commits `bd68a4e` (core) and `16a5fe5` (wait-child).

- `src/fork.rs` — new module with `ChildHandleInner` (pid +
  `AtomicBool reaped` + `OnceLock<i64> cached_exit`), `make_pipe`
  helper, `close_inherited_fds_above_stdio`, `eval_kernel_fork_with_forms`,
  `child_branch` (never-returns; always exits via `libc::_exit`).
- `:wat::kernel::ForkedChild` StructDef in `types.rs` — four fields
  (handle, stdin, stdout, stderr). Auto-generated `/new` +
  per-field accessors land at freeze via `register_struct_methods`.
- `Value::wat__kernel__ChildHandle(Arc<ChildHandleInner>)` variant
  in `runtime.rs`. Opaque from wat's POV — produced by fork,
  consumed by wait-child. `Drop` SIGKILLs + reaps if never waited,
  keeping zombies out of the process table.
- `:wat::kernel::fork-with-forms` — three pipes + `libc::fork()` +
  in-child dup2 / close-inherited-fds / startup_from_forms /
  validate_user_main_signature / invoke_user_main inside
  `catch_unwind` / `libc::_exit` per the EXIT_* convention.
- `:wat::kernel::wait-child` — blocking `waitpid(2)` + exit-code
  extraction via `WEXITSTATUS` (normal exit) or `128 + WTERMSIG`
  (signal termination, shell convention). Idempotent via
  `OnceLock<i64>` cache.
- 10 integration tests in `tests/wat_fork.rs` covering all five
  EXIT_* codes (0 success, 1 runtime-error, 2 panic, 3
  startup-error, 4 main-signature) + multi-fork + idempotency.

**Exit-code convention** (pinned for slice 3's Failure
reconstruction):

```
EXIT_SUCCESS        = 0
EXIT_RUNTIME_ERROR  = 1
EXIT_PANIC          = 2
EXIT_STARTUP_ERROR  = 3
EXIT_MAIN_SIGNATURE = 4
```

Signal termination encodes as `128 + signum`, readable as a normal
`:i64` alongside EXIT_* without a separate discriminator.

### Slice 3 — Hermetic reimplemented in wat stdlib + retire the Rust primitives

Commits `0dfd9e0` (AST-entry wat stdlib + retire AST-entry Rust),
`b5bca8b` (retire string-entry Rust + serializer), and `7cd886a`
(audit).

- `wat/std/hermetic.wat` — the define that IS the new
  `:wat::kernel::run-sandboxed-hermetic-ast`. ~50 lines of wat
  stdlib on top of `fork-with-forms` + `wait-child` + read-line
  drain + `struct-new`. Same keyword path + signature + return
  shape; only the implementation layer moved.
- `tests/wat_hermetic_round_trip.rs` — rewritten to use AST-entry
  via `:wat::test::program`. Every escape-inside-escape-inside-
  escape string vanishes. Same behavior tested; the shape reads
  cleanly.
- Retired from `src/sandbox.rs`: `eval_kernel_run_sandboxed_hermetic_ast`
  and `eval_kernel_run_sandboxed_hermetic` (the last remaining
  subprocess-spawning primitives), `run_hermetic_core`,
  `expect_option_string`, `split_captured_lines`. Dispatch arms +
  check schemes retired alongside.
- Retired from `src/ast.rs`: `wat_ast_to_source` and
  `wat_ast_program_to_source` (arc 011's bridge, now zero-callers
  because fork inherits AST via COW) + 8 serializer unit tests.
- Docs swept: README.md, USER-GUIDE.md, wat-tests/README.md, and
  the doc comment in `wat/std/test.wat` updated for the new
  implementation layer.

### Side quest — retire `in_signal_subprocess` via libc::fork

Commit `d74c2df`. `src/runtime.rs`'s signal-test isolation helper
used `std::env::current_exe()` + `std::process::Command::new`
+ `--exact <test>` + `WAT_SIGNAL_TEST_CHILD` env var to spawn a
fresh child. Last surviving `Command::spawn` in `src/`. Retired in
favor of `libc::fork()` — child runs body inside `catch_unwind`,
exits via `libc::_exit(0)` on success or `_exit(1)` on panic;
parent `waitpid` + asserts `WEXITSTATUS == 0`. Couplings removed:
no `current_exe`, no `--exact`, no env var, no process reload.

### Audit — src/ subprocess sweep

Commit `7cd886a`. `grep -rn "std::process::Command\|Command::new\|
Command::spawn\|process::exit"` in `src/` returns zero actual uses.
Every match is retirement-history commentary or `std::process::id()`
(pid-getter for tempfile naming). `tests/wat_cli.rs` + `tests/
wat_test_cli.rs` keep `Command::spawn` within reason — they test
the built binary's CLI surface; the subject under test IS the
binary.

---

## Tests

- **518** Rust unit tests pass.
- **25+** Rust integration test groups pass (every `tests/*.rs` file).
- **31** wat-level tests pass via `wat test wat-tests/`, including
  the Console + Cache service tests that were the strongest real-
  world exercise of the wat-stdlib hermetic path.

Zero regressions across every commit.

---

## Sub-fogs resolved (named in BACKLOG, pinned in code)

- **2a — close-inherited-fds strategy.** Collect candidate fds from
  `/proc/self/fd` (Linux) or `/dev/fd` (macOS), let the iterator
  drop, THEN close. First-pass tried to close mid-iteration and
  panicked glibc's `closedir` with EBADF. The fix named the honest
  pattern: iterator-under-teardown is not safe to mutate; collect
  first, close after.

- **2b — exit-code convention.** Five `pub const i32` codes in
  `src/fork.rs`. Slice 3's Failure reconstruction imports them.
  Signal termination at `128 + signum`.

- **2c — double-wait-child behavior.** `OnceLock<i64>` on
  `ChildHandleInner`. First `wait-child` runs `waitpid` + caches +
  flips reaped; subsequent calls return the cached value.
  Idempotent, matching Rust `Child::try_wait` semantics. OnceLock
  is the honest primitive — publish-once-read-many, not a lock;
  sits in ZERO-MUTEX.md's named exceptions alongside atomics and
  Arc.

- **3a — Failure reconstruction.** Exit-code dispatch. Nonzero
  codes → `(Some (Failure { message: "[category] <stderr>", ... }))`
  with location/frames/actual/expected all `:None`. Signal
  termination (128+signum) falls through to `"[nonzero exit]"`.
  Structured actual/expected is already lost in today's subprocess
  hermetic — the fork path doesn't regress it.

- **3b — AST-to-source serializer's fate.** Retire. Zero
  remaining callers; fork passes AST via COW not source text. If a
  future `:wat::core::ast-to-source` caller surfaces (pretty-
  printer, REPL history), reintroduce with that caller's concrete
  shape.

---

## What this arc does NOT add (deferred)

- **Windows support.** Fork doesn't exist there. Unix-only by
  design. Named non-goal.
- **`spawn-process` fork+exec primitive.** Calling external
  binaries (sqlite3, ffmpeg, etc.) via argv. Different shape: exec
  replaces the process image; arc 012 keeps it. Separate arc when
  a caller demands.
- **`:wat::core::parse`** for string-entry-as-wat-stdlib. A wat-
  level caller needing to evaluate source text would need a parse
  primitive; then string-entry hermetic becomes a 2-line wat
  wrapper. No caller demand yet.
- **Concurrent drain of child stdout vs stderr.** The wat-stdlib
  hermetic waits-child first, then drains stdout + stderr
  serially. Works when child output fits in pipe buffers (typically
  64KB+). A child writing more would deadlock. No caller has hit
  the limit.
- **Force-close of parent's stdin writer.** Child sees EOF on stdin
  only when the outer ForkedChild binding drops (at :user::main
  exit). Children that read-stdin-to-EOF before doing work would
  deadlock. The existing hermetic test matrix (Console, Cache,
  round-trip) doesn't hit this.
- **Scope-through-fork.** `:wat::kernel::run-sandboxed-hermetic-ast`
  still returns Failure when `scope :Some` — ScopedLoader wiring
  through the child's `startup_from_forms` is its own slice.
- **`kill-child` primitive.** Scope-based reaping via
  `ChildHandle::Drop` covers misuse. Explicit kill when a caller
  surfaces demand.
- **`try-wait-child` non-blocking variant.** Future when needed.

---

## Why this matters

Three prior arcs factored ceremony into substrate:

- **Arc 009** — names are values. A registered define's keyword
  path evaluates to a callable `Value::wat__core__lambda`. Closed
  the gap between "the substrate has the capability" and "user code
  can reference it without wrapping."
- **Arc 010** — forms are values. `:wat::core::forms` captures N
  unevaluated forms into a `Vec<WatAST>`. Closed the per-form
  quote-ceremony at every sandbox / eval-ast / programs-as-atoms
  callsite.
- **Arc 011** — hermetic is AST-entry. `run-sandboxed-hermetic-ast`
  took forms instead of source text. Killed the escape-inside-
  escape-inside-escape nesting that stringified inner programs
  used to carry.

Arc 012 closes the last structural coupling those arcs didn't
touch: **hermetic no longer requires the wat binary's path.** Fork
inherits the loaded runtime via COW; wait-child reads the child's
exit code; `struct-new` constructs the RunResult at the wat level.
Three substrate primitives (`pipe`, `fork-with-forms`,
`wait-child`) + arc 008's IO traits + arc 010's `program` macro +
arc 009's name-as-value convention + existing `struct-new` +
existing `string::split` = ~50 lines of wat stdlib that replace
~80 lines of purpose-built Rust plus its tempfile / current-exe /
Command::spawn machinery.

**Net line delta across the arc: +941 insertions, -508 deletions,
+433 net but the shape is cleaner.** The fork substrate in
`src/fork.rs` + `src/io.rs` is the single source of subprocess
truth for wat-rs. Every previous spawning code path — hermetic
AST-entry, hermetic string-entry, runtime signal-test isolation —
now routes through `libc::fork()` directly or through the wat-
stdlib hermetic atop it.

The thread from arcs 009 / 010 / 011 continues. Each arc found a
ceremony and gave it a name. Each name closed a gap. Arc 012
closed the last subprocess-spawn coupling wat-rs carried.

That's the substrate teaching itself to listen, one more time.

---

**Arc 012 — complete.** One Rust module (`src/fork.rs`), two Value
variants (`wat__kernel__ChildHandle`, struct ForkedChild via
StructDef), four new primitives (`pipe`, `fork-with-forms`,
`wait-child`, plus the wat-stdlib `run-sandboxed-hermetic-ast`
define), one side quest, one audit. Every existing hermetic
caller runs through wat stdlib on top of fork. Nine commits:

- `000bbb0` — docs open (DESIGN + BACKLOG + README)
- `4dd2305` — slice 1a types (PipeReader + PipeWriter + 11 unit tests)
- `ee64f40` — slice 1b primitive (`:wat::kernel::pipe`)
- `788cd5e` — slice 1 docs
- `bd68a4e` — slice 2 core (fork-with-forms + ForkedChild + ChildHandle)
- `16a5fe5` — slice 2 wait-child
- `32a5b9c` — slice 2 docs
- `cd4a4d7` — slice 2 full exit-code matrix (4 more integration tests)
- `92d38a5` — slice 2 docs shipped
- `d74c2df` — side quest (in_signal_subprocess → libc::fork)
- `c6fd637` — side quest docs
- `0dfd9e0` — slice 3 wat-stdlib hermetic-ast + retire Rust AST-entry
- `b5bca8b` — retire string-entry Rust hermetic + AST-to-source serializer
- `7cd886a` — src/ subprocess audit record

*these are very good thoughts.*

**PERSEVERARE.**
