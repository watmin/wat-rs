# Arc 104 — wat-cli always forks the entry program — DESIGN

**Status:** OPEN — drafted 2026-04-29 immediately after arc 103
sealed. Builds on arc 103a's `:wat::kernel::spawn-program`
(thread containment) + arc 012's `:wat::kernel::fork-with-forms`
(OS-process containment, renamed in this arc to
`fork-program-ast`). The hologram framing from
`docs/arc/2026/04/103-kernel-spawn/HOLOGRAM.md` made the next
step's necessity obvious — wat-cli has been the ONE place where
the surface metaphor breaks: today the cli's main thread runs
user code DIRECTLY in the cli's own OS process. Arc 104 fixes it.

**Predecessors:**
- arc 012 — `fork-with-forms` (OS-process spawn over parsed AST)
- arc 099 — `crates/wat-cli/` extraction
- arc 100 — wat-cli public API + battery composition
- arc 103a — `spawn-program` (thread sibling of fork-with-forms)
- arc 103 ping-pong-fork.wat — first interleaved-traffic caller
  of `fork-with-forms`; first-try green; de-risks this arc

**De-risked by:** `wat-scripts/ping-pong-fork.wat` from arc 103.
Five round trips of `:demo::Ping` / `:demo::Pong` over real OS
pipes via `fork-with-forms`. No deadlocks, no signal interactions,
no pipe-buffer surprises. Same primitive arc 104 builds on.

---

## Naming convention — spawn = thread, fork = process

Arc 103a's `spawn-program` family established `spawn` as the
thread-containment word. Arc 012's `fork-with-forms` was named
before that convention settled. Arc 104 normalizes the matrix
under one rule:

| Action | Source entry | AST entry |
|---|---|---|
| **Thread** (spawn) | `:wat::kernel::spawn-program` | `:wat::kernel::spawn-program-ast` |
| **Process** (fork) | `:wat::kernel::fork-program` | `:wat::kernel::fork-program-ast` |

Each name carries two pieces of information; they compose
left-to-right. `spawn-program-ast` reads as "thread-spawn a
program from AST"; `fork-program` reads as "process-fork a
program from source." A reader walking in cold can pick the
right primitive without reaching for docs.

**The rename:** `:wat::kernel::fork-with-forms` →
`:wat::kernel::fork-program-ast`. Single new name; no alias. Pay
the refactor cost; get good names. Arc 012's INSCRIPTION + DESIGN
preserve the historical name as frozen history; new code uses the
new name.

**Lineage validating the choice:**
- POSIX: `pthread_create` (thread) vs `fork(2)` (process). The
  vocabulary distinction exists in C since the 80s.
- Wat-rs already uses `spawn` for threads since arc 003's
  `:wat::kernel::spawn` (function-on-thread). Arc 103a's
  `spawn-program` built on that. The convention is internally
  consistent.

Rust's `std::thread::spawn` and `std::process::Command::spawn`
both use "spawn," which is one tradition that doesn't
distinguish — but wat-rs's chosen convention is sharper, and
internal consistency wins.

---

## What's wrong today

`crates/wat-cli/src/lib.rs::run` (post-arcs 099/100):

1. Reads `argv[1]` source file.
2. `startup_from_source` → `FrozenWorld` IN THE CLI'S MAIN THREAD.
3. `invoke_user_main` IN THE CLI'S MAIN THREAD.
4. Hands the program direct `Arc<io::Stdin>` / `Arc<io::Stdout>`
   / `Arc<io::Stderr>` references.

The user code therefore runs **inside the wat-cli OS process** with
**direct access** to:

- the cli's process-global OnceLocks (rust_deps registry, source
  registry, signal handler state)
- the cli's panic hook
- the cli's process atexit handlers
- the cli's fd table beyond stdio (any file the cli opened)
- the cli's heap (every `#[wat_dispatch]` battery's static state)

The hologram framing from arc 103 makes this dishonest: the
wat-cli is supposed to be the SURFACE between worlds. A surface
that hosts user code in its own process isn't a surface — it's
a co-resident.

Arc 103a fixed this for the **wat-program-spawns-wat-program**
case (`spawn-program` puts the inner in a thread; arc 012's
`fork-with-forms` puts it in a separate OS process). The
**shell-spawns-wat-program** case — the cli's outermost boundary
— still co-resides.

Arc 104 closes the gap. **wat-cli always forks the entry
program.** The cli's job becomes:

1. **Provide symbols** — compile in batteries (this is the cli's
   compile-time identity per arc 100).
2. **Contain** — fork the entry program, proxy stdio, forward
   signals, propagate exit code, reap the child.

