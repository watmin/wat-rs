# Zero Mutex — wat-rs's Concurrency Architecture

The wat runs on dozens of OS threads. It serializes writes to
stdout across every program that wants to print. It owns LRU caches
that clients across the process hit concurrently. It composes
streaming pipelines where every stage is its own thread doing real
parallel work across cores.

And it has zero Mutex.

Not fewer. Not mostly. **Zero.** Every piece of state that multiple
actors appear to contend for is architected so the contention
doesn't exist. The Mutex is not avoided; the *situation that would
need one* is never constructed.

This document states the architecture, names the three mechanisms
that replace Mutex across the substrate's operating range, and
walks through the concrete cases — the ones that would demand a
Mutex in a conventional Rust codebase — to show where each one
goes in the wat-rs world.

---

## The claim

A Mutex is a patch over a bad architectural decision: *put shared
mutable state in one address space, then point multiple threads at
it.* The Mutex is the scar tissue on the bad situation. Most
codebases accept the scar because they accept the situation.

**wat-rs rejects the situation.** Every piece of state lives in
one of three shapes:

1. **Immutable** — an `Arc<T>` frozen at startup. Many readers;
   zero writers; no lock needed by construction. wat-rs's
   `FrozenWorld` is the canonical case — after `freeze()` nothing
   mutates; every thread reads the same `&SymbolTable`, `&TypeEnv`,
   `&MacroRegistry`, `&Config` at will, forever.

2. **Thread-owned, runtime-checked** — a `ThreadOwnedCell<T>` that
   holds state for exactly one thread and refuses access from any
   other. No lock; a `ThreadId` guard fires an error on misuse.
   For state that mutates fast on a hot path and only one thread
   has a reason to touch it.

3. **Program-owned, message-addressed** — state owned by a
   spawned wat program, accessed by clients via bounded channels.
   The program's single-threaded loop serializes every access
   without locking because it is *structurally* sequential. Its
   body IS the serialization. For state multiple threads want to
   mutate and read.

These three tiers cover every situation a working codebase
encounters. Combined with Rust's borrow checker — which makes
crossing a tier boundary without permission a compile error — the
architecture is not just Mutex-free in practice, it is Mutex-free
*enforceably*.

The pipeline stdlib (arc 004) and TCO (arc 003) make this
architecture ergonomic: driver loops run indefinitely without
stack growth; pipeline stages compose with adapter fluency.
Together they close the loop. The Mutex-free pattern, from the
builder's Ruby services forward through this wat implementation,
stops being a discipline and becomes the path of least resistance.

---

## Tier 1 — Immutable shared state

The simplest tier. State is computed once at startup, wrapped in an
`Arc<T>`, and shared across every thread that needs to read it.
Rust's type system guarantees `&T` cannot mutate; many readers are
safe; zero locks are needed.

The substrate examples:

- **`FrozenWorld`** — the result of the 12-step startup pipeline.
  After `freeze()`, the symbol table, type environment, macro
  registry, and config are bundled into an immutable value.
  Every thread holds an `Arc<FrozenWorld>` or `&FrozenWorld`. No
  mutation possible; no lock needed.
- **`EncodingCtx`** — `EncoderRegistry` (per-dim VM/Scalar pair) +
  `Config`, populated at freeze, referenced by every thread that
  encodes or presence-measures. The encoder registry's internal
  caches use lock-free structures (deterministic hashing; entries
  never change); the outer `Arc<EncodingCtx>` is read-only. Per
  arc 057 the `AtomTypeRegistry` field is gone — typed HolonAST
  leaves replaced its dyn-Any dispatch job.
- **Deterministic atom vectors** — per FOUNDATION, the same atom
  at the same seed and dimension always produces the same vector.
  Threads that both need `atom("rsi")` compute independently and
  get byte-identical results. No shared cache needed; no cache
  invalidation to coordinate.

**Concrete test:** read-read concurrency on `FrozenWorld` fields
from N threads returns the same bytes on every read, forever,
regardless of interleaving. This is what `Arc<T>` where `T: Sync`
guarantees. The test is trivial because the architecture is
trivial.

**What gets kept in this tier:** anything the system commits to
once and never changes. Configuration. Parsed ASTs. Registered
function bodies. Type declarations. The deterministic atom seed.

**What doesn't fit:** anything that needs to update as the program
runs. Those go to tier 2 or tier 3.

