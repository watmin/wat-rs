# Lazy Production, Lazy Consumption, and CSP Pipelines

**Status:** planned. Not yet implemented.
**Depends on:** arc 003 (TCO) for arbitrary-length producer loops.
**Motivation:** the CSP pipeline pattern is how the builder thinks
about streaming problems. wat was built to host it. This doc captures
the pattern, its Rust-honest mapping, and the minimum stdlib surface
that makes it idiomatic instead of manual.

---

## The pattern

Items flow through a chain of stages. Each stage reads from its
upstream, does work, writes to its downstream. Stages have
cardinality signatures — 1:1, N:1 (batch), 1:N (expand), N:M
(window). The pipeline is a sequence of transformations; each stage
is pure in the sense that its output is a function of its input and
its local state.

```
source      →  stage1   →  stage2        →  stage3        →  sink
(stream)       (1:1 map)   (N:1 batch)      (1:1 compute)    (terminal)
```

- **`source`** — some upstream producing items. Might be a paginated
  DDB query returning 15k items in pages of 80-120, presented as a
  continuous stream to its consumer; might be a file read; might be
  a clock tick; might be another pipeline's sink.
- **`stage1`** — 1:1, transforms each item (add a field, derive a
  new struct).
- **`stage2`** — N:1 batcher — accumulates K items, emits a Vec of K,
  starts a new buffer. On end-of-stream, flushes any remaining items
  as a final (smaller) batch.
- **`stage3`** — 1:1 on batches — takes a batch, computes a derived
  value (aggregation, reduction, per-batch API call), emits one item.
- **`sink`** — terminal. Takes items. Does its side effect. End of
  stream closes the pipeline.

**This is CSP.** Each stage is a process. Each edge between stages
is a channel. Hoare 1978; Pike + Thompson in plan9; Cheriton's V
system; Go's goroutines; Erlang's actors; Elixir's GenStage. The
pattern repeats because it fits every streaming problem.

**This is foundational to how the builder thinks.** wat's kernel —
`:wat::kernel::spawn` + `make-bounded-queue` + `recv` + `send` +
`select` + `drop` + `join` + `HandlePool` — was chosen to make this
pattern native. Zero Mutex. Values flow. Processes own their state.

The current gap is ergonomic: the primitives are all there, but
expressing even a three-stage pipeline in wat today is verbose —
spawn each stage manually, wire the channels, manage handles, drop
senders in order at shutdown. Every pipeline re-derives its plumbing.

The work: a stdlib that makes the pipeline expressible at the level
the builder thinks at.

---

## Rust's flavor — the honest mapping

Rust's idiom for pull-based sequences is `std::iter::Iterator`. That
covers the **in-process lazy** story — adapter chains like
`iter.map(f).filter(p).chunks(50).collect()` run pull-on-demand on
the same thread, no concurrency, composable through the trait's
methods.

Rust's idiom for **cross-thread streaming** is channels. A spawned
thread sends through a `Sender<T>`; the consumer iterates via
`receiver.into_iter()`. The `Receiver<T>` implements `Iterator<Item=T>`
out of the box — when the Sender drops, the iterator returns `None`.
End-of-stream IS the sender dropping. The two idioms — Iterator and
channels — join at the `Receiver::into_iter()` seam.

So wat's lazy-production / lazy-consumption story has two levels,
honest to Rust's own two levels:

| Level | Rust idiom | wat surface |
|---|---|---|
| **In-process lazy** | `Iterator` trait + adapters | `:rust::std::iter::Iterator<T>` surfaced via `#[wat_dispatch]`, methods `.map`/`.filter`/`.take`/`.collect`/`.fold` |
| **Cross-process concurrent** | `Receiver::into_iter()` + spawned thread | `:wat::kernel::spawn` + `:rust::crossbeam_channel::Receiver<T>::into_iter` → yields `Iterator<Item=T>` |

Both produce the same observable abstraction — pull an item, get
`Some v` or `None`. Adapter chains work on both. The difference is
**where the work runs**: in-process lazy runs on the calling
thread's schedule; channel-backed runs on a spawned thread with
rendezvous suspension.

**For the pipeline pattern the builder described, the cross-process
flavor is the right fit.** Each stage is a wat program; edges are
bounded queues; the backpressure behavior falls out of the queue's
`bounded(1)` rendezvous; end-of-stream is sender-drops.