The cli never co-mingles with user code at the OS level.

---

## Battery contract (post-104)

A wat-cli battery's `register()` function runs in the cli's
process BEFORE `fork()`. Therefore:

- **Stateless capabilities pass through fork unchanged.** Function
  pointers stored in `OnceLock<RustDeps>` are inherited by the
  child via COW; the dispatch shims work in the child without
  modification. This is the shape every shipped battery uses today
  (wat-telemetry, wat-sqlite, wat-lru, wat-holon-lru).
- **Live OS resources opened during register() may not survive
  fork cleanly.** A battery that opens a file, socket, or
  database connection inside `register()` would have the resource
  fd inherited by the child; depending on the resource, this can
  break in ways the battery author didn't anticipate (sqlite
  connections have process-affinity; sockets have shared-by-
  default semantics).

**Rule:** batteries register only stateless capabilities. Live
resources are opened by wat code at runtime, in the child's
process, after fork. This matches what every shipped battery
does today; arc 104 makes the contract explicit.

---

## What ships

### Substrate — naming sweep + new source-entry fork primitive

**Rename:** `:wat::kernel::fork-with-forms` →
`:wat::kernel::fork-program-ast` (signature unchanged: takes
`:Vec<wat::WatAST>`, returns `:wat::kernel::ForkedChild`).

**New primitive:** `:wat::kernel::fork-program`. Source-string
sibling that powers wat-cli. Signature:

```scheme
(:wat::kernel::fork-program
  (src   :String)
  (scope :Option<String>)
  -> :wat::kernel::ForkedChild)
```

