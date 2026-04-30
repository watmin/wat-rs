# Arc 103 — `:wat::kernel::spawn-program` — INSCRIPTION

**Status:** shipped 2026-04-29 (slices 103a + 103c + HOLOGRAM.md).
Slice 103b ships PARTIAL — substrate `Vec<String>` survives one
more round at the test-convenience boundary; the full deletion
waits on a follow-up arc that surfaces spawn-program's startup
errors as `:Result<Process, StartupError>` data.

**Predecessor:** [arc 012](../012-fork-and-pipes/INSCRIPTION.md) —
`fork-with-forms`, the heavyweight OS-process sibling. Arc 103 is
its in-thread counterpart.

**Surfaced by:** the arc-093 follow-up dispatcher conversation
2026-04-29. The user wanted a wat program to spawn another wat
program over real kernel pipes with the SAME pressure-paced
discipline `wat-rs/docs/ZERO-MUTEX.md` documents for in-process
crossbeam channels. The framing landed mid-conversation:

> "i never want to see Vec<String> ever again outside of tests —
> for real work we use real kernel pipes as the surface area of
> our programs"

> "the protocol is like mini-tcp again... producer does a
> writeln! to the consumer's stdin pipe... then blocks on readln!
> from either the consumer's stdout,stderr pipes... programs are
> delegating control of execution between each other on the
> deliverance of edn + newline"

The realization that fell out is captured in
[HOLOGRAM.md](./HOLOGRAM.md): the wat binary is a one-way
projection surface between the Rust universe (compile-time
batteries, shims) and the wat universe (frozen program, jailed
evaluation). Wat code sees through but cannot reach back.
Holograms nest cleanly via spawn-program; the EDN+newline protocol
is the only channel that crosses surfaces.

---

## What shipped

### Slice 103a — substrate primitive

**`src/spawn.rs`** (new). Three new symbols:

- `:wat::kernel::spawn-program (src :String, scope :Option<String>)
  -> :wat::kernel::Process` — source-string entry.
- `:wat::kernel::spawn-program-ast (forms :Vec<wat::WatAST>, scope
  :Option<String>) -> :wat::kernel::Process` — AST entry sibling.
- `:wat::kernel::Process` — struct: `{ stdin: IOWriter, stdout:
  IOReader, stderr: IOReader, join: ProgramHandle<()> }`.

Implementation: allocate three `pipe(2)` pairs (via the
`make_pipe` helper promoted to `pub(crate)` from arc 012's
`fork.rs`); freeze the inner world on the calling thread (so
freeze errors surface immediately as `RuntimeError`); spawn a
`std::thread` running `invoke_user_main` with the child-side pipe
ends as `:user::main` args; return the parent-side ends as a
`Process` struct value. The thread sends its `SpawnOutcome` on a
one-shot crossbeam channel — same shape `:wat::kernel::spawn`
uses, so the existing `:wat::kernel::join` /
`:wat::kernel::join-result` (arc 060) primitives work without
modification on `Process.join`.

Drop-cascade is identical to in-process channel discipline,
transported. Parent drops `proc.stdin` → underlying OwnedFd drops
→ `close(2)` runs → child's `IOReader/read-line` returns `:None`.
Child returns from `:user::main` → its writer Arcs drop → pipe
write-ends close → parent's `read-line` on `proc.stdout` /
`proc.stderr` returns `:None`. Either side panics → its handles
drop → the other side sees EOF.

**No `spawn-program-hermetic-ast`.** The user spotted the
double-fork-without-value during the conversation. Today's
"hermetic" distinction means **separate-OS-process isolation** —
`wat/std/hermetic.wat` is already a wat-level wrapper over
`fork-with-forms` (real fork, fresh address space, fresh frozen
world). For an in-thread spawn, "hermetic" reduces to "inner
declares its own Config preamble" — a wat-level discipline, not a
substrate primitive. Two substrate primitives plus the existing
`fork-with-forms` cover the matrix; nothing called "hermetic"
needs to live at the Rust layer.

**6 tests** in `tests/wat_arc103_spawn_program.rs`:

- `spawn_program_ast_child_writes_stdout_parent_reads_line` —
  basic stdout flow.
- `spawn_program_ast_round_trip_via_pipes` — the mini-TCP shape:
  parent writes a request, child reads it, writes a response,
  parent reads the response.
- `spawn_program_ast_stdout_eof_after_child_returns` — drop-cascade
  on child exit (second read on stdout returns `:None`).
- `spawn_program_ast_stderr_is_separate_pipe` — stderr isolated
  from stdout.
- `spawn_program_ast_join_returns_unit_on_clean_exit` — joining
  the Process.join handle yields `:()`.
- `spawn_program_source_string_entry` — source-string entry path.

### Pipeline-proof bonus (committed alongside 103a)

The `cat events.edn | wat router.wat | wat aggregator.wat | wat
sink.wat` shape proven end-to-end via 4 hand-written wat
programs:

- `wat-scripts/events.edn` — fixture (5 `#demo/Event` lines).
- `wat-scripts/router.wat` — drops events with `n <= 0`, forwards
  positives as `#demo/Hit`.
- `wat-scripts/aggregator.wat` — running sum of hits, emits
  `#demo/Partial` after each.
- `wat-scripts/sink.wat` — emits the last partial as `#demo/Total`
  on EOF.