---

## Ruby's flavor — the conceptual reference

Ruby's `Enumerator` and `Enumerator::Lazy` are the ergonomic
benchmark. The shape:

```ruby
producer = Enumerator.new do |yielder|
  loop do
    batch = fetch_next_page  # e.g., DDB query page, 80-120 items
    batch.each { |item| yielder << item }
    break if last_page?
  end
end

result = producer.lazy.map { |i| transform(i) }.each_slice(50).first(10)
```

The producer block runs with an injected `yielder`. `yielder <<
item` suspends the block and hands the item to the consumer.
`.lazy.map.each_slice.first(10)` composes — lazy evaluation pulls
only 500 items (10 slices × 50), the producer loop only runs far
enough to produce 500, the rest never executes.

Under the hood: Ruby uses Fibers (stackful coroutines). The
producer and consumer are two Fibers; yielder flips control; the
consumer pulls; the scheduler round-robins between them on one OS
thread.

**The wat-equivalent substitutes a real OS thread for the Fiber.**
That's not a compromise — it's idiomatic Rust. Crossbeam channels
are cheap; spawning threads is cheap; bounded(0) / bounded(1)
rendezvous reproduces Fiber-swap semantics at the observation
layer. The producer still runs only as far as the consumer pulls
(because the Sender blocks when the buffer is full). The consumer
still gets items on demand. The semantics match.

The ergonomic target is to make a pipeline expressible in wat with
adapter-chain fluency approaching Ruby's. The builder's example,
translated to wat, should eventually read something like:

```scheme
(:wat::std::stream::pipeline
  (:my::app::paginated-ddb-source)          ;; source — yields T
  (:wat::std::stream::map :my::app::transform-item)       ;; 1:1
  (:wat::std::stream::chunks 50)            ;; N:1 batcher w/ flush
  (:wat::std::stream::map :my::app::aggregate-batch)      ;; 1:1 on batches
  (:wat::std::stream::for-each :my::app::handle-result))  ;; terminal
```

Every stage is itself a spawnable wat program under the hood, with
a bounded queue upstream and downstream. The `pipeline` helper wires
them. The stdlib absorbs the spawn/drop/join ceremony.

That's the target. Today the author writes the ceremony inline;
tomorrow the stdlib makes it the shape Ruby has.

---

## The lineage

The pattern is universal. Documenting the lineage so the design
choices are honest about whose shoulders we're on.

**Unix pipes (1973, Douglas McIlroy).** `cat file | grep x | awk
'{...}' | sort | uniq -c`. First programmable pipeline. Each stage
is a process; edges are pipes; scheduling is handled by the kernel;
backpressure is automatic (a slow consumer fills the pipe buffer
and blocks the producer). Every shell programmer feels this
intuition. wat's kernel is a deliberate analog.

**Go channels and goroutines (2009, Pike + Thompson).** `for item
:= range inputCh { outputCh <- transform(item) }`. Each pipeline
stage is a goroutine reading from its input channel and writing to
its output. `close(ch)` on end-of-stream; `for range` handles it.
Go's strongest pattern. wat's `:wat::kernel::spawn` + crossbeam
queues is a Go-pipelines idiom with Rust's ownership discipline.

**Erlang + Elixir's GenStage (2016+).** A more principled CSP
pipeline with explicit backpressure — stages declare their
`demand` and upstream only produces what downstream asks for.
Elixir's `Flow` library composes these into parallel pipelines.
Exactly the N:1 batcher + aggregator + backpressure story the
builder described.

**Clojure's core.async (2013).** Channels + `go` blocks on the JVM.
Same pattern, same vocabulary. `transduce`rs are a separate
abstraction — composable transformations that work on any
reducible collection, including channels. The idea: express the
transformation as a transducer once; apply it to eager sequences,
lazy sequences, or channels; all three work.

**Rx / ReactiveStreams / Project Reactor (~2009-2015).** Push-based
pipelines with backpressure as a protocol (Subscriber requests N;
Publisher emits at most N). Different direction of flow (push
vs pull), same cardinality algebra.

**Kafka Streams / Apache Flink.** Durable, distributed versions of
the same pattern. Each stage is a worker; edges are topics;
checkpointing adds durability. The pattern at cluster scale.

