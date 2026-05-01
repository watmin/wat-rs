## Status

Drafted 2026-04-30 during arc 112 slice 2a. **DESIGN revised
2026-04-30 mid-pipeline** to remove the inherited `stdin/stdout/
stderr` framing on `Thread<I,O>` — threads don't have OS-process
stdio; their honest fields are `input` / `output` channels.
Captures the architectural commitment that **programs have no
useful return value** — they communicate values via channels in,
channels out, period. The contract.

User direction (2026-04-30):

> i don't think i want threads to have a meaningful ret val... they
> stream results out.. that's the contract
>
> i want the user to choose if they want intra-process or
> inter-process comms - they make the choice "does this belong in
> a fork or a thread?" but the contract is the same on how to
> interface with these programs - the hosting platform is a user
> choice - the protocol isn't

User direction (2026-04-30, mid-pipeline correction):

> threads do not have stdin,out,err -- that's a property of
> processes....
>
> a program is hosted somewhere. there's a communication into it...
> a pipe in and a pipe out.. a thread can have their panic handled
> explicitly.. for a process there's stdin and stdout and stderr
> is used to transport panic
>
> the thing-who-interfaces-with-a-program doesn't need to know or
> care what the hosting environment is

## The contract

A *Program* is a hosted computation with **three things every
caller can reach for**:

1. **Input channel** — a way to feed values IN.
2. **Output channel** — a way to read values OUT.
3. **Error mechanism** — a way to learn the program panicked and
   recover the cause.

That's it. Three concerns. Every Program — Thread or Process —
satisfies the contract; concrete satisfiers realize the three
concerns differently, but the abstraction is uniform:

| Concern | Thread<I,O> | Process<I,O> |
|---|---|---|
| Input | `Sender<I>` (in-memory crossbeam channel) | `stdin: IOWriter` (OS pipe) |
| Output | `Receiver<O>` (in-memory crossbeam channel) | `stdout: IOReader` (OS pipe) |
| Error mechanism | panic surfaces in `ThreadDiedError` chain on `Thread/join-result` (arc 060 + arc 113) | panic written to `stderr: IOReader` as EDN-framed chain (arc 113 slice 3) → `ProcessDiedError` on `Process/join-result` |

**The thing that interfaces with a Program does not know or care
which host backs it.** It uses the polymorphic verbs
(`process-send`, `process-recv`, `Program/join-result`) — the
substrate dispatches on the concrete satisfier's type tag.
Trade-offs of the host (cost, isolation, fault tolerance) are an
engineering decision the user makes at construction time;
downstream code is hosting-agnostic.

### Why threads return unit

A process **cannot** return `R` via "return" — the only path OUT
of a process is its stdout pipe. There's no mechanism to ferry
arbitrary `R` back through `fork` + `wait`.

For Programs to be uniform across hosts, threads must obey the
same constraint: **values flow through the output channel
exclusively**. The thread body's "return value" is `()` (unit)
— a marker for "completed without panic", not a value carrier.
`Thread/join-result` returns `:Result<:wat::core::unit,
:wat::kernel::ThreadDiedError>` — life status, not data.

This is the principle arc 114 names. Today's `:wat::kernel::spawn`
returning `:ProgramHandle<R>` (with `R` ferried via join) is
inconsistent with the Program contract and retires.

### Why threads have no stderr stream

Threads share memory with the parent. A thread's panic is caught
by the spawn driver's `catch_unwind`; the panic info rides through
the substrate's `SpawnOutcome` channel and surfaces on
`Thread/join-result`'s `Err` arm as a `ThreadDiedError` (or, per
arc 113, a `Vec<ThreadDiedError>` chain). The error mechanism
travels through the join handle, NOT through a side stream.

Processes need stderr because they share zero memory with the
parent post-fork — the only way out is OS streams. The substrate
frames panic info as EDN on stderr (arc 113 slice 3); the parent
parses it back into `Vec<ProcessDiedError>` via
`extract-panics`. **stderr exists on Process<I,O> because
processes need it; it does NOT exist on Thread<I,O> because
threads don't.**

The naming follows the realization: `stdin/stdout/stderr` is OS
process vocabulary; carrying it onto Thread<I,O> would be
dishonest. Thread<I,O>'s honest fields are `input` and `output`
(channels) plus the join handle — no stderr stream.

