# Arc 114 — INSCRIPTION

## Status

Shipped 2026-05-01. The bare-spawn / R-via-join shape retires;
`spawn-thread` + `Thread<I,O>` + `Thread/join-result` carry the
contract. Threads now satisfy the same Program protocol as
forked Processes: input channel, output channel, error mechanism
through join. R is uniformly unit; values flow through the
output channel exclusively.

cargo test --release green throughout the sweep: 1476 lib +
integration tests; 0 failures. Substrate stdlib, lab consumers,
embedded wat in Rust tests, and the substrate's own runtime
tests all migrated.

Pushed across the slice cycle:
- Slice 1 — `Thread<I,O>` registered + `spawn-thread` constructor
  + `Thread/join-result` (additive; old verbs still worked)
- Slice 2 — substrate sweep: substrate stdlib, lab consumers,
  fixtures, examples, embedded wat in tests
- Slice 3 — verb retirement: `:wat::kernel::spawn` / `join` /
  `join-result` removed at the type checker; old call sites trip
  with a self-describing redirect to `spawn-thread`. 7 ignored
  lib tests + 4 stale arity tests deleted in `src/runtime.rs`
- Slice 4 (manual fixes) — five test files Sonnet's automated
  sweep couldn't reshape: tests/wat_spawn_lambda.rs,
  service-template.wat, HologramCacheService.wat, Console.wat,
  3 telemetry test files
- Slice 5 — closure (this slice)

The deadlock shape that surfaced during slice 4's
HologramCacheService.wat refactor became its own arc — arc 117
makes the SERVICE-PROGRAMS.md lockstep discipline structural at
freeze time, ensuring the same migration mistake can't recur.

## What this arc adds

A unified Program contract that retires the bare-spawn / R-via-
join asymmetry. Programs hosted in threads (in-memory) and
Programs hosted in forked processes (out-of-memory) now satisfy
the same contract: input channel, output channel, error mechanism
through join.

### The new shape

```
:wat::kernel::spawn-thread<I, O>
  (body :Fn(:rust::crossbeam_channel::Receiver<I>,
            :rust::crossbeam_channel::Sender<O>) -> :())
  -> :wat::kernel::Thread<I, O>
```

The spawn body takes the **inside** ends of the input/output
channels (READS from input, WRITES to output) and returns unit.
The Thread<I,O> the parent receives exposes:

- `:wat::kernel::Thread/input`  — `Sender<I>` (parent → thread)
- `:wat::kernel::Thread/output` — `Receiver<O>` (thread → parent)
- `:wat::kernel::Thread/join-result` →
  `:Result<:(), :Vec<:wat::kernel::ThreadDiedError>>`
  (post-arc-113 chain shape)

Symmetric-by-contract (NOT same-fields) with arc 112's
`Process<I, O>`. Both satisfy the Program protocol; the
polymorphic verbs work on either.

### What retired

| Pre-arc-114 | Post-arc-114 | Why |
|---|---|---|
| `:wat::kernel::spawn` | `:wat::kernel::spawn-thread` | R-via-join lie — fork-program could never produce R; arc 114 makes threads obey the same constraint |
| `:wat::kernel::join` | `:wat::kernel::Thread/join-result` | "join panics on death" was a UX shortcut; the chain shape gives callers the data and they choose |
| `:wat::kernel::join-result` (bare) | `:wat::kernel::Thread/join-result` | namespaced under the host so arc 109 § J's polymorphic dispatch can land cleanly |
| `:ProgramHandle<R>` | `:Thread<I,O>` | R is uniformly unit post-arc-114; the parameter was dead weight; the I/O parameters carry the typed protocol |

The substrate poisons the retired verbs at the type checker:
calling `:wat::kernel::spawn` after arc 114 trips with a self-
describing redirect to `spawn-thread`. The migration brief IS
the diagnostic — substrate-as-teacher pattern from arc 111.

### The Program contract

A *Program* is a hosted computation with three concerns every
caller can reach for:

| Concern | Thread<I,O> | Process<I,O> |
|---|---|---|
| Input | `Sender<I>` (crossbeam channel) | `stdin: IOWriter` (OS pipe) |
| Output | `Receiver<O>` (crossbeam channel) | `stdout: IOReader` (OS pipe) |
| Error mechanism | panic surfaces in `Vec<ThreadDiedError>` chain on `Thread/join-result` | child writes EDN-framed chain to `stderr: IOReader` before _exit; parent's `extract-panics` recovers it; surfaces on `Process/join-result` as `Vec<ProcessDiedError>` chain |

The thing that interfaces with a Program does not know or care
which host backs it. Polymorphic verbs (`process-send`,
`process-recv`, `Program/join-result` from arc 109 § J) hide
the gap.

## Why