---

## Tier 2 — Thread-owned, runtime-checked

Some state mutates often on a hot path, but only one thread has a
reason to touch it. A per-thread LRU cache. A parsed compiled
regex with interior state. A database connection that single-
threaded ownership would suffice for if only the handle could
travel as a value.

**`ThreadOwnedCell<T>`** is the tier-2 primitive. A cell that:

1. Stores its payload in `UnsafeCell<T>`.
2. On construction, captures the creating thread's `ThreadId`.
3. On every access (`.with_ref`, `.with_mut`), asserts the current
   thread matches the stored id. If not, returns a clean error —
   `RuntimeError::TypeMismatch` with a clear "this cell is owned
   by thread X, you are thread Y" message.
4. `unsafe impl Send + Sync`, justified by the runtime check.

The cell can travel between threads as a value (Send), and it can
be held by multiple threads as a handle (Sync), but it can only be
*touched* from the thread that owns it. The soundness is
structural: the cell itself proves ownership on every access. No
lock; no fairness logic; no deadlock surface; just "wrong thread"
errors for the misuse case.

**Concrete uses in wat-rs:**

- **`:rust::lru::LruCache<K,V>`** — the wat-level LRU cache is an
  `lru::LruCache<String, Value>` wrapped in a `ThreadOwnedCell`.
  `LruCache::put` takes `&mut self`; without `ThreadOwnedCell`,
  sharing the cache across threads would require a `Mutex`. With
  `ThreadOwnedCell`, the cache lives on one thread; that thread
  puts and gets at hot-path speeds; other threads that try to
  touch it get an error, not a race.
- **`:wat::std::LocalCache<K,V>`** — a wat-level wrapper over the
  LruCache shim. The stdlib form authors who want per-program
  caching use this — no coordination overhead; no lock; no
  wakeups.
- **Scope = `thread_owned`** in `#[wat_dispatch]` — the macro
  pattern for Rust types with `!Sync` interior. Applies to any
  crate type we surface where the interior is not thread-safe by
  default (ThreadLocal random generators, some parser states,
  connection handles that only make sense on one thread).

**When this tier is right:** the state's natural owner is one
thread for its entire lifetime, and the operation cost is low
enough that forwarding messages through a channel would add
unacceptable latency. The hot path of a single wat program that
caches its own intermediate computations.

**When this tier is wrong:** when multiple threads legitimately
need to read OR write the state. That's tier 3's job. A
`ThreadOwnedCell` shared across workers would give a "wrong
thread" error on every access but one — the cell's whole job is to
REFUSE cross-thread access, cleanly.

---

## Tier 3 — Program-owned, message-addressed

The richest tier. State lives inside a spawned wat program; other
threads access it by sending messages through bounded channels to
that program's mailbox. The program's single-threaded body reads
messages, mutates state, writes replies. Serialization is
*structural* — the body is sequential; there is no lock because
there is no parallel access.

This is the Erlang `gen_server` pattern. BEAM processes doing it
since 1986. Go's canonical "share memory by communicating." Unix
pipes filtered through 40 years of OS design. The oldest, most-
proven concurrency pattern that is not a lock.

**The substrate examples:**

- **`:wat::std::service::Console`** — owns the real `io::Stdout`
  and `io::Stderr` handles. Every program that wants to print
  pops a `Console::Handle = (Tx, AckRx)` from a HandlePool and
  uses it through `Console/out` / `Console/err`. The Console
  driver runs a select loop across all client request channels;
  on a select fire it writes to stdout or stderr, then sends `()`
  on the ack-tx paired with that request channel by index. The
  producer's helper blocks on ack-rx until that ack lands. No
  lock on stdout; no garbled output; concurrent writers serialize
  through the driver's single-threaded body; AND each producer
  unblocks only AFTER the bytes are durable. The "Console is the
  lock, except there's no lock" case — see also "Mini-TCP via
  paired channels" below.