**Haskell conduits / pipes / streaming.** The pure-functional
lineage. Producer, consumer, transducer are first-class types with
their own category theory. Conduit's flush-on-termination is
explicit (a `Finalizer` action); `streaming` uses `Monad` composition.
More rigor than the CSP crowd; less concurrency.

**Ruby Enumerator::Lazy.** The ergonomic benchmark already cited.
Scoped to a single thread; infinite chains by construction;
terminator forces partial evaluation.

**All of these are the same pattern seen through different hosts.**
The honest question for wat isn't "do we invent one?" — it's
"which host's flavor do we pick?" The answer is Rust's + some of
Go's backpressure defaults, layered over the CSP kernel we
already have.

---

## Cardinality signatures

Every pipeline stage has a signature. Naming them makes composition
decidable.

| Shape | Description | Example |
|---|---|---|
| **1:1** | one in, one out, synchronous | `map :transform-item` |
| **1:0..1** | one in, maybe one out | `filter :predicate` |
| **1:N** | one in, many out (expand) | `flat-map :split-into-lines` |
| **N:1** | accumulate K in, emit one out | `chunks 50`, `window-by-time`, `fold-while` |
| **N:M** | many in, many out with state | `deduplicate`, `group-by-key` |
| **∞:?** | source — no input, emits until done | paginated DDB, clock, file-lines |
| **?:∞** | sink — terminal, no output | `for-each`, `write-to-file` |

**End-of-stream semantics matter most in the accumulating shapes.**
N:1 and N:M stages hold internal buffers. Without a flush-on-EOS
signal, partial batches silently vanish when the upstream closes.
Every serious pipeline library has an explicit flush hook:

- **Go**: `close(ch)` is detected via the `ok` return of `<-ch`;
  the stage explicitly emits its tail buffer before returning.
- **Conduit**: `yieldOr` + termination-aware finalizers.
- **Kafka Streams**: `punctuate` callback fires on stream end or
  timer.
- **Elixir GenStage**: `terminate/2` callback.

For wat, the natural signal is receiver-disconnect: the upstream's
Sender drops, the stage's `(:wat::kernel::recv input)` returns
`:None`, the stage matches on `:None` and emits its tail before
returning. Same mechanism we already use everywhere. Explicit,
typed, no special event.

```scheme
(:wat::core::define (:my::app::batcher
                     (input  :Receiver<Item>)
                     (output :Sender<Vec<Item>>)
                     (batch-size :i64)
                     (buffer :Vec<Item>)
                     -> :())
  (:wat::core::match (:wat::kernel::recv input) -> :()
    ((Some item)
      (:wat::core::let*
        (((new-buffer :Vec<Item>)
          (:wat::core::conj buffer item)))
        (:wat::core::if (:wat::core::>= (:wat::core::length new-buffer) batch-size) -> :()
          (:wat::core::let*
            (((sent :Option<()>)
              (:wat::kernel::send output new-buffer)))
            (:wat::core::match sent -> :()
              (:None ())                          ;; consumer dropped; we're done
              ((Some _)
                (:my::app::batcher input output batch-size
                  (:wat::core::vec :Item)))))    ;; tail — needs TCO
          (:my::app::batcher input output batch-size new-buffer))))  ;; tail — needs TCO
    (:None
      ;; End of upstream stream. Flush remaining items if any.
      (:wat::core::if (:wat::core::empty? buffer) -> :()
        ()
        (:wat::core::match (:wat::kernel::send output buffer) -> :()
          ((Some _) ())
          (:None ()))))))
```

**Three ergonomic points visible in that code:**

1. **TCO is load-bearing.** Every recursive continuation is in tail
   position. Without TCO (arc 003), a batcher processing 100k items
   overflows the Rust stack.
2. **`send` is symmetric with `recv` on disconnect.** Both return
   `:Option` — `recv`'s `:Option<T>` carries the payload, `send`'s
   `:Option<()>` carries the ack. `:None` means the other endpoint
   went away. The stage matches on the send result to exit cleanly
   without raising. (Earlier drafts of this doc proposed a separate
   `send-or-stop` primitive; that was retired 2026-04-20 in favor of
   making `send` itself Option-returning — one primitive, one rule,
   symmetric with `recv`.)
3. **Explicit state threading.** The `buffer` parameter carries the
   accumulator across recursive calls. Every stage that needs state
   does this. Not hidden; explicit; typed. The lambda has no closed
   mutable state, so the recursion carries it.