## The principle: hosting is a user choice; protocol is fixed

This is the architectural commitment arc 114 names. It lives
beyond arc 114's lifetime — every future substrate decision about
"running a Program somewhere" defers to it.

```
USER CHOICE                       SUBSTRATE INVARIANT
                                  (the protocol)

  ┌───────────────────┐
  │ spawn-thread<I,O> │──────────►┐
  │ (in-thread host)  │           │
  └───────────────────┘           ▼
                          ┌──────────────────────────┐
                          │   Program<I,O>           │
                          │   ─────────────────────  │
                          │   input  channel         │
                          │   output channel         │
                          │   error mechanism (panic │
                          │     surfaces on join)    │
                          │   join-result :          │
                          │     Result<unit,         │
                          │       *DiedError-chain>  │
                          └──────────────────────────┘
                                  ▲
  ┌───────────────────┐           │
  │ fork-program<I,O> │──────────►┘
  │ (out-of-process)  │
  └───────────────────┘
```

**Reading the picture:** the user picks the box on the left based
on engineering trade-offs (cost, isolation, fault tolerance). The
box on the right is what they GET — the same three concerns
(input / output / error mechanism) regardless of the choice.
Calling code on top of the right-side shape is hosting-agnostic;
nobody on the right of the picture knows or cares whether the
program runs in a thread or a forked process. The CONCRETE field
shapes differ between Thread<I,O> and Process<I,O> (channels vs
OS pipes); the polymorphic verbs hide that gap.

### `ProgramHandle` de-parameterizes

A direct consequence of "programs return unit": today's
`:wat::kernel::ProgramHandle<R>` is always `:ProgramHandle<()>`
post-arc-114 — every join handle yields unit. The `<R>` type
parameter has no use; the substrate carries dead weight every
time a handle is annotated.

Arc 114 collapses `:wat::kernel::ProgramHandle<R>` to plain
`:wat::kernel::ProgramHandle` (no type parameter). The `Result`
shape from `Program/join-result` becomes
`:Result<:wat::core::unit, :wat::kernel::ProgramDiedError-chain>`
without a phantom-`R` slot leaking the retired bare-spawn path.

This is symmetric with arc 113's
`Vec<*DiedError>`-not-a-Vec-of-anything-else realization: the
shape collapses to its honest minimum once the principle that
forced the parameter retires.

This generalizes prior wat substrate principles:

- **Arc 103** unified channels under `Sender<T>` / `Receiver<T>`
  with one send/recv API regardless of bounded vs. unbounded
  backing.
- **Arc 110** made silent kernel-comm illegal — one grammar rule
  applies to every comm site.
- **Arc 111** made the recv-result shape uniform —
  `Result<Option<T>, ThreadDiedError>` regardless of channel kind.
- **Arc 112** unified the typed Program shape across in-thread
  and forked hosts.
- **Arc 114** names the meta-principle these instances express:
  the substrate fixes the interface; the user picks the host.

After arc 114, code that wants to swap a Program from in-thread to
out-of-process (or vice versa) changes ONE call site (the
`spawn-thread` ↔ `fork-program`) and nothing else. Hosting
becomes a tunable, not a rewrite.

## Transport asymmetry: same contract, two implementations

The contract surface is uniform — every Program has input,
output, and an error mechanism. The substrate's TRANSPORT
mechanism under the hood differs by host:

| Concern | Thread<I,O> (in-memory) | Process<I,O> (forked OS process) |
|---|---|---|
| Input | `input: Sender<I>` (crossbeam) | `stdin: IOWriter` (OS pipe) |
| Output | `output: Receiver<O>` (crossbeam) | `stdout: IOReader` (OS pipe) |
| Error mechanism | panic caught in spawn driver's catch_unwind; surfaces on `join-result`'s Err arm as `ThreadDiedError` chain (no separate stream) | child writes EDN-framed chain to `stderr: IOReader` before _exit; parent's `extract-panics` recovers it; surfaces on `join-result` as `ProcessDiedError` chain |
| Wire format | `Arc<Value>` zero-copy enqueued in-memory | EDN-serialized `String` per line |

User direction (2026-04-30):

> threads need to use crossbeam to pass values - not edn - in
> threads we can pass concrete things in memory - forks cannot do
> this so its edn strings between them

