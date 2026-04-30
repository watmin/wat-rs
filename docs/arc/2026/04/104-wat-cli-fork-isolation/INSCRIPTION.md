# Arc 104 — wat-cli always forks the entry program — INSCRIPTION

**Status:** shipped 2026-04-29 (all five slices same day).

**Predecessor:** [arc 103](../103-kernel-spawn/INSCRIPTION.md). The
hologram framing from arc 103's HOLOGRAM.md made arc 104's necessity
obvious — wat-cli was the ONE place where the surface metaphor
broke. Today (post-104) the cli's job collapses to two lines:
provide symbols, contain.

**De-risked by:** `wat-scripts/ping-pong-fork.wat` from arc 103.
Five round trips of `:demo::Ping` / `:demo::Pong` over real OS
pipes via `fork-program-ast`. First-try green. The same primitive
arc 104b builds on as `fork-program`.

**Surfaced by:** mid-arc-103 conversation:

> "the wat-cli just exists to provide symbols and containment"

> "the jail /cannot/ modify the outer layer"

The cli's old shape (running user code in its own main thread,
sharing process state) was hologram-aspirational, not hologram-
structural. Wat code could reach into the cli's `OnceLock`s, panic
hook, atexit handlers, fd table, every battery's static state.
Logical isolation, not OS isolation. Arc 104 fixes it geometrically.

---

## What shipped

### Slice 104a — naming sweep

**`fork-with-forms` → `fork-program-ast`**, applied uniformly across
the substrate. The naming convention from DESIGN.md now holds the
matrix:

| Action | Source entry | AST entry |
|---|---|---|
| **Thread** (spawn) | `:wat::kernel::spawn-program` | `:wat::kernel::spawn-program-ast` |
| **Process** (fork) | `:wat::kernel::fork-program` | `:wat::kernel::fork-program-ast` |

The convention validated by Unix tradition (`pthread_create` for
thread vs `fork(2)` for process) and by wat-rs's own internal
consistency (`spawn` has meant thread since arc 003's
`:wat::kernel::spawn`).