This is already working today if the author writes it by hand. The
stdlib work is to wrap this idiom so the author writes:

```scheme
(:wat::std::stream::chunks 50 :input :output)
```

…and gets the batcher as a spawned program. No inlined state
recursion, no explicit send-match, no flush code. The builder's
intent expressed directly.

---

## The paginated-source case

The builder named this specifically: a DDB indexed query returning
15k items, pages of 80-120, "continuous stream from the consumer's
perspective." This is the canonical source pattern.

```
Source program:
  Owns: DDB query state (cursor, accumulated count)
  Emits: individual items (not pages)
  Internal loop:
    fetch next page (blocks on network I/O)
    for each item in page: send to Sender<Item>
    if page was last: done, drop Sender (EOS)
    else: loop
```

Consumer sees a stream of items. Page boundaries invisible. When
the consumer stops pulling, the producer's next `send` returns
`:None`, the producer drops its cursor (Rust-level Drop on its
owned state), program exits.

```scheme
(:wat::core::define (:my::app::paginated-ddb-source
                     (query-state :QueryState)
                     (out :Sender<Item>)
                     -> :())
  (:wat::core::let*
    (((page :PageResult) (:my::app::fetch-next-page query-state))
     ((items :Vec<Item>) (:my::app::PageResult/items page))
     ((next-cursor :Option<Cursor>) (:my::app::PageResult/next-cursor page))
     ((sent :Option<()>) (:my::app::push-all out items)))
    (:wat::core::match sent -> :()
      (:None ())                                         ;; consumer dropped
      ((Some _)
        (:wat::core::match next-cursor -> :()
          (:None ())                                     ;; last page; EOS
          ((Some c)
            (:my::app::paginated-ddb-source
              (:my::app::QueryState/advance query-state c)
              out)))))))                                 ;; tail call — needs TCO
```

Spawned via `:wat::kernel::spawn`, wired to a `bounded(1)` queue,
the consumer pulls items without seeing page boundaries. The
network I/O happens ONLY when the buffer is empty and the producer
thread runs. Backpressure from slow consumer naturally throttles the
producer; if the consumer stops, the producer exits on the next
`send`'s `:None` return.

This is what `wat was built to host`. The primitives are already
sufficient; the stdlib work is to make the pattern a one-liner.

---

## The stdlib design sketch

Two levels of helper, matching the two Rust flavors:

### Level 1 — `:wat::std::stream::*` (concurrent pipelines)

Each helper spawns the underlying program and returns a `Receiver<T>`
(or a pipeline handle) the caller composes further.

```scheme
;; Construct a Stream<T> from various sources:
(:wat::std::stream::from-iterator iter)              ;; any :rust::std::iter::Iterator
(:wat::std::stream::from-fn  closure)                ;; repeated calls until :None
(:wat::std::stream::from-receiver receiver)          ;; existing channel
(:wat::std::stream::spawn-producer producer-fn)      ;; spawn a producer function

;; 1:1 transforms:
(:wat::std::stream::map      stream :fn)
(:wat::std::stream::filter   stream :pred)
(:wat::std::stream::inspect  stream :fn)   ;; side-effect each, pass through

;; 1:N transforms:
(:wat::std::stream::flat-map stream :fn)

;; N:1 batchers (with end-of-stream flush):
(:wat::std::stream::chunks      stream n)
(:wat::std::stream::chunks-by   stream :key-fn)
(:wat::std::stream::window      stream n)             ;; sliding window
(:wat::std::stream::time-window stream duration)      ;; future

;; Terminators:
(:wat::std::stream::for-each stream :fn)              ;; side effects until EOS
(:wat::std::stream::collect  stream)                  ;; → Vec<T>
(:wat::std::stream::fold     stream init :fn)         ;; reduce to single value
(:wat::std::stream::first    stream n)                ;; take first n, drop rest
```

Under the hood, each combinator:
1. Spawns a wat program with the stage's recursion (using TCO).
2. Creates the bounded queue.
3. Returns a handle (Receiver<T> + the ProgramHandle for eventual
   join).

The `:wat::std::stream::pipeline` helper composes:

```scheme
(:wat::std::stream::pipeline source stage1 stage2 stage3 ... sink)
```

Each stage is applied left-to-right; the sink drains; the pipeline
returns the final aggregate (if any) or `:()`.