This is not a contract leak — it's a substrate optimization the
contract hides. The honest read:

- **Thread<I,O> can pass Value-as-Arc.** Both ends share memory;
  the crossbeam channel ferries `Arc<Value>` zero-copy.
  Serialization is wasted work; types are preserved natively;
  arbitrary Rust `Value` shapes survive the transport without
  `:wat::edn::write` / `:wat::edn::read` round-trips. EDN at
  the call site would be a tax for crossing nothing.
- **Process<I,O> cannot.** The forked child shares zero memory
  with the parent post-fork; pointers / Arcs are meaningless
  across the boundary. The wire is bytes on a pipe; the framing
  is line-delimited EDN per arc 092. Every send/recv is
  serialize → write → read → parse.

The user-visible verbs (`process-send` / `process-recv`) are the
same on both. Substrate dispatches on the concrete `Program`
satisfier:

```rust
fn eval_kernel_process_send(prog: Value, value: Value) -> ... {
    match prog {
        Thread<I,O> => sender.send(Arc::new(value)),     // crossbeam
        Process<I,O> => {                                // pipe + EDN
            let edn = render_edn(&value);
            iowriter.write_string(format!("{}\n", edn))
        }
    }
}
```

User code that does
`(:wat::kernel::process-send prog (:my::struct/new ...))` reads
identically regardless of host. The internal cost differs (zero-
copy vs. EDN render+parse); the surface doesn't.

### Implications for arc 109 § J's Program supertype

`:wat::kernel::Program<I,O>` is the abstraction. Concrete satisfiers
have DIFFERENT field shapes internally:

