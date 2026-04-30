# Arc 112 — INSCRIPTION

## Status

Shipped 2026-04-30. Inter-process typed I/O via `Process<I,O>`
unified shape + `process-send` / `process-recv` verbs + grammar
rule extension. cargo test --release green throughout slices;
97 test result rows, 0 failures across the workspace + lab.

Pushed:
- Slice 1: `592f564` (phantom params)
- Slice 2a/C1: `2bb0a7c` (mint ProcessDiedError)
- Slice 2a/C2: `2b4fc54` (ProgramHandle internals lift)
- Slice 2a/C3: `6f3f804` (substrate flip — Process<I,O> unification)
- Slice 2a/C4: `51c7549` (sonnet fixture sweep, 22 sites, 4 files)
- Slice 2b: `bae621c` (process-send / process-recv runtime + schemes)
- Slice 3: `6243ec9` (grammar rule extension)
- Slice 4: `a5184b0` (sonnet demo sweep — ping-pong, ping-pong-fork, dispatch)
- Slice 5: this INSCRIPTION + USER-GUIDE update + 058 row

DESIGN evolution: started as "fork-program returns ForkedChild;
slice 2 lands process-send/recv on it." Re-scoped mid-arc when
the user surfaced the structural-mirror principle (arc 111 had
ONE channel-type, ONE verb pair) — slice 2a unified
`Process<I,O>` and `ForkedChild<I,O>` under one struct as the
honest mirror. Captured at `5159f92` (initial DESIGN), `3b15372`
(slice 2a re-scope DESIGN update).

## What this arc adds

A typed-I/O protocol layer over fork-program / spawn-program
output. The two verbs that didn't exist before:

| Verb | Pre-arc-112 (raw) | Arc 112 (typed) |
|---|---|---|
| Send to peer | `(IOWriter/write-string proc.stdin (edn::write v))` + `\n` | `(:wat::kernel::process-send proc v)` |
| Recv from peer | `(IOReader/read-line proc.stdout)` → `(edn::read line)` | `(:wat::kernel::process-recv proc)` |

Plus the `Process<I,O>` type unification (one struct from both
spawn-program AND fork-program), the `ProcessDiedError` enum
(parallel to `ThreadDiedError` for the Process subject),
`Process/join-result` as the canonical wait verb, and the arc-110
grammar rule extended to flag silent disconnect on the new verbs.

### The three states (recv side)

Arc 112's `process-recv` mirrors arc 111's intra-process recv:

- `Ok(Some v)` — child wrote one EDN-framed `O` to stdout;
  parsed; here.
- `Ok(:None)` — child closed stdout cleanly; clean shutdown
  (stdout EOF + exit code 0 + no stderr).
- `Err(ProcessDiedError)` — child wrote to stderr OR exited
  non-zero. `died.message` carries the joined stderr + exit-code
  text. Three sub-conditions inside ProcessDiedError:
    - `Panic { message, failure }` — primary case
    - `RuntimeError { message }` — substrate-bug-class issues
    - `ChannelDisconnected` — pipe-side OS error

Arc 113 widens the `Err` arm to a `Vec<ProgramDiedError>` chain
that conjs at every cross-host hand-off boundary.

### Type unification — Process<I,O> is one struct

Pre-arc-112 the substrate had two near-identical struct types:

- `:wat::kernel::Process<I,O>` (returned by `spawn-program`,
  in-thread)
- `:wat::kernel::ForkedChild<I,O>` (returned by `fork-program`,
  out-of-process)

Same stdio fields (stdin/stdout/stderr); only the wait-handle
field differed (ProgramHandle<()> vs ChildHandle). Arc 112
slice 2a collapses them: `Process<I,O>` is the canonical type;
its `join` field is a `ProgramHandle<()>` whose internal Rust
enum (`ProgramHandleInner`) discriminates between in-thread
(crossbeam Receiver<SpawnOutcome>) and forked-OS-process
(Arc<ChildHandleInner> + waitpid). The wait mechanism became
implementation detail; the user-facing surface is one type.

`:wat::kernel::ForkedChild` retired. `:wat::kernel::wait-child`
(returning `:i64`) retired. `:wat::kernel::ChildHandle`
no longer wat-visible (still an internal Rust type backing the
Forked variant). The canonical wait verb is
`:wat::kernel::Process/join-result` returning
`:Result<:(), :wat::kernel::ProcessDiedError>`.

### The polymorphic-by-typeclass future (arc 109 § J)