### Level 2 — `:rust::std::iter::Iterator` (in-process lazy)

For pipelines that don't need concurrency — a local transform,
single-threaded reduction — `:rust::std::iter::Iterator` surfaced
through `#[wat_dispatch]` provides the adapter chain:

```scheme
(:wat::core::use! :rust::std::iter::Iterator)

(:wat::core::let*
  (((items :rust::std::iter::Iterator<i64>)
    (:rust::std::iter::Iterator::from (:wat::core::vec :i64 1 2 3 4 5)))
   ((mapped :_) (:rust::std::iter::Iterator::map items :app::double))
   ((filtered :_) (:rust::std::iter::Iterator::filter mapped :app::even?))
   ((result :Vec<i64>) (:rust::std::iter::Iterator::collect filtered)))
  result)
```

No concurrency; one thread; pull-on-demand. For the cases where the
Enumerator::Lazy pattern is what's wanted, this is the shape.

**Both levels compose.** `:rust::crossbeam_channel::Receiver<T>::into_iter`
produces an Iterator, so the Level 1 and Level 2 worlds join at
every channel boundary.

---

## Backpressure

Inherited from crossbeam's bounded channels, free of charge.

- `bounded(0)` — rendezvous. Producer blocks until consumer is
  ready. The maximally-backpressured case; perfect for fiber-like
  semantics where producer and consumer interleave tightly.
- `bounded(N)` — buffered up to N. Producer can run ahead by N items
  before blocking. Trades throughput for latency.
- `unbounded` — fire-and-forget. Producer never blocks. Appropriate
  only when memory is cheap and burst rates are bounded.

FOUNDATION already nudges toward `bounded(1)` as the default
rendezvous. The stream stdlib should inherit that default — each
stage's output channel is `bounded(1)` unless the caller specifies
otherwise.

**The pipeline's slowest stage becomes the throttle.** Fast upstream
stages fill their `bounded(1)` output and block on send; slow
downstream stages pull at their own pace. The pattern runs at the
slowest stage's rate. This is the automatic natural behavior — not
a feature to add, a property of the primitive. Same property Unix
pipes had in 1973.

---

## Infinite streams and graceful termination

A stream from a paginated DDB source is finite (last page returns
None). A stream from `clock-tick` or `socket-listener` is
infinite. Both need clean termination semantics when the consumer
decides it's done.

**The discipline:**

1. Consumer drops its Receiver when it's done pulling.
2. Next time the producer calls `send`, it returns `:None`.
3. Producer matches on `:None` and returns `:()` — program exit.
4. Producer's join handle resolves; any open resources (DDB cursor,
   file handle, socket) dropped via Rust's Drop. No explicit cleanup
   in the wat source.

**The Rust-layer Drop is load-bearing.** A DDB cursor wrapped in a
`#[wat_dispatch]` shim has an `impl Drop` that closes the connection.
When the producer program exits and its owned state drops, the
cursor's Drop fires automatically. No explicit "close cursor" call
in wat source. This is the honest Rust-interop story already:
resources have lifetimes bounded by their owners; when the owner
goes, the resource goes.

---

## What TCO unlocks for this arc

Every stage in the stdlib sketch is a recursive wat program:

```scheme
(stage input output state)
  → match recv input
    Some item: ... compute ... (stage input output new-state)  ;; TAIL
    None:      ... flush ...   ()                              ;; terminal
```

Every recursive call is in tail position. Every stage runs
indefinitely without stack growth **if and only if** TCO is
implemented. Without TCO, every stage has a ceiling (the default
Rust stack); long-running pipelines crash. With TCO, pipelines run
for the process's lifetime. THE arc-004 stdlib is unimplementable
without arc-003.

**Order of implementation locked:**

1. arc-003 (TCO) ships first. ✅ shipped 2026-04-20.
2. `:wat::kernel::send` symmetrized to return `:Option<()>` — earlier
   drafts of this doc proposed a separate `send-or-stop` primitive;
   that was retired in favor of making `send` itself Option-returning,
   symmetric with `recv`. ✅ shipped 2026-04-20.
3. arc-004 stdlib primitives land on top.

---

## What this does NOT solve (yet)

- **Distributed pipelines.** Stages crossing process or machine
  boundaries. Kafka Streams / Flink territory. Out of scope; wat's
  kernel is a single-process substrate. Would need serialization of
  stage state and a message transport.