Output: `#demo/Total {:total 6}` from input `[1, -100, 2, -200, 3]`.
Four independent wat-vm processes, three OS pipes between them,
back-pressure flowing through every boundary. Demonstrates the
EDN+newline protocol composes at any number of stages.

### Slice 103b — partial: IOWriter/close shipped, sandbox.wat scaffolded

**`:wat::io::IOWriter/close`** — explicit-close primitive
needed to enable wat-level run-sandboxed atop spawn-program. The
inner program reading stdin to EOF requires the parent's writer
to release its fd; without explicit close, the writer Arc lives
inside the Process struct for the program's lifetime.

PipeWriter refactored: `OwnedFd` → `AtomicI32 + custom Drop`. The
fd lives in an atomic; close swaps to `-1` and `libc::close`'s the
original; subsequent writes return errors. Drop is idempotent.
Lock-free per ZERO-MUTEX. Non-pipe backings (StringIoWriter,
RealStdout, RealStderr) get a default no-op close — closing real
OS stdio would break the parent process.

**`wat/std/sandbox.wat`** — wat-level reimplementation of
`run-sandboxed` / `-ast` atop spawn-program. **Intentionally not
bundled in `src/stdlib.rs`.** The substrate Rust impls in
`src/sandbox.rs` absorb startup / validation / panic failures
into `RunResult.failure`; the wat-level path can't yet replicate
that capture without the spawn-program error-as-data refactor
(returning `:Result<Process, StartupError>` instead of raising).
The wat-level scaffold lives in source as documentation of the
future shape — when the refactor lands, flip the `stdlib.rs`
inclusion and delete the Rust impls in one commit.

The scaffold compiles and is internally consistent; the
substrate dispatch arms keep winning at runtime because
`eval_tail` checks `sym.functions` after the dispatch match, so
the Rust impls take precedence. No regressions.

### Slice 103c — dispatcher demo (the EDN-stdin RPC pattern)

`wat-scripts/dispatch.wat` — 70 lines of wat. Reads one
`#demo/Job {:db-path :query-program}` EDN line from stdin, reads
the named program's source via `:wat::io::read-file`, spawns it
via `:wat::kernel::spawn-program` with the db-path piped in as
the inner's single stdin line, forwards the inner's stdout to
the dispatcher's own stdout, joins.

```bash
$ echo /tmp/demo.db | wat ./wat-scripts/seed-fixture.wat
seeded 5 logs to: /tmp/demo.db

$ echo '#demo/Job {:db-path "/tmp/demo.db" :query-program "./wat-scripts/count-logs.wat"}' \
    | wat ./wat-scripts/dispatch.wat
logs: 5

$ echo '#demo/Job {:db-path "/tmp/demo.db" :query-program "./wat-scripts/metrics-summary.wat"}' \
    | wat ./wat-scripts/dispatch.wat
logs: 5  metrics: 0
```

Two wat programs, two frozen worlds, three OS pipes between them.
The inner programs cannot see the dispatcher's bindings; they
share the binary's Rust shims (`:wat::sqlite::*`) but otherwise
communicate only through the pipe surface. **Hologram nesting in
operational form.**

### Slice 103d — INSCRIPTION + ZERO-MUTEX subsection + USER-GUIDE row + 058 row

This file. Plus:

- `docs/ZERO-MUTEX.md` gains "Mini-TCP across kernel pipes" as a
  subsection of the existing "Mini-TCP via paired channels"
  pattern. Same discipline, different transport.
- `docs/USER-GUIDE.md` §7 adds a `spawn-program` row alongside
  `spawn` / `send` / `recv` / `select`.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/`
  language spec gains a row for the new substrate symbols.
- `docs/arc/2026/04/103-kernel-spawn/HOLOGRAM.md` (already
  shipped) — the framing that fell out of the arc.

---

## What's deferred

The full deletion of `Vec<String>` from the substrate waits on:

1. **`spawn-program` error-as-data refactor** — change return type
   from `:Process` (raising on startup failure) to
   `:Result<:Process, :StartupError>`. The wat-level `run-sandboxed`
   helper handles the Err arm by building `RunResult.failure`.

2. **`ThreadDiedError::message` accessor** — extract the panic
   message string for inclusion in `RunResult.failure.message`.
   Today's `:wat::kernel::join-result` returns the typed enum but
   pattern-matching on its variants from wat hits a type-check
   bug (the scrutinee mis-infers as `:Option<?>`). Either fix the
   pattern matcher or add a substrate accessor.

Both are small. Both unlock the `wat/std/sandbox.wat` scaffold to
become the canonical implementation. Likely arc 104 territory.

---

## Operating principle that landed

The user's framing:

> "for real work we use real kernel pipes as the surface area of
> our programs"

Now scoped to its honest limit: **substrate primitives traffic in
real pipes; `Vec<String>` survives only inside the wat-level
test-convenience layer where collected output IS the assertion
target.** Arc 103a + 103c live up to it. Arc 103b's deferred
piece is the last sin to clear.

The HOLOGRAM.md framing makes this principled rather than
aesthetic: the surface between worlds is the binary's
capabilities; the program inside cannot remake that surface; the
protocol that crosses surfaces is line-delimited EDN. Real pipes
are the substrate's only honest answer for "communication between
holograms."