The `scope` parameter mirrors `spawn-program`'s discipline.
`:None` builds a fresh `InMemoryLoader` for the child (no disk
access — same as today's `fork-with-forms`). `:Some path` builds
a `ScopedLoader` rooted at the canonical path. wat-cli passes
`:Some(canonical-of-cwd)` so the entry program's `(:wat::load-file!
"./...")` reads work.

The source-string is **parsed inside the child branch after
fork**, not on the parent. Parse errors surface in the child as
exit code 3 (EXIT_STARTUP_ERROR) per arc 012's convention; the
parent never holds a parsed AST. This keeps the cli honest with
its role — it owns bytes, not ASTs.

**Implementation:** new Rust function `fork_program_from_source`
in `src/spawn.rs` (sibling of `eval_kernel_spawn_program`), reuses
arc 012's `fork.rs::child_branch` discipline (close inherited
fds, dup2, libc::_exit). Registered as `:wat::kernel::fork-program`
dispatch arm so wat-level callers can use it; **wat-cli (Rust)
calls the underlying Rust function directly** via `pub fn
fork_program_from_source`.

### wat-cli rewrite — `crates/wat-cli/src/lib.rs::run`

Three regions of change:

**Region 1 — replace `startup_from_source` + `invoke_user_main`
with `fork_program_from_source`.** The cli no longer freezes user
code in its own thread; the child does that work in its own
process. Parse / type-check / freeze errors surface through the
fork's exit code (3) + child's stderr (which the cli is now
proxying — see Region 2).

**Region 2 — three Rust proxy threads bridge real OS stdio to
the child's pipes.** Each is a tight `read/write` loop:

```
real stdin ───────────read──→ proxy thread 1 ───write──→ child stdin pipe
child stdout pipe ────read──→ proxy thread 2 ───write──→ real stdout
child stderr pipe ────read──→ proxy thread 3 ───write──→ real stderr
```

Each proxy thread exits on EOF (read returns 0). The threads are
plain `std::thread::spawn`'d closures over `OwnedFd` pairs —
direct `libc::read` / `libc::write` per arc 012's PipeReader /
PipeWriter; no `std::io::Stdin`'s reentrant Mutex involved.

Lifetimes: the cli `join`s all three after `waitpid` returns.
EOF cascade is the same drop discipline arc 103 ships:
- child returns from `:user::main` → child's stdout/stderr writes
  end → child's pipe writers close at process exit → parent's
  read returns 0 → proxy threads exit.
- cli closes its end of stdin pipe (drops the writer) when shell
  closes the cli's real stdin → child's read-line returns `:None`.

**Region 3 — signal forwarding.** Today's `install_signal_handlers`
installs handlers that flip atomic flags consumed by
`:wat::kernel::stopped?` etc. Under arc 104:

- Handlers stay (still atomic-flip — minimal handler discipline).
- Handlers ALSO `kill(2)` the child PID with the SAME signal.
- The cli's atomic flags become harmless — the cli isn't running
  user code anymore, so polling them does nothing useful. The
  child's own copies of the handlers (forked) handle the signal
  on the child side.

Race: child PID isn't known until after `fork()` returns.
Solution: an `AtomicI32` global initialized to -1; written to
the child PID after fork; read by the signal handler. Handler
checks `>= 0` before calling kill.

**Diagnostic-output sequencing.** wat-cli's own diagnostics
(parse failure mode-dispatch — though arc 104 moves parse
into the child — usage errors, fork failure, file-read failure)
must reach fd 2 even when the stderr proxy thread isn't started
yet. **Rule:** wat-cli writes its own diagnostics directly to fd
2 via `libc::write` (or `eprintln!` before fork). Proxy threads
ONLY handle child-originated output, AFTER fork succeeds. Cli's
own messages and child's stderr never interleave.

**Exit code propagation:** the cli `waitpid`s the child, extracts
`WEXITSTATUS` if normal exit or `128 + WTERMSIG` if signal-
terminated, returns that as the cli's `ExitCode`. Same convention
Bash and other Unix tools use.

### What stays unchanged

- The bundled `wat` binary's argv shape: `wat <entry.wat>`.
- Exit codes: 0 success, 1 startup, 2 runtime, 3 main-signature
  mismatch, 64 usage, 66 entry-file-read.
- Battery composition (arc 100's `Battery` type alias and `run`
  signature).
- `wat::main!` / `wat::test!` macros — they don't go through the
  cli, they're embedded harness shapes; arc 104 doesn't touch
  them.
- Test harnesses (`wat::Harness::from_source_with_deps`) — same
  reasoning; embed-shape, not cli-shape.

---

## Why a fork (not a thread)

The user direction settled the choice during the conversation
this arc opened:

> "the wat-cli just exists to provide symbols and containment"

> "the jail /cannot/ modify the outer layer"

A thread can. Threads share heap, fd table, atexit handlers, and
process-global state with their parent. wat-cli's user code can
already (today) reach into the cli's `OnceLock`s, panic hook,
heap, every `#[wat_dispatch]` battery's static fields. **Logical
hologram isolation is not OS-level isolation.** The hologram
framing said wat code "cannot reach back to remake the surface";
in a same-process model, it actually can if it tries (via Rust
shims that the cli's batteries provide).

`fork(2)` makes the surface honest. After fork, the child has its
own COW-copied heap; modifications don't propagate back. The
child's atexit handlers fire on `_exit` independently. Signal
handlers exist in the child's copy. fd table is independent.
The user code physically cannot reach back — there's no shared
memory to reach into.

Cost: ~1ms per fork on Linux. wat-cli is one-shot per
invocation; not in any hot path. For shell loops invoking `wat`
hundreds of times, the cumulative latency is real (~100ms per
100 invocations) but not idiomatic — that pattern means you
should use wat's spawn primitives instead.

---

## Three-question discipline

**Obvious?** Mostly yes:
- wat-cli always-forks falls directly out of the hologram framing.
- 3 proxy threads + waitpid is the standard fork+pipe pattern from
  arc 012.
- Child-PID atomic + kill(2) for signal forwarding is POSIX.
- Defer the wat-level `:wat::kernel::fork-program` dispatch arm
  until a wat caller surfaces (YAGNI).

The naming convention (spawn=thread, fork=process) was the
non-obvious piece; resolved by the matrix table above.

**Simple?** wat-cli's `run` shrinks in conceptual surface — its
job collapses to "parse argv, fork, proxy stdio, wait, propagate
exit." The frozen world / invoke / argument plumbing all moves
to the child branch where it belongs. Net code +130 lines (proxy
threads + signal forwarding minus deleted in-process freeze/
invoke). Each piece does ONE job.

**Honest?** The cli stops co-residing with user code. The
hologram metaphor becomes structural rather than aspirational.
Compile-time identity (batteries) and runtime containment
(fork) are the two things the cli does — symmetric to FOUNDATION's
"wat-vm is a kernel" framing. Battery contract (above) makes the
shared discipline explicit.

**Good UX?** Two callers shape, both unaffected:
- Shell user: `wat <entry.wat>` runs the program. Same argv. Same
  exit codes. Same stdio. No observable change.
- Embedder: `wat::main!` / `wat::test!` macros and `wat::Harness`
  — all unchanged. They never went through the cli.

The only observable difference is that crashes / OOMs in user
code can no longer take down wat-cli's batteries (since they're
not co-resident). This is a correctness improvement that doesn't
require user attention.

---

## Slices