Arc 112's slice 2a unification is a STEPPING STONE. Arc 109 § J
(planned slices 10a–10g) will:

- Rename today's unified `Process<I,O>` → `Program<I,O>` (the
  abstract supertype).
- Split `Program<I,O>` back into concrete `Thread<I,O>`
  (returned by `spawn-program`) and `Process<I,O>` (returned by
  `fork-program`). Both satisfy the `Program<I,O>` supertype.
- Mint `ProgramDiedError` as the error supertype; `ThreadDiedError`
  and `ProcessDiedError` both satisfy it (the error hierarchy
  mirrors the type hierarchy).
- Make `:wat::kernel::join-result`, `send`, and `recv`
  polymorphic over `Program<I,O>` via typeclass-style dispatch.

Arc 112 ships forward-compatibly: every today-`Process<I,O>` site
post-arc-112 will swap to `Program<I,O>` (or the appropriate
concrete type) via mechanical sweeps under § J. The substrate-as-
teacher pattern handles the migration.

### Grammar rule extension (slice 3)

Arc 110 made silent intra-process comm a compile error.
Slice 3 extends the same `validate_comm_positions` walk to the
inter-process verbs:

```rust
if matches!(head_str,
    ":wat::kernel::send"
    | ":wat::kernel::recv"
    | ":wat::kernel::process-send"     // arc 112 slice 3
    | ":wat::kernel::process-recv"     // arc 112 slice 3
) { ... }
```

Silent peer-Process death is impossible to express. Receivers
must match all three states; senders must handle the
`ChannelDisconnected` case via match or `result::expect`.

### Migration hint (substrate-as-teacher, fourth instance)

`src/check.rs::arc_112_migration_hint` detects the slice-2a
breakage classes (ForkedChild ↔ Process; ChildHandle ↔
Process/join) and appends the explicit migration path to every
TypeMismatch / ReturnTypeMismatch Display output. Same self-
describing pattern arcs 110 + 111 used. Sonnet's slice 2a + slice
4 sweeps consumed this stream as their brief.

Hint retirement task to follow when no consumer wat code emits
arc-112-shape mismatches anywhere — same retirement pattern as
arc 111's `arc_111_migration_hint`.

## Why

Arc 111 made intra-process comm typed end-to-end. The
out-of-process (fork) side was still bytes — every fork-program
caller did manual EDN render + `IOWriter::write-string` + manual
`IOReader::read-line` + EDN parse. Three concerns mixed at every
call site: transport, framing, semantics.