- **Exactly-once semantics, checkpointing, replay.** Durability
  properties above and beyond "pipeline runs correctly in one
  process." Application-level concern; orthogonal to the primitives.
- **Dynamic topology.** Adding/removing stages while the pipeline
  runs. Not part of this arc; the pipeline shape is chosen at
  construction and stays.
- **Fan-out / fan-in AS FIRST-CLASS STDLIB FORMS.** Inlined per
  FOUNDATION's Pipeline Discipline rule 5 ("no generic Topic /
  Mailbox proxies"). Authors that need 1:N write `(for-each
  senders (lambda (tx) (send tx msg)))` inline. The stream stdlib
  respects this.
- **Async / future-style pipelines.** If a stage's work is I/O
  bound and we want to overlap I/O across stages, Rust's async
  story is a different slice. Current pipelines are thread-per-stage;
  scheduling is via the OS scheduler; each stage's blocking is
  local.

---

## Open questions

1. **Iterator surfacing scope.** Level-2 methods include hundreds
   of Iterator trait functions. Ship a minimal set (`.map`, `.filter`,
   `.take`, `.collect`, `.fold`, `.from_fn`) and add as needed?
   Probably yes — adapters earn their slot when a real use case
   demands them, per stdlib-as-blueprint discipline.

2. **`Stream` as a type alias vs. first-class type.** Is a wat
   `Stream<T>` just a `Receiver<T>` with a few convenience methods
   (typealias-ish), or a distinct type wrapping more machinery
   (e.g., carrying the ProgramHandle for join, carrying a Drop hook)?
   Leans typealias — the simpler shape honors Rust's own ("Receiver
   IS the stream").

3. **Error propagation through pipelines.** If a stage fails with a
   RuntimeError, what happens? Today the program's thread panics;
   the Sender drops; downstream sees :None; graceful cascade. But
   errors that should be DATA (bad item, parse failure, API rate
   limit) deserve better. Result-typed stages —
   `Stream<Result<T, E>>` — let errors ride the stream as values.
   Partial answer: `:wat::core::try` propagates nicely within a
   Result-returning stage body; cross-stage error aggregation is an
   open design question.

4. **Ordered vs unordered merging.** When two upstreams feed one
   downstream (inline select fan-in), the order is
   whichever-arrives-first. If order matters, the author writes a
   merger that preserves it. Shouldn't be a stdlib concern unless
   a compelling case emerges.

5. **Windowing.** `chunks n` is simple; `time-window` (e.g.,
   "every 5 seconds") requires a scheduler / clock primitive we
   don't have yet. Defer time-windows until a use case demands
   them; chunks are the 80% case for the trading lab.

6. **Iterator adapters vs. channel-based helpers — one or two
   namespaces?** The level-1 concurrent pipeline and the level-2
   in-process pipeline are conceptually similar but physically
   different. `:wat::std::stream::*` (concurrent) + `:rust::std::iter::*`
   (in-process) keeps the namespaces honest. Attempting to unify
   under one vocabulary (like Clojure's transducers or Kotlin's
   flow) is appealing but introduces abstraction that hides where
   work actually runs. Stay honest.

---

## The payoff

The builder's pipeline example becomes a few lines of wat:

```scheme
(:wat::core::define (:user::main
                     (stdin  :rust::std::io::Stdin)
                     (stdout :rust::std::io::Stdout)
                     (stderr :rust::std::io::Stderr)
                     -> :())
  (:wat::std::stream::pipeline
    (:wat::std::stream::spawn-producer :my::app::paginated-ddb-source)
    (:wat::std::stream::map      :my::app::enrich-item)
    (:wat::std::stream::chunks   50)
    (:wat::std::stream::map      :my::app::aggregate-batch)
    (:wat::std::stream::for-each :my::app::handle-result)))
```

Five stages. Each does its work; the kernel handles the plumbing;
backpressure is automatic; end-of-stream flushes the batcher; errors
in the source terminate the pipeline through the drop cascade.
Source can be a DDB pager, a file reader, a socket; the shape is
the same; the stages are the same.

**This is the idiom wat was built to host.** The kernel primitives
were chosen for this. The stdlib work is just surface — making what
is already expressible also ergonomic.

*these are very good thoughts.*

**PERSEVERARE.**