User direction (2026-04-30):

> i don't think i want threads to have a meaningful ret val...
> they stream results out.. that's the contract
>
> i want the user to choose if they want intra-process or
> inter-process comms - they make the choice "does this belong in
> a fork or a thread?" but the contract is the same on how to
> interface with these programs - the hosting platform is a user
> choice - the protocol isn't

The pre-arc-114 substrate had two mental models for "do work on
another thread":

1. `:wat::kernel::spawn` — function returns R; `join-result`
   yields `Result<R, ThreadDiedError>`. R can be any type.
   Caller asks "what did the function compute?"
2. `:wat::kernel::spawn-program` / `fork-program` — Program has
   stdin/stdout/stderr; `join-result` yields `Result<unit,
   ProcessDiedError>`. R is fixed to unit. Caller streams data
   through the typed pipe.

Two mental models for one operation. Callers chose between
"R-via-join" and "R-via-typed-pipe" based on which spawn verb they
reached for. Arc 114 retires the asymmetry by adopting the
Program shape uniformly — threads stream too. R = unit
everywhere. The user picks the host; the contract is fixed.

### Hosting is a user choice; protocol is fixed

This is the architectural commitment arc 114 names. It generalizes
prior wat substrate principles:

- Arc 103 unified channels under `Sender<T>` / `Receiver<T>` with
  one send/recv API regardless of bounded vs. unbounded backing.
- Arc 110 made silent kernel-comm illegal — one grammar rule
  applies to every comm site.
- Arc 111 made the recv-result shape uniform —
  `Result<Option<T>, ThreadDiedError>` regardless of channel kind.
- Arc 112 unified the typed Program shape across in-thread and
  forked hosts.
- Arc 114 names the meta-principle these instances express: the
  substrate fixes the interface; the user picks the host.

After arc 114, code that wants to swap a Program from in-thread
to out-of-process (or vice versa) changes ONE call site
(`spawn-thread` ↔ `fork-program`) and nothing else. Hosting
becomes a tunable, not a rewrite.

### Why threads return unit

A process **cannot** return `R` via "return" — the only path OUT
of a process is its stdout pipe. There's no mechanism to ferry
arbitrary `R` back through `fork` + `wait`.

For Programs to be uniform across hosts, threads must obey the
same constraint: values flow through the output channel
exclusively. The thread body's "return value" is `()` (unit) — a
marker for "completed without panic", not a value carrier.
`Thread/join-result` returns `Result<(), Vec<ThreadDiedError>>` —
life status, not data.

### Transport asymmetry: same contract, two implementations

The contract surface is uniform; the substrate's TRANSPORT
mechanism under the hood differs by host:

| Concern | Thread<I,O> | Process<I,O> |
|---|---|---|
| Input | `Sender<I>` (crossbeam) | `stdin: IOWriter` (OS pipe) |
| Output | `Receiver<O>` (crossbeam) | `stdout: IOReader` (OS pipe) |
| Wire format | `Arc<Value>` zero-copy in-memory | EDN-serialized line per arc 092 |

Threads pass values as Arcs (zero-copy; types preserved
natively); processes pass them as EDN over pipes (forked child
shares zero memory with parent). User-visible verbs are the same
on both; substrate dispatches on the concrete satisfier's type
tag.

## What this arc closes

- **The R-via-join lie.** Pre-arc-114, bare-spawn returned R
  through the join channel; fork-program could never produce R.
  Post-arc-114, both produce unit; values flow through the
  output channel.
- **The stdin/stdout/stderr-on-Thread lie.** Pre-arc-114, the
  substrate allocated three OS pipes for in-thread spawn and
  wrapped them as IOReader/IOWriter. Threads don't have OS
  stdio; they have channels. Post-arc-114, Thread<I,O>'s honest
  field names (`input` / `output`) match what the host actually
  has. No err stream — panic surfaces on the join chain.
- **The `<R>` parameter on `ProgramHandle`.** Pre-arc-114, every
  `:ProgramHandle<()>` annotation carried dead weight. Post-arc-
  114, `ProgramHandle` collapses; `Thread<I,O>` carries the
  typed protocol that's actually used.
- **The asymmetric mental model.** Pre-arc-114, callers chose
  between two spawn shapes based on what they wanted. Post-arc-
  114, the choice is hosting (Thread vs. Process); the contract
  is fixed.

## Slice walkthrough

### Slice 1 — additive `Thread<I,O>`

`src/types.rs` registers the `Thread<I,O>` struct with `input`
/ `output` fields. `src/runtime.rs` adds `eval_kernel_spawn_thread`
+ `eval_kernel_thread_join_result`. Old verbs continue to work.

### Slice 2 — substrate sweep