Arc 112 collapses those into one verb pair with the same
three-state return shape arc 111 minted. A wat program that
talks to a peer Program writes the SAME match arms regardless
of whether the peer runs in-thread or in a forked OS process.
The host kind is a user choice; the protocol is fixed (the
arc 114 principle, named during this arc's design phase).

User direction (2026-04-30):

> we are mirroring this for inter-process comms - the prior arc
> handled intra-process -- we are making inter-process
> structurally identical
>
> if you need to resolve something to provide the building blocks
> of simple things who compose into some complex thing with a
> simple surface - we do it / there are no shortcuts

The "no shortcuts" framing is what landed slice 2a — unifying
the type at the substrate level cost more than two-pairs-of-verbs
would have, and it was the right move.

## What this arc closes

- **The bytes-vs-typed asymmetry.** Pre-arc-112: intra-process
  comm was typed (`Result<Option<T>, ThreadDiedError>`); inter-
  process comm was bytes-with-manual-EDN. Post-arc-112: same
  shape, same arms, same algebra at every call site.
- **The two-types-for-one-thing redundancy.** Pre-arc-112:
  `Process<I,O>` and `ForkedChild<I,O>` were structurally
  identical except for the wait handle. Post-arc-112: one
  struct, one set of accessors, one wait verb.
- **The hosting-host-leak.** Pre-arc-112: the user picked the
  abstraction (in-thread vs forked) and wrote different code to
  consume them. Post-arc-112: the user picks the host; the
  protocol code is the same.
- **The class of bug arc 110 left half-closed.** Arc 110
  blocked silent intra-process disconnect. Slice 3 extends the
  same rule to inter-process. No silent peer-death in either
  direction.

## Slice walkthrough

### Slice 1 — phantom params (`592f564`)

Lifted `Process<I,O>` and `ForkedChild<I,O>` to carry phantom
type params documenting the typed protocol. Annotation-only;
no runtime change. Sonnet swept 22 fixture sites across 4 files.

REALIZATIONS captured: the `eprintln!`-as-debug-primitive trick
(the substrate-author audience, fourth in the substrate-as-
teacher layered-audience pattern); the `head -5` output-buffering
trap (`grep -c "type-check error"` IS the discipline).

### Slice 2a — Process<I,O> unification (4 commits)

C1 (`2bb0a7c`): mint `:wat::kernel::ProcessDiedError` enum +
constructor functions + `/message` and `/to-failure` accessors.
Additive; no behavior change.

C2 (`2b4fc54`): lift `Value::wat__kernel__ProgramHandle` from
`Arc<crossbeam_channel::Receiver<SpawnOutcome>>` to
`Arc<ProgramHandleInner>` (new internal enum with `InThread` /
`Forked` variants). `eval_kernel_join` + `eval_kernel_join_result`
dispatch on the variant. Forked arm unreachable until C3.

C3 (`6f3f804`): the substrate flip. `fork-program-ast` /
`fork-program` construct `Process<I,O>` (not `ForkedChild`) with
the join field's ProgramHandle wrapping the new Forked variant.
ForkedChild StructDef retired; wait-child verb retired;
`Process/join-result` minted; `arc_112_migration_hint` wired.
Substrate green; consumer fixtures fail with self-describing
TypeMismatch + hint.

C4 (`51c7549`): sonnet swept 15 fixture sites across 3 files
(tests/wat_arc104_fork_program.rs, tests/wat_fork.rs,
crates/wat-cli/tests/wat_cli.rs). Caught a pre-existing bug in
unwrap helpers (Result surfaces as `Value::Result`, not
`Value::Enum`) and corrected it.

### Slice 2b — process-send / process-recv (`bae621c`)

`eval_kernel_process_send`: pulls `Process.stdin` (IOWriter),
renders value via `edn_shim::value_to_edn_with` + `wat_edn::write`,
appends `\n`, calls `WatWriter::write_all`. `Ok(:())` on landed
write; `Err(ProcessDiedError::ChannelDisconnected)` on pipe close.

`eval_kernel_process_recv`: pulls `Process.stdout` (IOReader),
reads one line via `WatReader::read_line`, parses via
`edn_shim::read_edn`. On stdout EOF: drains stderr, dispatches on
the join handle's variant (InThread channel recv vs Forked
waitpid), synthesizes `Ok(:None)` or `Err(ProcessDiedError)` from
exit + stderr.