- **`:wat::lru::CacheService<K,V>`** — the L2 caching program
  (external workspace member `crates/wat-lru/`; namespace promoted
  to `:wat::*` via arc 036). Owns its own LocalCache internally
  (on the driver's thread, using tier-2 ThreadOwnedCell). Clients
  send tagged requests — `(GET, k, :None)` or `(PUT, k, (Some v))`
  — paired with a reply-to sender; driver reads request, hits or
  misses its LocalCache, sends reply. Any number of client threads
  hit the same cache concurrently; the driver serializes; no lock
  on the cache map.
- **Pipeline stages (arc 004)** — every stage in a streaming
  pipeline owns its local state (accumulator buffer, cursor,
  enrichment cache). Edges between stages are typed channels.
  State never escapes a stage; the only coupling is values
  flowing down the pipe. The substrate scales to dozens of
  concurrent stages; each runs at its own rate; none can corrupt
  another's state because none can reach another's state.

**Infrastructure that makes this tier ergonomic:**

- `:wat::kernel::spawn` — starts a wat function on a new OS
  thread, returns a `ProgramHandle<R>` the caller can `join`.
- `:wat::kernel::make-bounded-queue :T n` — a typed, bounded
  crossbeam channel. `n=1` rendezvous is the FOUNDATION default.
- `:wat::kernel::select` — select across N receivers; returns
  `(index, Option<T>)` where `:None` means the receiver at that
  index disconnected. The fan-in primitive.
- `:wat::kernel::HandlePool<T>` — claim-or-panic discipline for
  distributing N client handles across N consumers. Catches
  orphaned handles at wiring time instead of letting them deadlock
  the driver at shutdown.
- **Arc 003 TCO** — lets driver loops recurse in tail position
  without stack growth. Without it, a Console running for hours
  would eventually overflow. With it, drivers run for the
  process's lifetime in constant stack.
- **Arc 004 stream stdlib** — wraps the spawn-channel-drop-join
  ceremony so authors write pipelines at the level of their
  intent, not at the level of their plumbing.

**When this tier is right:** any state that multiple threads want
to read or write. Caches shared across threads. I/O resources
(stdout, a DB connection pool). Accumulators that bridge pipeline
stages. Any "shared resource" that a non-wat codebase would wrap
in a Mutex.

**When this tier is wrong:** when the round-trip latency of
message-passing is too expensive. This is rare. Bounded(1)
crossbeam channels are extremely fast (nanoseconds); for anything
above hot-inner-loop-of-inner-loop cost, channels are cheaper than
the contention a Mutex would introduce. In the rare case where
it's not, tier 2 (ThreadOwnedCell) or tier 1 (immutable snapshot)
is the fallback.

---

## Mini-TCP via paired channels — the canonical mutex-replacement pattern

The substrate's answer to *"I have a shared resource and N
producers want to touch it without corrupting each other"* lives
inside Tier 3 — but the ergonomic shape is sharp enough to name
on its own. The trader called it **mini-TCP** when it surfaced
during arc 089: producer writes on one pipe, blocks on the
companion pipe until the consumer signals "done." Two pipes per
producer, bounded(1) on each, mutually blocking through the
substrate's existing rendezvous discipline.

```
PRODUCER SCOPE                                  DRIVER THREAD
══════════════                                  ═════════════

req-Tx ──────write──→ req-Rx                       ┐
                                                    │  one of these
ack-Rx ←──read──── ack-Tx                          │  per producer
                                                    ┘

  producer's two ends                              driver's two ends
   = Console::Handle                                = Console::DriverPair
     (Tx, AckRx)                                      (Rx, AckTx)
```

The driver's loop is **`io.select(things-who-want-to-touch-data)`**
— substrate-native via `:wat::kernel::select`. Only one producer's
message can be processed at a time because select picks one and
the driver runs sequentially. Bounded(1) on the request pipe
means a producer can't enqueue another message while the
previous one is in-flight. Bounded(1) on the ack pipe means the
producer's `recv` blocks until the driver explicitly signals
completion. Together they give **organic backoff** — slow
consumer naturally throttles fast producers; fast consumer
unblocks producers immediately; no tuning, no policy, no lock.

### What replaces Mutex

A Mutex codifies *"only one thread at a time"*. The mini-TCP
pattern dissolves that question: there ARE no parallel touches
of the resource. The select loop is sequential by construction;
the producer is paused waiting for the ack; the consumer holds
the resource alone for as long as the work takes; the ack
releases. The "lock" is the loop body itself; the "release" is
the ack send. Both are the substrate's primitives; neither is
a lock.

### Routing acks: pair-by-index vs embedded reply-tx

Two routing strategies, both substrate-supported:

- **Pair-by-index** (`Console`, single-verb services). The
  driver's `Vec<(Rx, AckTx)>` holds request and ack ends paired
  by index. `select` returns the index that fired; the driver
  looks up the matching ack-tx at the same index and sends `()`.
  Ack address is implicit in the channel's identity. The
  cleanest shape when ALL replies are unit and the service has
  one verb. Reference: `wat-rs/wat/std/service/Console.wat`.

- **Embedded reply-tx in payload** (`Service<E,G>`,
  `CacheService<K,V>`, the canonical `service-template.wat`).
  The request payload includes the producer's ack/reply channel
  as a field. The driver reads the request, dispatches per-verb,
  sends the reply on the address embedded in the payload.
  Necessary when reply types differ per verb (`Ack` returns
  unit; `Get` returns the domain state) — the embedded address
  lets the driver pick the right typed channel per request.
  Reference: `wat-rs/wat-tests/std/service-template.wat`.

Both shapes give the same in-memory-TCP discipline. Pick
pair-by-index when the service is single-verb-unit-reply (no
dispatch needed); pick embedded reply-tx when the service has
multiple verbs with heterogeneous reply types.

### Why "the system breathes this way"

A Mutex is held until released; producer goroutines/threads
contend; the OS scheduler arbitrates; throughput depends on the
hardware's atomic-instruction speed and the tradeoffs the
scheduler picks. The mini-TCP pattern has none of that:
producers send when they have something to say; the consumer
serves at its natural rate; bounded(1) makes the request channel
its own backpressure mechanism. The system runs at the speed of
the slowest consumer, applies no fairness logic, and stays
correct under any interleaving the channels allow. **No lock
contention because there are no locks; no thundering herd
because each producer's queue is exactly one slot wide.**

### When this is the right shape

- Any "shared resource with multiple producers" situation a
  conventional Rust codebase would solve with `Mutex<T>` —
  Console (stdout / stderr), DB writers, accumulators that bridge
  pipeline stages, audit logs, registry services.
- Any case where the producer needs to know when the work is
  *done*, not just *queued* — durability boundaries (commit
  acked; bytes written to fd; transaction sealed). Bounded send
  alone gives backpressure on accept; the ack gives backpressure
  on completion. Use both when "done" matters.

### When something else fits better

- **Tier 1 immutable snapshot** when the data doesn't mutate
  after startup. A pipeline that hands out `Arc<Frozen>` references
  doesn't need the round-trip; readers compute against the
  snapshot directly.
- **Pure-dataflow streams** (`wat/std/stream.wat` map / filter /
  reduce) where the channel itself IS the protocol — no separate
  ack needed because each downstream stage's `recv` IS the ack
  for the upstream stage's `send`. Bounded(1) along the pipeline
  gives the same backpressure for free.
- **Fire-and-forget cases** where the producer genuinely doesn't
  care about completion and bounded(1) accept-pressure is enough.
  Rare; default to mini-TCP and downgrade only when measurement
  shows the ack is wasted.

---

## What Rust contributes

The three tiers work in Ruby. The builder has written production
services using this architecture for years, each one
full-concurrency, full-parallelism, zero-Mutex, *on discipline*.
Nothing in Ruby stopped a misbehaving programmer from reaching
outside the pattern and grabbing at shared mutable state through
the back door. The discipline was voluntary; the pattern worked
because the authors were honest.

**Rust promotes the discipline to a guarantee.**

- A `Sender<T>` moved into a `thread::spawn`'d closure is literally
  inaccessible from the parent thread afterward. Not "by
  convention"; the compiler proves it and refuses to compile the
  alternative.
- A `ThreadOwnedCell<T>` holding a `!Sync` type is `Send` only
  because the cell itself proves the thread-ownership invariant
  on every access. The `unsafe impl` is justified by a runtime
  check; the check is airtight because Rust's lifetime system
  prevents escape of the borrow.
- A `FrozenWorld` behind an `Arc` is `Sync` by Rust's rules for
  `Arc<T> where T: Sync`. Readers cannot mutate. The compiler
  enforces it on every `&frozen.symbols()`.
- An orphaned Sender (one that never reaches a consumer) causes a
  `HandlePool::finish` panic at wiring time — in the main thread,
  before any worker starts — naming the resource. The deadlock
  that would have happened at shutdown becomes a panic at
  startup. Detectable; loud; fixable.

**The pattern that was voluntary in Ruby becomes the only thing
that compiles in Rust.** The discipline survives because it has to.
A programmer who tried to reach for `Mutex<T>` in wat-rs would
first have to justify why none of the three tiers fits, which in
practice never happens.

---

## The cases that would reach for Mutex in a conventional codebase

Walking through what a junior Rust programmer, trained on standard
examples, might think they need — and where the situation actually
goes in wat-rs.

**"I have a shared counter across threads."**
→ `AtomicU64` with ordering. No lock. Used in wat-rs for
`KERNEL_STOPPED`, `KERNEL_SIGUSR1/2/HUP`, and any hot counter.

**"I have a shared hash map."**
→ Wrong question. Is it read-only after setup? Tier 1
(`Arc<HashMap<...>>`, immutable). Is it one program's private hash
map? Tier 2 (`ThreadOwnedCell<HashMap<...>>`). Is it shared across
programs? Tier 3 — wrap it in a program with a mailbox
(`:wat::lru::CacheService<K,V>` is the template).

**"I have a shared connection pool."**
→ Tier 3. The pool itself is a program that hands out connections
via a HandlePool. Each connection is owned by whoever has the
handle for the duration of their work; returned to the pool when
dropped. No lock on the pool; the HandlePool mechanism handles
distribution lock-free.

**"I have a complex cache with multiple readers and writers."**
→ Tier 3. `:wat::lru::CacheService<K,V>` or a specialization.
Multiple clients; one program owns the data; requests/replies
through channels.

**"I need to synchronize two pieces of work."**
→ The synchronization IS the channel handoff. `Sender` sends
"done"; `Receiver` receives. No Mutex, no CondVar. `:wat::kernel::join`
on a `ProgramHandle` is this idiom for program-level completion.

**"I have a queue multiple producers push to."**
→ `make-bounded-queue`. Multiple `Sender<T>` clones (wait — wat
doesn't clone Senders; each Sender belongs to one owner per
FOUNDATION's queue discipline). Multiple producer *programs* each
hold their own Sender; each sends independently; the receiver
program drains. No lock.

**"I need to protect an invariant that spans two resources."**
→ The resources belong to one program; the program's sequential
body maintains the invariant. If the two resources belong to
different programs, the invariant is across a protocol between
them, not a transactional locked state — and the protocol is
designed to tolerate any interleaving the channels allow (which is
constrained by their arity and ordering).

**"I want to rate-limit a resource."**
→ A rate-limiter is a program that issues tokens via a
`Sender<Token>` at the chosen rate; callers acquire by receiving.
Canonical `channel-as-semaphore` pattern.

**"I want to let one thread block until another thread signals."**
→ `bounded(0)` rendezvous channel. The signaling thread sends;
the waiting thread receives. Dropping the Sender closes the
"condition variable" — analogous to notify-all.

**"I have a writer-dominated workload with many occasional readers."**
→ Tier 3 — writer program owns the data; readers send queries.
The writer is structurally sequential; readers queue up waiting
for their reply; throughput is limited by the writer's rate. If
that's too slow, the answer is to redesign — maybe partition by
key range into multiple writer programs — not to add a `RwLock`
which only paper-overs the throughput ceiling.

**"I have a read-dominated workload — many readers, rare writers."**
→ Consider whether the writes can be batched into a periodic
snapshot. If yes, tier 1: the snapshot is immutable `Arc<T>`;
readers work against the current snapshot; writers produce new
snapshots periodically; an `ArcSwap<T>` (crate) atomically
replaces the current reference. No lock on reads; writers
serialize naturally through the snapshot-production program. If
no, tier 3.

**In every case** the answer is "which tier fits?" — never "which
Mutex variant do I want?" The question shape has changed. The
architecture routes around the need.

---

## Honest caveats

Three classes of primitive that are NOT quite Mutex but are also
not the three tiers, and which wat-rs legitimately uses:

1. **Atomic primitives** (`AtomicBool`, `AtomicU64`, `AtomicPtr`).
   Lock-free, hardware-supported, wait-free for common ordering
   choices. Used for single-word coordination: the kernel stop
   flag, the user-signal flags, counters. Different mechanism
   from a Mutex; not a violation of the zero-Mutex claim.
2. **`OnceLock<T>` / `Once`**. Lazy one-time initialization. The
   first thread to call `get_or_init` runs the initializer; all
   subsequent threads read the committed value lock-free. Used in
   wat-rs for the rust-deps registry's lazy init. Cheaper than
   locking once at startup because the cost only materializes if
   anyone calls; faster than a Mutex because the hot path is a
   single atomic load.
3. **`Arc<T>`**. Not a lock; an atomic reference count. Readers
   share access; the count is updated atomically on clone / drop.
   Used everywhere immutable shared state travels. The zero-Mutex
   claim doesn't exclude `Arc`.

These primitives are the Rust standard library's correct answers
for the narrow jobs they do. They are NOT Mutex, NOT RwLock, NOT
Condvar, NOT Barrier — the scar-tissue-on-shared-mutable-state
primitives. They are the atomic and reference-counting primitives
the architecture relies on, and their absence from the
"Mutex-replacement tiers" is not an omission; they do a different
job.

---

## The empirical demonstration

The trading lab's wat (production ancestor of this interpreter)
has run with 30+ threads, **zero Mutex**, for months of
development and test runs. The program composes observers,
brokers, treasury, Console, Cache, regime observers, paper-trade
orchestration, ledger — every stage concurrent, every stage in its
own program, every stage communicating through bounded channels.
Zero deadlocks caused by lock ordering. Zero priority inversion.
Zero lost wakeups. Zero torn reads. Zero contention profiles.

When a bug surfaced, it was never a Mutex bug. It was an ordering
bug (shutdown cascade), a capacity bug (Kanerva's limit, now
guarded), or a type mismatch caught by the checker. The class of
failure that Mutex introduces — the one that wakes up at 3am
because a production server hung — simply hasn't occurred, because
the class of situation that needs Mutex hasn't been created.

That's not marketing. It's the outcome of an architecture where
the only thing the compiler permits happens to also be the only
thing that works cleanly. The discipline is the guarantee; the
guarantee is the absence of the failure class.

---

## What this means for anyone adopting wat-rs

A new codebase that starts on top of wat-rs inherits this
architecture by default. The kernel primitives do not include a
lock. The stdlib's program templates do not reach for one.
`#[wat_dispatch]` provides `thread_owned` and `owned_move` scopes,
not `shared_mutex`. The pipeline stdlib composes channel-based
stages whose interiors are tier-2 at most.

An adopter who imports a third-party crate that uses Mutex
internally is free to do so — the crate runs its Mutex inside its
own module; the wat interacts with the crate's public API
through a `#[wat_dispatch]` shim that never sees the Mutex. The
shim's scope (`shared`, `thread_owned`, or `owned_move`) determines
how the crate's type travels through wat. The Mutex inside the
crate is a private implementation detail; wat-rs's zero-Mutex
claim is about *our own* architecture, not about the world's
crates.

**The aspirational position:** when the builder's trading lab and
future domain applications are fully ported onto wat, their
concurrency architecture will be Mutex-free not by accident but by
design, with the compiler as the enforcer. The pattern the builder
has been hand-composing in Ruby for years becomes the path of
least resistance — the ergonomic default, not the discipline.

---

## Why this matters

Concurrency is the feature Rust was built for, and Mutex is the
first answer most Rust tutorials teach. It's a correct answer for
the narrow situation where shared mutable state is already present
and cannot be refactored. It is a terrible default for new code,
because it trains the programmer to think in terms of "what do I
lock?" when the better question is "what owns this?"

wat-rs's three tiers force the better question. Immutable snapshot
at startup (tier 1), single-thread-owned with structural guard
(tier 2), or program-owned with message-addressed access (tier 3)
— these exhaust the situations where state needs to be accessible
from more than one thread. Once the authors are thinking in these
terms, the question of "which Mutex" never arises because the
situation that would demand a Mutex never forms.

The payoff is real and measurable: no deadlocks, no priority
inversion, no lost wakeups, real parallelism up to the number of
cores, and a compile-time proof that the disciplines that fail in
non-trivial codebases are structurally unreachable. The ceremony
— spawn + channel + select + join — looks heavier than a Mutex at
first. It isn't; it's the honest surface area of real concurrency,
and every other concurrent system (Unix pipes, Erlang gen_servers,
Go goroutines, Elixir GenStage) has reached the same conclusion
over decades.

wat-rs joins that lineage in the Rust idiom.

---

*these are very good thoughts.*

**PERSEVERARE.**
