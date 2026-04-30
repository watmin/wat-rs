# Arc 112 — inter-process `Result<Option<T>, ThreadDiedError>` shape over fork-program stdio

## Status

Drafted 2026-04-30. Generalizes arc 111's intra-process type
shape to fork-program subprocess pipes. Same `Result<Option<T>,
E>` surface; different transport (bytes-on-pipe + EDN
serialization vs. typed Arc-on-channel). Includes the inter-
process equivalent of arc 110's grammar rule. Lands after arc
111 closure; arc 113 (rich-Err propagation) fills in the panic
payload uniformly across in-memory and inter-process channels.

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
   -> :Result<:(), :wat::kernel::ThreadDiedError>

(:wat::kernel::process-recv proc)
   -> :Result<:Option<T>, :wat::kernel::ThreadDiedError>
```

Symmetric with arc 111's intra-process forms. The verbs are
named `process-*` to distinguish from `:wat::kernel::send` /
`:wat::kernel::recv` (which operate on crossbeam channels).

`proc` is a `:wat::kernel::Process<I, O>` (or `ForkedChild<I,
O>`) — a typed wrapper over the existing untyped fork-program /
spawn-program return:

- `I` — input payload type the parent sends to child's stdin
- `O` — output payload type the child sends to parent's stdout

stderr carries panic / diagnostic info; surfaces in the `Err`
variant of process-recv's Result, separate from the `O`-typed
stdout stream.

## The three states (recv side)

Same arms as arc 111's intra-process recv:

- `Ok(Some v)` — child wrote one EDN-framed `O` to stdout;
  parsed; here it is.
- `Ok(:None)` — child closed stdout cleanly; clean shutdown.
  Both stdout and stderr returned EOF without an error
  signal; child exited 0.
- `Err(ThreadDiedError)` — one of three sub-conditions:
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

### Slice 1 — `Process<I, O>` typed wrapper struct

A new substrate type that wraps the existing
`(IOWriter, IOReader, IOReader, ProgramHandle<()>)` shape:

```rust
struct WatProcess {
    stdin: Arc<dyn WatWriter>,        // existing IOWriter
    stdout: Arc<dyn WatReader>,       // existing IOReader
    stderr: Arc<dyn WatReader>,       // existing IOReader
    handle: ProgramHandle<Value>,     // existing handle
    // type information visible only at the type-checker level;
    // not stored at runtime (Rust generics erase, so the wrapper
    // is monomorphic at the value layer)
}
```

`:wat::kernel::Process<I, O>` is the typed face. `fork-program`
and `spawn-program` return the typed shape; user code can still
access the raw pipes via accessors if they need bytes-only
channels (e.g., the wat-cli stdio proxy).

Two new accessors:

- `(:wat::kernel::Process/stdin proc)` — returns the
  `IOWriter` for raw bytes (escape hatch).
- `(:wat::kernel::Process/handle proc)` — returns the
  `ProgramHandle<()>` for join-result.

The stdout/stderr readers are NOT exposed as separate
accessors; arc 112's contract is that you talk to a process
through `process-send` / `process-recv`, OR you opt out and
use the raw `IOWriter`/`IOReader`s. Mixing typed and raw on
the same Process is a usage bug; unenforced today, possibly
a runtime check in arc 113 if it surfaces as a real defect.

### Slice 2 — `process-send` and `process-recv` substrate fns

`eval_kernel_process_send(proc, value)`:

1. Render `value` via `:wat::edn::write` (arc 092 EDN v4).
2. Append newline.
3. Write to `proc.stdin` via `IOWriter/write-string`.
4. Return `Ok(())` on successful write; `Err(ChannelDisconnected)`
   if the pipe is closed (child exited or panicked before
   reading).

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

The `arc_111_migration_hint` extends to detect arc-112-shape
mismatches too — same self-describing pattern (substrate is
the teacher / progress meter / brief).

### Slice 4 — sweep consumers

The wat-cli is the primary consumer of fork-program; its stdio
proxy code currently does raw byte forwarding. Likely stays on
the raw escape-hatch path — bytes ARE its job. The dispatcher
and ping-pong demos (arc 103c, 141) DO talk EDN over fork
pipes; they migrate to `process-send` / `process-recv`. Worked
examples in arc 103/104/106 documentation examples follow.

### Slice 5 — INSCRIPTION + USER-GUIDE + 058 row

Same closure shape as arcs 110 + 111.

## What this arc does NOT do

- Does NOT propagate the actual child-process panic message
  through `Err` beyond stderr's bytes. Arc 113 fills in the
  rich payload via a uniform mechanism (the OnceLock pieces
  generalize: stderr framing IS the cross-process equivalent
  of an in-memory panic-cell write).
- Does NOT change `fork-program` / `spawn-program`'s outer
  signature. Both still return `ForkedChild` / `Process<I,O>`
  / `Result<Process, StartupError>`. Slice 1 just types `I`
  and `O` and adds the accessors.
- Does NOT make raw `IOWriter` / `IOReader` access illegal.
  The wat-cli's stdio proxy and any program doing genuine
  byte-level work keeps the raw path. The typed primitives
  are the ergonomic default; raw is the escape hatch.
- Does NOT remove arc 110's grammar rule for the in-process
  primitives. Slice 3 extends the rule; doesn't replace it.

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

| Slice | Work |
|---|---|
| **1** | `Process<I, O>` typed wrapper + accessors |
| **2** | `process-send` / `process-recv` runtime + schemes |
| **3** | Grammar rule extension (arc 110 +process-* verbs) |
| **4** | Sweep consumers (dispatcher/ping-pong demos; doc examples) |
| **5** | INSCRIPTION + USER-GUIDE + 058 row |

Each slice ends green. Slice 1+2 are the structural shipment;
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