Slice-2b limitation (matches arc 105c hermetic.wat's pattern):
sequential read — stdout primarily, stderr drained on stdout EOF.
Children writing to stderr WHILE stdout is being read surface
stderr only after stdout EOFs. Multiplex-during-stream is
follow-up substrate work when a caller needs it.

Schemes parametric over `<I, O>`. Probe at
`tests/arc112_slice2b_process_send_recv.rs`.

### Slice 3 — grammar rule extension (`6243ec9`)

One-line change to `validate_comm_positions` in `src/check.rs`:
add `:wat::kernel::process-send` and `:wat::kernel::process-recv`
to the matches!. Same compile-error class; same self-describing
error message; silent peer-Process death becomes impossible.

Probe updated to use the canonical shapes (process-send wrapped
in `result::expect`; process-recv matched as scrutinee with three
arms) — exactly what the rule requires.

### Slice 4 — sonnet demo sweep (`a5184b0`)

Three demo scripts migrated:

- **ping-pong-fork.wat** — full fossil sweep (was still using
  ForkedChild + wait-child fossils from pre-slice-2a). Manual
  EDN + IOWriter became process-send inside result::expect;
  manual read-line + edn::read became process-recv inside match
  with three arms. Loop signature collapsed from
  `(req-w IOWriter, resp-r IOReader, ...)` to
  `(proc Process<Ping,Pong>, ...)`.
- **ping-pong.wat** — verb-only ergonomic migration; same loop
  collapse + send/recv pattern.
- **dispatch.wat** — minimal touch: kept raw byte-level child
  protocol (wire format would break under EDN encoding); only
  the type unification touch + `Process/join-result` migration.

Verification: all three scripts produce expected output verbatim;
cargo test --release green.

### Slice 5 — closure (this slice)

INSCRIPTION + USER-GUIDE update + 058 changelog row. The loot.

## The four questions (final)

**Obvious?** Yes. `process-send` / `process-recv` mirror arc 111's
`send` / `recv` shape. The same three-arm match works. The
unified `Process<I,O>` reads as "a typed channel to a running
program"; whether thread or fork is implementation. Same algebra
at every call site.

**Simple?** Yes. One struct (Process<I,O>), one wait verb
(Process/join-result), one pair of typed comm verbs (process-send,
process-recv), one error type (ProcessDiedError) with three
variants matching ThreadDiedError. The substrate's mechanism is
wider (ProgramHandleInner enum, EDN dispatch on send, sequential
multiplex on recv) but the surface stays narrow.

**Honest?** Yes. The transport asymmetry (crossbeam zero-copy for
in-thread vs. EDN bytes for forked) lives in the substrate; the
protocol surface is uniform. Slice-2b's "stdout-first, stderr-
drain-on-EOF" is named as a limitation in the doc, with the same
caveat hermetic.wat already shipped with. ProcessDiedError uses
the same variants as ThreadDiedError because the failure modes
are genuinely the same; only the subject's name differs.

**Good UX?** Yes. The substrate-as-teacher pattern delivered
again — sonnet swept 37 fixture sites across two passes (slice
2a + slice 4) consuming TypeMismatch + arc_112_migration_hint
output as its brief. The diagnostic stream told the migration
exactly. The user-facing API stays minimal (3 new verbs total
on Process<I,O>: send, recv, join-result).

## Cross-references

- `docs/arc/2026/04/112-inter-process-result-shape/DESIGN.md` —
  the original five-slice plan (with slice 2a re-scope captured
  inline).
- `docs/arc/2026/04/112-inter-process-result-shape/REALIZATIONS.md`
  — slice 1 realizations: eprintln-as-debug, output-buffering
  trap, substrate-as-teacher's fourth audience.
- `docs/arc/2026/04/112-inter-process-result-shape/SLICE-1-INVESTIGATION-2026-04-30.md`
  — the bank-and-retreat note kept as honest record of the
  threshold the user pushed me through.
- `docs/arc/2026/04/110-kernel-comm-expect/INSCRIPTION.md` — the
  intra-process grammar rule arc 112 slice 3 extends.
- `docs/arc/2026/04/111-result-option-recv/INSCRIPTION.md` — the
  intra-process type shape arc 112 mirrors.
- `docs/arc/2026/04/103-kernel-spawn/INSCRIPTION.md` —
  spawn-program substrate; today's `Process<I,O>` evolved from
  arc 103a's `Process` struct.
- `docs/arc/2026/04/104-wat-cli-fork-isolation/INSCRIPTION.md` —
  fork-program substrate; today's unified `Process<I,O>`
  generalizes ForkedChild's role.
- `docs/arc/2026/04/092-wat-edn-uuid-v4/` — the EDN v4 framing
  process-send/recv use on the wire.
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § J — the
  Program/Thread/Process supertype split + `ProgramDiedError`
  the slice 2a stepping stone leads into.
- `docs/arc/2026/04/113-cascading-runtime-errors/DESIGN.md` —
  the `Vec<ProgramDiedError>` chained-cause backtrace this arc's
  Err shape lifts cleanly into.
- `docs/arc/2026/04/114-spawn-as-thread/DESIGN.md` — the
  meta-principle "hosting is user choice; protocol is fixed"
  arc 112 instantiated and named during this arc's session.

## Queued follow-ups

- **Arc 113** — `Vec<ProgramDiedError>` chained-cause backtrace.
  Same conj-onto-Vec shape at every cross-host hand-off; lights
  up cross-host test failure diagnostics.
- **Arc 114** — kill spawn's R; ALL thread-side Programs return
  `:wat::kernel::unit` (no R-yielding bare spawn). Generalizes
  arc 112's "Programs have R = unit" stance across the substrate.
- **Arc 109 § J** — Program supertype split + typeclass dispatch.
  Renames today's unified `Process<I,O>` to `Program<I,O>`;
  splits into concrete Thread + Process; mints
  `ProgramDiedError` supertype; poly `:wat::kernel::send` /
  `:recv` / `:join-result`.
- **Hint retirement (task #168 + new arc 112 hint)** —
  `arc_112_migration_hint` removed when no consumer wat code
  emits arc-112-shape errors anywhere.
- **Slice-2b multiplex follow-up** — full concurrent stdout/
  stderr multiplex when a real caller needs streaming stderr
  during stdout consumption. Today's sequential pattern matches
  hermetic.wat's caveat.

After arcs 113 + 114 + 109 § J close, **arc 109 (kill-std)**
resumes its outer slicing (1c, 1d, 9d, 9e, 9f-9i).
