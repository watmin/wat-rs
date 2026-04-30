## Status

Drafted 2026-04-30 during arc 112 slice 2a. Captures the
architectural commitment that **threads have no useful return
value** — they stream results out via channels. The contract.

User direction (2026-04-30):

> i don't think i want threads to have a meaningful ret val... they
> stream results out.. that's the contract
>
> i want the user to choose if they want intra-process or
> inter-process comms - they make the choice "does this belong in
> a fork or a thread?" but the contract is the same on how to
> interface with these programs - the hosting platform is a user
> choice - the protocol isn't

Arc 114 names the principle the substrate has been growing toward
since arc 103: **the hosting platform is a user choice; the
protocol is fixed.**

- **Hosting platform** (the user chooses): in-thread (spawn-thread)
  vs. out-of-process (fork-program). Trade-offs are engineering —
  memory isolation, fault isolation, OS resource limits, copy-on-
  write efficiency, IPC cost. Different concerns at different
  scales.
- **Protocol** (the substrate fixes): every running Program — Thread
  or Process — exposes the SAME interface. `stdin: IOWriter`,
  `stdout: IOReader` (typed `O` via `process-recv` /
  `process-send`), `stderr: IOReader` (typed errors), and a wait
  verb (`Program/join-result` → `:Result<:wat::core::unit,
  *DiedError>`). Code that interacts with a Program does NOT
  branch on host kind.

Arc 114 generalizes arc 112's "Programs have R = unit" stance to
ALL thread-side abstractions. After arc 114 closes, no
substrate-provided thread/process verb yields a typed `R` via the
join channel — every running computation that wants to convey
data does so via a Channel/pipe. The shape of "I have a Program;
let me feed it data and read its output" is one thing the user
writes once, regardless of which host they picked.

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
                          ┌─────────────────────┐
                          │  Program<I,O>       │
                          │  ─────────────────  │
                          │  stdin  : IOWriter  │
                          │  stdout : IOReader  │
                          │  stderr : IOReader  │
                          │  join-result :      │
                          │    Result<unit,     │
                          │      *DiedError>    │
                          └─────────────────────┘
                                  ▲
  ┌───────────────────┐           │
  │ fork-program<I,O> │──────────►┘
  │ (out-of-process)  │
  └───────────────────┘
```

**Reading the picture:** the user picks the box on the left based
on engineering trade-offs (cost, isolation, fault tolerance). The
box on the right is what they GET — the same shape regardless of
the choice. Calling code on top of the right-side shape is
hosting-agnostic; nobody on the right of the picture knows or
cares whether the program runs in a thread or a forked process.

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

## Transport asymmetry: same protocol, two implementations

The protocol surface (`process-send` / `process-recv` on
`Program<I,O>`) is uniform — but the substrate's TRANSPORT
mechanism under the hood differs by host:

| Host | Stdin | Stdout / Stderr | Wire format |
|---|---|---|---|
| **Thread<I,O>** (in-memory) | crossbeam `Sender<Value>` | crossbeam `Receiver<Value>` | `Arc<Value>` zero-copy enqueued in-memory |
| **Process<I,O>** (forked OS process) | OS pipe `IOWriter` | OS pipe `IOReader` | EDN-serialized `String` per line |

User direction (2026-04-30):

> threads need to use crossbeam to pass values - not edn - in
> threads we can pass concrete things in memory - forks cannot do
> this so its edn strings between them

This is not a contract leak — it's a substrate optimization the
protocol surface hides. The honest read:

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

- `Thread<I,O>` fields: `stdin: Sender<Value>`, `stdout:
  Receiver<Value>`, `stderr: Receiver<Value>`, internal
  ProgramHandle.
- `Process<I,O>` fields: `stdin: IOWriter`, `stdout: IOReader`,
  `stderr: IOReader`, internal ProgramHandle.

The supertype satisfaction rule isn't "same field types" — it's
"same operations supported." The polymorphic verbs
(`process-send`, `process-recv`, `Program/join-result`) work on
either; the user-visible accessors (`Thread/stdin`,
`Process/stdin`) might return different concrete types but the
typed comm verbs work uniformly.

This makes Program<I,O> a **typeclass / protocol** in the formal
sense — it's defined by the operations it supports, not by its
internal field shape. The substrate's first such abstraction.

### Direct accessor escape hatch

For programs that want to bypass the protocol (e.g., wat-cli's
stdio proxy moves bytes; a direct-Sender consumer wants the raw
crossbeam reference), the typed accessors stay available:

- `:wat::kernel::Thread/stdout` returns `:Receiver<O>`.
- `:wat::kernel::Process/stdout` returns `:wat::io::IOReader`.

Caller chooses: protocol-level (`process-recv` works on either)
or implementation-level (`Thread/stdout` for the crossbeam
receiver; `Process/stdout` for the pipe). The protocol is the
ergonomic default; the raw accessors are the escape hatch.

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
  (body :Fn(:wat::io::IOReader,
            :wat::io::IOWriter,
            :wat::io::IOWriter) -> :wat::core::unit)
  -> :wat::kernel::Thread<I, O>
```

The spawn body has the same signature shape as `:user::main` of
a wat program: takes stdin / stdout / stderr; returns unit.
The Thread<I, O> exposes:

- `:wat::kernel::Thread/stdin`  — `IOWriter` (parent → thread)
- `:wat::kernel::Thread/stdout` — `IOReader` (thread → parent, typed `O`)
- `:wat::kernel::Thread/stderr` — `IOReader` (thread → parent, errors)
- `:wat::kernel::Thread/join-result` →
  `:Result<:wat::core::unit, :wat::kernel::ThreadDiedError>`

Symmetric with arc 112's `Process<I, O>`. Both satisfy
`:wat::kernel::Program<I, O>` (arc 109 § J's supertype). The
polymorphic `:wat::kernel::join-result` from arc 109 § J slice
10d works on either.

## Migration of existing `:wat::kernel::spawn` consumers

Today's bare-spawn callers fall into three categories:

1. **Background workers** that compute X and send it on a
   channel the caller already holds. Migration is mostly
   cosmetic — they were already streaming out; their ProgramHandle
   was just being used as a "did the worker finish?" signal.
   `(:wat::kernel::spawn (fn () (some-work-and-send! tx)))` →
   `(:wat::kernel::spawn-thread (fn (stdin stdout stderr)
   (some-work-and-send! tx)))`.

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
| **3** | Retire `:wat::kernel::spawn` + `:wat::kernel::ProgramHandle<R>`. Bare-spawn errors at startup with self-describing redirect to `:wat::kernel::spawn-thread`. |
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

**Obvious?** Yes. Threads and Processes are unified — both run a
program-shaped body with stdin/stdout/stderr and produce no R via
join. One mental model.

**Simple?** Caller-side: simpler (one mechanism for value flow).
Substrate-side: removes a verb (spawn) and a type
(`ProgramHandle<R>`); adds `spawn-thread` + `Thread<I,O>`. Net
substrate complexity ≈ neutral, but the conceptual surface
collapses.

**Honest?** Yes — the asymmetry pre-arc-114 was that bare-spawn
had an R-via-join channel that fork-program could never produce
(OS processes don't have one). Arc 114 acknowledges that
constraint at the abstraction level.

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