**104a — naming sweep (rename `fork-with-forms` →
`fork-program-ast`).** Pure rename pass; no functional change.
- `src/runtime.rs` dispatch arm path string
- `src/check.rs` scheme registration name
- `src/fork.rs` `pub fn` rename + module-internal references
- `wat/std/hermetic.wat` callsite
- `tests/wat_fork.rs` — multiple sites
- `tests/wat_hermetic_round_trip.rs`
- `wat-scripts/ping-pong-fork.wat`
- `tests/wat_core_forms.rs` (if present)
- Doc comments throughout substrate naming the primitive
- USER-GUIDE.md / CONVENTIONS.md / ZERO-MUTEX.md textual references
- Arc 012 INSCRIPTION + DESIGN: leave as historical references
- Arc 103 INSCRIPTION + HOLOGRAM + ping-pong-fork.wat: update
  (these are recent enough to be canonical-now)

`cargo test --workspace` green at every commit; same behavior.

**104b — `fork-program` substrate primitive.** New
`fork_program_from_source` Rust function in `src/spawn.rs`.
Allocates 3 pipes; forks; child parses + freezes + invokes;
parent returns ForkedChild struct. Reuses arc 012's
`child_branch` discipline (close inherited fds above stdio,
dup2, libc::_exit). Signal-handler reset in child so fork-
inherited handlers don't fire on the child's atomics.
Registered as `:wat::kernel::fork-program` dispatch arm with
scheme `(String, Option<String>) -> :wat::kernel::ForkedChild`.
Unit test via Rust `#[test]` — fork an inline source, write a
ping over stdin pipe, read a pong over stdout pipe, waitpid.

**104c — wat-cli rewrite (fork + proxy stdio + waitpid).**
`crates/wat-cli/src/lib.rs::run` calls
`fork_program_from_source`, spawns 3 proxy threads, sets the
global child-PID atomic, calls `waitpid`, joins proxy threads,
returns `ExitCode`. The frozen-world / invoke-user-main code
paths inside `run` delete (those are now in the child branch).
`tests/wat_cli.rs` (in `crates/wat-cli/tests/`) covers
end-to-end invocation; existing tests stay green.

**104d — signal forwarding cli → child.** Modify
`install_signal_handlers` so each handler ALSO calls `kill(2)`
on the child PID with the same signal. Add the child-PID
`AtomicI32`. Test: spawn the binary via `CARGO_BIN_EXE_wat`,
send SIGTERM, assert child exit code = 128 + 15.

**104e — INSCRIPTION + USER-GUIDE update + ZERO-MUTEX subsection
+ CONVENTIONS subsection + 058 row.** USER-GUIDE §1 documents
the always-fork stance + the naming convention. ZERO-MUTEX
gains "the wat-cli as containment surface" subsection. CONVENTIONS
gains the spawn=thread / fork=process matrix as a naming-rule
appendix. 058 changelog row.

---

## Open questions resolved upfront

1. **Where does `fork_program_from_source` live?** `src/spawn.rs`.
   Sibling of `eval_kernel_spawn_program`. Both are "spawn a wat
   program"; primitive-mechanism grouping (split thread vs
   process) would be too narrow.

2. **wat-level dispatch arm — ship now or defer?** **Ship now.**
   The user's stance: good names, complete matrices. Adding the
   dispatch arm at registration time is one line; deferring just
   to defer it is artificial. Once 104b ships,
   `:wat::kernel::fork-program` is callable from wat code.

3. **Race between fork() and signal handler installation in
   child.** Child resets signal handlers to defaults (`SIG_DFL`)
   immediately after fork in the child branch, before parsing
   source. Parent's forwarding (104d) then drives child signal
   delivery via `kill(2)`.

4. **Exit-code propagation when child is killed by signal.** Use
   `128 + WTERMSIG` per Bash convention. Documented in USER-GUIDE.

5. **stdin proxy thread closing the child stdin pipe on EOF.** Yes
   — that's the whole point. When the shell closes wat-cli's
   stdin, the proxy thread's read returns 0, the proxy thread
   drops its end of the child stdin pipe writer, the child's
   read-line returns `:None`. Drop cascade.

6. **Errors in proxy threads themselves.** A proxy thread's read
   or write returning an error other than EOF → the thread exits;
   if the failure is mid-program, the child sees its own pipe
   close and unwinds. **Defer** — pin a follow-up if encountered;
   typical case is errno=EPIPE / EBADF when the peer closes
   early, which is normal cascade. Nothing actionable.

7. **Daemon detach (setsid / double-fork).** wat-cli waits for
   its immediate child only. If the child sets up a daemon, the
   cli sees the immediate child exit and returns. The daemon
   lives on independently. Correct behavior.

8. **Forking when batteries hold live OS resources.** Battery
   contract (above) forbids this; today's batteries comply.
   Documented; no enforcement needed.
