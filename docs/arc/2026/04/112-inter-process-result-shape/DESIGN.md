# Arc 112 — inter-process `Result<Option<T>, ProcessDiedError>` shape over Process<I,O> stdio

## Future evolution (architectural pointer; arc 109 § J / 113 / 114)

Arc 112 slice 2a unified `:wat::kernel::Process<I,O>` as the
single struct returned by both spawn-program and fork-program.
That's the right structural mirror for arc 111 (one type, one
verb pair); it's also a STEPPING STONE toward a sharper
architecture captured across arcs 109 / 113 / 114:

```
Program<I,O>          ⟸  Thread<I,O>      |  Process<I,O>
ProgramDiedError      ⟸  ThreadDiedError  |  ProcessDiedError
Vec<ProgramDiedError> ← chained-cause backtrace, conj at every hand-off
```

**Today's `Process<I,O>` (slice 2a's name) becomes tomorrow's
`Program<I,O>` (the abstract supertype).** Concrete satisfiers
`Thread<I,O>` (in-thread, from spawn-program) and
`Process<I,O>` (out-of-process, from fork-program) split out
under the supertype with different transport mechanisms
(crossbeam vs pipe+EDN) but uniform protocol surface.

Pointers:
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § J — Program /
  Thread / Process supertype split + ProgramDiedError mirror.
  Slice 10d mints the typeclass infrastructure.
- `docs/arc/2026/04/113-cascading-runtime-errors/DESIGN.md` —
  `Vec<ProgramDiedError>` chained-cause backtrace.
- `docs/arc/2026/04/114-spawn-as-thread/DESIGN.md` — kill
  spawn's R; the meta-principle "hosting is user choice;
  protocol is fixed."

Arc 112's slice 2a shipment is forward-compatible — every
`Process<I,O>` annotation in the substrate + lab today migrates
to the new shape via mechanical sweeps (arcs 109 § J / 114 each
do their own substrate-as-teacher pass).

## Status