- `Thread<I,O>` fields: `input: Sender<I>`, `output: Receiver<O>`,
  `join: ProgramHandle` — three fields. **No stderr** (errors
  surface on join-result's chain).
- `Process<I,O>` fields: `stdin: IOWriter`, `stdout: IOReader`,
  `stderr: IOReader`, `join: ProgramHandle` — four fields. stderr
  is the panic transport (parent's `extract-panics` reads it
  post-waitpid).

The supertype satisfaction rule isn't "same field types" — it's
"same operations supported." The polymorphic verbs
(`process-send`, `process-recv`, `Program/join-result`) work on
either; the per-host accessors (`Thread/input`, `Process/stdin`)
return different concrete types but the typed comm verbs work
uniformly.

This makes Program<I,O> a **typeclass / protocol** in the formal
sense — it's defined by the operations it supports, not by its
internal field shape. The substrate's first such abstraction.

### Direct accessor escape hatch

For programs that want to bypass the protocol (e.g., wat-cli's
stdio proxy moves raw bytes through the Process; a direct-Sender
consumer wants the raw crossbeam reference), the per-host
accessors stay available:

- `:wat::kernel::Thread/output` returns `:Receiver<O>`.
- `:wat::kernel::Process/stdout` returns `:wat::io::IOReader`.

Caller chooses: protocol-level (`process-recv` works on either)
or implementation-level (`Thread/output` for the crossbeam
receiver; `Process/stdout` for the pipe). The protocol is the
ergonomic default; the per-host accessors are the escape hatch
for callers who genuinely need the underlying transport.

## The pathology

Today's substrate has TWO different shapes for "do work on
another thread":

1. `:wat::kernel::spawn (fn :Fn(...) -> :R) -> :ProgramHandle<R>` —
   arc 060. The spawned function returns R; `join-result` yields
   `:Result<R, ThreadDiedError>`. R can be any type. Caller asks
   "what did the function compute?"
2. `:wat::kernel::spawn-program (...) -> :Process<I, O>` /
   `:wat::kernel::fork-program (...) -> :Process<I, O>` — arcs 103
   + 104 + 112. The spawned PROGRAM has stdin/stdout/stderr;
   `join-result` yields `:Result<:wat::core::unit, ProcessDiedError>`.
   R is fixed to unit. Caller streams data through the typed pipe.

Two different mental models for fundamentally one operation
("run X on another thread"). The duplication leaks into every
caller — they choose between "R via join channel" and "R via
typed pipe" based on which spawn verb they reached for.

The arc 112 + arc 109 § J architectural commitment is:
**Programs have no useful return value.** Arc 114 extends that
across the substrate: ALL thread-side primitives produce typed
I/O channels, never an R via join.

## The new shape

```
:wat::kernel::spawn-thread<I, O>
  (body :Fn(:wat::kernel::Receiver<I>,
            :wat::kernel::Sender<O>) -> :wat::core::unit)
  -> :wat::kernel::Thread<I, O>
```

The spawn body takes the **inside** ends of the input / output
channels (it READS from input, WRITES to output) and returns
unit. **Different from `:user::main`'s signature** — programs
running on a process get OS stdio (because that's all a process
has); programs running on a thread get typed channel halves
(because that's the honest in-memory primitive). The body shape
matches the host.

The Thread<I, O> the parent receives from `spawn-thread` exposes:

- `:wat::kernel::Thread/input`  — `Sender<I>` (parent → thread)
- `:wat::kernel::Thread/output` — `Receiver<O>` (thread → parent, typed `O`)
- `:wat::kernel::Thread/join-result` →
  `:Result<:wat::core::unit, :Vec<:wat::kernel::ThreadDiedError>>`
  (post-arc-113 chain shape)

No `Thread/stderr` accessor — threads have no err stream; panic
travels through the chain on `join-result`.

Symmetric-by-contract (NOT same-fields) with arc 112's
`Process<I, O>`. Both satisfy `:wat::kernel::Program<I, O>` (arc
109 § J's supertype). The polymorphic `:wat::kernel::join-result`
from arc 109 § J slice 10d works on either; per-host accessors
stay available for the rare caller who genuinely needs the raw
channel half or pipe.

## Migration of existing `:wat::kernel::spawn` consumers

Today's bare-spawn callers fall into three categories:

1. **Background workers** that compute X and send it on a
   channel the caller already holds. Migration is mostly
   cosmetic — they were already streaming out; their ProgramHandle
   was just being used as a "did the worker finish?" signal.
   `(:wat::kernel::spawn (fn () (some-work-and-send! tx)))` →
   `(:wat::kernel::spawn-thread (fn (input output)
   (some-work-and-send! tx)))` (the body's `input` / `output`
   are channel halves; the caller-held `tx` of pre-arc-114 is
   the same shape as the post-arc-114 `output`).

2. **Compute parallelism** — fork/join: spawn N functions, each
   returns R, parent collects the Rs. This pattern needs a
   migration: instead of `R via join`, the caller passes in a
   pre-made channel pair, the body sends R on its end, the parent
   collects from the receiver.
   ```
   ;; pre-arc-114
   ((handles :Vec<ProgramHandle<R>>) (map spawn-fn inputs))
   ((results :Vec<R>)                (map join-result handles))

   ;; post-arc-114
   ((channel-pairs :Vec<(Sender<R>, Receiver<R>)>)
    (map make-bounded-queue inputs))
   ((threads       :Vec<Thread<(),()>>)
    (map (fn (in pair) (spawn-thread (fn (...) (send (snd pair) (compute in))))) inputs pairs))
   ((results       :Vec<R>)
    (map (fn (pair) (recv (snd pair))) channel-pairs))
   ```
   More code at the call site; honest about the data path.

3. **Fire-and-forget** — spawn doesn't care about R. Migration
   trivial; the body just doesn't write to stdout.

## Implementation slices

Following arc 112's pattern (additive → sweep → retire):

| Slice | Work |
|---|---|
| **1** | Mint `:wat::kernel::Thread<I,O>` + accessors + `Thread/join-result` returning `:Result<:wat::core::unit, ThreadDiedError>`. Mint `:wat::kernel::spawn-thread` constructor verb. Additive — `:wat::kernel::spawn` continues to work. |
| **2** | Sweep substrate stdlib + lab — every `:wat::kernel::spawn` call site migrates. The fork/join compute-parallelism callers gain explicit channel pairs. |
| **3** | Retire `:wat::kernel::spawn` + the `<R>` parameter on `:wat::kernel::ProgramHandle`. Bare-spawn errors at startup with self-describing redirect to `:wat::kernel::spawn-thread`. `ProgramHandle` collapses to a non-parametric type — every join handle yields unit; the `<R>` slot was dead weight the bare-spawn path forced. Sonnet sweep against the resulting TypeMismatch hints. |
| **4** | Slice into arc 109 § J: rename arc-112's unified `Process<I,O>` (today returned by both spawn-program and fork-program) → `Program<I,O>` (abstract supertype); split into concrete `Thread<I,O>` (returned by spawn-program) + `Process<I,O>` (returned by fork-program); join-result becomes poly via the typeclass dispatch arc 109 § J slice 10d mints. |
| **5** | INSCRIPTION + USER-GUIDE + 058 row. |

Slice 4 is where this arc and arc 109 § J interlock. Either arc
can ship 1–3 first; the slice 4 work is shared.

## What this arc does NOT do

- Does NOT change channel semantics (arc 111 owns those).
- Does NOT touch fork-program / spawn-program — those are
  Program-shaped today and remain so. Arc 114 only generalizes
  the bare-spawn path TO match.
- Does NOT introduce typeclass dispatch for `join-result` — that
  lives in arc 109 § J slice 10d. Arc 114 lands the typed
  `Thread/join-result` form; bare `join-result` polymorphism is
  a separate substrate addition.

## The four questions

**Obvious?** Yes. Threads and Processes are unified by the
**contract** (input / output / error mechanism), not by field
shape. Both produce no R via join. The user picks the host; the
verbs that work on Programs work the same on either. One mental
model: "I have a Program; I feed it values via input; I read
values via output; I learn about panic via join-result." That's
all.

**Simple?** Caller-side: simpler (one mechanism for value flow).
Substrate-side: removes a verb (spawn) and a type
(`ProgramHandle<R>`); adds `spawn-thread` + `Thread<I,O>`. Net
substrate complexity ≈ neutral, but the conceptual surface
collapses.

**Honest?** Yes. Three asymmetries dissolve:

1. The R-via-join lie — bare-spawn had a return channel
   fork-program could never produce. Arc 114 retires R uniformly;
   programs deliver values through their output channel because
   that's the only thing every host can do.
2. The stdin/stdout/stderr-on-Thread lie — pre-arc-114, the
   substrate allocated three OS pipes for in-thread spawn and
   wrapped them as IOReader/IOWriter. Threads don't have OS
   stdio; they have channels. Arc 114 gives Thread<I,O> honest
   field names (`input` / `output`) backed by crossbeam, and
   no err stream (panic surfaces on the join chain).
3. The `<R>` parameter on `ProgramHandle` — once R is uniformly
   unit, the type parameter has no use. Arc 114 collapses
   `:ProgramHandle<()>` to plain `:ProgramHandle`.

**Good UX?** Mixed in transition (compute-parallelism callers
write more code post-migration). Long-term: stronger — one
pattern (channels for data, join for terminal-state) covers
every concurrency primitive.

## Error hierarchy parallel

Arc 114's Thread<I,O> uses `:wat::kernel::ThreadDiedError` as
the Err arm of `Thread/join-result` — same enum arc 060 minted.
Arc 109 § J slice 10d adds `:wat::kernel::ProgramDiedError` as
the supertype both `ThreadDiedError` and `ProcessDiedError`
satisfy; receivers that don't care about host match against
`ProgramDiedError` and read both kinds via the typeclass.

```
:wat::kernel::Program<I,O>      ⟸  Thread<I,O>      |  Process<I,O>
:wat::kernel::ProgramDiedError  ⟸  ThreadDiedError  |  ProcessDiedError
```

Arc 113 (`Vec<ProgramDiedError>` chained-cause backtrace)
generalizes naturally over arc 114's Thread output — every conj
at every hand-off operates against the supertype; the chain
crosses host boundaries data-faithfully.

## Cross-references

- `docs/arc/2026/04/060-join-result/INSCRIPTION.md` — the original
  spawn+ThreadDiedError shape arc 114 generalizes.
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § J — the
  Program/Thread/Process supertype split arc 114 lands into;
  also where `ProgramDiedError` supertype lives.
- `docs/arc/2026/04/112-inter-process-result-shape/DESIGN.md` —
  the "Programs have R = unit" commitment arc 114 generalizes
  across the substrate.
- `docs/arc/2026/04/113-cascading-runtime-errors/DESIGN.md` —
  `Vec<ProgramDiedError>` chained-cause backtrace arc 114's
  Thread/join-result lifts cleanly into.