User direction settled the choice ("we /must/ have good names —
our names must be remarkably good. we eat what refactor cost it
has"). 30 callsites updated across substrate, tests, demos, and
recent docs. Frozen historical references (`docs/arc/2026/04/{012,
015,027,031,100}/`) preserve the old name as period-correct
history.

### Slice 104b — `:wat::kernel::fork-program` substrate primitive

Source-string sibling of `fork-program-ast`. Source is parsed inside
the child branch (post-fork); the parent never holds parsed AST.
Parse errors surface as exit code 3 + stderr text.

```scheme
(:wat::kernel::fork-program
  (src   :String)
  (scope :Option<String>)
  -> :wat::kernel::ForkedChild)
```

Two entry points:
- `fork_program_from_source` — Rust function. wat-cli calls this
  directly. Takes `Arc<dyn SourceLoader>` so the cli can pass
  `Arc<FsLoader>` (cwd-relative file reads, no scope restriction).
- `eval_kernel_fork_program` — wat-level dispatch arm. Builds
  ScopedLoader / InMemoryLoader from the wat-side `:Option<String>`
  scope arg, calls through.

**Signal-handler reset post-fork**: child sets SIGINT, SIGTERM,
SIGUSR1, SIGUSR2, SIGHUP back to `SIG_DFL` immediately after fork
in the child branch, before parsing source. Per DESIGN's open
question 3: fork-inherited parent handlers reference the parent's
atomics (now in the child's COW-copy); resetting forces the child
to use kernel-default action; arc 104d's parent forwarding then
drives child signal delivery via `kill(2)`.

4 tests in `tests/wat_arc104_fork_program.rs`:
- `fork_program_child_writes_stdout_parent_reads_line`
- `fork_program_round_trip_via_pipes` — mini-TCP shape over fork
- `fork_program_clean_exit_code_via_wait_child` — drop-cascade + exit 0
- `fork_program_parse_error_surfaces_as_exit_3` — bad source → 3 or 4

### Slice 104c — wat-cli rewrite

`crates/wat-cli/src/lib.rs::run` collapses to:

1. Parse argv → if not exactly `wat <entry.wat>`, exit 64.
2. Read entry source bytes → if read fails, exit 66.
3. Install signal handlers (forwarding lands in 104d).
4. `fork_program_from_source(source, canonical, FsLoader, None)`.
5. Spawn 3 proxy threads (real stdin → child stdin pipe; child
   stdout pipe → real stdout; child stderr pipe → real stderr).
6. `waitpid` the child; extract `WEXITSTATUS` or `128+WTERMSIG`.
7. Mark reaped (skip `ChildHandleInner::Drop`'s waitpid); join
   proxy threads.
8. Return ExitCode.

The frozen-world / invoke-user-main code paths inside `run` deleted
— that work now happens in the child branch.

**Proxy thread design**: each is a tight `libc::read` / `libc::write`
loop over `OwnedFd` pairs — direct syscalls, no `std::io::Stdin`'s
reentrant Mutex. Same discipline as arc 012's PipeReader/PipeWriter.
Each exits on EOF (read returns 0).

**Diagnostic-output sequencing**: cli writes its own diagnostics
(usage error, file-read failure, fork failure) directly via
`eprintln!` (real fd 2) BEFORE proxy threads start. Once fork
succeeds, the stderr proxy handles all child-originated output.
Cli's own messages and child's stderr never interleave.

**Exit code semantics align with `EXIT_*`** (was different
pre-104):

| Code | Pre-104 (cli in-thread) | Post-104 (forked child) |
|---|---|---|
| 0 | success | success |
| 1 | startup error | runtime error |
| 2 | runtime error | panic |
| 3 | main signature mismatch | startup error |
| 4 | (n/a) | main signature mismatch |
| 64 | usage error (cli) | usage error (cli) |
| 66 | entry-file read failed (cli) | entry-file read failed (cli) |

The OLD codes conflated "startup" (parse + type-check + signature)
under code 1 + 3; the NEW codes split honestly. Three cli
integration tests updated to match (renamed
`startup_error_bubbles_up_as_exit_1` → `..._3`; signature-mismatch
tests bumped 3 → 4).

### Slice 104d — signal forwarding cli → child

Static `AtomicI32 CHILD_PID` initialized to -1; written to the
child PID after fork. Each signal handler:

1. Flips the cli's local atomic flag (kernel_stop, sigusr1, etc.)
   — pre-arc-104 behavior preserved for embedders that use
   `wat::Harness::*` without going through fork.
2. Calls `forward_signal(sig)` which atomic-loads CHILD_PID and
   `kill(pid, sig)` if pid >= 0.

Child's own handlers (reset to SIG_DFL post-fork) observe kernel
defaults: SIGINT/SIGTERM/SIGHUP terminate; SIGUSR1/SIGUSR2 either
terminate or are ignored unless the child installs its own handler.
Async-signal-safe: atomic load + `libc::kill` are both legal in
handler context.

Race tolerance: if a signal arrives before fork, CHILD_PID is still
-1 and forward_signal no-ops; cli inherits the signal's default
action.

After waitpid, CHILD_PID stores back to -1 to prevent late signals
racing with PID reuse by the OS.

Test (`sigterm_to_cli_forwards_to_child` in `crates/wat-cli/tests/
wat_cli.rs`): spawn wat-cli running a tail-recursive read-loop
that never EOFs. Send SIGTERM to the cli PID. Without forwarding
the test would hang forever (proxy threads block on read; wait_child
blocks on waitpid). With forwarding, kill(2) reaches the child,
child terminates with default SIGTERM action, `wait_child` sees
`WIFSIGNALED` + `WTERMSIG=15`, returns 128+15=143, cli exits 143.

### Slice 104e — INSCRIPTION + docs

This file. Plus updates to:

- `docs/USER-GUIDE.md` §1 — always-fork stance + naming convention
  + exit-code table
- `docs/ZERO-MUTEX.md` — new subsection "the wat-cli as containment
  surface"
- `docs/CONVENTIONS.md` — naming-rule appendix (spawn=thread,
  fork=process)
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/
  FOUNDATION-CHANGELOG.md` — arc 104 row

---

## The hologram, geometrically enforced

After arc 104 ships, every layer of the wat substrate carries its
own honest containment:

| Layer | Containment | Mechanism |
|---|---|---|
| Lexical scope inside one program | wat bindings | `let*` / function args |
| Function on thread (spawn) | wat program → its own thread | `:wat::kernel::spawn` (arc 003) |
| Whole program on thread (spawn-program) | own frozen world, shared address space | arc 103a |
| Whole program on process (fork-program) | own OS process, COW-isolated heap | arcs 012 + 104b |
| **Cli boundary** | **always-fork the entry program** | **arc 104c** |

The hologram metaphor from arc 103's HOLOGRAM.md is now structural
across every level — wat code physically cannot reach back into
the surface that hosts it, regardless of which transport the
caller picks.

The bytes-on-the-wire protocol stays uniform: line-delimited EDN
across every transport (shell pipe, in-process kernel pipe, OS
pipe between processes, future TCP between machines). Same wire,
different bytes; same hologram, different transport.

---

## Battery contract — made explicit

Stated in DESIGN.md, restated here for posterity:

A wat-cli battery's `register()` function runs in the cli's process
BEFORE `fork()`. Therefore:

- **Stateless capabilities pass through fork unchanged.** Function
  pointers stored in `OnceLock<RustDeps>` are inherited by the
  child via COW; the dispatch shims work in the child without
  modification. Every shipped battery uses this shape (wat-
  telemetry, wat-sqlite, wat-lru, wat-holon-lru).
- **Live OS resources opened during register() may not survive fork
  cleanly.** A battery that opens a file, socket, or database
  connection inside `register()` would have the resource fd
  inherited by the child; depending on the resource, this can break
  in ways the battery author didn't anticipate (sqlite connections
  have process-affinity; sockets have shared-by-default semantics).

**Rule:** batteries register only stateless capabilities. Live
resources are opened by wat code at runtime, in the child's
process, after fork. Today's batteries comply.

---

## Lessons captured

1. **Naming matters more than refactor cost.** When the user said
   "we /must/ have good names," the rename of `fork-with-forms` to
   `fork-program-ast` paid for itself in the matrix table —
   spawn/fork × source/ast composes left-to-right; a reader walking
   in cold picks the right primitive without docs.

2. **De-risking arc N+1 with arc N's last demo.** ping-pong-fork.wat
   shipped at the end of arc 103 specifically because it was the
   first interleaved-traffic caller of fork-program-ast. Arc 104
   built on its proof — first-try green at every slice.

3. **The hologram framing pre-pays clarity.** Once "wat-cli is the
   surface, not a co-resident" lands as a metaphor, the
   "always-fork" decision is structurally obvious. Without the
   framing, this arc would have been a plumbing task with debatable
   tradeoffs; with it, the work was inevitable.

4. **Exit codes carry semantic content.** Arc 104 surfaced that
   the OLD cli's codes (1=startup, 2=runtime, 3=sig) conflated
   distinct failures. The NEW codes (3=startup, 1=runtime,
   2=panic, 4=sig) split honestly because the child writes them;
   the cli stops translating.

5. **Signal forwarding is one atomic + one kill.** No threading,
   no shared mutex, no signalfd dance. Pre-fork the parent registers
   handlers; post-fork the child resets to SIG_DFL; in between, an
   AtomicI32 carries the child PID to the handler; the handler does
   one atomic-load and one libc::kill. The simplest thing that
   could possibly work IS the right thing.

---

## What's next

Arc 105 (deferred from arc 103b) — surface `spawn-program`'s
startup errors as `:Result<:Process, :StartupError>` data instead
of raising. Once that lands, `wat/std/sandbox.wat` (the wat-level
`run-sandboxed` reimplementation that's been sitting unbundled
since arc 103b) replaces the substrate Rust impls in
`src/sandbox.rs`. `Vec<String>` exits the kernel boundary
permanently; the test-convenience layer is the only place that
still collects output to a vector.

Arc 104 doesn't depend on 105; 105 doesn't depend on 104. They're
orthogonal; ship as the work surfaces.