Drafted 2026-04-30. Slice 1 shipped (`592f564`) — phantom params
on `Process<I,O>` and `ForkedChild<I,O>`. **Re-scoped 2026-04-30
session-2** to mirror arc 111 structurally: ONE typed-channel
struct (`Process<I,O>`) returned by both `spawn-program` AND
`fork-program`; ONE pair of bare verbs (`process-send` /
`process-recv`); ONE error type for Process-side death
(`ProcessDiedError`, distinct from `ThreadDiedError` because
the subjects are distinct — channels' peer is a thread,
processes' peer is a Program). Same `Result<Option<T>, E>`
surface; different transport (bytes-on-pipe + EDN serialization
vs. typed Arc-on-channel). Includes the inter-process equivalent
of arc 110's grammar rule. Arc 113 (rich-Err propagation) fills
in the panic payload uniformly across in-memory and inter-
process channels.

User direction (2026-04-30, session 2):

> we are mirroring this for inter-process comms - the prior arc
> handled intra-process -- we are making inter-process
> structurally identical
>
> ThreadDiedError should become ProcessDiedError when using
> processes?
>
> if you need to resolve something to provide the building blocks
> of simple things who compose into some complex thing with a
> simple surface - we do it / there are no shortcuts

## The pathology

`:wat::kernel::fork-program` returns a `ForkedChild` exposing
`(IOWriter, IOReader, IOReader, ProgramHandle<()>)` — raw stdio
pipes. Today's wat-cli and dispatcher demos read/write bytes
+ newline framing + EDN parse/render at the call site. Three
separate concerns mix at every fork-program user:

1. **Transport** — bytes on a Unix pipe; readable line by line.
2. **Framing** — newline-delimited EDN per arc 092 (line per
   message).
3. **Semantics** — what TYPE the bytes encode; whether `:None`
   was sent; whether stderr noise means panic.

Compare with arc 111's intra-process channel: send/recv return
`Result<Option<T>, ThreadDiedError>` end-to-end, ALL three
concerns collapsed into one type-shaped read or write. The
caller's mental model is one operation per direction.

The asymmetry is the bug. A wat program reading from a
forked subprocess should get the same arms it gets from an
in-memory channel — only the transport differs.

User direction (2026-04-30, captured in arc 110 DESIGN
follow-up):

> once we make this work on in memory comms, then we'll do the
> same of inter process comms.. send on std in, recv on stdout
> -> good, recv on stderr -> bad
>
> this has the same shape as Result<Option<T>,E>  the Option<T>
> is sent on stdout and E is sent on stderr

## The new return shape

```
(:wat::kernel::process-send proc value)
   -> :Result<:(), :wat::kernel::ProcessDiedError>

(:wat::kernel::process-recv proc)
   -> :Result<:Option<T>, :wat::kernel::ProcessDiedError>
```

Symmetric with arc 111's intra-process forms. The verbs are
named `process-*` to distinguish from `:wat::kernel::send` /
`:wat::kernel::recv` (which operate on crossbeam channels).

`proc` is a `:wat::kernel::Process<I, O>` — a typed wrapper that
arc 112 unifies as the SINGLE return shape for both
`spawn-program` (in-thread) AND `fork-program` (out-of-process).
Pre-arc-112 the substrate had two near-identical struct types
(`Process<I,O>` for spawn, `ForkedChild<I,O>` for fork) that
carried the same stdio fields plus a different wait-handle
field. Arc 112 collapses them: `Process<I,O>` is the canonical
type, its `join` field a `ProgramHandle<()>` whose internal
representation discriminates between in-thread (Arc-on-channel
SpawnOutcome) and out-of-process (waitpid-on-pid). The wait
mechanism becomes implementation detail; the user sees one
type, one set of accessors, one wait verb.

- `I` — input payload type the parent sends to child's stdin
- `O` — output payload type the child sends to parent's stdout

stderr carries panic / diagnostic info; surfaces in the `Err`
variant of process-recv's Result, separate from the `O`-typed
stdout stream.

### The error type — `ProcessDiedError`

Arc 112 introduces `:wat::kernel::ProcessDiedError` as the Err
type for verbs that operate on `Process<I,O>`. Channels keep
`:wat::kernel::ThreadDiedError`. Both have identical variant
shapes:

- `Panic { message: String, failure: Option<Failure> }`
- `RuntimeError { message: String }`
- `ChannelDisconnected`

Why two enums with identical variants: the subject's name
tracks at the type. A receiver holding a `Receiver<T>` (channel
peer = thread) gets `ThreadDiedError`; a receiver holding a
`Process<I,O>` (peer = Program — in-thread or forked) gets
`ProcessDiedError`. Receivers write `((Err died) ...)` either
way; the type annotation at the binding signals which subject
died.

Arc 060's `:wat::kernel::join-result` becomes polymorphic on
the `ProgramHandle<T>` it operates on. A `ProgramHandle<T>`
returned by `:wat::kernel::spawn` keeps `Result<T,
ThreadDiedError>` (arc 060 unchanged at that surface). A
`ProgramHandle<()>` accessed via `Process/join` returns
`Result<(), ProcessDiedError>`. Same handle TYPE shape; the Err
variant tracks the subject the handle was minted for.

Arc 113's `Vec<TDE>` chained-cause backtrace generalizes
naturally: `Vec<ProcessDiedError>` for Process-side, `Vec<ThreadDiedError>`
for channel-side. Same chain shape; element type tracks the
subject.

## The three states (recv side)

Same arms as arc 111's intra-process recv:

- `Ok(Some v)` — child wrote one EDN-framed `O` to stdout;
  parsed; here it is.
- `Ok(:None)` — child closed stdout cleanly; clean shutdown.
  Both stdout and stderr returned EOF without an error
  signal; child exited 0.
- `Err(ProcessDiedError)` — one of three sub-conditions:
  - Child wrote to stderr (anything non-empty) — `Err(Panic
    { message, failure })` carries the stderr bytes (or a
    parsed assertion payload if the framing matches arc 064's
    `AssertionPayload` shape).
  - Child exited non-zero with empty stderr — `Err(Panic
    { message: "child exited <code>", failure: :None })`.
  - Pipe-side error (rare) — `Err(ChannelDisconnected)` for
    OS-level fd issues that aren't attributable to the child.

The receiver's match arms read the same as in-process recv;
the implementation underneath does the multiplexing work.

## Implementation

### Slice 1 (SHIPPED 2026-04-30, `592f564`) — phantom params

`Process<I, O>` and `ForkedChild<I, O>` both gained two phantom
type params. Schemes for `spawn-program` / `spawn-program-ast`
/ `fork-program` / `fork-program-ast` lifted to parametric
returns. 22-site fixture sweep + REALIZATIONS captured (the
eprintln-as-debug-primitive realization). Slice 1 is annotation-
only; no runtime change.

Slice 1 left BOTH types alive. Slice 2a unifies them.

### Slice 2a — unify `Process<I,O>` and `ForkedChild<I,O>`

The structural-mirror requirement (one channel-type, one verb
pair, like arc 111) demands one Process struct. `ForkedChild<I,
O>` retires; `fork-program` / `fork-program-ast` return
`Process<I, O>`.

Substrate (Rust):

- `Value::wat__kernel__ProgramHandle` lifts from
  `Arc<crossbeam_channel::Receiver<SpawnOutcome>>` to an inner
  enum:
  ```rust
  enum ProgramHandleInner {
      InThread(crossbeam_channel::Receiver<SpawnOutcome>),
      Forked(Arc<ChildHandleInner>),
  }
  ```
- `eval_kernel_join_result` dispatches on the variant. InThread
  arm: arc 060 logic unchanged (returns
  `Result<R, ThreadDiedError>` when called on a spawn-handle).
  Forked arm: waitpid + exit-code interpretation (returns
  `Result<(), ProcessDiedError>` when called on a Process/join
  handle).
- `:wat::kernel::ProcessDiedError` enum minted with the same
  three variants as `ThreadDiedError`: `Panic { message,
  failure }`, `RuntimeError { message }`, `ChannelDisconnected`.
- `fork-program-ast` / `fork-program` return
  `Value::Struct { type_name: ":wat::kernel::Process", ... }`
  with `join: ProgramHandle<()>` carrying the Forked variant.
- `Value::wat__kernel__ChildHandle` retires (or stays as an
  internal-only payload of the Forked variant; not user-visible).
- `:wat::kernel::wait-child` retires. Callers reach for
  `(:wat::kernel::join-result (:wat::kernel::Process/join proc))`.
- `:wat::kernel::ForkedChild` type retires from `types.rs`.

Substrate stdlib (wat):

- `wat/std/sandbox.wat::drive-sandbox` already uses Process; no
  change.
- `wat/std/hermetic.wat::run-sandboxed-hermetic-ast` swaps
  `ForkedChild<I,O>` → `Process<I,O>`. The exit-code
  interpretation (`failure-from-exit`, `exit-code-prefix`,
  `failure-message-for-code`) MOVES into substrate
  `eval_kernel_join_result`'s Forked arm — its result lands as
  `Result<(), ProcessDiedError>` and the `Err` variants map
  cleanly to `Failure` via the existing
  `ThreadDiedError/to-failure` accessor (arc 105c) which gains
  a `ProcessDiedError/to-failure` sibling. The hand-written
  exit-code-to-prefix logic in hermetic.wat collapses.

Migration hint: `src/check.rs::arc_112_migration_hint(callee,
expected, got)` detects:
- `:ForkedChild<I,O>` annotations → suggest `:Process<I,O>`.
- `(:wait-child handle) → :i64` → suggest
  `(:join-result handle) → :Result<(),:ProcessDiedError>`.
- Type mismatch involving `:Result<:Option<T>,
  :ThreadDiedError>` on a Process subject → suggest
  `:Result<:Option<T>, :ProcessDiedError>`.

Self-describing pattern. Same three-audience read as arc 111
(humans / agents / orchestrators).

### Slice 2b — `process-send` / `process-recv` substrate fns

`eval_kernel_process_send(proc, value)`:

1. Render `value` via `:wat::edn::write` (arc 092 EDN v4).
2. Append newline.
3. Write to `proc.stdin` via `IOWriter/write-string`.
4. Return `Ok(())` on successful write;
   `Err(ChannelDisconnected)` if the pipe is closed (child
   exited or panicked before reading).

`eval_kernel_process_recv(proc)`:

1. **Multiplex stdout / stderr.** Both pipes can produce
   independently. Read one line from whichever pipe is ready
   first.
2. Cases:
   - **stdout ready, line non-empty:** parse via
     `:wat::edn::read`; return `Ok(Some parsed)`.
   - **stdout ready, EOF (empty line + close):** return
     `Ok(:None)` (clean shutdown).
   - **stderr ready, line non-empty:** parse as
     `AssertionPayload` if framing matches; otherwise raw
     string. Return `Err(Panic { message, failure })`.
   - **Both EOF, no stderr lines, child exited 0:** return
     `Ok(:None)`.
   - **Both EOF, child exited non-zero:** return
     `Err(Panic { message: "child exited <code>", failure: :None })`.

Multiplex implementation: `crossbeam_channel::select` over
two oneshot reader threads, OR `mio` / `nix::poll` on the
two fds. Spawn-thread per pipe is the simplest portable
shape and matches the existing wat-cli stdio-proxy pattern.

Schemes register `process-send` and `process-recv` polymorphic
over `<I, O>`, taking `Process<I, O>` as the first arg. `T` in
`Result<Option<T>, ProcessDiedError>` resolves to `O` for
`process-recv`; `process-send`'s second arg is typed `:I`.

### Slice 3 — grammar rule extends to inter-process comm

`validate_comm_positions` from arc 110 currently flags
`:wat::kernel::send` / `:wat::kernel::recv`. Slice 3 adds
`:wat::kernel::process-send` / `:wat::kernel::process-recv`
to the same rule:

```rust
if matches!(head_str,
    ":wat::kernel::send"
    | ":wat::kernel::recv"
    | ":wat::kernel::process-send"
    | ":wat::kernel::process-recv"
) {
    // ... same grammar check ...
}
```

The `arc_112_migration_hint` covers slice 2a's type rename
cases AND slice 2b's send/recv shape. Same self-describing
pattern (substrate is the teacher / progress meter / brief).

### Slice 4 — sweep consumers

After slice 2a lands, the substrate emits arc-112 migration
hints at every breakage point. The sweep is mechanical:

- `tests/wat_arc104_fork_program.rs`,
  `tests/wat_fork.rs`,
  `crates/wat-cli/tests/wat_cli.rs` —
  ForkedChild → Process; wait-child → join-result + match.
- The wat-cli's stdio proxy is the bytes-on-pipe path; stays on
  the raw escape-hatch path (its job IS bytes, not values).
- Dispatcher and ping-pong demos (arc 103c, 141) DO talk EDN
  over fork pipes; migrate to `process-send` / `process-recv`.

Sonnet sweep dispatched against the substrate's natural
diagnostic output (same pattern as arc 111).

### Slice 5 — INSCRIPTION + USER-GUIDE + 058 row

Same closure shape as arcs 110 + 111.

### Test discipline

Following the arc-111-validated pattern: write a small probe.wat
on disk that intentionally surfaces the new error class, run the
wat interpreter, verify the migration hint stream is exact. Then
sonnet sweep against `cargo test` output. `grep -c "hint: arc
112"` is the orchestrator-visible progress meter.

The arc 112 slice 1 probe (`tests/arc112_scheme_probe.rs`,
shipped 2026-04-30) is the unit-test promotion of that pattern;
slices 2a / 2b add a probe for the new error class.

## What this arc does NOT do

- Does NOT propagate the actual child-process panic message
  through `Err` beyond stderr's bytes. Arc 113 fills in the
  rich payload via a uniform mechanism (the OnceLock pieces
  generalize: stderr framing IS the cross-process equivalent
  of an in-memory panic-cell write).
- Does NOT make raw `IOWriter` / `IOReader` access illegal.
  The wat-cli's stdio proxy and any program doing genuine
  byte-level work keeps the raw path. The typed primitives
  are the ergonomic default; raw is the escape hatch.
- Does NOT remove arc 110's grammar rule for the in-process
  primitives. Slice 3 extends the rule; doesn't replace it.
- Does NOT rename `ThreadDiedError`. Channels keep that error
  type — their peer IS a thread. Only `Process<I,O>`-side death
  uses the new `ProcessDiedError`.

## The four questions

**Obvious?** Yes. Same return shape as arc 111. Mental model
is identical: `process-recv` returns the same arms as `recv`;
the multiplex over stdout/stderr is implementation detail.
The substrate hides the byte plumbing; users see typed values.

**Simple?** Mostly. The Process struct is a thin typed wrapper.
process-send is a single EDN-render-and-write. process-recv
multiplexes two pipes — non-trivial but bounded; one helper
function with a `select!` over per-pipe reader threads.

**Honest?** Yes. The transport difference (bytes vs.
typed-channels) is acknowledged in the IMPL but hidden from
the user behind the same algebra. Programs that legitimately
need raw bytes use the escape-hatch accessors; programs that
care about VALUES use process-send/recv and stay in the
algebra.

**Good UX?** Yes.
- Symmetry with intra-process is the win — caller writes
  the same code regardless of transport.
- Migration hint extension: same substrate-as-teacher pattern
  applies to arc-112-shape mismatches.
- The escape-hatch accessors mean no caller is forced into
  a typed shape they don't want.

## Slicing summary

| Slice | Work | Status |
|---|---|---|
| **1** | Phantom params on Process<I,O> + ForkedChild<I,O> | shipped `592f564` |
| **2a** | Unify Process<I,O> = ForkedChild<I,O>; mint ProcessDiedError; collapse wait-child into join-result | this slice |
| **2b** | `process-send` / `process-recv` runtime + schemes | this slice |
| **3** | Grammar rule extension (arc 110 +process-* verbs) | next |
| **4** | Sweep consumers (dispatcher/ping-pong demos; doc examples) | sonnet, after substrate green |
| **5** | INSCRIPTION + USER-GUIDE + 058 row | closure |

Each slice ends green. Slice 2a is the structural shipment
(the type unification building block); 2b lands the verb pair;
3 is the discipline; 4+5 close the arc.

## Cross-references

- `docs/arc/2026/04/110-kernel-comm-expect/` — grammar rule
  arc 112 extends.
- `docs/arc/2026/04/111-result-option-recv/INSCRIPTION.md` —
  intra-process type shape arc 112 mirrors.
- `docs/arc/2026/04/103-kernel-spawn/` — `spawn-program`
  primitive arc 112's typed wrapper encloses.
- `docs/arc/2026/04/104-wat-cli-fork-isolation/` —
  `fork-program` primitive same.
- `docs/arc/2026/04/092-wat-edn-uuid-v4/` — EDN v4 framing
  used at the typed-pipe boundary.
- `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels" —
  the pre-arc-112 per-call EDN render/parse pattern arc 112
  formalizes.

## Queued follow-up

**Arc 113 — cascading runtime error messages, as a value-built
backtrace.** User direction (2026-04-30):

> can we just... (conj backtrace err) and return a Vec of
> errors?... that'd be amazing

The shape arc 113 lands isn't a single `ThreadDiedError` per
panic. It's a **`Vec<ThreadDiedError>`** representing the
chain of failures that produced this Err — head is the most
recent panic; tail is the cause; recursively. A thread that
panics because it received an Err extends the incoming Vec
with its own panic info; an original panic creates a
singleton Vec.

`recv` / `send` / `process-recv` / `process-send` all share
the same chained-Err shape. The receiver's match arm reads:

```scheme
((Err died-chain)
  ;; died-chain :Vec<:wat::kernel::ThreadDiedError>
  ;; head = the thread we were talking to
  ;; tail = whatever killed it, transitively
  (handle-chain died-chain))
```

Implementation: arc 111's six OnceLock pieces from DESIGN.md,
but the panic-cell carries `Vec<ThreadDiedInfo>` instead of
single. Cross-process arc-112 stderr framing carries the Vec
as an EDN sequence; arc 113's wire format handles both
transports with one shape.

The chained-cause backtrace, surfaced as data, matchable,
propagatable — no special "Caused by" construct. The Vec IS
the call chain. Programs that want to ignore the depth use
`first` and look at the head; programs that want to reason
about cascades walk the Vec.

After 113, `Err(ChannelDisconnected)` becomes a rare empty-
chain variant for OS-level disconnects with no traceable
cause; rich `Panic` chains carry the backtrace whenever a
thread (or chain of threads) actually died.