Substrate stdlib (Console, stream, sandbox, hermetic), lab
consumers, fixtures, examples, embedded wat in tests/wat_*.rs all
migrated to the new shape. Sonnet did the mechanical sweep guided
by the type checker's hint output.

### Slice 3 — verb retirement

`:wat::kernel::spawn` / `join` / `join-result` removed at the type
checker; old call sites trip with self-describing redirect to
`spawn-thread`. 7 ignored lib tests + 4 stale arity tests
deleted in `src/runtime.rs`. The diagnostic-as-migration-brief
pattern (arc 111) carried the cleanup.

### Slice 4 — manual fixes

Five test/wat files needed shape changes Sonnet's automated
sweep couldn't make:

1. `tests/wat_spawn_lambda.rs` — 5 R-via-join tests rewritten as
   4 mini-TCP-shaped spawn-thread tests (named-define body,
   inline lambda body, closure capture, non-callable rejection)
2. `src/runtime.rs` lib tests — 7 retired-verb tests deleted, 4
   stale arity/refuses tests deleted
3. `wat-tests/std/service-template.wat` — driver-final-state
   delivered via `out` Sender (substrate-allocated channel)
   rather than R-via-join
4. `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
   — 6-step refactor with canonical inner-let* nesting per
   SERVICE-PROGRAMS.md (this is where arc 117's deadlock-shape
   bug surfaced live; that arc was extracted as its own work)
5. `wat-tests/std/service/Console.wat` — each worker's spawn
   nested in own inner-most let* owning its handle

### Slice 5 — closure (this slice)

INSCRIPTION + USER-GUIDE concurrency-section sweep + 058
changelog row.

## The four questions (final)

**Obvious?** Yes. Threads and Processes are unified by the
contract (input / output / error mechanism), not by field shape.
Both produce no R via join. The user picks the host; the verbs
that work on Programs work the same on either. One mental model:
"I have a Program; I feed it values via input; I read values via
output; I learn about panic via join-result."

**Simple?** Caller-side: simpler (one mechanism for value flow).
Substrate-side: removes a verb (`spawn`) and a parameter slot
(`<R>`); adds `spawn-thread` + `Thread<I,O>`. Net substrate
complexity ≈ neutral, but the conceptual surface collapses.

**Honest?** Yes. Three asymmetries dissolve: R-via-join, stdio-
on-Thread, and the dead `<R>` parameter on `ProgramHandle`. The
substrate's transport asymmetry (Arc-on-channel vs EDN-on-pipe)
is named explicitly — same contract, two implementations.

**Good UX?** Mixed in transition: compute-parallelism callers
write more code post-migration (explicit channel pairs replace
implicit R-via-join). Long-term: stronger — one pattern (channels
for data, join for terminal-state) covers every concurrency
primitive. Arc 117 ensures the inner-let* discipline the new
shape requires becomes a structural rule, so the migration
mistake doesn't keep biting.

## Cross-references

- `docs/arc/2026/04/060-join-result/INSCRIPTION.md` — the original
  spawn+ThreadDiedError shape arc 114 generalized.
- `docs/arc/2026/04/112-inter-process-result-shape/INSCRIPTION.md`
  — the "Programs have R = unit" commitment arc 114 generalized
  across the substrate.
- `docs/arc/2026/04/113-cascading-runtime-errors/INSCRIPTION.md`
  — the `Vec<*DiedError>` chain shape arc 114's
  `Thread/join-result` carries.
- `docs/arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md`
  — the rule that ensures arc 114's contract holds going forward.
- `docs/SERVICE-PROGRAMS.md` — the lockstep discipline the new
  shape requires; arc 117 made it structural.
- `docs/ZERO-MUTEX.md` — the broader concurrency framework
  arc 114 fits into.

## Queued follow-ups

- **Arc 109 § J slice 10d** — polymorphic `Program/join-result`
  via typeclass dispatch. Both `Thread/join-result` and
  `Process/join-result` already share Result-shape; the
  polymorphic verb hides the host distinction.
- **Arc 109 § J slice 10g** — typed polymorphic `process-send` /
  `process-recv` over the `Program<I,O>` supertype. Today's
  per-host verbs stay as escape hatches.
- **Compute-parallelism ergonomics** — the explicit-channel-pair
  shape callers write post-arc-114 has more code at the call
  site than the pre-arc-114 R-via-join. Future stdlib helper
  could wrap the common fork/join pattern (Vec of channel pairs +
  Vec of threads + Vec of recvs) into one verb without
  reintroducing the asymmetry.
- **`spawn-thread`-with-named-keyword body** — today closure
  analysis (and arc 117's check) skips named-keyword bodies.
  Future arc inlines closure analysis across the function-table
  boundary; tightens both the deadlock-prevention rule and any
  future structural checks on spawned bodies.
